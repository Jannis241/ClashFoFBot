use strum_macros::AsRefStr;
use time::OffsetDateTime;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, AsRefStr)]
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

pub struct TrainingStats {
    pub trained_epochen: i32,
    pub start_rating: f64,
    pub avg_time_per_epoche: OffsetDateTime,
    pub avg_improvement_per_epoche: f64,
    pub avg_improvement_per_hour: f64,
    pub current_rating: f64,
    pub rating_improvement: f64,
    pub start_time: OffsetDateTime,
    pub current_time: OffsetDateTime,
    pub trainings_dauer: OffsetDateTime,
}

pub struct ModelStats {
    pub num_of_finished_trainings: i32,
    pub overall_training_time: OffsetDateTime,
    pub num_of_trained_epochen: i32,
    pub current_rating: i32,
    pub avg_rating_improvement_per_trainign: f64,
    pub avg_rating_improvement_per_hour: f64,
    pub avg_rating_improvement_per_epoche: f64,
    pub avg_time_per_epoche: OffsetDateTime,
}

impl ModelStats {
    fn new(
        num_of_finished_trainings: i32,
        overall_training_time: OffsetDateTime,
        num_of_trained_epochen: i32,
        current_rating: i32,
        avg_rating_improvement_per_trainign: f64,
        avg_rating_improvement_per_hour: f64,
        avg_rating_improvement_per_epoche: f64,
        avg_time_per_epoche: OffsetDateTime,
    ) -> Self {
        Self {
            num_of_finished_trainings,
            overall_training_time,
            num_of_trained_epochen,
            current_rating,
            avg_rating_improvement_per_trainign,
            avg_rating_improvement_per_hour,
            avg_rating_improvement_per_epoche,
            avg_time_per_epoche,
        }
    }
}

impl TrainingStats {
    fn new(
        trained_epochen: i32,
        start_rating: f64,
        avg_time_per_epoche: OffsetDateTime,
        avg_improvement_per_epoche: f64,
        avg_improvement_per_hour: f64,
        current_rating: f64,
        rating_improvement: f64,
        start_time: OffsetDateTime,
        current_time: OffsetDateTime,
        trainings_dauer: OffsetDateTime,
    ) -> Self {
        Self {
            trained_epochen,
            start_rating,
            avg_time_per_epoche,
            avg_improvement_per_epoche,
            avg_improvement_per_hour,
            current_rating,
            rating_improvement,
            start_time,
            current_time,
            trainings_dauer,
        }
    }
}

// pub fn get_model_stats(model_name: &str) -> ModelStats {}
//
// pub fn get_training_stats(model_name: &str) -> TrainingStats {}

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
    println!("Trying to read last metrics from {}", path);
    let file = File::open(&path).ok()?;
    let mut rdr = Reader::from_reader(file);
    let mut last: Option<Metrics> = None;

    for result in rdr.deserialize() {
        if let Ok(row) = result {
            last = Some(row);
        }
    }
    println!("Found metrics: {:?} in {}", last, path);
    last
}

fn get_rating(model_name: &str) -> Result<f64, FofError> {
    println!("Searching for metrics of the '{}' model.", model_name);
    if let Some(m) = read_last_metrics(model_name) {
        println!("Success: Found Metrics for the '{}' model.", model_name);
        Ok(calculate_score(&m))
    } else {
        eprintln!(
            "Error: No Metrics found for {}! (Model wurde wahrscheinlich nicht korrekt erstellt -> evtl. python error beim erstellen -> results.csv fehlt.)",
            model_name
        );
        return Err(FofError::NoMetricsFoundForModel(
            model_name.clone().to_string(),
        ));
    }
}

