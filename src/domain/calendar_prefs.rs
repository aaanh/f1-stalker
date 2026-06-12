use openf1::Meeting;

use crate::domain::calendar_filter::is_testing_meeting;
use crate::domain::{compute_race_triplet, next_season_countdown, CountdownTarget, RaceTriplet};
use chrono::{DateTime, Utc};
use openf1::Session;

pub fn apply_calendar_preferences(
    meetings: &mut Vec<Meeting>,
    sessions: &mut Vec<Session>,
    include_testing: bool,
    now: DateTime<Utc>,
) -> Option<(RaceTriplet, CountdownTarget)> {
    if !include_testing {
        meetings.retain(|meeting| !is_testing_meeting(meeting));
        let allowed: std::collections::HashSet<i64> =
            meetings.iter().map(|meeting| meeting.meeting_key).collect();
        sessions.retain(|session| allowed.contains(&session.meeting_key));
    }

    let triplet = compute_race_triplet(meetings, now)?;
    let countdown = next_season_countdown(meetings, sessions, now);
    Some((triplet, countdown))
}
