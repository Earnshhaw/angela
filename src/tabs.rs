use crate::dragndrop::DropTarget;
use crate::gui::GoToMethod;
use crate::gui::LoadedEntries;
use crate::gui::Message;

use crate::gui::State;
use crate::gui::Tab;
use crate::sort::SortBy;
use crate::style::menu_container;
use crate::style::primary_button;
use crate::style::secondary_button;
use iced::Element;
use iced::Length;
use iced::Task;
use iced::widget::button;
use iced::widget::container;
use iced::widget::mouse_area;
use iced::widget::row;
use iced::widget::text;
use iced_aw::ContextMenu;

#[derive(Debug, Clone)]
pub enum TabOps {
    NewTab,
    SwitchTab(usize),
    CloseTab(usize),
    CloseAllTabs,
}

pub fn tab_ops(state: &mut State, ops: TabOps) -> Task<Message> {
    match ops {
        TabOps::NewTab => {
            let home = state.home_dir.clone();
            state.tabs.push(Tab {
                loaded_entries: LoadedEntries::default(),
                search_field: String::new(),
                search_results_displayed: false,
                sorted_by: SortBy::None,
            });
            state.current_tab = state.tabs.len() - 1;
            Task::done(Message::GoToDir(GoToMethod::Path(home)))
        }
        TabOps::SwitchTab(index) => {
            if index >= state.tabs.len() {
                return Task::none();
            }
            state.current_tab = index;
            Task::done(Message::GoToDir(GoToMethod::Reload))
        }
        TabOps::CloseTab(index) => {
            if state.tabs.len() == 1 {
                Task::none()
            } else {
                state.tabs.remove(index);
                state.current_tab = state.tabs.len() - 1;
                Task::done(Message::GoToDir(GoToMethod::Reload))
            }
        }
        TabOps::CloseAllTabs => {
            state.tabs.truncate(1);
            state.current_tab = 0;
            Task::done(Message::GoToDir(GoToMethod::Reload))
        }
    }
}

pub fn tab_bar_view(state: &State) -> Element<'_, Message> {
    let tabs = state.tabs.iter().enumerate().fold(row![], |row, (i, tab)| {
        let label = mouse_area(
            button(text(
                tab.root_entry()
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("/"),
            ))
            .on_press(Message::TabOp(TabOps::SwitchTab(i)))
            .style(if i == state.current_tab {
                primary_button
            } else {
                secondary_button
            }),
        )
        .on_enter(Message::HoverTarget(Some(DropTarget::Tab(i))))
        .on_exit(Message::HoverTarget(None));

        let with_context = ContextMenu::new(label, move || {
            let close_tab = button("Close").on_press(Message::TabOp(TabOps::CloseTab(i)));
            let close_all = button("Close All").on_press(Message::TabOp(TabOps::CloseAllTabs));
            row![
                close_tab.style(secondary_button),
                close_all.style(secondary_button)
            ]
            .into()
        })
        .style(menu_container);

        row.push(with_context)
    });

    let new_tab_button = button(text("+"))
        .on_press(Message::TabOp(TabOps::NewTab))
        .style(primary_button);

    container(row![tabs, new_tab_button].spacing(4))
        .height(Length::Fixed(32.0))
        .into()
}
