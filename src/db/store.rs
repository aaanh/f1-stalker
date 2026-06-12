use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::data::cache::{calendar_from_cache, drivers_from_cache, CalendarCacheBlob, DriversCacheBlob};
use crate::data::championship::{championship_from_cache, ChampionshipCacheBlob, ChampionshipData};
use crate::data::drivers::DriversData;
use crate::data::grid::{quali_grid_from_cache, QualiGridCacheBlob, QualiGridData};
use crate::data::open_meteo::ForecastData;
use crate::data::track_weather::{track_weather_from_cache, TrackWeatherCacheBlob, TrackWeatherData};
use crate::db::schema::{
    calendar_cache_key, championship_cache_key, drivers_cache_key, forecast_cache_key,
    quali_grid_cache_key, schema_sql, track_weather_cache_key, ASSET_CACHE_TTL_SECS,
    ASSET_FAILED_TTL_SECS, CALENDAR_CACHE_TTL_SECS, CHAMPIONSHIP_CACHE_TTL_SECS,
    DRIVERS_CACHE_TTL_SECS, FORECAST_CACHE_TTL_SECS, QUALI_GRID_CACHE_TTL_SECS,
    SETTING_BACKGROUND_ON_CLOSE, SETTING_FIRST_RUN_COMPLETE, SETTING_INCLUDE_TESTING,
    SETTING_NOTIFICATIONS_ENABLED,     SETTING_NOTIFY_SESSIONS, SETTING_NOTIFY_STANDINGS,
    SETTING_RIVAL_COMPARE_ACTIVE, SETTING_RIVAL_DRIVER_FIRST, SETTING_RIVAL_DRIVER_SECOND,
    SETTING_SEASON_YEAR, SETTING_SESSION_REMINDER_MINUTES, SETTING_THEME_ID, SETTING_TIMEZONE,
    TRACK_WEATHER_CACHE_TTL_SECS,
};
use crate::db::Settings;

fn parse_bool(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes")
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("calendar cache: {0}")]
    Calendar(String),
    #[error("data dir unavailable")]
    NoDataDir,
}

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct PinnedDriver {
    pub driver_number: i64,
    pub sort_order: i32,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub fetched_at: DateTime<Utc>,
    pub ttl_secs: i64,
}

impl CacheEntry {
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now.signed_duration_since(self.fetched_at).num_seconds() > self.ttl_secs
    }
}

impl Database {
    pub fn open_default() -> Result<Self, DbError> {
        let path = default_db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::open(&path)
    }

    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<(), DbError> {
        self.conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        self.conn.execute_batch(schema_sql())?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<Settings, DbError> {
        let mut settings = Settings::default();
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                SETTING_SEASON_YEAR => {
                    if let Ok(year) = value.parse() {
                        settings.season_year = year;
                    }
                }
                SETTING_TIMEZONE if !value.is_empty() => settings.timezone = value,
                SETTING_FIRST_RUN_COMPLETE => settings.first_run_complete = parse_bool(&value),
                SETTING_THEME_ID => {
                    settings.theme_id = crate::ui::theme::ThemePresetId::from_key(&value)
                }
                SETTING_BACKGROUND_ON_CLOSE => settings.background_on_close = parse_bool(&value),
                SETTING_INCLUDE_TESTING => settings.include_testing = parse_bool(&value),
                SETTING_NOTIFICATIONS_ENABLED => {
                    settings.notifications_enabled = parse_bool(&value)
                }
                SETTING_NOTIFY_STANDINGS => settings.notify_standings = parse_bool(&value),
                SETTING_NOTIFY_SESSIONS => settings.notify_sessions = parse_bool(&value),
                SETTING_SESSION_REMINDER_MINUTES => {
                    if let Ok(minutes) = value.parse() {
                        settings.session_reminder_minutes = minutes;
                    }
                }
                SETTING_RIVAL_DRIVER_FIRST => {
                    if let Ok(number) = value.parse() {
                        settings.rival_driver_first = number;
                    }
                }
                SETTING_RIVAL_DRIVER_SECOND => {
                    if let Ok(number) = value.parse() {
                        settings.rival_driver_second = number;
                    }
                }
                SETTING_RIVAL_COMPARE_ACTIVE => {
                    settings.rival_compare_active = parse_bool(&value)
                }
                _ => {}
            }
        }

