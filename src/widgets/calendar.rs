use chrono::{DateTime, Datelike, Local};

pub fn format_calendar(now: DateTime<Local>, month_view: bool) -> String {
    if month_view {
        format!("{} {}", now.format("%B"), now.year())
    } else {
        now.format("%a %b %d, %Y").to_string()
    }
}
