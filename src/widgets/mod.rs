use chrono::Local;
use glam::Vec2;

use crate::config::WeatherConfig;
use crate::config::{OverlayConfig, OverlayWidget};
use crate::weather::WeatherState;

pub mod calendar;
pub mod clock;

// ─── Per-widget text generators ──────────────────────────────────────────────

pub fn clock_text(cfg: &OverlayConfig) -> Option<String> {
    if !cfg.show_clock {
        return None;
    }
    Some(clock::format_clock(
        Local::now(),
        cfg.clock_24h,
        cfg.clock_show_seconds,
    ))
}

pub fn calendar_text(cfg: &OverlayConfig) -> Option<String> {
    if !cfg.show_calendar {
        return None;
    }
    Some(calendar::format_calendar(
        Local::now(),
        cfg.calendar_month_view,
    ))
}

pub fn weather_temp_text(ws: &WeatherState, wc: &WeatherConfig) -> Option<String> {
    let w = ws.weather.as_ref()?;
    let temp = w.temperature_display(wc.units_fahrenheit);
    let cond = w.condition.label();
    let icon = condition_icon_text(w.condition.clone());
    Some(format!("{icon} {temp} {cond}"))
}

pub fn weather_wind_text(ws: &WeatherState) -> Option<String> {
    let w = ws.weather.as_ref()?;
    if w.wind_kmh < 0.5 {
        return None;
    }
    Some(format!("Wind  {:.0} km/h", w.wind_kmh))
}

pub fn weather_aqi_text(ws: &WeatherState) -> Option<String> {
    let air = ws.air.as_ref()?;
    Some(format!("AQI {}  {}", air.aqi, air.level.label()))
}

/// Alias used by tests
pub use weather_aqi_text as aqi_text;

// ─── Icon text ────────────────────────────────────────────────────────────────

use crate::weather::WeatherCondition;

pub fn condition_icon_text(cond: WeatherCondition) -> &'static str {
    match cond {
        WeatherCondition::Clear => "\u{2600}",
        WeatherCondition::PartlyCloudy => "\u{26C5}",
        WeatherCondition::Cloudy => "\u{2601}",
        WeatherCondition::Overcast => "\u{2601}",
        WeatherCondition::Fog => "~",
        WeatherCondition::Drizzle => "\u{2614}",
        WeatherCondition::Rain => "\u{2602}",
        WeatherCondition::HeavyRain => "\u{2602}\u{2602}",
        WeatherCondition::Thunderstorm => "\u{26A1}",
        WeatherCondition::Snow => "\u{2744}",
        WeatherCondition::Unknown => "?",
    }
}

// ─── Draggable widget instance ─────────────────────────────────────────────

const W: f32 = 420.0;
const H: f32 = 50.0;

#[derive(Debug, Clone)]
pub struct DraggableWidget {
    pub pos_px: Vec2,
    anchor_px: Vec2,
    drag_offset: Vec2,
    dragging: bool,
}

impl DraggableWidget {
    pub fn from_cfg(cfg: &OverlayWidget, viewport: Vec2) -> Self {
        let anchor = Vec2::new(
            cfg.position[0].clamp(0.0, 1.0) * viewport.x,
            cfg.position[1].clamp(0.0, 1.0) * viewport.y,
        );
        Self {
            pos_px: anchor,
            anchor_px: anchor,
            drag_offset: Vec2::ZERO,
            dragging: false,
        }
    }

    pub fn update(&mut self, t: f32, viewport: Vec2, cfg: &OverlayWidget) {
        if self.dragging {
            return;
        }
        if cfg.float_mode {
            let s = cfg.float_speed.max(0.01);
            self.pos_px =
                self.anchor_px + Vec2::new((t * s).sin() * 80.0, (t * s * 1.37).cos() * 55.0);
        } else {
            self.pos_px = self.anchor_px;
        }
        self.pos_px.x = self.pos_px.x.clamp(0.0, (viewport.x - W).max(0.0));
        self.pos_px.y = self.pos_px.y.clamp(0.0, (viewport.y - H).max(0.0));
    }

    pub fn hit_test(&self, cursor: Vec2) -> bool {
        cursor.x >= self.pos_px.x
            && cursor.x <= self.pos_px.x + W
            && cursor.y >= self.pos_px.y
            && cursor.y <= self.pos_px.y + H
    }

