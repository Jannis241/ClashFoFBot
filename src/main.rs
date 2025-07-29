use crate::prelude::*;
mod bot_actions;
mod debug;
mod image_data_wrapper;
mod prelude;
mod screener;
mod settings_manager;
mod ui;

fn main() {
    ui::start_ui();
    debug::run_tests();
}
