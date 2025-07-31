use std::fmt::format;

use crate::prelude::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Building {
    pub class_id: i32,
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: (f32, f32, f32, f32),
}

#[derive(Debug, Deserialize)]
struct Metrics {
    #[serde(rename = "metrics/precision(B)")]
    precision: f64,

    #[serde(rename = "metrics/recall(B)")]
    recall: f64,

    #[serde(rename = "metrics/mAP50(B)")]
    map_50: f64,

    #[serde(rename = "metrics/mAP50-95(B)")]
    map_50_95: f64,
}

fn calculate_score(m: &Metrics) -> f64 {
    0.4 * m.map_50_95 + 0.3 * m.map_50 + 0.15 * m.precision + 0.15 * m.recall
}

fn read_last_metrics(model_name: &str) -> Option<Metrics> {
    let path = format!("runs/detect/{}/results.csv", model_name);
    let file = File::open(&path).ok()?;
    let mut rdr = Reader::from_reader(file);
    let mut last: Option<Metrics> = None;

    for result in rdr.deserialize() {
        if let Ok(row) = result {
            last = Some(row);
        }
    }
    last
}

pub fn get_rating(model_name: &str) -> Result<f64, FofError> {
    println!("Searching for metrics of the '{}' model.", model_name);
    if let Some(m) = read_last_metrics(model_name) {
        println!("Success: Found Metrics for the '{}' model.", model_name);
        Ok(calculate_score(&m))
    } else {
        eprintln!("Error: No Metrics found for {}!", model_name);
        Err(FofError::NoMetricsFoundForModel(model_name.to_string()))
    }
}

pub fn get_model_names() -> Result<Vec<String>, FofError> {
    println!("Searching for all model names..");

    let mut model_names = Vec::new();
    let path = Path::new("runs/detect");

    let entries = match fs::read_dir(path) {
        Ok(entries) => {
            println!("✅ Found directory: {}", path.display());
            entries
        }
        Err(err) => {
            eprintln!("❌ Failed to read directory '{}': {}", path.display(), err);
            return Err(FofError::FailedReadingDirectory(path.display().to_string()));
        }
    };

    for entry_result in entries {
        match entry_result {
            Ok(entry) => {
                let metadata = match entry.metadata() {
                    Ok(md) => md,
                    Err(err) => {
                        eprintln!("⚠️ Failed to get metadata for {:?}: {}", entry.path(), err);
                        return Err(FofError::Failed(format!(
                            "Failed to get metadata for {:?}: {}",
                            entry.path(),
                            err
                        )));
                    }
                };

                if metadata.is_dir() {
                    match entry.file_name().to_str() {
                        Some(name) => {
                            println!("📁 Found model directory: {}", name);
                            model_names.push(name.to_string());
                        }
                        None => {
                            eprintln!(
                                "⚠️ Invalid UTF-8 in directory name: {:?}",
                                entry.file_name()
                            );
                            return Err(FofError::Failed(format!(
                                "Invalid UTF-8 in directory name: {:?}",
                                entry.file_name()
                            )));
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("⚠️ Failed to read a directory entry: {}", err);
                return Err(FofError::Failed(format!(
                    "Failed to read a directory entry: {}",
                    err
                )));
            }
        }
    }

    Ok(model_names)
}

#[derive(Debug)]
pub enum YoloModel {
    yolov8n,
    YOLOv8s,
    YOLOv8m,
    YOLOv8l,
    YOLOv8x,
}

pub fn get_avg_confidence(buildings: &[Building]) -> Result<f32, FofError> {
    println!("Calculating average confidence..");
    if buildings.is_empty() {
        eprintln!("Error: Es wurden keine Buildings angegeben.");
        return Err(FofError::DivisionByZero);
    }

    let sum: f32 = buildings.iter().map(|b| b.confidence).sum();
    Ok(sum / buildings.len() as f32)
}

pub fn create_model(model_name: &str, yolo_model: YoloModel) -> Option<FofError> {
    println!(
        "Creating new model '{}' with the '{:?}' yolo base model.",
        model_name, yolo_model
    );

    if let Ok(true) = fs::exists(format!("runs/detect/{}", model_name)) {
        eprintln!("Error: Model '{}' already exists. Aborting..", model_name);
        return Some(FofError::ModelAlreadyExists);
    }

    println!("Parsing YOLO base model..");
    let yolo_model_string = match yolo_model {
        YoloModel::yolov8n => "yolov8n.pt",
        YoloModel::YOLOv8s => "yolov8s.pt",
        YoloModel::YOLOv8m => "yolov8m.pt",
        YoloModel::YOLOv8l => "yolov8l.pt",
        YoloModel::YOLOv8x => "yolov8x.pt",
    };
    println!("Found YOLO base model!");

    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--create-model")
        .arg("--base")
        .arg(yolo_model_string)
        .arg("--model-name")
        .arg(model_name)
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Python script executed.");
            None
        }
        Ok(output) => {
            eprintln!("Python error: {}", String::from_utf8_lossy(&output.stderr));
            Some(FofError::PythonError)
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            Some(FofError::FailedToStartPython)
        }
    }
}

pub fn delete_model(model_name: &str) -> Option<FofError> {
    let path = format!("runs/detect/{}", model_name);

    println!("Trying to delete model '{}'..", model_name);
    println!("Searching in: {}", path);

    if let Ok(true) = fs::exists(&path) {
        println!("Found model. Deleting...");
        if let Err(e) = fs::remove_dir_all(&path) {
            eprintln!("Failed to delete {}: {}", path, e);
            return Some(FofError::FailedDeletingDirectory(path));
        }
        println!("Successfully deleted {}", path);
        None
    } else {
        eprintln!("Model '{}' not found at '{}'", model_name, path);
        Some(FofError::ModelNotFound(model_name.to_string()))
    }
}

pub fn train_model(model_name: &str, epochen: i32) -> Option<FofError> {
    println!("Training model '{}'", model_name);
    let path = format!("runs/detect/{}", model_name);

    if let Ok(false) = fs::exists(&path) {
        eprintln!("Error: Model '{}' not found ({}).", model_name, path);
        return Some(FofError::ModelNotFound(model_name.to_string()));
    }

    println!(
        "Found model '{}' in '{}'. Starting to train for {} epochs.",
        model_name, path, epochen
    );

    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--train")
        .arg("--model-name")
        .arg(model_name)
        .arg("--epochs")
        .arg(epochen.to_string())
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Training complete.");
            None
        }
        Ok(output) => {
            eprintln!("Python Error: {}", String::from_utf8_lossy(&output.stderr));
            Some(FofError::PythonError)
        }
        Err(e) => {
            eprintln!("Failed to start training process: {}", e);
            Some(FofError::FailedToStartPython)
        }
    }
}

