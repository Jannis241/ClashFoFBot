use crate::prelude::*;

mod bot_actions;
mod debug;
mod filter_buildings;
mod image_data_wrapper;
mod prelude;
mod screener;
mod settings_manager;
mod split_image;
mod threading;
mod ui;
mod walls;

#[derive(Clone, Debug, PartialEq)]
pub enum FofError {
    ThreadNotInitialized,
    NoMetricsFoundForModel(String), // Wenn man get rating callt und er keine ratings für das
    // angegebene modell findet
    FailedReadingDirectory(String), // Welches Dir
    FailedReadingFile(String),      // Welches File
    Failed(String),                 // mehr infos in dem string (kann alles mögliche sein)
    DivisionByZero, // wenn man z.B get avg confidence aufruft aber einen leeren Vec an buildings
    ModelAlreadyExists, // wenn man ein model erstellen will, was es schon gibt
    YoloModelNotFound, // Wenn man ein invalies base model angibt und er es nicht findet.
    PythonError(Vec<u8>), // Wenn es in dem python script einen Error gibt
    FailedToStartPython, // Wenn es probleme gibt imaga_data.py zu starten
    ModelNotFound(String), // Wenn man versucht ein Modell zu löschen oder zu trainieren welches es nciht gibt. String
    JsonParseError(String),
    FailedDeletingDirectory(String),
    YamlParseError(String),
    MissingField(String),
    UnsupportedDataset(String),
    IoError(String),
    NoStatsFound(String), // model name
}

impl From<io::Error> for FofError {
    fn from(e: io::Error) -> Self {
        FofError::IoError(e.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct YamlData {
    train: String,
    val: String,
    names: HashMap<u32, String>,
}

fn check_unique_prefixes(class_names: &[String], prefix_len: usize) {
    let mut prefix_map: HashMap<String, Vec<String>> = HashMap::new();

    for name in class_names {
        let prefix = name
            .chars()
            .take(prefix_len)
            .collect::<String>()
            .to_lowercase();
        prefix_map.entry(prefix).or_default().push(name.clone());
    }

    let mut has_conflict = false;
    for (prefix, names) in &prefix_map {
        if names.len() > 1 {
            has_conflict = true;
            println!("❌ Konflikt beim Präfix '{}': {:?}", prefix, names);
        }
    }

    if !has_conflict {
        println!("✅ Alle Präfixe mit Länge {} sind eindeutig.", prefix_len);
    }
}

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

fn main() {
    // conflicts
    // let file_content =
    //     fs::read_to_string("dataset_buildings/data.yaml").expect("Kann Datei nicht lesen");
    //
    // let data: YamlData =
    //     serde_yaml::from_str(&file_content).expect("Fehler beim Parsen der YAML-Datei");
    //
    // let class_names: Vec<String> = data.names.values().cloned().collect();
    //
    // let prefix_length = 3; // Anzahl der Buchstaben, die als Präfix geprüft werden
    // check_unique_prefixes(&class_names, prefix_length);

    // heheha goey
    ui::start_ui();
    // debug::run_tests();
}
