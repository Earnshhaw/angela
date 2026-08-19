use crate::dragndrop::{
    DragPayload, DropTarget, cursor_moved, drag_released, hover_target, move_done, press_start,
};
use crate::gui::Message::SearchEverythingUpdate;
use crate::rename::{
    RenameField, rename_done, rename_field_update, rename_go, rename_popup, rename_toggle,
};
use crate::search::{Search, search_everything, search_everything_update};
use crate::settings::{SettingsAction, settings_action, settings_panel, toggle_settings};
use crate::sort::{SortBy, sort_by};
use crate::style::*;
use crate::tabs::{TabOps, tab_bar_view, tab_ops};
use crate::{DEBUG_MODE, fs_handling::*};
use iced::keyboard::key::{Code, Named, Physical};
use iced::widget::pane_grid::{self, PaneGrid, ResizeEvent};
use iced::widget::scrollable::Viewport;
use iced::widget::{Space, button, center, mouse_area, opaque, operation, stack, text};
use iced::{Color, Point, Theme, keyboard};
use iced::{
    Element, Length, Task,
    widget::{column, container, row, scrollable, text_input},
};
use iced_aw::ContextMenu;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct State {
    pub panes: pane_grid::State<PaneKind>,
    pub tabs: Vec<Tab>,
    pub current_tab: usize,
    pub home_dir: PathBuf,
    pub search_method: Search,
    pub theme: Theme,
    pub max_results: usize,
    pub dragging: Option<DragPayload>,
    pub hovered_target: Option<DropTarget>,
    pub hovered_row: Option<usize>,
    pub hovered_shortcut: Option<usize>,
    pub overlay: Overlay,
    pub rename_field: RenameField,
    pub shortcut_dirs: [(PathBuf, String); 7],
    pub scroll_offset: f32,
    pub viewport_height: f32,
}

#[derive(Debug, Clone)]
pub enum Overlay {
    Settings,
    Rename,
    None,
}

#[derive(Debug, Clone, Default)]
pub struct LoadedEntries {
    pub root: DirInfo,
    pub entries: Vec<DirInfo>,
    pub expanded: bool,
}

impl LoadedEntries {
    pub fn new(root: DirInfo, entries: Vec<DirInfo>) -> Self {
        Self {
            root,
            entries,
            expanded: false,
        }
    }
    pub fn expand(&mut self) {
        let previous = DirInfo {
            path: self.root.clone().path.parent().unwrap().to_owned(),

            name: "📁 ..".to_string(),
            is_dir: true,
            ..Default::default()
        };

        self.entries.push(previous);
        self.expanded = true;
    }
    pub fn collapse(&mut self) {
        self.entries.pop();
        self.expanded = false;
    }
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub loaded_entries: LoadedEntries,
    pub search_field: String,
    pub search_results_displayed: bool,
    pub sorted_by: SortBy,
}

#[derive(Debug, Clone)]
pub enum PaneKind {
    Sidebar,
    FileView,
}

#[derive(Debug, Clone)]
pub enum GoToMethod {
    Index(usize),
    Path(PathBuf),
    Reload,
    Shortcut(usize),
}

#[derive(Debug, Clone)]
pub enum Message {
    PaneResized(ResizeEvent),
    GoToDir(GoToMethod),
    GoBack,
    UpdatePath(String),
    OpenFile(GoToMethod),
    UpdateContent(Result<LoadedEntries, CError>),
    TabOp(TabOps),
    ContextMenuAction(usize, EntryAction),
    DeleteDone,
    SearchEverythingUpdate(String),
    SearchEverything,
    ToggleSettings,
    SettingsAction(SettingsAction),
    SortBy(SortBy),
    //////////////
    CursorMoved(Point),
    PressStart(usize, bool),
    DragReleased,
    HoverTarget(Option<DropTarget>),
    MoveDone(Result<(), CError>),
    RowHoverStart(usize, bool, Area),
    RowHoverEnd(usize, Area),
    RenameToggle,
    RenameFieldUpdate(String),
    RenameGo,
    RenameDone(Result<(), CError>),
    OpenInTerminalRoot,
    RfdOpen,
    Scrolled(Viewport),
    KeyPressed {
        key: keyboard::Key,
        physical_key: Physical,
        modifiers: keyboard::Modifiers,
    },
}

