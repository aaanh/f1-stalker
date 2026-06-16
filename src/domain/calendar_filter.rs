use chrono::{DateTime, Utc};
use openf1::Meeting;

pub fn is_testing_meeting(meeting: &Meeting) -> bool {
    let name = meeting.meeting_name.to_ascii_lowercase();
    let official = meeting.meeting_official_name.to_ascii_lowercase();
    name.contains("test") || official.contains("test")
}

pub fn next_session_reminder_at(
    sessions: &[openf1::Session],
    now: DateTime<Utc>,
    lead_minutes: i64,
) -> Option<(DateTime<Utc>, String)> {
    sessions
        .iter()
        .filter(|session| !session.is_cancelled)
        .filter_map(|session| {
            DateTime::parse_from_rfc3339(&session.date_start)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .filter(|start| *start > now)
                .map(|start| (start, session.session_name.clone()))
        })
        .min_by_key(|(start, _)| *start)
        .map(|(start, name)| (start - chrono::Duration::minutes(lead_minutes), name))
}
