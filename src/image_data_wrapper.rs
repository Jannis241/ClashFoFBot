use crate::prelude::*;

pub struct BoundingBox {
    pub top_left: (f32, f32),
    pub top_right: (f32, f32),
}

pub struct Building {
    pub class_id: i32,
    pub class_name: i32,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
}

pub fn get_buildings(screeenshot_path: &Path) -> Vec<Building> {
    // Todo:
    // 1. copy the screenshot to communications / screenshot damit das python script das image
    // auslesen kann.
    // 2. python script callen
    // 3. python script soll die ergebnisse in data.json speichern
    // 4. ergebnisse hier auslesen5
    // 5. geparste ergebnisse (json zu Vec<Buildings> returnen)
    let target = Path::new("Communication/screenshot.png");

    let res = fs::copy(screeenshot_path, target);

    match res {
        Ok(_) => println!("Datei wurde erfolgreich kopiert!"),
        Err(e) => println!(
            "Error while trying to copy {:?} to {:?}.",
            screeenshot_path, target
        ),
    }

    return vec![];
}
