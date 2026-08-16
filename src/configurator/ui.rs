// ─── Space theme palette ─────────────────────────────────────────────────────

pub const VOID: egui::Color32 = egui::Color32::from_rgb(5, 8, 18);
pub const DEEP: egui::Color32 = egui::Color32::from_rgb(10, 15, 30);
pub const CARD: egui::Color32 = egui::Color32::from_rgb(16, 22, 46);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(30, 42, 80);
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(72, 130, 230);
pub const ACCENT_DIM: egui::Color32 = egui::Color32::from_rgb(40, 72, 140);
pub const NEBULA: egui::Color32 = egui::Color32::from_rgb(140, 75, 215);
pub const STAR: egui::Color32 = egui::Color32::from_rgb(215, 228, 255);
pub const DIM: egui::Color32 = egui::Color32::from_rgb(100, 120, 170);
pub const GLOW_GREEN: egui::Color32 = egui::Color32::from_rgb(55, 195, 110);
pub const DANGER: egui::Color32 = egui::Color32::from_rgb(220, 75, 75);

fn space_theme() -> egui::Visuals {
    use egui::epaint::Shadow;
    use egui::style::{WidgetVisuals, Widgets};
    use egui::{Color32, Rounding, Stroke};

    let widget_inactive = WidgetVisuals {
        bg_fill: DEEP,
        weak_bg_fill: DEEP,
        bg_stroke: Stroke::new(1.0_f32, BORDER),
        fg_stroke: Stroke::new(1.0_f32, DIM),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };
    let widget_hovered = WidgetVisuals {
        bg_fill: CARD,
        weak_bg_fill: CARD,
        bg_stroke: Stroke::new(1.5_f32, ACCENT_DIM),
        fg_stroke: Stroke::new(1.5_f32, STAR),
        rounding: Rounding::same(6.0),
        expansion: 1.0,
    };
    let widget_active = WidgetVisuals {
        bg_fill: ACCENT_DIM,
        weak_bg_fill: ACCENT_DIM,
        bg_stroke: Stroke::new(1.5_f32, ACCENT),
        fg_stroke: Stroke::new(2.0_f32, STAR),
        rounding: Rounding::same(6.0),
        expansion: 1.0,
    };
    let widget_open = WidgetVisuals {
        bg_fill: CARD,
        weak_bg_fill: CARD,
        bg_stroke: Stroke::new(1.0_f32, ACCENT_DIM),
        fg_stroke: Stroke::new(1.5_f32, STAR),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };
    let noninteractive = WidgetVisuals {
        bg_fill: DEEP,
        weak_bg_fill: DEEP,
        bg_stroke: Stroke::new(1.0_f32, BORDER),
        fg_stroke: Stroke::new(1.0_f32, DIM),
        rounding: Rounding::same(4.0),
        expansion: 0.0,
    };

    egui::Visuals {
        dark_mode: true,
        override_text_color: Some(STAR),
        window_rounding: Rounding::same(10.0),
        window_shadow: Shadow::big_dark(),
        popup_shadow: Shadow::small_dark(),
        window_fill: DEEP,
        panel_fill: VOID,
        window_stroke: Stroke::new(1.0_f32, BORDER),
        extreme_bg_color: VOID,
        code_bg_color: CARD,
        faint_bg_color: DEEP,
        selection: egui::style::Selection {
            bg_fill: Color32::from_rgba_premultiplied(72, 130, 230, 60),
            stroke: Stroke::new(1.0_f32, ACCENT),
        },
        hyperlink_color: ACCENT,
        widgets: Widgets {
            noninteractive,
            inactive: widget_inactive,
            hovered: widget_hovered,
            active: widget_active,
            open: widget_open,
        },
        ..egui::Visuals::dark()
    }
}

// ─── Tabs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Control,
    Galaxy,
    Widgets,
    Weather,
    System,
}

