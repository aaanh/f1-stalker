use chrono::{DateTime, Utc};
use openf1::Meeting;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaceTripletSlot {
    Previous,
    Current,
    Upcoming,
}

#[derive(Debug, Clone)]
pub struct RaceTriplet {
    pub previous: Option<Meeting>,
    pub current: Meeting,
    pub upcoming: Meeting,
    pub current_is_weekend: bool,
}

pub fn compute_race_triplet(meetings: &[Meeting], now: DateTime<Utc>) -> Option<RaceTriplet> {
    let mut calendar: Vec<Meeting> = meetings
        .iter()
        .filter(|m| !m.is_cancelled)
        .cloned()
        .collect();

    calendar.sort_by(|a, b| parse_start(a).cmp(&parse_start(b)));

    if calendar.is_empty() {
        return None;
    }

    let upcoming_idx = calendar
        .iter()
        .position(|m| parse_end(m) > now)
        .unwrap_or(calendar.len() - 1);

    let upcoming = calendar[upcoming_idx].clone();
    let previous = upcoming_idx
        .checked_sub(1)
        .map(|idx| calendar[idx].clone());

    let start = parse_start(&upcoming);
    let end = parse_end(&upcoming);
    let current_is_weekend = start <= now && now <= end;

    Some(RaceTriplet {
        previous,
        current: upcoming.clone(),
        upcoming,
        current_is_weekend,
    })
}

pub fn parse_start(meeting: &Meeting) -> DateTime<Utc> {
    parse_dt(&meeting.date_start)
}

pub fn parse_end(meeting: &Meeting) -> DateTime<Utc> {
    parse_dt(&meeting.date_end)
}

fn parse_dt(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn meeting(key: i64, start: &str, end: &str) -> Meeting {
        Meeting {
            circuit_image: String::new(),
            circuit_info_url: String::new(),
            circuit_key: 1,
            circuit_short_name: "Test Circuit".into(),
            circuit_type: "Permanent".into(),
            country_code: "TST".into(),
            country_flag: String::new(),
            country_key: 1,
            country_name: "Testland".into(),
            date_end: end.into(),
            date_start: start.into(),
            gmt_offset: "00:00:00".into(),
            is_cancelled: false,
            location: "Test City".into(),
            meeting_key: key,
            meeting_name: format!("GP {key}"),
            meeting_official_name: format!("TEST GP {key}"),
            year: 2026,
        }
    }

    #[test]
    fn triplet_picks_previous_current_upcoming() {
        let meetings = vec![
            meeting(1, "2026-03-01T00:00:00Z", "2026-03-03T00:00:00Z"),
            meeting(2, "2026-03-08T00:00:00Z", "2026-03-10T00:00:00Z"),
            meeting(3, "2026-03-15T00:00:00Z", "2026-03-17T00:00:00Z"),
        ];
        let now = DateTime::parse_from_rfc3339("2026-03-09T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let triplet = compute_race_triplet(&meetings, now).unwrap();

        assert_eq!(triplet.previous.as_ref().map(|m| m.meeting_key), Some(1));
        assert_eq!(triplet.upcoming.meeting_key, 2);
        assert!(triplet.current_is_weekend);
    }

    #[test]
    fn triplet_excludes_cancelled() {
        let mut cancelled = meeting(1, "2026-03-01T00:00:00Z", "2026-03-03T00:00:00Z");
        cancelled.is_cancelled = true;
        let meetings = vec![
            cancelled,
            meeting(2, "2026-03-08T00:00:00Z", "2026-03-10T00:00:00Z"),
        ];
        let now = DateTime::parse_from_rfc3339("2026-03-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let triplet = compute_race_triplet(&meetings, now).unwrap();
        assert!(triplet.previous.is_none());
        assert_eq!(triplet.upcoming.meeting_key, 2);
    }
}
