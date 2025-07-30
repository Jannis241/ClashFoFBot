use crate::prelude::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Building {
    pub class_id: i32,
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: (f32, f32, f32, f32),
}

pub fn get_avg_confidence(buildings: &Vec<Building>) -> f32 {
    let mut sum = 0.0;

    for building in buildings {
        sum += building.confidence;
    }
    return sum / buildings.len() as f32;
}

pub fn create_model(model_name: String, yolo_model: String) {
    println!("Creating model");
    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--create-model")
        .arg("--base")
        .arg(yolo_model.to_string())
        .arg("--model-name")
        .arg(model_name.to_string())
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("image_data.py finished executing..");
        }
        Ok(output) => {
            eprintln!("Python error:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
        }
    }
}

pub fn delete_model(model_name: String) {
    println!("deleting model '{}'..", model_name);
    let path = format!("runs/detect/{}", model_name);

    if fs::exists(&path).unwrap() {
        println!("Found Model! Deleting it now..");
        fs::remove_dir_all(&path);
        println!("Successfully deleted {}", path);
    } else {
        eprintln!("Error: Did not found {}", &path);
    }
}

pub fn train_model(model_name: String, epochen: i32) {
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
        }
        Ok(output) => {
            eprintln!("Python error:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
        }
    }
}

pub fn get_buildings(model_name: String, screeenshot_path: &Path) -> Vec<Building> {
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
        Err(e) => eprintln!(
            "Error: {}  | Tried to copy {:?} to {:?}.",
            e, screeenshot_path, target
        ),
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
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
        }
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

    return buildings;
}
