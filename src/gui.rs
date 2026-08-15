use crate::dragndrop::{
    DragPayload, DropTarget, cursor_moved, drag_released, hover_target, move_done, press_start,
};
use crate::fs_handling::*;
use crate::gui::Message::SearchEverythingUpdate;
use crate::rename::{
    RenameField, rename_done, rename_field_update, rename_go, rename_popup, rename_toggle,
};
use crate::search::{SearchMethod, search_everything, search_everything_update};
use crate::settings::{SettingsAction, settings_action, settings_panel, toggle_settings};
use crate::sort::{SortBy, sort_by};
use crate::style::*;
use crate::tabs::{TabOps, tab_bar_view, tab_ops};
use iced::widget::pane_grid::ResizeEvent;
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{Space, button, center, mouse_area, opaque, operation, stack, text};
use iced::{Color, Point, Theme};
use iced::{
    Element, Length, Task,
    widget::{column, container, row, scrollable, text_input},
};
use iced_aw::ContextMenu;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct State {
    pub panes: pane_grid::State<PaneKind>,
    pub tabs: Vec<Tab>,
    pub current_tab: usize,
    pub home_dir: PathBuf,
    pub search_method: SearchMethod,
    pub theme: Theme,
    pub max_results: usize,
    pub dragging: Option<DragPayload>,
    pub hovered_target: Option<DropTarget>,
    pub hovered_row: Option<usize>,
    pub overlay: Overlay,
    pub rename_field: RenameField,
}

#[derive(Debug, Clone)]
pub enum Overlay {
    Settings,
    Rename,
    None,
}

#[derive(Debug, Clone, Default)]
pub struct LoadedEntries {
    root: DirInfo,
    entries: Vec<DirInfo>,
}

impl LoadedEntries {
    pub fn new(root: DirInfo, entries: Vec<DirInfo>) -> Self {
        Self { root, entries }
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
    RowHoverStart(usize, bool),
    RowHoverEnd,
    RenameToggle,
    RenameFieldUpdate(String),
    RenameGo,
    RenameDone(Result<(), CError>),
}

pub const MAX_RESULTS_DEFAULT: usize = 100;
const SHORTCUTS: [(&str, &str); 6] = [
    ("Documents", "📄 Documents"),
    ("Downloads", "📥 Downloads"),
    ("Pictures", "🖼️ Pictures"),
    ("Videos", "📽️ Videos"),
    ("Music", "🎵 Music"),
    ("Desktop", "🖥️ Desktop"),
];
const HOME_BUTTON: &str = "🏠 Home";

#[derive(Debug, Clone)]
pub enum EntryAction {
    Open,
    Delete,
    CopyAbsolutePath,
    OpenFileLocation,
    Rename,
}

impl State {
    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.current_tab]
    }
    pub fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab]
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
        Message::RowHoverStart(index, is_dir) => row_hover_start(state, index, is_dir),
        Message::RowHoverEnd => row_hover_end(state),
        Message::RenameToggle => rename_toggle(state),
        Message::RenameFieldUpdate(field) => rename_field_update(state, field),
        Message::RenameGo => rename_go(state),
        Message::RenameDone(res) => rename_done(state, res),
    }
}

