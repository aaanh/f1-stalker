const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pinned_drivers (
    driver_number INTEGER PRIMARY KEY NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS cache (
    cache_key TEXT PRIMARY KEY NOT NULL,
    payload TEXT NOT NULL,
    fetched_at TEXT NOT NULL,
    ttl_secs INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS asset_cache (
    url TEXT PRIMARY KEY NOT NULL,
    file_name TEXT NOT NULL UNIQUE,
    fetched_at TEXT NOT NULL,
    ttl_secs INTEGER NOT NULL,
    failed INTEGER NOT NULL DEFAULT 0
);
";

pub const SETTING_SEASON_YEAR: &str = "season_year";
pub const SETTING_TIMEZONE: &str = "timezone";
pub const DEFAULT_TIMEZONE: &str = "system";
pub const CALENDAR_CACHE_TTL_SECS: i64 = 21_600;
pub const DRIVERS_CACHE_TTL_SECS: i64 = 21_600;
pub const CHAMPIONSHIP_CACHE_TTL_SECS: i64 = 43_200;
pub const FORECAST_CACHE_TTL_SECS: i64 = 10_800;
pub const TRACK_WEATHER_CACHE_TTL_SECS: i64 = 3_600;
pub const QUALI_GRID_CACHE_TTL_SECS: i64 = 3_600;
pub const ASSET_CACHE_TTL_SECS: i64 = 2_592_000;
pub const ASSET_FAILED_TTL_SECS: i64 = 3_600;

pub fn calendar_cache_key(season: i32) -> String {
    format!("calendar:{season}")
}

pub fn drivers_cache_key(season: i32) -> String {
    format!("drivers:{season}")
}

pub fn championship_cache_key(season: i32) -> String {
    format!("championship:{season}")
}

pub fn forecast_cache_key(meeting_key: i64) -> String {
    format!("forecast:{meeting_key}")
}

pub fn track_weather_cache_key(meeting_key: i64) -> String {
    format!("track_weather:{meeting_key}")
}

pub fn quali_grid_cache_key(meeting_key: i64) -> String {
    format!("quali_grid:{meeting_key}")
}

pub fn schema_sql() -> &'static str {
    SCHEMA
}
