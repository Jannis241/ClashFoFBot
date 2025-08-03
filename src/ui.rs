use crate::{prelude::*, threading::WorkerHandle};
use eframe::egui::{Pos2, Vec2};
use egui::{vec2, Rect};

pub fn start_ui() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Screenshot Tool",
        options,
        Box::new(|_cc| Ok(Box::new(ui::ScreenshotApp::default()))),
    );
}

#[derive(PartialEq, Clone, Copy)]
pub enum MessageType {
    Success,
    Warning,
    Error,
}

pub struct UiMessage {
    pub message: String,
    pub kind: MessageType,
    pub created: std::time::Instant,
}

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
#[derive(PartialEq)]
enum Tab {
    Settings,
    YoloLabel,
    Model,
    Split,
}

use crate::threading::*;
use std::any::Any;
use std::i128;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
enum TrainStatus {
    Idle,
    Running,
    Done(Option<FofError>),
}

struct TrainThread {
    model_name: String,
    status: Arc<Mutex<TrainStatus>>,
    request_start: bool,
}

impl AutoThread for TrainThread {
    fn run(&mut self) {
        let mut status = self.status.lock().unwrap();

        if matches!(*status, TrainStatus::Idle | TrainStatus::Done(_)) {
            let model = self.model_name.clone();
            let status_ref = self.status.clone();

            *status = TrainStatus::Running;
            self.request_start = false;

            thread::spawn(move || {
                let result = image_data_wrapper::train_model(&model, 1);
                dbg!(&result);
                *status_ref.lock().unwrap() = TrainStatus::Done(result);
            });
        }
    }

    fn handle_field_set(&mut self, field: &str, value: Box<dyn Any + Send>) {
        auto_set_field!(self, field, value, {
            "start_training" => request_start: bool,
            "model_name" => model_name: String
        });
    }

    fn handle_field_get(&self, field: &str) -> Option<Box<dyn Any + Send>> {
        auto_get_field!(self, field, {
            "model_name" => model_name: String,
            "status" => status: Arc<Mutex<TrainStatus>>
        })
    }
}

impl TrainThread {
    fn new(model_name: String) -> Self {
        Self {
            model_name,
            status: Arc::new(Mutex::new(TrainStatus::Idle)),
            request_start: true, // direkt starten!
        }
    }
}

struct GetBuildingsThread {
    path_to_image: String,
    buildings: Result<Vec<image_data_wrapper::Building>, FofError>,
    model_name: String,
    should_get_prediction: bool,
}

impl threading::AutoThread for GetBuildingsThread {
    fn run(&mut self) {
        if self.should_get_prediction {
            dbg!(&self.buildings);
            self.buildings =
                image_data_wrapper::get_prediction(&self.model_name.clone(), &self.path_to_image);
            dbg!(&self.buildings);
            self.should_get_prediction = false;
        }
    }
    fn handle_field_get(&self, field: &str) -> Option<Box<dyn std::any::Any + Send>> {
        auto_get_field!(self, field, {
            "buildings" => buildings: Result<Vec<image_data_wrapper::Building>, FofError>,
            "model_name" => model_name: String,
            "path_to_image" => path_to_image: String,
        })
    }
    fn handle_field_set(&mut self, field: &str, value: Box<dyn std::any::Any + Send>) {
        auto_set_field!(self, field, value, {
            "model_name" => model_name: String,
            "path_to_image"=> path_to_image: String,
            "should_get_prediction" => should_get_prediction: bool
        })
    }
}

pub struct ScreenshotApp {
    screenshot_path: String,
    keybind: String,
    split_count: i32,
    preview_texture: Option<egui::TextureHandle>,
    selected_image: Option<String>,
    image_folder: Option<PathBuf>,
    available_images: Vec<String>,
    selected_model: Option<String>,
    selected_yolo_model: Option<image_data_wrapper::YoloModel>,
    messages: Vec<UiMessage>,
    labeling_que: Vec<String>,
    selected_images: HashSet<String>,
    train_threads: Vec<threading::WorkerHandle<TrainThread>>,
    get_building_thread: threading::WorkerHandle<GetBuildingsThread>,
    current_buildings: Option<Vec<image_data_wrapper::Building>>,
    active_tab: Tab,
    image_texture: Option<egui::TextureHandle>,
    labeled_rects: Vec<SmthLabeled>,
    current_rect_start: Option<egui::Pos2>,
    current_rect_end: Option<egui::Pos2>,
    current_line_start: Option<egui::Pos2>,
    current_line_end: Option<egui::Pos2>,
    new_model_name: String,
    dataset_mode: Option<image_data_wrapper::DatasetType>,
    current_models: Vec<image_data_wrapper::Model>,
    in_test_mode: bool,
    current_avg_conf: Option<f32>,
}

// Wie stark sich Rechtecke überlappen (0.0 = kein Overlap, 0.5 = 50% Overlap)
const OVERLAP_PERCENT: f32 = 0.35;

#[derive(Clone)]
struct LabeledRect {
    rect: egui::Rect,
    label: String,
}

#[derive(Clone)]
struct LabeledLine {
    start: Pos2,
    end: Pos2,
    divisions: usize, // Anzahl Zwischenpunkte → Rechtecke = divisions + 1
    label: String,
}

#[derive(Clone)]
enum SmthLabeled {
    Rect(LabeledRect),
    Line(LabeledLine),
}

impl SmthLabeled {
    fn get_label(&self) -> String {
        match self {
            SmthLabeled::Rect(re) => re.label.clone(),
            SmthLabeled::Line(li) => li.label.clone(),
        }
    }

    fn set_label(&mut self, new: String) {
        match self {
            SmthLabeled::Rect(re) => re.label = new,
            SmthLabeled::Line(li) => li.label = new,
        }
    }

    fn push_str_to_label(&mut self, s: &str) {
        match self {
            SmthLabeled::Rect(re) => re.label.push_str(s),
            SmthLabeled::Line(li) => li.label.push_str(s),
        }
    }

    fn pop(&mut self) {
        match self {
            SmthLabeled::Rect(re) => {
                re.label.pop();
            }
            SmthLabeled::Line(li) => {
                li.label.pop();
            }
        }
    }

