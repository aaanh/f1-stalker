mod app;
mod assets;
mod data;
mod db;
mod debug;
mod domain;
mod platform;
mod state;
mod ui;

fn main() -> iced::Result {
    assets::early_platform_init();
    if platform::request_focus_if_running() {
        return Ok(());
    }
    app::run()
}
