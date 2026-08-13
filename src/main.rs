#![windows_subsystem = "windows"]
mod fs_handling;
mod gui;
mod style;
use iced::application;

const WIDTH: u32 = 1120;
const HEIGHT: u32 = 600;

fn main() -> iced::Result {
    application(gui::boot, gui::update, gui::view)
        .window_size((WIDTH, HEIGHT))
        .decorations(true)
        .theme(gui::theme)
        .title("Angela")
        .run()
}
