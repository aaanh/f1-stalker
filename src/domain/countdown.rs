use chrono::{DateTime, Duration, Utc};
use openf1::{Meeting, Session};

#[derive(Debug, Clone)]
pub struct SessionSchedule {
    pub meeting_key: i64,
    pub sessions: Vec<Session>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountdownTarget {
    NextSession {
        session_name: String,
        starts_at: DateTime<Utc>,
    },
    SchedulePending,
    SeasonComplete,
}

pub fn sessions_for_meeting(sessions: &[Session], meeting_key: i64) -> SessionSchedule {
    let mut list: Vec<Session> = sessions
        .iter()
        .filter(|s| s.meeting_key == meeting_key && !s.is_cancelled)
        .cloned()
        .collect();
    list.sort_by(|a, b| parse_start(a).cmp(&parse_start(b)));
    SessionSchedule {
        meeting_key,
        sessions: list,
    }
}

pub fn next_countdown(schedule: &SessionSchedule, now: DateTime<Utc>) -> CountdownTarget {
    next_season_countdown(&[], &schedule.sessions, now)
}

/// Next session across the full season calendar (advances to the next meeting when the
/// current one has no future sessions left).
pub fn next_season_countdown(
    meetings: &[Meeting],
    sessions: &[Session],
    now: DateTime<Utc>,
) -> CountdownTarget {
    let mut future: Vec<&Session> = sessions
        .iter()
        .filter(|session| !session.is_cancelled && parse_start(session) > now)
        .collect();
    future.sort_by_key(|session| parse_start(session));

    if let Some(session) = future.first() {
        return CountdownTarget::NextSession {
            session_name: session.session_name.clone(),
            starts_at: parse_start(session),
        };
    }

    let meeting_active = meetings.iter().any(|meeting| {
        !meeting.is_cancelled && parse_meeting_start(meeting) <= now && now <= parse_meeting_end(meeting)
    });

    if meeting_active {
        CountdownTarget::SchedulePending
    } else {
        CountdownTarget::SeasonComplete
    }
}

fn parse_meeting_start(meeting: &Meeting) -> DateTime<Utc> {
    parse_dt(&meeting.date_start)
}

fn parse_meeting_end(meeting: &Meeting) -> DateTime<Utc> {
    parse_dt(&meeting.date_end)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountdownSegment {
    pub value: String,
    pub label: &'static str,
}

pub fn format_countdown(now: DateTime<Utc>, target: DateTime<Utc>) -> String {
    format_countdown_precise(now, target)
}

/// F1-style countdown with millisecond precision (`DD:HH:MM:SS.mmm` or `HH:MM:SS.mmm`).
pub fn format_countdown_precise(now: DateTime<Utc>, target: DateTime<Utc>) -> String {
    format_countdown_segments(&countdown_segments(now, target))
}

pub fn countdown_segments(now: DateTime<Utc>, target: DateTime<Utc>) -> Vec<CountdownSegment> {
    let remaining = target.signed_duration_since(now);
    if remaining <= Duration::zero() {
        return countdown_segments_zero();
    }

    build_countdown_segments(decompose_remaining(remaining))
}

pub fn countdown_segments_zero() -> Vec<CountdownSegment> {
    build_countdown_segments((0, 0, 0, 0, 0))
}

pub fn countdown_segments_pending() -> Vec<CountdownSegment> {
    vec![
        CountdownSegment {
            value: "--".into(),
            label: "HRS",
        },
        CountdownSegment {
            value: "--".into(),
            label: "MIN",
        },
        CountdownSegment {
            value: "--".into(),
            label: "SEC",
        },
        CountdownSegment {
            value: "---".into(),
            label: "MS",
        },
    ]
}

fn build_countdown_segments((days, hours, mins, secs, millis): (i64, i64, i64, i64, i64)) -> Vec<CountdownSegment> {
    let mut segments = Vec::with_capacity(5);
    if days > 0 {
        segments.push(CountdownSegment {
            value: format!("{days:02}"),
            label: "DAYS",
        });
    }
    segments.push(CountdownSegment {
        value: format!("{hours:02}"),
        label: "HRS",
    });
    segments.push(CountdownSegment {
        value: format!("{mins:02}"),
        label: "MIN",
    });
    segments.push(CountdownSegment {
        value: format!("{secs:02}"),
        label: "SEC",
    });
    segments.push(CountdownSegment {
        value: format!("{millis:03}"),
        label: "MS",
    });
    segments
}

fn decompose_remaining(remaining: Duration) -> (i64, i64, i64, i64, i64) {
    let total_ms = remaining.num_milliseconds();
    let millis = total_ms.rem_euclid(1000);
    let total_secs = total_ms / 1000;
    let secs = total_secs.rem_euclid(60);
    let mins = (total_secs / 60).rem_euclid(60);
    let hours = (total_secs / 3600).rem_euclid(24);
    let days = total_secs / 86400;
    (days, hours, mins, secs, millis)
}

fn format_countdown_segments(segments: &[CountdownSegment]) -> String {
    if segments.len() < 2 {
        return segments
            .first()
            .map(|segment| segment.value.clone())
            .unwrap_or_else(|| "00:00:00.000".into());
    }

    let (time, millis) = segments.split_at(segments.len() - 1);
    format!(
        "{}.{}",
        time.iter()
            .map(|segment| segment.value.as_str())
            .collect::<Vec<_>>()
            .join(":"),
        millis[0].value
    )
}

fn parse_start(session: &Session) -> DateTime<Utc> {
    parse_dt(&session.date_start)
}

fn parse_end(session: &Session) -> DateTime<Utc> {
    parse_dt(&session.date_end)
}

fn parse_dt(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use openf1::Session;
    use pretty_assertions::assert_eq;

    fn session(name: &str, start: &str, end: &str) -> Session {
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
            location: "Test".into(),
            meeting_key: 99,
            session_key: 1,
            session_name: name.into(),
            session_type: name.into(),
            year: 2026,
        }
    }

    #[test]
    fn picks_next_session() {
        let schedule = SessionSchedule {
            meeting_key: 99,
            sessions: vec![
                session(
                    "Practice 1",
                    "2026-03-08T10:00:00Z",
                    "2026-03-08T11:00:00Z",
                ),
                session(
                    "Qualifying",
                    "2026-03-09T14:00:00Z",
                    "2026-03-09T15:00:00Z",
                ),
            ],
        };
        let now = DateTime::parse_from_rfc3339("2026-03-08T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let target = next_countdown(&schedule, now);
        assert_eq!(
            target,
            CountdownTarget::NextSession {
                session_name: "Qualifying".into(),
                starts_at: DateTime::parse_from_rfc3339("2026-03-09T14:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            }
        );
    }

    #[test]
    fn advances_to_next_meeting_after_last_session() {
        let meetings = vec![meeting(
            1,
            "2026-03-08T00:00:00Z",
            "2026-03-10T00:00:00Z",
        )];
        let sessions = vec![
            session(
                "Race",
                "2026-03-09T14:00:00Z",
                "2026-03-09T16:00:00Z",
            ),
            session(
                "Practice 1",
                "2026-03-15T10:00:00Z",
                "2026-03-15T11:00:00Z",
            ),
        ];
        let now = DateTime::parse_from_rfc3339("2026-03-09T17:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let target = next_season_countdown(&meetings, &sessions, now);
        assert_eq!(
            target,
            CountdownTarget::NextSession {
                session_name: "Practice 1".into(),
                starts_at: DateTime::parse_from_rfc3339("2026-03-15T10:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            }
        );
    }

    #[test]
    fn countdown_segments_include_days_when_needed() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let target = now + Duration::days(2) + Duration::hours(3) + Duration::minutes(4) + Duration::seconds(5) + Duration::milliseconds(6);

        let segments = countdown_segments(now, target);
        assert_eq!(
            segments,
            vec![
                CountdownSegment {
                    value: "02".into(),
                    label: "DAYS",
                },
                CountdownSegment {
                    value: "03".into(),
                    label: "HRS",
                },
                CountdownSegment {
                    value: "04".into(),
                    label: "MIN",
                },
                CountdownSegment {
                    value: "05".into(),
                    label: "SEC",
                },
                CountdownSegment {
                    value: "006".into(),
                    label: "MS",
                },
            ]
        );
        assert_eq!(format_countdown_precise(now, target), "02:03:04:05.006");
    }

    #[test]
    fn countdown_segments_omit_days_under_one_day() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let target = now + Duration::hours(1) + Duration::minutes(2) + Duration::seconds(3) + Duration::milliseconds(4);

        let segments = countdown_segments(now, target);
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].label, "HRS");
        assert_eq!(format_countdown_precise(now, target), "01:02:03.004");
    }

    #[test]
    fn schedule_pending_during_active_meeting_without_future_sessions() {
        let meetings = vec![meeting(
            1,
            "2026-03-08T00:00:00Z",
            "2026-03-10T00:00:00Z",
        )];
        let sessions = vec![session(
            "Race",
            "2026-03-09T14:00:00Z",
            "2026-03-09T16:00:00Z",
        )];
        let now = DateTime::parse_from_rfc3339("2026-03-09T17:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let target = next_season_countdown(&meetings, &sessions, now);
        assert_eq!(target, CountdownTarget::SchedulePending);
    }

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
}
