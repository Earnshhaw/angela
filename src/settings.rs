use std::path::PathBuf;

use crate::{
    gui::{MAX_RESULTS_DEFAULT, Overlay},
    style::*,
};
use iced::{
    Element, Length, Task, Theme,
    widget::{button, column, container, text, text_input},
};

use crate::gui::{Message, State};

#[derive(Debug, Clone)]
pub enum SettingsAction {
    ChangeTheme(Theme),
    ChangeSearchRoot(PathBuf),
    ChangeMatchLimit(usize),
}

pub fn toggle_settings(state: &mut State) -> Task<Message> {
    state.overlay = match state.overlay {
        Overlay::None => Overlay::Settings,
        Overlay::Settings => Overlay::None,
        _ => state.overlay.clone(),
    };
    Task::none()
}

pub fn settings_panel(state: &State) -> Element<'_, Message> {
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
            .on_press(Message::SettingsAction(SettingsAction::ChangeSearchRoot(
                state.home_dir.clone()
            )))
            .style(secondary_button),
        button("Custom Directory")
            .on_press(Message::RfdOpen)
            .style(secondary_button),
    ])
    .into()
}

pub fn settings_action(state: &mut State, action: SettingsAction) -> Task<Message> {
    match action {
        SettingsAction::ChangeTheme(theme) => state.theme = theme,
        SettingsAction::ChangeMatchLimit(limit) => state.max_results = limit,
        SettingsAction::ChangeSearchRoot(root) => state.search_method.root_dir = root,
    }
    Task::none()
}