pub const MAX_RESULTS_DEFAULT: usize = 100;

#[derive(Debug, Clone)]
pub enum EntryAction {
    Open,
    Delete,
    CopyAbsolutePath,
    OpenFileLocation,
    Rename,
    OpenInTerminal,
}

#[derive(Debug, Clone)]
pub enum Area {
    Shortcut,
    RowEntries,
}

impl State {
    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.current_tab]
    }
    pub fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab]
    }
    fn is_pressed(&self, entry_index: usize, area: Area) -> bool {
        match area {
            Area::RowEntries => self
                .dragging
                .as_ref()
                .is_some_and(|d| d.source_tab == self.current_tab && d.source_path == entry_index),
            Area::Shortcut => false,
        }
    }
    fn is_hovered(&self, entry_index: usize, area: Area) -> bool {
        match area {
            Area::Shortcut => self.hovered_shortcut == Some(entry_index),
            Area::RowEntries => self.hovered_row == Some(entry_index),
        }
    }
}

impl Tab {
    pub fn root_entry(&self) -> &DirInfo {
        &self.loaded_entries.root
    }
    pub fn root_entry_mut(&mut self) -> &mut DirInfo {
        &mut self.loaded_entries.root
    }
    pub fn entries_mut(&mut self) -> &mut [DirInfo] {
        &mut self.loaded_entries.entries
    }
    pub fn entries(&self) -> &[DirInfo] {
        &self.loaded_entries.entries
    }
    pub fn expanded(&self) -> bool {
        self.loaded_entries.expanded
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let pane = PaneGrid::new(&state.panes, |_id, pane, _is_maximized| {
        let content = match pane {
            PaneKind::Sidebar => sidebar_view(state),
            PaneKind::FileView => file_view(state),
        };
        pane_grid::Content::new(content)
    })
    .on_resize(10, Message::PaneResized)
    .spacing(4)
    .style(pane_grid_style);

    match state.overlay {
        Overlay::Settings => modal(pane, settings_panel(state), Message::ToggleSettings),
        Overlay::Rename => modal(pane, rename_popup(state), Message::RenameToggle),
        Overlay::None => pane.into(),
    }
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::TabOp(ops) => tab_ops(state, ops),
        Message::GoToDir(method) => go_to_dir(state, method),
        Message::UpdateContent(res) => update_content(state, res),
        Message::GoBack => go_back(state),
        Message::UpdatePath(path) => update_path(state, path),
        Message::OpenFile(path) => open_file(state, path),
        Message::PaneResized(resize_event) => pane_resized(state, resize_event),
        Message::ContextMenuAction(index, action) => context_menu_action(state, index, action),
        Message::DeleteDone => Task::done(Message::GoToDir(GoToMethod::Reload)),
        SearchEverythingUpdate(inp) => search_everything_update(state, inp),
        Message::SearchEverything => search_everything(state),
        Message::ToggleSettings => toggle_settings(state),
        Message::SettingsAction(action) => settings_action(state, action),
        Message::SortBy(sortmethod) => sort_by(state, sortmethod),
        Message::CursorMoved(pos) => cursor_moved(pos, state),
        Message::PressStart(index, is_dir) => press_start(index, is_dir, state),
        Message::DragReleased => drag_released(state),
        Message::HoverTarget(target) => hover_target(target, state),
        Message::MoveDone(result) => move_done(result, state),
        Message::RowHoverStart(index, is_dir, area) => row_hover_start(state, index, is_dir, area),
        Message::RowHoverEnd(index, area) => row_hover_end(state, index, area),
        Message::RenameToggle => rename_toggle(state),
        Message::RenameFieldUpdate(field) => rename_field_update(state, field),
        Message::RenameGo => rename_go(state),
        Message::RenameDone(res) => rename_done(state, res),
        Message::OpenInTerminalRoot => open_in_terminal_root(state),
        Message::RfdOpen => rfd_open(state),
        Message::Scrolled(view) => scrolled(state, view),
        Message::KeyPressed {
            key,
            physical_key,
            modifiers,
        } => key_pressed(state, key, physical_key, modifiers),
    }
}

