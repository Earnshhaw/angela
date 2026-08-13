#![windows_subsystem = "windows"]
mod fs_handling;
mod gui;
mod style;
use iced::application;

use crate::style::{rose_pine, win11_dark};

const WIDTH: u32 = 1120;
const HEIGHT: u32 = 600;

fn main() -> iced::Result {
    application(gui::boot, gui::update, gui::view)
        .theme(win11_dark())
        .window_size((WIDTH, HEIGHT))
        .decorations(true)
        .title("Angela")
        .run()
}
