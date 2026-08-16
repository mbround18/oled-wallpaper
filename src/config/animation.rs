//! Animation and overlay configuration parameters.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

// ─── Animation ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Global orbital speed multiplier (0.1 – 5.0, default 1.0)
    #[serde(default = "default_planet_speed")]
    pub planet_speed: f32,

    /// Per-planet RGBA colours (mercury → saturn order)
    #[serde(default = "default_planet_colors")]
    pub planet_colors: Vec<[f32; 4]>,

    /// Per-planet visual radius multipliers (1.0 = default size)
    #[serde(default = "default_planet_sizes")]
    pub planet_sizes: Vec<f32>,

    /// Binary star A colour (RGBA)
    #[serde(default = "default_star_a_color")]
    pub star_a_color: [f32; 4],

    /// Binary star B colour (RGBA)
    #[serde(default = "default_star_b_color")]
    pub star_b_color: [f32; 4],

    /// Legacy sun colour retained for compatibility with existing tests/configs.
    #[serde(default = "default_sun_color")]
    pub sun_color: [f32; 4],

    /// Legacy sun size retained for compatibility with existing tests/configs.
    #[serde(default = "default_sun_size")]
    pub sun_size: f32,

    /// Default camera zoom (0.1 – 5.0, default 1.0 — affects view_half_h scale)
    #[serde(default = "default_camera_zoom")]
    pub camera_zoom: f32,

    /// Pulse effect base colour (RGBA) — random if not set
    #[serde(default = "default_pulse_color")]
    pub pulse_color: Option<[f32; 4]>,

    /// Pulse ring brightness multiplier (0.1 – 2.0, default 1.0)
    #[serde(default = "default_pulse_intensity")]
    pub pulse_intensity: f32,
}

fn default_planet_speed() -> f32 {
    1.0
}
fn default_camera_zoom() -> f32 {
    1.0
}
fn default_pulse_color() -> Option<[f32; 4]> {
    None
} // random per-click
fn default_pulse_intensity() -> f32 {
    1.0
}

fn default_planet_colors() -> Vec<[f32; 4]> {
    vec![
        [0.70, 0.60, 0.50, 1.0], // mercury
        [0.92, 0.76, 0.44, 1.0], // venus
    ]
}
fn default_planet_sizes() -> Vec<f32> {
    vec![1.0; 2]
}
fn default_star_a_color() -> [f32; 4] {
    [1.00, 0.88, 0.32, 1.0]
}
fn default_star_b_color() -> [f32; 4] {
    [0.98, 0.72, 0.55, 1.0]
}
fn default_sun_color() -> [f32; 4] {
    [1.0, 0.9, 0.0, 1.0]
}
fn default_sun_size() -> f32 {
    2.0
}

impl AnimationConfig {
    pub fn validate(&self) -> Result<()> {
        if !(0.1..=5.0).contains(&self.planet_speed) {
            return Err(Error::Validation(format!(
                "planet_speed {} out of range 0.1–5.0",
                self.planet_speed
            )));
        }
        if !(0.1..=5.0).contains(&self.camera_zoom) {
            return Err(Error::Validation(format!(
                "camera_zoom {} out of range 0.1–5.0",
                self.camera_zoom
            )));
        }
        if !(0.1..=2.0).contains(&self.pulse_intensity) {
            return Err(Error::Validation(format!(
                "pulse_intensity {} out of range 0.1–2.0",
                self.pulse_intensity
            )));
        }
        for (i, &s) in self.planet_sizes.iter().enumerate() {
            if s <= 0.0 {
                return Err(Error::Validation(format!("planet_sizes[{i}] must be > 0")));
            }
        }
        for (i, color) in self.planet_colors.iter().enumerate() {
            for (j, c) in color.iter().enumerate() {
                if !(0.0..=1.0).contains(c) {
                    return Err(Error::Validation(format!(
                        "planet_colors[{i}][{j}] out of range 0.0–1.0"
                    )));
                }
            }
            if color[3] <= 0.0 {
                return Err(Error::Validation(format!(
                    "planet_colors[{i}] alpha must be > 0"
                )));
            }
        }
        for (i, &c) in self.sun_color.iter().enumerate() {
            if !(0.0..=1.0).contains(&c) {
                return Err(Error::Validation(format!(
                    "sun_color[{i}] out of range 0.0–1.0"
                )));
            }
        }
        if self.sun_color[3] <= 0.0 {
            return Err(Error::Validation("sun_color alpha must be > 0".to_string()));
        }
        if self.sun_size <= 0.0 {
            return Err(Error::Validation(format!(
                "sun_size {} must be > 0",
                self.sun_size
            )));
        }
        Ok(())
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        AnimationConfig {
            planet_speed: default_planet_speed(),
            planet_colors: default_planet_colors(),
            planet_sizes: default_planet_sizes(),
            star_a_color: default_star_a_color(),
            star_b_color: default_star_b_color(),
            sun_color: default_sun_color(),
            sun_size: default_sun_size(),
            camera_zoom: default_camera_zoom(),
            pulse_color: default_pulse_color(),
            pulse_intensity: default_pulse_intensity(),
        }
    }
}

