use std::collections::HashMap;

use chrono::{DateTime, Utc};
use openf1::Meeting;
use thiserror::Error;

use crate::data::grid::{fetch_quali_grid, QualiGridData};
use crate::data::open_meteo::{self, ForecastData};
use crate::data::track_weather::{fetch_track_weather, TrackWeatherData};
use crate::domain::weather::{
    ForecastState, TrackState, WeatherPanel,
};
use crate::domain::{find_gp_qualifying, quali_grid_visibility, QualiGridVisibility};
use openf1::Session;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("grid: {0}")]
    Grid(#[from] crate::data::grid::FetchError),
    #[error("track: {0}")]
    Track(#[from] crate::data::track_weather::FetchError),
}

#[derive(Debug, Clone)]
pub struct WeekendDetailData {
    pub quali_grid: Option<QualiGridData>,
    pub quali_visibility: QualiGridVisibility,
    pub weather_by_meeting: HashMap<i64, WeatherPanel>,
    pub forecasts_to_cache: Vec<ForecastData>,
    pub tracks_to_cache: Vec<TrackWeatherData>,
    pub fetched_at: DateTime<Utc>,
}

pub struct WeekendFetchInput {
    pub focus_meeting_key: i64,
    pub meetings: Vec<Meeting>,
    pub sessions: Vec<Session>,
    pub pinned_numbers: Vec<i64>,
    pub cached_grid: Option<QualiGridData>,
    pub cached_track: HashMap<i64, TrackWeatherData>,
    pub cached_forecasts: HashMap<i64, ForecastData>,
}

pub async fn fetch_weekend_details(input: WeekendFetchInput) -> Result<WeekendDetailData, FetchError> {
    let now = Utc::now();
    let quali = find_gp_qualifying(&input.sessions, input.focus_meeting_key);

    let quali_grid = if input.pinned_numbers.is_empty() {
        None
    } else if let Some(cached) = input
        .cached_grid
        .clone()
        .filter(|data| !data.slots.is_empty())
    {
        Some(cached)
    } else if quali.is_some() {
        match fetch_quali_grid(
            input.focus_meeting_key,
            &input.sessions,
            &input.pinned_numbers,
        )
        .await
        {
            Ok(data) if !data.slots.is_empty() => Some(data),
            Ok(_) => input.cached_grid,
            Err(_) => input.cached_grid,
        }
    } else {
        None
    };

    let grid_available = quali_grid
        .as_ref()
        .map(|data| !data.slots.is_empty())
        .unwrap_or(false);
    let quali_visibility = quali_grid_visibility(
        !input.pinned_numbers.is_empty(),
        quali,
        now,
        grid_available,
    );

    let mut weather_by_meeting = HashMap::new();
    let mut forecasts_to_cache = Vec::new();
    let mut tracks_to_cache = Vec::new();

    for meeting in &input.meetings {
        let track = if let Some(cached) = input.cached_track.get(&meeting.meeting_key) {
            cached.clone()
        } else {
            let fetched = fetch_track_weather(meeting.meeting_key, &input.sessions).await?;
            tracks_to_cache.push(fetched.clone());
            fetched
        };

        let forecast_state = if let Some(cached) = input.cached_forecasts.get(&meeting.meeting_key) {
            ForecastState::Ready(cached.forecast.clone())
        } else {
            match open_meteo::fetch_meeting_forecast(meeting).await {
                Ok(data) => {
                    forecasts_to_cache.push(data.clone());
                    ForecastState::Ready(data.forecast)
                }
                Err(error) => ForecastState::Error(error.to_string()),
            }
        };

        let track_state = match track.conditions {
            Some(conditions) => TrackState::Ready(conditions),
            None => TrackState::NoSessionData,
        };

        weather_by_meeting.insert(
            meeting.meeting_key,
            WeatherPanel {
                meeting_key: meeting.meeting_key,
                forecast: forecast_state,
                track: track_state,
            },
        );
    }

    Ok(WeekendDetailData {
        quali_grid,
        quali_visibility,
        weather_by_meeting,
        forecasts_to_cache,
        tracks_to_cache,
        fetched_at: now,
    })
}

pub fn assemble_weekend_from_cache(
    focus_meeting_key: i64,
    meetings: &[Meeting],
    sessions: &[Session],
    pinned_numbers: &[i64],
    cached_grid: Option<QualiGridData>,
    cached_track: &HashMap<i64, TrackWeatherData>,
    cached_forecasts: &HashMap<i64, ForecastData>,
) -> Option<WeekendDetailData> {
    if meetings.is_empty() {
        return None;
    }

    let now = Utc::now();
    let quali = find_gp_qualifying(sessions, focus_meeting_key);
    let quali_grid = cached_grid;
    let grid_available = quali_grid
        .as_ref()
        .map(|data| !data.slots.is_empty())
        .unwrap_or(false);
    let quali_visibility = quali_grid_visibility(
        !pinned_numbers.is_empty(),
        quali,
        now,
        grid_available,
    );

    let mut weather_by_meeting = HashMap::new();
    for meeting in meetings {
        let forecast_state = if let Some(cached) = cached_forecasts.get(&meeting.meeting_key) {
            ForecastState::Ready(cached.forecast.clone())
        } else {
            ForecastState::Loading
        };

        let track_state = if let Some(cached) = cached_track.get(&meeting.meeting_key) {
            match &cached.conditions {
                Some(conditions) => TrackState::Ready(conditions.clone()),
                None => TrackState::NoSessionData,
            }
        } else {
            TrackState::Loading
        };

        weather_by_meeting.insert(
            meeting.meeting_key,
            WeatherPanel {
                meeting_key: meeting.meeting_key,
                forecast: forecast_state,
                track: track_state,
            },
        );
    }

    Some(WeekendDetailData {
        quali_grid,
        quali_visibility,
        weather_by_meeting,
        forecasts_to_cache: Vec::new(),
        tracks_to_cache: Vec::new(),
        fetched_at: now,
    })
}

pub fn meetings_for_weather(triplet: &crate::domain::RaceTriplet) -> Vec<Meeting> {
    let mut meetings = Vec::new();
    if let Some(previous) = &triplet.previous {
        meetings.push(previous.clone());
    }
    meetings.push(triplet.current.clone());
    if triplet.upcoming.meeting_key != triplet.current.meeting_key {
        meetings.push(triplet.upcoming.clone());
    }
    meetings
}
