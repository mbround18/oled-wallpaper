use tempfile::tempdir;
use oled_wallpaper::config::Config;

#[test]
fn widget_config_serialize_roundtrip() {
    let cfg = Config::default();
    let toml = toml::to_string_pretty(&cfg).expect("serialize");
    let dir = tempdir().unwrap();
    let mut path = dir.path().to_path_buf();
    path.push("config.toml");
    std::fs::write(&path, toml).expect("write");
    let loaded = Config::load_from_file(&path).expect("load");
    assert_eq!(loaded.overlay.show_clock, cfg.overlay.show_clock);
}