fn get_dataset_type(name: &str) -> Result<DatasetType, FofError> {
    let path = format!("runs/detect/{}/args.yaml", name);
    println!("Searching for {}", path);
    let file = File::open(&path).map_err(|_| FofError::FailedReadingFile(path.clone()))?;

    let args: ArgsYaml =
        serde_yaml::from_reader(file).map_err(|e| FofError::YamlParseError(e.to_string()))?;

    let data_field = args
        .data
        .ok_or_else(|| FofError::MissingField(format!("'data' field missing in {}", path)))?;

    if data_field.starts_with("dataset_buildings/") {
        println!("Found dataset_buildings for {}", name);
        Ok(DatasetType::Buildings)
    } else if data_field.starts_with("dataset_level/") {
        println!("Found dataset_level for {}", name);
        Ok(DatasetType::Level)
    } else {
        eprintln!("Error: wasnt able to figure data set for {} out.", name);
        Err(FofError::UnsupportedDataset(data_field))
    }
}

pub fn get_all_models() -> Result<Vec<Model>, FofError> {
    println!("Searching for all models.");

    let mut models = Vec::new();
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
                            let rating = match get_rating(name) {
                                Ok(r) => r,
                                Err(fof_error) => return Err(fof_error),
                            };
                            let dataset_type = match get_dataset_type(name) {
                                Ok(r) => r,
                                Err(fof_error) => return Err(fof_error),
                            };
                            println!(
                                "Sucessfully got rating and dataset_type for model '{}'.",
                                name
                            );

                            let m = Model::new(name.to_string(), rating, dataset_type);
                            models.push(m);
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

    println!("Sucessfully found {} models.", models.len());

    Ok(models)
}

#[derive(Debug, PartialEq, EnumIter, Display, Eq, Clone)]
pub enum YoloModel {
    YOLOv8n,
    YOLOv8s,
    YOLOv8m,
    YOLOv8l,
    YOLOv8x,
}

pub fn get_avg_confidence(buildings: &[Building]) -> f32 {
    println!("Calculating average confidence..");
    if buildings.is_empty() {
        eprintln!("Error: Es wurden keine Buildings angegeben um die average confidence zu berechnen. Returne 0 für avg confidence.");
        return 0.0;
    }

    let sum: f32 = buildings.iter().map(|b| b.confidence).sum();
    sum / buildings.len() as f32
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

            let stats_dir = format!("Stats/{}", model_name);
            if let Ok(true) = fs::exists(&stats_dir) {
                eprintln!("Error: Tried to initialise Stats directory in '{}' for model '{}' but it already exists. The model probably didnt get removed correctly. Aborting..", stats_dir, model_name);
                return Some(FofError::Failed(format!("Es wurden bereits initialisierte Stats für das Model '{}' gefunden. Kann keine neuen erstellen. Ein vorheriges Model mit diesem namen wurde wahrscheinlich nicht korrekt entfernt", model_name)));
            }
            if let Err(e) = fs::create_dir(&stats_dir) {
                eprintln!("Error while trying to create stats dir.");
                return Some(FofError::Failed(e.to_string()));
            }
            println!(
                "Sucessfully initialised Stats directory for model '{}' in {}.",
                model_name, &stats_dir
            );

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

        let stats_dir = format!("Stats/{}", model_name);
        if let Ok(false) = fs::exists(&stats_dir) {
            eprintln!(
                "No stats found for model '{}' to delete. Still trying to continue..",
                model_name
            );
        } else {
            let r = fs::remove_dir_all(stats_dir);
            match r {
                Ok(o) => println!("Successfully removed Stats for model '{}'.", model_name),
                Err(e) => {
                    println!(
                        "Error while trying to delete stats dir for model '{}': {}",
                        model_name, e
                    );

                    return Some(FofError::Failed(e.to_string()));
                }
            }
        }
        None
    } else {
        eprintln!("Model '{}' not found at '{}'", model_name, path);
        Some(FofError::ModelNotFound(model_name.to_string()))
    }
}

