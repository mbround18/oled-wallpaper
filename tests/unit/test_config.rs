use tempfile::tempdir;
use oled_wallpaper::config::Config;

#[test]
fn save_and_load_config_roundtrip() {
    let dir = tempdir().unwrap();
    let mut path = dir.path().to_path_buf();
    path.push("config.toml");

    let cfg = Config::default();
    let toml = toml::to_string_pretty(&cfg).expect("serialize");
    std::fs::write(&path, toml).expect("write");
    let loaded = Config::load_from_file(&path).expect("load failed");
    assert!((loaded.animation.planet_speed - cfg.animation.planet_speed).abs() < 1e-6);
}