fn sidebar_view<'a>(state: &'a State) -> Element<'a, Message> {
    let search_everything = text_input("🔍 Search...", &state.current_tab().search_field)
        .style(text_input_style)
        .on_input(Message::SearchEverythingUpdate)
        .on_submit(Message::SearchEverything);

    let shortcut_buttons = state.shortcut_dirs.iter().enumerate().fold(
        column![search_everything],
        |col, (index, (_, merged_name))| {
            let is_hovered = state.is_hovered(index, Area::Shortcut);
            let is_drop_target =
                matches!(state.hovered_target, Some(DropTarget::Shortcut(i)) if i == index)
                    && state.dragging.is_some();
            col.push(
                mouse_area(
                    container(text(merged_name).size(18)).style(shortcut_style(is_hovered, is_drop_target)).width(Length::Fill),
                )
                .on_enter(Message::RowHoverStart(index, true, Area::Shortcut))
                .on_exit(Message::RowHoverEnd(index, Area::Shortcut))
                .on_press(Message::GoToDir(GoToMethod::Shortcut(index))),
            )
        },
    );

    let settings_button = button("⚙")
        .on_press(Message::ToggleSettings)
        .style(secondary_button);
    container(column![
        shortcut_buttons,
        Space::new().height(Length::Fill),
        settings_button
    ])
    .style(sidebar_container)
    .into()
}

fn file_view(state: &State) -> Element<'_, Message> {
    let entries = state.current_tab().entries();

    let (start, end, top_spacer, bottom_spacer) = get_bounds(state);

    let list_of_entries = scrollable(
        entries
            .iter()
            .enumerate()
            .skip(start)
            .take(end - start)
            .fold(column![top_spacer], |column, (index, entry)| {
                let entry_row = row![
                    text(&entry.name).width(Length::FillPortion(4)),
                    text(&entry.modified).width(Length::FillPortion(2)),
                    text(&entry.size.0).width(Length::FillPortion(1))
                ]
                .spacing(50)
                .align_y(iced::Alignment::Center);

                let is_pressed = state.is_pressed(index, Area::RowEntries);
                let is_hovered = state.is_hovered(index, Area::RowEntries);
                let entry_button =
                    mouse_area(container(entry_row).style(row_style(is_hovered, is_pressed)))
                        .on_press(Message::PressStart(index, entry.is_dir))
                        .on_enter(Message::RowHoverStart(
                            index,
                            entry.is_dir,
                            Area::RowEntries,
                        ))
                        .on_exit(Message::RowHoverEnd(index, Area::RowEntries));

                let entry_with_menu = inst_context_menu(index, entry_button, entry.is_dir);

                column.push(entry_with_menu)
            })
            .push(bottom_spacer),
    )
    .on_scroll(Message::Scrolled)
    .style(scrollable_style)
    .width(Length::Fill)
    .height(Length::Fill);

    let path_display = text_input("", &state.current_tab().root_entry().path.to_string_lossy())
        .style(text_input_style)
        .on_input(Message::UpdatePath)
        .on_submit(Message::GoToDir(GoToMethod::Path(
            state.current_tab().root_entry().path.clone(),
        )));
    let back_button = button("←")
        .on_press(Message::GoBack)
        .style(secondary_button);
    let sorting_options = row![
        button("Type")
            .on_press(Message::SortBy(SortBy::FileType))
            .style(if state.current_tab().sorted_by == SortBy::FileType {
                sort_button_active
            } else {
                sort_button_inactive
            })
            .width(Length::FillPortion(1)),
        button("Name")
            .on_press(Message::SortBy(SortBy::Name))
            .style(if state.current_tab().sorted_by == SortBy::Name {
                sort_button_active
            } else {
                sort_button_inactive
            })
            .width(Length::FillPortion(6)),
        button("Date")
            .on_press(Message::SortBy(SortBy::Date))
            .style(if state.current_tab().sorted_by == SortBy::Date {
                sort_button_active
            } else {
                sort_button_inactive
            })
            .width(Length::FillPortion(4)),
        button("Size")
            .on_press(Message::SortBy(SortBy::Size))
            .style(if state.current_tab().sorted_by == SortBy::Size {
                sort_button_active
            } else {
                sort_button_inactive
            })
            .width(Length::FillPortion(3))
    ]
    .width(Length::Fill);

    container(
        column![
            tab_bar_view(state),
            row![back_button, path_display],
            sorting_options,
            list_of_entries
        ]
        .spacing(4),
    )
    .into()
}

