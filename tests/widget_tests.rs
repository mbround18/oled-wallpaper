//! Unit tests for widget system and configurator logic.

use oled_wallpaper::{
    config::{Config, OverlayConfig},
    weather::{
        AirQualityData, AqiLevel, WeatherCondition, WeatherData, WeatherInfluence, WeatherState,
    },
    widgets::{
        aqi_text, calendar_text, clock_text, condition_icon_text, weather_temp_text,
        weather_wind_text,
    },
};

// ── Config round-trip ────────────────────────────────────────────────────────

#[test]
fn config_default_valid() {
    let cfg = Config::default();
    assert!(cfg.animation.validate().is_ok());
}

#[test]
fn config_round_trips_toml() {
    let mut cfg = Config::default();
    cfg.animation.planet_speed = 2.5;
    cfg.overlay.clock_w.position = [0.1, 0.9];

    let toml = toml::to_string_pretty(&cfg).expect("serialize");
    let cfg2: Config = toml::from_str(&toml).expect("deserialize");

    assert!((cfg2.animation.planet_speed - 2.5).abs() < 0.001);
    assert!((cfg2.overlay.clock_w.position[0] - 0.1).abs() < 0.001);
}

// ── Widget text ──────────────────────────────────────────────────────────────

#[test]
fn clock_text_24h() {
    let mut cfg = OverlayConfig::default();
    cfg.show_clock = true;
    cfg.clock_24h = true;
    cfg.clock_show_seconds = false;
    let t = clock_text(&cfg).expect("clock text");
    assert!(t.contains(':'));
    assert!(!t.contains("AM"), "expected 24h but got AM/PM: {t}");
    assert!(!t.contains("PM"), "expected 24h but got AM/PM: {t}");
}

#[test]
fn clock_text_disabled_returns_none() {
    let mut cfg = OverlayConfig::default();
    cfg.show_clock = false;
    assert!(clock_text(&cfg).is_none());
}

#[test]
fn calendar_text_disabled() {
    let mut cfg = OverlayConfig::default();
    cfg.show_calendar = false;
    assert!(calendar_text(&cfg).is_none());
}

#[test]
fn weather_temp_text_formats_celsius() {
    use oled_wallpaper::config::WeatherConfig;
    let ws = WeatherState {
        weather: Some(WeatherData {
            temperature_c: 22.5,
            condition: WeatherCondition::Clear,
            wind_kmh: 10.0,
            precipitation_mm: 0.0,
        }),
        air: None,
        last_fetch: None,
        error: None,
    };
    let wc = WeatherConfig {
        units_fahrenheit: false,
        ..Default::default()
    };
    let t = weather_temp_text(&ws, &wc).expect("weather text");
    assert!(
        t.contains("23") || t.contains("22"),
        "expected celsius in: {t}"
    );
    assert!(t.contains('C'), "expected C in: {t}");
    assert!(t.contains("Clear"), "expected condition in: {t}");
}

#[test]
fn weather_temp_text_fahrenheit() {
    use oled_wallpaper::config::WeatherConfig;
    let ws = WeatherState {
        weather: Some(WeatherData {
            temperature_c: 0.0,
            condition: WeatherCondition::Snow,
            wind_kmh: 5.0,
            precipitation_mm: 0.0,
        }),
        air: None,
        last_fetch: None,
        error: None,
    };
    let wc = WeatherConfig {
        units_fahrenheit: true,
        ..Default::default()
    };
    let t = weather_temp_text(&ws, &wc).expect("weather text");
    assert!(t.contains("32"), "0C should be 32F in: {t}");
    assert!(t.contains('F'), "expected F in: {t}");
}

#[test]
fn weather_temp_no_data_returns_none() {
    use oled_wallpaper::config::WeatherConfig;
    let ws = WeatherState::default();
    let wc = WeatherConfig::default();
    assert!(weather_temp_text(&ws, &wc).is_none());
}

