use chrono::{DateTime, Local, Utc};

pub fn format_meeting_date_range(start: &str, end: &str) -> String {
    let start = parse_local(start);
    let end = parse_local(end);
    format!(
        "{} – {}",
        start.format("%d %b"),
        end.format("%d %b %Y")
    )
}

pub fn format_session_start(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%a %d %b · %H:%M")
        .to_string()
}

pub fn format_fetched_at(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%H:%M %Z").to_string()
}

fn parse_local(value: &str) -> DateTime<Local> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Local))
        .unwrap_or_else(|_| Local::now())
}
