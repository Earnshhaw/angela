use iced::Task;

use crate::{
    fs_handling::{CError, move_entry},
    gui::{GoToMethod, Message, State},
};

#[derive(Debug, Clone)]
pub enum DropTarget {
    Tab(usize), // drop onto a tab label -> move into that tab's directory
    FolderRow { tab: usize, index: usize }, // drop onto a folder row -> move into that folder
}

#[derive(Debug, Clone)]
pub struct DragPayload {
    pub source_path: usize,
    pub source_tab: usize,
    pub is_dir: bool,
    pub press_position: Option<iced::Point>,
    pub confirmed: bool,
}

pub const DRAG_THRESHOLD: f32 = 1.0;

pub fn subscription(_state: &State) -> iced::Subscription<Message> {
    iced::event::listen_with(|event, _status, _window| match event {
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::CursorMoved(position))
        }
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::DragReleased)
        }
        _ => None,
    })
}

pub fn cursor_moved(position: iced::Point, state: &mut State) -> Task<Message> {
    let Some(drag) = &mut state.dragging else {
        return Task::none();
    };
    let anchor = *drag.press_position.get_or_insert(position);
    if !drag.confirmed && anchor.distance(position) > DRAG_THRESHOLD {
        drag.confirmed = true;
    }
    Task::none()
}

pub fn press_start(index: usize, is_dir: bool, state: &mut State) -> Task<Message> {
    state.dragging = Some(DragPayload {
        source_path: index,
        is_dir,
        source_tab: state.current_tab,
        press_position: None,
        confirmed: false,
    });

    Task::none()
}

pub fn drag_released(state: &mut State) -> Task<Message> {
    {
        let Some(drag) = state.dragging.take() else {
            return Task::none();
        };

        if !drag.confirmed {
            return if drag.is_dir {
                Task::done(Message::GoToDir(GoToMethod::Index(drag.source_path)))
            } else {
                Task::done(Message::OpenFile(GoToMethod::Index(drag.source_path)))
            };
        }

        let hovered = state.hovered_target.take();
        let Some(target) = hovered else {
            return Task::none();
        };

        let dest_dir = match target {
            DropTarget::Tab(i) => state.tabs[i].root_entry().path.clone(),
            DropTarget::FolderRow { tab, index } => state.tabs[tab].entries()[index].path.clone(),
        };

        let source_path = state.tabs[drag.source_tab].entries()[drag.source_path]
            .path
            .clone();

        Task::perform(
            async move { move_entry(source_path, dest_dir).await },
            Message::MoveDone,
        )
    }
}

pub fn hover_target(target: Option<DropTarget>, state: &mut State) -> Task<Message> {
    state.hovered_target = target;
    Task::none()
}

pub fn move_done(result: Result<(), CError>, state: &mut State) -> Task<Message> {
    state.dragging = None;
    state.hovered_target = None;
    match result {
        Ok(_) => {}
        Err(_) => {}
    }
    Task::done(Message::GoToDir(GoToMethod::Reload))
}