    pub fn begin_drag(&mut self, cursor: Vec2) {
        self.dragging = true;
        self.drag_offset = cursor - self.pos_px;
    }

    pub fn drag_to(&mut self, cursor: Vec2, viewport: Vec2) {
        if !self.dragging {
            return;
        }
        self.pos_px = cursor - self.drag_offset;
        self.pos_px.x = self.pos_px.x.clamp(0.0, (viewport.x - W).max(0.0));
        self.pos_px.y = self.pos_px.y.clamp(0.0, (viewport.y - H).max(0.0));
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
        self.anchor_px = self.pos_px;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn position_norm(&self, viewport: Vec2) -> [f32; 2] {
        [
            (self.anchor_px.x / viewport.x.max(1.0)).clamp(0.0, 1.0),
            (self.anchor_px.y / viewport.y.max(1.0)).clamp(0.0, 1.0),
        ]
    }
}

// ─── Multi-widget system ──────────────────────────────────────────────────────

pub struct WidgetSystem {
    pub clock: DraggableWidget,
    pub weather: DraggableWidget,
    pub wind: DraggableWidget,
    pub aqi: DraggableWidget,
}

impl WidgetSystem {
    pub fn new(cfg: &OverlayConfig, viewport: Vec2) -> Self {
        Self {
            clock: DraggableWidget::from_cfg(&cfg.clock_w, viewport),
            weather: DraggableWidget::from_cfg(&cfg.weather_w, viewport),
            wind: DraggableWidget::from_cfg(&cfg.wind_w, viewport),
            aqi: DraggableWidget::from_cfg(&cfg.aqi_w, viewport),
        }
    }

    pub fn update(&mut self, t: f32, viewport: Vec2, cfg: &OverlayConfig) {
        self.clock.update(t, viewport, &cfg.clock_w);
        self.weather.update(t, viewport, &cfg.weather_w);
        self.wind.update(t, viewport, &cfg.wind_w);
        self.aqi.update(t, viewport, &cfg.aqi_w);
    }

    pub fn hit_test(&self, cursor: Vec2, cfg: &OverlayConfig) -> Option<WidgetId> {
        if cfg.clock_w.enabled && self.clock.hit_test(cursor) {
            return Some(WidgetId::Clock);
        }
        if cfg.weather_w.enabled && self.weather.hit_test(cursor) {
            return Some(WidgetId::Weather);
        }
        if cfg.wind_w.enabled && self.wind.hit_test(cursor) {
            return Some(WidgetId::Wind);
        }
        if cfg.aqi_w.enabled && self.aqi.hit_test(cursor) {
            return Some(WidgetId::Aqi);
        }
        None
    }

    pub fn begin_drag(&mut self, id: WidgetId, cursor: Vec2) {
        self.widget_mut(id).begin_drag(cursor);
    }
    pub fn drag_to(&mut self, id: WidgetId, cursor: Vec2, viewport: Vec2) {
        self.widget_mut(id).drag_to(cursor, viewport);
    }
    pub fn end_drag(&mut self, id: WidgetId) {
        self.widget_mut(id).end_drag();
    }
    pub fn any_dragging(&self) -> Option<WidgetId> {
        if self.clock.is_dragging() {
            return Some(WidgetId::Clock);
        }
        if self.weather.is_dragging() {
            return Some(WidgetId::Weather);
        }
        if self.wind.is_dragging() {
            return Some(WidgetId::Wind);
        }
        if self.aqi.is_dragging() {
            return Some(WidgetId::Aqi);
        }
        None
    }

    fn widget_mut(&mut self, id: WidgetId) -> &mut DraggableWidget {
        match id {
            WidgetId::Clock => &mut self.clock,
            WidgetId::Weather => &mut self.weather,
            WidgetId::Wind => &mut self.wind,
            WidgetId::Aqi => &mut self.aqi,
        }
    }

    pub fn write_positions(&self, cfg: &mut OverlayConfig, viewport: Vec2) {
        cfg.clock_w.position = self.clock.position_norm(viewport);
        cfg.weather_w.position = self.weather.position_norm(viewport);
        cfg.wind_w.position = self.wind.position_norm(viewport);
        cfg.aqi_w.position = self.aqi.position_norm(viewport);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetId {
    Clock,
    Weather,
    Wind,
    Aqi,
}