impl Tab {
    fn all() -> &'static [Tab] {
        &[
            Tab::Control,
            Tab::Galaxy,
            Tab::Widgets,
            Tab::Weather,
            Tab::System,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            Tab::Control => " Control",
            Tab::Galaxy => " Galaxy",
            Tab::Widgets => " Widgets",
            Tab::Weather => " Weather",
            Tab::System => " System",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Tab::Control => "◉",
            Tab::Galaxy => "✦",
            Tab::Widgets => "◷",
            Tab::Weather => "◌",
            Tab::System => "⊞",
        }
    }
}

// ─── App state ────────────────────────────────────────────────────────────────

use crate::config::{Config, WeatherProvider};
use crate::runtime::{autostart_enabled, set_autostart_enabled, wallpaper_status};
use eframe::egui;
use egui::plot::{Line, Plot};
use std::collections::VecDeque;
use std::process::{Command, Stdio};
use sysinfo::CpuExt;
use sysinfo::{System, SystemExt};

// ─── Geo-locate state ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub enum LocateStatus {
    #[default]
    Idle,
    Working,
    Found {
        lat: f32,
        lon: f32,
        label: String,
    },
    Failed(String),
}

/// Spawn a thread that tries two free IP-geolocation APIs and writes the result
/// back into `state`. The caller should `request_repaint()` after the thread
/// writes, which happens automatically since we call it on the next repaint.
fn spawn_locate(state: std::sync::Arc<std::sync::Mutex<LocateStatus>>) {
    *state.lock().unwrap() = LocateStatus::Working;
    std::thread::spawn(move || {
        let result = try_locate();
        *state.lock().unwrap() = result;
    });
}

/// Try ipinfo.io first, fall back to ip-api.com.
fn try_locate() -> LocateStatus {
    // ── ipinfo.io ───────────────────────────────────────────────────────
    if let Ok(resp) = ureq::get("https://ipinfo.io/json")
        .timeout(std::time::Duration::from_secs(6))
        .call()
    {
        if let Ok(body) = resp.into_json::<serde_json::Value>() {
            if let Some(loc) = body["loc"].as_str() {
                let parts: Vec<&str> = loc.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(lat), Ok(lon)) = (
                        parts[0].trim().parse::<f32>(),
                        parts[1].trim().parse::<f32>(),
                    ) {
                        let city = body["city"].as_str().unwrap_or("");
                        let country = body["country"].as_str().unwrap_or("");
                        let label = if city.is_empty() {
                            format!("{lat:.4}, {lon:.4}")
                        } else {
                            format!("{city}, {country}")
                        };
                        return LocateStatus::Found { lat, lon, label };
                    }
                }
            }
        }
    }

    // ── ip-api.com fallback ──────────────────────────────────────────────
    if let Ok(resp) = ureq::get("http://ip-api.com/json")
        .timeout(std::time::Duration::from_secs(6))
        .call()
    {
        if let Ok(body) = resp.into_json::<serde_json::Value>() {
            if body["status"].as_str() == Some("success") {
                if let (Some(lat), Some(lon)) = (
                    body["lat"].as_f64().map(|v| v as f32),
                    body["lon"].as_f64().map(|v| v as f32),
                ) {
                    let city = body["city"].as_str().unwrap_or("");
                    let country = body["countryCode"].as_str().unwrap_or("");
                    let label = if city.is_empty() {
                        format!("{lat:.4}, {lon:.4}")
                    } else {
                        format!("{city}, {country}")
                    };
                    return LocateStatus::Found { lat, lon, label };
                }
            }
        }
    }

    LocateStatus::Failed("Could not determine location via IP geolocation".to_string())
}

pub struct ConfiguratorApp {
    pub cfg: Config,
    tab: Tab,
    planet_speed_input: f32,
    camera_zoom_input: f32,
    sys: System,
    cpu_history: VecDeque<f64>,
    mem_history: VecDeque<f64>,
    history_len: usize,
    wallpaper_running: bool,
    wallpaper_pid: Option<u32>,
    startup_enabled: bool,
    save_message: Option<(String, f64)>, // (text, timestamp)
    theme_applied: bool,
    locate_status: std::sync::Arc<std::sync::Mutex<LocateStatus>>,
}

