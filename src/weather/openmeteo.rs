//! Open-Meteo weather + air quality fetcher (free, no API key required).
//!
//! Weather: https://api.open-meteo.com/v1/forecast
//! Air Quality: https://air-quality-api.open-meteo.com/v1/air-quality

use crate::config::WeatherConfig;
use crate::weather::{AirQualityData, AqiLevel, FetchResult, WeatherCondition, WeatherData};

pub fn fetch(cfg: &WeatherConfig) -> FetchResult {
    let weather = match fetch_weather(cfg) {
        Ok(w) => w,
        Err(e) => return FetchResult::Err(e),
    };
    let air = fetch_air(cfg).ok();
    FetchResult::Ok { weather, air }
}

fn fetch_weather(cfg: &WeatherConfig) -> Result<WeatherData, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?\
         latitude={lat}&longitude={lon}\
         &current=temperature_2m,wind_speed_10m,precipitation,weather_code\
         &wind_speed_unit=kmh&timezone=auto",
        lat = cfg.latitude,
        lon = cfg.longitude,
    );

    let body: serde_json::Value = ureq::get(&url)
        .call()
        .map_err(|e| format!("Open-Meteo request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("Open-Meteo JSON parse error: {e}"))?;

    let cur = body
        .get("current")
        .ok_or("Open-Meteo: missing 'current' key")?;

    let temperature_c = cur["temperature_2m"].as_f64().unwrap_or(0.0) as f32;
    let wind_kmh = cur["wind_speed_10m"].as_f64().unwrap_or(0.0) as f32;
    let precipitation_mm = cur["precipitation"].as_f64().unwrap_or(0.0) as f32;
    let code = cur["weather_code"].as_u64().unwrap_or(0) as u16;

    Ok(WeatherData {
        temperature_c,
        condition: WeatherCondition::from_wmo(code),
        wind_kmh,
        precipitation_mm,
    })
}

fn fetch_air(cfg: &WeatherConfig) -> Result<AirQualityData, String> {
    let url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?\
         latitude={lat}&longitude={lon}\
         &current=us_aqi,pm10,pm2_5&timezone=auto",
        lat = cfg.latitude,
        lon = cfg.longitude,
    );

    let body: serde_json::Value = ureq::get(&url)
        .call()
        .map_err(|e| format!("Open-Meteo AQI request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("Open-Meteo AQI parse error: {e}"))?;

    let cur = body
        .get("current")
        .ok_or("Open-Meteo AQI: missing 'current' key")?;

    let aqi = cur["us_aqi"].as_u64().unwrap_or(0) as u32;
    let pm25 = cur["pm2_5"].as_f64().unwrap_or(0.0) as f32;
    let pm10 = cur["pm10"].as_f64().unwrap_or(0.0) as f32;

    Ok(AirQualityData {
        aqi,
        level: AqiLevel::from_index(aqi),
        pm25,
        pm10,
    })
}
