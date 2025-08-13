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

// ----------------------- chat gpt kocht??????????? -------------------------------------------

fn center_x(bbox: (f32, f32, f32, f32)) -> f32 {
    (bbox.0 + bbox.2) / 2.0
}

fn center_y(bbox: (f32, f32, f32, f32)) -> f32 {
    (bbox.1 + bbox.3) / 2.0
}

fn bbox_width(bbox: (f32, f32, f32, f32)) -> f32 {
    bbox.2 - bbox.0
}

/// Gruppiert Mauern in Reihen anhand ähnlicher Y-Koordinate
fn group_walls_by_row(walls: &[Building], y_threshold: f32) -> Vec<Vec<Building>> {
    let mut rows: Vec<Vec<Building>> = Vec::new();

    'outer: for wall in walls.iter() {
        let cy = center_y(wall.bounding_box);
        for row in rows.iter_mut() {
            // Prüfe die y-Differenz zur ersten Mauer in der Reihe
            let row_cy = center_y(row[0].bounding_box);
            if (cy - row_cy).abs() <= y_threshold {
                row.push(wall.clone());
                continue 'outer;
            }
        }
        // Neue Reihe anlegen
        rows.push(vec![wall.clone()]);
    }

    rows
}

/// Prüft, ob unterhalb eines x-Intervalls ein Gebäude liegt (overlap und y größer)
fn building_below(
    buildings: &[Building],
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_threshold: f32,
) -> bool {
    for b in buildings.iter() {
        let (bx_min, by_min, bx_max, by_max) = b.bounding_box;

        // Überprüfe x-Overlap
        let x_overlap = (bx_min < x_max) && (bx_max > x_min);

        // Prüfe ob Gebäude y-Min unterhalb der Mauer y-Min liegt (also Gebäude darunter)
        let below = by_min > y_min && (by_min - y_min) <= y_threshold;

        if x_overlap && below {
            return true;
        }
    }
    false
}

/// Findet Lücken zwischen Mauern in einer Reihe, sortiert nach x
/// Rückgabe: Vec<(x_start, x_end)> der Lücken
fn find_gaps_in_row(row: &[Building], max_gap: f32) -> Vec<(f32, f32)> {
    if row.len() < 2 {
        return vec![];
    }

    // Sortiere nach x_min
    let mut sorted = row.to_vec();
    sorted.sort_by(|a, b| a.bounding_box.0.partial_cmp(&b.bounding_box.0).unwrap());

    let mut gaps = Vec::new();

    for i in 0..(sorted.len() - 1) {
        let right_wall = &sorted[i];
        let left_wall = &sorted[i + 1];

        let gap = left_wall.bounding_box.0 - right_wall.bounding_box.2; // Abstand zwischen rechter und linker Mauer

        if gap > 0.0 && gap <= max_gap {
            gaps.push((right_wall.bounding_box.2, left_wall.bounding_box.0));
        }
    }

    gaps
}

/// Berechnet die bbox einer Mauer, die in einer Lücke vermutet wird.
/// Wir nehmen die mittlere y-Position der Reihe und geben eine Bounding Box mit Standardbreite.
fn bbox_for_hidden_wall(
    x_start: f32,
    x_end: f32,
    row_y: f32,
    wall_width: f32,
) -> (f32, f32, f32, f32) {
    let center_x = (x_start + x_end) / 2.0;
    let half_width = wall_width / 2.0;

    // Höhe der Mauer approximieren als 1/2 der Breite (kann angepasst werden)
    let height = wall_width * 0.5;
    let y_min = row_y - height / 2.0;
    let y_max = row_y + height / 2.0;

    (center_x - half_width, y_min, center_x + half_width, y_max)
}

pub fn find_hidden_walls(buildings: &Vec<Building>) -> Vec<Building> {
    // Parameter:
    let y_threshold = 10.0; // wie nah müssen Mauern in y sein, um in einer Reihe zu sein
    let max_gap = 15.0; // max Abstand zwischen Mauern, um Lücke als potenzielle Mauer zu sehen
    let building_y_threshold = 20.0; // max vertikaler Abstand Gebäude unter Mauerreihe
    let default_wall_confidence = 0.7; // Confidence für vermutete Mauern

    // 1. Mauern und andere Gebäude trennen
    let walls: Vec<Building> = buildings
        .iter()
        .filter(|b| b.class_id == 56)
        .cloned()
        .collect();

    let other_buildings: Vec<Building> = buildings
        .iter()
        .filter(|b| b.class_id != 56)
        .cloned()
        .collect();

    if walls.is_empty() {
        return Vec::new();
    }

    // 2. Gruppiere Mauern in Reihen nach y
    let rows = group_walls_by_row(&walls, y_threshold);

    // Wir nehmen die durchschnittliche Breite einer Mauer als Standard-Breite
    let avg_wall_width = walls
        .iter()
        .map(|w| bbox_width(w.bounding_box))
        .sum::<f32>()
        / walls.len() as f32;

    let mut hidden_walls = Vec::new();

    // 3. Für jede Reihe: Lücken suchen und prüfen, ob Gebäude darunter sind
    for row in rows.iter() {
        // Sortiere Mauern der Reihe nach x für Lückensuche
        let mut sorted_row = row.clone();
        sorted_row.sort_by(|a, b| a.bounding_box.0.partial_cmp(&b.bounding_box.0).unwrap());

        let gaps = find_gaps_in_row(&sorted_row, max_gap);

        // y-Mittelpunkt der Reihe als Anhaltspunkt
        let row_y = sorted_row
            .iter()
            .map(|w| center_y(w.bounding_box))
            .sum::<f32>()
            / sorted_row.len() as f32;

        for (gap_start, gap_end) in gaps {
            // Prüfe, ob ein Gebäude direkt unterhalb liegt (x overlap & y tiefer)
            if building_below(
                &other_buildings,
                gap_start,
                gap_end,
                row_y,
                building_y_threshold,
            ) {
                // Vermutete Mauer bbox
                let bbox = bbox_for_hidden_wall(gap_start, gap_end, row_y, avg_wall_width);

                hidden_walls.push(Building {
                    class_id: 56,
                    class_name: "mauer".to_string(),
                    bounding_box: bbox,
                    confidence: default_wall_confidence,
                });
            }
        }
    }

    hidden_walls
}
