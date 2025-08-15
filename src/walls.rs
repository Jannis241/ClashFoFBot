use crate::image_data_wrapper::Building;
use crate::prelude::*;
use crate::prelude::*;
use std::collections::HashMap;

type BoundingBox = (f32, f32, f32, f32);
type Point = (f32, f32);

#[derive(PartialEq, Clone)]
struct Wall {
    bbox: BoundingBox,
    processed: bool,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Direction {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

fn angle_diff(a: f32, b: f32) -> f32 {
    let mut diff = (a - b) % 360.0;
    if diff < -180.0 {
        diff += 360.0;
    }
    if diff > 180.0 {
        diff -= 360.0;
    }
    diff.abs()
}

impl Wall {
    fn new(bbox: BoundingBox) -> Self {
        Self {
            bbox,
            processed: false,
        }
    }

    fn get_center_of_bbox(&self) -> Point {
        let (x_min, y_min, x_max, y_max) = self.bbox;
        ((x_min + x_max) / 2.0, (y_min + y_max) / 2.0)
    }

    fn distance(p1: Point, p2: Point) -> f32 {
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        (dx * dx + dy * dy).sqrt()
    }

    fn calc_dist_to_other(&self, other: &Wall) -> f32 {
        Wall::distance(self.get_center_of_bbox(), other.get_center_of_bbox())
    }

    fn create_line_to_other(&self, other: &Wall) -> (Point, Point) {
        (self.get_center_of_bbox(), other.get_center_of_bbox())
    }

    fn get_dir_to_other(&self, other: &Wall, angle_tolerance_deg: f32) -> Option<Direction> {
        let my_center = self.get_center_of_bbox();
        let other_center = other.get_center_of_bbox();

        let dx = other_center.0 - my_center.0;
        let dy = other_center.1 - my_center.1;

        // Winkel in Grad (y nach unten → evtl. invertieren falls y-Achse oben 0 hat)
        let angle_deg = dy.atan2(dx).to_degrees();

        // Definierte Zielwinkel für jede Richtung
        let directions = [
            (Direction::TopRight, -45.0),
            (Direction::BottomRight, 45.0),
            (Direction::BottomLeft, 135.0),
            (Direction::TopLeft, -135.0),
        ];

        for (dir, target_angle) in directions {
            if angle_diff(angle_deg, target_angle) <= angle_tolerance_deg {
                return Some(dir);
            }
        }

        None
    }

    fn set_processed(&mut self, state: bool) {
        self.processed = state;
    }

    fn get_neighbors<'a>(
        &self,
        all_walls: &'a [Wall],
        dist: f32,
        angle_tolerance_deg: f32,
    ) -> Vec<&'a Wall> {
        let mut neighbors: HashMap<Direction, &'a Wall> = HashMap::new();
        let mut distances: HashMap<Direction, f32> = HashMap::new();

        for wall in all_walls {
            if wall.processed || wall == self {
                continue;
            }

            let distance = self.calc_dist_to_other(wall);
            if distance > dist {
                continue;
            }

            let Some(direction) = self.get_dir_to_other(wall, angle_tolerance_deg) else {
                continue;
            };

            let current_best = *distances.get(&direction).unwrap_or(&f32::MAX);

            if distance < current_best {
                distances.insert(direction.clone(), distance);
                neighbors.insert(direction.clone(), wall);
            }
        }

        neighbors.into_values().collect()
    }
}

pub fn connect_walls(
    buildings: &[Building],
    min_dist_to_connect: f32,
    angle_tolerance_deg: f32,
) -> (Vec<Building>, Vec<((f32, f32), (f32, f32))>) {
    let mut buildings_without_walls = Vec::new();
    let mut walls = Vec::new();
    let mut wall_lines: Vec<((f32, f32), (f32, f32))> = Vec::new();

    // Mauern von normalen Gebäuden trennen
    for building in buildings.iter().cloned() {
        if building.class_name.as_str().contains("mauer") {
            walls.push(Wall::new(building.bounding_box));
        } else {
            buildings_without_walls.push(building);
        }
    }

    // Jede Mauer verbinden
    for i in 0..walls.len() {
        if walls[i].processed {
            continue;
        }

        let neighbors = walls[i].get_neighbors(&walls, min_dist_to_connect, angle_tolerance_deg);

        if neighbors.is_empty() {
            // Einzelne Mauer: kurzer Dummy-Strich
            let line = walls[i].create_line_to_other(&walls[i]);
            wall_lines.push(line);
        } else {
            for neighbor in neighbors {
                wall_lines.push(walls[i].create_line_to_other(neighbor));
            }
        }

        walls[i].set_processed(true);
    }

    (buildings_without_walls, wall_lines)
}

// ----------------------- chat gpt kocht??????????? -------------------------------------------

pub fn center_x(bbox: (f32, f32, f32, f32)) -> f32 {
    (bbox.0 + bbox.2) / 2.0
}

pub fn center_y(bbox: (f32, f32, f32, f32)) -> f32 {
    (bbox.1 + bbox.3) / 2.0
}

fn bbox_width(bbox: (f32, f32, f32, f32)) -> f32 {
    bbox.2 - bbox.0
}

/// Gruppiert Mauern in Reihen anhand ähnlicher Y-Koordinate
pub fn group_walls_by_row(walls: &[Building], threshold45deg: f32) -> Vec<Vec<Building>> {
    let mut rows: Vec<Vec<Building>> = Vec::new();

    'outer: for wall in walls.iter() {
        let cy = center_y(wall.bounding_box);
        let cx = center_x(wall.bounding_box);

        for row in rows.iter_mut() {
            // Prüfe die y-Differenz zur ersten Mauer in der Reihe
            let row_cy = center_y(row[0].bounding_box);
            let row_cx = center_x(row[0].bounding_box);
            let diff_y = (cy - row_cy).abs();
            let diff_x = (cx - row_cx).abs();

            let angle = (diff_x.atan2(diff_y).to_degrees() + 360.) % 90.;

            if angle - 25. <= threshold45deg || angle - 75. <= threshold45deg {
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

pub fn find_hidden_walls(
    buildings: &Vec<Building>,
    threshold45deg: f32, // wie nah müssen Mauern in y sein, um in einer Reihe zu sein
    max_gap: f32,        // max Abstand zwischen Mauern, um Lücke als potenzielle Mauer zu sehen
    building_y_threshold: f32, // max vertikaler Abstand Gebäude unter Mauerreihe
) -> Vec<Building> {
    // Parameter:

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
    let rows = group_walls_by_row(&walls, threshold45deg);

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