fn inst_context_menu<'a>(
    entry_index: usize,
    button: iced::widget::MouseArea<'a, Message>,
    is_dir: bool,
) -> Element<'a, Message> {
    let cx = ContextMenu::new(button, move || {
        let mut column = column![
            iced::widget::button("Open")
                .on_press(Message::ContextMenuAction(entry_index, EntryAction::Open))
                .style(secondary_button)
                .width(Length::Fill),
            iced::widget::button("Copy Absolute Path")
                .style(secondary_button)
                .on_press(Message::ContextMenuAction(
                    entry_index,
                    EntryAction::CopyAbsolutePath
                ))
                .width(Length::Fill),
            iced::widget::button("Rename")
                .style(secondary_button)
                .on_press(Message::ContextMenuAction(entry_index, EntryAction::Rename))
                .width(Length::Fill),
            iced::widget::button("Delete")
                .style(danger_button)
                .on_press(Message::ContextMenuAction(entry_index, EntryAction::Delete))
                .width(Length::Fill),
        ]
        .width(Length::Fixed(150.0));
        if !is_dir {
            column = column.push(
                iced::widget::button(text("Open File Location"))
                    .on_press(Message::ContextMenuAction(
                        entry_index,
                        EntryAction::OpenFileLocation,
                    ))
                    .style(secondary_button)
                    .width(Length::Fill),
            )
        }
        if is_dir {
            column = column.push(
                iced::widget::button(text("Open in Terminal"))
                    .on_press(Message::ContextMenuAction(
                        entry_index,
                        EntryAction::OpenInTerminal,
                    ))
                    .style(secondary_button)
                    .width(Length::Fill),
            )
        }

        column.into()
    })
    .style(menu_container);
    cx.into()
}

fn go_to_dir(state: &mut State, method: GoToMethod) -> Task<Message> {
    let now = if DEBUG_MODE {
        Some(Instant::now())
    } else {
        None
    };
    match method {
        GoToMethod::Path(path) => Task::perform(
            async move { get_dir(path, now).await },
            Message::UpdateContent,
        ),
        GoToMethod::Index(index) => {
            let path = state.current_tab().entries()[index].path.clone();
            Task::perform(
                async move { get_dir(path, now).await },
                Message::UpdateContent,
            )
        }
        GoToMethod::Reload => {
            let path = state.current_tab().root_entry().path.clone();

            Task::perform(
                async move { get_dir(path, now).await },
                Message::UpdateContent,
            )
        }
        GoToMethod::Shortcut(index) => {
            let path = state.shortcut_dirs[index].0.clone();
            Task::perform(
                async move { get_dir(path, now).await },
                Message::UpdateContent,
            )
        }
    }
}

