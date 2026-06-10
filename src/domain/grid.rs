use chrono::{DateTime, Utc};
use openf1::{Session, StartingGrid};

#[derive(Debug, Clone, PartialEq)]
pub struct GridSlot {
    pub driver_number: i64,
    pub position: i64,
    pub gap_to_pole_secs: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualiGridVisibility {
    Hidden,
    Pending,
    Ready,
}

pub fn find_gp_qualifying<'a>(sessions: &'a [Session], meeting_key: i64) -> Option<&'a Session> {
    sessions.iter().find(|session| {
        session.meeting_key == meeting_key
            && !session.is_cancelled
            && session.session_name == "Qualifying"
            && session.session_type == "Qualifying"
    })
}

pub fn quali_has_ended(session: &Session, now: DateTime<Utc>) -> bool {
    parse_session_end(session) <= now
}

pub fn quali_grid_visibility(
    has_pins: bool,
    quali: Option<&Session>,
    now: DateTime<Utc>,
    grid_available: bool,
) -> QualiGridVisibility {
    if !has_pins {
        return QualiGridVisibility::Hidden;
    }

    let Some(quali) = quali else {
        return QualiGridVisibility::Hidden;
    };

    if !quali_has_ended(quali, now) {
        return QualiGridVisibility::Hidden;
    }

    if grid_available {
        QualiGridVisibility::Ready
    } else {
        QualiGridVisibility::Pending
    }
}

pub fn build_grid_slots(grid: &[StartingGrid], pinned_numbers: &[i64]) -> Vec<GridSlot> {
    if grid.is_empty() || pinned_numbers.is_empty() {
        return Vec::new();
    }

    let pole_lap = grid
        .iter()
        .find(|entry| entry.position == 1)
        .and_then(|entry| entry.lap_duration);

    let mut slots = Vec::new();
    for driver_number in pinned_numbers {
        let Some(entry) = grid.iter().find(|row| row.driver_number == *driver_number) else {
            continue;
        };

        let gap_to_pole_secs = match (pole_lap, entry.lap_duration) {
            (Some(pole), Some(lap)) if entry.position != 1 => Some((lap - pole).max(0.0)),
            _ => None,
        };

        slots.push(GridSlot {
            driver_number: *driver_number,
            position: entry.position,
            gap_to_pole_secs,
        });
    }

    slots.sort_by_key(|slot| slot.position);
    slots
}

pub fn format_grid_position(position: i64) -> String {
    if position == 1 {
        "Pole".into()
    } else {
        format!("P{position}")
    }
}

pub fn format_gap_to_pole(gap_secs: Option<f64>) -> Option<String> {
    gap_secs.map(|gap| format!("+{gap:.3}s"))
}

fn parse_session_end(session: &Session) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&session.date_end)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quali_session(meeting_key: i64, end: &str) -> Session {
        Session {
            circuit_key: 1,
            circuit_short_name: "Test".into(),
            country_code: "TST".into(),
            country_key: 1,
            country_name: "Testland".into(),
            date_end: end.into(),
            date_start: "2026-03-08T12:00:00Z".into(),
            gmt_offset: "00:00:00".into(),
            is_cancelled: false,
            location: "Test City".into(),
            meeting_key,
            session_key: 100,
            session_name: "Qualifying".into(),
            session_type: "Qualifying".into(),
            year: 2026,
        }
    }

    fn grid_row(driver_number: i64, position: i64, lap: Option<f64>) -> StartingGrid {
        StartingGrid {
            driver_number,
            lap_duration: lap,
            meeting_key: 1,
            position,
            session_key: 100,
        }
    }

    #[test]
    fn ignores_sprint_qualifying() {
        let sessions = vec![Session {
            session_name: "Sprint Qualifying".into(),
            session_type: "Qualifying".into(),
            meeting_key: 1,
            ..quali_session(1, "2026-03-08T14:00:00Z")
        }];
        assert!(find_gp_qualifying(&sessions, 1).is_none());
    }

    #[test]
    fn builds_gap_to_pole() {
        let grid = vec![
            grid_row(1, 1, Some(90.0)),
            grid_row(44, 2, Some(90.512)),
        ];
        let slots = build_grid_slots(&grid, &[44, 1]);
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].position, 1);
        let gap = slots[1].gap_to_pole_secs.unwrap();
        assert!((gap - 0.512).abs() < 0.001);
    }

    #[test]
    fn visibility_requires_pins_and_finished_quali() {
        let quali = quali_session(1, "2026-03-08T14:00:00Z");
        let now = DateTime::parse_from_rfc3339("2026-03-08T15:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(
            quali_grid_visibility(false, Some(&quali), now, true),
            QualiGridVisibility::Hidden
        );
        assert_eq!(
            quali_grid_visibility(true, Some(&quali), now, false),
            QualiGridVisibility::Pending
        );
        assert_eq!(
            quali_grid_visibility(true, Some(&quali), now, true),
            QualiGridVisibility::Ready
        );
    }
}
