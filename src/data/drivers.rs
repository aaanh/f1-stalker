use chrono::{DateTime, Utc};
use openf1::{Driver, DriversListParams, OpenF1Client, OpenF1Error, OpenF1Key};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("openf1: {0}")]
    Api(#[from] OpenF1Error),
    #[error("no drivers returned")]
    Empty,
}

#[derive(Debug, Clone)]
pub struct DriversData {
    pub season: i32,
    pub drivers: Vec<Driver>,
    pub fetched_at: DateTime<Utc>,
}

pub async fn fetch_season_drivers(season: i32) -> Result<DriversData, FetchError> {
    let client = OpenF1Client::new(None);
    let drivers = client
        .drivers()
        .list(DriversListParams {
            session_key: Some(OpenF1Key::Latest),
            driver_number: None,
        })
        .await?;

    if drivers.is_empty() {
        return Err(FetchError::Empty);
    }

    Ok(DriversData {
        season,
        drivers: dedupe_drivers(drivers),
        fetched_at: Utc::now(),
    })
}

fn dedupe_drivers(drivers: Vec<Driver>) -> Vec<Driver> {
    let mut unique = std::collections::HashMap::new();
    for driver in drivers {
        unique.entry(driver.driver_number).or_insert(driver);
    }
    let mut roster: Vec<Driver> = unique.into_values().collect();
    roster.sort_by(|left, right| left.driver_number.cmp(&right.driver_number));
    roster
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_driver(number: i64, name: &str) -> Driver {
        Driver {
            broadcast_name: name.into(),
            country_code: "GB".into(),
            driver_number: number,
            first_name: name.into(),
            full_name: name.into(),
            headshot_url: String::new(),
            last_name: name.into(),
            meeting_key: 1,
            name_acronym: name.into(),
            session_key: 1,
            team_colour: "3671C6".into(),
            team_name: "Team".into(),
        }
    }

    #[test]
    fn dedupe_keeps_latest_entry_per_number() {
        let drivers = dedupe_drivers(vec![
            sample_driver(44, "HAM"),
            sample_driver(44, "HAM2"),
            sample_driver(1, "VER"),
        ]);
        assert_eq!(drivers.len(), 2);
        assert_eq!(drivers[0].driver_number, 1);
        assert_eq!(drivers[1].driver_number, 44);
        assert_eq!(drivers[1].name_acronym, "HAM");
    }
}