fn update_content(state: &mut State, res: Result<LoadedEntries, CError>) -> Task<Message> {
    if let Ok(result) = res {
        state.current_tab_mut().loaded_entries = result;
        state.current_tab_mut().search_field.clear();
        state.current_tab_mut().search_results_displayed = false;
        //let sorted_by = state.current_tab().sorted_by.clone();
        state.current_tab_mut().sorted_by = SortBy::None;
        state.scroll_offset = 0.0;
        //return Task::done(Message::SortBy(sorted_by));
    }
    Task::none()
}

fn go_back(state: &mut State) -> Task<Message> {
    match state.current_tab().search_results_displayed {
        true => {
            state.current_tab_mut().search_results_displayed = false;
            Task::done(Message::GoToDir(GoToMethod::Reload))
        }
        false => {
            if state.current_tab_mut().root_entry_mut().path.pop() {
                Task::done(Message::GoToDir(GoToMethod::Reload))
            } else {
                Task::none()
            }
        }
    }
}

fn update_path(state: &mut State, path: String) -> Task<Message> {
    state.current_tab_mut().root_entry_mut().path = path.into();
    Task::none()
}

fn open_file(state: &mut State, method: GoToMethod) -> Task<Message> {
    match method {
        GoToMethod::Index(index) => {
            open::that(&state.current_tab().entries()[index].path).unwrap();
            Task::none()
        }
        GoToMethod::Path(path) => {
            open::that(&path).unwrap();
            Task::none()
        }
        _ => Task::none(), // Unreachable in practice
    }
}

fn pane_resized(state: &mut State, resize_event: pane_grid::ResizeEvent) -> Task<Message> {
    state.panes.resize(resize_event.split, resize_event.ratio);
    Task::none()
}

fn context_menu_action(state: &mut State, index: usize, action: EntryAction) -> Task<Message> {
    match action {
        EntryAction::Open => {
            if state.current_tab().entries()[index].is_dir {
                Task::done(Message::GoToDir(GoToMethod::Index(index)))
            } else {
                Task::done(Message::OpenFile(GoToMethod::Index(index)))
            }
        }
        EntryAction::Delete => {
            let path = state.current_tab().entries()[index].path.clone();
            Task::future(async move {
                delete_dir(path).await.ok();
                Message::DeleteDone
            })
        }
        EntryAction::CopyAbsolutePath => {
            let path = state.current_tab().entries()[index].path.clone();
            return iced::clipboard::write(path.to_string_lossy().into_owned());
        }
        EntryAction::OpenFileLocation => {
            let mut path = state.current_tab().entries()[index].path.clone();
            Task::done(Message::GoToDir(GoToMethod::Path(if path.pop() {
                path
            } else {
                PathBuf::new()
            })))
        }
        EntryAction::Rename => {
            state.overlay = Overlay::Rename;
            state.rename_field.source = state.current_tab().entries()[index].path.clone();
            state.rename_field.dest = state.current_tab().entries()[index].path.clone();
            let id: &'static str = "rename_input";
            operation::focus(id)
        }
        EntryAction::OpenInTerminal => {
            let path = state.current_tab().entries()[index].path.to_string_lossy();
            std::process::Command::new("wt")
                .args(["-p", "Command Prompt", "-d", &path])
                .spawn()
                .ok();
            Task::none()
        }
    }
}

pub fn theme(state: &State) -> Theme {
    state.theme.clone()
}

fn row_hover_start(state: &mut State, index: usize, is_dir: bool, area: Area) -> Task<Message> {
    match area {
        Area::Shortcut => {
            state.hovered_shortcut = Some(index);
            state.hovered_target = Some(DropTarget::Shortcut(index));
        }
        Area::RowEntries => {
            state.hovered_row = Some(index);
            if is_dir {
                state.hovered_target = Some(DropTarget::FolderRow {
                    tab: state.current_tab,
                    index,
                });
            }
        }
    }
    Task::none()
}

