use std::path::PathBuf;

use iced::Task;

use crate::{
    fs_handling::search_all,
    gui::{Message, State},
    sort::SortBy,
};

#[derive(Debug, Clone)]
pub struct Search {
    pub root_dir: PathBuf,
}

pub fn search_everything_update(state: &mut State, inp: String) -> Task<Message> {
    state.current_tab_mut().search_field = inp;
    Task::none()
}

pub fn search_everything(state: &mut State) -> Task<Message> {
    state.current_tab_mut().sorted_by = SortBy::None;
    let previous_dir = state.current_tab().root_entry().clone();
    let query = state.current_tab().search_field.clone();
    let root = state.search_method.root_dir.clone();
    let max_results = state.max_results;
    state.current_tab_mut().search_results_displayed = true;
    Task::perform(
        async move { search_all(query, root, previous_dir, max_results).await },
        |entries| Message::UpdateContent(Ok(entries)),
    )
}
