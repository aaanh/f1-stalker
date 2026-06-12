use notify_rust::Notification;

pub fn notify(title: &str, body: &str) {
    let _ = Notification::new()
        .summary(title)
        .body(body)
        .appname("F1 Stalker")
        .show();
}

pub fn notify_standings_change(driver_name: &str, position: i64, points: i64) {
    notify(
        "Championship update",
        &format!("{driver_name} is now P{position} with {points} points"),
    );
}

pub fn notify_session_reminder(session_name: &str, starts_in: &str) {
    notify(
        "Upcoming session",
        &format!("{session_name} starts in {starts_in}"),
    );
}