fn row_hover_end(state: &mut State, index: usize, area: Area) -> Task<Message> {
    match area {
        Area::Shortcut => {
            if state.hovered_shortcut == Some(index) {
                state.hovered_shortcut = None;
            }
            if matches!(state.hovered_target, Some(DropTarget::Shortcut(i)) if i == index) {
                state.hovered_target = None;
            }
        }
        Area::RowEntries => {
            if state.hovered_row == Some(index) {
                state.hovered_row = None;
            }
            if matches!(state.hovered_target, Some(DropTarget::FolderRow { index: i, .. }) if i == index)
            {
                state.hovered_target = None;
            }
        }
    }
    Task::none()
}

pub fn modal<'a>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_dismiss: Message,
) -> Element<'a, Message> {
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.6,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_dismiss)
        )
    ]
    .into()
}

fn open_in_terminal_root(state: &mut State) -> Task<Message> {
    let path = state.current_tab().root_entry().path.to_string_lossy();
    std::process::Command::new("wt")
        .args(["-p", "Command Prompt", "-d", &path])
        .spawn()
        .ok();
    Task::none()
}

fn rfd_open(state: &mut State) -> Task<Message> {
    let home = state.home_dir.clone();
    Task::perform(
        async move {
            let path = rfd::AsyncFileDialog::new().pick_folder().await;
            if let Some(path) = path {
                path.path().to_owned()
            } else {
                home
            }
        },
        |path| Message::SettingsAction(SettingsAction::ChangeSearchRoot(path)),
    )
}

pub fn scrolled(state: &mut State, view: Viewport) -> Task<Message> {
    state.scroll_offset = view.absolute_offset().y;
    state.viewport_height = view.bounds().height;
    Task::none()
}

const ROW_HEIGHT: f32 = 24.0;
const BUFFER_ROWS: usize = 5;

pub fn get_bounds(state: &State) -> (usize, usize, Space, Space) {
    let total = state.current_tab().entries().len();

    let first_visible = (state.scroll_offset / ROW_HEIGHT).floor() as usize;
    let visible_rows = if state.viewport_height <= 0.0 {
        50
    } else {
        (state.viewport_height / ROW_HEIGHT).ceil() as usize
    };

    let start = first_visible.saturating_sub(BUFFER_ROWS);
    let end = (first_visible + visible_rows + BUFFER_ROWS).min(total);

    let top_spacer = Space::new().height(start as f32 * ROW_HEIGHT);
    let bottom_spacer = Space::new().height((total - end) as f32 * ROW_HEIGHT);

    (start, end, top_spacer, bottom_spacer)
}

fn key_pressed(
    state: &mut State,
    key: keyboard::Key,
    physical_key: Physical,
    modifiers: keyboard::Modifiers,
) -> Task<Message> {
    if !matches!(state.overlay, Overlay::None) {
        return Task::none();
    }

    if modifiers.control() {
        if let Physical::Code(code) = physical_key {
            let tab_index = match code {
                Code::Digit1 => Some(0),
                Code::Digit2 => Some(1),
                Code::Digit3 => Some(2),
                Code::Digit4 => Some(3),
                Code::Digit5 => Some(4),
                Code::Digit6 => Some(5),
                Code::Digit7 => Some(6),
                Code::Digit8 => Some(7),
                Code::Digit9 => Some(8),
                _ => None,
            };
            if let Some(index) = tab_index {
                return Task::done(Message::TabOp(TabOps::SwitchTab(index)));
            }
        }
    }

    match key.as_ref() {
        keyboard::Key::Character("t") if modifiers.control() => {
            Task::done(Message::OpenInTerminalRoot)
        }
        keyboard::Key::Named(Named::Tab) => {
            let next = (state.current_tab + 1) % state.tabs.len().max(1);
            Task::done(Message::TabOp(TabOps::SwitchTab(next)))
        }

        _ => Task::none(),
    }
}
