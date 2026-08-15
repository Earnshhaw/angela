//#![windows_subsystem = "windows"]
mod boot;
mod dragndrop;
mod fs_handling;
mod gui;
mod rename;
mod search;
mod settings;
mod sort;
mod style;
mod tabs;
use iced::{Size, application, window};

use crate::dragndrop::subscription;

const WIDTH: f32 = 1120.00;
const HEIGHT: f32 = 600.00;

fn main() -> iced::Result {
    application(boot::boot, gui::update, gui::view)
        .window(window::Settings {
            icon: Some(window::icon::from_file("assets/a2.png").unwrap()),
            size: Size::new(WIDTH, HEIGHT),
            ..Default::default()
        })
        .subscription(subscription)
        .decorations(true)
        .theme(gui::theme)
        .title("Angela")
        .run()
}
