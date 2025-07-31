use crate::image_data_wrapper::*;
use crate::prelude::*;

pub fn run_tests() {
    println!("Running tests..");

    image_data_wrapper::create_model(
        "buildings_test",
        image_data_wrapper::DatasetType::Buildings,
        YoloModel::YOLOv8n,
    );

    // println!(
    //     "{:?}",
    //     image_data_wrapper::get_dataset_type("buildings_test")
    // );

    image_data_wrapper::create_model(
        "fufu_model",
        image_data_wrapper::DatasetType::Buildings,
        YoloModel::YOLOv8n,
    );
    image_data_wrapper::create_model(
        "level_model",
        image_data_wrapper::DatasetType::Level,
        YoloModel::YOLOv8n,
    );
    image_data_wrapper::create_model(
        "dede_model",
        image_data_wrapper::DatasetType::Level,
        YoloModel::YOLOv8n,
    );

    image_data_wrapper::train_model("buildings_test", 1);
    image_data_wrapper::train_model("dede", 1);
    image_data_wrapper::train_model("fufu_model", 1);

    let all_models = image_data_wrapper::get_all_models();
}
