use crate::prelude::*;
mod bot_actions;
mod data_augmentation;
mod debug;
mod image_data_wrapper;
mod prelude;
mod screener;
mod settings_manager;
mod threading;
mod ui;

#[derive(Debug, PartialEq)]
pub enum FofError {
    NoMetricsFoundForModel(String), // Wenn man get rating callt und er keine ratings für das
    // angegebene modell findet
    FailedReadingDirectory(String),      // Welches Dir
    FailedReadingFile(String),           // Welches File
    Failed(String),                      // mehr infos in dem string (kann alles mögliche sein)
    DivisionByZero, // wenn man z.B get avg confidence aufruft aber einen leeren Vec an buildings
    ModelAlreadyExists, // wenn man ein model erstellen will, was es schon gibt
    YoloModelNotFound, // Wenn man ein invalies base model angibt und er es nicht findet.
    PythonError(Vec<u8>), // Wenn es in dem python script einen Error gibt
    FailedToStartPython(std::io::Error), // Wenn es probleme gibt imaga_data.py zu starten
    ModelNotFound(String), // Wenn man versucht ein Modell zu löschen oder zu trainieren welches es nciht gibt. String
    JsonParseError(String),
    FailedDeletingDirectory(String),
}

fn main() {
    // ui::start_ui();
    debug::run_tests();
}
