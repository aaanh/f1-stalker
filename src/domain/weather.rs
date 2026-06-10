use chrono::{DateTime, Utc};
use openf1::{Meeting, Session, Weather};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ForecastDay {
    pub label: String,
    pub temp_min_c: f64,
    pub temp_max_c: f64,
    pub precip_probability: u8,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LocationForecast {
    pub fetched_at: DateTime<Utc>,
    pub location_query: String,
    pub summary: String,
    pub days: Vec<ForecastDay>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrackConditions {
    pub session_name: String,
    pub sampled_at: DateTime<Utc>,
    pub air_temperature_c: f64,
    pub track_temperature_c: f64,
    pub humidity: i64,
    pub rainfall: i64,
    pub wind_speed: f64,
    pub wind_direction: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForecastState {
    Loading,
    Ready(LocationForecast),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackState {
    NoSessionData,
    Loading,
    Ready(TrackConditions),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherPanel {
    pub meeting_key: i64,
    pub forecast: ForecastState,
    pub track: TrackState,
}

pub fn location_query(meeting: &Meeting) -> String {
    format!("{}, {}", meeting.location, meeting.country_name)
}

pub fn most_recent_completed_session<'a>(
    sessions: &'a [Session],
    meeting_key: i64,
    now: DateTime<Utc>,
) -> Option<&'a Session> {
    let mut completed: Vec<&Session> = sessions
        .iter()
        .filter(|session| {
            session.meeting_key == meeting_key
                && !session.is_cancelled
                && parse_session_end(session) <= now
        })
        .collect();

    completed.sort_by_key(|session| parse_session_end(session));
    completed.pop()
}

pub fn latest_track_conditions(
    session: &Session,
    samples: &[Weather],
) -> Option<TrackConditions> {
    let latest = samples
        .iter()
        .filter(|sample| sample.session_key == session.session_key)
        .max_by_key(|sample| parse_weather_date(sample))?;

    Some(TrackConditions {
        session_name: session.session_name.clone(),
        sampled_at: parse_weather_date(latest),
        air_temperature_c: latest.air_temperature,
        track_temperature_c: latest.track_temperature,
        humidity: latest.humidity,
        rainfall: latest.rainfall,
        wind_speed: latest.wind_speed,
        wind_direction: latest.wind_direction,
    })
}

pub fn format_forecast_summary(forecast: &LocationForecast) -> String {
    if forecast.summary.is_empty() {
        forecast
            .days
            .first()
            .map(|day| {
                format!(
                    "{}-{}°C · {}% rain",
                    day.temp_min_c.round() as i64,
                    day.temp_max_c.round() as i64,
                    day.precip_probability
                )
            })
            .unwrap_or_else(|| "No forecast".into())
    } else {
        forecast.summary.clone()
    }
}

pub fn format_track_summary(track: &TrackConditions) -> String {
    let rain = if track.rainfall > 0 {
        format!("Rain {}", track.rainfall)
    } else {
        "Dry".into()
    };

    format!(
        "Air {}°C · Track {}°C · {rain}",
        track.air_temperature_c.round() as i64,
        track.track_temperature_c.round() as i64,
    )
}

fn parse_session_end(session: &Session) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&session.date_end)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_weather_date(sample: &Weather) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&sample.date)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meeting() -> Meeting {
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
            meeting_key: 7,
            meeting_name: "Test GP".into(),
            meeting_official_name: "TEST GP".into(),
            year: 2026,
        }
    }

    #[test]
    fn builds_location_query() {
        assert_eq!(location_query(&meeting()), "Test City, Testland");
    }
}
