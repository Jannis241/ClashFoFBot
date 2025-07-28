use crate::prelude::*;

use scrap::{Capturer, Display};
use std::{thread, time::Duration};

pub fn get_screenshot() -> (Vec<u8>, usize, usize) {
    // Hole das primäre Display
    let display = loop {
        if let Ok(display) = Display::primary() {
            break display;
        }
        eprintln!("Warte auf Display...");
        thread::sleep(Duration::from_millis(100));
    };

    // Starte den Capturer
    let mut capturer = loop {
        if let Ok(capturer) = Capturer::new(display.clone()) {
            break capturer;
        }
        eprintln!("Warte auf Capturer...");
        thread::sleep(Duration::from_millis(100));
    };

    let (width, height) = (capturer.width(), capturer.height());

    // Hole das Frame
    let frame = loop {
        match capturer.frame() {
            Ok(frame) => break frame.to_vec(),
            Err(_) => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    };

    (frame, width, height)
}
