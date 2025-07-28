use crate::prelude::*;
pub fn run_tests() {
    println!("Runnings tests..");

    let buildings = image_data_wrapper::get_buildings(Path::new("images/fufu.png"));
    println!("Buildings: {:?}", buildings);

    let screen = screenshot::Screenshot::get_screenshot();
    let area = screen.get_area(500, 500, 400, 200);
    area.save(&Path::new("images/test_area.png"));
    screen.save(&Path::new("images/test.png"));
}
