use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnimationConfig {
    pub planet_speed: f32,
    pub camera_zoom: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WidgetConfig {
    pub enabled: bool,
    pub x: i32,
    pub y: i32,
    pub float_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: u32,
    pub animation: AnimationConfig,
    pub widgets: Widgets,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Widgets {
    pub clock: WidgetConfig,
    pub calendar: WidgetConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            animation: AnimationConfig { planet_speed: 1.0, camera_zoom: 1.0 },
            widgets: Widgets {
                clock: WidgetConfig { enabled: true, x: 50, y: 50, float_mode: false },
                calendar: WidgetConfig { enabled: true, x: 200, y: 50, float_mode: false },
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, anyhow::Error> {
    let p = path.as_ref();
    if !p.exists() {
        return Ok(Config::default());
    }
    let s = fs::read_to_string(p)?;
    let cfg: Config = toml::from_str(&s)?;
    Ok(cfg)
}

pub fn save_config<P: AsRef<Path>>(path: P, cfg: &Config) -> Result<(), anyhow::Error> {
    let p = path.as_ref();
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    // Atomic write: write to temp then rename
    let tmp = p.with_extension("toml.tmp");
    let mut f = fs::File::create(&tmp)?;
    let s = toml::to_string_pretty(cfg)?;
    f.write_all(s.as_bytes())?;
    f.flush()?;
    drop(f);
    fs::rename(&tmp, p)?;
    Ok(())
}
