use clap::Parser;
use std::path::PathBuf;

use oled_wallpaper::config::presets::{
    apply_preset_to_config, export_preset, import_preset, Preset,
};
use oled_wallpaper::config::Config;
use oled_wallpaper::configurator::ConfiguratorApp;
use oled_wallpaper::runtime::set_autostart_enabled;

#[derive(Parser)]
#[command(name = "oled-config")]
struct Args {
    #[arg(long)]
    headless: bool,
    #[arg(long)]
    apply: Option<String>,
    #[arg(long, value_name = "DIR")]
    config_dir: Option<PathBuf>,
    /// Enable or disable "start at login" without launching the GUI. Accepts "on"/"off".
    #[arg(long, value_name = "on|off")]
    set_autostart: Option<String>,
    /// Write the current default config out as a named preset file.
    #[arg(long, value_name = "PATH")]
    export_preset: Option<PathBuf>,
    /// Read a preset file and apply it to the config at --config-dir (or the default path).
    #[arg(long, value_name = "PATH")]
    import_preset: Option<PathBuf>,
    /// Name to embed when exporting a preset (only used with --export-preset).
    #[arg(long, value_name = "NAME", default_value = "headless")]
    preset_name: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.headless {
        if let Some(mode) = args.set_autostart.as_deref() {
            let enabled = match mode {
                "on" => true,
                "off" => false,
                other => anyhow::bail!("--set-autostart expects \"on\" or \"off\", got {other:?}"),
            };
            set_autostart_enabled(enabled)?;
            println!("Autostart {}", if enabled { "enabled" } else { "disabled" });
            return Ok(());
        }

        let config_path = args.config_dir.unwrap_or_else(|| {
            let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            p.push("oled-wallpaper");
            p.push("config.toml");
            p
        });

        if let Some(preset_path) = args.export_preset {
            let mut cfg = Config::default();
            cfg.animation.planet_speed = 2.0; // distinguishable, non-default value for round-trip checks
            let preset = Preset {
                name: args.preset_name,
                config: cfg,
            };
            export_preset(&preset_path, &preset)?;
            println!("Exported preset to {}", preset_path.display());
            return Ok(());
        }

        if let Some(preset_path) = args.import_preset {
            let preset = import_preset(&preset_path)?;
            let cfg = apply_preset_to_config(&preset);
            write_config(&config_path, &cfg)?;
            println!(
                "Imported preset {:?} from {} to {}",
                preset.name,
                preset_path.display(),
                config_path.display()
            );
            return Ok(());
        }

        let mut cfg = Config::default();
        if let Some(preset) = args.apply {
            match preset.as_str() {
                "widget-off" => {
                    cfg.overlay.show_clock = false;
                    cfg.overlay.widget_enabled = false;
                }
                "widget-on" => {
                    cfg.overlay.show_clock = true;
                    cfg.overlay.widget_enabled = true;
                }
                _ => {
                    cfg.animation.planet_speed = 2.0;
                }
            }
        }
        write_config(&config_path, &cfg)?;
        println!("Wrote config to {}", config_path.display());
        return Ok(());
    }

    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(860.0, 620.0)),
        min_window_size: Some(eframe::egui::vec2(480.0, 400.0)),
        resizable: true,
        ..Default::default()
    };
    let _ = eframe::run_native(
        "OLED Wallpaper  ·  Control Center",
        options,
        Box::new(|_cc| Box::new(ConfiguratorApp::default())),
    );
    Ok(())
}

fn write_config(config_path: &std::path::Path, cfg: &Config) -> anyhow::Result<()> {
    if let Some(dir) = config_path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = config_path.with_extension("toml.tmp");
    let toml = toml::to_string_pretty(cfg)?;
    let mut f = std::fs::File::create(&tmp)?;
    use std::io::Write as _;
    f.write_all(toml.as_bytes())?;
    f.flush()?;
    drop(f);
    std::fs::rename(&tmp, config_path)?;
    Ok(())
}
