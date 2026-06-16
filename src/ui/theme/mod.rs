#[cfg(test)]
mod contrast;
mod palette;
mod presets;

pub use palette::ThemePalette;
pub use presets::{ThemePresetId, palette_for, palette_for_settings};

use std::sync::{OnceLock, RwLock};

use crate::db::Settings;

static ACTIVE: OnceLock<RwLock<ThemePalette>> = OnceLock::new();

pub fn init_palette(settings: &Settings) {
    let palette = palette_for_settings(settings);
    let lock = ACTIVE.get_or_init(|| RwLock::new(palette.clone()));
    *lock.write().expect("theme palette lock") = palette;
}

pub fn active_palette() -> ThemePalette {
    ACTIVE
        .get_or_init(|| RwLock::new(palette_for(ThemePresetId::Dark)))
        .read()
        .expect("theme palette lock")
        .clone()
}

pub fn iced_theme() -> iced::Theme {
    active_palette().iced_theme()
}

pub fn bg() -> iced::Color {
    active_palette().bg
}
pub fn surface() -> iced::Color {
    active_palette().surface
}
pub fn border() -> iced::Color {
    active_palette().border
}
pub fn text_color() -> iced::Color {
    active_palette().text
}
pub fn muted() -> iced::Color {
    active_palette().muted
}
pub fn accent() -> iced::Color {
    active_palette().accent
}
pub fn live() -> iced::Color {
    active_palette().live
}

pub const FLAG_GREEN: iced::Color = iced::Color::from_rgb(0.0, 0.72, 0.25);
pub const FLAG_YELLOW: iced::Color = iced::Color::from_rgb(0.95, 0.82, 0.0);
pub const FLAG_RED: iced::Color = iced::Color::from_rgb(0.92, 0.12, 0.15);
pub const FLAG_BLUE: iced::Color = iced::Color::from_rgb(0.15, 0.45, 0.95);

pub const CHECKER_LIGHT: iced::Color = iced::Color::from_rgb(0.92, 0.92, 0.92);
pub const CHECKER_DARK: iced::Color = iced::Color::from_rgb(0.12, 0.12, 0.12);
