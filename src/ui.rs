use crate::prelude::*;
macro_rules! generate_keycode_match {
    ( $key:expr, $( $name:ident ),* ) => {{
        use device_query::Keycode::*;
        match $key.to_uppercase().as_str() {
            $(
                stringify!($name) => Some($name),
            )*
            _ => None,
        }
    }};
}

fn keycode_from_str(key: &str) -> Option<Keycode> {
    generate_keycode_match!(
        key, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, F1, F2,
        F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7,
        Key8, Key9, Escape, Space, Enter, Backspace, LShift, RShift
    )
}

pub struct ScreenshotApp {
    pub screenshot_path: String,
    pub keybind: String,
    pub selected_image: Option<String>,
    pub image_folder: Option<PathBuf>,
    pub available_images: Vec<String>,

    // Neu:
    image_texture: Option<egui::TextureHandle>,
    labeled_rects: Vec<LabeledRect>,
    current_rect_start: Option<egui::Pos2>,
    current_rect_end: Option<egui::Pos2>,
    current_label: String,
}

#[derive(Clone)]
struct LabeledRect {
    rect: egui::Rect,
    label: String,
}

impl Default for ScreenshotApp {
    fn default() -> Self {
        Self {
            screenshot_path: "screenshot.png".to_string(),
            keybind: "r".to_string(),
            selected_image: None,
            image_folder: None,
            available_images: vec![],
            image_texture: None,
            current_label: "".to_string(),
            current_rect_end: None,
            current_rect_start: None,
            labeled_rects: vec![],
        }
    }
}

impl ScreenshotApp {
    fn update_image_list(&mut self) {
        if let Some(folder) = &self.image_folder {
            if let Ok(entries) = fs::read_dir(folder) {
                self.available_images = entries
                    .filter_map(|entry| entry.ok())
                    .filter_map(|e| {
                        let path = e.path();
                        if path.extension()?.to_str()? == "png" {
                            Some(path.display().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
            }
        }
    }
}

impl eframe::App for ScreenshotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style: egui::Style = (*ctx.style()).clone();
        let size = 2.;
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(28.0 * size, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(18.0 * size, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(18.0 * size, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(16.0 * size, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(14.0 * size, egui::FontFamily::Proportional),
            ),
        ]
        .into();

        ctx.set_style(style);

        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.collapsing("📸 Keybinds", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Screenshot-Key:");
                    ui.text_edit_singleline(&mut self.keybind);
                });

                ui.horizontal(|ui| {
                    ui.label("Speicherpfad:");
                    ui.text_edit_singleline(&mut self.screenshot_path);
                });

                let state = DeviceState::new();

                if state
                    .query_keymap()
                    .contains(&keycode_from_str(&self.keybind).unwrap_or(Keycode::V))
                {
                    println!("took screenshot");
                    let img = screener::make_screenshot(0);
                    img.save(&self.screenshot_path)
                        .expect("error while saving img");
                }
            });

            ui.collapsing("🖼️ Labeln", |ui| {
                if ui.button("📂 Ordner wählen").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.image_folder = Some(path.clone());
                        self.update_image_list();
                    }
                }

                if let Some(folder) = &self.image_folder {
                    ui.label(format!("📁 Ordner: {}", folder.display()));
                }

                for img in &self.available_images {
                    if ui
                        .selectable_label(self.selected_image.as_deref() == Some(img), img)
                        .clicked()
                    {
                        self.selected_image = Some(img.clone());
                    }
                }
            });
        });
    }
}