impl Default for ConfiguratorApp {
    fn default() -> Self {
        let cfg = Config::load();
        let mut sys = System::new_all();
        sys.refresh_cpu();
        sys.refresh_memory();
        let history_len = 120;
        Self {
            planet_speed_input: cfg.animation.planet_speed,
            camera_zoom_input: cfg.animation.camera_zoom,
            cfg,
            tab: Tab::Control,
            sys,
            cpu_history: VecDeque::with_capacity(history_len),
            mem_history: VecDeque::with_capacity(history_len),
            history_len,
            wallpaper_running: false,
            wallpaper_pid: None,
            startup_enabled: autostart_enabled(),
            save_message: None,
            theme_applied: false,
            locate_status: Default::default(),
        }
    }
}

impl ConfiguratorApp {
    pub fn clamp_zoom(v: f32) -> f32 {
        v.clamp(0.1, 5.0)
    }

    fn sample_system(&mut self) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        let cpu = self.sys.global_cpu_info().cpu_usage() as f64;
        let total = self.sys.total_memory() as f64;
        let used = if total > 0.0 {
            (self.sys.used_memory() as f64) / total * 100.0
        } else {
            0.0
        };
        if self.cpu_history.len() == self.history_len {
            self.cpu_history.pop_front();
        }
        if self.mem_history.len() == self.history_len {
            self.mem_history.pop_front();
        }
        self.cpu_history.push_back(cpu);
        self.mem_history.push_back(used);
    }

    fn save(&mut self) {
        self.cfg.animation.planet_speed = self.planet_speed_input;
        self.cfg.animation.camera_zoom = Self::clamp_zoom(self.camera_zoom_input);
        match self.cfg.save() {
            Ok(()) => {
                tracing::info!("Config saved");
            }
            Err(e) => {
                tracing::error!("Save failed: {e}");
            }
        }
    }
}

// ─── Tab panels ──────────────────────────────────────────────────────────────

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(text).color(ACCENT).size(13.0).strong());
    ui.add_space(2.0);
}

fn dim_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(DIM).small());
}

fn row_label(ui: &mut egui::Ui, label: &str) {
    ui.label(egui::RichText::new(label).color(STAR));
}

fn labeled_slider(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(STAR).monospace());
        ui.add(egui::Slider::new(val, range).show_value(true));
    });
}

// ── Control tab ──────────────────────────────────────────────────────────────

fn tab_control(app: &mut ConfiguratorApp, ui: &mut egui::Ui) {
    let is_running = app.wallpaper_running;

    // Status card
    let status_color = if is_running { GLOW_GREEN } else { DIM };
    let status_dot = if is_running {
        "● RUNNING"
    } else {
        "○ STOPPED"
    };
    let pid_str = app
        .wallpaper_pid
        .map(|p| format!("  pid {p}"))
        .unwrap_or_default();

    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(status_dot)
                        .color(status_color)
                        .strong()
                        .size(15.0),
                );
                ui.label(egui::RichText::new(&pid_str).color(DIM).small());
            });
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let start = ui.add_enabled(
                    !is_running,
                    egui::Button::new(
                        egui::RichText::new("  Start Wallpaper  ").color(if !is_running {
                            STAR
                        } else {
                            DIM
                        }),
                    )
                    .fill(if !is_running { ACCENT_DIM } else { DEEP }),
                );
                if start.clicked() {
                    match Command::new("oled-wallpaper")
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                    {
                        Ok(_) => app.save_message = Some(("Wallpaper launched".into(), 0.0)),
                        Err(e) => app.save_message = Some((format!("Launch failed: {e}"), 0.0)),
                    }
                }

                let stop = ui.add_enabled(
                    is_running && app.wallpaper_pid.is_some(),
                    egui::Button::new(egui::RichText::new("  Stop  ").color(if is_running {
                        DANGER
                    } else {
                        DIM
                    }))
                    .fill(DEEP)
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        if is_running { DANGER } else { BORDER },
                    )),
                );
                if stop.clicked() {
                    if let Some(pid) = app.wallpaper_pid {
                        let _ = Command::new("kill")
                            .arg("-TERM")
                            .arg(pid.to_string())
                            .status();
                        app.save_message = Some((format!("Sent SIGTERM to pid {pid}"), 0.0));
                    }
                }
            });

            ui.add_space(8.0);
            let mut startup = app.startup_enabled;
            if ui
                .checkbox(
                    &mut startup,
                    egui::RichText::new("Start automatically at login"),
                )
                .changed()
            {
                match set_autostart_enabled(startup) {
                    Ok(()) => {
                        app.startup_enabled = startup;
                        app.save_message = Some((
                            if startup {
                                "Autostart enabled".into()
                            } else {
                                "Autostart disabled".into()
                            },
                            0.0,
                        ));
                    }
                    Err(e) => app.save_message = Some((format!("Autostart error: {e}"), 0.0)),
                }
            }
        });

    ui.add_space(16.0);
    section_heading(ui, "ABOUT");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("OLED Wallpaper")
                    .size(18.0)
                    .color(STAR)
                    .strong(),
            );
            ui.label(egui::RichText::new("v0.1.0").color(DIM).small());
            ui.add_space(4.0);
            dim_label(ui, "Interactive galaxy wallpaper for OLED displays.");
            dim_label(
                ui,
                "Binary stars · Kepler orbits · Meteor showers · Weather effects",
            );
        });
}