    fn get_rects(&self) -> Vec<LabeledRect> {
        match self {
            SmthLabeled::Rect(re) => vec![re.clone()],
            SmthLabeled::Line(li) => {
                let divisions = li.divisions;

                if divisions == 0 {
                    return vec![LabeledRect {
                        rect: Rect::from_two_pos(li.start, li.end),
                        label: li.label.clone(),
                    }];
                }

                let count = divisions + 1;
                let dummy_width = 1.0;
                let step = dummy_width * (1.0 - OVERLAP_PERCENT);

                // Simuliere Dummy Zentren entlang X-Achse
                let mut dummy_centers = Vec::with_capacity(count);
                for i in 0..count {
                    dummy_centers.push(i as f32 * step + dummy_width / 2.0);
                }

                // Dummy Rechteck: links und rechts
                let first_left = dummy_centers[0] - dummy_width / 2.0;
                let last_right = dummy_centers[count - 1] + dummy_width / 2.0;
                let simulated_length = last_right - first_left;

                // Echte Vektoren und Richtung
                let start = li.start.to_vec2();
                let end = li.end.to_vec2();
                let direction = end - start;
                let real_length = direction.length();
                let dir_norm = direction / real_length;

                let mut rects = Vec::with_capacity(count);

                for &center_x in &dummy_centers {
                    // Relative Position auf Strecke [0..1]
                    let relative_pos = (center_x - first_left) / simulated_length;

                    // Skalierte Mitte
                    let center_pos = start + dir_norm * (relative_pos * real_length);

                    // Skalierte halbe Breite
                    let half_width = (dummy_width / simulated_length) * real_length * 0.5;

                    // Punkte links und rechts vom Zentrum entlang Richtung
                    let p1 = (center_pos - dir_norm * half_width).to_pos2();
                    let p2 = (center_pos + dir_norm * half_width).to_pos2();

                    rects.push(LabeledRect {
                        rect: Rect::from_two_pos(p1, p2),
                        label: li.label.clone(),
                    });
                }

                rects
            }
        }
    }
}

impl Default for ScreenshotApp {
    fn default() -> Self {
        let mut s = Self {
            current_avg_conf: None,
            current_buildings: None,
            split_count: 1,
            preview_texture: None,
            screenshot_path: "/home/jesko/programmieren/ClashFoFBot/images".to_string(),
            keybind: "r".to_string(),
            selected_image: None,
            image_folder: Some(
                PathBuf::from_str("/home/jesko/programmieren/ClashFoFBot/images").unwrap(),
            ),
            available_images: vec![],
            selected_model: None,
            selected_yolo_model: None,
            messages: vec![],
            labeling_que: vec![],
            selected_images: HashSet::new(),
            train_threads: vec![],
            get_building_thread: threading::WorkerHandle::start(
                GetBuildingsThread {
                    path_to_image: "".to_string(),
                    buildings: Err(FofError::ThreadNotInitialized),
                    model_name: "".to_string(),
                    should_get_prediction: false,
                },
                true,
            ),
            active_tab: Tab::Settings,
            image_texture: None,
            labeled_rects: vec![],
            current_rect_start: None,
            current_rect_end: None,
            current_line_end: None,
            current_line_start: None,
            new_model_name: "".to_string(),
            dataset_mode: None, // Standardwert
            current_models: vec![],
            in_test_mode: false,
        };

        s.reload_models();

        s
    }
}

