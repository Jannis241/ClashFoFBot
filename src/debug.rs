use crate::image_data_wrapper::*;
use crate::prelude::*;

pub fn run_tests() {
    image_data_wrapper::create_model("testmodel", DatasetType::Level, YoloModel::YOLOv8n);
    image_data_wrapper::get_prediction("testmodel", &"images/fufu.jpg");
}
