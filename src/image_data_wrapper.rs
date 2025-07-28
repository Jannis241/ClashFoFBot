use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct Building {
    pub class_id: i32,
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: (f32, f32, f32, f32),
}

pub fn train_model(epochen: i32) {
    println!("Training model..");
    match Command::new("python3")
        .arg("src/image_data.py")
        .arg("--continue-train")
        .arg("--epochs")
        .arg(epochen.to_string())
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("image_data.py executed without any problems.");
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

pub fn get_buildings(screeenshot_path: &Path) -> Vec<Building> {
    println!("Getting prediction from best model");
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
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("image_data.py executed without any problems.");
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

    let reader = BufReader::new(file);

    let buildings: Vec<Building> =
        serde_json::from_reader(reader).expect("Error while trying to read from data.json.");

    return buildings;
}
