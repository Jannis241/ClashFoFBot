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

fn read_last_metrics(model_name: &String) -> Option<Metrics> {
    let path = format!("runs/detect/{}/results.csv", model_name);
    let file = File::open(&path).ok()?;
    let mut rdr = Reader::from_reader(file);
    let mut last: Option<Metrics> = None;

    for result in rdr.deserialize() {
        if let Ok(row) = result {
            last = Some(row);
        }
    }
    return last;
}

pub fn get_rating(model_name: &String) -> Result<f64, FofError> {
    println!("Searching for metrics of the '{}' model.", model_name);
    let metrics = read_last_metrics(&model_name);

    if let Some(m) = metrics {
        println!("Success: Found Metrics for the '{}' model.", model_name);
        return Ok(calculate_score(&m));
    } else {
        eprintln!("Error: No Metrics found for {}!", model_name);
        return Err(FofError::NoMetricsFoundForModel(model_name));
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
                        return Err(FofError::Failed(String::from(format!(
                            "Failed to get metadata for {:?}: {}",
                            entry.path(),
                            err
                        ))));
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

                            return Err(FofError::Failed(String::from(format!(
                                "⚠️ Invalid UTF-8 in directory name: {:?}",
                                entry.file_name()
                            ))));
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("⚠️ Failed to read a directory entry: {}", err);
                return Err(FofError::Failed(String::from(format!(
                    "⚠️ Failed to read a directory entry: {}",
                    err
                ))));
            }
        }
    }

    Ok(model_names)
}

#[derive(EnumIter, Display, Clone, PartialEq, Eq, Debug)]
pub enum YoloModel {
    YOLOv8n,
    YOLOv8s,
    YOLOv8m, // <-- bestes schnelligkeits / leistungs verhältnis
    YOLOv8l,
    YOLOv8x,
}

pub fn get_avg_confidence(buildings: &Vec<Building>) -> Result<f32, FofError> {
    println!("Calclulating average confidence..");
    if buildings.len() == 0 {
        eprintln!("Error: Es wurden keine Buildings angegeben. Nicht möglich die Confidence zu berechnen.");
        return Err(FofError::DivisionByZero);
    }
    let mut sum = 0.0;

    for building in buildings {
        sum += building.confidence;
    }
    return Ok(sum / buildings.len() as f32);
}

pub fn create_model(model_name: &String, yolo_model: YoloModel) -> Option<FofError> {
    println!(
        "Creating new model '{}' with the '{:?}' yolo base model.",
        model_name, yolo_model
    );

    if fs::exists(format!("runs/detect/{}", model_name)).unwrap() {
        eprintln!(
            "Error: es existiert bereits ein model mit dem namen {}. Breche die Erstellung des Models ab..",
            model_name
        );
        return Some(FofError::ModelAlreadyExists);
    }
    println!("Parse das YOLO base model..");
    let yolo_model_string = match yolo_model {
        YoloModel::YOLOv8n => String::from("yolov8n.pt"),
        YoloModel::YOLOv8s => String::from("yolov8s.pt"),
        YoloModel::YOLOv8m => String::from("yolov8m.pt"),
        YoloModel::YOLOv8l => String::from("yolov8l.pt"),
        YoloModel::YOLOv8x => String::from("yolov8x.pt"),
        _ => {
            eprintln!("Error: Base model '{:?}' not found!", yolo_model);
            return Some(FofError::YoloModelNotFound);
        }
    };
    println!("Found YOLO base model!");

    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--create-model")
        .arg("--base")
        .arg(yolo_model_string.to_string())
        .arg("--model-name")
        .arg(model_name.to_string())
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Python script is done executing.");
            return None; // alles hat richtig funktioniert
        }
        Ok(output) => {
            eprintln!("Python error: {}", String::from_utf8_lossy(&output.stderr));
            return Some(FofError::PythonError);
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            return Some(FofError::FailedToStartPython);
        }
    }
}

pub fn delete_model(model_name: &String) -> Option<FofError> {
    let path = format!("runs/detect/{}", model_name);

    println!("Trying to delete model '{}'..", model_name);
    println!("Searching in: {}", path);

    if fs::exists(&path).unwrap() {
        println!("Found Model! Deleting it now..");
        fs::remove_dir_all(&path);
        println!("Successfully deleted {}", path);
        return None;
    } else {
        eprintln!(
            "Error: Kein Modell mit dem Namen '{}' in 'runs/detect' gefunden. ({})",
            model_name, &path
        );
        return Some(FofError::ModelNotFound(model_name.clone()));
    }
}

pub fn train_model(model_name: &String, epochen: i32) -> Option<FofError> {
    println!("Training model '{}'", model_name);
    println!("Searching..");
    let path = format!("runs/detect/{}", model_name);
    if !fs::exists(&path).unwrap() {
        eprintln!(
            "Error: Es existiert kein model mit dem namen {}. ({} nicht gefunden)",
            model_name, path
        );
        return Some(FofError::ModelNotFound(model_name.clone()));
    }
    println!(
        "Found model '{}' in {}. Starting to train it for {} epochs.",
        model_name, path, epochen
    );
    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--train")
        .arg("--model-name")
        .arg(model_name.to_string())
        .arg("--epochs")
        .arg(epochen.to_string())
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Python script is done executing.");
            return None;
        }
        Ok(output) => {
            eprintln!("Python Error: {}", String::from_utf8_lossy(&output.stderr));
            return Some(FofError::PythonError);
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            return Some(FofError::FailedToStartPython);
        }
    }
}

fn remove_communication() {
    if fs::exists("Communication").unwrap() {
        fs::remove_dir_all("Communication");
        println!("Successfully removed Communication directory.")
    }
}
pub fn get_prediction<P>(
    model_name: &String,
    screeenshot_path: &P,
) -> Result<Vec<Building>, FofError>
where
    P: AsRef<Path>,
    P: Debug,
{
    println!(
        "Getting prediction from model '{}' for {:?}.",
        model_name, screeenshot_path
    );

    remove_communication();

    println!("Creating temporary communication directory..");
    fs::create_dir("Communication").expect("Failed to create Communication dir.");

    let target = Path::new("Communication/screenshot.png");
    let res = fs::copy(screeenshot_path, target);

    match res {
        Ok(_) => {
            println!("Screenshot wurde erfolgreich nach 'Communication/screenshot.png' kopiert!")
        }
        Err(e) => {
            eprintln!(
                "Error: {}  | Tried to copy {:?} to {:?}.",
                &e, &screeenshot_path, target
            );
            remove_communication();
            return Err(FofError::CommunicationError);
        }
    }

    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--predict")
        .arg("--model-name")
        .arg(model_name.to_string())
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Python script is done executing.");
        }
        Ok(output) => {
            eprintln!("Python Error: {}", String::from_utf8_lossy(&output.stderr));
            remove_communication();
            return Err(FofError::PythonError);
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            remove_communication();
            return Err(FofError::FailedToStartPython);
        }
    }

    if !fs::exists("Communication/data.json").unwrap() {
        eprintln!("Error: Communication/data.json nicht gefunden.");
        remove_communication();

        return Err(FofError::FailedReadingFile(String::from(
            "Communication/data.json",
        )));
    }
    let file = File::open("Communication/data.json").expect("Konnte data.json nicht öffnen");
    println!("Reading data.json..");

    let reader = BufReader::new(file);

    let buildings: Vec<Building> =
        serde_json::from_reader(reader).expect("Error while trying to read from data.json.");

    remove_communication();

    return Ok(buildings);
}
