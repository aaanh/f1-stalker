use iced::Color;

use super::presets::{self, ThemePresetId};

#[derive(Debug, Clone, PartialEq)]
pub struct ThemePalette {
    pub id: ThemePresetId,
    pub name: String,
    pub bg: Color,
    pub surface: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub live: Color,
}

impl Default for ThemePalette {
    fn default() -> Self {
        presets::palette_for(ThemePresetId::Dark)
    }
}

impl ThemePalette {
    pub fn iced_theme(&self) -> iced::Theme {
        let base = if self.id == ThemePresetId::Light {
            iced::theme::Palette::LIGHT
        } else {
            iced::theme::Palette::DARK
        };

        iced::Theme::custom(
            self.name.clone(),
            iced::theme::Palette {
                background: self.bg,
                text: self.text,
                primary: self.accent,
                success: self.live,
                danger: self.accent,
                ..base
            },
        )
    }
}
