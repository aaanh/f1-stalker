use chrono::{DateTime, Utc};
use iced::Color;
use openf1::{Driver, Meeting, Session};

use crate::db::PinnedDriver;
use crate::domain::{driver_display_name, team_colour};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChampionshipTab {
    #[default]
    Drivers,
    Constructors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartMode {
    #[default]
    Championship,
    #[serde(rename = "race_standing")]
    RaceStanding,
}

impl ChampionshipTab {
    pub fn from_key(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "constructors" => Self::Constructors,
            _ => Self::Drivers,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Drivers => "drivers",
            Self::Constructors => "constructors",
        }
    }
}

impl ChartMode {
    pub fn from_key(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "race_standing" | "race" => Self::RaceStanding,
            _ => Self::Championship,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Championship => "championship",
            Self::RaceStanding => "race_standing",
        }
    }
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
    pub race_results: Vec<RaceResultSnapshot>,
}

#[derive(Debug, Clone)]
pub struct RaceResultSnapshot {
    pub driver_number: i64,
    pub classified_position: i64,
    pub dnf: bool,
    pub dns: bool,
    pub dsq: bool,
    pub points: i64,
}

#[derive(Debug, Clone)]
pub struct DriverStandingSnapshot {
    pub driver_number: i64,
    pub position: i64,
    pub points: i64,
    pub race_points: i64,
}

#[derive(Debug, Clone)]
pub struct TeamStandingSnapshot {
    pub team_name: String,
    pub position: i64,
    pub points: i64,
    pub race_points: i64,
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
    pub driver_championship_axis: PositionAxis,
    pub constructor_championship_axis: PositionAxis,
    pub driver_race_axis: PositionAxis,
    pub constructor_race_axis: PositionAxis,
    pub round_labels: Vec<String>,
    pub driver_championship_series: Vec<ChartSeries>,
    pub constructor_championship_series: Vec<ChartSeries>,
    pub driver_race_series: Vec<ChartSeries>,
    pub constructor_race_series: Vec<ChartSeries>,
}

pub fn classify_race_results(
    rows: &[RaceResultInput],
) -> Vec<RaceResultSnapshot> {
    let mut finishers: Vec<&RaceResultInput> = rows
        .iter()
        .filter(|row| row.position.filter(|position| *position > 0).is_some())
        .collect();
    finishers.sort_by_key(|row| row.position.unwrap_or(i64::MAX));

    let mut others: Vec<&RaceResultInput> = rows
        .iter()
        .filter(|row| row.position.filter(|position| *position > 0).is_none())
        .collect();
    others.sort_by(|left, right| {
        right
            .number_of_laps
            .cmp(&left.number_of_laps)
            .then_with(|| left.driver_number.cmp(&right.driver_number))
    });

    let mut classified = Vec::with_capacity(rows.len());
    let mut max_finisher_position = 0i64;
    for row in &finishers {
        let position = row.position.unwrap_or(0);
        max_finisher_position = max_finisher_position.max(position);
        classified.push(RaceResultSnapshot {
            driver_number: row.driver_number,
            classified_position: position,
            dnf: row.dnf,
            dns: row.dns,
            dsq: row.dsq,
            points: row.points,
        });
    }

    let mut next_position = max_finisher_position + 1;
    for row in others {
        classified.push(RaceResultSnapshot {
            driver_number: row.driver_number,
            classified_position: next_position,
            dnf: row.dnf,
            dns: row.dns,
            dsq: row.dsq,
            points: row.points,
        });
        next_position += 1;
    }

    classified
}

