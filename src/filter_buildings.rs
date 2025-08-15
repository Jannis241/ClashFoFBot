use std::usize;

use crate::{image_data_wrapper::Building, prelude::*};

fn get_similarity(bbox1: (f32, f32, f32, f32), bbox2: (f32, f32, f32, f32)) -> f32 {
    let (x1_min, y1_min, x1_max, y1_max) = bbox1;
    let (x2_min, y2_min, x2_max, y2_max) = bbox2;

    let x_left = x1_min.max(x2_min);
    let y_top = y1_min.max(y2_min);
    let x_right = x1_max.min(x2_max);
    let y_bottom = y1_max.min(y2_max);

    if x_right < x_left || y_bottom < y_top {
        return 0.0; // Kein Schnitt
    }

    let intersection_area = (x_right - x_left) * (y_bottom - y_top);

    let bbox1_area = (x1_max - x1_min) * (y1_max - y1_min);
    let bbox2_area = (x2_max - x2_min) * (y2_max - y2_min);

    intersection_area / (bbox1_area + bbox2_area - intersection_area)
}

fn average_bbox(bbox1: (f32, f32, f32, f32), bbox2: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    (
        (bbox1.0 + bbox2.0) / 2.0, // top_left_x
        (bbox1.1 + bbox2.1) / 2.0, // top_left_y
        (bbox1.2 + bbox2.2) / 2.0, // bottom_right_x
        (bbox1.3 + bbox2.3) / 2.0, // bottom_right_y
    )
}

pub fn connect_level_and_buildings(
    buildings: &Vec<Building>,
    level: &Vec<Building>,
    min_iou: f32,
) -> Vec<Building> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for building in buildings {
        for lvl in level {
            let iou = get_similarity(building.bounding_box, lvl.bounding_box);
            if iou >= min_iou {
                let avg_bbox = average_bbox(building.bounding_box, lvl.bounding_box);

                let key = (
                    building.class_name.clone(),
                    lvl.class_name.clone(),
                    bbox_to_key(avg_bbox),
                );

                if !seen.contains(&key) {
                    seen.insert(key);
                    result.push(Building {
                        class_id: building.class_id,
                        class_name: format!("{}{}", building.class_name, lvl.class_name),
                        confidence: (building.confidence + lvl.confidence) / 2.,
                        bounding_box: avg_bbox,
                    });
                }
            }
        }
    }

    dbg!(result.len());

    result
}

fn bbox_to_key(bbox: (f32, f32, f32, f32)) -> (i32, i32, i32, i32) {
    (
        (bbox.0 * 1000.0).round() as i32,
        (bbox.1 * 1000.0).round() as i32,
        (bbox.2 * 1000.0).round() as i32,
        (bbox.3 * 1000.0).round() as i32,
    )
}

pub fn get_building_type(building: &Building) -> (bool, bool, bool) {
    let defences: Vec<String> = vec![
        "bogenschützenturm".to_string(),
        "minenwerfer".to_string(),
        "multibogenschützenturm".to_string(),
        "magierturm".to_string(),
        "tesla".to_string(),
        "luftabwehr".to_string(),
        "querschlägerkanone".to_string(),
        "xbogenluft".to_string(),
        "entwicklungsturmkanone".to_string(),
        "feuerspeier".to_string(),
        "bombenturm".to_string(),
        "warden".to_string(),
        "queen".to_string(),
        "king".to_string(),
        "infernoturmmulti".to_string(),
        "giftzauberturm".to_string(),
        "streukatapult".to_string(),
        "fegerO".to_string(),
        "monolyth".to_string(),
        "wutzauberturm".to_string(),
        "unsichtbarkeitszauberturm".to_string(),
        "kanone".to_string(),
        "adlerartillerie".to_string(),
        "infernoturmeinzel".to_string(),
        "xbogenboden".to_string(),
        "fegerOR".to_string(),
        "fegerR".to_string(),
        "fegerUR".to_string(),
        "fegerU".to_string(),
        "fegerUL".to_string(),
        "fegerL".to_string(),
        "fegerOL".to_string(),
        "entwicklungsturmbogenschützenturm".to_string(),
    ];
    let is_wall = building.class_name.contains("mauer");
    let is_defence = defences
        .iter()
        .any(|name| building.class_name.contains(name));
    let is_normal = !is_wall && !is_defence && building.class_name.parse::<usize>().is_err();

    if is_wall {
        (true, false, false)
    } else if is_defence {
        (false, true, false)
    } else if is_normal {
        (false, false, true)
    } else {
        (false, false, false)
    }
}

pub fn apply_filter(
    buildings: &Vec<Building>,
    show_normal_buildings: bool,
    show_walls: bool,
    show_defences: bool,
) -> Vec<Building> {
    let mut result = Vec::new();

    for building in buildings.iter() {
        let (is_wall, is_defence, is_normal) = get_building_type(building);

        if is_wall && show_walls {
            result.push(building.clone());
        } else if is_defence && show_defences {
            result.push(building.clone());
        } else if is_normal && show_normal_buildings {
            result.push(building.clone());
        }
    }

    result
}
