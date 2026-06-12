use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use openf1::{
    ChampionshipDriver, ChampionshipTeam, OpenF1Client, OpenF1Error, OpenF1Key, Session,
    SessionResult, ChampionshipDriversListParams, ChampionshipTeamsListParams,
    SessionResultListParams,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::debug;
use crate::domain::championship::{
    classify_race_results, completed_race_sessions, ChampionshipRoundSnapshot,
    DriverStandingSnapshot, RaceResultInput, RaceResultSnapshot, TeamStandingSnapshot,
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
    #[serde(default)]
    pub race_results: Vec<RaceResultBlob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverStandingBlob {
    pub driver_number: i64,
    pub position: i64,
    pub points: i64,
    #[serde(default)]
    pub race_points: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStandingBlob {
    pub team_name: String,
    pub position: i64,
    pub points: i64,
    #[serde(default)]
    pub race_points: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResultBlob {
    pub driver_number: i64,
    pub classified_position: i64,
    pub dnf: bool,
    pub dns: bool,
    pub dsq: bool,
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
                            race_points: driver.race_points,
                        })
                        .collect(),
                    teams: round
                        .teams
                        .iter()
                        .map(|team| TeamStandingBlob {
                            team_name: team.team_name.clone(),
                            position: team.position,
                            points: team.points,
                            race_points: team.race_points,
                        })
                        .collect(),
                    race_results: round
                        .race_results
                        .iter()
                        .map(|result| RaceResultBlob {
                            driver_number: result.driver_number,
                            classified_position: result.classified_position,
                            dnf: result.dnf,
                            dns: result.dns,
                            dsq: result.dsq,
                            points: result.points,
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
                        race_points: driver.race_points,
                    })
                    .collect(),
                teams: round
                    .teams
                    .into_iter()
                    .map(|team| TeamStandingSnapshot {
                        team_name: team.team_name,
                        position: team.position,
                        points: team.points,
                        race_points: team.race_points,
                    })
                    .collect(),
                race_results: round
                    .race_results
                    .into_iter()
                    .map(|result| RaceResultSnapshot {
                        driver_number: result.driver_number,
                        classified_position: result.classified_position,
                        dnf: result.dnf,
                        dns: result.dns,
                        dsq: result.dsq,
                        points: result.points,
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

    for (round, session) in &completed {
        if let Some(cached) = cached_rounds.get(&session.session_key) {
            if round_has_race_results(cached) {
                rounds.push(snapshot_with_round(*round, cached));
                continue;
            }

            if round_has_championship_data(cached) {
                match fetch_race_results(&client, session).await {
                    Ok(race_results) if !race_results.is_empty() => {
                        fetched_any = true;
                        rounds.push(ChampionshipRoundSnapshot {
                            round: *round,
                            session_key: cached.session_key,
                            meeting_key: cached.meeting_key,
                            drivers: cached.drivers.clone(),
                            teams: cached.teams.clone(),
                            race_results,
                        });
                        continue;
                    }
                    Ok(_) => {
                        debug::warn(format!(
                            "Race results empty for round {round} (session {})",
                            session.session_key
                        ));
                        rounds.push(snapshot_with_round(*round, cached));
                        continue;
                    }
                    Err(error) => {
                        debug::warn(format!(
                            "Race results unavailable for round {round} (session {}): {error}",
                            session.session_key
                        ));
                        rounds.push(snapshot_with_round(*round, cached));
                        continue;
                    }
                }
            }
        }

        if fetched_any {
            tokio::time::sleep(REQUEST_DELAY).await;
        }

        match fetch_round(&client, *round, session).await {
            Ok(snapshot) => {
                fetched_any = true;
                rounds.push(snapshot);
            }
            Err(error) => {
                if let Some(cached) = cached_rounds.get(&session.session_key) {
                    debug::warn(format!(
                        "Round {round} fetch failed, using cached snapshot: {error}"
                    ));
                    rounds.push(snapshot_with_round(*round, cached));
                    continue;
                }

                if rounds.is_empty() {
                    return Err(error);
                }

                debug::warn(format!(
                    "Round {round} fetch failed with no cache, skipping: {error}"
                ));
            }
        }
    }

    merge_missing_cached_rounds(&mut rounds, &completed, &cached_rounds);

    Ok(ChampionshipData {
        season,
        rounds,
        fetched_at: now,
    })
}

fn round_has_race_results(round: &ChampionshipRoundSnapshot) -> bool {
    !round.race_results.is_empty()
}

fn round_has_championship_data(round: &ChampionshipRoundSnapshot) -> bool {
    !round.drivers.is_empty()
}

fn snapshot_with_round(round: u32, cached: &ChampionshipRoundSnapshot) -> ChampionshipRoundSnapshot {
    ChampionshipRoundSnapshot {
        round,
        session_key: cached.session_key,
        meeting_key: cached.meeting_key,
        drivers: cached.drivers.clone(),
        teams: cached.teams.clone(),
        race_results: cached.race_results.clone(),
    }
}

fn merge_missing_cached_rounds(
    rounds: &mut Vec<ChampionshipRoundSnapshot>,
    completed: &[(u32, Session)],
    cached_rounds: &HashMap<i64, ChampionshipRoundSnapshot>,
) {
    for (round, session) in completed {
        if rounds
            .iter()
            .any(|entry| entry.session_key == session.session_key)
        {
            continue;
        }

        if let Some(cached) = cached_rounds.get(&session.session_key) {
            debug::warn(format!(
                "Restoring cached round {round} (session {}) after partial fetch",
                session.session_key
            ));
            rounds.push(snapshot_with_round(*round, cached));
        }
    }

    rounds.sort_by_key(|round| round.round);
}

async fn fetch_race_results(
    client: &OpenF1Client,
    session: &Session,
) -> Result<Vec<RaceResultSnapshot>, FetchError> {
    let results = fetch_session_results(client, session.session_key).await?;
    Ok(map_race_results(&results))
}

async fn fetch_session_results(
    client: &OpenF1Client,
    session_key: i64,
) -> Result<Vec<SessionResult>, FetchError> {
    client
        .session_result()
        .list(SessionResultListParams {
            session_key: Some(OpenF1Key::Id(session_key)),
            position: None,
        })
        .await
        .map_err(FetchError::from)
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
    let results = fetch_session_results(client, session.session_key)
        .await
        .unwrap_or_else(|error| {
            debug::warn(format!(
                "session_result unavailable for session {}: {error}",
                session.session_key
            ));
            Vec::new()
        });

    Ok(ChampionshipRoundSnapshot {
        round,
        session_key: session.session_key,
        meeting_key: session.meeting_key,
        drivers: map_driver_standings(drivers),
        teams: map_team_standings(teams),
        race_results: map_race_results(&results),
    })
}

fn map_race_results(results: &[SessionResult]) -> Vec<RaceResultSnapshot> {
    let inputs: Vec<RaceResultInput> = results
        .iter()
        .map(|result| RaceResultInput {
            driver_number: result.driver_number,
            position: result.position,
            number_of_laps: result.number_of_laps,
            dnf: result.dnf,
            dns: result.dns,
            dsq: result.dsq,
            points: result.points.unwrap_or(0),
        })
        .collect();
    classify_race_results(&inputs)
}

fn map_driver_standings(rows: Vec<ChampionshipDriver>) -> Vec<DriverStandingSnapshot> {
    rows.into_iter()
        .map(|row| DriverStandingSnapshot {
            driver_number: row.driver_number,
            position: row.position_current,
            points: row.points_current,
            race_points: row.points_current.saturating_sub(row.points_start),
        })
        .collect()
}

fn map_team_standings(rows: Vec<ChampionshipTeam>) -> Vec<TeamStandingSnapshot> {
    rows.into_iter()
        .map(|row| TeamStandingSnapshot {
            team_name: row.team_name,
            position: row.position_current,
            points: row.points_current,
            race_points: row.points_current.saturating_sub(row.points_start),
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
                    race_points: 25,
                }],
                teams: vec![TeamStandingSnapshot {
                    team_name: "Team".into(),
                    position: 1,
                    points: 43,
                    race_points: 43,
                }],
                race_results: vec![RaceResultSnapshot {
                    driver_number: 1,
                    classified_position: 1,
                    dnf: false,
                    dns: false,
                    dsq: false,
                    points: 25,
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
        assert!(round_has_race_results(&existing.rounds[0]));
    }
}
