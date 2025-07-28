pub mod bot_actions;
pub mod image_data_wrapper;
mod prelude;
mod screenshot;
mod settings_manager;
mod tests;
mod ui;

fn main() {
    tests::run_tests();
}
