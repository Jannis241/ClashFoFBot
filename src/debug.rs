use crate::prelude::*;
pub fn run_tests() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Screenshot Tool",
        options,
        Box::new(|_cc| Ok(Box::new(ui::ScreenshotApp::default()))),
    );
}
