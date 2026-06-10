use chrono::{DateTime, Utc};
use iced::Color;
use openf1::{Driver, Meeting, Session};

use crate::db::PinnedDriver;
use crate::domain::{driver_display_name, team_colour};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChampionshipTab {
    #[default]
    Drivers,
    Constructors,
}

#[derive(Debug, Clone)]
pub struct StandingPoint {
    pub round: u32,
    pub position: i64,
    pub points: i64,
}

#[derive(Debug, Clone)]
pub struct ChartSeries {
    pub label: String,
    pub code: String,
    pub color: Color,
    pub points: Vec<StandingPoint>,
}

#[derive(Debug, Clone)]
pub struct ChampionshipRoundSnapshot {
    pub round: u32,
    pub session_key: i64,
    pub meeting_key: i64,
    pub drivers: Vec<DriverStandingSnapshot>,
    pub teams: Vec<TeamStandingSnapshot>,
}

#[derive(Debug, Clone)]
pub struct DriverStandingSnapshot {
    pub driver_number: i64,
    pub position: i64,
    pub points: i64,
}

#[derive(Debug, Clone)]
pub struct TeamStandingSnapshot {
    pub team_name: String,
    pub position: i64,
    pub points: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionAxis {
    pub min: u32,
    pub max: u32,
}

impl PositionAxis {
    pub fn from_series(series: &[ChartSeries]) -> Self {
        let mut min: Option<i64> = None;
        let mut max: Option<i64> = None;
        for entry in series {
            for point in &entry.points {
                min = Some(min.map_or(point.position, |value| value.min(point.position)));
                max = Some(max.map_or(point.position, |value| value.max(point.position)));
            }
        }

        match (min, max) {
            (Some(lo), Some(hi)) => {
                let min = lo.max(1) as u32;
                let max = hi.max(1) as u32;
                Self {
                    min: min.min(max),
                    max: max.max(min),
                }
            }
            _ => Self { min: 1, max: 10 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChampionshipCharts {
    pub round_count: u32,
    pub driver_axis: PositionAxis,
    pub constructor_axis: PositionAxis,
    pub round_labels: Vec<String>,
    pub driver_series: Vec<ChartSeries>,
    pub constructor_series: Vec<ChartSeries>,
}

pub fn round_axis_labels(
    rounds: &[ChampionshipRoundSnapshot],
    meetings: &[Meeting],
    sessions: &[Session],
) -> Vec<String> {
    rounds
        .iter()
        .map(|round| {
            let session = sessions
                .iter()
                .find(|session| session.session_key == round.session_key);
            let meeting = meetings
                .iter()
                .find(|meeting| meeting.meeting_key == round.meeting_key);
            round_axis_label(meeting, session, round.round)
        })
        .collect()
}

pub fn round_axis_label(
    meeting: Option<&Meeting>,
    session: Option<&Session>,
    round: u32,
) -> String {
    let session_name = session
        .map(|session| session.session_name.as_str())
        .unwrap_or("Race");

    if session_name == "Sprint" {
        return meeting
            .map(|meeting| format!("{} Sprint", gp_short_name(meeting)))
            .unwrap_or_else(|| "Sprint".into());
    }

    meeting
        .map(gp_short_name)
        .or_else(|| session.map(|session| session.location.clone()))
        .unwrap_or_else(|| round.to_string())
}

fn gp_short_name(meeting: &Meeting) -> String {
    let name = meeting.meeting_name.trim();
    if name.is_empty() {
        return meeting.location.clone();
    }
    if let Some(stripped) = name.strip_suffix(" Grand Prix") {
        return stripped.to_string();
    }
    name.to_string()
}

fn constructor_code(team_name: &str) -> String {
    team_name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_uppercase()
}

pub fn completed_race_sessions(sessions: &[Session], now: DateTime<Utc>) -> Vec<(u32, Session)> {
    let mut races: Vec<Session> = sessions
        .iter()
        .filter(|session| {
            !session.is_cancelled
                && session.session_type == "Race"
                && parse_session_end(session) < now
        })
        .cloned()
        .collect();
    races.sort_by(|left, right| {
        parse_session_start(left)
            .cmp(&parse_session_start(right))
            .then_with(|| left.session_key.cmp(&right.session_key))
    });

    races
        .into_iter()
        .enumerate()
        .map(|(index, session)| ((index + 1) as u32, session))
        .collect()
}

pub fn build_championship_charts(
    rounds: &[ChampionshipRoundSnapshot],
    pinned: &[PinnedDriver],
    roster: &[Driver],
    meetings: &[Meeting],
    sessions: &[Session],
) -> ChampionshipCharts {
    let round_count = rounds.len() as u32;
    let round_labels = round_axis_labels(rounds, meetings, sessions);

    let driver_series = pinned
        .iter()
        .filter_map(|pin| {
            roster
                .iter()
                .find(|driver| driver.driver_number == pin.driver_number)
                .map(|driver| {
                    let points = rounds
                        .iter()
                        .filter_map(|round| {
                            round
                                .drivers
                                .iter()
                                .find(|entry| entry.driver_number == pin.driver_number)
                                .map(|entry| StandingPoint {
                                    round: round.round,
                                    position: entry.position,
                                    points: entry.points,
                                })
                        })
                        .collect();
                    ChartSeries {
                        label: driver_display_name(driver).to_string(),
                        code: driver.name_acronym.clone(),
                        color: team_colour(&driver.team_colour),
                        points,
                    }
                })
        })
        .collect::<Vec<_>>();

    let constructor_series = if let Some(latest) = rounds.last() {
        let mut teams = latest.teams.clone();
        teams.sort_by_key(|team| team.position);
        teams
            .into_iter()
            .take(10)
            .map(|team| {
                let points = rounds
                    .iter()
                    .filter_map(|round| {
                        round
                            .teams
                            .iter()
                            .find(|entry| entry.team_name == team.team_name)
                            .map(|entry| StandingPoint {
                                round: round.round,
                                position: entry.position,
                                points: entry.points,
                            })
                    })
                    .collect();
                ChartSeries {
                    label: team.team_name.clone(),
                    code: constructor_code(&team.team_name),
                    color: constructor_colour(&team.team_name, roster),
                    points,
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let driver_axis = PositionAxis::from_series(&driver_series);
    let constructor_axis = PositionAxis::from_series(&constructor_series);

    ChampionshipCharts {
        round_count,
        driver_axis,
        constructor_axis,
        round_labels,
        driver_series,
        constructor_series,
    }
}

fn constructor_colour(team_name: &str, roster: &[Driver]) -> Color {
    roster
        .iter()
        .find(|driver| driver.team_name == team_name)
        .map(|driver| team_colour(&driver.team_colour))
        .unwrap_or_else(|| team_colour("808080"))
}

fn parse_session_start(session: &Session) -> DateTime<Utc> {
    parse_session_timestamp(&session.date_start)
}

fn parse_session_end(session: &Session) -> DateTime<Utc> {
    parse_session_timestamp(&session.date_end)
}

fn parse_session_timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session(key: i64, start: &str, end: &str) -> Session {
        Session {
            circuit_key: 1,
            circuit_short_name: "Test".into(),
            country_code: "TST".into(),
            country_key: 1,
            country_name: "Testland".into(),
            date_end: end.into(),
            date_start: start.into(),
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

    fn sample_driver(number: i64, acronym: &str, team: &str, colour: &str) -> Driver {
        Driver {
            broadcast_name: acronym.into(),
            country_code: String::new(),
            driver_number: number,
            first_name: acronym.into(),
            full_name: format!("Driver {number}"),
            headshot_url: String::new(),
            last_name: acronym.into(),
            meeting_key: 1,
            name_acronym: acronym.into(),
            session_key: 1,
            team_colour: colour.into(),
            team_name: team.into(),
        }
    }

    #[test]
    fn completed_race_sessions_assign_round_numbers() {
        let now = DateTime::parse_from_rfc3339("2026-04-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let sessions = vec![
            sample_session(
                2,
                "2026-03-20T12:00:00Z",
                "2026-03-20T15:00:00Z",
            ),
            sample_session(
                1,
                "2026-03-06T12:00:00Z",
                "2026-03-06T15:00:00Z",
            ),
            sample_session(
                3,
                "2026-04-10T12:00:00Z",
                "2026-04-10T15:00:00Z",
            ),
        ];

        let completed = completed_race_sessions(&sessions, now);
        assert_eq!(completed.len(), 2);
        assert_eq!(completed[0].0, 1);
        assert_eq!(completed[1].0, 2);
        assert_eq!(completed[0].1.meeting_key, 1);
    }

    #[test]
    fn build_charts_for_pinned_drivers_and_top_constructors() {
        let rounds = vec![
            ChampionshipRoundSnapshot {
                round: 1,
                session_key: 10,
                meeting_key: 1,
                drivers: vec![
                    DriverStandingSnapshot {
                        driver_number: 1,
                        position: 2,
                        points: 18,
                    },
                    DriverStandingSnapshot {
                        driver_number: 44,
                        position: 1,
                        points: 25,
                    },
                ],
                teams: vec![
                    TeamStandingSnapshot {
                        team_name: "Red Bull Racing".into(),
                        position: 1,
                        points: 43,
                    },
                    TeamStandingSnapshot {
                        team_name: "Ferrari".into(),
                        position: 2,
                        points: 30,
                    },
                ],
            },
            ChampionshipRoundSnapshot {
                round: 2,
                session_key: 20,
                meeting_key: 2,
                drivers: vec![
                    DriverStandingSnapshot {
                        driver_number: 1,
                        position: 1,
                        points: 43,
                    },
                    DriverStandingSnapshot {
                        driver_number: 44,
                        position: 3,
                        points: 33,
                    },
                ],
                teams: vec![
                    TeamStandingSnapshot {
                        team_name: "Red Bull Racing".into(),
                        position: 1,
                        points: 76,
                    },
                    TeamStandingSnapshot {
                        team_name: "Ferrari".into(),
                        position: 2,
                        points: 55,
                    },
                ],
            },
        ];
        let pinned = vec![PinnedDriver {
            driver_number: 1,
            sort_order: 0,
        }];
        let roster = vec![
            sample_driver(1, "VER", "Red Bull Racing", "3671C6"),
            sample_driver(44, "HAM", "Ferrari", "E80020"),
        ];

        let charts = build_championship_charts(&rounds, &pinned, &roster, &[], &[]);
        assert_eq!(charts.round_count, 2);
        assert_eq!(charts.round_labels, vec!["1", "2"]);
        assert_eq!(charts.driver_axis, PositionAxis { min: 1, max: 2 });
        assert_eq!(charts.driver_series.len(), 1);
        assert_eq!(charts.driver_series[0].points.len(), 2);
        assert_eq!(charts.constructor_series.len(), 2);
        assert_eq!(charts.constructor_series[0].label, "Red Bull Racing");
    }

    fn sample_meeting(key: i64, name: &str) -> Meeting {
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
            meeting_key: key,
            meeting_name: name.into(),
            meeting_official_name: name.into(),
            year: 2026,
        }
    }

    #[test]
    fn round_axis_label_uses_gp_short_name() {
        let meeting = sample_meeting(1, "Australian Grand Prix");
        let session = sample_session(1, "2026-03-06T12:00:00Z", "2026-03-06T15:00:00Z");
        assert_eq!(
            round_axis_label(Some(&meeting), Some(&session), 1),
            "Australian"
        );
    }

    #[test]
    fn round_axis_label_uses_sprint_name() {
        let meeting = sample_meeting(2, "Chinese Grand Prix");
        let mut session = sample_session(2, "2026-03-20T12:00:00Z", "2026-03-20T15:00:00Z");
        session.session_name = "Sprint".into();
        assert_eq!(
            round_axis_label(Some(&meeting), Some(&session), 2),
            "Chinese Sprint"
        );
    }
}
