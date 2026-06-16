use chrono::{DateTime, NaiveDate, Utc};
use openf1::Meeting;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use crate::domain::weather::{location_query, ForecastDay, LocationForecast};

const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";

const FORECAST_PAST_DAYS: i64 = 92;
const FORECAST_FUTURE_DAYS: i64 = 16;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("location not found")]
    LocationNotFound,
    #[error("invalid response")]
    InvalidResponse,
    #[error("forecast unavailable: {0}")]
    Unavailable(String),
    #[error("{0}")]
    Api(String),
}

#[derive(Debug, Clone)]
pub struct ForecastData {
    pub meeting_key: i64,
    pub forecast: LocationForecast,
}

#[derive(Debug, Deserialize)]
struct GeocodeResponse {
    results: Option<Vec<GeocodeResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodeResult {
    name: String,
    latitude: f64,
    longitude: f64,
    #[serde(default)]
    country: String,
    #[serde(default)]
    country_code: String,
    #[serde(default)]
    #[allow(dead_code)]
    feature_code: String,
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    daily: DailyForecast,
}

#[derive(Debug, Deserialize)]
struct DailyForecast {
    time: Vec<String>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    #[serde(default)]
    precipitation_probability_max: Vec<Option<f64>>,
    #[serde(default, alias = "weather_code")]
    weathercode: Vec<serde_json::Number>,
}

pub async fn fetch_meeting_forecast(meeting: &Meeting) -> Result<ForecastData, FetchError> {
    let query = location_query(meeting);
    let (lat, lon) = geocode(meeting).await?;
    let forecast = daily_forecast(meeting, &query, lat, lon).await?;

    Ok(ForecastData {
        meeting_key: meeting.meeting_key,
        forecast,
    })
}

async fn geocode(meeting: &Meeting) -> Result<(f64, f64), FetchError> {
    let client = reqwest::Client::new();
    let mut candidates = Vec::new();

    for term in geocode_search_terms(meeting) {
        let found = search_locations(&client, &term).await?;
        for candidate in found {
            if !candidates.iter().any(|existing: &GeocodeResult| {
                (existing.latitude - candidate.latitude).abs() < f64::EPSILON
                    && (existing.longitude - candidate.longitude).abs() < f64::EPSILON
            }) {
                candidates.push(candidate);
            }
        }
    }

    let Some(result) = pick_location(meeting, &candidates) else {
        return Err(FetchError::LocationNotFound);
    };

    Ok((result.latitude, result.longitude))
}

fn geocode_search_terms(meeting: &Meeting) -> Vec<String> {
    let mut terms = Vec::new();
    let mut push = |value: &str| {
        let trimmed = value.trim();
        if trimmed.len() >= 2 && !terms.iter().any(|existing| existing == trimmed) {
            terms.push(trimmed.to_string());
        }
    };

    if meeting.country_name.eq_ignore_ascii_case("Monaco") {
        push("Monaco");
    }

    push(&meeting.location);
    push(&meeting.country_name);
    push(&meeting.circuit_short_name);
    push(&format!("{} {}", meeting.location, meeting.country_name));

    terms
}

async fn search_locations(
    client: &reqwest::Client,
    name: &str,
) -> Result<Vec<GeocodeResult>, FetchError> {
    if name.trim().len() < 2 {
        return Ok(Vec::new());
    }

    let response = client
        .get(GEOCODE_URL)
        .query(&[
            ("name", name),
            ("count", "10"),
            ("language", "en"),
            ("format", "json"),
        ])
        .send()
        .await?
        .error_for_status()?;

    let payload: GeocodeResponse = response.json().await?;
    Ok(payload.results.unwrap_or_default())
}

fn pick_location<'a>(meeting: &Meeting, candidates: &'a [GeocodeResult]) -> Option<&'a GeocodeResult> {
    if candidates.is_empty() {
        return None;
    }

    let country = meeting.country_name.to_ascii_lowercase();
    let location = meeting.location.to_ascii_lowercase();
    let circuit = meeting.circuit_short_name.to_ascii_lowercase();

    if country == "monaco" {
        if let Some(found) = candidates.iter().find(|candidate| {
            candidate.country_code.eq_ignore_ascii_case("MC")
                || candidate.country.eq_ignore_ascii_case("Monaco")
        }) {
            return Some(found);
        }
    }

    candidates
        .iter()
        .find(|candidate| candidate.country.to_ascii_lowercase() == country)
        .or_else(|| {
            candidates.iter().find(|candidate| {
                let name = candidate.name.to_ascii_lowercase();
                name == location
                    || name == circuit
                    || name.contains(&location)
                    || location.contains(&name)
            })
        })
        .or_else(|| candidates.first())
}

