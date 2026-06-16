use crate::ui::theme::ThemePresetId;

use crate::db::CustomTheme;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub season_year: i32,
    pub timezone: String,
    pub first_run_complete: bool,
    pub theme_id: ThemePresetId,
    pub custom_theme: CustomTheme,
    pub background_on_close: bool,
    pub include_testing: bool,
    pub notifications_enabled: bool,
    pub notify_standings: bool,
    pub notify_sessions: bool,
    pub session_reminder_minutes: i64,
    pub rival_driver_first: i64,
    pub rival_driver_second: i64,
    pub rival_compare_active: bool,
    pub font_scale: f32,
    pub standings_tab: crate::domain::ChampionshipTab,
    pub standings_mode: crate::domain::ChartMode,
}

impl Settings {
    pub fn effective_season(&self, fallback: i32) -> i32 {
        if self.season_year == 0 {
            fallback
        } else {
            self.season_year
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            season_year: 0,
            timezone: crate::db::schema::DEFAULT_TIMEZONE.into(),
            first_run_complete: false,
            theme_id: ThemePresetId::Dark,
            custom_theme: CustomTheme::default(),
            background_on_close: false,
            include_testing: false,
            notifications_enabled: true,
            notify_standings: true,
            notify_sessions: false,
            session_reminder_minutes: 60,
            rival_driver_first: 0,
            rival_driver_second: 0,
            rival_compare_active: false,
            font_scale: crate::ui::layout::FONT_SCALE_DEFAULT,
            standings_tab: crate::domain::ChampionshipTab::Drivers,
            standings_mode: crate::domain::ChartMode::Championship,
        }
    }
}
