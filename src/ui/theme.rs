use iced::Color;

use crate::assets::APP_DISPLAY_NAME;

pub const BG: Color = Color::from_rgb(0.07, 0.07, 0.08);
pub const SURFACE: Color = Color::from_rgb(0.11, 0.11, 0.13);
pub const BORDER: Color = Color::from_rgb(0.22, 0.22, 0.26);
pub const TEXT: Color = Color::from_rgb(0.93, 0.93, 0.95);
pub const MUTED: Color = Color::from_rgb(0.55, 0.55, 0.62);
pub const ACCENT: Color = Color::from_rgb(0.89, 0.11, 0.18);
pub const LIVE: Color = Color::from_rgb(0.13, 0.77, 0.37);

pub const FLAG_GREEN: Color = Color::from_rgb(0.0, 0.72, 0.25);
pub const FLAG_YELLOW: Color = Color::from_rgb(0.95, 0.82, 0.0);
pub const FLAG_RED: Color = Color::from_rgb(0.92, 0.12, 0.15);
pub const FLAG_BLUE: Color = Color::from_rgb(0.15, 0.45, 0.95);
pub const FLAG_BLACK: Color = Color::from_rgb(0.12, 0.12, 0.14);

pub const CHECKER_LIGHT: Color = Color::from_rgb(0.92, 0.92, 0.92);
pub const CHECKER_DARK: Color = Color::from_rgb(0.12, 0.12, 0.12);

pub fn theme() -> iced::Theme {
    iced::Theme::custom(
        APP_DISPLAY_NAME.into(),
        iced::theme::Palette {
            background: BG,
            text: TEXT,
            primary: ACCENT,
            success: LIVE,
            danger: ACCENT,
            ..iced::theme::Palette::DARK
        },
    )
}
