use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum DatasetType {
    Buildings,
    Level,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Building {
    pub class_id: i32,
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: (f32, f32, f32, f32),
}

// funktioniert nicht weil man das immer erst von git runter laden muss und dann in den path packen
pub fn read_number(image_path: &String) -> Result<i32, FofError> {
    if let Ok(false) = fs::exists(image_path) {
        eprintln!("Image path '{}' nicht gefunden.", image_path);
        return Err(FofError::FailedReadingFile(image_path.clone()));
    }

    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--zahl_erkennen")
        .arg("--path")
        .arg(image_path)
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Python script executed.");
            Ok(0)
        }
        Ok(output) => {
            eprintln!("Python error: {}", String::from_utf8_lossy(&output.stderr));
            Err(FofError::PythonError(output.stderr))
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            Err(FofError::FailedToStartPython)
        }
    }
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

#[derive(Debug, PartialEq, EnumIter, Display, Eq, Clone)]
pub enum YoloModel {
    YOLOv8n,
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

pub fn create_model(
    model_name: &str,
    dataset_type: DatasetType,
    yolo_model: YoloModel,
) -> Option<FofError> {
    println!(
        "Creating new model '{}' with the '{:?}' yolo base model.",
        model_name, yolo_model
    );

    if let Ok(true) = fs::exists(format!("runs/detect/{}", model_name)) {
        eprintln!("Error: Model '{}' already exists. Aborting..", model_name);
        return Some(FofError::ModelAlreadyExists);
    }

    let t = match dataset_type {
        DatasetType::Level => "level",
        DatasetType::Buildings => "buildings",
    };

    println!("Parsing YOLO base model..");
    let yolo_model_string = match yolo_model {
        YoloModel::YOLOv8n => "yolov8n.pt",
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
        .arg("--dataset_type")
        .arg(t)
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Python script executed.");
            None
        }
        Ok(output) => {
            eprintln!("Python error: {}", String::from_utf8_lossy(&output.stderr));
            Some(FofError::PythonError(output.stderr))
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

pub fn train_model(model_name: &str, dataset_type: DatasetType, epochen: i32) -> Option<FofError> {
    println!("Training model '{}'", model_name);
    let t = match dataset_type {
        DatasetType::Level => "level",
        DatasetType::Buildings => "buildings",
    };
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
        .arg("--dataset_type")
        .arg(t)
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Training complete.");
            None
        }
        Ok(output) => {
            eprintln!("Python Error: {}", String::from_utf8_lossy(&output.stderr));
            Some(FofError::PythonError(output.stderr))
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
            eprintln!("Logs: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("Python Error: {}", String::from_utf8_lossy(&output.stderr));
            remove_communication();
            return Err(FofError::PythonError(output.stderr));
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