// ── Galaxy tab ───────────────────────────────────────────────────────────────

fn tab_galaxy(app: &mut ConfiguratorApp, ui: &mut egui::Ui) {
    section_heading(ui, "SIMULATION");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            labeled_slider(ui, "Planet Speed ", &mut app.planet_speed_input, 0.1..=5.0);
            dim_label(ui, "Orbital period multiplier (1.0 = realistic)");
            ui.add_space(6.0);
            labeled_slider(ui, "Camera Zoom  ", &mut app.camera_zoom_input, 0.1..=5.0);
            dim_label(ui, "Higher = tighter view (default 1.0)");
            ui.add_space(6.0);
            labeled_slider(
                ui,
                "Pulse Intensity",
                &mut app.cfg.animation.pulse_intensity,
                0.1..=2.0,
            );
        });

    ui.add_space(12.0);
    section_heading(ui, "BINARY STARS");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                row_label(ui, "Star A color  ");
                ui.color_edit_button_rgba_unmultiplied(&mut app.cfg.animation.star_a_color);
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                row_label(ui, "Star B color  ");
                ui.color_edit_button_rgba_unmultiplied(&mut app.cfg.animation.star_b_color);
            });
        });
}

// ── Widgets tab ──────────────────────────────────────────────────────────────

fn tab_widgets(app: &mut ConfiguratorApp, ui: &mut egui::Ui, t: f32) {
    ui.checkbox(
        &mut app.cfg.overlay.widget_enabled,
        egui::RichText::new("Enable overlay widgets").color(STAR),
    );
    ui.add_space(6.0);

    ui.set_enabled(app.cfg.overlay.widget_enabled);

    section_heading(ui, "CLOCK & CALENDAR");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].checkbox(&mut app.cfg.overlay.show_clock, "Show clock");
                cols[0].checkbox(&mut app.cfg.overlay.clock_24h, "24h format");
                cols[0].checkbox(&mut app.cfg.overlay.clock_show_seconds, "Show seconds");
                cols[1].checkbox(&mut app.cfg.overlay.show_calendar, "Show calendar");
                cols[1].checkbox(&mut app.cfg.overlay.calendar_month_view, "Month view");
            });
        });

    ui.add_space(10.0);
    section_heading(ui, "POSITION & MOTION");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.checkbox(&mut app.cfg.overlay.widget_float_mode, "Float (drift) mode");
            if app.cfg.overlay.widget_float_mode {
                labeled_slider(
                    ui,
                    "Drift speed ",
                    &mut app.cfg.overlay.widget_float_speed,
                    0.01..=2.0,
                );
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                row_label(ui, "Position X");
                ui.add(egui::Slider::new(
                    &mut app.cfg.overlay.widget_position[0],
                    0.0..=1.0,
                ));
            });
            ui.horizontal(|ui| {
                row_label(ui, "Position Y");
                ui.add(egui::Slider::new(
                    &mut app.cfg.overlay.widget_position[1],
                    0.0..=1.0,
                ));
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                row_label(ui, "Text color  ");
                ui.color_edit_button_rgba_unmultiplied(&mut app.cfg.overlay.widget_color);
            });
            labeled_slider(
                ui,
                "Font scale  ",
                &mut app.cfg.overlay.widget_font_scale,
                0.5..=3.0,
            );
        });

    ui.add_space(10.0);
    section_heading(ui, "PREVIEW");
    let preview_h = 180.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), preview_h),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    // Starfield background
    painter.rect_filled(rect, 6.0, VOID);
    painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0_f32, BORDER));
    // Tiny stars
    for i in 0..60u32 {
        let seed = i.wrapping_mul(2654435761);
        let sx = (seed & 0xFFFF) as f32 / 65535.0;
        let sy = ((seed >> 16) & 0xFFFF) as f32 / 65535.0;
        let px = rect.left() + sx * rect.width();
        let py = rect.top() + sy * rect.height();
        let bright = 0.2 + 0.6 * (seed & 0xFF) as f32 / 255.0;
        let twinkle = 0.7 + 0.3 * (t * (1.0 + bright) + sx * 6.0).sin();
        let alpha = (bright * twinkle * 200.0) as u8;
        painter.circle_filled(
            egui::pos2(px, py),
            1.2,
            egui::Color32::from_rgba_premultiplied(200, 210, 255, alpha),
        );
    }
    // Widget dot at configured position
    let float_offset = if app.cfg.overlay.widget_float_mode {
        let s = app.cfg.overlay.widget_float_speed;
        egui::vec2((t * s).sin() * 12.0, (t * s * 1.37).cos() * 8.0)
    } else {
        egui::vec2(0.0, 0.0)
    };
    let wx = rect.left() + app.cfg.overlay.widget_position[0] * rect.width();
    let wy = rect.top() + app.cfg.overlay.widget_position[1] * rect.height();
    let wpos = egui::pos2(wx, wy) + float_offset;
    let [r, g, b, a] = app.cfg.overlay.widget_color;
    let wcol = egui::Color32::from_rgba_unmultiplied(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    );
    painter.circle_filled(wpos, 14.0, wcol.linear_multiply(0.3));
    painter.circle_stroke(wpos, 14.0, egui::Stroke::new(1.5_f32, wcol));
    let galactic = egui::FontId::proportional(11.0);
    painter.text(
        wpos + egui::vec2(0.0, -26.0),
        egui::Align2::CENTER_CENTER,
        "12:34:56",
        galactic,
        wcol,
    );
}

