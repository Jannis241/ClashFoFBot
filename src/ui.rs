use crate::prelude::*;

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
}

pub struct ScreenshotApp {
    pub screenshot_path: String,
    pub keybind: String,
    pub selected_image: Option<String>,
    pub image_folder: Option<PathBuf>,
    pub available_images: Vec<String>,

    active_tab: Tab,
    image_texture: Option<egui::TextureHandle>,
    labeled_rects: Vec<LabeledRect>,
    current_rect_start: Option<egui::Pos2>,
    current_rect_end: Option<egui::Pos2>,
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
            });

            ui.separator();

            match self.active_tab {
                Tab::Settings => {
                    ui.collapsing("📸 Keybinds", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Screenshot-Key:");
                            ui.text_edit_singleline(&mut self.keybind);
                        });

                        ui.horizontal(|ui| {
                            if ui.button("📂 Speicher Ordner wählen").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.screenshot_path = String::from_utf8(
                                        path.clone().as_os_str().as_bytes().to_vec(),
                                    )
                                    .unwrap();
                                }
                            }

                            ui.label(format!("📁 Speicher Ordner: {}", self.screenshot_path));
                        });

                        let state = DeviceState::new();
                        if state
                            .query_keymap()
                            .contains(&keycode_from_str(&self.keybind).unwrap_or(Keycode::V))
                        {
                            println!("took screenshot");
                            let now = Local::now();
                            let filename = format!("{}.png", now.format("%Y-%m-%d_%H-%M-%S"));

                            let save_path = Path::new(&self.screenshot_path).join(filename);
                            if let Err(e) = std::fs::create_dir_all(&self.screenshot_path) {
                                eprintln!("Fehler beim Erstellen des Ordners: {e}");
                            }
                            let screen = screener::make_screenshot(0);
                            screen.save(&save_path).expect("error while saving img");
                            println!("📸 Screenshot gespeichert unter: {}", save_path.display());

                            self.update_image_list(); // 👈 Bildliste neu laden
                        }
                    });
                    ui.collapsing("🖼️ Labeln", |ui| {
                        if ui.button("📂 Ordner wählen").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.image_folder = Some(path.clone());
                                self.image_texture = None;
                            }
                        }

                        if let Some(folder) = &self.image_folder {
                            ui.label(format!("📁 Ordner: {}", folder.display()));
                        }

                        self.update_image_list();

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

                            let label = egui::RichText::new(filename).color(color);

                            if ui.selectable_label(is_selected, label).clicked() {
                                self.selected_image = Some(img.clone());
                                self.image_texture = None;
                            }
                        }
                    });
                }

                Tab::YoloLabel => {
                    if let Some(selected) = &self.selected_image {
                        // Bild laden & in Texture umwandeln
                        if self.image_texture.is_none() {
                            if let Ok(img) = image::open(selected) {
                                let img = img.to_rgba8();
                                let size = [img.width() as usize, img.height() as usize];
                                let color_img =
                                    egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
                                self.image_texture = Some(ctx.load_texture(
                                    "selected_image",
                                    color_img,
                                    Default::default(),
                                ));
                            }
                        }

                        if let Some(texture) = &self.image_texture {
                            // Verfügbare Größe des Panels
                            let available_size = ui.available_size();
                            let tex_size =
                                egui::vec2(texture.size()[0] as f32, texture.size()[1] as f32);

                            // Bildgröße proportional anpassen (Seitenverhältnis behalten)
                            let scale =
                                (available_size.x / tex_size.x).min(available_size.y / tex_size.y);
                            let final_size = tex_size * scale;

                            // Bild anzeigen
                            let img = egui::Image::new(texture).fit_to_exact_size(final_size);
                            let response = ui.add(img);

                            // Das gezeichnete Rechteck
                            let rect = response.rect;

                            // Cursor-Position innerhalb des Bildes ermitteln
                            let cursor_pos = ui.ctx().input(|i| i.pointer.hover_pos());
                            let cursor_over_image =
                                cursor_pos.map_or(false, |pos| rect.contains(pos));

                            let pointer_pos = ui.input(|i| i.pointer.hover_pos());

                            if cursor_over_image {
                                let pointer_down = ui.input(|i| i.pointer.primary_down());
                                let pointer_clicked = ui.input(|i| i.pointer.primary_clicked());
                                let pointer_released = ui.input(|i| i.pointer.primary_released());

                                if pointer_clicked {
                                    dbg!("clicked");
                                    self.current_rect_start = pointer_pos;
                                    self.current_rect_end = self.current_rect_start;
                                }
                                // Ziehen
                                if pointer_down {
                                    dbg!(pointer_pos);
                                    if self.current_rect_start.is_none() {
                                        self.current_rect_start = pointer_pos;
                                    }
                                    self.current_rect_end = pointer_pos;
                                }
                                // Loslassen
                                if pointer_released {
                                    dbg!(self.current_rect_start, self.current_rect_end);
                                    if let (Some(start), Some(end)) =
                                        (self.current_rect_start, self.current_rect_end)
                                    {
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

                            if self.current_rect_start.is_none() {
                                if let Some(r) = self.labeled_rects.last_mut() {
                                    // Lade bekannte Klassen aus data.yaml
                                    let yaml_path = std::path::Path::new("dataset/data.yaml");
                                    let yaml_content =
                                        std::fs::read_to_string(yaml_path).unwrap_or_default();

                                    #[derive(Deserialize)]
                                    struct DataYaml {
                                        names: std::collections::HashMap<usize, String>,
                                    }

                                    let class_names: Vec<String> = if let Ok(data) =
                                        serde_yaml::from_str::<DataYaml>(&yaml_content)
                                    {
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

                            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                                if let Some(image_path) = &self.selected_image {
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
                                    let yaml_content = fs::read_to_string(yaml_path)
                                        .expect("❌ Kann data.yaml nicht lesen");
                                    let mut data: DataYaml = serde_yaml::from_str(&yaml_content)
                                        .expect("❌ Kann data.yaml nicht parsen");

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
                                    let filename = Path::new(image_path)
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap();

                                    let target_img_path = img_target.join(filename);
                                    fs::create_dir_all(img_target).unwrap();
                                    fs::copy(image_path, &target_img_path)
                                        .expect("❌ Bild konnte nicht kopiert werden");

                                    // Label-Datei schreiben
                                    let label_path =
                                        label_target.join(filename.replace(".png", ".txt"));
                                    fs::create_dir_all(label_target).unwrap();

                                    let mut label_file = fs::File::create(&label_path)
                                        .expect("❌ Konnte .txt nicht schreiben");

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
                                        let new_yaml = serde_yaml::to_string(&data)
                                            .expect("❌ Fehler beim YAML-Serialisieren");
                                        fs::write(yaml_path, new_yaml)
                                            .expect("❌ Fehler beim Schreiben von data.yaml");
                                        println!("📄 Neue Labels wurden zu data.yaml hinzugefügt.");
                                    }

                                    println!("✅ YOLO-Label gespeichert: {}", label_path.display());
                                    self.labeled_rects.clear(); // fertig gelabelt
                                    self.active_tab = Tab::Settings;
                                }
                            }

                            // Rechtecke zeichnen
                            let painter = ui.painter();
                            dbg!(self.labeled_rects.len());
                            for lr in &self.labeled_rects {
                                painter.rect_stroke(
                                    lr.rect,
                                    0.0,
                                    (2.0, egui::Color32::RED),
                                    StrokeKind::Middle,
                                );
                                painter.text(
                                    lr.rect.left_top(),
                                    egui::Align2::LEFT_TOP,
                                    &lr.label,
                                    egui::TextStyle::Body.resolve(&ctx.style()),
                                    egui::Color32::YELLOW,
                                );
                            }

                            if let (Some(start), Some(current)) =
                                (self.current_rect_start, self.current_rect_end)
                            {
                                let rect = egui::Rect::from_two_pos(start, current);
                                painter.rect_stroke(
                                    rect,
                                    0.0,
                                    (1.0, egui::Color32::GREEN),
                                    StrokeKind::Middle,
                                );
                            }
                        }
                    } else {
                        ui.label("Kein Bild ausgewählt.");
                    }
                }
            }
        });
    }
}
