use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use openf1::{
    ChampionshipDriver, ChampionshipTeam, OpenF1Client, OpenF1Error, OpenF1Key, Session,
    ChampionshipDriversListParams, ChampionshipTeamsListParams,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::debug;
use crate::domain::championship::{
    completed_race_sessions, ChampionshipRoundSnapshot, DriverStandingSnapshot,
    TeamStandingSnapshot,
};

const REQUEST_DELAY: Duration = Duration::from_millis(300);

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("openf1: {0}")]
    Api(#[from] OpenF1Error),
}

#[derive(Debug, Clone)]
pub struct ChampionshipData {
    pub season: i32,
    pub rounds: Vec<ChampionshipRoundSnapshot>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionshipCacheBlob {
    pub season: i32,
    pub rounds: Vec<ChampionshipRoundBlob>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionshipRoundBlob {
    pub round: u32,
    pub session_key: i64,
    pub meeting_key: i64,
    pub drivers: Vec<DriverStandingBlob>,
    pub teams: Vec<TeamStandingBlob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverStandingBlob {
    pub driver_number: i64,
    pub position: i64,
    pub points: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStandingBlob {
    pub team_name: String,
    pub position: i64,
    pub points: i64,
}

impl ChampionshipCacheBlob {
    pub fn from_data(data: &ChampionshipData) -> Self {
        Self {
            season: data.season,
            rounds: data
                .rounds
                .iter()
                .map(|round| ChampionshipRoundBlob {
                    round: round.round,
                    session_key: round.session_key,
                    meeting_key: round.meeting_key,
                    drivers: round
                        .drivers
                        .iter()
                        .map(|driver| DriverStandingBlob {
                            driver_number: driver.driver_number,
                            position: driver.position,
                            points: driver.points,
                        })
                        .collect(),
                    teams: round
                        .teams
                        .iter()
                        .map(|team| TeamStandingBlob {
                            team_name: team.team_name.clone(),
                            position: team.position,
                            points: team.points,
                        })
                        .collect(),
                })
                .collect(),
            fetched_at: data.fetched_at,
        }
    }
}

pub fn championship_from_cache(blob: ChampionshipCacheBlob) -> ChampionshipData {
    ChampionshipData {
        season: blob.season,
        rounds: blob
            .rounds
            .into_iter()
            .map(|round| ChampionshipRoundSnapshot {
                round: round.round,
                session_key: round.session_key,
                meeting_key: round.meeting_key,
                drivers: round
                    .drivers
                    .into_iter()
                    .map(|driver| DriverStandingSnapshot {
                        driver_number: driver.driver_number,
                        position: driver.position,
                        points: driver.points,
                    })
                    .collect(),
                teams: round
                    .teams
                    .into_iter()
                    .map(|team| TeamStandingSnapshot {
                        team_name: team.team_name,
                        position: team.position,
                        points: team.points,
                    })
                    .collect(),
            })
            .collect(),
        fetched_at: blob.fetched_at,
    }
}

pub async fn fetch_season_championship(
    season: i32,
    sessions: &[Session],
    existing: Option<ChampionshipData>,
) -> Result<ChampionshipData, FetchError> {
    let client = OpenF1Client::new(None);
    let now = Utc::now();
    let completed = completed_race_sessions(sessions, now);

    if completed.is_empty() {
        return Ok(ChampionshipData {
            season,
            rounds: Vec::new(),
            fetched_at: now,
        });
    }

    let cached_rounds: HashMap<i64, ChampionshipRoundSnapshot> = existing
        .as_ref()
        .map(|data| {
            data.rounds
                .iter()
                .map(|round| (round.session_key, round.clone()))
                .collect()
        })
        .unwrap_or_default();

    let mut rounds = Vec::with_capacity(completed.len());
    let mut fetched_any = false;

    for (round, session) in completed {
        if let Some(cached) = cached_rounds.get(&session.session_key) {
            rounds.push(ChampionshipRoundSnapshot {
                round,
                session_key: cached.session_key,
                meeting_key: cached.meeting_key,
                drivers: cached.drivers.clone(),
                teams: cached.teams.clone(),
            });
            continue;
        }

        if fetched_any {
            tokio::time::sleep(REQUEST_DELAY).await;
        }

        match fetch_round(&client, round, &session).await {
            Ok(snapshot) => {
                fetched_any = true;
                rounds.push(snapshot);
            }
            Err(error) => {
                if rounds.is_empty() {
                    return Err(error);
                }

                debug::warn(format!(
                    "Championship fetch stopped after {} rounds: {error}",
                    rounds.len()
                ));
                break;
            }
        }
    }

    Ok(ChampionshipData {
        season,
        rounds,
        fetched_at: now,
    })
}

async fn fetch_round(
    client: &OpenF1Client,
    round: u32,
    session: &Session,
) -> Result<ChampionshipRoundSnapshot, FetchError> {
    let drivers = client
        .championship_drivers()
        .list(ChampionshipDriversListParams {
            session_key: Some(OpenF1Key::Id(session.session_key)),
            driver_number: None,
        })
        .await?;
    let teams = client
        .championship_teams()
        .list(ChampionshipTeamsListParams {
            session_key: Some(OpenF1Key::Id(session.session_key)),
            team_name: None,
        })
        .await?;

    Ok(ChampionshipRoundSnapshot {
        round,
        session_key: session.session_key,
        meeting_key: session.meeting_key,
        drivers: map_driver_standings(drivers),
        teams: map_team_standings(teams),
    })
}

fn map_driver_standings(rows: Vec<ChampionshipDriver>) -> Vec<DriverStandingSnapshot> {
    rows.into_iter()
        .map(|row| DriverStandingSnapshot {
            driver_number: row.driver_number,
            position: row.position_current,
            points: row.points_current,
        })
        .collect()
}

fn map_team_standings(rows: Vec<ChampionshipTeam>) -> Vec<TeamStandingSnapshot> {
    rows.into_iter()
        .map(|row| TeamStandingSnapshot {
            team_name: row.team_name,
            position: row.position_current,
            points: row.points_current,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::championship::ChampionshipRoundSnapshot;

    fn sample_session(key: i64) -> Session {
        Session {
            circuit_key: 1,
            circuit_short_name: "Test".into(),
            country_code: "TST".into(),
            country_key: 1,
            country_name: "Testland".into(),
            date_end: "2026-03-06T15:00:00Z".into(),
            date_start: "2026-03-06T12:00:00Z".into(),
            gmt_offset: "00:00:00".into(),
            is_cancelled: false,
            location: "Test City".into(),
            meeting_key: key,
            session_key: key * 10,
            session_name: "Race".into(),
            session_type: "Race".into(),
            year: 2026,
        }
    }

    #[test]
    fn reuses_cached_rounds_by_session_key() {
        let existing = ChampionshipData {
            season: 2026,
            rounds: vec![ChampionshipRoundSnapshot {
                round: 99,
                session_key: 10,
                meeting_key: 1,
                drivers: vec![DriverStandingSnapshot {
                    driver_number: 1,
                    position: 1,
                    points: 25,
                }],
                teams: vec![TeamStandingSnapshot {
                    team_name: "Team".into(),
                    position: 1,
                    points: 43,
                }],
            }],
            fetched_at: Utc::now(),
        };

        let sessions = vec![sample_session(1)];
        let completed = completed_race_sessions(
            &sessions,
            DateTime::parse_from_rfc3339("2026-04-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        );
        assert_eq!(completed.len(), 1);

        let cached_rounds: HashMap<i64, ChampionshipRoundSnapshot> = existing
            .rounds
            .iter()
            .map(|round| (round.session_key, round.clone()))
            .collect();
        assert!(cached_rounds.contains_key(&10));
    }
}
