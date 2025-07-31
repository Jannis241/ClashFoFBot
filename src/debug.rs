use crate::{image_data_wrapper::Building, image_data_wrapper::YoloModel, prelude::*};

pub fn run_tests() {
    println!("Runnings tests..");
    image_data_wrapper::create_model("small_model".into(), YoloModel::YOLOv8s);
    image_data_wrapper::create_model("medium_model".into(), YoloModel::YOLOv8m);

    for i in 0..100 {
        image_data_wrapper::train_model("medium_model".into(), 1);

        println!(
            "Small model score: {}",
            image_data_wrapper::get_rating("small_model".into())
        );
        println!(
            "Medium model score: {}",
            image_data_wrapper::get_rating("medium_model".into())
        );
    }
}
