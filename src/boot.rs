use std::{
    env::home_dir,
    path::{Path, PathBuf},
};

use crate::{
    fs_handling::sync_get_dir,
    gui::{MAX_RESULTS_DEFAULT, Overlay, PaneKind, State, Tab},
    rename::RenameField,
    search::Search,
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
        search_method: Search {
            root_dir: home.clone(),
        },
        theme: win11_dark(),
        max_results: MAX_RESULTS_DEFAULT,
        dragging: None,
        hovered_target: None,
        hovered_row: None,
        hovered_shortcut: None,
        overlay: Overlay::None,
        rename_field: RenameField::default(),
        shortcut_dirs: init_shortcut_dirs(&home),
        scroll_offset: 0.0,
        viewport_height: 600.0,
    }
}

pub fn init_shortcut_dirs(home: &Path) -> [(PathBuf, String); 7] {
    let init_paths = [
        home.to_owned(),
        home.join("Documents"),
        home.join("Downloads"),
        home.join("Pictures"),
        home.join("Videos"),
        home.join("Music"),
        home.join("Desktop"),
    ];
    let init_names = [
        "🏠 Home".to_string(),
        "📄 Documents".to_string(),
        "📥 Downloads".to_string(),
        "🖼️ Pictures".to_string(),
        "📽️ Videos".to_string(),
        "🎵 Music".to_string(),
        "🖥️ Desktop".to_string(),
    ];
    init_paths
        .into_iter()
        .zip(init_names.into_iter())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
