//! OpenWeatherMap weather + air quality fetcher (requires an API key).
//!
//! Weather: https://api.openweathermap.org/data/2.5/weather
//! Air Pollution: https://api.openweathermap.org/data/2.5/air_pollution

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
        "https://api.openweathermap.org/data/2.5/weather?\
         lat={lat}&lon={lon}&appid={key}&units=metric",
        lat = cfg.latitude,
        lon = cfg.longitude,
        key = cfg.api_key.trim(),
    );

    let body: serde_json::Value = ureq::get(&url)
        .call()
        .map_err(|e| format!("OWM request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("OWM JSON parse error: {e}"))?;

    let temperature_c = body["main"]["temp"].as_f64().unwrap_or(0.0) as f32;
    let wind_ms = body["wind"]["speed"].as_f64().unwrap_or(0.0) as f32;
    let wind_kmh = wind_ms * 3.6;
    // rain/snow precipitation in last 1h (OWM may omit this key)
    let precipitation_mm = body["rain"]["1h"]
        .as_f64()
        .or_else(|| body["snow"]["1h"].as_f64())
        .unwrap_or(0.0) as f32;

    let condition = body["weather"][0]["id"]
        .as_u64()
        .map(|id| WeatherCondition::from_owm(id as u16))
        .unwrap_or_default();

    Ok(WeatherData {
        temperature_c,
        condition,
        wind_kmh,
        precipitation_mm,
    })
}

fn fetch_air(cfg: &WeatherConfig) -> Result<AirQualityData, String> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/air_pollution?\
         lat={lat}&lon={lon}&appid={key}",
        lat = cfg.latitude,
        lon = cfg.longitude,
        key = cfg.api_key.trim(),
    );

    let body: serde_json::Value = ureq::get(&url)
        .call()
        .map_err(|e| format!("OWM AQI request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("OWM AQI parse error: {e}"))?;

    let entry = body["list"].get(0).ok_or("OWM AQI: empty list")?;

    // OWM AQI scale is 1–5; convert to approximate US AQI bucket midpoints
    let owm_aqi = entry["main"]["aqi"].as_u64().unwrap_or(1) as u32;
    let us_aqi = match owm_aqi {
        1 => 25,
        2 => 75,
        3 => 125,
        4 => 175,
        _ => 250,
    };
    let pm25 = entry["components"]["pm2_5"].as_f64().unwrap_or(0.0) as f32;
    let pm10 = entry["components"]["pm10"].as_f64().unwrap_or(0.0) as f32;

    Ok(AirQualityData {
        aqi: us_aqi,
        level: AqiLevel::from_index(us_aqi),
        pm25,
        pm10,
    })
}
