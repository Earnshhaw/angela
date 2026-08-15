use std::path::PathBuf;

use crate::{
    fs_handling::{CError, rename_entry},
    gui::{GoToMethod, Message, Overlay, State},
    style::text_input_style,
};
use iced::widget::text_input;
use iced::{Element, Length, Task};

#[derive(Clone, Debug, Default)]
pub struct RenameField {
    pub source: PathBuf,
    pub dest: PathBuf,
}

impl RenameField {
    fn clear(&mut self) {
        self.dest.clear();
        self.source.clear();
    }
}
pub fn rename_toggle(state: &mut State) -> Task<Message> {
    match state.overlay {
        Overlay::Rename => state.overlay = Overlay::None,
        _ => state.overlay = Overlay::Rename,
    }
    Task::none()
}

pub fn rename_field_update(state: &mut State, field: String) -> Task<Message> {
    state.rename_field.dest = field.into();
    Task::none()
}

pub fn rename_go(state: &mut State) -> Task<Message> {
    state.overlay = Overlay::None;
    let source = state.rename_field.source.clone();
    let dest = state.rename_field.dest.clone();
    Task::perform(
        async move { rename_entry(source, dest).await },
        Message::RenameDone,
    )
}

pub fn rename_done(state: &mut State, res: Result<(), CError>) -> Task<Message> {
    match res {
        Ok(_) => {
            println!("Changed")
        }
        Err(_) => {
            eprintln!("Something fucked up")
        }
    }
    state.rename_field.clear();
    Task::done(Message::GoToDir(GoToMethod::Reload))
}

pub fn rename_popup(state: &State) -> Element<'_, Message> {
    text_input("...", &state.rename_field.dest.to_string_lossy())
        .style(text_input_style)
        .id("rename_input")
        .on_input(Message::RenameFieldUpdate)
        .on_submit(Message::RenameGo)
        .width(Length::Fixed(500.0))
        .into()
}
