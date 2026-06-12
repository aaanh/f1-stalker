use chrono::{DateTime, Utc};
use openf1::{Meeting, Session};

use crate::data::ChampionshipData;
use crate::data::QualiGridData;
use crate::db::store::CacheEntry;
use crate::domain::championship::{completed_race_sessions, ChampionshipRoundSnapshot};
use crate::domain::{find_gp_qualifying, quali_has_ended};

pub fn cache_is_fresh(entry: &CacheEntry, now: DateTime<Utc>) -> bool {
    !entry.is_expired(now)
}

pub fn championship_needs_refresh(
    cached: Option<&ChampionshipData>,
    sessions: &[Session],
    now: DateTime<Utc>,
) -> bool {
    let completed = completed_race_sessions(sessions, now);
    let cached_sessions = cached
        .map(|data| {
            data.rounds
                .iter()
                .map(|round| round.session_key)
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();

    completed
        .iter()
        .any(|(_, session)| !cached_sessions.contains(&session.session_key))
        || cached
            .map(|data| data.rounds.iter().any(|round| round.race_results.is_empty()))
            .unwrap_or(false)
}

pub fn weekend_weather_needs_refresh(
    meetings: &[Meeting],
    now: DateTime<Utc>,
    forecast_fresh: impl Fn(i64) -> bool,
    track_fresh: impl Fn(i64) -> bool,
) -> bool {
    meetings.iter().any(|meeting| {
        !forecast_fresh(meeting.meeting_key) || !track_fresh(meeting.meeting_key)
    }) && !meetings.is_empty()
}

pub fn quali_grid_needs_refresh(
    focus_meeting: &Meeting,
    sessions: &[Session],
    cached: Option<&QualiGridData>,
    now: DateTime<Utc>,
) -> bool {
    let Some(quali) = find_gp_qualifying(sessions, focus_meeting.meeting_key) else {
        return false;
    };

    if !quali_has_ended(quali, now) {
        return false;
    }

    cached
        .map(|data| data.slots.is_empty())
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::championship::{
        DriverStandingSnapshot, RaceResultSnapshot, TeamStandingSnapshot,
    };

    fn sample_session(key: i64, end: &str) -> Session {
        Session {
            circuit_key: 1,
            circuit_short_name: "Test".into(),
            country_code: "TST".into(),
            country_key: 1,
            country_name: "Testland".into(),
            date_end: end.into(),
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
    fn detects_missing_completed_rounds() {
        let now = DateTime::parse_from_rfc3339("2026-04-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let sessions = vec![
            sample_session(1, "2026-03-06T15:00:00Z"),
            sample_session(2, "2026-03-20T15:00:00Z"),
        ];
        let cached = ChampionshipData {
            season: 2026,
            rounds: vec![ChampionshipRoundSnapshot {
                round: 1,
                session_key: 10,
                meeting_key: 1,
                drivers: vec![],
                teams: vec![],
                race_results: vec![],
            }],
            fetched_at: now,
        };

        assert!(championship_needs_refresh(Some(&cached), &sessions, now));
    }

    #[test]
    fn skips_refresh_when_all_completed_rounds_cached() {
        let now = DateTime::parse_from_rfc3339("2026-04-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let sessions = vec![sample_session(1, "2026-03-06T15:00:00Z")];
        let cached = ChampionshipData {
            season: 2026,
            rounds: vec![ChampionshipRoundSnapshot {
                round: 1,
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
            fetched_at: now,
        };

        assert!(!championship_needs_refresh(Some(&cached), &sessions, now));
    }
}
