use std::fmt::format;

use crate::prelude::*;

#[derive(Debug, PartialEq, EnumIter, Eq, Clone)]
pub enum YoloModel {
    YOLOv8n,
    YOLOv8s,
    YOLOv8m,
    YOLOv8l,
    YOLOv8x,
}
#[derive(Debug, Clone, PartialEq)]
pub enum DatasetType {
    Buildings,
    Level,
}
impl ToString for YoloModel {
    fn to_string(&self) -> String {
        match self {
            YoloModel::YOLOv8n => {
                return "yolov8n".to_string();
            }
            YoloModel::YOLOv8s => {
                return "yolov8s".to_string();
            }
            YoloModel::YOLOv8m => {
                return "yolov8m".to_string();
            }
            YoloModel::YOLOv8l => {
                return "yolov8l".to_string();
            }
            YoloModel::YOLOv8x => {
                return "yolov8x".to_string();
            }
        }
    }
}

impl ToString for DatasetType {
    fn to_string(&self) -> String {
        match self {
            DatasetType::Buildings => {
                return "Buildings".to_string();
            }
            DatasetType::Level => {
                return "Level".to_string();
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Building {
    pub class_id: i32,
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: (f32, f32, f32, f32),
}

#[derive(Clone, Debug)]
pub struct Model {
    pub name: String,
    pub rating: f64,
    pub dataset_type: DatasetType,
}

#[derive(Deserialize)]
struct ArgsYaml {
    data: Option<String>,
}

impl Model {
    pub fn new(name: String, rating: f64, dataset_type: DatasetType) -> Self {
        Model {
            name,
            rating,
            dataset_type,
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

fn get_rating(model_name: &str) -> Result<f64, FofError> {
    let metrics = read_last_metrics(model_name);

    if let Some(m) = metrics {
        return Ok(calculate_score(&m));
    }

    return Err(FofError::NoMetricsFoundForModel(model_name.to_string()));
}

fn start_python(args: Vec<&str>) -> Result<String, FofError> {
    match Command::new("python3").args(args).output() {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        Ok(output) => Err(FofError::PythonError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )),
        Err(e) => Err(FofError::FailedToStartPython),
    }
}

pub fn get_dataset_type(name: &str) -> Result<DatasetType, FofError> {
    let path = format!("runs/detect/{}/args.yaml", name);
    let file = File::open(&path).map_err(|_| FofError::FailedReadingFile(path.clone()))?;

    let args: ArgsYaml =
        serde_yaml::from_reader(file).map_err(|e| FofError::YamlParseError(e.to_string()))?;

    let data_field = args
        .data
        .ok_or_else(|| FofError::MissingField(format!("'data' field missing in {}", path)))?;

    if data_field.starts_with("dataset_buildings/") {
        Ok(DatasetType::Buildings)
    } else if data_field.starts_with("dataset_level/") {
        Ok(DatasetType::Level)
    } else {
        Err(FofError::UnsupportedDataset(data_field))
    }
}

pub fn get_testvals(model_name: String) -> Result<(), FofError> {
    let dataset_type = get_dataset_type(model_name.as_str())?.to_string();
    let args = vec![
        "src/image_data.py",
        "--testvals",
        "--model-name",
        model_name.as_str(),
        "--dataset_type",
        dataset_type.as_str(),
    ];
    let python_output = start_python(args)?;

    println!("Python output: {}", python_output);

    Ok(())
}

pub fn get_all_models() -> Result<Vec<Model>, FofError> {
    let mut models = Vec::new();
    let path = Path::new("runs/detect");

    let entries = fs::read_dir(path)
        .map_err(|_| FofError::FailedReadingDirectory(path.display().to_string()))?;

    for entry_result in entries {
        let entry = entry_result
            .map_err(|_| FofError::Failed("Failed reading entry results".to_string()))?;

        let metadata = entry
            .metadata()
            .map_err(|_| FofError::Failed("Failed reading metadata".to_string()))?;

        if metadata.is_dir() {
            let filename = entry.file_name();

            let filename = filename.to_str().ok_or(FofError::FailedReadingFile(
                "Failed reading file name".to_string(),
            ))?;

            let rating = get_rating(filename)?;
            let dataset_type = get_dataset_type(filename)?;
            let model = Model::new(filename.to_string(), rating, dataset_type);
            models.push(model);
        }
    }

    Ok(models)
}

pub fn get_avg_confidence(buildings: &[Building]) -> f32 {
    if buildings.is_empty() {
        return 0.0;
    }

    let sum: f32 = buildings.iter().map(|b| b.confidence).sum();
    sum / buildings.len() as f32
}

pub fn create_model(
    model_name: &str,
    dataset_type: DatasetType,
    yolo_model: YoloModel,
) -> Result<(), FofError> {
    let model_path = format!("runs/detect/{}", model_name);

    if check_if_exists(&model_path)? {
        return Err(FofError::ModelAlreadyExists);
    }

    let dataset_type = dataset_type.to_string();
    let yolo_model_string = yolo_model.to_string();

    let args = vec![
        "src/image_data.py",
        "--create-model",
        "--base",
        yolo_model_string.as_str(),
        "--model-name",
        model_name,
        "--dataset_type",
        dataset_type.as_str(),
    ];

    let python_output = start_python(args);
    println!("Python output: {:?}", python_output);

    return Ok(());
}

pub fn delete_model(model_name: &str) -> Result<(), FofError> {
    let model_path = format!("runs/detect/{}", model_name);

    if !check_if_exists(&model_path)? {
        return Err(FofError::ModelNotFound(model_name.to_string()));
    }

    fs::remove_dir_all(&model_path).map_err(|_| FofError::FailedDeletingDirectory(model_path))?;
    Ok(())
}

pub fn start_training(model_name: &str, epochen: i32) -> Result<Child, FofError> {
    let dataset_type = get_dataset_type(model_name)?.to_string();

    let model_path = format!("runs/detect/{}", model_name);
    fs::metadata(&model_path).map_err(|_| FofError::ModelNotFound(model_name.to_string()))?;

    let child = Command::new("python3")
        .arg("src/image_data.py")
        .arg("--train")
        .arg("--model-name")
        .arg(model_name)
        .arg("--epochs")
        .arg(epochen.to_string())
        .arg("--dataset_type")
        .arg(dataset_type)
        .spawn()
        .map_err(|e| {
            eprintln!("Fehler beim Starten des Trainingsprozesses: {}", e);
            FofError::FailedToStartPython
        })?;

    Ok(child)
}

pub fn stop_training(child: &mut Child) -> Result<(), FofError> {
    child.kill().map_err(|_| FofError::FailedToStopTraining)?;
    child.wait().map_err(|_| FofError::FailedToStopTraining)?;
    Ok(())
}

fn check_if_exists<P: Debug + AsRef<Path> + Display>(path: &P) -> Result<bool, FofError> {
    let exists = fs::exists(&path).map_err(|_| FofError::FailedReadingFile(path.to_string()))?;
    Ok(exists)
}

fn create_communication(model_name: &str) -> Result<(), FofError> {
    let path = format!("Communication/{}", model_name);
    fs::create_dir(path).map_err(|e| FofError::FailedCreatingCommunication(e.to_string()))?;
    Ok(())
}

fn remove_communication(model_name: &str) -> Result<(), FofError> {
    let comm_path = format!("Communication/{}", model_name);

    if !check_if_exists(&comm_path)? {
        return Err(FofError::FailedReadingDirectory(comm_path.to_string()));
    }

    fs::remove_dir_all(&comm_path)
        .map_err(|_| FofError::FailedDeletingDirectory(comm_path.to_string()))?;
    Ok(())
}

pub fn get_prediction<P>(model_name: &str, screenshot_path: &P) -> Result<Vec<Building>, FofError>
where
    P: AsRef<Path> + Debug + Display,
{
    let model_path = format!("runs/detect/{}", model_name);
    if !check_if_exists(&model_path)? {
        return Err(FofError::ModelNotFound(model_path.to_string()));
    }

    if !check_if_exists(&screenshot_path)? {
        return Err(FofError::FailedReadingFile(screenshot_path.to_string()));
    }

    remove_communication(model_name);
    create_communication(model_name);

    let target_screenshot_path = format!("Communication/{}/screenshot.png", model_name);

    fs::copy(screenshot_path, target_screenshot_path).map_err(|e| {
        FofError::FailedToCopyData(String::from(
            "Failed to copy screenshot to Communication directory.",
        ))
    })?;

    dbg!("Starte jetzt python");
    let args = vec!["src/image_data.py", "--predict", "--model-name", model_name];
    let python_output = start_python(args)?;

    let data_path = format!("Communication/{}/data.json", model_name);
    if !check_if_exists(&data_path)? {
        return Err(FofError::FailedReadingFile(
            format!("data.json in {} nicht gefunden", data_path).to_string(),
        ));
    }

    let file = File::open(data_path).map_err(|e| FofError::FailedReadingFile(e.to_string()))?;

    let reader = BufReader::new(file);

    let buildings: Vec<Building> =
        serde_json::from_reader(reader).map_err(|e| FofError::FailedReadingFile(e.to_string()))?;

    Ok(buildings)
}
