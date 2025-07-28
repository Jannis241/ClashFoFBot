use crate::prelude::*;

pub struct Screenshot {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Screenshot {
    pub fn get_screenshot() -> Screenshot {
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

        let frame = loop {
            match capturer.frame() {
                Ok(frame) => {
                    break frame
                        .to_vec()
                        .iter()
                        .enumerate()
                        .map(|(idx, val)| if (idx + 1) % 4 == 0 { 255u8 } else { *val })
                        .collect::<Vec<u8>>()
                }
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
    pub fn save(&self, path: &Path) {
        let img: image::RgbaImage =
            ImageBuffer::from_raw(self.width as u32, self.height as u32, self.data.clone())
                .expect("Ungültige Bilddaten");

        img.save(path).expect("Konnte Screenshot nicht speichern");
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8, u8) {
        let idx = (y * self.width + x) * 4;
        let r = self.data[idx];
        let g = self.data[idx + 1];
        let b = self.data[idx + 2];
        let a = self.data[idx + 3];
        (r, g, b, a)
    }

    pub fn get_area(
        &self,
        top_left_x: usize,
        top_left_y: usize,
        width: usize,
        height: usize,
    ) -> Screenshot {
        let mut area_data = Vec::with_capacity(width * height * 4);

        for y in 0..height {
            for x in 0..width {
                let src_x = top_left_x + x;
                let src_y = top_left_y + y;

                if src_x < self.width && src_y < self.height {
                    let idx = (src_y * self.width + src_x) * 4;
                    area_data.extend_from_slice(&self.data[idx..idx + 4]);
                } else {
                    area_data.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }

        Screenshot {
            data: area_data,
            width,
            height,
        }
    }

    pub fn load(path: &Path) -> Screenshot {
        let img = ImageReader::open(path)
            .expect("Datei nicht gefunden")
            .decode()
            .expect("Bild konnte nicht geladen werden")
            .to_rgba8();

        let (width, height) = img.dimensions();
        Screenshot {
            data: img.into_raw(),
            width: width as usize,
            height: height as usize,
        }
    }
}
