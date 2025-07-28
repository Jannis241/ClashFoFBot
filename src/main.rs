mod bot_actions;
mod image_data_wrapper;
mod prelude;
mod screenshot;
mod settings_manager;
mod ui;

fn main() {
    // test
    image_data_wrapper::get_buildings(Path::new("images/fufu.png"));
}