pub fn train_model(model_name: &str, epochen: i32) -> Option<FofError> {
    println!("Training model '{}'", model_name);
    let dataset_type = match get_dataset_type(model_name) {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "Error while trying to figure dataset of '{}' out: {:?}",
                model_name, e
            );
            return Some(e);
        }
    };

    let dataset_type = match dataset_type {
        DatasetType::Level => "level",
        DatasetType::Buildings => "buildings",
    };

    println!(
        "Training: found out dataset for '{}' : {:?}",
        model_name, dataset_type
    );

    let path = format!("runs/detect/{}", model_name);

    if let Ok(false) = fs::exists(&path) {
        eprintln!("Error: Model '{}' not found ({}).", model_name, path);
        return Some(FofError::ModelNotFound(model_name.to_string()));
    }

    println!(
        "Found model '{}' in '{}'. Starting to train for {} epochs.",
        model_name, path, epochen
    );

    println!("Searching for stats..");

    let stats_path = format!("Stats/{}", model_name);

    if let Ok(false) = fs::exists(&stats_path) {
        eprintln!(
            "Error: Keine initialisierten Stats für '{}' in {} gefunden . (Model wurde wahrscheinlich nicht korrekt erstellt.)",
            model_name,
            stats_path,
        );
        return Some(FofError::NoStatsFound(model_name.to_string()));
    }

    println!("Found Stats for model '{}' in {}", model_name, stats_path);

    let start_time = time::OffsetDateTime::now_utc();

    let start_rating = get_rating(model_name);

    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--train")
        .arg("--model-name")
        .arg(model_name)
        .arg("--epochs")
        .arg(epochen.to_string())
        .arg("--dataset_type")
        .arg(dataset_type.to_string())
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Training complete.");

            let end_time = time::OffsetDateTime::now_utc();

            let training_time = end_time - start_time;

            let num_of_epochs = epochen;
            let end_rating = get_rating(model_name);

            // training muss in stats hinzugefügt werdne
            //
            //
            //
            //
            //

            println!("\n📊 Training Stats:");
            println!("─────────────────────────────");

            println!("🔹 Startzeit         : {}", start_time);
            println!("🔹 Endzeit           : {}", end_time);

            let seconds = training_time.whole_seconds();
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            let secs = seconds % 60;

            println!(
                "🕒 Trainingsdauer    : {:02}h {:02}m {:02}s",
                hours, minutes, secs
            );

            println!(
                "📈 Start-Rating      : {:.2}",
                start_rating.clone().unwrap()
            );
            println!("📉 End-Rating        : {:.2}", end_rating.clone().unwrap());

            let rating_improvement = end_rating.clone().unwrap() - start_rating.clone().unwrap();
            let avg_rating_per_epoche = rating_improvement / epochen as f64;
            let training_hours = training_time.whole_seconds() as f64 / 3600.0;
            let avg_rating_per_hour = if training_hours > 0.0 {
                rating_improvement / training_hours
            } else {
                0.0
            };

            println!("➕ Verbesserung       : {:.2}", rating_improvement);
            println!("📊 Ø Rating/Epoche   : {:.4}", avg_rating_per_epoche);
            println!("⚡ Ø Rating/Stunde   : {:.4}", avg_rating_per_hour);
            println!("🔁 Epochen           : {}", epochen);
            println!("─────────────────────────────");

            // epochen, start

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

    println!("Model path: {}", &path);

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

    println!("Pre python checks alle valid");
    println!("Starting python script to get prediction from the model..");

    let output = Command::new("python3")
        .arg("src/image_data.py")
        .arg("--predict")
        .arg("--model-name")
        .arg(model_name)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Prediction complete. Keine Probleme bei python.");
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

    println!("data.json gefunden.");

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

    println!("json reader: {:?}", reader);

    let buildings: Vec<Building> = match serde_json::from_reader(reader) {
        Ok(data) => {
            println!("Got json data: {:?}", &data);
            data
        }
        Err(e) => {
            eprintln!("JSON parse error: {}", e);
            remove_communication();
            return Err(FofError::JsonParseError(e.to_string()));
        }
    };

    remove_communication();
    println!("Sucessfully got prediction: {:?}", &buildings);
    Ok(buildings)
}
