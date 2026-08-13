use crate::fs_handling::*;
use crate::gui::Message::SearchEverythingUpdate;
use crate::style::*;
use iced::widget::pane_grid::ResizeEvent;
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{Space, button, center, mouse_area, opaque, stack, text};
use iced::{Color, Theme};
use iced::{
    Element, Length, Task,
    widget::{column, container, row, scrollable, text_input},
};
use iced_aw::ContextMenu;
use std::{env::home_dir, path::PathBuf};

#[derive(Debug, Clone)]
pub struct State {
    pub panes: pane_grid::State<PaneKind>,
    pub tabs: Vec<Tab>,
    pub current_tab: usize,
    pub home_dir: PathBuf,
    pub search_method: SearchMethod,
    pub theme: Theme,
    pub settings_open: bool,
    pub max_results: usize,
}

const MAX_RESULTS_DEFAULT: usize = 50;

#[derive(Debug, Clone)]
enum SearchMethod {
    FromHomeDirectory(PathBuf),
    FromCustomDirectory,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub loaded_entries: Vec<DirInfo>,
    pub current_path: PathBuf,
    pub search_field: String,
    pub search_results_displayed: bool,
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
pub enum TabOps {
    NewTab,
    SwitchTab(usize),
    CloseTab(usize),
    CloseAllTabs,
}

#[derive(Debug, Clone)]
pub enum Message {
    PaneResized(ResizeEvent),
    GoToDir(GoToMethod),
    GoBack,
    UpdatePath(String),
    OpenFile(GoToMethod),
    UpdateContent(Result<Vec<DirInfo>, CError>),
    TabOp(TabOps),
    ContextMenuAction(usize, EntryAction),
    DeleteDone,
    SearchEverythingUpdate(String),
    SearchEverything,
    ToggleSettings,
    SettingsAction(SettingsAction),
}

#[derive(Debug, Clone)]
enum SettingsAction {
    ChangeTheme(Theme),
    ChangeSearchMethod(SearchMethod),
    ChangeMatchLimit(usize),
}

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
}

impl State {
    fn current_tab(&self) -> &Tab {
        &self.tabs[self.current_tab]
    }
    fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab]
    }
}

impl Tab {
    fn self_entry(&self) -> Option<&DirInfo> {
        self.loaded_entries.first()
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

    if state.settings_open {
        modal(pane, settings_panel(state), Message::ToggleSettings)
    } else {
        pane.into()
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
    }
}

pub fn boot() -> State {
    let panes = pane_grid::State::with_configuration(pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 1.0 / 6.0,
        a: Box::new(pane_grid::Configuration::Pane(PaneKind::Sidebar)),
        b: Box::new(pane_grid::Configuration::Pane(PaneKind::FileView)),
    });
    let home = home_dir().unwrap();
    let entries = sync_get_dir(&home, None).unwrap();
    let tabs: Vec<Tab> = vec![Tab {
        loaded_entries: entries,
        current_path: home.clone(),
        search_field: String::new(),
        search_results_displayed: false,
    }];
    State {
        panes,
        tabs,
        current_tab: 0,
        home_dir: home.clone(),
        search_method: SearchMethod::FromHomeDirectory(home),
        theme: win11_dark(),
        settings_open: false,
        max_results: MAX_RESULTS_DEFAULT,
    }
}

