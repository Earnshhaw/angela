use crate::{
    DEBUG_MODE,
    fs_handling::{CError, move_entry},
    gui::{GoToMethod, Message, State},
};
use iced::Task;

#[derive(Debug, Clone)]
pub enum DropTarget {
    Tab(usize),
    FolderRow { tab: usize, index: usize },
    Shortcut(usize),
}

#[derive(Debug, Clone)]
pub struct DragPayload {
    pub source_path: usize,
    pub source_tab: usize,
    pub is_dir: bool,
    pub press_position: Option<iced::Point>,
    pub confirmed: bool,
}

pub const DRAG_THRESHOLD: f32 = 3.0;

pub fn cursor_moved(position: iced::Point, state: &mut State) -> Task<Message> {
    if let Some(drag) = &mut state.dragging {
        let anchor = *drag.press_position.get_or_insert(position);
        if !drag.confirmed && anchor.distance(position) > DRAG_THRESHOLD {
            drag.confirmed = true;
        }
        if drag.confirmed && !state.current_tab_mut().loaded_entries.expanded {
            state.current_tab_mut().loaded_entries.expand();
        }
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
    let drag = match state.dragging.take() {
        Some(drag) => drag,
        None => return Task::none(),
    };

    if !drag.confirmed {
        return if drag.is_dir {
            Task::done(Message::GoToDir(GoToMethod::Index(drag.source_path)))
        } else {
            Task::done(Message::OpenFile(GoToMethod::Index(drag.source_path)))
        };
    }

    let hovered = state.hovered_target.take();
    let target = match hovered {
        Some(target) => target,
        None => {
            if state.current_tab().expanded() {
                state.current_tab_mut().loaded_entries.collapse();
            }
            return Task::none();
        }
    };

    let dest_dir = match target {
        DropTarget::Tab(i) => state.tabs[i].root_entry().path.clone(),
        DropTarget::FolderRow { tab, index } => state.tabs[tab].entries()[index].path.clone(),
        DropTarget::Shortcut(i) => state.shortcut_dirs[i].0.clone(),
    };

    let source_path = state.tabs[drag.source_tab].entries()[drag.source_path]
        .path
        .clone();

    if DEBUG_MODE {
        println!(
            "dest: {} source: {}",
            dest_dir.to_string_lossy(),
            source_path.to_string_lossy()
        );
    }
    if state.current_tab().expanded() {
        state.current_tab_mut().loaded_entries.collapse();
    }

    Task::perform(
        async move { move_entry(source_path, dest_dir).await },
        Message::MoveDone,
    )
}

pub fn hover_target(target: Option<DropTarget>, state: &mut State) -> Task<Message> {
    if DEBUG_MODE {
        println!("{:?}", target);
    }
    state.hovered_target = target;
    Task::none()
}

pub fn move_done(result: Result<(), CError>, state: &mut State) -> Task<Message> {
    if DEBUG_MODE {
        println!("move_done: {:?}", result);
    }
    state.dragging = None;
    state.hovered_target = None;

    match result {
        Ok(_) => {}
        Err(_) => {}
    }
    Task::done(Message::GoToDir(GoToMethod::Reload))
}
