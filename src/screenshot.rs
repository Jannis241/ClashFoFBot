use crate::prelude::*;

pub struct Screenshot {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Screenshot {
    pub fn get_screenshot() -> Screenshot {
        // Hole das primäre Display

        // Starte den Capturer
        let mut capturer = loop {
            let display = loop {
                if let Ok(display) = Display::primary() {
                    break display;
                }
                eprintln!("Warte auf Display...");
                thread::sleep(Duration::from_millis(100));
            };
            if let Ok(capturer) = Capturer::new(display) {
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

        return Screenshot {
            data: frame,
            width: width,
            height: height,
        };
    }
}