fn tab_bar_view(state: &State) -> Element<'_, Message> {
    let tabs = state.tabs.iter().enumerate().fold(row![], |row, (i, tab)| {
        let id = format!("dropzone_{}", i);
        let label = button(text(
            tab.current_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("/"),
        ))
        .on_press(Message::TabOp(TabOps::SwitchTab(i)))
        .style(if i == state.current_tab {
            primary_button
        } else {
            secondary_button
        });
        let with_context = ContextMenu::new(label, move || {
            let close_tab = button("Close").on_press(Message::TabOp(TabOps::CloseTab(i)));
            let close_all = button("Close All").on_press(Message::TabOp(TabOps::CloseAllTabs));
            row![
                close_tab.style(secondary_button),
                close_all.style(secondary_button)
            ]
            .into()
        })
        .style(menu_container);

        row.push(with_context)
    });

    let new_tab_button = button(text("+"))
        .on_press(Message::TabOp(TabOps::NewTab))
        .style(primary_button);

    container(row![tabs, new_tab_button].spacing(4))
        .height(Length::Fixed(32.0))
        .into()
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
    let current_tab = state.current_tab();
    // entry 0 is a sentinel representing this directory itself, therefore it is skipped for display
    let list_of_entries = scrollable(current_tab.loaded_entries.iter().enumerate().skip(1).fold(
        column![],
        |column, (index, entry)| {
            let maybe_dir = if entry.is_dir { "📁" } else { "" };
            let size = if let Some(size) = &entry.size {
                format!("{} {}", size.value, size.unit)
            } else {
                String::new()
            };
            let entry_row = row![
                text(format!("{}{}", maybe_dir, entry.name)).width(Length::FillPortion(4)),
                text(&entry.modified).width(Length::FillPortion(2)),
                text(size).width(Length::FillPortion(1))
            ]
            .spacing(50)
            .align_y(iced::Alignment::Center);
            let entry_button = button(entry_row)
                .style(row_button)
                .on_press(if entry.is_dir {
                    Message::GoToDir(GoToMethod::Index(index))
                } else {
                    Message::OpenFile(GoToMethod::Index(index))
                });
            let entry_with_menu = ContextMenu::new(entry_button, move || {
                column![
                    button(text("Open"))
                        .on_press(Message::ContextMenuAction(index, EntryAction::Open))
                        .style(secondary_button)
                        .width(Length::Fill),
                    button(text("Copy Absolute Path"))
                        .style(secondary_button)
                        .on_press(Message::ContextMenuAction(
                            index,
                            EntryAction::CopyAbsolutePath
                        ))
                        .width(Length::Fill),
                    button(text("Delete"))
                        .style(danger_button)
                        .on_press(Message::ContextMenuAction(index, EntryAction::Delete))
                        .width(Length::Fill),
                ]
                .width(Length::Fixed(150.0))
                .into()
            })
            .style(menu_container);

            column.push(entry_with_menu)
        },
    ))
    .style(scrollable_style)
    .width(Length::Fill);

    let path_display = text_input("", &current_tab.current_path.to_string_lossy())
        .style(text_input_style)
        .on_input(Message::UpdatePath)
        .on_submit(Message::GoToDir(GoToMethod::Path(
            current_tab.current_path.clone(),
        )));
    let back_button = button("<-")
        .on_press(Message::GoBack)
        .style(secondary_button);

    column![
        tab_bar_view(state),
        row![back_button, path_display],
        list_of_entries
    ]
    .into()
}

fn tab_ops(state: &mut State, ops: TabOps) -> Task<Message> {
    match ops {
        TabOps::NewTab => {
            let home = state.home_dir.clone();
            state.tabs.push(Tab {
                loaded_entries: Vec::new(),
                current_path: home.clone(),
                search_field: String::new(),
                search_results_displayed: false,
            });
            state.current_tab = state.tabs.len() - 1;
            Task::done(Message::GoToDir(GoToMethod::Path(home)))
        }
        TabOps::SwitchTab(index) => {
            state.current_tab = index;
            Task::done(Message::GoToDir(GoToMethod::Reload))
        }
        TabOps::CloseTab(index) => {
            if state.tabs.len() == 1 {
                Task::none()
            } else {
                state.tabs.remove(index);
                state.current_tab = state.tabs.len() - 1;
                Task::done(Message::GoToDir(GoToMethod::Reload))
            }
        }
        TabOps::CloseAllTabs => {
            state.tabs.truncate(1);
            state.current_tab = 0;
            Task::done(Message::GoToDir(GoToMethod::Reload))
        }
    }
}

fn go_to_dir(state: &mut State, method: GoToMethod) -> Task<Message> {
    state.current_tab_mut().search_field.clear();
    state.current_tab_mut().search_results_displayed = false;
    match method {
        GoToMethod::Path(path) => Task::perform(
            async move { get_dir(path, None).await },
            Message::UpdateContent,
        ),
        GoToMethod::Index(index) => {
            let path = state.current_tab().loaded_entries[index].path.clone();
            Task::perform(
                async move { get_dir(path, None).await },
                Message::UpdateContent,
            )
        }
        GoToMethod::Reload => {
            let path = state.current_tab().self_entry().unwrap().path.clone();

            Task::perform(
                async move { get_dir(path, None).await },
                Message::UpdateContent,
            )
        }
    }
}

