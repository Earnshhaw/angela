use iced::Task;

use crate::gui::{Message, State};

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    FileType,
    Name,
    Size,
    Date,
    None,
}

pub fn sort_by(state: &mut State, sortmethod: SortBy) -> Task<Message> {
    match sortmethod {
        SortBy::FileType => match state.current_tab().sorted_by {
            SortBy::FileType => {
                state.current_tab_mut().entries_mut().reverse();
            }
            _ => {
                state
                    .current_tab_mut()
                    .entries_mut()
                    .sort_unstable_by(|a, b| b.is_dir.cmp(&a.is_dir));
                state.current_tab_mut().sorted_by = SortBy::FileType;
            }
        },
        SortBy::Date => match state.current_tab().sorted_by {
            SortBy::Date => {
                state.current_tab_mut().entries_mut().reverse();
            }
            _ => {
                state
                    .current_tab_mut()
                    .entries_mut()
                    .sort_unstable_by(|a, b| a.modified.cmp(&b.modified));
                state.current_tab_mut().sorted_by = SortBy::Date;
            }
        },
        SortBy::Name => match state.current_tab().sorted_by {
            SortBy::Name => {
                state.current_tab_mut().entries_mut().reverse();
            }
            _ => {
                state
                    .current_tab_mut()
                    .entries_mut()
                    .sort_unstable_by(|a, b| a.name.cmp(&b.name));
                state.current_tab_mut().sorted_by = SortBy::Name;
            }
        },
        SortBy::Size => match state.current_tab().sorted_by {
            SortBy::Size => {
                state.current_tab_mut().entries_mut().reverse();
            }
            _ => {
                state
                    .current_tab_mut()
                    .entries_mut()
                    .sort_unstable_by(|a, b| a.size.1.cmp(&b.size.1));
                state.current_tab_mut().sorted_by = SortBy::Size;
            }
        },
        SortBy::None => {
            state.current_tab_mut().sorted_by = SortBy::None;
        }
    }
    Task::none()
}
