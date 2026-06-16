use crate::domain::championship::ChampionshipRoundSnapshot;

pub fn average_quali_position(
    rounds: &[ChampionshipRoundSnapshot],
    driver_number: i64,
) -> Option<f64> {
    let positions: Vec<f64> = rounds
        .iter()
        .filter_map(|round| {
            round
                .starting_grid
                .iter()
                .find(|slot| slot.driver_number == driver_number)
                .map(|slot| slot.position as f64)
        })
        .collect();

    if positions.is_empty() {
        None
    } else {
        Some(positions.iter().sum::<f64>() / positions.len() as f64)
    }
}

pub fn format_average_quali(position: f64) -> String {
    let rounded = (position * 10.0).round() / 10.0;
    if (rounded.fract()).abs() < f64::EPSILON {
        format!("P{} avg quali", rounded as i64)
    } else {
        format!("P{rounded:.1} avg quali")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::championship::ChampionshipRoundSnapshot;
    use crate::domain::grid::GridSlot;

    #[test]
    fn averages_grid_positions_across_rounds() {
        let rounds = vec![
            ChampionshipRoundSnapshot {
                round: 1,
                session_key: 1,
                meeting_key: 1,
                drivers: vec![],
                teams: vec![],
                starting_grid: vec![GridSlot {
                    driver_number: 44,
                    position: 2,
                    gap_to_pole_secs: None,
                }],
                race_results: vec![],
            },
            ChampionshipRoundSnapshot {
                round: 2,
                session_key: 2,
                meeting_key: 2,
                drivers: vec![],
                teams: vec![],
                starting_grid: vec![GridSlot {
                    driver_number: 44,
                    position: 4,
                    gap_to_pole_secs: None,
                }],
                race_results: vec![],
            },
        ];

        assert_eq!(average_quali_position(&rounds, 44), Some(3.0));
        assert_eq!(format_average_quali(3.0), "P3 avg quali");
        assert_eq!(format_average_quali(3.25), "P3.3 avg quali");
    }
}