fn remove_communication() {
    if let Ok(true) = fs::exists("Communication") {
        if let Err(e) = fs::remove_dir_all("Communication") {
            eprintln!("Failed to remove Communication directory: {}", e);
        } else {
            println!("Successfully removed Communication directory.");
        }
    }
}

pub fn get_prediction<P>(model_name: &str, screenshot_path: &P) -> Result<Vec<Building>, FofError>
where
    P: AsRef<Path>,
    P: Debug,
    P: Display,
{
    println!(
        "Getting prediction from model '{}' for {:?}.",
        model_name, screenshot_path
    );

    let path = format!("runs/detect/{}", model_name);

    if let Ok(false) = fs::exists(&screenshot_path) {
        eprintln!("Error: No screenshot found in {:?}.", screenshot_path);
        return Err(FofError::FailedReadingFile(screenshot_path.to_string()));
    }

    println!("Found screenshot in {}", screenshot_path);

    if let Ok(false) = fs::exists(&path) {
        eprintln!("Error: Model '{}' not found ({}).", model_name, path);
        return Err(FofError::ModelNotFound(model_name.to_string()));
    }

    println!(
        "Found model '{}' in '{}'. Getting Prediction...",
        model_name, path
    );

    remove_communication();

    println!("Creating temporary communication directory..");
    if let Err(e) = fs::create_dir("Communication") {
        eprintln!("Failed to create Communication dir: {}", e);
        return Err(FofError::Failed(String::from("Failed to created communication directory (shouldnt be possible to be here, since Communication is deleted before this.)")));
    }

    let target = Path::new("Communication/screenshot.png");
    if let Err(e) = fs::copy(screenshot_path, target) {
        eprintln!(
            "Error copying screenshot: {} | {:?} → {:?}",
            e, screenshot_path, target
        );
        remove_communication();
        return Err(FofError::Failed(
            "failed copying image to communication/screenshot.png".to_string(),
        ));
    }

    let output = Command::new("python3")
        .arg("src/image_data.py")
        .arg("--predict")
        .arg("--model-name")
        .arg(model_name)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Prediction complete.");
        }
        Ok(output) => {
            eprintln!("Python Error: {}", String::from_utf8_lossy(&output.stderr));
            remove_communication();
            return Err(FofError::PythonError);
        }
        Err(e) => {
            eprintln!("Failed to start prediction process: {}", e);
            remove_communication();
            return Err(FofError::FailedToStartPython);
        }
    }

    if let Ok(false) = fs::exists("Communication/data.json") {
        eprintln!("Error: data.json not found.");
        remove_communication();
        return Err(FofError::FailedReadingFile(
            "Communication/data.json".to_string(),
        ));
    }

    let file = match File::open("Communication/data.json") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not open data.json: {}", e);
            remove_communication();
            return Err(FofError::FailedReadingFile(
                "Communication/data.json".to_string(),
            ));
        }
    };

    println!("Reading data.json..");
    let reader = BufReader::new(file);

    let buildings: Vec<Building> = match serde_json::from_reader(reader) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("JSON parse error: {}", e);
            remove_communication();
            return Err(FofError::JsonParseError(e.to_string()));
        }
    };

    remove_communication();
    Ok(buildings)
}

pub fn try_all_image_data_wrapper_functions() {
    use crate::image_data_wrapper::*;
    use std::{fs::File, io::Write, path::PathBuf};

    println!("\n==== START TESTING image_data_wrapper ====\n");

    // Setup Testvariablen
    let valid_model = "test_model";
    let invalid_model = "non_existent_model";
    let valid_image = PathBuf::from("example_data/screenshot.png");
    let invalid_image = PathBuf::from("example_data/does_not_exist.png");
    let broken_json_path = PathBuf::from("Communication/data.json");

    // --- Test: get_model_names ---
    match get_model_names() {
        Ok(models) => println!("✅ get_model_names: {:?}", models),
        Err(e) => println!("❌ get_model_names: {:?}", e),
    }

    // --- Test: create_model ---
    println!("\n-- create_model --");
    let _ = delete_model(valid_model); // ensure clean state
    match create_model(valid_model, YoloModel::yolov8n) {
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