// ── Weather tab ──────────────────────────────────────────────────────────────

fn tab_weather(app: &mut ConfiguratorApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut app.cfg.weather.enabled,
            egui::RichText::new("Enable weather widget").color(STAR),
        );
        if app.cfg.weather.enabled {
            ui.label(
                egui::RichText::new("  (fetches on wallpaper start)")
                    .color(DIM)
                    .small(),
            );
        }
    });
    ui.add_space(6.0);
    ui.set_enabled(app.cfg.weather.enabled);

    section_heading(ui, "PROVIDER");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            let is_meteo = app.cfg.weather.provider == WeatherProvider::OpenMeteo;
            ui.horizontal(|ui| {
                if ui
                    .radio(is_meteo, egui::RichText::new("Open-Meteo").color(STAR))
                    .clicked()
                {
                    app.cfg.weather.provider = WeatherProvider::OpenMeteo;
                }
                if ui
                    .radio(!is_meteo, egui::RichText::new("OpenWeatherMap").color(STAR))
                    .clicked()
                {
                    app.cfg.weather.provider = WeatherProvider::OpenWeatherMap;
                }
            });
            if is_meteo {
                dim_label(ui, "Free · no API key · open-meteo.com");
            } else {
                ui.add_space(6.0);
                ui.label(egui::RichText::new("API Key").color(DIM).small());
                ui.add(
                    egui::TextEdit::singleline(&mut app.cfg.weather.api_key)
                        .hint_text("paste key from openweathermap.org")
                        .desired_width(f32::INFINITY)
                        .password(false),
                );
                dim_label(ui, "Free tier key from openweathermap.org → API keys");
            }
        });

    ui.add_space(10.0);
    section_heading(ui, "LOCATION");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            // Lat/Lon drag values
            ui.columns(2, |cols| {
                cols[0].label(egui::RichText::new("Latitude").color(DIM).small());
                cols[0].add(
                    egui::DragValue::new(&mut app.cfg.weather.latitude)
                        .speed(0.01)
                        .clamp_range(-90.0f32..=90.0),
                );
                cols[0].add_space(2.0);
                cols[0].label(
                    egui::RichText::new("e.g.  37.77  (San Francisco)")
                        .color(DIM)
                        .small(),
                );

                cols[1].label(egui::RichText::new("Longitude").color(DIM).small());
                cols[1].add(
                    egui::DragValue::new(&mut app.cfg.weather.longitude)
                        .speed(0.01)
                        .clamp_range(-180.0f32..=180.0),
                );
                cols[1].add_space(2.0);
                cols[1].label(egui::RichText::new("e.g.  -122.41").color(DIM).small());
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Read locate status (lock is brief, clone what we need)
            let status_snap = app.locate_status.lock().unwrap().clone();

            match &status_snap {
                LocateStatus::Idle => {
                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("  ⌖  Detect My Location  ")
                                .color(STAR)
                                .strong(),
                        )
                        .fill(ACCENT_DIM)
                        .stroke(egui::Stroke::new(1.0_f32, ACCENT)),
                    );
                    if btn.clicked() {
                        spawn_locate(app.locate_status.clone());
                    }
                    dim_label(ui, "Uses your external IP address via ipinfo.io → ip-api.com fallback");
                }

                LocateStatus::Working => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("Locating via IP geolocation…").color(DIM),
                        );
                    });
                    // Keep repainting until done
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));
                }

                LocateStatus::Found { lat, lon, label } => {
                    let lat = *lat;
                    let lon = *lon;
                    let label = label.clone();
                    egui::Frame::none()
                        .fill(DEEP)
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::style::Margin::same(10.0))
                        .stroke(egui::Stroke::new(1.0_f32, GLOW_GREEN))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("◉")
                                        .color(GLOW_GREEN)
                                        .strong(),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{label}  ({lat:.4}, {lon:.4})"
                                    ))
                                    .color(STAR),
                                );
                            });
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                let apply_btn = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new("  Apply  ")
                                            .color(STAR)
                                            .strong(),
                                    )
                                    .fill(ACCENT_DIM)
                                    .stroke(egui::Stroke::new(1.0_f32, ACCENT)),
                                );
                                if apply_btn.clicked() {
                                    app.cfg.weather.latitude = lat;
                                    app.cfg.weather.longitude = lon;
                                    *app.locate_status.lock().unwrap() = LocateStatus::Idle;
                                    app.save_message =
                                        Some((format!("Location set to {label}"), 0.0));
                                }

                                if ui
                                    .button(egui::RichText::new("Retry").color(DIM))
                                    .clicked()
                                {
                                    spawn_locate(app.locate_status.clone());
                                }
                            });
                        });
                }

                LocateStatus::Failed(msg) => {
                    let msg = msg.clone();
                    egui::Frame::none()
                        .fill(DEEP)
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::style::Margin::same(10.0))
                        .stroke(egui::Stroke::new(1.0_f32, DANGER))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("✗  {msg}"))
                                    .color(DANGER)
                                    .small(),
                            );
                            ui.add_space(4.0);
                            if ui
                                .button(egui::RichText::new("  Retry  ").color(STAR))
                                .clicked()
                            {
                                spawn_locate(app.locate_status.clone());
                            }
                        });
                }
            }
        });

    ui.add_space(10.0);
    section_heading(ui, "DISPLAY");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].checkbox(&mut app.cfg.weather.units_fahrenheit, "Fahrenheit");
                cols[0].checkbox(&mut app.cfg.weather.show_aqi, "Show Air Quality Index");
                cols[1].checkbox(&mut app.cfg.weather.affect_galaxy, "Weather affects galaxy");
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                row_label(ui, "Refresh every");
                ui.add(
                    egui::DragValue::new(&mut app.cfg.weather.refresh_minutes)
                        .speed(1)
                        .clamp_range(5u32..=120),
                );
                row_label(ui, "minutes");
            });
        });

    ui.add_space(10.0);
    section_heading(ui, "GALAXY EFFECTS");
    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            let effects = [
                ("☀ Clear", "Fewer meteors, brighter stars"),
                ("☁ Cloudy/Fog", "Dimmer stars, fewer meteors"),
                ("~· Drizzle", "1.8× meteors, ice-blue tint"),
                ("// Rain", "2.8× meteors, ice-blue tint"),
                ("/// Heavy Rain", "4× meteors + cosmic rays"),
                ("⚡ Storm", "3.5× meteors, random thunder pulses"),
                ("* Snow", "1.5× white meteors"),
            ];
            for (cond, effect) in &effects {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(*cond).color(ACCENT).monospace());
                    ui.label(
                        egui::RichText::new(format!("→ {effect}"))
                            .color(DIM)
                            .small(),
                    );
                });
            }
        });
}