fn sidebar_view<'a>(state: &State) -> Element<'a, Message> {
    let search_everything = text_input("🔍 Search...", &state.current_tab().search_field)
        .style(text_input_style)
        .on_input(Message::SearchEverythingUpdate)
        .on_submit(Message::SearchEverything);

    let home_button = button(text(HOME_BUTTON))
        .style(secondary_button)
        .width(Length::Fill)
        .on_press(Message::GoToDir(GoToMethod::Path(state.home_dir.clone())));

    let shortcut_buttons = SHORTCUTS.iter().fold(
        column![search_everything, home_button],
        |col, (name, merged_name)| {
            col.push(
                button(text(*merged_name))
                    .style(secondary_button)
                    .width(Length::Fill)
                    .on_press(Message::GoToDir(GoToMethod::Path(
                        state.home_dir.join(name),
                    ))),
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

    let list_of_entries = scrollable(entries.iter().enumerate().fold(
        column![],
        |column, (index, entry)| {
            let entry_row = row![
                text(&entry.name).width(Length::FillPortion(4)),
                text(&entry.modified).width(Length::FillPortion(2)),
                text(&entry.size.0).width(Length::FillPortion(1))
            ]
            .spacing(50)
            .align_y(iced::Alignment::Center);

            let is_pressed = state
                .dragging
                .as_ref()
                .is_some_and(|d| d.source_tab == state.current_tab && d.source_path == index);
            let is_hovered = state.hovered_row == Some(index);
            let entry_button =
                mouse_area(container(entry_row).style(row_style(is_hovered, is_pressed)))
                    .on_press(Message::PressStart(index, entry.is_dir))
                    .on_enter(Message::RowHoverStart(index, entry.is_dir))
                    .on_exit(Message::RowHoverEnd);

            let entry_with_menu = ContextMenu::new(entry_button, move || {
                let mut column = column![
                    button("Open")
                        .on_press(Message::ContextMenuAction(index, EntryAction::Open))
                        .style(secondary_button)
                        .width(Length::Fill),
                    button("Copy Absolute Path")
                        .style(secondary_button)
                        .on_press(Message::ContextMenuAction(
                            index,
                            EntryAction::CopyAbsolutePath
                        ))
                        .width(Length::Fill),
                    button("Rename")
                        .style(secondary_button)
                        .on_press(Message::ContextMenuAction(index, EntryAction::Rename))
                        .width(Length::Fill),
                    button("Delete")
                        .style(danger_button)
                        .on_press(Message::ContextMenuAction(index, EntryAction::Delete))
                        .width(Length::Fill),
                ]
                .width(Length::Fixed(150.0));
                if !entry.is_dir {
                    column = column.push(
                        button(text("Open file location"))
                            .on_press(Message::ContextMenuAction(
                                index,
                                EntryAction::OpenFileLocation,
                            ))
                            .style(secondary_button)
                            .width(Length::Fill),
                    )
                }

                column.into()
            })
            .style(menu_container);

            column.push(entry_with_menu)
        },
    ))
    .style(scrollable_style)
    .width(Length::Fill)
    .height(Length::Fill);

    let path_display = text_input("", &state.current_tab().root_entry().path.to_string_lossy())
        .style(text_input_style)
        .on_input(Message::UpdatePath)
        .on_submit(Message::GoToDir(GoToMethod::Path(
            state.current_tab().root_entry().path.clone(),
        )));
    let back_button = button("<-")
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
    .height(Length::Fixed(32.0))
    .into()
}

fn go_to_dir(state: &mut State, method: GoToMethod) -> Task<Message> {
    state.current_tab_mut().search_field.clear();
    state.current_tab_mut().search_results_displayed = false;
    state.current_tab_mut().sorted_by = SortBy::None;
    match method {
        GoToMethod::Path(path) => Task::perform(
            async move { get_dir(path, None).await },
            Message::UpdateContent,
        ),
        GoToMethod::Index(index) => {
            let path = state.current_tab().entries()[index].path.clone();
            Task::perform(
                async move { get_dir(path, None).await },
                Message::UpdateContent,
            )
        }
        GoToMethod::Reload => {
            let path = state.current_tab().root_entry().path.clone();

            Task::perform(
                async move { get_dir(path, None).await },
                Message::UpdateContent,
            )
        }
    }
}

fn update_content(state: &mut State, res: Result<LoadedEntries, CError>) -> Task<Message> {
    if let Ok(result) = res {
        state.current_tab_mut().loaded_entries = result;
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
        GoToMethod::Reload => Task::none(), //Unreachable in practice
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
    }
}

pub fn theme(state: &State) -> Theme {
    state.theme.clone()
}

fn row_hover_start(state: &mut State, index: usize, is_dir: bool) -> Task<Message> {
    state.hovered_row = Some(index);
    if is_dir {
        state.hovered_target = Some(DropTarget::FolderRow {
            tab: state.current_tab,
            index,
        });
    }
    Task::none()
}

fn row_hover_end(state: &mut State) -> Task<Message> {
    state.hovered_row = None;
    state.hovered_target = None;
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
