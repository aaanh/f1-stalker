use chrono::{DateTime, Utc};
use openf1::{
    OpenF1Client, OpenF1Error, OpenF1Key, Session, StartingGrid, StartingGridListParams,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::grid::{build_grid_slots, find_gp_qualifying, find_sprint_qualifying, GridSlot};

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("openf1: {0}")]
    Api(#[from] OpenF1Error),
}

#[derive(Debug, Clone)]
pub struct QualiGridData {
    pub meeting_key: i64,
    pub slots: Vec<GridSlot>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualiGridCacheBlob {
    pub meeting_key: i64,
    pub slots: Vec<GridSlotBlob>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSlotBlob {
    pub driver_number: i64,
    pub position: i64,
    pub gap_to_pole_secs: Option<f64>,
}

impl QualiGridCacheBlob {
    pub fn from_data(data: &QualiGridData) -> Self {
        Self {
            meeting_key: data.meeting_key,
            slots: data
                .slots
                .iter()
                .map(|slot| GridSlotBlob {
                    driver_number: slot.driver_number,
                    position: slot.position,
                    gap_to_pole_secs: slot.gap_to_pole_secs,
                })
                .collect(),
            fetched_at: data.fetched_at,
        }
    }
}

pub fn quali_grid_from_cache(blob: QualiGridCacheBlob) -> QualiGridData {
    QualiGridData {
        meeting_key: blob.meeting_key,
        slots: blob
            .slots
            .into_iter()
            .map(|slot| GridSlot {
                driver_number: slot.driver_number,
                position: slot.position,
                gap_to_pole_secs: slot.gap_to_pole_secs,
            })
            .collect(),
        fetched_at: blob.fetched_at,
    }
}

pub async fn fetch_quali_grid(
    meeting_key: i64,
    sessions: &[Session],
    pinned_numbers: &[i64],
) -> Result<QualiGridData, FetchError> {
    let now = Utc::now();
    let Some(quali) = find_gp_qualifying(sessions, meeting_key) else {
        return Ok(QualiGridData {
            meeting_key,
            slots: Vec::new(),
            fetched_at: now,
        });
    };

    let grid = fetch_starting_grid(quali.session_key).await?;
    let slots = build_grid_slots(&grid, pinned_numbers);

    Ok(QualiGridData {
        meeting_key,
        slots,
        fetched_at: now,
    })
}

pub async fn fetch_sprint_quali_grid(
    meeting_key: i64,
    sessions: &[Session],
    pinned_numbers: &[i64],
) -> Result<QualiGridData, FetchError> {
    let now = Utc::now();
    let Some(sprint_quali) = find_sprint_qualifying(sessions, meeting_key) else {
        return Ok(QualiGridData {
            meeting_key,
            slots: Vec::new(),
            fetched_at: now,
        });
    };

    let grid = fetch_starting_grid(sprint_quali.session_key).await?;
    let slots = build_grid_slots(&grid, pinned_numbers);

    Ok(QualiGridData {
        meeting_key,
        slots,
        fetched_at: now,
    })
}

async fn fetch_starting_grid(session_key: i64) -> Result<Vec<StartingGrid>, FetchError> {
    let client = OpenF1Client::new(None);
    client
        .starting_grid()
        .list(StartingGridListParams {
            session_key: Some(OpenF1Key::Id(session_key)),
        })
        .await
        .map_err(Into::into)
}
