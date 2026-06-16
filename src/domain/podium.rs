use iced::Color;
use openf1::Driver;

use crate::domain::championship::{ChampionshipRoundSnapshot, RaceResultSnapshot};
use crate::domain::{driver_display_name, team_colour};

#[derive(Debug, Clone)]
pub struct PodiumEntry {
    pub position: i64,
    pub driver_number: i64,
    pub code: String,
    pub accent: Color,
}

pub fn podium_for_meeting(
    rounds: &[ChampionshipRoundSnapshot],
    meeting_key: i64,
    roster: &[Driver],
) -> Vec<PodiumEntry> {
    let Some(round) = rounds
        .iter()
        .find(|entry| entry.meeting_key == meeting_key)
    else {
        return Vec::new();
    };

    let mut podium: Vec<&RaceResultSnapshot> = round
        .race_results
        .iter()
        .filter(|result| {
            !result.dnf
                && !result.dns
                && !result.dsq
                && result.classified_position >= 1
                && result.classified_position <= 3
        })
        .collect();

    if podium.is_empty() {
        return Vec::new();
    }

    podium.sort_by_key(|result| result.classified_position);

    podium
        .into_iter()
        .map(|result| podium_entry(result, roster))
        .collect()
}

fn podium_entry(result: &RaceResultSnapshot, roster: &[Driver]) -> PodiumEntry {
    let driver = roster
        .iter()
        .find(|entry| entry.driver_number == result.driver_number);
    let code = if let Some(driver) = driver {
        let acronym = driver.name_acronym.trim();
        if acronym.is_empty() {
            driver_display_name(driver).to_string()
        } else {
            acronym.to_ascii_uppercase()
        }
    } else {
        format!("#{}", result.driver_number)
    };
    let accent = driver
        .map(|entry| team_colour(&entry.team_colour))
        .unwrap_or_else(|| team_colour("808080"));

    PodiumEntry {
        position: result.classified_position,
        driver_number: result.driver_number,
        code,
        accent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::championship::{
        ChampionshipRoundSnapshot, RaceResultSnapshot,
    };

    fn sample_driver(number: i64, acronym: &str, colour: &str) -> Driver {
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
            team_name: "Team".into(),
        }
    }

    #[test]
    fn returns_top_three_finishers() {
        let rounds = vec![ChampionshipRoundSnapshot {
            round: 1,
            session_key: 10,
            meeting_key: 42,
            drivers: vec![],
            teams: vec![],
            starting_grid: vec![],
            race_results: vec![
                RaceResultSnapshot {
                    driver_number: 1,
                    classified_position: 2,
                    dnf: false,
                    dns: false,
                    dsq: false,
                    points: 18,
                },
                RaceResultSnapshot {
                    driver_number: 44,
                    classified_position: 1,
                    dnf: false,
                    dns: false,
                    dsq: false,
                    points: 25,
                },
                RaceResultSnapshot {
                    driver_number: 16,
                    classified_position: 3,
                    dnf: false,
                    dns: false,
                    dsq: false,
                    points: 15,
                },
            ],
        }];
        let roster = vec![
            sample_driver(1, "VER", "3671C6"),
            sample_driver(44, "HAM", "E80020"),
            sample_driver(16, "LEC", "ED1131"),
        ];

        let podium = podium_for_meeting(&rounds, 42, &roster);
        assert_eq!(podium.len(), 3);
        assert_eq!(podium[0].position, 1);
        assert_eq!(podium[0].code, "HAM");
        assert_eq!(podium[1].code, "VER");
        assert_eq!(podium[2].code, "LEC");
    }
}
