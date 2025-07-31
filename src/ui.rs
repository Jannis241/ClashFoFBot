use eframe::egui::Widget;

use crate::{image_data_wrapper::Building, prelude::*, threading::AutoThread};

pub fn start_ui() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Screenshot Tool",
        options,
        Box::new(|_cc| Ok(Box::new(ui::ScreenshotApp::default()))),
    );
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
}

struct TrainThread {
    model_name: String,
}

impl threading::AutoThread for TrainThread {
    fn run(&mut self) {
        image_data_wrapper::train_model(self.model_name.clone(), 1);
    }
    fn set_field_any(&mut self, field: &str, value: Box<dyn std::any::Any>) -> bool {
        panic!("shouldnt set any fields")
    }
    fn get_field_any(&self, field: &str) -> Option<&dyn std::any::Any> {
        panic!("shouldnt get any outputs")
    }
}

struct GetBuildingsThread {
    path_to_image: String,
    buildings: Vec<image_data_wrapper::Building>,
    model_name: String,
}

impl threading::AutoThread for GetBuildingsThread {
    fn run(&mut self) {
        let res;
        (self.buildings, res) = image_data_wrapper::get_buildings(
            self.model_name.clone(),
            &Path::new(&self.path_to_image),
        );
        assert_eq!(res, true);
    }
    fn set_field_any(&mut self, field: &str, value: Box<dyn std::any::Any>) -> bool {
        auto_set_field!(field, value, "path_to_image", |val: String| self
            .path_to_image =
            *val);

        false
    }
    fn get_field_any(&self, field: &str) -> Option<&dyn std::any::Any> {
        match field {
            "buildings" => Some(&self.buildings),
            other => panic!("is not able to get '{other}'"),
        }
    }
}

pub struct ScreenshotApp {
    pub screenshot_path: String,
    pub keybind: String,
    pub selected_image: Option<String>,
    pub image_folder: Option<PathBuf>,
    pub available_images: Vec<String>,
    pub epoche: String,
    pub current_buildings: Option<Vec<image_data_wrapper::Building>>,
    selected_model: Option<String>,

    pub labeling_que: Vec<String>,
    selected_images: HashSet<String>,

    train_threads: Vec<threading::WorkerHandle<TrainThread>>,
    get_building_thread: threading::WorkerHandle<GetBuildingsThread>,

    active_tab: Tab,
    image_texture: Option<egui::TextureHandle>,
    labeled_rects: Vec<LabeledRect>,
    current_rect_start: Option<egui::Pos2>,
    current_rect_end: Option<egui::Pos2>,

    new_model_name: String,
}

#[derive(Clone)]
struct LabeledRect {
    rect: egui::Rect,
    label: String,
}

impl Default for ScreenshotApp {
    fn default() -> Self {
        Self {
            screenshot_path: "/home/jesko/programmieren/ClashFoFBot/images".to_string(),
            keybind: "r".to_string(),
            selected_image: None,
            image_folder: Some(
                PathBuf::from_str("/home/jesko/programmieren/ClashFoFBot/images").unwrap(),
            ),
            selected_images: HashSet::new(),
            epoche: "".to_string(),
            current_buildings: None,
            selected_model: None,
            new_model_name: "".to_string(),

            train_threads: vec![],
            get_building_thread: threading::WorkerHandle::new(GetBuildingsThread {
                path_to_image: "".to_string(),
                buildings: vec![],
                model_name: "".to_string(),
            }),
            labeling_que: vec![],

            available_images: vec![],
            active_tab: Tab::Settings,
            image_texture: None,
            current_rect_end: None,
            current_rect_start: None,
            labeled_rects: vec![],
        }
    }
}

impl ScreenshotApp {
    fn is_image_in_dataset(&self, filename: &str) -> bool {
        let train_path = Path::new("dataset/images/train").join(filename);
        let val_path = Path::new("dataset/images/val").join(filename);
        train_path.exists() || val_path.exists()
    }

