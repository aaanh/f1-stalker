use iced::Color;
use serde::{Deserialize, Serialize};

use crate::ui::theme::{palette_for, ThemePalette, ThemePresetId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomTheme {
    pub background: String,
    pub surface: String,
    pub accent: String,
}

impl Default for CustomTheme {
    fn default() -> Self {
        let base = palette_for(ThemePresetId::Dark);
        Self {
            background: color_to_hex(&base.bg),
            surface: color_to_hex(&base.surface),
            accent: color_to_hex(&base.accent),
        }
    }
}

impl CustomTheme {
    pub fn to_palette(&self) -> Option<ThemePalette> {
        let bg = parse_hex_color(&self.background)?;
        let surface = parse_hex_color(&self.surface)?;
        let accent = parse_hex_color(&self.accent)?;
        let dark = relative_luminance(bg) < 0.5;

        let (text, muted, border, live) = if dark {
            (
                Color::from_rgb(0.93, 0.93, 0.95),
                Color::from_rgb(0.55, 0.55, 0.62),
                Color::from_rgb(0.22, 0.22, 0.26),
                Color::from_rgb(0.13, 0.77, 0.37),
            )
        } else {
            (
                Color::from_rgb(0.12, 0.12, 0.14),
                Color::from_rgb(0.42, 0.42, 0.48),
                Color::from_rgb(0.82, 0.82, 0.86),
                Color::from_rgb(0.0, 0.55, 0.25),
            )
        };

        Some(ThemePalette {
            id: ThemePresetId::Custom,
            name: "Custom".into(),
            bg,
            surface,
            border,
            text,
            muted,
            accent,
            live,
        })
    }
}

pub fn parse_hex_color(value: &str) -> Option<Color> {
    let trimmed = value.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
    let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
    let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
    Some(Color::from_rgb8(r, g, b))
}

pub fn color_to_hex(color: &Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
    )
}

fn relative_luminance(color: Color) -> f64 {
    fn channel(value: f32) -> f64 {
        let value = value as f64;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_hex_colors() {
        let theme = CustomTheme::default();
        let palette = theme.to_palette().expect("palette");
        assert_eq!(color_to_hex(&palette.bg), theme.background);
    }
}
