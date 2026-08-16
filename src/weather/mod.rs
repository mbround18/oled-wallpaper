//! Weather data fetching and galaxy-influence calculation.
//!
//! Weather is fetched in a background thread and stored in a shared
//! `Arc<Mutex<WeatherState>>` so the render loop is never blocked.

pub mod openmeteo;
pub mod openweather;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::config::{WeatherConfig, WeatherProvider};

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum WeatherCondition {
    Clear,
    PartlyCloudy,
    Cloudy,
    Overcast,
    Fog,
    Drizzle,
    Rain,
    HeavyRain,
    Thunderstorm,
    Snow,
    #[default]
    Unknown,
}

impl WeatherCondition {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Clear => "☀",
            Self::PartlyCloudy => "⛅",
            Self::Cloudy | Self::Overcast => "☁",
            Self::Fog => "~",
            Self::Drizzle => "~·",
            Self::Rain => "//",
            Self::HeavyRain => "///",
            Self::Thunderstorm => "⚡",
            Self::Snow => "*",
            Self::Unknown => "?",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::PartlyCloudy => "Partly Cloudy",
            Self::Cloudy => "Cloudy",
            Self::Overcast => "Overcast",
            Self::Fog => "Fog",
            Self::Drizzle => "Drizzle",
            Self::Rain => "Rain",
            Self::HeavyRain => "Heavy Rain",
            Self::Thunderstorm => "Thunderstorm",
            Self::Snow => "Snow",
            Self::Unknown => "Unknown",
        }
    }

    /// Map WMO weather interpretation code → condition.
    pub fn from_wmo(code: u16) -> Self {
        match code {
            0 => Self::Clear,
            1 => Self::PartlyCloudy,
            2 => Self::Cloudy,
            3 => Self::Overcast,
            45 | 48 => Self::Fog,
            51 | 53 | 55 | 56 | 57 => Self::Drizzle,
            61 | 63 | 80 | 81 => Self::Rain,
            65 | 82 => Self::HeavyRain,
            66 | 67 | 71 | 73 | 75 | 77 | 85 | 86 => Self::Snow,
            95 | 96 | 99 => Self::Thunderstorm,
            _ => Self::Unknown,
        }
    }

    /// Map OpenWeatherMap condition ID → condition.
    pub fn from_owm(id: u16) -> Self {
        match id {
            200..=232 => Self::Thunderstorm,
            300..=321 => Self::Drizzle,
            500..=501 => Self::Rain,
            502..=531 => Self::HeavyRain,
            600..=622 => Self::Snow,
            700..=781 => Self::Fog,
            800 => Self::Clear,
            801 => Self::PartlyCloudy,
            802 => Self::Cloudy,
            803 | 804 => Self::Overcast,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WeatherData {
    pub temperature_c: f32,
    pub condition: WeatherCondition,
    pub wind_kmh: f32,
    pub precipitation_mm: f32,
}

impl WeatherData {
    pub fn temperature_display(&self, fahrenheit: bool) -> String {
        if fahrenheit {
            format!("{:.0}F", self.temperature_c * 9.0 / 5.0 + 32.0)
        } else {
            format!("{:.0}C", self.temperature_c)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AqiLevel {
    #[default]
    Good,
    Moderate,
    UnhealthySensitive,
    Unhealthy,
    VeryUnhealthy,
    Hazardous,
}

impl AqiLevel {
    pub fn from_index(aqi: u32) -> Self {
        match aqi {
            0..=50 => Self::Good,
            51..=100 => Self::Moderate,
            101..=150 => Self::UnhealthySensitive,
            151..=200 => Self::Unhealthy,
            201..=300 => Self::VeryUnhealthy,
            _ => Self::Hazardous,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Good => "Good",
            Self::Moderate => "Moderate",
            Self::UnhealthySensitive => "Unhealthy(S)",
            Self::Unhealthy => "Unhealthy",
            Self::VeryUnhealthy => "Very Unhealthy",
            Self::Hazardous => "Hazardous",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AirQualityData {
    pub aqi: u32,
    pub level: AqiLevel,
    pub pm25: f32,
    pub pm10: f32,
}

#[derive(Debug, Clone, Default)]
pub struct WeatherState {
    pub weather: Option<WeatherData>,
    pub air: Option<AirQualityData>,
    pub last_fetch: Option<Instant>,
    pub error: Option<String>,
}

// ─── Galaxy influence ─────────────────────────────────────────────────────────

/// Per-frame values that nudge the visual simulation based on current weather.
#[derive(Debug, Clone)]
pub struct WeatherInfluence {
    /// >1.0 means spawn meteors faster (divide the interval).
    pub meteor_rate: f32,
    /// >1.0 means spawn cosmic rays faster.
    pub cosmic_rate: f32,
    /// 0.0–1.0+ multiplier on background star brightness.
    pub star_brightness: f32,
    /// Tint rain meteors ice-blue.
    pub rain_meteor_tint: bool,
    /// Tint snow meteors pure white.
    pub snow_tint: bool,
    /// Occasionally fire a bright random pulse flash (thunder).
    pub thunder_pulses: bool,
}

impl Default for WeatherInfluence {
    fn default() -> Self {
        Self {
            meteor_rate: 1.0,
            cosmic_rate: 1.0,
            star_brightness: 1.0,
            rain_meteor_tint: false,
            snow_tint: false,
            thunder_pulses: false,
        }
    }
}

impl WeatherInfluence {
    pub fn from_condition(cond: &WeatherCondition) -> Self {
        match cond {
            WeatherCondition::Clear => Self {
                meteor_rate: 0.6,
                cosmic_rate: 0.8,
                star_brightness: 1.15,
                ..Default::default()
            },
            WeatherCondition::Cloudy | WeatherCondition::Overcast => Self {
                star_brightness: 0.75,
                ..Default::default()
            },
            WeatherCondition::Fog => Self {
                star_brightness: 0.45,
                meteor_rate: 0.5,
                ..Default::default()
            },
            WeatherCondition::Drizzle => Self {
                meteor_rate: 1.8,
                rain_meteor_tint: true,
                star_brightness: 0.85,
                ..Default::default()
            },
            WeatherCondition::Rain => Self {
                meteor_rate: 2.8,
                rain_meteor_tint: true,
                star_brightness: 0.65,
                ..Default::default()
            },
            WeatherCondition::HeavyRain => Self {
                meteor_rate: 4.0,
                cosmic_rate: 1.6,
                rain_meteor_tint: true,
                star_brightness: 0.5,
                ..Default::default()
            },
            WeatherCondition::Thunderstorm => Self {
                meteor_rate: 3.5,
                cosmic_rate: 2.5,
                rain_meteor_tint: true,
                thunder_pulses: true,
                star_brightness: 0.4,
                ..Default::default()
            },
            WeatherCondition::Snow => Self {
                meteor_rate: 1.5,
                snow_tint: true,
                star_brightness: 0.9,
                ..Default::default()
            },
            _ => Self::default(),
        }
    }
}

// ─── Fetch dispatch ───────────────────────────────────────────────────────────

pub enum FetchResult {
    Ok {
        weather: WeatherData,
        air: Option<AirQualityData>,
    },
    Err(String),
}

fn do_fetch(cfg: &WeatherConfig) -> FetchResult {
    match cfg.provider {
        WeatherProvider::OpenMeteo => openmeteo::fetch(cfg),
        WeatherProvider::OpenWeatherMap => {
            if cfg.api_key.trim().is_empty() {
                FetchResult::Err("OpenWeatherMap API key not set".to_string())
            } else {
                openweather::fetch(cfg)
            }
        }
    }
}

// ─── Background poller ────────────────────────────────────────────────────────

/// Spawn a background thread that periodically fetches weather and updates
/// the shared `WeatherState`. Returns immediately if weather is disabled.
pub fn start_weather_thread(cfg: WeatherConfig, state: Arc<Mutex<WeatherState>>) {
    if !cfg.enabled {
        return;
    }
    std::thread::spawn(move || loop {
        match do_fetch(&cfg) {
            FetchResult::Ok { weather, air } => {
                if let Ok(mut g) = state.lock() {
                    g.weather = Some(weather);
                    g.air = air;
                    g.error = None;
                    g.last_fetch = Some(Instant::now());
                    tracing::info!("Weather updated");
                }
            }
            FetchResult::Err(e) => {
                tracing::warn!("Weather fetch error: {e}");
                if let Ok(mut g) = state.lock() {
                    g.error = Some(e);
                    g.last_fetch = Some(Instant::now());
                }
            }
        }
        let secs = (cfg.refresh_minutes.max(5) as u64) * 60;
        std::thread::sleep(Duration::from_secs(secs));
    });
}

// ─── Widget text ──────────────────────────────────────────────────────────────

pub fn weather_widget_text(state: &WeatherState, cfg: &WeatherConfig) -> Option<String> {
    let w = state.weather.as_ref()?;
    let temp = w.temperature_display(cfg.units_fahrenheit);
    let icon = w.condition.icon();
    let cond = w.condition.label();
    let mut lines = vec![format!("{icon} {temp}  {cond}")];
    if w.wind_kmh > 0.5 {
        lines.push(format!("Wind {:.0} km/h", w.wind_kmh));
    }
    if cfg.show_aqi {
        if let Some(air) = &state.air {
            lines.push(format!("AQI {}  {}", air.aqi, air.level.label()));
        }
    }
    Some(lines.join("\n"))
}
