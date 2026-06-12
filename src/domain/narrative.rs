use crate::domain::championship::{ChampionshipRoundSnapshot, DriverStandingSnapshot};
use crate::domain::driver_display_name;
use openf1::Driver;

#[derive(Debug, Clone)]
pub struct ChampionshipNarrative {
    pub headline: String,
    pub detail: String,
}

pub fn build_championship_narrative(
    rounds: &[ChampionshipRoundSnapshot],
    roster: &[Driver],
) -> Option<ChampionshipNarrative> {
    let latest = rounds.last()?;
    if latest.drivers.is_empty() {
        return None;
    }

    let mut standings = latest.drivers.clone();
    standings.sort_by_key(|entry| entry.position);

    let leader = standings.first()?;
    let leader_name = driver_name(roster, leader.driver_number);

    if standings.len() == 1 {
        return Some(ChampionshipNarrative {
            headline: format!("{leader_name} leads the championship"),
            detail: format!("{} points after {} rounds", leader.points, rounds.len()),
        });
    }

    let runner_up = &standings[1];
    let gap = leader.points.saturating_sub(runner_up.points);

    Some(ChampionshipNarrative {
        headline: format!("{leader_name} leads the championship"),
        detail: format!(
            "P{} with {} points · {} ahead of {}",
            leader.position,
            leader.points,
            gap,
            driver_name(roster, runner_up.driver_number)
        ),
    })
}

pub fn build_rival_narrative(
    rounds: &[ChampionshipRoundSnapshot],
    roster: &[Driver],
    first: i64,
    second: i64,
) -> Option<ChampionshipNarrative> {
    let latest = rounds.last()?;
    let first_entry = latest
        .drivers
        .iter()
        .find(|entry| entry.driver_number == first)?;
    let second_entry = latest
        .drivers
        .iter()
        .find(|entry| entry.driver_number == second)?;

    let first_name = driver_name(roster, first);
    let second_name = driver_name(roster, second);
    let gap = first_entry
        .points
        .abs_diff(second_entry.points);

    let (leader_name, leader_pts, trailing_name, trailing_pts) =
        if first_entry.points >= second_entry.points {
            (
                first_name,
                first_entry.points,
                second_name,
                second_entry.points,
            )
        } else {
            (
                second_name,
                second_entry.points,
                first_name,
                first_entry.points,
            )
        };

    Some(ChampionshipNarrative {
        headline: format!("{leader_name} vs {trailing_name}"),
        detail: format!(
            "{} leads {}–{} · {} point{} gap after {} rounds",
            leader_name,
            leader_pts,
            trailing_pts,
            gap,
            if gap == 1 { "" } else { "s" },
            rounds.len()
        ),
    })
}

pub fn build_season_complete_narrative(
    rounds: &[ChampionshipRoundSnapshot],
    roster: &[Driver],
) -> Option<ChampionshipNarrative> {
    let latest = rounds.last()?;
    let mut standings = latest.drivers.clone();
    standings.sort_by_key(|entry| entry.position);
    let champion = standings.first()?;

    Some(ChampionshipNarrative {
        headline: format!(
            "World Champion: {}",
            driver_name(roster, champion.driver_number)
        ),
        detail: format!("{} points · P{}", champion.points, champion.position),
    })
}

fn driver_name(roster: &[Driver], driver_number: i64) -> String {
    roster
        .iter()
        .find(|driver| driver.driver_number == driver_number)
        .map(|driver| driver_display_name(driver).to_string())
        .unwrap_or_else(|| format!("#{driver_number}"))
        .to_string()
}

pub fn standings_signature(standings: &[DriverStandingSnapshot]) -> String {
    let mut rows = standings.to_vec();
    rows.sort_by_key(|entry| entry.driver_number);
    rows.iter()
        .map(|entry| format!("{}:{}:{}", entry.driver_number, entry.position, entry.points))
        .collect::<Vec<_>>()
        .join("|")
}
