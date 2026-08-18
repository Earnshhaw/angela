#![windows_subsystem = "windows"]
mod boot;
mod dragndrop;
mod fs_handling;
mod gui;
mod rename;
mod search;
mod settings;
mod sort;
mod style;
mod subscriptions;
mod tabs;
use iced::application;

pub const DEBUG_MODE: bool = false;
const WIDTH: f32 = 1120.00;
const HEIGHT: f32 = 600.00;

fn main() -> iced::Result {
    let icon = iced::window::icon::from_file_data(include_bytes!("../assets/icon.ico"), None).ok();

    application(boot::boot, gui::update, gui::view)
        .subscription(subscriptions::subscription)
        .window(iced::window::Settings {
            icon,
            ..Default::default()
        })
        .window_size((WIDTH, HEIGHT))
        .decorations(true)
        .theme(gui::theme)
        .title("Angela")
        .run()
}
