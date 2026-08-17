//! Preset export/import round-trip tests.
//!
//! Migrated from tests/unit/test_presets.rs, which cargo never discovered or
//! ran: cargo only auto-discovers test binaries directly under tests/, not in
//! subdirectories, unless explicitly wired via Cargo.toml or a mod include.

use oled_wallpaper::config::presets::{export_preset, import_preset, Preset};
use oled_wallpaper::config::Config;
use tempfile::tempdir;

#[test]
fn preset_export_import_roundtrip() {
    let dir = tempdir().unwrap();
    let mut path = dir.path().to_path_buf();
    path.push("preset.toml");

    let cfg = Config::default();
    let preset = Preset {
        name: "test".to_string(),
        config: cfg,
    };
    export_preset(&path, &preset).expect("export failed");
    let imported = import_preset(&path).expect("import failed");
    assert_eq!(imported.name, "test");
    assert_eq!(
        imported.config.animation.planet_speed,
        preset.config.animation.planet_speed
    );
}