impl ScreenshotApp {
    fn split(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.ordner_wählen(
            ui,
            "Ordner Wählen wo die Imgs die du Splitten willst gespeichert sind:",
        );
        self.update_image_list();
        self.show_available_pngs_multiple(ui);
        ui.separator();

        ui.label("Number of splits (e.g. 4 = 2x2, 9 = 3x3):");
        ui.add(egui::DragValue::new(&mut self.split_count).clamp_range(1..=10000));

        if ui.button("SPLITEN").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                for img in self.selected_images.iter() {
                    let save_path = ScreenshotApp::build_split_filename(img, path.clone());
                    let res = fs::create_dir(&save_path);
                    let _ = res.expect(
                        "Wie konnte das passiereenenenenenene oi nein gelgelglegleglegflegl",
                    );
                    split_image::split(img, self.split_count, save_path.to_str().unwrap());
                }

                self.selected_images.clear();
                self.split_count = 1;

                self.create_error(
                    format!("Ausgewählte Imgs Gesplittet",),
                    MessageType::Success,
                );
            }
        }

        // If at least one image is selected, show the preview
        if let Some(first_image_path) = self.selected_images.iter().next() {
            // Only load once or when the path changes
            if self.preview_texture.is_none() {
                if let Ok(img) = image::open(first_image_path) {
                    let size = [img.width() as usize, img.height() as usize];
                    let rgba = img.to_rgba8();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    self.preview_texture = Some(ui.ctx().load_texture(
                        "preview",
                        color_image,
                        Default::default(),
                    ));
                }
            }

            if let Some(texture) = &self.preview_texture {
                let (img, scale) = self.get_scaled_texture(ui, texture);
                let img_size = img.size().unwrap() * scale;
                let response = ui.add(img);

                // Draw split grid
                let painter = ui.painter_at(response.rect);
                let parts = (self.split_count as f32).sqrt().floor() as i32;
                if parts >= 1 {
                    let part_width = img_size.x / parts as f32;
                    let part_height = img_size.y / parts as f32;

                    for i in 1..parts {
                        // Vertical lines
                        let x = response.rect.left_top().x + i as f32 * part_width;
                        painter.line_segment(
                            [
                                egui::pos2(x, response.rect.top()),
                                egui::pos2(x, response.rect.bottom()),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::WHITE),
                        );

                        // Horizontal lines
                        let y = response.rect.left_top().y + i as f32 * part_height;
                        painter.line_segment(
                            [
                                egui::pos2(response.rect.left(), y),
                                egui::pos2(response.rect.right(), y),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::WHITE),
                        );
                    }
                }
            }
        }
    }
    pub fn build_split_filename(image_path: &str, folder: PathBuf) -> PathBuf {
        let path = Path::new(image_path);

        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();

        folder.join(format!("{}_split", file_stem))
    }

    fn reverse_modified_filename(modified_path: &str) -> Option<String> {
        let path = std::path::Path::new(modified_path);

        // Get file name (e.g. "th17GELG1234567890.png")
        let file_name = path.file_name()?.to_string_lossy();

        // Separate the extension (e.g. ".png")
        let extension = path.extension()?.to_string_lossy();

        // Remove extension from file name
        let stem_with_gelg = file_name.trim_end_matches(&format!(".{}", extension));

        // Find the index of "GELG"
        let gelg_index = stem_with_gelg.find("GELG")?;

        // Get the original stem (everything before "GELG")
        let original_stem = &stem_with_gelg[..gelg_index];

        // Reconstruct original file name
        let original_file_name = format!("{}.{}", original_stem, extension);

        // Combine with the original parent path
        let original_path = path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .join(original_file_name);

        Some(original_path.to_string_lossy().to_string())
    }

    fn is_image_in_dataset(&self, filename: &str) -> bool {
        fn file_matches(dir: &Path, original_filename: &str) -> bool {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        if let Some(restored) = ScreenshotApp::reverse_modified_filename(file_name)
                        {
                            if restored == original_filename {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }

        let train_path = Path::new("dataset_buildings/images/train");
        let val_path = Path::new("dataset_buildings/images/val");

        file_matches(train_path, filename) || file_matches(val_path, filename)
    }

    pub fn create_error(&mut self, msg: impl Into<String>, kind: MessageType) {
        self.messages.push(UiMessage {
            message: msg.into(),
            kind,
            created: std::time::Instant::now(),
        });
    }

    fn update_err(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let fade_start = std::time::Duration::from_secs(4);
        let fade_duration = std::time::Duration::from_secs(2);
        let now = std::time::Instant::now();
        let error_multi = 2.;
        let warning_multi = 1.5;

        let max_msgs = 3;

        while self.messages.len() > max_msgs {
            self.messages.remove(0);
        }

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("ui_messages"),
        ));

        let spacing = 20.0;
        let mut total_height = 15.0;

        // Zeichne von rechts nach links (neueste Meldung rechts)
        for msg in self.messages.iter().rev() {
            let age = now.duration_since(msg.created);
            let mut alpha = 1.0;
            let thismulti = if msg.kind == MessageType::Success {
                1.
            } else if msg.kind == MessageType::Warning {
                warning_multi
            } else {
                error_multi
            };

            if age.as_secs_f32() > fade_start.as_secs_f32() * thismulti {
                let t = (age.as_secs_f32() - fade_start.as_secs_f32() * thismulti)
                    / fade_duration.as_secs_f32();
                alpha = 1.0 - t.clamp(0.0, 1.0);
            }

            if alpha <= 0.0 {
                continue;
            }

            let bg_color = match msg.kind {
                MessageType::Success => {
                    egui::Color32::from_rgba_unmultiplied(0, 200, 100, (255.0 * alpha) as u8)
                }
                MessageType::Warning => {
                    egui::Color32::from_rgba_unmultiplied(255, 180, 0, (255.0 * alpha) as u8)
                }
                MessageType::Error => {
                    egui::Color32::from_rgba_unmultiplied(200, 50, 50, (255.0 * alpha) as u8)
                }
            };
            let text = egui::RichText::new(&msg.message)
                .color(bg_color.blend(Color32::GRAY))
                .strong();

            let padding = egui::vec2(8.0, 4.0);

            let font_id = egui::FontId::proportional(15.0);
            let max_width = 400.0;

            let dark_col = bg_color.blend(Color32::from_rgba_unmultiplied(50, 50, 50, 177));

            // `layout` takes a `LayoutJob` or `&str` and a max width.
            let galley = painter.layout(text.text().to_string(), font_id, dark_col, max_width);

            let size = galley.size() + padding * 2.0;

            let pos = ctx.screen_rect().right_top() + egui::vec2(-size.x - spacing, total_height);

            let rect = egui::Rect::from_min_size(pos, size);
            let rect_expanded = rect.expand2(padding);

            painter.rect_filled(rect_expanded, 5.0, bg_color);

            painter.rect_stroke(
                rect_expanded,
                5.0,
                egui::Stroke::new(5., dark_col),
                StrokeKind::Middle,
            );

            painter.galley(pos + padding, galley, Color32::RED);

            total_height += size.y + spacing;
        }

        // Entferne alte Meldungen
        self.messages
            .retain(|msg| now.duration_since(msg.created) < fade_start + fade_duration);
    }

    fn update_image_list(&mut self) {
        if let Some(folder) = &self.image_folder {
            if let Ok(entries) = fs::read_dir(folder) {
                let mut images: Vec<_> = entries
                    .filter_map(|entry| entry.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| {
                                vec!["png", "jpg", "jpeg", "pdf", "gif", "webp"]
                                    .contains(&ext.to_str().unwrap())
                            })
                            .unwrap_or(false)
                    })
                    .collect();

                // Sortiere nach Änderungszeit (neueste zuerst)
                images.sort_by_key(|e| {
                    e.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                });
                images.reverse();

                self.available_images = images
                    .into_iter()
                    .map(|e| e.path().display().to_string())
                    .collect();
            }
        }
    }

    fn set_style(ctx: &egui::Context) {
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
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.active_tab == Tab::Settings, "|Einstellungen|")
                .clicked()
            {
                self.active_tab = Tab::Settings;
            }
            if ui
                .selectable_label(self.active_tab == Tab::YoloLabel, "|YOLO-Label|")
                .clicked()
            {
                self.active_tab = Tab::YoloLabel;
            }
            if ui
                .selectable_label(self.active_tab == Tab::Model, "|Model|")
                .clicked()
            {
                self.active_tab = Tab::Model;
            }
            if ui
                .selectable_label(self.active_tab == Tab::Split, "|Split|")
                .clicked()
            {
                self.active_tab = Tab::Split;
            }
        });
    }

    fn keybinds(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("Einstellungen");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Screenshot-Taste:");
                if ui.text_edit_singleline(&mut self.keybind).changed() {
                    self.create_error("Keybind geändert", MessageType::Success);
                }
            });

            if ui
                .button("📂 Speicher Ordner der Screenshots wählen")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.screenshot_path =
                        String::from_utf8(path.clone().as_os_str().as_bytes().to_vec()).unwrap();
                    self.create_error("Speicher Ordner Geändert", MessageType::Success);
                }
            }

            ui.label(format!(
                "📁 Ausgewähter Speicher Ordner: {}",
                self.screenshot_path
            ));

            let state = DeviceState::new();
            if state
                .query_keymap()
                .contains(&keycode_from_str(&self.keybind).unwrap_or(Keycode::V))
            {
                self.take_labeled_screenshot();
                self.update_image_list();
            }
        });
    }

    fn take_labeled_screenshot(&mut self) {
        let now = Local::now();
        let filename = format!("{}.png", now.format("%Y-%m-%d_%H-%M-%S"));

        let save_path = Path::new(&self.screenshot_path).join(filename);
        if let Err(e) = std::fs::create_dir_all(&self.screenshot_path) {
            self.create_error(
                format!("Fehler beim Erstellen des Ordners: {e}"),
                MessageType::Error,
            );
        }
        let screen = screener::make_screenshot(0);
        screen.save(&save_path).expect("error while saving img");
        self.create_error(
            format!("📸 Screenshot gespeichert unter: {}", save_path.display()),
            MessageType::Success,
        );
    }

    fn ordner_wählen(&mut self, ui: &mut egui::Ui, message: &str) {
        if ui.button(message).clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.image_folder = Some(path.clone());
                self.image_texture = None;
                self.create_error(format!("Ordner Geändert",), MessageType::Success);
            }
        }

        if let Some(folder) = &self.image_folder {
            ui.label(format!("📁 Ausgewählter Ordner: {}", folder.display()));
        }
    }

    fn show_available_pngs(&mut self, ui: &mut egui::Ui) {
        let resp = egui::ComboBox::from_label("Bild auswählen")
            .selected_text(
                self.selected_image
                    .as_ref()
                    .map(|s| {
                        Path::new(s)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    })
                    .unwrap_or_else(|| "Kein Bild ausgewählt".into()),
            )
            .show_ui(ui, |ui| {
                for img in &self.available_images {
                    let filename = Path::new(img)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();

                    let in_dataset = self.is_image_in_dataset(&filename);
                    let is_selected = self.selected_image.as_deref() == Some(img);

                    // Farbe festlegen
                    let color = if is_selected {
                        egui::Color32::from_rgb(100, 150, 255) // blau
                    } else if in_dataset {
                        egui::Color32::from_rgb(0, 200, 100) // grün
                    } else {
                        egui::Color32::from_rgb(200, 50, 50) // rot
                    };

                    let label = RichText::new(filename.clone()).color(color);

                    if ui.selectable_label(is_selected, label).clicked() {
                        self.selected_image = Some(img.clone());
                        self.image_texture = None;
                        ui.close_menu(); // schließt das Dropdown nach Auswahl
                    }
                }
            });

        if resp.response.changed() {
            self.create_error("Changed Img", MessageType::Success);
        }
    }

    fn update_image_texture(&mut self, ctx: &egui::Context, selected: String) {
        if self.image_texture.is_none() {
            if let Ok(img) = image::open(selected) {
                let img = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
                self.image_texture =
                    Some(ctx.load_texture("selected_image", color_img, Default::default()));
            }
        }
    }

    fn get_scaled_texture(
        &self,
        ui: &mut egui::Ui,
        texture: &egui::TextureHandle,
    ) -> (egui::Image, f32) {
        let available_size = ui.available_size();
        let tex_size = egui::vec2(texture.size()[0] as f32, texture.size()[1] as f32);

        // Seitenverhältnis beibehalten
        let scale = (available_size.x / tex_size.x).min(available_size.y / tex_size.y);
        let final_size = tex_size * scale;

        // Bild anzeigen
        (
            egui::Image::new(texture).fit_to_exact_size(final_size),
            scale,
        )
    }

    fn draw_buildings(
        &self,
        ui: &mut egui::Ui,
        buildings: Vec<image_data_wrapper::Building>,
        rect: egui::Rect,
        scale: f32,
    ) {
        for building in buildings {
            let (x, y, w, h) = building.bounding_box;

            let top_left = egui::pos2(rect.left() + x * scale, rect.top() + y * scale);
            let bottom_right = egui::pos2(rect.left() + w * scale, rect.top() + h * scale);

            let bounding_rect = egui::Rect::from_min_max(top_left, bottom_right);

            let color = Color32::RED;

            ui.painter().rect_stroke(
                bounding_rect,
                0.0,
                egui::Stroke::new(2.0, color),
                StrokeKind::Middle,
            );

            let label_text = format!(
                "{} ({:.0}%)",
                building.class_name,
                building.confidence * 100.0
            );

            ui.painter().text(
                top_left,
                egui::Align2::LEFT_TOP,
                label_text,
                egui::TextStyle::Body.resolve(ui.style()),
                Color32::BLUE,
            );
        }
    }

    fn show_selectable_models(&mut self, ui: &mut egui::Ui) {
        self.current_models.sort_by(|a, b| {
            a.rating
                .partial_cmp(&b.rating)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        egui::ComboBox::from_label("Modell auswählen")
            .selected_text(
                self.selected_model
                    .clone()
                    .unwrap_or_else(|| "Kein Modell gewählt".into()),
            )
            .show_ui(ui, |ui| {
                for model in self.current_models.clone() {
                    let score = model.rating;
                    let name = model.name;

                    let label = format!(
                        "{name} ({score:.2}) Typ: {}",
                        match model.dataset_type {
                            image_data_wrapper::DatasetType::Buildings => "🏗️ Building Model",
                            image_data_wrapper::DatasetType::Level => "🎯 Level Model",
                        }
                    );

                    if ui
                        .selectable_label(self.selected_model.as_deref() == Some(&name), label)
                        .clicked()
                    {
                        self.selected_model = Some(name.clone());
                        self.create_error("Model geändert", MessageType::Success);
                    }
                }
            });
    }

    fn show_selectable_yolo_models(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("YOLO-Modell wählen")
            .selected_text(
                self.selected_yolo_model
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "Keins gewählt".to_owned()),
            )
            .show_ui(ui, |ui| {
                for model in image_data_wrapper::YoloModel::iter() {
                    let is_selected = Some(&model) == self.selected_yolo_model.as_ref();

                    if ui
                        .selectable_label(is_selected, model.to_string())
                        .clicked()
                    {
                        self.selected_yolo_model = Some(model.clone());
                        self.create_error(format!("Yolo Model geändert",), MessageType::Success);
                    }
                }
            });
    }

    pub fn ui_dataset_mode_dropdown(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Datensatztyp:");

            let resp = egui::ComboBox::from_id_source("dataset_mode_selector")
                .selected_text(match self.dataset_mode {
                    None => "Nicht ausgewählt",
                    Some(image_data_wrapper::DatasetType::Buildings) => "Building Model",
                    Some(image_data_wrapper::DatasetType::Level) => "Level Model",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.dataset_mode,
                        Some(image_data_wrapper::DatasetType::Buildings),
                        "Building Model",
                    );
                    ui.selectable_value(
                        &mut self.dataset_mode,
                        Some(image_data_wrapper::DatasetType::Level),
                        "Level Model",
                    );
                });

            if resp.response.changed() {
                self.create_error("Datensatztyp geändert", MessageType::Success);
            }
        });
    }

    fn reload_models(&mut self) {
        let model_res = image_data_wrapper::get_all_models();

        if let Ok(ms) = model_res {
            self.current_models = ms;
        } else if let Err(e) = model_res {
            self.create_error(
                format!("Konnte Models nicht lade: {:?}", e),
                MessageType::Error,
            );
        }
    }

    fn manage_models(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Manage Models", |ui: &mut egui::Ui| {
            ui.group(|ui: &mut egui::Ui| {
                ui.heading("Neues Model Erstellen");
                ui.separator();
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("Model name: ");
                    ui.text_edit_singleline(&mut self.new_model_name);
                });
                self.show_selectable_yolo_models(ui);
                self.ui_dataset_mode_dropdown(ui);

                if let Some(yolo_model) = &self.selected_yolo_model {
                    if !self.new_model_name.is_empty() {
                        if let Some(datamode) = self.dataset_mode.clone() {
                            let button_text =
                                RichText::new("Model Hinzufügen").color(Color32::WHITE);

                            let button = egui::Button::new(button_text)
                                .fill(Color32::from_rgb(0, 180, 0)) // grün
                                .stroke(egui::Stroke::new(1.0, Color32::DARK_GREEN)); // optionaler Rand

                            if ui.add(button).clicked() {
                                image_data_wrapper::create_model(
                                    self.new_model_name.as_str(),
                                    datamode,
                                    yolo_model.clone(),
                                );
                                self.new_model_name.clear();
                                self.selected_yolo_model = None;
                                self.create_error(
                                    format!("Neues Model Erstellt",),
                                    MessageType::Success,
                                );
                                self.reload_models();
                            }
                        }
                    }
                }
            });
            ui.group(|ui: &mut egui::Ui| {
                ui.heading("Model Löschen");
                ui.separator();

                self.show_selectable_models(ui);

                if let Some(name) = &self.selected_model {
                    if ui
                        .add(egui::Button::new("Modell löschen").fill(egui::Color32::RED))
                        .clicked()
                    {
                        println!("Modell gelöscht: {name}");
                        let res = image_data_wrapper::delete_model(&name.to_string());
                        if let Some(e) = res {
                            self.create_error(
                                format!("Konnte Model nicht löschen: {:?}", e),
                                MessageType::Error,
                            );
                        } else {
                            self.create_error(format!("Model gelöscht",), MessageType::Success);
                        }
                        self.reload_models();

                        self.selected_model = None;
                    }
                }
            });
        });
    }

    fn update_buildings(&mut self) {
        if self.current_buildings.is_some() {
            return;
        }

        let buildings_res = self
            .get_building_thread
            .poll_field::<Result<Vec<image_data_wrapper::Building>, FofError>>("buildings");

        let buildings = if let Some(val) = buildings_res {
            val
        } else {
            self.create_error("Konnte Buildings nicht Laden", MessageType::Error);
            Ok(vec![])
        };

        if let Err(e) = buildings.clone() {
            if e == FofError::ThreadNotInitialized {
                self.create_error(
                    "Thread um Buildings zu bekommen ist noch nicht inizialisiert",
                    MessageType::Warning,
                );
            } else {
                self.create_error(
                    format!("Konnte Buildings nicht bekommen: {:?}", e),
                    MessageType::Error,
                );
            }
        } else if let Ok(bldngs) = buildings {
            self.current_buildings = Some(bldngs.clone());
            self.current_avg_conf = Some(image_data_wrapper::get_avg_confidence(&bldngs));
            self.create_error("Buildings Bekommen", MessageType::Success);
        }
    }

    fn model_testen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.collapsing("Model Testen", |ui: &mut egui::Ui| {
            ui.group(|ui: &mut egui::Ui| {
                self.ordner_wählen(ui, "📂 Speicher Ordner der Test Images wählen");
                self.show_available_pngs(ui);
                self.update_image_list();
                self.show_selectable_models(ui);
            });

            if !self.in_test_mode {
                if let Some(img) = self.selected_image.clone() {
                    if let Some(mdl) = &self.selected_model {
                        if ui.button("Show Test").clicked() {
                            self.get_building_thread
                                .set_field("model_name", mdl.to_string());
                            self.get_building_thread
                                .set_field("path_to_image", img.to_string());
                            self.get_building_thread
                                .set_field("should_get_prediction", true);
                            self.get_building_thread
                                .poll_field::<Result<Vec<image_data_wrapper::Building>, FofError>>(
                                    "buildings",
                                );
                            self.current_buildings = None;
                            self.current_avg_conf = None;
                            self.in_test_mode = true;
                        }
                    }
                }
            }

            let mut modeclone = self.in_test_mode;

            // The extra window
            egui::Window::new("Model Test")
                .open(&mut modeclone)
                .show(ctx, |ui: &mut egui::Ui| {
                    if let Some(selected) = &self.selected_image {
                        self.update_image_texture(ctx, selected.to_string());

                        if let Some(texture) = &self.image_texture {
                            let (img, scale) = self.get_scaled_texture(ui, texture);
                            let response = ui.add(img);

                            let rect = response.rect;

                            self.update_buildings();

                            if let Some(buildings) = self.current_buildings.clone() {
                                if let Some(avg) = self.current_avg_conf {
                                    ui.label(format!("Durchschnittliche Confidence: {}", avg));
                                }

                                self.draw_buildings(ui, buildings, rect, scale);
                            }
                        }
                    }
                });

            self.in_test_mode = modeclone;

            if let Some(selected) = &self.selected_image {
                self.update_image_texture(ctx, selected.to_string());

                if let Some(texture) = &self.image_texture {
                    ui.label("Vorschau: ");
                    let (img, scale) = self.get_scaled_texture(ui, texture);
                    let response = ui.add(img);
                }
            }
        });
    }

    fn model_training(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Training", |ui: &mut egui::Ui| {
            self.current_models.sort_by(|a, b| {
                a.rating
                    .partial_cmp(&b.rating)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            egui::ComboBox::from_label("Modell auswählen")
                .selected_text(
                    self.selected_model
                        .clone()
                        .unwrap_or_else(|| "Kein Modell gewählt".into()),
                )
                .show_ui(ui, |ui| {
                    for model in self.current_models.clone() {
                        let score = model.rating;
                        let name = model.name.clone();

                        let label = format!(
                            "{name} ({score:.2}) Typ: {}",
                            match model.dataset_type {
                                image_data_wrapper::DatasetType::Buildings => "🏗️ Building Model",
                                image_data_wrapper::DatasetType::Level => "🎯 Level Model",
                            }
                        );

                        let mut is_training = false;

                        let m = Some(model.name);

                        for thrd in self.train_threads.iter() {
                            if m == thrd.poll_field::<String>("model_name") {
                                if thrd.is_running() {
                                    is_training = true;
                                }
                            }
                        }

                        if ui
                            .selectable_label(
                                self.selected_model == m,
                                RichText::new(label).color(if is_training {
                                    Color32::YELLOW
                                } else {
                                    Color32::RED
                                }),
                            )
                            .clicked()
                        {
                            self.selected_model = Some(name.clone());
                            self.create_error("Model geändert", MessageType::Success);
                        }
                    }
                });

            for (idx, thrd) in self.train_threads.iter_mut().enumerate() {
                if thrd.poll_field::<String>("model_name") == self.selected_model
                    && self.selected_model.is_some()
                {
                    if thrd.is_running() {
                        let text = "Stop Training";
                        if ui
                            .add(
                                egui::Button::new(RichText::new(text).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(200, 50, 50)),
                            )
                            .clicked()
                        {
                            let t = self.train_threads.remove(idx);
                            t.stop();
                            self.create_error("Training gestoppt", MessageType::Success);
                        }
                        return;
                    }
                }
            }

            if self.selected_model.is_none() {
                return;
            }

            let text = "Start Training";
            if ui
                .add(
                    egui::Button::new(RichText::new(text).color(Color32::WHITE))
                        .fill(Color32::from_rgb(0, 180, 80)), // Grün
                )
                .clicked()
            {
                let wrkh = WorkerHandle::start(
                    TrainThread::new(self.selected_model.clone().unwrap()),
                    true,
                );
                self.train_threads.push(wrkh);
                self.create_error("Training gestartet", MessageType::Success);
            }
        });
    }

    fn model(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.separator();
        self.manage_models(ui);
        ui.separator();
        self.model_training(ui);
        ui.separator();
        self.model_testen(ui, ctx);
        ui.separator();
    }

    fn extract_numbers(s: &str) -> Vec<i32> {
        let re = regex::Regex::new(r"\d+").unwrap(); // matches sequences of digits
        re.find_iter(s)
            .filter_map(|mat| mat.as_str().parse::<i32>().ok())
            .collect()
    }

    fn handel_labeling_cursor(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        // Cursor-Position innerhalb des Bildes ermitteln
        let cursor_pos = ui.ctx().input(|i| i.pointer.hover_pos());
        let cursor_over_image = cursor_pos.map_or(false, |pos| rect.contains(pos));

        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let pointer_down = ui.input(|i| i.pointer.primary_down());
        let pointer_clicked = ui.input(|i| i.pointer.primary_clicked());
        let pointer_released = ui.input(|i| i.pointer.primary_released());

        let pointer_down2 = ui.input(|i| i.pointer.secondary_down());
        let pointer_clicked2 = ui.input(|i| i.pointer.secondary_clicked());
        let pointer_released2 = ui.input(|i| i.pointer.secondary_released());

        if cursor_over_image {
            if pointer_clicked {
                self.current_rect_start = pointer_pos;
                self.current_rect_end = self.current_rect_start;
            }

            if pointer_clicked2 {
                self.current_line_start = pointer_pos;
                self.current_line_end = self.current_line_start;
            }
            // Ziehen
            if pointer_down {
                if self.current_rect_start.is_none() {
                    self.current_rect_start = pointer_pos;
                }
                self.current_rect_end = pointer_pos;
            }
        }
        // Loslassen
        if pointer_released {
            if let Some(SmthLabeled::Rect(r)) = self.labeled_rects.last() {
                let lvls = ScreenshotApp::extract_numbers(&r.label);

                if lvls.len() > 1 {
                    self.create_error("Mehr als ein Level in Label Gefunden", MessageType::Warning);
                } else if lvls.is_empty() {
                    self.create_error("Kein Level in Label Gefunden", MessageType::Warning);
                } else if lvls[0] < 1 || lvls[0] > 17 {
                    self.create_error(
                        "Level in Label nicht zwischen 1 und 17",
                        MessageType::Warning,
                    );
                }
            }
            if let (Some(start), Some(end)) = (self.current_rect_start, self.current_rect_end) {
                let rect = egui::Rect::from_two_pos(start, end).expand(2.0);
                self.labeled_rects.push(SmthLabeled::Rect(LabeledRect {
                    rect,
                    label: String::new(),
                }));

                self.current_rect_end = None;
                self.current_rect_start = None;
            }
        }
        // Ziehen
        if pointer_down2 {
            if self.current_line_start.is_none() {
                self.current_line_start = pointer_pos;
            }
            self.current_line_end = pointer_pos;
        }
        // Loslassen
        if pointer_released2 {
            if let (Some(start), Some(end)) = (self.current_line_start, self.current_line_end) {
                let mut avg_divisons = vec![];
                for lsmth in self.labeled_rects.iter() {
                    if let SmthLabeled::Line(li) = lsmth {
                        let length = li.start.distance(li.end);
                        avg_divisons.push(li.divisions as f32 / length);
                    }
                }

                let divisions = if avg_divisons.len() != 0 {
                    let avg_divisons_per_unit =
                        avg_divisons.iter().sum::<f32>() / avg_divisons.len() as f32;

                    let this_length = start.distance(end);

                    let this_div = this_length * avg_divisons_per_unit;

                    this_div as usize
                } else {
                    0
                };

                self.labeled_rects.push(SmthLabeled::Line(LabeledLine {
                    start,
                    end,
                    divisions,
                    label: String::from("mauer"),
                }));

                self.current_line_end = None;
                self.current_line_start = None;
            }
        }
    }

    fn add_lable_to_yaml(&mut self, ctx: &egui::Context) {
        let mut parts: Vec<String> = self
            .labeled_rects
            .iter()
            .map(|r| {
                r.get_label()
                    .chars()
                    .take_while(|c| !c.is_numeric())
                    .collect()
            })
            .collect();
        if self.current_rect_start.is_none() {
            if let Some(r) = self.labeled_rects.last_mut() {
                // Lade bekannte Klassen aus data.yaml
                let yaml_path = std::path::Path::new("dataset_buildings/data.yaml");
                let yaml_content = std::fs::read_to_string(yaml_path).unwrap_or_default();

                #[derive(Deserialize)]
                struct DataYaml {
                    names: std::collections::HashMap<usize, String>,
                }

                let mut class_names: Vec<String> =
                    if let Ok(data) = serde_yaml::from_str::<DataYaml>(&yaml_content) {
                        data.names.values().cloned().collect()
                    } else {
                        vec![]
                    };

                class_names.append(&mut parts);

                let mut seen = std::collections::HashSet::new();
                let class_names: Vec<String> = class_names
                    .into_iter()
                    .filter(|s| seen.insert(s.clone()))
                    .collect();

                //dbg!(&class_names);

                let label = r.get_label();

                for event in &ctx.input(|i| i.events.clone()) {
                    match event {
                        egui::Event::Text(text) => {
                            if text == " " {
                                let trimmed = label.trim();
                                let matches: Vec<&String> = class_names
                                    .iter()
                                    .filter(|name| name.starts_with(trimmed) && *name != trimmed)
                                    .collect();

                                if matches.len() == 1 {
                                    r.set_label(matches[0].clone());
                                }
                            } else if text == "+" || text == "-" {
                            } else {
                                r.push_str_to_label(text);
                            }
                        }
                        egui::Event::Key {
                            key, pressed: true, ..
                        } => match key {
                            egui::Key::Backspace => {
                                r.pop();
                            }
                            egui::Key::Plus => {
                                if let SmthLabeled::Line(li) = r {
                                    li.divisions += 1;
                                }
                            }
                            egui::Key::Minus => {
                                if let SmthLabeled::Line(li) = r {
                                    if li.divisions == 0 {
                                    } else {
                                        li.divisions -= 1;
                                    }
                                }
                            }
                            egui::Key::Space => {
                                let trimmed = label.trim();
                                let matches: Vec<&String> = class_names
                                    .iter()
                                    .filter(|name| name.starts_with(trimmed) && *name != trimmed)
                                    .collect();

                                if matches.len() == 1 {
                                    r.set_label(matches[0].clone());
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }

        for event in &ctx.input(|i| i.events.clone()) {
            match event {
                egui::Event::Key {
                    key, pressed: true, ..
                } => match key {
                    egui::Key::Escape => {
                        self.labeled_rects.pop();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn save_labeld_rects(&mut self, final_size: egui::Vec2) {
        if let Some(image_path) = self.labeling_que.clone().last() {
            let mut rng = rand::thread_rng();
            self.create_error("Speichere YOLO-Labels...", MessageType::Success);

            let old_img_path = image_path;

            let image_paths = image_path.split(".").collect::<Vec<&str>>();
            let mut stem = image_paths[0].to_string();
            let ending = ".".to_string() + image_paths[1];
            stem.push_str("GELG");
            stem.push_str(rng.random_range(i128::MIN..i128::MAX).to_string().as_str());
            stem.push_str(&ending);

            let image_path = stem;

            use regex::Regex;
            use std::collections::HashMap;
            use std::fs;
            use std::io::Write;
            use std::path::{Path, PathBuf};

            #[derive(Debug, Deserialize, Serialize)]
            struct DataYaml {
                train: String,
                val: String,
                names: HashMap<usize, String>,
            }

            let dataset_paths = [
                ("dataset_buildings", Regex::new(r"\D+").unwrap()), // Nur Buchstaben
                ("dataset_level", Regex::new(r"\d+").unwrap()),     // Nur Ziffern
            ];

            for (idx, (dataset_base, label_regex)) in dataset_paths.iter().enumerate() {
                let str_path = format!("{}/data.yaml", dataset_base);
                let yaml_path = Path::new(&str_path);
                let yaml_content = match fs::read_to_string(yaml_path) {
                    Ok(content) => content,
                    Err(_) => {
                        self.create_error(
                            &format!("Konnte {}/data.yaml nicht lesen.", dataset_base),
                            MessageType::Error,
                        );
                        continue;
                    }
                };

                let mut data: DataYaml = match serde_yaml::from_str(&yaml_content) {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        self.create_error(
                            &format!("Konnte {}/data.yaml nicht parsen.", dataset_base),
                            MessageType::Error,
                        );
                        continue;
                    }
                };

                let mut class_map: HashMap<String, usize> =
                    data.names.iter().map(|(k, v)| (v.clone(), *k)).collect();

                let w = final_size.x;
                let h = final_size.y;

                let is_train = rng.gen_bool(0.8);
                let (img_target, label_target) = if is_train {
                    (
                        &format!("{}/images/train", dataset_base),
                        &format!("{}/labels/train", dataset_base),
                    )
                } else {
                    (
                        &format!("{}/images/val", dataset_base),
                        &format!("{}/labels/val", dataset_base),
                    )
                };

                let (img_target, label_target) = (Path::new(img_target), Path::new(label_target));

                let filename = Path::new(&image_path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let target_img_path = img_target.join(filename.as_str());

                dbg!(&target_img_path);

                if fs::create_dir_all(img_target).is_err()
                    || fs::copy(&old_img_path, &target_img_path).is_err()
                {
                    self.create_error(
                        &format!("Bild konnte nicht nach {dataset_base} kopiert werden."),
                        MessageType::Error,
                    );
                    continue;
                }

                if fs::create_dir_all(label_target).is_err() {
                    self.create_error(
                        &format!("{dataset_base}: Label-Ordner konnte nicht erstellt werden."),
                        MessageType::Error,
                    );
                    continue;
                }

                let stem = Path::new(&filename)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let label_path = label_target.join(format!("{}.txt", stem));

                dbg!(&label_path);

                let mut label_file = match fs::File::create(&label_path) {
                    Ok(file) => file,
                    Err(_) => {
                        self.create_error(
                            &format!("Konnte .txt-Datei für {dataset_base} nicht schreiben."),
                            MessageType::Error,
                        );
                        continue;
                    }
                };

                let mut yaml_updated = false;

                let mut all_labeled_rects = vec![];

                for lr in self.labeled_rects.clone().iter() {
                    all_labeled_rects.append(&mut lr.get_rects());
                }

                for lr in all_labeled_rects.iter() {
                    let raw_label = lr.label.trim();
                    let extracted = label_regex.find(raw_label).map(|m| m.as_str().to_string());

                    if extracted.is_none() {
                        continue;
                    }

                    let extracted_label = extracted.unwrap();

                    let class_id = if let Some(id) = class_map.get(&extracted_label) {
                        *id
                    } else if idx == 0 {
                        self.create_error("Label not found in buildings data.yaml!!! (was hat bro schon wieder getan)", MessageType::Error);
                        continue;
                    } else {
                        let new_id = data.names.len();
                        data.names.insert(new_id, extracted_label.clone());
                        class_map.insert(extracted_label.clone(), new_id);
                        yaml_updated = true;
                        new_id
                    };

                    let x = (lr.rect.min.x + lr.rect.max.x) / 2.0 / w;
                    let y = (lr.rect.min.y + lr.rect.max.y) / 2.0 / h;
                    let bw = (lr.rect.max.x - lr.rect.min.x) / w;
                    let bh = (lr.rect.max.y - lr.rect.min.y) / h;

                    if writeln!(
                        label_file,
                        "{} {:.6} {:.6} {:.6} {:.6}",
                        class_id, x, y, bw, bh
                    )
                    .is_err()
                    {
                        self.create_error(
                            &format!("Fehler beim Schreiben der Label-Datei ({dataset_base})"),
                            MessageType::Error,
                        );
                        continue;
                    }
                }

                if yaml_updated {
                    match serde_yaml::to_string(&data) {
                        Ok(new_yaml) => {
                            if fs::write(yaml_path, new_yaml).is_err() {
                                self.create_error(
                                    &format!(
                                        "Fehler beim Schreiben von {}/data.yaml",
                                        dataset_base
                                    ),
                                    MessageType::Error,
                                );
                            } else {
                                self.create_error(
                                    &format!(
                                        "Neue Klassen zu {}/data.yaml hinzugefügt.",
                                        dataset_base
                                    ),
                                    MessageType::Success,
                                );
                            }
                        }
                        Err(_) => {
                            self.create_error(
                                &format!(
                                    "Fehler beim Serialisieren von {}/data.yaml",
                                    dataset_base
                                ),
                                MessageType::Error,
                            );
                        }
                    }
                }

                self.create_error("YOLO-Labels gespeichert", MessageType::Success);
            }

            self.labeled_rects.clear();
        } else {
            self.create_error("Kein Bild zum Speichern ausgewählt.", MessageType::Warning);
        }
    }

    fn draw_rects(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Rechtecke zeichnen
        let painter = ui.painter();

        let mut all_labeled_rects = vec![];

        for lr in self.labeled_rects.clone().iter() {
            all_labeled_rects.append(&mut lr.get_rects());
        }

        for (idx, lr) in all_labeled_rects.iter().enumerate() {
            painter.rect_stroke(lr.rect, 0.0, (2.0, egui::Color32::RED), StrokeKind::Middle);
            if idx + 1 == all_labeled_rects.len() {
                painter.text(
                    lr.rect.left_top(),
                    egui::Align2::LEFT_TOP,
                    &lr.label,
                    egui::TextStyle::Body.resolve(&ctx.style()),
                    egui::Color32::RED,
                );
            }
        }

        if let (Some(start), Some(current)) = (self.current_rect_start, self.current_rect_end) {
            let rect = egui::Rect::from_two_pos(start, current);
            painter.rect_stroke(rect, 0.0, (1.0, egui::Color32::GREEN), StrokeKind::Middle);
        }

        if let (Some(start), Some(current)) = (self.current_line_start, self.current_line_end) {
            painter.line_segment([start, current], (1.0, egui::Color32::GREEN));
        }
    }

    fn show_available_pngs_multiple(&mut self, ui: &mut egui::Ui) {
        ui.label("Bilder auswählen:");
        egui::ScrollArea::vertical()
            .max_height(1000.0)
            .show(ui, |ui| {
                for img in self.available_images.clone().iter() {
                    let filename = Path::new(img)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let in_dataset = self.is_image_in_dataset(&filename);
                    let is_selected = self.selected_images.contains(img);

                    let color = if is_selected {
                        egui::Color32::from_rgb(100, 150, 255) // blau
                    } else if in_dataset {
                        egui::Color32::from_rgb(0, 200, 100) // grün
                    } else {
                        egui::Color32::from_rgb(200, 50, 50) // rot
                    };

                    let label = RichText::new(&filename).color(color);

                    if ui.selectable_label(is_selected, label).clicked() {
                        if is_selected {
                            self.selected_images.remove(img);
                            self.create_error("Png von Session entfernt", MessageType::Success);
                        } else {
                            self.selected_images.insert(img.clone());
                            self.create_error("Png zur Session hinzugefügt", MessageType::Success);
                        }
                        self.image_texture = None; // Optional: bei Änderung neuladen
                    }
                }
            });
    }

    fn session_button(&mut self, ui: &mut egui::Ui) {
        let is_running = !self.labeling_que.is_empty();

        let (button_text, button_color) = if is_running {
            (
                format!(
                    "Stop Session({}/{} Bildern Gelabelt)",
                    self.selected_images.len() - self.labeling_que.len(),
                    self.selected_images.len()
                ),
                Color32::from_rgb(200, 50, 50),
            ) // rot
        } else {
            (
                format!(
                    "Start Session ({} ausgewählte Bilder)",
                    self.selected_images.len()
                ),
                Color32::from_rgb(0, 200, 100),
            ) // grün
        };

        if ui
            .add(
                egui::Button::new(RichText::new(button_text).color(Color32::BLACK))
                    .fill(button_color)
                    .min_size(egui::vec2(150.0, 30.0)),
            )
            .clicked()
        {
            if is_running {
                self.labeling_que.clear();
                self.selected_images.clear();
                self.labeled_rects.clear();
                self.create_error("Session beendet", MessageType::Success);
            } else {
                self.labeling_que = self.selected_images.iter().cloned().collect();
                self.create_error("Session gestartet", MessageType::Success);
            }
        }
    }

    fn yolo_label(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let is_running = !self.labeling_que.is_empty();

        if !is_running {
            ui.group(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(500.)
                    .show(ui, |ui| {
                        ui.heading("Png(s) Zum Labeln Wählen");
                        ui.separator();
                        self.ordner_wählen(
                            ui,
                            "📂 Speicher Ordner der zu Labelnden Images wählen",
                        );
                        self.update_image_list();
                        self.show_available_pngs_multiple(ui);
                    });

                ui.horizontal(|ui: &mut egui::Ui| {
                    if ui.button("Alle roten hinzufügen").clicked() {
                        for img in self.available_images.clone().iter() {
                            let filename = Path::new(img)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();

                            let in_dataset = self.is_image_in_dataset(&filename);

                            if !in_dataset {
                                self.selected_images.insert(img.to_string());
                            }
                        }
                        self.create_error(
                            "ALle Pngs, die nicht im Dataset, sind zur Labeling Que hinzugefügt",
                            MessageType::Success,
                        );
                    }
                    if ui.button("Alle entfernen").clicked() {
                        self.selected_images.clear();
                        self.create_error(
                            "ALle Pngs aus der Labeling Que entfernt",
                            MessageType::Success,
                        );
                    }
                });
            });
        }

        self.session_button(ui);

        if is_running {
            if let Some(selected) = self.labeling_que.last() {
                self.update_image_texture(ctx, selected.to_string());

                if let Some(texture) = &self.image_texture {
                    let (img, _scale) = self.get_scaled_texture(ui, texture);
                    let response = ui.add(img);

                    // Das gezeichnete Rechteck
                    let rect = response.rect;
                    self.handel_labeling_cursor(ui, rect);
                    self.add_lable_to_yaml(ctx);

                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.save_labeld_rects(rect.size());
                        self.labeling_que.pop();
                        if self.labeling_que.is_empty() {
                            self.selected_images.clear();
                        }
                        self.image_texture = None;
                    }

                    self.draw_rects(ui, ctx);
                }
            } else {
                ui.label("Kein Bild ausgewählt.");
            }
        }
    }
}

impl eframe::App for ScreenshotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ScreenshotApp::set_style(ctx);
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs(ui);

            ui.separator();

            match self.active_tab {
                Tab::Settings => {
                    self.keybinds(ui);
                }
                Tab::Model => {
                    self.model(ui, ctx);
                }
                Tab::YoloLabel => {
                    self.yolo_label(ui, ctx);
                }
                Tab::Split => {
                    self.split(ui, ctx);
                }
            }
            self.update_err(ui, ctx);

            let mut errors = vec![];

            for thrd in self.train_threads.iter() {
                if let Some(winterarc) = thrd.poll_field::<Arc<Mutex<TrainStatus>>>("status") {
                    let trainingstatus = winterarc.lock().unwrap().clone();
                    if let TrainStatus::Done(Some(e)) = trainingstatus {
                        errors.push(e);
                    }
                }
            }

            for e in errors {
                self.create_error(format!("Error while Training: {:?}", e), MessageType::Error);
            }
        });
    }
}
