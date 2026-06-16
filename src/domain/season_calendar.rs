use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use openf1::{Meeting, Session};

use crate::domain::calendar::{parse_end, parse_start, RaceTriplet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RacePhase {
    Past,
    Current,
    Future,
}

#[derive(Debug, Clone)]
pub struct DaySession {
    pub session_name: String,
    pub starts_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SeasonCalendarDay {
    pub date: NaiveDate,
    pub phase: Option<RacePhase>,
    pub meeting_name: Option<String>,
    pub round: Option<u32>,
    pub sessions: Vec<DaySession>,
}

#[derive(Debug, Clone)]
pub struct SeasonCalendarMonth {
    pub year: i32,
    pub month: u32,
    pub leading_blanks: usize,
    pub days: Vec<SeasonCalendarDay>,
}

#[derive(Debug, Clone)]
pub struct SeasonCalendar {
    pub months: Vec<SeasonCalendarMonth>,
}

pub fn build_season_calendar(
    meetings: &[Meeting],
    sessions: &[Session],
    triplet: &RaceTriplet,
    now: DateTime<Utc>,
) -> Option<SeasonCalendar> {
    let mut race_meetings: Vec<&Meeting> = meetings
        .iter()
        .filter(|meeting| !meeting.is_cancelled)
        .collect();
    race_meetings.sort_by_key(|meeting| parse_start(meeting));

    if race_meetings.is_empty() {
        return None;
    }

    let meeting_meta: HashMap<i64, (u32, RacePhase, String)> = race_meetings
        .iter()
        .enumerate()
        .map(|(index, meeting)| {
            let round = (index + 1) as u32;
            let phase = meeting_phase(meeting, triplet, now);
            (meeting.meeting_key, (round, phase, meeting.meeting_name.clone()))
        })
        .collect();

    let first_date = local_date(parse_start(race_meetings[0]));
    let last_meeting = race_meetings[race_meetings.len() - 1];
    let last_date = local_date(parse_end(last_meeting));

    let mut day_map: HashMap<NaiveDate, SeasonCalendarDay> = HashMap::new();

    for session in sessions.iter().filter(|session| !session.is_cancelled) {
        let Some((round, phase, meeting_name)) = meeting_meta.get(&session.meeting_key) else {
            continue;
        };

        let date = local_date(parse_session_start(session));
        let entry = day_map.entry(date).or_insert_with(|| SeasonCalendarDay {
            date,
            phase: None,
            meeting_name: None,
            round: None,
            sessions: Vec::new(),
        });

        entry.phase = Some(*phase);
        entry.meeting_name = Some(meeting_name.clone());
        entry.round = Some(*round);
        entry.sessions.push(DaySession {
            session_name: session.session_name.clone(),
            starts_at: parse_session_start(session),
        });
    }

    for day in day_map.values_mut() {
        day.sessions
            .sort_by_key(|session| session.starts_at);
    }

    let mut months = Vec::new();
    let mut cursor = NaiveDate::from_ymd_opt(first_date.year(), first_date.month(), 1)?;
    let end_month = NaiveDate::from_ymd_opt(last_date.year(), last_date.month(), 1)?;

    while cursor <= end_month {
        months.push(build_month(cursor.year(), cursor.month(), &day_map));
        cursor = next_month(cursor)?;
    }

    Some(SeasonCalendar { months })
}

fn build_month(
    year: i32,
    month: u32,
    day_map: &HashMap<NaiveDate, SeasonCalendarDay>,
) -> SeasonCalendarMonth {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month");
    let leading_blanks = first.weekday().num_days_from_monday() as usize;
    let days_in_month = days_in_month(year, month);

    let days = (1..=days_in_month)
        .map(|day| {
            let date = NaiveDate::from_ymd_opt(year, month, day).expect("valid day");
            day_map.get(&date).cloned().unwrap_or_else(|| SeasonCalendarDay {
                date,
                phase: None,
                meeting_name: None,
                round: None,
                sessions: Vec::new(),
            })
        })
        .collect();

    SeasonCalendarMonth {
        year,
        month,
        leading_blanks,
        days,
    }
}

fn meeting_phase(meeting: &Meeting, triplet: &RaceTriplet, now: DateTime<Utc>) -> RacePhase {
    if parse_end(meeting) < now {
        RacePhase::Past
    } else if meeting.meeting_key == triplet.current.meeting_key {
        RacePhase::Current
    } else {
        RacePhase::Future
    }
}

fn local_date(dt: DateTime<Utc>) -> NaiveDate {
    dt.with_timezone(&Local).date_naive()
}

fn parse_session_start(session: &Session) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&session.date_start)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1))
        .and_then(|next| next.pred_opt())
        .map(|last| last.day())
        .unwrap_or(28)
}

