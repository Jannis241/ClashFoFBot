use crate::image_data_wrapper::*;
use crate::prelude::*;

pub fn run_tests() {
    println!("Running tests..");

    image_data_wrapper::create_model(
        "buildings_test",
        image_data_wrapper::DatasetType::Buildings,
        YoloModel::YOLOv8n,
    );
    image_data_wrapper::create_model(
        "level_test",
        image_data_wrapper::DatasetType::Level,
        YoloModel::YOLOv8n,
    );
}
