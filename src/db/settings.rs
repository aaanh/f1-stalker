#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub season_year: i32,
    pub timezone: String,
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
        }
    }
}
