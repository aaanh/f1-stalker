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
        let mut palette = if self.id == ThemePresetId::Light {
            iced::theme::Palette::LIGHT
        } else {
            iced::theme::Palette::DARK
        };
        palette.background = self.bg;
        palette.text = self.text;
        palette.primary = self.accent;
        palette.success = self.live;
        palette.danger = self.accent;

        iced::Theme::custom(self.name.clone(), palette)
    }
}
