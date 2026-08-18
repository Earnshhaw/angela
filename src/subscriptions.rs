use crate::gui::Message;

pub fn subscription(_state: &crate::gui::State) -> iced::Subscription<Message> {
    iced::event::listen_with(|event, _status, _window| match event {
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::CursorMoved(position))
        }
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::DragReleased)
        }
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) => Some(Message::KeyPressed {
            key,
            physical_key,
            modifiers,
        }),
        _ => None,
    })
}
