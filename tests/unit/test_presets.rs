use tempfile::tempdir;
use oled_wallpaper::config::Config;
use oled_wallpaper::config::presets::{Preset, export_preset, import_preset};

#[test]
fn preset_export_import_roundtrip() {
    let dir = tempdir().unwrap();
    let mut path = dir.path().to_path_buf();
    path.push("preset.toml");

    let cfg = Config::default();
    let preset = Preset { name: "test".to_string(), config: cfg };
    export_preset(&path, &preset).expect("export failed");
    let imported = import_preset(&path).expect("import failed");
    assert_eq!(imported.name, "test");
    assert_eq!(imported.config.animation.planet_speed, preset.config.animation.planet_speed);
}
