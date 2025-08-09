use crate::{image_data_wrapper::Building, prelude::*};

fn get_near_walls(wall: &Building, walls: &Vec<Building>, dist: f32) -> Vec<Building> {
    let mut res = Vec::new();
    for w in walls {
        if calc_dist_between_walls(w, wall) <= dist {
            // todo: verhindern, dass man die wall mit sich selber connected, da die wall selber ja
            // auch in walls drinne ist.
            res.push(w.clone());
        }
    }
    return res;
}

fn center_of_box(bbox: (f32, f32, f32, f32)) -> (f32, f32) {
    let (x_min, y_min, x_max, y_max) = bbox;
    ((x_min + x_max) / 2.0, (y_min + y_max) / 2.0)
}

fn distance(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    (dx * dx + dy * dy).sqrt()
}

fn calc_dist_between_walls(wall1: &Building, wall2: &Building) -> f32 {
    let bounding_box1 = wall1.bounding_box;
    let bounding_box2 = wall2.bounding_box;

    let center1 = center_of_box(bounding_box1);
    let center2 = center_of_box(bounding_box2);

    let dist = distance(center1, center2);

    return dist;
}

fn create_line_between_walls(wall1: &Building, wall2: &Building) -> ((f32, f32), (f32, f32)) {
    let bounding_box1 = wall1.bounding_box;
    let bounding_box2 = wall2.bounding_box;

    let center1 = center_of_box(bounding_box1);
    let center2 = center_of_box(bounding_box2);

    return (center1, center2);
}

pub fn connect_walls(
    buildings: &Vec<Building>,
    min_dist_to_connect: f32,
) -> (Vec<Building>, Vec<((f32, f32), (f32, f32))>) {
    let mut buildings_without_walls = Vec::new();
    let mut walls = Vec::new();
    let mut wall_lines: Vec<((f32, f32), (f32, f32))> = Vec::new();

    for building in buildings.clone() {
        if building.class_name.as_str() == "mauer" {
            walls.push(building);
        } else {
            buildings_without_walls.push(building);
        }
    }

    // todo: doppelete connections verhindern, da man jetzt durch alle mauern geht, wird jede mauer
    // mit einander mehrmal connected (aber kein bock grade das zu machen)
    for wall in &walls {
        let near_walls = get_near_walls(&wall, &walls, min_dist_to_connect);

        for near_wall in near_walls {
            let line = create_line_between_walls(&wall, &near_wall);

            wall_lines.push(line);
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
