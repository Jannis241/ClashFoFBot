use crate::prelude::*;

pub fn run_tests() {
    println!("Runnings tests..");

    let buildings = image_data_wrapper::get_buildings(Path::new("images/fufu.png"));
    println!("Buildings: {:?}", buildings);

    let screenshot = screener::make_screenshot(0);
    screenshot.save("images/test.png").unwrap()
}
