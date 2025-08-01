use crate::image_data_wrapper::*;
use crate::prelude::*;

pub fn run_tests() {
    image_data_wrapper::create_model("fufu", DatasetType::Buildings, YoloModel::YOLOv8n);

    image_data_wrapper::train_model("fufu", 1);
    image_data_wrapper::train_model("fufu", 10);
    image_data_wrapper::train_model("fufu", 20);
    image_data_wrapper::train_model("fufu", 100);
}
