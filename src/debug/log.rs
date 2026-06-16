use std::sync::{Mutex, OnceLock};

use chrono::Local;

const MAX_LINES: usize = 500;

static LOG: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn buffer() -> &'static Mutex<Vec<String>> {
    LOG.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn info(message: impl AsRef<str>) {
    push("INFO", message);
}

pub fn warn(message: impl AsRef<str>) {
    push("WARN", message);
}

pub fn error(message: impl AsRef<str>) {
    push("ERROR", message);
}

#[allow(dead_code)]
pub fn debug(message: impl AsRef<str>) {
    push("DEBUG", message);
}

pub fn entries() -> Vec<String> {
    buffer()
        .lock()
        .map(|lines| lines.clone())
        .unwrap_or_default()
}

pub fn entries_text() -> String {
    entries().join("\n")
}

fn push(level: &str, message: impl AsRef<str>) {
    let line = format!(
        "[{}] {level}: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        message.as_ref()
    );
    if let Ok(mut lines) = buffer().lock() {
        lines.push(line);
        if lines.len() > MAX_LINES {
            let drain = lines.len() - MAX_LINES;
            lines.drain(0..drain);
        }
    }
}
