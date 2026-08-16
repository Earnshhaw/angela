use iced::keyboard;

use crate::gui::{Message, State};

pub fn subscription(_state: &State) -> iced::Subscription<Message> {
    iced::event::listen_with(|event, _status, _window| match event {
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::CursorMoved(position))
        }
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::DragReleased)
        }
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            match key.as_ref() {
                keyboard::Key::Character("t") if modifiers.control() => {
                    Some(Message::OpenInTerminalRoot)
                }
                _ => None,
            }
        }
        _ => None,
    })
}
