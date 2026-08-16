#![allow(clippy::field_reassign_with_default)]

use glam::Vec2;
use oled_wallpaper::config::OverlayConfig;
use oled_wallpaper::widgets::WidgetSystem;

#[test]
fn widget_text_renders_and_remains_in_viewport() {
    let mut cfg = OverlayConfig::default();
    cfg.widget_enabled = true;
    cfg.show_clock = true;
    cfg.show_calendar = true;
    cfg.widget_float_mode = true;

    let viewport = Vec2::new(2560.0, 1440.0);
    let mut widgets = WidgetSystem::new(&cfg, viewport);
    widgets.update(10.0, viewport, &cfg);
    let text = widgets.text(&cfg);
    assert!(!text.is_empty());
    let p = widgets.position_px();
    assert!(p.x >= 0.0 && p.x <= viewport.x);
    assert!(p.y >= 0.0 && p.y <= viewport.y);
}