fn next_month(date: NaiveDate) -> Option<NaiveDate> {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compute_race_triplet;
    use pretty_assertions::assert_eq;

    fn meeting(key: i64, name: &str, start: &str, end: &str) -> Meeting {
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
            date_end: end.into(),
            date_start: start.into(),
            gmt_offset: "00:00:00".into(),
            is_cancelled: false,
            location: "Test".into(),
            meeting_key: key,
            meeting_name: name.into(),
            meeting_official_name: format!("{name} GP"),
            year: 2026,
        }
    }

    fn session(meeting_key: i64, name: &str, start: &str) -> Session {
        Session {
            circuit_key: 1,
            circuit_short_name: "Test".into(),
            country_code: "TST".into(),
            country_key: 1,
            country_name: "Testland".into(),
            date_end: start.into(),
            date_start: start.into(),
            gmt_offset: "00:00:00".into(),
            is_cancelled: false,
            location: "Test".into(),
            meeting_key,
            session_key: start.len() as i64,
            session_name: name.into(),
            session_type: name.into(),
            year: 2026,
        }
    }

    fn day_by_date(calendar: &SeasonCalendar, date: NaiveDate) -> Option<&SeasonCalendarDay> {
        calendar
            .months
            .iter()
            .flat_map(|month| month.days.iter())
            .find(|day| day.date == date)
    }

    #[test]
    fn spans_first_to_last_race_months() {
        let meetings = vec![
            meeting(1, "Bahrain", "2026-03-01T12:00:00Z", "2026-03-03T12:00:00Z"),
            meeting(2, "Monaco", "2026-06-01T12:00:00Z", "2026-06-03T12:00:00Z"),
        ];
        let sessions = vec![
            session(1, "Race", "2026-03-02T12:00:00Z"),
            session(2, "Race", "2026-06-02T12:00:00Z"),
        ];
        let now = DateTime::parse_from_rfc3339("2026-04-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let triplet = compute_race_triplet(&meetings, now).unwrap();

        let calendar = build_season_calendar(&meetings, &sessions, &triplet, now).unwrap();
        assert!(calendar.months.len() >= 4);
        assert_eq!(calendar.months.first().map(|m| m.month), Some(3));
        assert_eq!(calendar.months.last().map(|m| m.month), Some(6));
    }

    #[test]
    fn groups_sessions_on_same_day() {
        let meetings = vec![meeting(
            1,
            "Bahrain",
            "2026-03-01T00:00:00Z",
            "2026-03-03T00:00:00Z",
        )];
        let sessions = vec![
            session(1, "Practice 1", "2026-03-01T10:00:00Z"),
            session(1, "Practice 2", "2026-03-01T14:00:00Z"),
            session(1, "Qualifying", "2026-03-02T14:00:00Z"),
        ];
        let now = DateTime::parse_from_rfc3339("2026-02-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let triplet = compute_race_triplet(&meetings, now).unwrap();
        let calendar = build_season_calendar(&meetings, &sessions, &triplet, now).unwrap();

        let practice_day = day_by_date(
            &calendar,
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        )
        .expect("practice day");
        assert_eq!(practice_day.sessions.len(), 2);
        assert_eq!(practice_day.round, Some(1));
        assert_eq!(practice_day.phase, Some(RacePhase::Current));
    }

    #[test]
    fn assigns_past_current_and_future_phases() {
        let meetings = vec![
            meeting(1, "Bahrain", "2026-03-01T12:00:00Z", "2026-03-03T12:00:00Z"),
            meeting(2, "Monaco", "2026-06-01T12:00:00Z", "2026-06-03T12:00:00Z"),
            meeting(3, "Abu Dhabi", "2026-12-01T12:00:00Z", "2026-12-03T12:00:00Z"),
        ];
        let sessions = vec![
            session(1, "Race", "2026-03-02T12:00:00Z"),
            session(2, "Race", "2026-06-02T12:00:00Z"),
            session(3, "Race", "2026-12-02T12:00:00Z"),
        ];
        let now = DateTime::parse_from_rfc3339("2026-04-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let triplet = compute_race_triplet(&meetings, now).unwrap();
        let calendar = build_season_calendar(&meetings, &sessions, &triplet, now).unwrap();

        let past = day_by_date(
            &calendar,
            NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
        )
        .unwrap();
        let current = day_by_date(
            &calendar,
            NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
        )
        .unwrap();
        let future = day_by_date(
            &calendar,
            NaiveDate::from_ymd_opt(2026, 12, 2).unwrap(),
        )
        .unwrap();

        assert_eq!(past.phase, Some(RacePhase::Past));
        assert_eq!(current.phase, Some(RacePhase::Current));
        assert_eq!(future.phase, Some(RacePhase::Future));
    }
}