// ── System tab ───────────────────────────────────────────────────────────────

fn tab_system(app: &mut ConfiguratorApp, ui: &mut egui::Ui) {
    section_heading(ui, "LIVE SYSTEM");

    let cpu_now = app.cpu_history.back().copied().unwrap_or(0.0);
    let mem_now = app.mem_history.back().copied().unwrap_or(0.0);

    egui::Frame::none()
        .fill(CARD)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::style::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].label(egui::RichText::new("CPU").color(DIM).small());
                cols[0].label(
                    egui::RichText::new(format!("{cpu_now:.1}%"))
                        .size(26.0)
                        .color(ACCENT)
                        .strong(),
                );
                cols[1].label(egui::RichText::new("Memory").color(DIM).small());
                cols[1].label(
                    egui::RichText::new(format!("{mem_now:.1}%"))
                        .size(26.0)
                        .color(NEBULA)
                        .strong(),
                );
            });
        });

    ui.add_space(10.0);
    section_heading(ui, "USAGE HISTORY  (last 2 min)");

    let cpu_vec: Vec<[f64; 2]> = app
        .cpu_history
        .iter()
        .enumerate()
        .map(|(i, v)| [i as f64, *v])
        .collect();
    let mem_vec: Vec<[f64; 2]> = app
        .mem_history
        .iter()
        .enumerate()
        .map(|(i, v)| [i as f64, *v])
        .collect();

    let plot_h = (ui.available_height() - 80.0).max(140.0);
    Plot::new("sys_plot")
        .height(plot_h)
        .include_y(0.0)
        .include_y(100.0)
        .show_axes([false, true])
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(cpu_vec)
                    .name("CPU %")
                    .color(egui::Color32::from_rgb(72, 130, 230)),
            );
            plot_ui.line(
                Line::new(mem_vec)
                    .name("RAM %")
                    .color(egui::Color32::from_rgb(140, 75, 215)),
            );
        });

    dim_label(
        ui,
        "Note: system metric access may be limited inside Flatpak sandbox.",
    );
}