    fn update_image_list(&mut self) {
        if let Some(folder) = &self.image_folder {
            if let Ok(entries) = fs::read_dir(folder) {
                let mut images: Vec<_> = entries
                    .filter_map(|entry| entry.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "png")
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
                .selectable_label(self.active_tab == Tab::Settings, "⚙️ Einstellungen")
                .clicked()
            {
                self.active_tab = Tab::Settings;
            }
            if ui
                .selectable_label(self.active_tab == Tab::YoloLabel, "🖼️ YOLO-Label")
                .clicked()
            {
                self.active_tab = Tab::YoloLabel;
            }
            if ui
                .selectable_label(self.active_tab == Tab::Model, "|| Model")
                .clicked()
            {
                self.active_tab = Tab::Model;
            }
        });
    }

    fn keybinds(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("Settings");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Screenshot-Key:");
                ui.text_edit_singleline(&mut self.keybind);
            });

            ui.horizontal(|ui| {
                if ui
                    .button("📂 Speicher Ordner der Screenshots wählen")
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.screenshot_path =
                            String::from_utf8(path.clone().as_os_str().as_bytes().to_vec())
                                .unwrap();
                    }
                }

                ui.label(format!(
                    "📁 Ausgewähter Speicher Ordner: {}",
                    self.screenshot_path
                ));
            });

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
            eprintln!("Fehler beim Erstellen des Ordners: {e}");
        }
        let screen = screener::make_screenshot(0);
        screen.save(&save_path).expect("error while saving img");
        println!("📸 Screenshot gespeichert unter: {}", save_path.display());
    }

    fn ordner_wählen(&mut self, ui: &mut egui::Ui, message: &str) {
        if ui.button(message).clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.image_folder = Some(path.clone());
                self.image_texture = None;
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
            self.get_building_thread
                .set_field("path_to_image", self.selected_image.clone().unwrap());
            self.get_building_thread.start_once();
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
        buildings: Vec<Building>,
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
                color,
            );
        }
    }

    fn show_selectable_models(&mut self, ui: &mut egui::Ui) {
        // Modelle holen und nach Score sortieren (absteigend)
        let mut models = image_data_wrapper::get_model_names();
        models.sort_by(|a, b| {
            image_data_wrapper::get_rating(b.clone())
                .partial_cmp(&image_data_wrapper::get_rating(a.clone()))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        egui::ComboBox::from_label("Modell auswählen")
            .selected_text(
                self.selected_model
                    .clone()
                    .unwrap_or_else(|| "Kein Modell gewählt".into()),
            )
            .show_ui(ui, |ui| {
                for model in models {
                    let score = image_data_wrapper::get_rating(model.clone());
                    let label = format!("{model} ({score:.2})");

                    if ui
                        .selectable_label(self.selected_model.as_deref() == Some(&model), label)
                        .clicked()
                    {
                        self.selected_model = Some(model);
                        self.get_building_thread
                            .set_field("model_name", model.to_string());
                    }
                }
            });
    }

    fn model(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.collapsing("Manage Models", |ui: &mut egui::Ui| {
            ui.group(|ui: &mut egui::Ui| {
                ui.heading("Neues Model Erstellen");
                ui.separator();
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("model namen: ");
                    ui.text_edit_singleline(&mut self.new_model_name);
                });
            });
        });
        ui.collapsing("Training", |ui: &mut egui::Ui| {});
        ui.collapsing("Model testen", |ui: &mut egui::Ui| {
            ui.group(|ui: &mut egui::Ui| {
                self.ordner_wählen(ui, "📂 Speicher Ordner der Test Images wählen");
                self.show_available_pngs(ui);
                self.update_image_list();
                self.show_selectable_models(ui);
            });
            if let Some(selected) = &self.selected_image {
                self.update_image_texture(ctx, selected.to_string());

                if let Some(texture) = &self.image_texture {
                    let (img, scale) = self.get_scaled_texture(ui, texture);
                    let response = ui.add(img);

                    let buildings = self
                        .get_building_thread
                        .get_output::<Vec<Building>>("buildings");

                    let rect = response.rect;

                    if let Some(buildings) = buildings {
                        let avg_confidence = image_data_wrapper::get_avg_confidence(&buildings);
                        ui.label(format!("Durchschnittliche Confidence: {}", avg_confidence));
                        self.draw_buildings(ui, buildings, rect, scale);
                    }
                }
            }
        });
    }

    fn handel_labeling_cursor(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        // Cursor-Position innerhalb des Bildes ermitteln
        let cursor_pos = ui.ctx().input(|i| i.pointer.hover_pos());
        let cursor_over_image = cursor_pos.map_or(false, |pos| rect.contains(pos));

        let pointer_pos = ui.input(|i| i.pointer.hover_pos());

        if cursor_over_image {
            let pointer_down = ui.input(|i| i.pointer.primary_down());
            let pointer_clicked = ui.input(|i| i.pointer.primary_clicked());
            let pointer_released = ui.input(|i| i.pointer.primary_released());

            if pointer_clicked {
                self.current_rect_start = pointer_pos;
                self.current_rect_end = self.current_rect_start;
            }
            // Ziehen
            if pointer_down {
                if self.current_rect_start.is_none() {
                    self.current_rect_start = pointer_pos;
                }
                self.current_rect_end = pointer_pos;
            }
            // Loslassen
            if pointer_released {
                if let (Some(start), Some(end)) = (self.current_rect_start, self.current_rect_end) {
                    let rect = egui::Rect::from_two_pos(start, end).expand(2.0);
                    self.labeled_rects.push(LabeledRect {
                        rect,
                        label: String::new(),
                    });

                    self.current_rect_end = None;
                    self.current_rect_start = None;
                }
            }
        }
    }
    fn add_lable_to_yaml(&mut self, ctx: &egui::Context) {
        if self.current_rect_start.is_none() {
            if let Some(r) = self.labeled_rects.last_mut() {
                // Lade bekannte Klassen aus data.yaml
                let yaml_path = std::path::Path::new("dataset/data.yaml");
                let yaml_content = std::fs::read_to_string(yaml_path).unwrap_or_default();

                #[derive(Deserialize)]
                struct DataYaml {
                    names: std::collections::HashMap<usize, String>,
                }

                let class_names: Vec<String> =
                    if let Ok(data) = serde_yaml::from_str::<DataYaml>(&yaml_content) {
                        data.names.values().cloned().collect()
                    } else {
                        vec![]
                    };

                for event in &ctx.input(|i| i.events.clone()) {
                    match event {
                        egui::Event::Text(text) => {
                            r.label.push_str(text);
                        }
                        egui::Event::Key {
                            key, pressed: true, ..
                        } => match key {
                            egui::Key::Backspace => {
                                r.label.pop();
                            }
                            egui::Key::Tab => {
                                let trimmed = r.label.trim();
                                let matches: Vec<&String> = class_names
                                    .iter()
                                    .filter(|name| name.starts_with(trimmed))
                                    .collect();

                                if matches.len() == 1 {
                                    r.label = matches[0].clone();
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    fn save_labeld_rects(&mut self, final_size: egui::Vec2) {
        if let Some(image_path) = &self.labeling_que.last() {
            println!("📦 Speichere YOLO-Labels...");

            use std::collections::HashMap;
            use std::fs;
            use std::io::Write;
            use std::path::Path;

            #[derive(Debug, Deserialize, Serialize)]
            struct DataYaml {
                train: String,
                val: String,
                names: HashMap<usize, String>,
            }

            let yaml_path = Path::new("dataset/data.yaml");
            let yaml_content =
                fs::read_to_string(yaml_path).expect("❌ Kann data.yaml nicht lesen");
            let mut data: DataYaml =
                serde_yaml::from_str(&yaml_content).expect("❌ Kann data.yaml nicht parsen");

            // Umgekehrtes Mapping: Label → ID
            let mut class_map: HashMap<String, usize> =
                data.names.iter().map(|(k, v)| (v.clone(), *k)).collect();

            let w = final_size.x;
            let h = final_size.y;

            // Zielordner zufällig: train oder val
            let mut rng = rand::thread_rng();
            let is_train = rng.gen_bool(0.8); // 80% train
            let (img_target, label_target) = if is_train {
                (
                    Path::new("dataset/images/train"),
                    Path::new("dataset/labels/train"),
                )
            } else {
                (
                    Path::new("dataset/images/val"),
                    Path::new("dataset/labels/val"),
                )
            };

            // Datei kopieren
            let filename = Path::new(image_path).file_name().unwrap().to_str().unwrap();

            let target_img_path = img_target.join(filename);
            fs::create_dir_all(img_target).unwrap();
            fs::copy(image_path, &target_img_path).expect("❌ Bild konnte nicht kopiert werden");

            // Label-Datei schreiben
            let label_path = label_target.join(filename.replace(".png", ".txt"));
            fs::create_dir_all(label_target).unwrap();

            let mut label_file =
                fs::File::create(&label_path).expect("❌ Konnte .txt nicht schreiben");

            let mut yaml_updated = false;

            for lr in &self.labeled_rects {
                let label = lr.label.trim().to_string();

                // Wenn Klasse nicht existiert, hinzufügen
                let class_id = if let Some(id) = class_map.get(&label) {
                    *id
                } else {
                    let new_id = data.names.len();
                    data.names.insert(new_id, label.clone());
                    class_map.insert(label.clone(), new_id);
                    yaml_updated = true;
                    new_id
                };

                let x = (lr.rect.min.x + lr.rect.max.x) / 2.0 / w as f32;
                let y = (lr.rect.min.y + lr.rect.max.y) / 2.0 / h as f32;
                let bw = (lr.rect.max.x - lr.rect.min.x) / w as f32;
                let bh = (lr.rect.max.y - lr.rect.min.y) / h as f32;

                writeln!(
                    label_file,
                    "{} {:.6} {:.6} {:.6} {:.6}",
                    class_id, x, y, bw, bh
                )
                .expect("❌ Schreiben fehlgeschlagen");
            }

            // YAML zurückschreiben, wenn geändert
            if yaml_updated {
                let new_yaml =
                    serde_yaml::to_string(&data).expect("❌ Fehler beim YAML-Serialisieren");
                fs::write(yaml_path, new_yaml).expect("❌ Fehler beim Schreiben von data.yaml");
                println!("📄 Neue Labels wurden zu data.yaml hinzugefügt.");
            }

            println!("✅ YOLO-Label gespeichert: {}", label_path.display());
            self.labeled_rects.clear(); // fertig gelabelt
        }
    }

    fn draw_rects(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Rechtecke zeichnen
        let painter = ui.painter();

        for lr in &self.labeled_rects {
            painter.rect_stroke(lr.rect, 0.0, (2.0, egui::Color32::RED), StrokeKind::Middle);
            painter.text(
                lr.rect.left_top(),
                egui::Align2::LEFT_TOP,
                &lr.label,
                egui::TextStyle::Body.resolve(&ctx.style()),
                egui::Color32::RED,
            );
        }

        if let (Some(start), Some(current)) = (self.current_rect_start, self.current_rect_end) {
            let rect = egui::Rect::from_two_pos(start, current);
            painter.rect_stroke(rect, 0.0, (1.0, egui::Color32::GREEN), StrokeKind::Middle);
        }
    }

    fn show_available_pngs_multiple(&mut self, ui: &mut egui::Ui) {
        ui.label("Bilder auswählen:");
        egui::ScrollArea::vertical()
            .max_height(1000.0)
            .show(ui, |ui| {
                for img in &self.available_images {
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
                        } else {
                            self.selected_images.insert(img.clone());
                        }
                        self.image_texture = None; // Optional: bei Änderung neuladen
                    }
                }
            });
    }

    fn session_button(&mut self, ui: &mut egui::Ui) {
        let is_running = !self.labeling_que.is_empty();

        let (button_text, button_color) = if is_running {
            ("Stop Session", Color32::from_rgb(200, 50, 50)) // rot
        } else {
            ("Start Session", Color32::from_rgb(0, 200, 100)) // grün
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
            } else {
                self.labeling_que = self.selected_images.iter().cloned().collect();
            }
        }
    }

    fn yolo_label(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let is_running = !self.labeling_que.is_empty();

        if !is_running {
            ui.group(|ui| {
                ui.heading("Png(s) Zum Labeln Wählen");
                ui.separator();
                self.ordner_wählen(ui, "📂 Speicher Ordner der zu Labelnden Images wählen");
                self.show_available_pngs(ui);
                self.update_image_list();
                self.show_available_pngs_multiple(ui);
            });
        }

        self.session_button(ui);

        if is_running {
            if let Some(selected) = self.labeling_que.last() {
                self.update_image_texture(ctx, selected.to_string());

                if let Some(texture) = &self.image_texture {
                    let (img, scale) = self.get_scaled_texture(ui, texture);
                    let response = ui.add(img);

                    // Das gezeichnete Rechteck
                    let rect = response.rect;
                    self.handel_labeling_cursor(ui, rect);
                    self.add_lable_to_yaml(ctx);

                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.save_labeld_rects(rect.size());
                        self.labeling_que.pop();
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
            }
        });
    }
}