#[test]
fn wind_text_shows_speed() {
    let ws = WeatherState {
        weather: Some(WeatherData {
            wind_kmh: 25.0,
            ..Default::default()
        }),
        ..Default::default()
    };
    let t = weather_wind_text(&ws).expect("wind text");
    assert!(t.contains("25"), "expected 25 km/h in: {t}");
}

#[test]
fn wind_text_calm_returns_none() {
    let ws = WeatherState {
        weather: Some(WeatherData {
            wind_kmh: 0.3,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(weather_wind_text(&ws).is_none());
}

#[test]
fn aqi_text_shows_level() {
    let ws = WeatherState {
        air: Some(AirQualityData {
            aqi: 45,
            level: AqiLevel::Good,
            pm25: 5.0,
            pm10: 8.0,
        }),
        ..Default::default()
    };
    let t = aqi_text(&ws).expect("aqi text");
    assert!(t.contains("45"), "expected AQI value in: {t}");
    assert!(t.contains("Good"), "expected level in: {t}");
}

// ── Weather icons ────────────────────────────────────────────────────────────

#[test]
fn all_condition_icons_are_non_empty() {
    let conditions = [
        WeatherCondition::Clear,
        WeatherCondition::PartlyCloudy,
        WeatherCondition::Cloudy,
        WeatherCondition::Overcast,
        WeatherCondition::Fog,
        WeatherCondition::Drizzle,
        WeatherCondition::Rain,
        WeatherCondition::HeavyRain,
        WeatherCondition::Thunderstorm,
        WeatherCondition::Snow,
        WeatherCondition::Unknown,
    ];
    for cond in conditions {
        let icon = condition_icon_text(cond.clone());
        assert!(!icon.is_empty(), "icon for {cond:?} is empty");
    }
}

// ── WeatherInfluence ──────────────────────────────────────────────────────────

#[test]
fn rain_increases_meteor_rate() {
    let influence = WeatherInfluence::from_condition(&WeatherCondition::Rain);
    assert!(
        influence.meteor_rate > 1.0,
        "rain should increase meteor rate"
    );
    assert!(influence.rain_meteor_tint, "rain should set tint flag");
}

#[test]
fn storm_enables_thunder_pulses() {
    let influence = WeatherInfluence::from_condition(&WeatherCondition::Thunderstorm);
    assert!(influence.thunder_pulses);
    assert!(influence.meteor_rate > 1.0);
    assert!(influence.cosmic_rate > 1.0);
}

#[test]
fn clear_reduces_meteor_rate() {
    let influence = WeatherInfluence::from_condition(&WeatherCondition::Clear);
    assert!(
        influence.meteor_rate < 1.0,
        "clear sky should calm the meteors"
    );
    assert!(
        influence.star_brightness > 1.0,
        "clear sky should brighten stars"
    );
}

#[test]
fn snow_sets_tint() {
    let influence = WeatherInfluence::from_condition(&WeatherCondition::Snow);
    assert!(influence.snow_tint);
}

#[test]
fn fog_dims_stars() {
    let influence = WeatherInfluence::from_condition(&WeatherCondition::Fog);
    assert!(
        influence.star_brightness < 0.6,
        "fog should dim stars significantly"
    );
}

// ── OverlayWidget defaults ───────────────────────────────────────────────────

#[test]
fn overlay_widget_positions_in_range() {
    let cfg = OverlayConfig::default();
    for pos in [
        cfg.clock_w.position,
        cfg.weather_w.position,
        cfg.wind_w.position,
        cfg.aqi_w.position,
    ] {
        assert!((0.0..=1.0).contains(&pos[0]), "x out of range: {}", pos[0]);
        assert!((0.0..=1.0).contains(&pos[1]), "y out of range: {}", pos[1]);
    }
}

#[test]
fn overlay_widget_font_scales_positive() {
    let cfg = OverlayConfig::default();
    for w in [&cfg.clock_w, &cfg.weather_w, &cfg.wind_w, &cfg.aqi_w] {
        assert!(w.font_scale > 0.0);
    }
}
