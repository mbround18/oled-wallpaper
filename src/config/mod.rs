//! Configuration loading — TOML file at ~/.config/oled-wallpaper/config.toml

pub mod animation;
pub use animation::{AnimationConfig, OverlayConfig, OverlayWidget};

mod weather_config;
pub use weather_config::{WeatherConfig, WeatherProvider};

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub animation: AnimationConfig,
    #[serde(default)]
    pub overlay: OverlayConfig,
    #[serde(default)]
    pub weather: WeatherConfig,
}

impl Config {
    /// Load from `~/.config/oled-wallpaper/config.toml`, creating defaults if absent.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match Self::load_from_file(&path) {
                Ok(c) => {
                    info!("Config loaded from {:?}", path);
                    c
                }
                Err(e) => {
                    warn!("Config parse error ({e}), using defaults");
                    Self::default()
                }
            }
        } else {
            info!("No config found at {:?}, writing defaults", path);
            let cfg = Self::default();
            cfg.write_defaults();
            cfg
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let src = std::fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&src)?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        self.animation.validate()?;
        if !(0.01..=2.0).contains(&self.overlay.widget_float_speed) {
            return Err(Error::Validation(format!(
                "widget_float_speed {} out of range 0.01–2.0",
                self.overlay.widget_float_speed
            )));
        }
        if !(0.5..=3.0).contains(&self.overlay.widget_font_scale) {
            return Err(Error::Validation(format!(
                "widget_font_scale {} out of range 0.5–3.0",
                self.overlay.widget_font_scale
            )));
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        std::fs::create_dir_all(Self::config_dir())?;
        let toml = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {e}")))?;
        std::fs::write(Self::config_path(), toml)?;
        Ok(())
    }

    /// Write a commented default config so users know what to edit.
    pub fn write_defaults(&self) {
        let dir = Self::config_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            warn!("Could not create config dir: {e}");
            return;
        }
        let content = r#"# OLED Wallpaper configuration
# Edit and save — changes take effect on next launch.

[animation]
# Global orbital speed multiplier (0.1 – 5.0)
planet_speed = 1.0

# Per-planet RGBA colours: mercury, venus, earth, mars, jupiter, saturn
planet_colors = [
  [0.70, 0.60, 0.50, 1.0],
  [0.92, 0.76, 0.44, 1.0],
  [0.18, 0.48, 0.90, 1.0],
  [0.82, 0.32, 0.16, 1.0],
  [0.80, 0.66, 0.48, 1.0],
  [0.88, 0.78, 0.50, 1.0],
]

# Per-planet size multipliers (1.0 = default)
planet_sizes = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0]

# Binary star colours (RGBA)
star_a_color = [1.00, 0.88, 0.32, 1.0]
star_b_color = [0.98, 0.72, 0.55, 1.0]

# Camera zoom (0.1 – 5.0)
camera_zoom = 1.0

# Pulse ring intensity (0.1 – 2.0; omit pulse_color to use random colours)
pulse_intensity = 1.0

[overlay]
# Show performance HUD (FPS / CPU / GPU / RAM) outside of --demo mode
show_hud = false

# Widget overlay settings
widget_enabled = true
show_clock = true
show_calendar = false
clock_24h = false
clock_show_seconds = true
calendar_month_view = false
widget_float_mode = true
widget_float_speed = 0.08
widget_position = [0.78, 0.78]
widget_color = [0.82, 0.92, 1.0, 0.95]
widget_font_scale = 1.0
"#;
        if let Err(e) = std::fs::write(Self::config_path(), content) {
            warn!("Could not write default config: {e}");
        }
    }

    pub fn config_dir() -> PathBuf {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".config/oled-wallpaper")
    }
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_valid() {
        assert!(Config::default().animation.validate().is_ok());
    }
    #[test]
    fn round_trip_toml() {
        let src = "[animation]\nplanet_speed = 1.5\n[overlay]\nshow_hud = true\n";
        let c: Config = toml::from_str(src).unwrap();
        assert_eq!(c.animation.planet_speed, 1.5);
        assert!(c.overlay.show_hud);
    }
}
