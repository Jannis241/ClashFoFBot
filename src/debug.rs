use crate::prelude::*;

pub fn run_tests() {
    println!("Runnings tests..");

    image_data_wrapper::create_model("test", image_data_wrapper::YoloModel::YOLOv8s);

    println!("{:?}", image_data_wrapper::get_model_names());
}
