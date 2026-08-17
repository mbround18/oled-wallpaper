use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preset {
    pub name: String,
    pub config: Config,
}

pub fn export_preset<P: AsRef<Path>>(path: P, preset: &Preset) -> Result<(), anyhow::Error> {
    let p = path.as_ref();
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = p.with_extension("toml.tmp");
    let mut f = fs::File::create(&tmp)?;
    let s = toml::to_string_pretty(preset)?;
    f.write_all(s.as_bytes())?;
    f.flush()?;
    drop(f);
    fs::rename(&tmp, p)?;
    Ok(())
}

pub fn import_preset<P: AsRef<Path>>(path: P) -> Result<Preset, anyhow::Error> {
    let p = path.as_ref();
    let s = fs::read_to_string(p)?;
    let preset: Preset = toml::from_str(&s)?;
    Ok(preset)
}

pub fn apply_preset_to_config(preset: &Preset) -> Config {
    preset.config.clone()
}
