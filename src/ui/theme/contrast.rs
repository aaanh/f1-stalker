use iced::Color;

use super::palette::ThemePalette;

const AA_NORMAL: f64 = 4.5;
const AA_LARGE: f64 = 3.0;

pub fn contrast_ratio(foreground: Color, background: Color) -> f64 {
    let l1 = relative_luminance(foreground);
    let l2 = relative_luminance(background);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
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

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ContrastIssue {
    pub pair: &'static str,
    pub ratio: f64,
    pub required: f64,
}

pub fn palette_contrast_issues(palette: &ThemePalette) -> Vec<ContrastIssue> {
    let mut issues = Vec::new();

    let checks = [
        ("text on background", palette.text, palette.bg, AA_NORMAL),
        ("text on surface", palette.text, palette.surface, AA_NORMAL),
        ("muted on background", palette.muted, palette.bg, AA_NORMAL),
        ("accent on background", palette.accent, palette.bg, AA_LARGE),
        ("live on background", palette.live, palette.bg, AA_LARGE),
    ];

    for (pair, foreground, background, required) in checks {
        let ratio = contrast_ratio(foreground, background);
        if ratio < required {
            issues.push(ContrastIssue {
                pair,
                ratio,
                required,
            });
        }
    }

    issues
}

pub fn palette_meets_wcag_aa(palette: &ThemePalette) -> bool {
    palette_contrast_issues(palette).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::presets::{palette_for, ThemePresetId};

    #[test]
    fn presets_meet_wcag_aa() {
        for preset in ThemePresetId::all_selectable() {
            if matches!(preset, ThemePresetId::Custom) {
                continue;
            }
            let palette = palette_for(*preset);
            assert!(
                palette_meets_wcag_aa(&palette),
                "{} failed WCAG AA contrast checks",
                preset.label()
            );
        }
    }
}