#[derive(Debug, Clone, Copy)]
pub struct RaceResultInput {
    pub driver_number: i64,
    pub position: Option<i64>,
    pub number_of_laps: i64,
    pub dnf: bool,
    pub dns: bool,
    pub dsq: bool,
    pub points: i64,
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
    focus_drivers: Option<&[i64]>,
) -> ChampionshipCharts {
    let driver_numbers: Vec<i64> = match focus_drivers {
        Some(numbers) => numbers.to_vec(),
        None => pinned.iter().map(|pin| pin.driver_number).collect(),
    };

    let round_count = rounds.len() as u32;
    let round_labels = round_axis_labels(rounds, meetings, sessions);

    let driver_championship_series = driver_numbers
        .iter()
        .filter_map(|driver_number| {
            roster
                .iter()
                .find(|driver| driver.driver_number == *driver_number)
                .map(|driver| {
                    let points = rounds
                        .iter()
                        .filter_map(|round| {
                            round
                                .drivers
                                .iter()
                                .find(|entry| entry.driver_number == *driver_number)
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

    let driver_race_series = driver_numbers
        .iter()
        .filter_map(|driver_number| {
            roster
                .iter()
                .find(|driver| driver.driver_number == *driver_number)
                .map(|driver| {
                    let points = rounds
                        .iter()
                        .filter_map(|round| {
                            round
                                .race_results
                                .iter()
                                .find(|result| result.driver_number == *driver_number)
                                .map(|result| StandingPoint {
                                    round: round.round,
                                    position: result.classified_position,
                                    points: result.points,
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

    let (constructor_championship_series, constructor_race_series) =
        constructor_series(rounds, roster, &driver_numbers);

    ChampionshipCharts {
        round_count,
        driver_championship_axis: PositionAxis::from_series(&driver_championship_series),
        constructor_championship_axis: PositionAxis::from_series(&constructor_championship_series),
        driver_race_axis: PositionAxis::from_series(&driver_race_series),
        constructor_race_axis: PositionAxis::from_series(&constructor_race_series),
        round_labels,
        driver_championship_series,
        constructor_championship_series,
        driver_race_series,
        constructor_race_series,
    }
}

fn constructor_series(
    rounds: &[ChampionshipRoundSnapshot],
    roster: &[Driver],
    pinned_driver_numbers: &[i64],
) -> (Vec<ChartSeries>, Vec<ChartSeries>) {
    let Some(latest) = rounds.last() else {
        return (Vec::new(), Vec::new());
    };

    let pinned_teams: Vec<String> = pinned_driver_numbers
        .iter()
        .filter_map(|driver_number| {
            roster
                .iter()
                .find(|driver| driver.driver_number == *driver_number)
                .map(|driver| driver.team_name.clone())
        })
        .fold(Vec::new(), |mut teams, team| {
            if !teams.iter().any(|existing| existing == &team) {
                teams.push(team);
            }
            teams
        });

    if pinned_teams.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut teams = latest
        .teams
        .iter()
        .filter(|team| pinned_teams.iter().any(|name| name == &team.team_name))
        .cloned()
        .collect::<Vec<_>>();
    teams.sort_by_key(|team| team.position);

    let championship = teams
        .iter()
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
        .collect();

    let race = teams
        .iter()
        .map(|team| {
            let points = rounds
                .iter()
                .filter_map(|round| {
                    team_race_rank(round, &team.team_name).map(|position| {
                        let race_points = round
                            .teams
                            .iter()
                            .find(|entry| entry.team_name == team.team_name)
                            .map(|entry| entry.race_points)
                            .unwrap_or(0);
                        StandingPoint {
                            round: round.round,
                            position,
                            points: race_points,
                        }
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
        .collect();

    (championship, race)
}

fn team_race_rank(round: &ChampionshipRoundSnapshot, team_name: &str) -> Option<i64> {
    let mut ranked: Vec<_> = round
        .teams
        .iter()
        .filter(|team| team.race_points > 0)
        .collect();
    if ranked.is_empty() {
        ranked = round.teams.iter().collect();
    }
    ranked.sort_by(|left, right| {
        right
            .race_points
            .cmp(&left.race_points)
            .then_with(|| left.position.cmp(&right.position))
    });

    ranked
        .iter()
        .position(|team| team.team_name == team_name)
        .map(|index| index as i64 + 1)
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

    fn race_results_for_round(entries: &[(i64, i64, i64, bool)]) -> Vec<RaceResultSnapshot> {
        let inputs: Vec<RaceResultInput> = entries
            .iter()
            .map(|(driver_number, position, points, dnf)| RaceResultInput {
                driver_number: *driver_number,
                position: Some(*position),
                number_of_laps: if *dnf { 40 } else { 57 },
                dnf: *dnf,
                dns: false,
                dsq: false,
                points: *points,
            })
            .collect();
        classify_race_results(&inputs)
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
                        race_points: 18,
                    },
                    DriverStandingSnapshot {
                        driver_number: 44,
                        position: 1,
                        points: 25,
                        race_points: 25,
                    },
                ],
                teams: vec![
                    TeamStandingSnapshot {
                        team_name: "Red Bull Racing".into(),
                        position: 1,
                        points: 43,
                        race_points: 43,
                    },
                    TeamStandingSnapshot {
                        team_name: "Ferrari".into(),
                        position: 2,
                        points: 30,
                        race_points: 30,
                    },
                ],
                race_results: race_results_for_round(&[(1, 2, 18, false), (44, 1, 25, false)]),
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
                        race_points: 25,
                    },
                    DriverStandingSnapshot {
                        driver_number: 44,
                        position: 3,
                        points: 33,
                        race_points: 8,
                    },
                ],
                teams: vec![
                    TeamStandingSnapshot {
                        team_name: "Red Bull Racing".into(),
                        position: 1,
                        points: 76,
                        race_points: 33,
                    },
                    TeamStandingSnapshot {
                        team_name: "Ferrari".into(),
                        position: 2,
                        points: 55,
                        race_points: 20,
                    },
                ],
                race_results: race_results_for_round(&[(1, 1, 25, false), (44, 3, 8, false)]),
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

        let charts = build_championship_charts(&rounds, &pinned, &roster, &[], &[], None);
        assert_eq!(charts.round_count, 2);
        assert_eq!(charts.round_labels, vec!["1", "2"]);
        assert_eq!(
            charts.driver_championship_axis,
            PositionAxis { min: 1, max: 2 }
        );
        assert_eq!(charts.driver_championship_series.len(), 1);
        assert_eq!(charts.driver_championship_series[0].points.len(), 2);
        assert_eq!(charts.driver_race_series.len(), 1);
        assert_eq!(charts.driver_race_series[0].points.len(), 2);
        assert_eq!(charts.constructor_championship_series.len(), 1);
        assert_eq!(charts.constructor_championship_series[0].label, "Red Bull Racing");
        assert_eq!(charts.constructor_race_series.len(), 1);
    }

    #[test]
    fn race_standing_uses_finish_position_and_team_race_rank() {
        let rounds = vec![ChampionshipRoundSnapshot {
            round: 1,
            session_key: 10,
            meeting_key: 1,
            drivers: vec![DriverStandingSnapshot {
                driver_number: 1,
                position: 2,
                points: 18,
                race_points: 12,
            }],
            teams: vec![
                TeamStandingSnapshot {
                    team_name: "Red Bull Racing".into(),
                    position: 1,
                    points: 43,
                    race_points: 25,
                },
                TeamStandingSnapshot {
                    team_name: "Ferrari".into(),
                    position: 2,
                    points: 30,
                    race_points: 18,
                },
            ],
            race_results: race_results_for_round(&[(1, 4, 12, false)]),
        }];
        let pinned = vec![PinnedDriver {
            driver_number: 1,
            sort_order: 0,
        }];
        let roster = vec![sample_driver(1, "VER", "Red Bull Racing", "3671C6")];

        let charts = build_championship_charts(&rounds, &pinned, &roster, &[], &[], None);
        assert_eq!(charts.driver_race_series[0].points[0].position, 4);
        assert_eq!(charts.constructor_race_series.len(), 1);
        assert_eq!(charts.constructor_race_series[0].points[0].position, 1);
    }

    #[test]
    fn classify_race_results_assigns_positions_after_dnf() {
        let classified = classify_race_results(&[
            RaceResultInput {
                driver_number: 1,
                position: Some(1),
                number_of_laps: 57,
                dnf: false,
                dns: false,
                dsq: false,
                points: 25,
            },
            RaceResultInput {
                driver_number: 30,
                position: None,
                number_of_laps: 46,
                dnf: true,
                dns: false,
                dsq: false,
                points: 0,
            },
            RaceResultInput {
                driver_number: 5,
                position: None,
                number_of_laps: 45,
                dnf: true,
                dns: false,
                dsq: false,
                points: 0,
            },
        ]);

        let by_driver: std::collections::HashMap<_, _> = classified
            .iter()
            .map(|result| (result.driver_number, result.classified_position))
            .collect();
        assert_eq!(by_driver[&1], 1);
        assert_eq!(by_driver[&30], 2);
        assert_eq!(by_driver[&5], 3);
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