fn update_content(state: &mut State, res: Result<Vec<DirInfo>, CError>) -> Task<Message> {
    if let Ok(result) = res {
        if let Some(first) = result.first() {
            println!("Some");
            state.current_tab_mut().current_path = first.path.clone();
            state.current_tab_mut().loaded_entries = result;
        }
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
            if state
                .current_tab_mut()
                .loaded_entries
                .first_mut()
                .unwrap()
                .path
                .pop()
            {
                Task::done(Message::GoToDir(GoToMethod::Reload))
            } else {
                Task::none()
            }
        }
    }
}

fn update_path(state: &mut State, path: String) -> Task<Message> {
    state.current_tab_mut().current_path = path.into();
    Task::none()
}

fn open_file(state: &mut State, method: GoToMethod) -> Task<Message> {
    match method {
        GoToMethod::Index(index) => {
            open::that(&state.current_tab().loaded_entries[index].path).unwrap();
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
            if state.current_tab().loaded_entries[index].is_dir {
                Task::done(Message::GoToDir(GoToMethod::Index(index)))
            } else {
                Task::done(Message::OpenFile(GoToMethod::Index(index)))
            }
        }
        EntryAction::Delete => {
            let path = state.current_tab().loaded_entries[index].path.clone();
            Task::future(async move {
                delete_dir(path).await.ok();
                Message::DeleteDone
            })
        }
        EntryAction::CopyAbsolutePath => {
            let path = state.current_tab().loaded_entries[index].path.clone();
            return iced::clipboard::write(path.to_string_lossy().into_owned());
        }
    }
}

fn search_everything_update(state: &mut State, inp: String) -> Task<Message> {
    state.current_tab_mut().search_field = inp;
    Task::none()
}

fn search_everything(state: &mut State) -> Task<Message> {
    let previous_dir = state
        .current_tab()
        .self_entry()
        .cloned()
        .unwrap_or_default();
    match &state.search_method {
        SearchMethod::FromHomeDirectory(home) => {
            let query = state.current_tab().search_field.clone();
            let root = home.clone();
            let previous_dir = previous_dir.clone();
            let max_results = state.max_results;
            state.current_tab_mut().search_results_displayed = true;
            Task::perform(
                async move { search_all(query, root, previous_dir, max_results).await },
                |entries| Message::UpdateContent(Ok(entries)),
            )
        }
        SearchMethod::FromCustomDirectory => Task::none(),
    }
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

fn toggle_settings(state: &mut State) -> Task<Message> {
    state.settings_open = !state.settings_open;
    Task::none()
}

fn settings_panel(state: &State) -> Element<'_, Message> {
    container(column![
        text("Settings").size(20),
        text("Set Theme"),
        button("Dark Theme")
            .on_press(Message::SettingsAction(SettingsAction::ChangeTheme(
                win11_dark()
            )))
            .style(secondary_button),
        button("Light Theme")
            .on_press(Message::SettingsAction(SettingsAction::ChangeTheme(
                win11_light()
            )))
            .style(secondary_button),
        button("Rosé Pine")
            .on_press(Message::SettingsAction(SettingsAction::ChangeTheme(
                rose_pine()
            )))
            .style(secondary_button),
        text("Set Maximum Amount of Results in Search"),
        text_input("", &state.max_results.to_string())
            .on_input(
                |value| Message::SettingsAction(SettingsAction::ChangeMatchLimit(
                    value.parse().unwrap_or(MAX_RESULTS_DEFAULT)
                ))
            )
            .width(Length::Fixed(100.00))
            .style(text_input_style),
        text("Set the start directory for the search"),
        button("Home Directory (Default)")
            .on_press(Message::SettingsAction(SettingsAction::ChangeSearchMethod(
                SearchMethod::FromHomeDirectory(state.home_dir.clone())
            )))
            .style(secondary_button),
        button("Custom Directory")
            .on_press(Message::SettingsAction(SettingsAction::ChangeSearchMethod(
                SearchMethod::FromCustomDirectory
            )))
            .style(secondary_button),
    ])
    .into()
}

fn settings_action(state: &mut State, action: SettingsAction) -> Task<Message> {
    match action {
        SettingsAction::ChangeTheme(theme) => state.theme = theme,
        SettingsAction::ChangeMatchLimit(limit) => state.max_results = limit,
        SettingsAction::ChangeSearchMethod(method) => state.search_method = method,
    }
    Task::none()
}

pub fn theme(state: &State) -> Theme {
    state.theme.clone()
}