async fn daily_forecast(
    meeting: &Meeting,
    query: &str,
    lat: f64,
    lon: f64,
) -> Result<LocationForecast, FetchError> {
    let meeting_start = meeting_date(&meeting.date_start)?;
    let meeting_end = meeting_date(&meeting.date_end)?;
    let today = Utc::now().date_naive();
    let (start_date, end_date) =
        forecast_request_dates(meeting_start, meeting_end, today)?;

    let response = reqwest::Client::new()
        .get(FORECAST_URL)
        .query(&[
            ("latitude", lat.to_string()),
            ("longitude", lon.to_string()),
            (
                "daily",
                "temperature_2m_max,temperature_2m_min,precipitation_probability_max,weathercode"
                    .into(),
            ),
            ("timezone", "auto".into()),
            ("start_date", start_date.format("%Y-%m-%d").to_string()),
            ("end_date", end_date.format("%Y-%m-%d").to_string()),
        ])
        .send()
        .await?
        .error_for_status()?;

    let body = response.text().await?;
    let payload = parse_forecast_response(&body)?;
    let now = Utc::now();
    let mut days = Vec::new();

    for index in 0..payload.daily.time.len() {
        let Some(date) = NaiveDate::parse_from_str(&payload.daily.time[index], "%Y-%m-%d").ok()
        else {
            continue;
        };

        if date < meeting_start || date > meeting_end {
            continue;
        }

        let temp_min = payload.daily.temperature_2m_min.get(index).copied().unwrap_or(0.0);
        let temp_max = payload.daily.temperature_2m_max.get(index).copied().unwrap_or(0.0);
        let precip = payload
            .daily
            .precipitation_probability_max
            .get(index)
            .and_then(|value| value.map(|probability| probability.round() as u8))
            .unwrap_or(0);
        let weather_code = payload
            .daily
            .weathercode
            .get(index)
            .map(number_to_i64)
            .unwrap_or(0);
        let description = weather_code_description(weather_code);

        days.push(ForecastDay {
            label: date.format("%a %d %b").to_string(),
            temp_min_c: temp_min,
            temp_max_c: temp_max,
            precip_probability: precip,
            description,
        });
    }

    if days.is_empty() {
        return Err(FetchError::Unavailable(
            "No forecast days overlap this race weekend".into(),
        ));
    }

    let summary = days
        .first()
        .map(|day| {
            format!(
                "{}-{}°C · {}% rain · {}",
                day.temp_min_c.round() as i64,
                day.temp_max_c.round() as i64,
                day.precip_probability,
                day.description
            )
        })
        .unwrap_or_else(|| "No forecast data".into());

    Ok(LocationForecast {
        fetched_at: now,
        location_query: query.to_string(),
        summary,
        days,
    })
}

fn forecast_request_dates(
    meeting_start: NaiveDate,
    meeting_end: NaiveDate,
    today: NaiveDate,
) -> Result<(NaiveDate, NaiveDate), FetchError> {
    let earliest = today - chrono::Duration::days(FORECAST_PAST_DAYS);
    let latest = today + chrono::Duration::days(FORECAST_FUTURE_DAYS);

    if meeting_end < earliest {
        return Err(FetchError::Unavailable(
            "Race weekend is outside the forecast window".into(),
        ));
    }

    if meeting_start > latest {
        return Err(FetchError::Unavailable(
            "Race weekend is too far ahead for a forecast".into(),
        ));
    }

    let start_date = meeting_start.max(earliest);
    let end_date = meeting_end.min(latest);

    if end_date < start_date {
        return Err(FetchError::Unavailable(
            "Race weekend is outside the forecast window".into(),
        ));
    }

    Ok((start_date, end_date))
}

fn parse_forecast_response(body: &str) -> Result<ForecastResponse, FetchError> {
    let value: Value = serde_json::from_str(body).map_err(|error| FetchError::Api(error.to_string()))?;

    if value
        .get("error")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let reason = value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("forecast request rejected");
        return Err(FetchError::Api(reason.to_string()));
    }

    serde_json::from_value(value).map_err(|error| FetchError::Api(error.to_string()))
}

fn number_to_i64(number: &serde_json::Number) -> i64 {
    number
        .as_i64()
        .or_else(|| number.as_f64().map(|value| value.round() as i64))
        .unwrap_or(0)
}

fn meeting_date(value: &str) -> Result<NaiveDate, FetchError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.date_naive())
        .map_err(|_| FetchError::InvalidResponse)
}

