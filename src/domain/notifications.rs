pub fn notification_already_sent(stored_signature: Option<&str>, signature: &str) -> bool {
    stored_signature.is_some_and(|stored| stored == signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_duplicate_standings_signature() {
        assert!(notification_already_sent(
            Some("ver:1:25|ham:2:18"),
            "ver:1:25|ham:2:18"
        ));
    }

    #[test]
    fn allows_new_standings_signature() {
        assert!(!notification_already_sent(
            Some("ver:1:25|ham:2:18"),
            "ver:1:25|ham:3:18"
        ));
    }

    #[test]
    fn allows_first_standings_notification() {
        assert!(!notification_already_sent(None, "ver:1:25"));
    }
}