// ─── Overlay ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayWidget {
    pub enabled: bool,
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub font_scale: f32,
    pub float_mode: bool,
    pub float_speed: f32,
}
impl Default for OverlayWidget {
    fn default() -> Self {
        Self {
            enabled: true,
            position: [0.78, 0.78],
            color: [0.82, 0.92, 1.0, 0.95],
            font_scale: 1.0,
            float_mode: true,
            float_speed: 0.08,
        }
    }
}
fn default_weather_w() -> OverlayWidget {
    OverlayWidget {
        position: [0.02, 0.92],
        ..Default::default()
    }
}
fn default_wind_w() -> OverlayWidget {
    OverlayWidget {
        position: [0.02, 0.96],
        ..Default::default()
    }
}
fn default_aqi_w() -> OverlayWidget {
    OverlayWidget {
        position: [0.02, 0.88],
        ..Default::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    /// Show performance HUD (FPS/CPU/GPU/RAM). Also enabled by --demo flag.
    #[serde(default)]
    pub show_hud: bool,

    /// Master toggle for overlay widgets.
    #[serde(default = "default_widget_enabled")]
    pub widget_enabled: bool,

    /// Clock widget visibility.
    #[serde(default = "default_show_clock")]
    pub show_clock: bool,

    /// Calendar widget visibility.
    #[serde(default)]
    pub show_calendar: bool,

    /// 24-hour clock format (true) or 12-hour AM/PM (false).
    #[serde(default)]
    pub clock_24h: bool,

    /// Show seconds in clock display.
    #[serde(default = "default_clock_show_seconds")]
    pub clock_show_seconds: bool,

    /// Calendar format: month-year header (true) vs full date line (false).
    #[serde(default)]
    pub calendar_month_view: bool,

    /// Move widgets continuously to reduce static pixels.
    #[serde(default = "default_widget_float_mode")]
    pub widget_float_mode: bool,

    /// Lissajous drift speed multiplier.
    #[serde(default = "default_widget_float_speed")]
    pub widget_float_speed: f32,

    /// Initial/anchor widget position as normalized [x,y] in [0,1].
    #[serde(default = "default_widget_position")]
    pub widget_position: [f32; 2],

    /// Widget text RGBA colour.
    #[serde(default = "default_widget_color")]
    pub widget_color: [f32; 4],

    /// Widget text font scale multiplier.
    #[serde(default = "default_widget_font_scale")]
    pub widget_font_scale: f32,

    #[serde(default)]
    pub clock_w: OverlayWidget,

    #[serde(default = "default_weather_w")]
    pub weather_w: OverlayWidget,

    #[serde(default = "default_wind_w")]
    pub wind_w: OverlayWidget,

    #[serde(default = "default_aqi_w")]
    pub aqi_w: OverlayWidget,
}

fn default_widget_enabled() -> bool {
    true
}
fn default_show_clock() -> bool {
    true
}
fn default_clock_show_seconds() -> bool {
    true
}
fn default_widget_float_mode() -> bool {
    true
}
fn default_widget_float_speed() -> f32 {
    0.08
}
fn default_widget_position() -> [f32; 2] {
    [0.78, 0.78]
}
fn default_widget_color() -> [f32; 4] {
    [0.82, 0.92, 1.0, 0.95]
}
fn default_widget_font_scale() -> f32 {
    1.0
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            show_hud: false,
            widget_enabled: default_widget_enabled(),
            show_clock: default_show_clock(),
            show_calendar: false,
            clock_24h: false,
            clock_show_seconds: default_clock_show_seconds(),
            calendar_month_view: false,
            widget_float_mode: default_widget_float_mode(),
            widget_float_speed: default_widget_float_speed(),
            widget_position: default_widget_position(),
            widget_color: default_widget_color(),
            widget_font_scale: default_widget_font_scale(),
            clock_w: OverlayWidget::default(),
            weather_w: default_weather_w(),
            wind_w: default_wind_w(),
            aqi_w: default_aqi_w(),
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_validates() {
        assert!(AnimationConfig::default().validate().is_ok());
    }
    #[test]
    fn bad_speed_fails() {
        let c = AnimationConfig {
            planet_speed: 0.0,
            ..AnimationConfig::default()
        };
        assert!(c.validate().is_err());
    }
    #[test]
    fn bad_zoom_fails() {
        let c = AnimationConfig {
            camera_zoom: 99.0,
            ..AnimationConfig::default()
        };
        assert!(c.validate().is_err());
    }
    #[test]
    fn bad_pulse_intensity_fails() {
        let c = AnimationConfig {
            pulse_intensity: 5.0,
            ..AnimationConfig::default()
        };
        assert!(c.validate().is_err());
    }
    #[test]
    fn bad_planet_size_fails() {
        let mut c = AnimationConfig::default();
        c.planet_sizes[0] = 0.0;
        assert!(c.validate().is_err());
    }
}
