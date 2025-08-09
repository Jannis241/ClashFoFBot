use crate::filter_buildings::apply_filter;
use crate::image_data_wrapper::*;
use crate::prelude::*;

pub fn run_tests() {
    test_filter();
}
pub fn test_filter() {
    let buildings = vec![
        Building {
            class_id: 0,
            class_name: "bogenschützenturm".to_string(),
            confidence: 0.9,
            bounding_box: (0.0, 0.0, 1.0, 1.0),
        }, // defence
        Building {
            class_id: 31,
            class_name: "mauer".to_string(),
            confidence: 0.8,
            bounding_box: (1.0, 1.0, 2.0, 2.0),
        }, // wall
        Building {
            class_id: 27,
            class_name: "goldlager".to_string(),
            confidence: 0.7,
            bounding_box: (2.0, 2.0, 3.0, 3.0),
        }, // normal building
    ];

    fn classes(buildings: &Vec<Building>) -> Vec<String> {
        buildings.iter().map(|b| b.class_name.clone()).collect()
    }

    fn print_result(test_name: &str, expected: Vec<&str>, actual: Vec<String>) {
        if expected == actual.iter().map(|s| s.as_str()).collect::<Vec<_>>() {
            println!("{} PASSED", test_name);
        } else {
            println!("{} FAILED", test_name);
            println!("  Expected: {:?}", expected);
            println!("  Got:      {:?}", actual);
        }
    }

    // Test 1: show all false -> Ergebnis sollte leer sein
    let filtered = apply_filter(&buildings, false, false, false);
    print_result(
        "Test 1: show_normal_buildings=false, show_walls=false, show_defences=false",
        vec![],
        classes(&filtered),
    );

    // Test 2: nur normale Gebäude zeigen
    let filtered = apply_filter(&buildings, true, false, false);
    print_result(
        "Test 2: show_normal_buildings=true, show_walls=false, show_defences=false",
        vec!["goldlager"],
        classes(&filtered),
    );

    // Test 3: nur Mauern zeigen
    let filtered = apply_filter(&buildings, false, true, false);
    print_result(
        "Test 3: show_normal_buildings=false, show_walls=true, show_defences=false",
        vec!["mauer"],
        classes(&filtered),
    );

    // Test 4: nur Verteidigungen zeigen
    let filtered = apply_filter(&buildings, false, false, true);
    print_result(
        "Test 4: show_normal_buildings=false, show_walls=false, show_defences=true",
        vec!["bogenschützenturm"],
        classes(&filtered),
    );

    // Test 5: alle drei true -> alle Gebäude sollen kommen
    let filtered = apply_filter(&buildings, true, true, true);
    print_result(
        "Test 5: show_normal_buildings=true, show_walls=true, show_defences=true",
        vec!["bogenschützenturm", "mauer", "goldlager"],
        classes(&filtered),
    );

    // Test 6: Mauern und Verteidigungen, keine normalen
    let filtered = apply_filter(&buildings, false, true, true);
    print_result(
        "Test 6: show_normal_buildings=false, show_walls=true, show_defences=true",
        vec!["bogenschützenturm", "mauer"],
        classes(&filtered),
    );

    // Test 7: Normale und Verteidigungen, keine Mauern
    let filtered = apply_filter(&buildings, true, false, true);
    print_result(
        "Test 7: show_normal_buildings=true, show_walls=false, show_defences=true",
        vec!["bogenschützenturm", "goldlager"],
        classes(&filtered),
    );

    // Test 8: Normale und Mauern, keine Verteidigungen
    let filtered = apply_filter(&buildings, true, true, false);
    print_result(
        "Test 8: show_normal_buildings=true, show_walls=true, show_defences=false",
        vec!["mauer", "goldlager"],
        classes(&filtered),
    );

    println!("Test run finished.");
}
