use chrono::{DateTime, Local};

pub fn format_clock(now: DateTime<Local>, use_24h: bool, show_seconds: bool) -> String {
    match (use_24h, show_seconds) {
        (true, true) => now.format("%H:%M:%S").to_string(),
        (true, false) => now.format("%H:%M").to_string(),
        (false, true) => now.format("%I:%M:%S %p").to_string(),
        (false, false) => now.format("%I:%M %p").to_string(),
    }
}
