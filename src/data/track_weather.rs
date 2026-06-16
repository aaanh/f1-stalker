use chrono::{DateTime, Utc};
use openf1::{OpenF1Client, OpenF1Error, OpenF1Key, Session, Weather, WeatherListParams};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::weather::{latest_track_conditions, most_recent_completed_session, TrackConditions};

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("openf1: {0}")]
    Api(#[from] OpenF1Error),
}

#[derive(Debug, Clone)]
pub struct TrackWeatherData {
    pub meeting_key: i64,
    pub conditions: Option<TrackConditions>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackWeatherCacheBlob {
    pub meeting_key: i64,
    pub conditions: Option<TrackConditionsBlob>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackConditionsBlob {
    pub session_name: String,
    pub sampled_at: DateTime<Utc>,
    pub air_temperature_c: f64,
    pub track_temperature_c: f64,
    pub humidity: i64,
    pub rainfall: i64,
    pub wind_speed: f64,
    pub wind_direction: i64,
}

impl TrackWeatherCacheBlob {
    pub fn from_data(data: &TrackWeatherData) -> Self {
        Self {
            meeting_key: data.meeting_key,
            conditions: data.conditions.as_ref().map(|track| TrackConditionsBlob {
                session_name: track.session_name.clone(),
                sampled_at: track.sampled_at,
                air_temperature_c: track.air_temperature_c,
                track_temperature_c: track.track_temperature_c,
                humidity: track.humidity,
                rainfall: track.rainfall,
                wind_speed: track.wind_speed,
                wind_direction: track.wind_direction,
            }),
            fetched_at: data.fetched_at,
        }
    }
}

pub fn track_weather_from_cache(blob: TrackWeatherCacheBlob) -> TrackWeatherData {
    TrackWeatherData {
        meeting_key: blob.meeting_key,
        conditions: blob.conditions.map(|track| TrackConditions {
            session_name: track.session_name,
            sampled_at: track.sampled_at,
            air_temperature_c: track.air_temperature_c,
            track_temperature_c: track.track_temperature_c,
            humidity: track.humidity,
            rainfall: track.rainfall,
            wind_speed: track.wind_speed,
            wind_direction: track.wind_direction,
        }),
        fetched_at: blob.fetched_at,
    }
}

pub async fn fetch_track_weather(
    meeting_key: i64,
    sessions: &[Session],
) -> Result<TrackWeatherData, FetchError> {
    let now = Utc::now();
    let Some(session) = most_recent_completed_session(sessions, meeting_key, now) else {
        return Ok(TrackWeatherData {
            meeting_key,
            conditions: None,
            fetched_at: now,
        });
    };

    let samples = fetch_weather_samples(session.session_key).await?;
    let conditions = latest_track_conditions(session, &samples);

    Ok(TrackWeatherData {
        meeting_key,
        conditions,
        fetched_at: now,
    })
}

async fn fetch_weather_samples(session_key: i64) -> Result<Vec<Weather>, FetchError> {
    let client = OpenF1Client::new(None);
    client
        .weather()
        .list(WeatherListParams {
            session_key: Some(OpenF1Key::Id(session_key)),
        })
        .await
        .map_err(Into::into)
}
