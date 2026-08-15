use std::env::home_dir;

use crate::{
    fs_handling::sync_get_dir,
    gui::{MAX_RESULTS_DEFAULT, Overlay, PaneKind, State, Tab},
    rename::RenameField,
    search::SearchMethod,
    sort::SortBy,
    style::win11_dark,
};
use iced::widget::pane_grid;

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
        search_field: String::new(),
        search_results_displayed: false,
        sorted_by: SortBy::None,
    }];
    State {
        panes,
        tabs,
        current_tab: 0,
        home_dir: home.clone(),
        search_method: SearchMethod::FromHomeDirectory(home),
        theme: win11_dark(),

        max_results: MAX_RESULTS_DEFAULT,
        dragging: None,
        hovered_target: None,
        hovered_row: None,
        overlay: Overlay::None,
        rename_field: RenameField::default(),
    }
}
