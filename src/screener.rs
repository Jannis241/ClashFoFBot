use crate::prelude::*;

pub fn make_screenshot(screen_index: usize) -> RgbaImage {
    let screen = Screen::all().unwrap()[screen_index];
    screen.capture().unwrap()
}

pub fn screenshot_area(screen_index: usize, x: i32, y: i32, width: u32, height: u32) -> RgbaImage {
    let screen = Screen::all().unwrap()[screen_index];
    screen.capture_area(x, y, width, height).unwrap()
}
