use crate::{image_data_wrapper::Building, prelude::*};

pub fn connect_walls(
    buildings: &Vec<Building>,
    min_dist_to_connect: f32,
) -> (Vec<Building>, Vec<(f32, f32, f32, f32)>) {
    let mut buildings_without_walls = Vec::new();
    let mut walls = Vec::new();
    let mut wall_lines: Vec<(f32, f32, f32, f32)> = Vec::new();

    for building in buildings.clone() {
        if building.class_name.as_str() == "mauer" {
            walls.push(building);
        } else {
            buildings_without_walls.push(building);
        }
    }

    return (buildings_without_walls, wall_lines);
}

pub fn apply_filter(
    buildings: &Vec<Building>,
    show_normal_buildings: bool,
    show_walls: bool,
    show_defences: bool,
) -> Vec<Building> {
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

    let mut result = Vec::new();

    for building in buildings.iter() {
        let is_wall = building.class_name == "mauer";
        let is_defence = defences.contains(&building.class_name);

        if is_wall && show_walls {
            result.push(building.clone());
        } else if is_defence && show_defences {
            result.push(building.clone());
        } else if !is_wall && !is_defence && show_normal_buildings {
            result.push(building.clone());
        }
    }

    result
}
