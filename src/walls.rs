use crate::image_data_wrapper::Building;
use crate::prelude::*;

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

fn get_near_walls(wall: &Building, walls: &Vec<Building>, dist: f32) -> Vec<Building> {
    let mut res = Vec::new();
    for w in walls {
        if w.bounding_box == wall.bounding_box {
            continue;
        }

        if calc_dist_between_walls(w, wall) <= dist {
            res.push(w.clone());
        }
    }
    res
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

    for (i, wall) in walls.iter().enumerate() {
        for (j, near_wall) in walls.iter().enumerate() {
            if i == j {
                continue;
            }

            if j <= i {
                continue;
            }

            if calc_dist_between_walls(wall, near_wall) <= min_dist_to_connect {
                let line = create_line_between_walls(wall, near_wall);
                wall_lines.push(line);
            }
        }
    }

    (buildings_without_walls, wall_lines)
}