fn weather_code_description(code: i64) -> String {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing drizzle",
        61 => "Light rain",
        63 => "Rain",
        65 => "Heavy rain",
        66 | 67 => "Freezing rain",
        71 => "Light snow",
        73 => "Snow",
        75 => "Heavy snow",
        77 => "Snow grains",
        80 => "Light showers",
        81 => "Showers",
        82 => "Heavy showers",
        85 => "Light snow showers",
        86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with hail",
        _ => "Unknown",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monaco_meeting() -> Meeting {
        Meeting {
            circuit_image: String::new(),
            circuit_info_url: String::new(),
            circuit_key: 22,
            circuit_short_name: "Monte Carlo".into(),
            circuit_type: "Temporary - Street".into(),
            country_code: "MON".into(),
            country_flag: String::new(),
            country_key: 114,
            country_name: "Monaco".into(),
            date_end: "2026-06-07T15:00:00Z".into(),
            date_start: "2026-06-05T11:30:00Z".into(),
            gmt_offset: "02:00:00".into(),
            is_cancelled: false,
            location: "Monte Carlo".into(),
            meeting_key: 1286,
            meeting_name: "Monaco Grand Prix".into(),
            meeting_official_name: "Monaco GP".into(),
            year: 2026,
        }
    }

    fn monza_meeting() -> Meeting {
        Meeting {
            circuit_image: String::new(),
            circuit_info_url: String::new(),
            circuit_key: 1,
            circuit_short_name: "Monza".into(),
            circuit_type: "Permanent".into(),
            country_code: "ITA".into(),
            country_flag: String::new(),
            country_key: 1,
            country_name: "Italy".into(),
            date_end: "2026-09-07T15:00:00Z".into(),
            date_start: "2026-09-05T11:00:00Z".into(),
            gmt_offset: "02:00:00".into(),
            is_cancelled: false,
            location: "Monza".into(),
            meeting_key: 1,
            meeting_name: "Italian Grand Prix".into(),
            meeting_official_name: "Italian GP".into(),
            year: 2026,
        }
    }

    #[test]
    fn weather_code_labels_are_human_readable() {
        assert_eq!(weather_code_description(0), "Clear sky");
        assert_eq!(weather_code_description(63), "Rain");
    }

    #[test]
    fn picks_monaco_over_mexico_monte_carlo() {
        let meeting = monaco_meeting();
        let candidates = vec![
            GeocodeResult {
                name: "Monte Carlo".into(),
                latitude: 16.6425,
                longitude: -93.82972,
                country: "Mexico".into(),
                country_code: "MX".into(),
                feature_code: "PPL".into(),
            },
            GeocodeResult {
                name: "Monaco".into(),
                latitude: 43.73718,
                longitude: 7.42145,
                country: "Monaco".into(),
                country_code: "MC".into(),
                feature_code: "PPLC".into(),
            },
        ];

        let picked = pick_location(&meeting, &candidates).unwrap();
        assert_eq!(picked.country_code, "MC");
    }

    #[test]
    fn monaco_search_terms_prefer_country() {
        let terms = geocode_search_terms(&monaco_meeting());
        assert_eq!(terms.first().map(String::as_str), Some("Monaco"));
    }

    #[test]
    fn parses_open_meteo_error_payload() {
        let body = r#"{"error":true,"reason":"Parameter 'start_date' is out of allowed range"}"#;
        let error = parse_forecast_response(body).unwrap_err();
        assert!(matches!(error, FetchError::Api(_)));
    }

    #[test]
    fn rejects_weekends_outside_forecast_window() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let start = NaiveDate::from_ymd_opt(2025, 5, 23).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 5, 25).unwrap();
        assert!(forecast_request_dates(start, end, today).is_err());
    }

    #[test]
    fn clamps_weekend_into_forecast_window() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let start = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();
        let (request_start, request_end) = forecast_request_dates(start, end, today).unwrap();
        assert_eq!(request_start, start);
        assert_eq!(request_end, end);
    }

    #[test]
    fn picks_country_match_when_available() {
        let meeting = monza_meeting();
        let candidates = vec![
            GeocodeResult {
                name: "Monza".into(),
                latitude: 45.58,
                longitude: 9.27,
                country: "Italy".into(),
                country_code: "IT".into(),
                feature_code: "PPL".into(),
            },
            GeocodeResult {
                name: "Monza".into(),
                latitude: 1.0,
                longitude: 1.0,
                country: "Other".into(),
                country_code: "XX".into(),
                feature_code: "PPL".into(),
            },
        ];

        let picked = pick_location(&meeting, &candidates).unwrap();
        assert_eq!(picked.country, "Italy");
    }
}
