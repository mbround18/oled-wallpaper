use clap::Parser;
use std::path::PathBuf;

use oled_wallpaper::config::Config;
use oled_wallpaper::configurator::ConfiguratorApp;

#[derive(Parser)]
#[command(name = "oled-config")]
struct Args {
    #[arg(long)]
    headless: bool,
    #[arg(long)]
    apply: Option<String>,
    #[arg(long, value_name = "DIR")]
    config_dir: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.headless {
        let config_path = args.config_dir.unwrap_or_else(|| {
            let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            p.push("oled-wallpaper");
            p.push("config.toml");
            p
        });
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
        if let Some(dir) = config_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = config_path.with_extension("toml.tmp");
        let toml = toml::to_string_pretty(&cfg)?;
        let mut f = std::fs::File::create(&tmp)?;
        use std::io::Write as _;
        f.write_all(toml.as_bytes())?;
        f.flush()?;
        drop(f);
        std::fs::rename(&tmp, &config_path)?;
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
