struct BoundingBox {
    top_left: (f32, f32),
    top_right: (f32,f32),

}

struct Building {
    class_id: i32,
    class_name: i32
    confidence: f32,
    bounding_box: BoundingBox,
}


fn get_buildings(screeenshot: &Path) -> Vec<Building> {}

