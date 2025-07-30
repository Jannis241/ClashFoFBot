use crate::{image_data_wrapper::Building, image_data_wrapper::YoloModel, prelude::*};

pub fn run_tests() {
    // println!("Runnings tests..");
    ui::start_ui();

    // image_data_wrapper::create_model(String::from("dede"), image_data_wrapper::YoloModel::yolov8n);
    // image_data_wrapper::train_model(String::from("dede"), 1);
    // image_data_wrapper::get_buildings(String::from("dede"), &Path::new("images/fufu.png"));
    // image_data_wrapper::delete_model(String::from("dede"));
    //
    // image_data_wrapper::create_model("test_model".into(), image_data_wrapper::YoloModel::yolov8n);
    // image_data_wrapper::train_model(String::from("rongor"), 1);
    // image_data_wrapper::get_buildings(String::from("fege"), &Path::new("images/fufu.png"));
    // image_data_wrapper::get_buildings(String::from("dede"), &Path::new("imaaaaaages/fufu.png"));
    // image_data_wrapper::get_buildings(String::from("fede"), &Path::new("imaaaaaages/fufu.png"));
    // image_data_wrapper::create_model(
    //     String::from("ljlllj"),
    //     image_data_wrapper::YoloModel::yolov8n,
    // );
    //
    // image_data_wrapper::create_model(
    //     String::from("test_model"),
    //     image_data_wrapper::YoloModel::yolov8n,
    // );
    // let buildings =
    //     image_data_wrapper::get_buildings(String::from("test_model"), Path::new("images/fufu.png"));
    // image_data_wrapper::train_model(1);
    // image_data_wrapper::train_model(50);
    // image_data_wrapper::train_model(50);
    // image_data_wrapper::train_model(50);

    // let buildings = image_data_wrapper::get_buildings(Path::new("images/fufu.png"));
    // println!("Buildings: {:?}", buildings);
    //
    // image_data_wrapper::train_model("test_model".into(), 1);
    // image_data_wrapper::train_model("test_model".into(), 1);
    // image_data_wrapper::delete_model(String::from("test_model"));

    // let screenshot = screener::make_screenshot(0);
    // screenshot.save("images/test.png").unwrap()
    //     // ✅ Modell "dede" erstellen, trainieren, predicten und löschen
    // image_data_wrapper::create_model("dede".into(), YoloModel::yolov8n);
    // image_data_wrapper::train_model("dede".into(), 1);
    // let _ = image_data_wrapper::get_buildings("dede".into(), &Path::new("images/fufu.png"));
    // image_data_wrapper::delete_model("dede".into());
    //
    // // ❌ Modell mit gleichem Namen nochmal erstellen (sollte fehlschlagen oder warnen)
    // image_data_wrapper::create_model("test_model".into(), YoloModel::yolov8n);
    // image_data_wrapper::create_model("test_model".into(), YoloModel::yolov8n); // erwartet Fehler
    //
    // // ❌ Training eines nicht existierenden Modells
    // image_data_wrapper::train_model("rongor".into(), 1); // erwartet Fehler
    //
    // // ❌ Vorhersage mit nicht existierendem Modell
    // let _ = image_data_wrapper::get_buildings("fege".into(), &Path::new("images/fufu.png")); // erwartet Fehler
    //
    // // ❌ Bildpfad existiert nicht
    // let _ = image_data_wrapper::get_buildings("dede".into(), &Path::new("imaaaaaages/fufu.png")); // erwartet Fehler
    //
    // // ❌ Modell & Bildpfad existieren beide nicht
    // let _ = image_data_wrapper::get_buildings("fede".into(), &Path::new("imaaaaaages/fufu.png")); // erwartet Fehler
    //
    // // ✅ Neues Modell korrekt erstellen
    // image_data_wrapper::create_model("ljlllj".into(), YoloModel::yolov8n);
    //
    // // ✅ Korrekte Vorhersage mit bestehendem Modell
    // let buildings =
    //     image_data_wrapper::get_buildings("test_model".into(), &Path::new("images/fufu.png"));
    // println!("Buildings: {:?}", buildings);
    //
    // // ✅ Mehrfaches Training desselben Modells
    // image_data_wrapper::train_model("test_model".into(), 1);
    // image_data_wrapper::train_model("test_model".into(), 1);
    //
    // // ✅ Modell löschen
    // image_data_wrapper::delete_model("test_model".into());
    //
    // // ❌ Vorhersage nach dem Löschen (sollte fehlschlagen)
    // let _ = image_data_wrapper::get_buildings("test_model".into(), &Path::new("images/fufu.png")); // erwartet Fehler
    //
    // // ❌ Löschen eines nicht existierenden Modells (sollte sauber behandelt werden)
    // image_data_wrapper::delete_model("nonexistent_model".into());

    // ❌ Ungültiges YoloModel (nur wenn du eigene YoloModel-Enum-Validierung einbauen willst)
    // image_data_wrapper::create_model("invalid".into(), YoloModel::Invalid); // nicht direkt testbar ohne ungültige Enum-Variante
}
