use crate::image_data_wrapper::*;
use crate::prelude::*;

pub fn run_tests() {
    println!("Running tests..");
    try_all_image_data_wrapper_functions();
}

pub fn try_all_image_data_wrapper_functions() {
    use crate::image_data_wrapper::*;
    use std::{fs::File, io::Write, path::PathBuf};

    println!("\n==== START TESTING image_data_wrapper ====\n");

    image_data_wrapper::create_model("test_model", YoloModel::YOLOv8n);

    // Setup Testvariablen
    let valid_model = "test_model";
    let invalid_model = "non_existent_model";
    let valid_image = &"images/fufu.png";
    let invalid_image = &"fof/fofofo.agp";
    let broken_json_path = &"Test/kaputt.json";

    // --- Test: get_model_names ---
    match get_model_names() {
        Ok(models) => println!("✅ get_model_names: {:?}", models),
        Err(e) => println!("❌ get_model_names: {:?}", e),
    }

    // --- Test: create_model ---
    println!("\n-- create_model --");
    let _ = delete_model(valid_model); // ensure clean state
    match create_model(valid_model, YoloModel::YOLOv8n) {
        None => println!("✅ create_model succeeded"),
        Some(e) => println!("❌ create_model failed: {:?}", e),
    }

    match create_model(valid_model, YoloModel::YOLOv8s) {
        Some(FofError::ModelAlreadyExists) => {
            println!("✅ create_model duplicate correctly failed")
        }
        other => println!("❌ create_model duplicate unexpected result: {:?}", other),
    }

    // --- Test: get_rating ---
    println!("\n-- get_rating --");
    match get_rating(valid_model) {
        Ok(score) => println!("✅ get_rating: {:.3}", score),
        Err(e) => println!("❌ get_rating: {:?}", e),
    }

    match get_rating(invalid_model) {
        Err(FofError::NoMetricsFoundForModel(_)) => {
            println!("✅ get_rating invalid correctly failed")
        }
        other => println!("❌ get_rating invalid unexpected result: {:?}", other),
    }

    // --- Test: get_prediction mit allen Kombinationen ---
    println!("\n-- get_prediction --");
    let test_cases = vec![
        ("valid model + valid image", valid_model, &valid_image, true),
        (
            "valid model + invalid image",
            valid_model,
            &invalid_image,
            false,
        ),
        (
            "invalid model + valid image",
            invalid_model,
            &valid_image,
            false,
        ),
        (
            "invalid model + invalid image",
            invalid_model,
            &invalid_image,
            false,
        ),
    ];

    for (desc, model, image, should_succeed) in test_cases {
        println!("\nTesting case: {desc}");
        match get_prediction(model, image) {
            Ok(buildings) if should_succeed => {
                println!("✅ Prediction OK, got {} results", buildings.len())
            }
            Ok(_) => println!("❌ Expected failure but prediction succeeded!"),
            Err(e) if should_succeed => println!("❌ Expected success but failed: {:?}", e),
            Err(_) => println!("✅ Prediction correctly failed"),
        }
    }

    // --- Test: corrupt data.json ---
    println!("\n-- corrupt data.json test --");

    // simulate broken json in Communication
    let _ = fs::create_dir("Communication");
    let mut file = File::create(&broken_json_path).unwrap();
    writeln!(file, "{{ this is not valid json").unwrap(); // deliberately broken

    match get_prediction(valid_model, &valid_image) {
        Err(FofError::JsonParseError(_)) => println!("✅ corrupt data.json correctly failed"),
        other => println!("❌ corrupt data.json unexpected result: {:?}", other),
    }

    let _ = fs::remove_dir_all("Communication");

    // --- Test: get_avg_confidence ---
    println!("\n-- get_avg_confidence --");
    let empty: Vec<Building> = vec![];
    let buildings = vec![
        Building {
            class_id: 0,
            class_name: "house".to_string(),
            confidence: 0.5,
            bounding_box: (0.0, 0.0, 1.0, 1.0),
        },
        Building {
            class_id: 1,
            class_name: "tower".to_string(),
            confidence: 1.0,
            bounding_box: (0.1, 0.1, 0.9, 0.9),
        },
    ];

    match get_avg_confidence(&empty) {
        Err(FofError::DivisionByZero) => println!("✅ empty confidence failed as expected"),
        other => println!("❌ empty confidence unexpected result: {:?}", other),
    }

    match get_avg_confidence(&buildings) {
        Ok(avg) if (avg - 0.75).abs() < 0.01 => println!("✅ avg confidence = {:.2}", avg),
        other => println!("❌ avg confidence unexpected result: {:?}", other),
    }

    // --- Test: train_model ---
    println!("\n-- train_model --");
    match train_model(valid_model, 1) {
        None => println!("✅ train_model succeeded"),
        Some(e) => println!("❌ train_model failed: {:?}", e),
    }

    match train_model(invalid_model, 1) {
        Some(FofError::ModelNotFound(_)) => println!("✅ train_model invalid correctly failed"),
        other => println!("❌ train_model invalid unexpected result: {:?}", other),
    }

    // --- Test: delete_model ---
    println!("\n-- delete_model --");
    match delete_model(valid_model) {
        None => println!("✅ delete_model succeeded"),
        Some(e) => println!("❌ delete_model failed: {:?}", e),
    }

    match delete_model(valid_model) {
        Some(FofError::ModelNotFound(_)) => println!("✅ delete_model invalid correctly failed"),
        other => println!("❌ delete_model invalid unexpected result: {:?}", other),
    }

    println!("\n==== DONE TESTING image_data_wrapper ====");
}
