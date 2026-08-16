//! Weather provider configuration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WeatherProvider {
    /// Free, no API key needed. Uses open-meteo.com
    #[default]
    OpenMeteo,
    /// Requires an API key from openweathermap.org
    OpenWeatherMap,
}

impl WeatherProvider {
    pub fn label(&self) -> &'static str {
        match self {
            Self::OpenMeteo => "Open-Meteo (free)",
            Self::OpenWeatherMap => "OpenWeatherMap (API key required)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    /// Master toggle — set false to disable all weather features.
    #[serde(default)]
    pub enabled: bool,

    /// Which provider to use.
    #[serde(default)]
    pub provider: WeatherProvider,

    /// API key used by providers that require one (e.g., OpenWeatherMap).
    #[serde(default)]
    pub api_key: String,

    /// Location latitude (decimal degrees, e.g. 37.77 for San Francisco).
    #[serde(default = "default_latitude")]
    pub latitude: f32,

    /// Location longitude (decimal degrees, e.g. -122.41 for San Francisco).
    #[serde(default = "default_longitude")]
    pub longitude: f32,

    /// Display temperature in Fahrenheit instead of Celsius.
    #[serde(default)]
    pub units_fahrenheit: bool,

    /// Also fetch and display Air Quality Index.
    #[serde(default = "default_show_aqi")]
    pub show_aqi: bool,

    /// How often to refresh weather data (minimum 5 minutes).
    #[serde(default = "default_refresh_minutes")]
    pub refresh_minutes: u32,

    /// Let weather conditions influence the galaxy visuals.
    #[serde(default = "default_affect_galaxy")]
    pub affect_galaxy: bool,
}

fn default_latitude() -> f32 {
    37.77
}
fn default_longitude() -> f32 {
    -122.41
}
fn default_show_aqi() -> bool {
    true
}
fn default_refresh_minutes() -> u32 {
    15
}
fn default_affect_galaxy() -> bool {
    true
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: WeatherProvider::OpenMeteo,
            api_key: String::new(),
            latitude: default_latitude(),
            longitude: default_longitude(),
            units_fahrenheit: false,
            show_aqi: default_show_aqi(),
            refresh_minutes: default_refresh_minutes(),
            affect_galaxy: default_affect_galaxy(),
        }
    }
}