        Ok(settings)
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), DbError> {
        self.set_setting(SETTING_SEASON_YEAR, settings.season_year.to_string())?;
        self.set_setting(SETTING_TIMEZONE, settings.timezone.clone())?;
        self.set_setting(
            SETTING_FIRST_RUN_COMPLETE,
            settings.first_run_complete.to_string(),
        )?;
        self.set_setting(SETTING_THEME_ID, settings.theme_id.key().into())?;
        self.set_setting(
            SETTING_BACKGROUND_ON_CLOSE,
            settings.background_on_close.to_string(),
        )?;
        self.set_setting(
            SETTING_INCLUDE_TESTING,
            settings.include_testing.to_string(),
        )?;
        self.set_setting(
            SETTING_NOTIFICATIONS_ENABLED,
            settings.notifications_enabled.to_string(),
        )?;
        self.set_setting(SETTING_NOTIFY_STANDINGS, settings.notify_standings.to_string())?;
        self.set_setting(SETTING_NOTIFY_SESSIONS, settings.notify_sessions.to_string())?;
        self.set_setting(
            SETTING_SESSION_REMINDER_MINUTES,
            settings.session_reminder_minutes.to_string(),
        )?;
        self.set_setting(
            SETTING_RIVAL_DRIVER_FIRST,
            settings.rival_driver_first.to_string(),
        )?;
        self.set_setting(
            SETTING_RIVAL_DRIVER_SECOND,
            settings.rival_driver_second.to_string(),
        )?;
        self.set_setting(
            SETTING_RIVAL_COMPARE_ACTIVE,
            settings.rival_compare_active.to_string(),
        )?;
        Ok(())
    }

    pub fn load_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(row.get(0)?))
    }

    pub fn save_setting(&self, key: &str, value: impl AsRef<str>) -> Result<(), DbError> {
        self.set_setting(key, value.as_ref().into())
    }

    fn set_setting(&self, key: &str, value: String) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn list_pinned_drivers(&self) -> Result<Vec<PinnedDriver>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT driver_number, sort_order FROM pinned_drivers ORDER BY sort_order")?;
        let rows = stmt.query_map([], |row| {
            Ok(PinnedDriver {
                driver_number: row.get(0)?,
                sort_order: row.get(1)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn replace_pinned_drivers(&self, pins: &[PinnedDriver]) -> Result<(), DbError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM pinned_drivers", [])?;
        for pin in pins {
            tx.execute(
                "INSERT INTO pinned_drivers (driver_number, sort_order) VALUES (?1, ?2)",
                params![pin.driver_number, pin.sort_order],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn save_calendar_cache(&self, blob: &CalendarCacheBlob) -> Result<(), DbError> {
        let payload = serde_json::to_string(blob)?;
        self.conn.execute(
            "INSERT INTO cache (cache_key, payload, fetched_at, ttl_secs) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(cache_key) DO UPDATE SET
                payload = excluded.payload,
                fetched_at = excluded.fetched_at,
                ttl_secs = excluded.ttl_secs",
            params![
                calendar_cache_key(blob.season),
                payload,
                blob.fetched_at.to_rfc3339(),
                CALENDAR_CACHE_TTL_SECS,
            ],
        )?;
        Ok(())
    }

    pub fn load_calendar_cache(&self, season: i32) -> Result<Option<CalendarCacheBlob>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT payload, fetched_at FROM cache WHERE cache_key = ?1",
        )?;
        let mut rows = stmt.query(params![calendar_cache_key(season)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let payload: String = row.get(0)?;
        let fetched_at: String = row.get(1)?;
        let mut blob: CalendarCacheBlob = serde_json::from_str(&payload)?;
        blob.fetched_at = DateTime::parse_from_rfc3339(&fetched_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(blob.fetched_at);
        Ok(Some(blob))
    }

    pub fn cache_entry_for_calendar(&self, season: i32) -> Result<Option<CacheEntry>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT fetched_at, ttl_secs FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![calendar_cache_key(season)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(CacheEntry {
            fetched_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            ttl_secs: row.get(1)?,
        }))
    }

    pub fn calendar_from_cache(&self, season: i32) -> Result<Option<crate::data::CalendarData>, DbError> {
        match self.load_calendar_cache(season)? {
            Some(blob) => calendar_from_cache(blob)
                .map(Some)
                .map_err(|error| DbError::Calendar(error.to_string())),
            None => Ok(None),
        }
    }

    pub fn save_drivers_cache(&self, blob: &DriversCacheBlob) -> Result<(), DbError> {
        let payload = serde_json::to_string(blob)?;
        self.conn.execute(
            "INSERT INTO cache (cache_key, payload, fetched_at, ttl_secs) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(cache_key) DO UPDATE SET
                payload = excluded.payload,
                fetched_at = excluded.fetched_at,
                ttl_secs = excluded.ttl_secs",
            params![
                drivers_cache_key(blob.season),
                payload,
                blob.fetched_at.to_rfc3339(),
                DRIVERS_CACHE_TTL_SECS,
            ],
        )?;
        Ok(())
    }

    pub fn load_drivers_cache(&self, season: i32) -> Result<Option<DriversCacheBlob>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload, fetched_at FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![drivers_cache_key(season)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let payload: String = row.get(0)?;
        let fetched_at: String = row.get(1)?;
        let mut blob: DriversCacheBlob = serde_json::from_str(&payload)?;
        blob.fetched_at = DateTime::parse_from_rfc3339(&fetched_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(blob.fetched_at);
        Ok(Some(blob))
    }

    pub fn cache_entry_for_drivers(&self, season: i32) -> Result<Option<CacheEntry>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT fetched_at, ttl_secs FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![drivers_cache_key(season)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(CacheEntry {
            fetched_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            ttl_secs: row.get(1)?,
        }))
    }

    pub fn drivers_from_cache(&self, season: i32) -> Result<Option<DriversData>, DbError> {
        Ok(self.load_drivers_cache(season)?.map(drivers_from_cache))
    }

    pub fn save_championship_cache(&self, blob: &ChampionshipCacheBlob) -> Result<(), DbError> {
        let payload = serde_json::to_string(blob)?;
        self.conn.execute(
            "INSERT INTO cache (cache_key, payload, fetched_at, ttl_secs) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(cache_key) DO UPDATE SET
                payload = excluded.payload,
                fetched_at = excluded.fetched_at,
                ttl_secs = excluded.ttl_secs",
            params![
                championship_cache_key(blob.season),
                payload,
                blob.fetched_at.to_rfc3339(),
                CHAMPIONSHIP_CACHE_TTL_SECS,
            ],
        )?;
        Ok(())
    }

    pub fn load_championship_cache(&self, season: i32) -> Result<Option<ChampionshipCacheBlob>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload, fetched_at FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![championship_cache_key(season)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let payload: String = row.get(0)?;
        let fetched_at: String = row.get(1)?;
        let mut blob: ChampionshipCacheBlob = serde_json::from_str(&payload)?;
        blob.fetched_at = DateTime::parse_from_rfc3339(&fetched_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(blob.fetched_at);
        Ok(Some(blob))
    }

    pub fn cache_entry_for_championship(&self, season: i32) -> Result<Option<CacheEntry>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT fetched_at, ttl_secs FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![championship_cache_key(season)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(CacheEntry {
            fetched_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            ttl_secs: row.get(1)?,
        }))
    }

    pub fn championship_from_cache(
        &self,
        season: i32,
    ) -> Result<Option<ChampionshipData>, DbError> {
        Ok(self
            .load_championship_cache(season)?
            .map(championship_from_cache))
    }

    pub fn save_forecast_cache(&self, data: &ForecastData) -> Result<(), DbError> {
        #[derive(serde::Serialize)]
        struct Blob<'a> {
            meeting_key: i64,
            forecast: &'a crate::domain::LocationForecast,
            fetched_at: DateTime<Utc>,
        }

        let blob = Blob {
            meeting_key: data.meeting_key,
            forecast: &data.forecast,
            fetched_at: data.forecast.fetched_at,
        };
        let payload = serde_json::to_string(&blob)?;
        self.conn.execute(
            "INSERT INTO cache (cache_key, payload, fetched_at, ttl_secs) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(cache_key) DO UPDATE SET
                payload = excluded.payload,
                fetched_at = excluded.fetched_at,
                ttl_secs = excluded.ttl_secs",
            params![
                forecast_cache_key(data.meeting_key),
                payload,
                data.forecast.fetched_at.to_rfc3339(),
                FORECAST_CACHE_TTL_SECS,
            ],
        )?;
        Ok(())
    }

    pub fn load_forecast_cache(&self, meeting_key: i64) -> Result<Option<ForecastData>, DbError> {
        #[derive(serde::Deserialize)]
        struct Blob {
            meeting_key: i64,
            forecast: crate::domain::LocationForecast,
            fetched_at: DateTime<Utc>,
        }

        let mut stmt = self
            .conn
            .prepare("SELECT payload, fetched_at FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![forecast_cache_key(meeting_key)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let payload: String = row.get(0)?;
        let fetched_at: String = row.get(1)?;
        let mut blob: Blob = serde_json::from_str(&payload)?;
        blob.forecast.fetched_at = DateTime::parse_from_rfc3339(&fetched_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(blob.fetched_at);
        Ok(Some(ForecastData {
            meeting_key: blob.meeting_key,
            forecast: blob.forecast,
        }))
    }

    pub fn cache_entry_for_forecast(&self, meeting_key: i64) -> Result<Option<CacheEntry>, DbError> {
        self.cache_entry_for_key(&forecast_cache_key(meeting_key))
    }

    pub fn save_track_weather_cache(&self, blob: &TrackWeatherCacheBlob) -> Result<(), DbError> {
        let payload = serde_json::to_string(blob)?;
        self.conn.execute(
            "INSERT INTO cache (cache_key, payload, fetched_at, ttl_secs) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(cache_key) DO UPDATE SET
                payload = excluded.payload,
                fetched_at = excluded.fetched_at,
                ttl_secs = excluded.ttl_secs",
            params![
                track_weather_cache_key(blob.meeting_key),
                payload,
                blob.fetched_at.to_rfc3339(),
                TRACK_WEATHER_CACHE_TTL_SECS,
            ],
        )?;
        Ok(())
    }

    pub fn load_track_weather_cache(
        &self,
        meeting_key: i64,
    ) -> Result<Option<TrackWeatherData>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload, fetched_at FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![track_weather_cache_key(meeting_key)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let payload: String = row.get(0)?;
        let fetched_at: String = row.get(1)?;
        let mut blob: TrackWeatherCacheBlob = serde_json::from_str(&payload)?;
        blob.fetched_at = DateTime::parse_from_rfc3339(&fetched_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(blob.fetched_at);
        Ok(Some(track_weather_from_cache(blob)))
    }

    pub fn cache_entry_for_track_weather(
        &self,
        meeting_key: i64,
    ) -> Result<Option<CacheEntry>, DbError> {
        self.cache_entry_for_key(&track_weather_cache_key(meeting_key))
    }

    pub fn save_quali_grid_cache(&self, blob: &QualiGridCacheBlob) -> Result<(), DbError> {
        let payload = serde_json::to_string(blob)?;
        self.conn.execute(
            "INSERT INTO cache (cache_key, payload, fetched_at, ttl_secs) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(cache_key) DO UPDATE SET
                payload = excluded.payload,
                fetched_at = excluded.fetched_at,
                ttl_secs = excluded.ttl_secs",
            params![
                quali_grid_cache_key(blob.meeting_key),
                payload,
                blob.fetched_at.to_rfc3339(),
                QUALI_GRID_CACHE_TTL_SECS,
            ],
        )?;
        Ok(())
    }

    pub fn load_quali_grid_cache(&self, meeting_key: i64) -> Result<Option<QualiGridData>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload, fetched_at FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![quali_grid_cache_key(meeting_key)])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let payload: String = row.get(0)?;
        let fetched_at: String = row.get(1)?;
        let mut blob: QualiGridCacheBlob = serde_json::from_str(&payload)?;
        blob.fetched_at = DateTime::parse_from_rfc3339(&fetched_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(blob.fetched_at);
        Ok(Some(quali_grid_from_cache(blob)))
    }

    pub fn cache_entry_for_quali_grid(
        &self,
        meeting_key: i64,
    ) -> Result<Option<CacheEntry>, DbError> {
        self.cache_entry_for_key(&quali_grid_cache_key(meeting_key))
    }

    fn cache_entry_for_key(&self, cache_key: &str) -> Result<Option<CacheEntry>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT fetched_at, ttl_secs FROM cache WHERE cache_key = ?1")?;
        let mut rows = stmt.query(params![cache_key])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(CacheEntry {
            fetched_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            ttl_secs: row.get(1)?,
        }))
    }

    pub fn clear_cache(&self) -> Result<(usize, usize), DbError> {
        let api_count = self.conn.execute("DELETE FROM cache", [])?;
        let assets_dir = default_assets_dir()?;
        let asset_count = self.clear_asset_cache(&assets_dir)?;
        Ok((api_count, asset_count))
    }

    pub fn load_cached_asset(
        &self,
        url: &str,
        assets_dir: &Path,
    ) -> Result<Option<Vec<u8>>, DbError> {
        let now = Utc::now();
        let mut stmt = self.conn.prepare(
            "SELECT file_name, fetched_at, ttl_secs, failed FROM asset_cache WHERE url = ?1",
        )?;
        let mut rows = stmt.query(params![url])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let file_name: String = row.get(0)?;
        let fetched_at = parse_asset_timestamp(row.get::<_, String>(1)?);
        let ttl_secs: i64 = row.get(2)?;
        let failed: i32 = row.get(3)?;
        let entry = CacheEntry {
            fetched_at,
            ttl_secs,
        };

        if entry.is_expired(now) {
            self.remove_asset_entry(url, &file_name, assets_dir)?;
            return Ok(None);
        }

        if failed != 0 {
            return Ok(None);
        }

        let path = assets_dir.join(&file_name);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(_) => {
                self.remove_asset_entry(url, &file_name, assets_dir)?;
                Ok(None)
            }
        }
    }

    pub fn is_asset_failed(&self, url: &str, assets_dir: &Path) -> Result<bool, DbError> {
        let now = Utc::now();
        let mut stmt = self.conn.prepare(
            "SELECT fetched_at, ttl_secs, failed, file_name FROM asset_cache WHERE url = ?1",
        )?;
        let mut rows = stmt.query(params![url])?;
        let Some(row) = rows.next()? else {
            return Ok(false);
        };

        let fetched_at = parse_asset_timestamp(row.get::<_, String>(0)?);
        let ttl_secs: i64 = row.get(1)?;
        let failed: i32 = row.get(2)?;
        let file_name: String = row.get(3)?;
        let entry = CacheEntry {
            fetched_at,
            ttl_secs,
        };

        if entry.is_expired(now) {
            self.remove_asset_entry(url, &file_name, assets_dir)?;
            return Ok(false);
        }

        Ok(failed != 0)
    }

    pub fn save_cached_asset(
        &self,
        url: &str,
        bytes: &[u8],
        assets_dir: &Path,
    ) -> Result<(), DbError> {
        std::fs::create_dir_all(assets_dir)?;
        let file_name = asset_file_name(url);
        std::fs::write(assets_dir.join(&file_name), bytes)?;
        self.conn.execute(
            "INSERT INTO asset_cache (url, file_name, fetched_at, ttl_secs, failed)
             VALUES (?1, ?2, ?3, ?4, 0)
             ON CONFLICT(url) DO UPDATE SET
                file_name = excluded.file_name,
                fetched_at = excluded.fetched_at,
                ttl_secs = excluded.ttl_secs,
                failed = 0",
            params![
                url,
                file_name,
                Utc::now().to_rfc3339(),
                ASSET_CACHE_TTL_SECS,
            ],
        )?;
        Ok(())
    }

    pub fn mark_asset_failed(&self, url: &str) -> Result<(), DbError> {
        let file_name = asset_file_name(url);
        self.conn.execute(
            "INSERT INTO asset_cache (url, file_name, fetched_at, ttl_secs, failed)
             VALUES (?1, ?2, ?3, ?4, 1)
             ON CONFLICT(url) DO UPDATE SET
                fetched_at = excluded.fetched_at,
                ttl_secs = excluded.ttl_secs,
                failed = 1",
            params![
                url,
                file_name,
                Utc::now().to_rfc3339(),
                ASSET_FAILED_TTL_SECS,
            ],
        )?;
        Ok(())
    }

    pub fn clear_asset_cache(&self, assets_dir: &Path) -> Result<usize, DbError> {
        let count = self.conn.execute("DELETE FROM asset_cache", [])?;
        if assets_dir.exists() {
            std::fs::remove_dir_all(assets_dir)?;
        }
        std::fs::create_dir_all(assets_dir)?;
        Ok(count)
    }

    fn remove_asset_entry(
        &self,
        url: &str,
        file_name: &str,
        assets_dir: &Path,
    ) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM asset_cache WHERE url = ?1", params![url])?;
        let path = assets_dir.join(file_name);
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }

    pub fn path(&self) -> Result<PathBuf, DbError> {
        default_db_path()
    }
}

pub fn rebuild_database(path: &Path) -> Result<Database, DbError> {
    let (settings, pins) = if path.exists() {
        let existing = Database::open(path)?;
        (
            existing.load_settings().unwrap_or_default(),
            existing.list_pinned_drivers().unwrap_or_default(),
        )
    } else {
        (Settings::default(), Vec::new())
    };

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    if let Ok(assets_dir) = default_assets_dir() {
        let _ = std::fs::remove_dir_all(&assets_dir);
    }

    let db = Database::open(path)?;
    db.save_settings(&settings)?;
    db.replace_pinned_drivers(&pins)?;
    Ok(db)
}

pub fn default_db_path() -> Result<PathBuf, DbError> {
    let dirs = ProjectDirs::from("com", "f1-stalker", "F1 Stalker").ok_or(DbError::NoDataDir)?;
    Ok(dirs.data_dir().join("f1-stalker.db"))
}

pub fn default_assets_dir() -> Result<PathBuf, DbError> {
    let dirs = ProjectDirs::from("com", "f1-stalker", "F1 Stalker").ok_or(DbError::NoDataDir)?;
    Ok(dirs.data_dir().join("assets"))
}

fn asset_file_name(url: &str) -> String {
    format!("{:x}.bin", Sha256::digest(url.as_bytes()))
}

fn parse_asset_timestamp(value: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use openf1::Meeting;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_db() -> Database {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("f1-stalker-test-{id}.db"));
        let _ = std::fs::remove_file(&path);
        Database::open(&path).unwrap()
    }

    fn temp_assets_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("f1-stalker-assets-{id}"))
    }

    fn sample_meeting(key: i64) -> Meeting {
        Meeting {
            circuit_image: String::new(),
            circuit_info_url: String::new(),
            circuit_key: 1,
            circuit_short_name: "Test".into(),
            circuit_type: "Permanent".into(),
            country_code: "TST".into(),
            country_flag: String::new(),
            country_key: 1,
            country_name: "Testland".into(),
            date_end: "2026-03-10T00:00:00Z".into(),
            date_start: "2026-03-08T00:00:00Z".into(),
            gmt_offset: "00:00:00".into(),
            is_cancelled: false,
            location: "Test City".into(),
            meeting_key: key,
            meeting_name: format!("GP {key}"),
            meeting_official_name: format!("TEST GP {key}"),
            year: 2026,
        }
    }

    #[test]
    fn settings_round_trip() {
        let db = temp_db();
        let settings = Settings {
            season_year: 2026,
            timezone: "Europe/London".into(),
            ..Settings::default()
        };
        db.save_settings(&settings).unwrap();
        let loaded = db.load_settings().unwrap();
        assert_eq!(loaded.season_year, 2026);
        assert_eq!(loaded.timezone, "Europe/London");
    }

    #[test]
    fn calendar_cache_round_trip() {
        let db = temp_db();
        let blob = CalendarCacheBlob {
            season: 2026,
            meetings: vec![sample_meeting(1), sample_meeting(2)],
            sessions: vec![],
            fetched_at: Utc::now(),
        };
        db.save_calendar_cache(&blob).unwrap();
        let loaded = db.calendar_from_cache(2026).unwrap().unwrap();
        assert_eq!(loaded.season, 2026);
        assert_eq!(loaded.meetings.len(), 2);
    }

    #[test]
    fn pinned_drivers_replace() {
        let db = temp_db();
        db.replace_pinned_drivers(&[
            PinnedDriver {
                driver_number: 44,
                sort_order: 0,
            },
            PinnedDriver {
                driver_number: 16,
                sort_order: 1,
            },
        ])
        .unwrap();
        let pins = db.list_pinned_drivers().unwrap();
        assert_eq!(pins.len(), 2);
        assert_eq!(pins[0].driver_number, 44);
    }

    #[test]
    fn asset_cache_round_trip() {
        let db = temp_db();
        let assets_dir = temp_assets_dir();
        let _ = std::fs::remove_dir_all(&assets_dir);
        let url = "https://example.com/flags/nl.svg";
        let bytes = b"flag-bytes".to_vec();

        db.save_cached_asset(url, &bytes, &assets_dir).unwrap();
        let loaded = db.load_cached_asset(url, &assets_dir).unwrap().unwrap();
        assert_eq!(loaded, bytes);
    }

    #[test]
    fn asset_failed_blocks_until_expired() {
        let db = temp_db();
        let assets_dir = temp_assets_dir();
        let _ = std::fs::remove_dir_all(&assets_dir);
        let url = "https://example.com/headshots/missing.png";

        db.mark_asset_failed(url).unwrap();
        assert!(db.is_asset_failed(url, &assets_dir).unwrap());
        assert!(db.load_cached_asset(url, &assets_dir).unwrap().is_none());
    }

    #[test]
    fn clear_asset_cache_removes_files() {
        let db = temp_db();
        let assets_dir = temp_assets_dir();
        let _ = std::fs::remove_dir_all(&assets_dir);
        let url = "https://example.com/logos/ferrari.png";
        db.save_cached_asset(url, b"logo", &assets_dir).unwrap();

        let count = db.clear_asset_cache(&assets_dir).unwrap();
        assert_eq!(count, 1);
        assert!(db.load_cached_asset(url, &assets_dir).unwrap().is_none());
        assert!(std::fs::read_dir(&assets_dir).unwrap().next().is_none());
    }
}
