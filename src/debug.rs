use crate::image_data_wrapper::*;
use crate::prelude::*;

pub fn run_tests() {
    split_image::split("images/images.png", 100000, "images/tests");
}
