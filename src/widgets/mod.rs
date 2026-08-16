use chrono::{DateTime, Local};
use glam::Vec2;

use crate::config::OverlayConfig;
use crate::weather::{weather_widget_text, WeatherState};

pub mod calendar;
pub mod clock;

const WIDGET_WIDTH: f32 = 400.0;
const WIDGET_HEIGHT: f32 = 130.0;

#[derive(Debug, Clone, Copy)]
pub struct ClockWidget {
    pub use_24h: bool,
    pub show_seconds: bool,
}

impl ClockWidget {
    pub fn render(&self, now: DateTime<Local>) -> String {
        clock::format_clock(now, self.use_24h, self.show_seconds)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CalendarWidget {
    pub month_view: bool,
}

impl CalendarWidget {
    pub fn render(&self, now: DateTime<Local>) -> String {
        calendar::format_calendar(now, self.month_view)
    }
}

#[derive(Debug, Clone)]
pub struct WidgetSystem {
    pos_px: Vec2,
    anchor_px: Vec2,
    drag_offset_px: Vec2,
    dragging: bool,
}

impl WidgetSystem {
    pub fn new(cfg: &OverlayConfig, viewport: Vec2) -> Self {
        let anchor = Vec2::new(
            cfg.widget_position[0].clamp(0.0, 1.0) * viewport.x,
            cfg.widget_position[1].clamp(0.0, 1.0) * viewport.y,
        );
        Self {
            pos_px: anchor,
            anchor_px: anchor,
            drag_offset_px: Vec2::ZERO,
            dragging: false,
        }
    }

    pub fn update(&mut self, t: f32, viewport: Vec2, cfg: &OverlayConfig) {
        if self.dragging {
            return;
        }
        if cfg.widget_float_mode {
            let speed = cfg.widget_float_speed.max(0.01);
            let ax = 110.0;
            let ay = 78.0;
            self.pos_px =
                self.anchor_px + Vec2::new((t * speed).sin() * ax, (t * speed * 1.37).cos() * ay);
            self.clamp_to_view(viewport);
        } else {
            self.pos_px = self.anchor_px;
            self.clamp_to_view(viewport);
        }
    }

    pub fn begin_drag(&mut self, cursor_px: Vec2) {
        self.dragging = true;
        self.drag_offset_px = cursor_px - self.pos_px;
    }

    pub fn drag_to(&mut self, cursor_px: Vec2, viewport: Vec2) {
        if !self.dragging {
            return;
        }
        self.pos_px = cursor_px - self.drag_offset_px;
        self.clamp_to_view(viewport);
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
        self.anchor_px = self.pos_px;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn hit_test(&self, cursor_px: Vec2) -> bool {
        cursor_px.x >= self.pos_px.x
            && cursor_px.x <= self.pos_px.x + WIDGET_WIDTH
            && cursor_px.y >= self.pos_px.y
            && cursor_px.y <= self.pos_px.y + WIDGET_HEIGHT
    }

    pub fn position_px(&self) -> Vec2 {
        self.pos_px
    }

    pub fn position_norm(&self, viewport: Vec2) -> [f32; 2] {
        let x = (self.anchor_px.x / viewport.x.max(1.0)).clamp(0.0, 1.0);
        let y = (self.anchor_px.y / viewport.y.max(1.0)).clamp(0.0, 1.0);
        [x, y]
    }

    pub fn text(
        &self,
        cfg: &OverlayConfig,
        weather: Option<&WeatherState>,
        weather_cfg: Option<&crate::config::WeatherConfig>,
    ) -> String {
        let now = Local::now();
        let clock = ClockWidget {
            use_24h: cfg.clock_24h,
            show_seconds: cfg.clock_show_seconds,
        };
        let calendar = CalendarWidget {
            month_view: cfg.calendar_month_view,
        };
        let mut lines = Vec::new();
        if cfg.show_clock {
            lines.push(clock.render(now));
        }
        if cfg.show_calendar {
            lines.push(calendar.render(now));
        }
        // Weather lines
        if let (Some(ws), Some(wc)) = (weather, weather_cfg) {
            if wc.enabled {
                if let Some(wtext) = weather_widget_text(ws, wc) {
                    lines.push(wtext);
                } else if ws.error.is_some() {
                    lines.push("Weather: unavailable".to_string());
                }
            }
        }
        lines.join("\n")
    }

    fn clamp_to_view(&mut self, viewport: Vec2) {
        self.pos_px.x = self
            .pos_px
            .x
            .clamp(0.0, (viewport.x - WIDGET_WIDTH).max(0.0));
        self.pos_px.y = self
            .pos_px
            .y
            .clamp(0.0, (viewport.y - WIDGET_HEIGHT).max(0.0));
    }
}