// ─── eframe App impl ─────────────────────────────────────────────────────────

impl eframe::App for ConfiguratorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme once (first frame)
        if !self.theme_applied {
            ctx.set_visuals(space_theme());
            self.theme_applied = true;
        }

        self.sample_system();
        let status = wallpaper_status();
        self.wallpaper_running = status.running;
        self.wallpaper_pid = status.pid;
        self.startup_enabled = autostart_enabled();

        let t = ctx.input(|i| i.time) as f32;

        // ── Top bar ──────────────────────────────────────────────────────
        egui::TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::none()
                    .fill(DEEP)
                    .inner_margin(egui::style::Margin {
                        left: 16.0,
                        right: 16.0,
                        top: 10.0,
                        bottom: 10.0,
                    })
                    .stroke(egui::Stroke::new(1.0_f32, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("✦ OLED WALLPAPER")
                            .size(16.0)
                            .color(ACCENT)
                            .strong()
                            .monospace(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let run_col = if self.wallpaper_running {
                            GLOW_GREEN
                        } else {
                            DANGER
                        };
                        let run_text = if self.wallpaper_running {
                            "● LIVE"
                        } else {
                            "○ OFF"
                        };
                        ui.label(
                            egui::RichText::new(run_text)
                                .color(run_col)
                                .small()
                                .strong(),
                        );
                    });
                });
            });

        // ── Tab bar ──────────────────────────────────────────────────────
        egui::TopBottomPanel::top("tab_bar")
            .frame(
                egui::Frame::none()
                    .fill(DEEP)
                    .inner_margin(egui::style::Margin {
                        left: 8.0,
                        right: 8.0,
                        top: 6.0,
                        bottom: 0.0,
                    })
                    .stroke(egui::Stroke::new(1.0_f32, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for tab in Tab::all() {
                        let selected = self.tab == *tab;
                        let label = format!("{}  {}", tab.icon(), tab.label());
                        let text = egui::RichText::new(&label)
                            .color(if selected { ACCENT } else { DIM })
                            .size(12.5);

                        let btn = egui::Button::new(text)
                            .fill(if selected {
                                CARD
                            } else {
                                egui::Color32::TRANSPARENT
                            })
                            .stroke(egui::Stroke::new(
                                if selected { 2.0_f32 } else { 0.0_f32 },
                                if selected {
                                    ACCENT
                                } else {
                                    egui::Color32::TRANSPARENT
                                },
                            ))
                            .rounding(egui::Rounding {
                                nw: 6.0,
                                ne: 6.0,
                                sw: 0.0,
                                se: 0.0,
                            });

                        if ui.add(btn).clicked() {
                            self.tab = *tab;
                        }
                        ui.add_space(2.0);
                    }
                });
            });

        // ── Bottom bar ───────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(
                egui::Frame::none()
                    .fill(DEEP)
                    .inner_margin(egui::style::Margin::same(10.0))
                    .stroke(egui::Stroke::new(1.0_f32, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let save_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("  Save Config  ").color(STAR).strong(),
                        )
                        .fill(ACCENT_DIM)
                        .stroke(egui::Stroke::new(1.0_f32, ACCENT)),
                    );
                    if save_btn.clicked() {
                        self.save();
                        self.save_message = Some(("Config saved.".into(), t as f64));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some((msg, ts)) = &self.save_message {
                            let age = t as f64 - ts;
                            if age < 4.0 {
                                let alpha = (1.0 - (age / 4.0)).max(0.0) as f32;
                                let col = egui::Color32::from_rgba_unmultiplied(
                                    200,
                                    230,
                                    255,
                                    (alpha * 200.0) as u8,
                                );
                                ui.label(egui::RichText::new(msg.as_str()).color(col).small());
                            }
                        }
                    });
                });
            });

        // ── Content ──────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(VOID)
                    .inner_margin(egui::style::Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(14.0);
                        let inner_width = ui.available_width() - 32.0;
                        ui.allocate_ui_with_layout(
                            egui::vec2(inner_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(inner_width);
                                ui.add_space(0.0);
                                // offset to center the content
                                ui.horizontal(|ui| {
                                    ui.add_space(16.0);
                                    ui.vertical(|ui| {
                                        match self.tab {
                                            Tab::Control => tab_control(self, ui),
                                            Tab::Galaxy => tab_galaxy(self, ui),
                                            Tab::Widgets => tab_widgets(self, ui, t),
                                            Tab::Weather => tab_weather(self, ui),
                                            Tab::System => tab_system(self, ui),
                                        }
                                        ui.add_space(20.0);
                                    });
                                });
                            },
                        );
                    });
            });

        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}
