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
    let mut path = format!("runs/detect/{}/results.csv", model_name);
    println!("Getting metrics from {}", path);
    let file = File::open(&path).ok()?;
    let mut rdr = Reader::from_reader(file);
    let mut last: Option<Metrics> = None;
    println!("Reader: {:?}", rdr);

    for result in rdr.deserialize() {
        println!("Result from rdr deserialze: {:?}", &result);
        if let Ok(row) = result {
            last = Some(row);
        }
    }

    println!("last metrics: {:?}", last);

    last
}

pub fn get_rating(model_name: String) -> f64 {
    let metrics = read_last_metrics(&model_name);
    if let Some(m) = metrics {
        return calculate_score(&m);
    } else {
        eprintln!("Error: Keine Metrics für {} gefunden!", model_name);
        return 0.0;
    }
}

pub fn get_model_names() -> Vec<String> {
    vec![]
}

pub enum YoloModel {
    yolov8n,
    YOLOv8s,
    YOLOv8m, // <-- bestes schnelligkeits / leistungs verhältnis
    YOLOv8l,
    YOLOv8x,
}

pub fn get_avg_confidence(buildings: &Vec<Building>) -> f32 {
    let mut sum = 0.0;

    for building in buildings {
        sum += building.confidence;
    }
    return sum / buildings.len() as f32;
}

pub fn create_model(model_name: String, yolo_model: YoloModel) -> bool {
    println!("Creating model");
    if fs::exists(format!("runs/detect/{}", model_name)).unwrap() {
        eprintln!(
            "Es existiert bereits ein model mit dem namen {}. Breche Erstellung ab.",
            model_name
        );
        return false;
    }
    println!("Parse yolo base model..");
    let yolo_model_string = match yolo_model {
        YoloModel::yolov8n => String::from("yolov8n.pt"),
        YoloModel::YOLOv8s => String::from("yolov8s.pt"),
        YoloModel::YOLOv8m => String::from("yolov8m.pt"),
        YoloModel::YOLOv8l => String::from("yolov8l.pt"),
        YoloModel::YOLOv8x => String::from("yolov8x.pt"),
        _ => {
            eprintln!("Error while parsing yolo model in image data wrapper.rs. Model not found!");
            return false;
        }
    };
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
            println!("image_data.py finished executing..");
            return true;
        }
        Ok(output) => {
            eprintln!("Python error:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return false;
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            return false;
        }
    }
}

pub fn delete_model(model_name: String) -> bool {
    println!("deleting model '{}'..", model_name);
    let path = format!("runs/detect/{}", model_name);

    if fs::exists(&path).unwrap() {
        println!("Found Model! Deleting it now..");
        fs::remove_dir_all(&path);
        println!("Successfully deleted {}", path);
        return true;
    } else {
        eprintln!("Error: Did not found {}", &path);
        println!("Failed!");
        return false;
    }
}

pub fn train_model(model_name: String, epochen: i32) -> bool {
    if !fs::exists(format!("runs/detect/{}", model_name)).unwrap() {
        eprintln!("Es existiert kein model mit dem namen {}", model_name);
        return false;
    }
    println!("Training model..");
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
            println!("image_data.py finished executing..");
            return true;
        }
        Ok(output) => {
            eprintln!("Python error:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return false;
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            return false;
        }
    }
}

pub fn get_buildings(model_name: String, screeenshot_path: &Path) -> (Vec<Building>, bool) {
    println!("Bekomme Prediction von {}", model_name);
    if fs::exists("Communication").unwrap() {
        fs::remove_dir_all("Communication");
    }
    fs::create_dir("Communication").expect("Failed to create Communication dir.");
    println!("Creating Communication directory..");

    let target = Path::new("Communication/screenshot.png");

    let res = fs::copy(screeenshot_path, target);

    match res {
        Ok(_) => println!("Datei wurde erfolgreich kopiert!"),
        Err(e) => {
            eprintln!(
                "Error: {}  | Tried to copy {:?} to {:?}.",
                e, screeenshot_path, target
            );
            return (vec![], false);
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
            println!("image_data.py finished executing.");
        }
        Ok(output) => {
            eprintln!("Python error:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            if fs::exists("Communication").unwrap() {
                fs::remove_dir_all("Communication");
            }
            return (vec![], false);
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            if fs::exists("Communication").unwrap() {
                fs::remove_dir_all("Communication");
            }
            return (vec![], false);
        }
    }

    if !fs::exists("Communication/data.json").unwrap() {
        eprintln!("data.json nicht gefunden.");
        if fs::exists("Communication").unwrap() {
            fs::remove_dir_all("Communication");
        }
        return (vec![], false);
    }
    let file = File::open("Communication/data.json").expect("Konnte data.json nicht öffnen");
    println!("Reading data.json..");

    let reader = BufReader::new(file);

    let buildings: Vec<Building> =
        serde_json::from_reader(reader).expect("Error while trying to read from data.json.");

    fs::remove_file(Path::new("Communication/screenshot.png"))
        .expect("Error while removing screenshot.png after model analyis. Something went wrong.");
    fs::remove_file(Path::new("Communication/data.json"))
        .expect("Error while removing data.json after model analyis. Something went wrong.");

    fs::remove_dir_all("Communication").expect("failed to remove Communication dir");
    println!("Removed temp Communication directory.");

    return (buildings, true);
}
