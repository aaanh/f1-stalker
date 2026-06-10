mod app;
mod assets;
mod data;
mod db;
mod debug;
mod domain;
mod state;
mod ui;

fn main() -> iced::Result {
    assets::early_platform_init();
    app::run()
}
