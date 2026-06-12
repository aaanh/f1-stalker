use iced::Color;

use super::palette::ThemePalette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ThemePresetId {
    #[default]
    Dark,
    Light,
    Ferrari,
    Mercedes,
    RedBull,
    Mclaren,
    AstonMartin,
    Alpine,
    Williams,
    Haas,
    Sauber,
    Rb,
    Custom,
}

impl ThemePresetId {
    pub fn from_key(key: &str) -> Self {
        match key {
            "light" => Self::Light,
            "ferrari" => Self::Ferrari,
            "mercedes" => Self::Mercedes,
            "red_bull" => Self::RedBull,
            "mclaren" => Self::Mclaren,
            "aston_martin" => Self::AstonMartin,
            "alpine" => Self::Alpine,
            "williams" => Self::Williams,
            "haas" => Self::Haas,
            "sauber" => Self::Sauber,
            "rb" => Self::Rb,
            "custom" => Self::Custom,
            _ => Self::Dark,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Ferrari => "ferrari",
            Self::Mercedes => "mercedes",
            Self::RedBull => "red_bull",
            Self::Mclaren => "mclaren",
            Self::AstonMartin => "aston_martin",
            Self::Alpine => "alpine",
            Self::Williams => "williams",
            Self::Haas => "haas",
            Self::Sauber => "sauber",
            Self::Rb => "rb",
            Self::Custom => "custom",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Ferrari => "Ferrari",
            Self::Mercedes => "Mercedes",
            Self::RedBull => "Red Bull",
            Self::Mclaren => "McLaren",
            Self::AstonMartin => "Aston Martin",
            Self::Alpine => "Alpine",
            Self::Williams => "Williams",
            Self::Haas => "Haas",
            Self::Sauber => "Sauber",
            Self::Rb => "RB",
            Self::Custom => "Custom",
        }
    }

    pub fn all_selectable() -> &'static [Self] {
        &[
            Self::Dark,
            Self::Light,
            Self::Ferrari,
            Self::Mercedes,
            Self::RedBull,
            Self::Mclaren,
            Self::AstonMartin,
            Self::Alpine,
            Self::Williams,
            Self::Haas,
            Self::Sauber,
            Self::Rb,
        ]
    }
}

pub fn palette_for(id: ThemePresetId) -> ThemePalette {
    match id {
        ThemePresetId::Light => palette(
            id,
            rgb(0.96, 0.96, 0.97),
            rgb(0.92, 0.92, 0.94),
            rgb(0.82, 0.82, 0.86),
            rgb(0.12, 0.12, 0.14),
            rgb(0.42, 0.42, 0.48),
            rgb(0.89, 0.11, 0.18),
            rgb(0.0, 0.55, 0.25),
        ),
        ThemePresetId::Ferrari => palette(
            id,
            rgb(0.07, 0.07, 0.08),
            rgb(0.11, 0.11, 0.13),
            rgb(0.22, 0.22, 0.26),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.86, 0.0, 0.0),
            rgb(0.13, 0.77, 0.37),
        ),
        ThemePresetId::Mercedes => palette(
            id,
            rgb(0.06, 0.07, 0.08),
            rgb(0.10, 0.11, 0.12),
            rgb(0.20, 0.22, 0.24),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.0, 0.82, 0.73),
            rgb(0.13, 0.77, 0.37),
        ),
        ThemePresetId::RedBull => palette(
            id,
            rgb(0.05, 0.06, 0.12),
            rgb(0.09, 0.10, 0.18),
            rgb(0.18, 0.20, 0.30),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.18, 0.24, 0.62),
            rgb(0.92, 0.12, 0.15),
        ),
        ThemePresetId::Mclaren => palette(
            id,
            rgb(0.06, 0.07, 0.08),
            rgb(0.10, 0.11, 0.12),
            rgb(0.20, 0.22, 0.24),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(1.0, 0.53, 0.0),
            rgb(0.13, 0.77, 0.37),
        ),
        ThemePresetId::AstonMartin => palette(
            id,
            rgb(0.05, 0.08, 0.07),
            rgb(0.09, 0.12, 0.11),
            rgb(0.18, 0.24, 0.22),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.0, 0.55, 0.45),
            rgb(0.13, 0.77, 0.37),
        ),
        ThemePresetId::Alpine => palette(
            id,
            rgb(0.05, 0.06, 0.10),
            rgb(0.09, 0.10, 0.16),
            rgb(0.18, 0.20, 0.28),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.0, 0.45, 0.95),
            rgb(0.13, 0.77, 0.37),
        ),
        ThemePresetId::Williams => palette(
            id,
            rgb(0.06, 0.07, 0.09),
            rgb(0.10, 0.11, 0.14),
            rgb(0.20, 0.22, 0.28),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.0, 0.35, 0.95),
            rgb(0.13, 0.77, 0.37),
        ),
        ThemePresetId::Haas => palette(
            id,
            rgb(0.07, 0.07, 0.08),
            rgb(0.11, 0.11, 0.13),
            rgb(0.22, 0.22, 0.26),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.75, 0.75, 0.78),
            rgb(0.92, 0.12, 0.15),
        ),
        ThemePresetId::Sauber => palette(
            id,
            rgb(0.06, 0.07, 0.08),
            rgb(0.10, 0.11, 0.12),
            rgb(0.20, 0.22, 0.24),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.0, 0.72, 0.45),
            rgb(0.13, 0.77, 0.37),
        ),
        ThemePresetId::Rb => palette(
            id,
            rgb(0.06, 0.07, 0.09),
            rgb(0.10, 0.11, 0.14),
            rgb(0.20, 0.22, 0.28),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.18, 0.45, 0.95),
            rgb(0.13, 0.77, 0.37),
        ),
        ThemePresetId::Custom | ThemePresetId::Dark => palette(
            id,
            rgb(0.07, 0.07, 0.08),
            rgb(0.11, 0.11, 0.13),
            rgb(0.22, 0.22, 0.26),
            rgb(0.93, 0.93, 0.95),
            rgb(0.55, 0.55, 0.62),
            rgb(0.89, 0.11, 0.18),
            rgb(0.13, 0.77, 0.37),
        ),
    }
}

fn palette(
    id: ThemePresetId,
    bg: Color,
    surface: Color,
    border: Color,
    text: Color,
    muted: Color,
    accent: Color,
    live: Color,
) -> ThemePalette {
    ThemePalette {
        id,
        name: id.label().into(),
        bg,
        surface,
        border,
        text,
        muted,
        accent,
        live,
    }
}

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::from_rgb(r, g, b)
}
