use crate::prelude::*;

use image::{ImageBuffer, Rgba, RgbaImage};
use std::{error::Error, thread::panicking};

pub fn split(image_path: &str, num_of_parts: i32, save_path: &str) {
    let image: RgbaImage = image::open(image_path).unwrap().into();

    let width = image.width();
    let height = image.height();

    // Wurzel von num_of_parts bestimmen, z. B. 4 → 2x2, 16 → 4x4
    let parts_per_side = (num_of_parts as f32).sqrt() as u32;

    let part_width = width / parts_per_side;
    let part_height = height / parts_per_side;

    let mut result = Vec::new();

    for y_part in 0..parts_per_side {
        for x_part in 0..parts_per_side {
            let mut sub_img = ImageBuffer::new(part_width, part_height);

            for y in 0..part_height {
                for x in 0..part_width {
                    let pixel = image.get_pixel(x + x_part * part_width, y + y_part * part_height);
                    sub_img.put_pixel(x, y, *pixel);
                }
            }

            result.push(sub_img);
        }
    }

    let path = Path::new(image_path);

    if let Some(file_name) = path.file_name() {
        for (idx, splited_image) in result.iter().enumerate() {
            let p = build_output_path(save_path, image_path, idx);

            println!("Saving splited image in {}", &p);
            splited_image.save(p);
        }
    } else {
        panic!("Error hier in split image");
    }
}

use std::path::Path;

fn build_output_path(save_path: &str, file_path: &str, idx: usize) -> String {
    let path = Path::new(file_path);

    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();

    let extension = path.extension().unwrap_or_default().to_string_lossy();

    format!("{}/{}_split_{}.{}", save_path, file_stem, idx, extension)
}
