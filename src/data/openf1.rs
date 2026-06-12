use chrono::{DateTime, Utc};
use openf1::{Meeting, Session, OpenF1Client, OpenF1Error, MeetingsListParams, SessionsListParams};
use thiserror::Error;

use crate::domain::{
    calendar_filter::is_testing_meeting, compute_race_triplet, next_season_countdown,
    CountdownTarget, RaceTriplet,
};

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("openf1: {0}")]
    Api(#[from] OpenF1Error),
    #[error("no meetings for season {0}")]
    EmptySeason(i32),
}

#[derive(Debug, Clone)]
pub struct CalendarData {
    pub season: i32,
    pub meetings: Vec<Meeting>,
    pub sessions: Vec<Session>,
    pub triplet: RaceTriplet,
    pub countdown: CountdownTarget,
    pub fetched_at: DateTime<Utc>,
}

impl CalendarData {
    pub fn apply_preferences(&mut self, include_testing: bool) {
        if !include_testing {
            self.meetings.retain(|meeting| !is_testing_meeting(meeting));
            let allowed: std::collections::HashSet<i64> = self
                .meetings
                .iter()
                .map(|meeting| meeting.meeting_key)
                .collect();
            self.sessions
                .retain(|session| allowed.contains(&session.meeting_key));
        }

        let now = Utc::now();
        if let Some(triplet) = compute_race_triplet(&self.meetings, now) {
            self.triplet = triplet;
        }
        self.countdown = next_season_countdown(&self.meetings, &self.sessions, now);
    }
}

pub async fn fetch_season_calendar(season: i32) -> Result<CalendarData, FetchError> {
    let client = OpenF1Client::new(None);
    let now = Utc::now();

    let meetings = client
        .meetings()
        .list(MeetingsListParams {
            year: Some(i64::from(season)),
            country_name: None,
        })
        .await?;

    let triplet = compute_race_triplet(&meetings, now).ok_or(FetchError::EmptySeason(season))?;

    let sessions = client
        .sessions()
        .list(SessionsListParams {
            year: Some(i64::from(season)),
            country_name: None,
            session_name: None,
        })
        .await?;

    let countdown = next_season_countdown(&meetings, &sessions, now);

    Ok(CalendarData {
        season,
        meetings,
        sessions,
        triplet,
        countdown,
        fetched_at: now,
    })
}
