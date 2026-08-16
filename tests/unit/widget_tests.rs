#![allow(clippy::field_reassign_with_default)]

use chrono::{Local, Timelike};
use glam::Vec2;
use oled_wallpaper::config::OverlayConfig;
use oled_wallpaper::widgets::{clock, WidgetSystem};

#[test]
fn clock_widget_time_accuracy_within_one_second() {
    let now = Local::now();
    let text = clock::format_clock(now, true, true);
    let parsed = chrono::NaiveTime::parse_from_str(&text, "%H:%M:%S").unwrap();
    let now_s = now.time().num_seconds_from_midnight() as i64;
    let got_s = parsed.num_seconds_from_midnight() as i64;
    assert!(
        (now_s - got_s).abs() <= 1,
        "clock mismatch >1s: now={now_s} got={got_s}"
    );
}

#[test]
fn widget_float_mode_moves_more_than_50px_over_60s() {
    let mut cfg = OverlayConfig::default();
    cfg.clock_w.float_mode = true;
    cfg.clock_w.float_speed = 0.08;
    cfg.clock_w.position = [0.5, 0.5];

    let viewport = Vec2::new(1920.0, 1080.0);
    let mut widgets = WidgetSystem::new(&cfg, viewport);
    widgets.update(0.0, viewport, &cfg);
    let p0 = widgets.clock.pos_px;
    widgets.update(60.0, viewport, &cfg);
    let p1 = widgets.clock.pos_px;
    assert!(
        p0.distance(p1) > 50.0,
        "widget drift too small: {}",
        p0.distance(p1)
    );
}
