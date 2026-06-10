use chrono::{DateTime, Utc};
use openf1::{Driver, Meeting, Session};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::CalendarData;
use crate::data::drivers::DriversData;
use crate::domain::{compute_race_triplet, next_season_countdown};

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("empty season calendar")]
    EmptySeason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarCacheBlob {
    pub season: i32,
    pub meetings: Vec<Meeting>,
    pub sessions: Vec<Session>,
    pub fetched_at: DateTime<Utc>,
}

impl CalendarCacheBlob {
    pub fn from_calendar(data: &CalendarData) -> Self {
        Self {
            season: data.season,
            meetings: data.meetings.clone(),
            sessions: data.sessions.clone(),
            fetched_at: data.fetched_at,
        }
    }
}

pub fn calendar_from_cache(blob: CalendarCacheBlob) -> Result<CalendarData, CacheError> {
    let now = Utc::now();
    let triplet = compute_race_triplet(&blob.meetings, now).ok_or(CacheError::EmptySeason)?;
    let countdown = next_season_countdown(&blob.meetings, &blob.sessions, now);

    Ok(CalendarData {
        season: blob.season,
        meetings: blob.meetings,
        sessions: blob.sessions,
        triplet,
        countdown,
        fetched_at: blob.fetched_at,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriversCacheBlob {
    pub season: i32,
    pub drivers: Vec<Driver>,
    pub fetched_at: DateTime<Utc>,
}

impl DriversCacheBlob {
    pub fn from_drivers(data: &DriversData) -> Self {
        Self {
            season: data.season,
            drivers: data.drivers.clone(),
            fetched_at: data.fetched_at,
        }
    }
}

pub fn drivers_from_cache(blob: DriversCacheBlob) -> DriversData {
    DriversData {
        season: blob.season,
        drivers: blob.drivers,
        fetched_at: blob.fetched_at,
    }
}
