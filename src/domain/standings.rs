use iced::Color;
use openf1::Driver;

use crate::domain::championship::{
    ChampionshipRoundSnapshot, ChampionshipTab, ChartMode, RaceResultSnapshot,
};
use crate::domain::{driver_display_name, team_colour};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionChange {
    Improved,
    Worsened,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct StandingRow {
    pub position: i64,
    pub position_label: String,
    pub label: String,
    pub code: String,
    pub accent: Color,
    pub grid_position: Option<i64>,
    pub position_change: Option<PositionChange>,
}

pub fn build_standings(
    rounds: &[ChampionshipRoundSnapshot],
    roster: &[Driver],
    tab: ChampionshipTab,
    mode: ChartMode,
) -> Vec<StandingRow> {
    let Some(latest) = rounds.last() else {
        return Vec::new();
    };

    let previous = rounds.len().checked_sub(2).map(|index| &rounds[index]);

    match (tab, mode) {
        (ChampionshipTab::Drivers, ChartMode::Championship) => {
            driver_championship_rows(&latest.drivers, previous, roster)
        }
        (ChampionshipTab::Drivers, ChartMode::RaceStanding) => {
            driver_race_rows(latest, roster)
        }
        (ChampionshipTab::Constructors, ChartMode::Championship) => {
            constructor_championship_rows(&latest.teams, previous, roster)
        }
        (ChampionshipTab::Constructors, ChartMode::RaceStanding) => {
            constructor_race_rows(latest, roster)
        }
    }
}

fn driver_championship_rows(
    standings: &[crate::domain::championship::DriverStandingSnapshot],
    previous: Option<&ChampionshipRoundSnapshot>,
    roster: &[Driver],
) -> Vec<StandingRow> {
    let mut rows: Vec<_> = standings
        .iter()
        .map(|entry| {
            let previous_position = previous.and_then(|round| {
                round
                    .drivers
                    .iter()
                    .find(|driver| driver.driver_number == entry.driver_number)
                    .map(|driver| driver.position)
            });
            let mut row = driver_row(entry.driver_number, entry.position, roster);
            row.position_change = position_change(previous_position, entry.position);
            row
        })
        .collect();
    rows.sort_by_key(|row| row.position);
    rows
}

fn driver_race_rows(
    round: &ChampionshipRoundSnapshot,
    roster: &[Driver],
) -> Vec<StandingRow> {
    let mut rows: Vec<_> = round
        .race_results
        .iter()
        .map(|result| {
            let position_label = race_result_position_label(result);
            let position = result.classified_position;
            let grid_position = grid_position_for_driver(round, result.driver_number);
            let mut row = driver_row(result.driver_number, position, roster);
            row.position_label = position_label;
            row.grid_position = grid_position;
            row.position_change = position_change(grid_position, position);
            row
        })
        .collect();
    rows.sort_by_key(|row| row.position);
    rows
}

fn driver_row(driver_number: i64, position: i64, roster: &[Driver]) -> StandingRow {
    let driver = roster
        .iter()
        .find(|entry| entry.driver_number == driver_number);
    let label = driver
        .map(|entry| entry.full_name.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("#{driver_number}"));
    let code = if let Some(driver) = driver {
        let acronym = driver.name_acronym.trim();
        if acronym.is_empty() {
            driver_display_name(driver).to_string()
        } else {
            acronym.to_ascii_uppercase()
        }
    } else {
        format!("#{driver_number}")
    };
    let accent = driver
        .map(|entry| team_colour(&entry.team_colour))
        .unwrap_or_else(|| team_colour("808080"));

    StandingRow {
        position,
        position_label: position.to_string(),
        label,
        code,
        accent,
        grid_position: None,
        position_change: None,
    }
}

fn constructor_championship_rows(
    teams: &[crate::domain::championship::TeamStandingSnapshot],
    previous: Option<&ChampionshipRoundSnapshot>,
    roster: &[Driver],
) -> Vec<StandingRow> {
    let mut rows: Vec<_> = teams
        .iter()
        .map(|team| {
            let previous_position = previous.and_then(|round| {
                round
                    .teams
                    .iter()
                    .find(|entry| entry.team_name == team.team_name)
                    .map(|entry| entry.position)
            });
            StandingRow {
                position: team.position,
                position_label: team.position.to_string(),
                label: team.team_name.clone(),
                code: constructor_code(&team.team_name),
                accent: constructor_colour(&team.team_name, roster),
                grid_position: None,
                position_change: position_change(previous_position, team.position),
            }
        })
        .collect();
    rows.sort_by_key(|row| row.position);
    rows
}

fn constructor_race_rows(
    round: &ChampionshipRoundSnapshot,
    roster: &[Driver],
) -> Vec<StandingRow> {
    let mut ranked: Vec<_> = round.teams.iter().collect();
    if ranked.iter().any(|team| team.race_points > 0) {
        ranked.retain(|team| team.race_points > 0);
    }
    ranked.sort_by(|left, right| {
        right
            .race_points
            .cmp(&left.race_points)
            .then_with(|| left.position.cmp(&right.position))
    });

    ranked
        .into_iter()
        .enumerate()
        .map(|(index, team)| StandingRow {
            position: index as i64 + 1,
            position_label: (index as i64 + 1).to_string(),
            label: team.team_name.clone(),
            code: constructor_code(&team.team_name),
            accent: constructor_colour(&team.team_name, roster),
            grid_position: None,
            position_change: None,
        })
        .collect()
}

fn grid_position_for_driver(round: &ChampionshipRoundSnapshot, driver_number: i64) -> Option<i64> {
    round
        .starting_grid
        .iter()
        .find(|slot| slot.driver_number == driver_number)
        .map(|slot| slot.position)
}

fn position_change(from: Option<i64>, to: i64) -> Option<PositionChange> {
    let from = from?;
    if from == to {
        Some(PositionChange::Unchanged)
    } else if to < from {
        Some(PositionChange::Improved)
    } else {
        Some(PositionChange::Worsened)
    }
}

fn race_result_position_label(result: &RaceResultSnapshot) -> String {
    if result.dsq {
        "DSQ".into()
    } else if result.dns {
        "DNS".into()
    } else if result.dnf {
        "DNF".into()
    } else {
        result.classified_position.to_string()
    }
}

fn constructor_code(team_name: &str) -> String {
    team_name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_uppercase()
}

fn constructor_colour(team_name: &str, roster: &[Driver]) -> Color {
    roster
        .iter()
        .find(|driver| driver.team_name == team_name)
        .map(|driver| team_colour(&driver.team_colour))
        .unwrap_or_else(|| team_colour("808080"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::championship::{
        DriverStandingSnapshot,
    };
    use crate::domain::grid::GridSlot;

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
    fn lists_all_drivers_by_championship_position() {
        let rounds = vec![ChampionshipRoundSnapshot {
            round: 1,
            session_key: 10,
            meeting_key: 1,
            drivers: vec![
                DriverStandingSnapshot {
                    driver_number: 44,
                    position: 1,
                    points: 25,
                    race_points: 25,
                },
                DriverStandingSnapshot {
                    driver_number: 1,
                    position: 2,
                    points: 18,
                    race_points: 18,
                },
            ],
            teams: vec![],
            race_results: vec![],
            starting_grid: vec![],
        }];
        let roster = vec![
            sample_driver(1, "VER", "Red Bull Racing", "3671C6"),
            sample_driver(44, "HAM", "Ferrari", "E80020"),
        ];

        let rows = build_standings(&rounds, &roster, ChampionshipTab::Drivers, ChartMode::Championship);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].position, 1);
        assert_eq!(rows[0].code, "HAM");
        assert_eq!(rows[1].position, 2);
    }

    #[test]
    fn championship_position_change_from_previous_round() {
        let rounds = vec![
            ChampionshipRoundSnapshot {
                round: 1,
                session_key: 10,
                meeting_key: 1,
                drivers: vec![DriverStandingSnapshot {
                    driver_number: 44,
                    position: 2,
                    points: 18,
                    race_points: 18,
                }],
                teams: vec![],
                race_results: vec![],
                starting_grid: vec![],
            },
            ChampionshipRoundSnapshot {
                round: 2,
                session_key: 20,
                meeting_key: 2,
                drivers: vec![DriverStandingSnapshot {
                    driver_number: 44,
                    position: 1,
                    points: 43,
                    race_points: 25,
                }],
                teams: vec![],
                race_results: vec![],
                starting_grid: vec![],
            },
        ];
        let roster = vec![sample_driver(44, "HAM", "Ferrari", "E80020")];

        let rows = build_standings(&rounds, &roster, ChampionshipTab::Drivers, ChartMode::Championship);
        assert_eq!(rows[0].position_change, Some(PositionChange::Improved));
    }

    #[test]
    fn race_position_change_from_starting_grid() {
        let rounds = vec![ChampionshipRoundSnapshot {
            round: 1,
            session_key: 10,
            meeting_key: 1,
            drivers: vec![],
            teams: vec![],
            starting_grid: vec![GridSlot {
                driver_number: 44,
                position: 5,
                gap_to_pole_secs: None,
            }],
            race_results: vec![RaceResultSnapshot {
                driver_number: 44,
                classified_position: 2,
                dnf: false,
                dns: false,
                dsq: false,
                points: 18,
            }],
        }];
        let roster = vec![sample_driver(44, "HAM", "Ferrari", "E80020")];

        let rows = build_standings(&rounds, &roster, ChampionshipTab::Drivers, ChartMode::RaceStanding);
        assert_eq!(rows[0].grid_position, Some(5));
        assert_eq!(rows[0].position_change, Some(PositionChange::Improved));
    }
}
