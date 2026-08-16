use crate::config::Config;
use crate::runtime::{autostart_enabled, set_autostart_enabled, wallpaper_status};
use eframe::egui;
use egui::plot::{Line, Plot};
use std::collections::VecDeque;
use std::process::{Command, Stdio};
use sysinfo::CpuExt;
use sysinfo::{System, SystemExt};

pub struct ConfiguratorApp {
    pub cfg: Config,
    // transient UI state
    planet_speed_input: f32,
    camera_zoom_input: f32,
    // usage stats
    sys: System,
    cpu_history: VecDeque<f64>,
    mem_history: VecDeque<f64>,
    history_len: usize,
    wallpaper_running: bool,
    wallpaper_pid: Option<u32>,
    startup_enabled: bool,
    runtime_message: Option<String>,
}

impl Default for ConfiguratorApp {
    fn default() -> Self {
        let cfg = Config::load();
        let mut sys = System::new_all();
        sys.refresh_cpu();
        sys.refresh_memory();
        let history_len = 120usize; // keep last 120 samples (~2 minutes at 1s)
        Self {
            planet_speed_input: cfg.animation.planet_speed,
            camera_zoom_input: cfg.animation.camera_zoom,
            cfg,
            sys,
            cpu_history: VecDeque::with_capacity(history_len),
            mem_history: VecDeque::with_capacity(history_len),
            history_len,
            wallpaper_running: false,
            wallpaper_pid: None,
            startup_enabled: autostart_enabled(),
            runtime_message: None,
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
        // Use global CPU info for a single percentage reading
        let cpu = self.sys.global_cpu_info().cpu_usage() as f64;
        let total = self.sys.total_memory() as f64;
        let used = (self.sys.used_memory() as f64) / total * 100.0;
        if self.cpu_history.len() == self.history_len {
            self.cpu_history.pop_front();
        }
        if self.mem_history.len() == self.history_len {
            self.mem_history.pop_front();
        }
        self.cpu_history.push_back(cpu);
        self.mem_history.push_back(used);
    }
}

impl eframe::App for ConfiguratorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // update samples once per second by tying to frame time; keep it simple
        self.sample_system();
        let status = wallpaper_status();
        self.wallpaper_running = status.running;
        self.wallpaper_pid = status.pid;
        self.startup_enabled = autostart_enabled();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("OLED Wallpaper Configurator");
            ui.group(|ui| {
                ui.heading("Runtime");
                let state_text = if self.wallpaper_running {
                    if let Some(pid) = self.wallpaper_pid {
                        format!("Running (pid {pid})")
                    } else {
                        "Running".to_string()
                    }
                } else {
                    "Stopped".to_string()
                };
                ui.label(format!("Wallpaper status: {state_text}"));

                ui.horizontal(|ui| {
                    let start_btn = ui.add_enabled(!self.wallpaper_running, egui::Button::new("Start wallpaper"));
                    if start_btn.clicked() {
                        match Command::new("oled-wallpaper")
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                        {
                            Ok(_) => self.runtime_message = Some("Started wallpaper".to_string()),
                            Err(e) => self.runtime_message = Some(format!("Failed to start wallpaper: {e}")),
                        }
                    }

                    let stop_btn = ui.add_enabled(
                        self.wallpaper_running && self.wallpaper_pid.is_some(),
                        egui::Button::new("Stop wallpaper"),
                    );
                    if stop_btn.clicked() {
                        if let Some(pid) = self.wallpaper_pid {
                            match Command::new("kill").arg("-TERM").arg(pid.to_string()).status() {
                                Ok(status) if status.success() => {
                                    self.runtime_message = Some(format!("Stopped wallpaper (pid {pid})"))
                                }
                                Ok(status) => {
                                    self.runtime_message =
                                        Some(format!("Failed to stop wallpaper (exit status {status})"))
                                }
                                Err(e) => {
                                    self.runtime_message = Some(format!("Failed to stop wallpaper: {e}"))
                                }
                            }
                        }
                    }
                });

                let mut startup = self.startup_enabled;
                if ui.checkbox(&mut startup, "Start on login").changed() {
                    match set_autostart_enabled(startup) {
                        Ok(()) => {
                            self.startup_enabled = startup;
                            self.runtime_message = Some(if startup {
                                "Enabled start on login".to_string()
                            } else {
                                "Disabled start on login".to_string()
                            });
                        }
                        Err(e) => {
                            self.runtime_message = Some(format!("Failed to update startup setting: {e}"));
                        }
                    }
                }

                if let Some(msg) = &self.runtime_message {
                    ui.label(msg);
                }
            });

            ui.separator();
            ui.label("Animation settings:");
            ui.horizontal(|ui| {
                ui.label("Planet speed");
                ui.add(egui::Slider::new(&mut self.planet_speed_input, 0.1..=5.0).show_value(true));
            });
            ui.horizontal(|ui| {
                ui.label("Camera zoom");
                ui.add(egui::Slider::new(&mut self.camera_zoom_input, 0.1..=5.0).show_value(true));
            });

            ui.separator();

            ui.group(|ui| {
                        ui.heading("Widgets");
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.cfg.overlay.show_clock, "Show clock");
                            ui.checkbox(&mut self.cfg.overlay.show_calendar, "Show calendar");
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.cfg.overlay.widget_float_mode, "Float mode");
                            ui.add(egui::Slider::new(&mut self.cfg.overlay.widget_float_speed, 0.01..=2.0).text("float speed"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Widget X");
                            let mut x = self.cfg.overlay.widget_position[0];
                            ui.add(egui::DragValue::new(&mut x));
                            ui.label("Widget Y");
                            let mut y = self.cfg.overlay.widget_position[1];
                            ui.add(egui::DragValue::new(&mut y));
                            self.cfg.overlay.widget_position = [x, y];
                        });

                        ui.separator();
                        ui.label("Preview");
                        // Simple preview area using a Frame
                        let (rect, _response) = ui.allocate_exact_size(egui::vec2(400.0, 200.0), egui::Sense::hover());
                        let painter = ui.painter_at(rect);
                        // Background
                        painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
                        // Draw a representation of the widget at normalized position
                        let pos = egui::pos2(
                            rect.left() + self.cfg.overlay.widget_position[0] * rect.width(),
                            rect.top() + self.cfg.overlay.widget_position[1] * rect.height(),
                        );
                        // float offset animation simple time-based
                        let t = ctx.input(|i| i.time) as f32;
                        let float_offset = if self.cfg.overlay.widget_float_mode {
                            let s = self.cfg.overlay.widget_float_speed;
                            egui::vec2((t * s).sin() * 8.0, (t * s * 0.7).cos() * 6.0)
                        } else {
                            egui::vec2(0.0, 0.0)
                        };
                        let draw_pos = pos + float_offset;
                        painter.circle_filled(draw_pos, 18.0, egui::Color32::from_rgba_premultiplied(
                            (self.cfg.overlay.widget_color[0] * 255.0) as u8,
                            (self.cfg.overlay.widget_color[1] * 255.0) as u8,
                            (self.cfg.overlay.widget_color[2] * 255.0) as u8,
                            (self.cfg.overlay.widget_color[3] * 255.0) as u8,
                        ));
                        painter.text(draw_pos + egui::vec2(0.0, -24.0), egui::Align2::CENTER_CENTER, "Clock", egui::FontId::proportional(14.0), egui::Color32::WHITE);
            });

            ui.separator();

            if ui.button("Save").clicked() {
                        // Apply and save
                        self.cfg.animation.planet_speed = self.planet_speed_input;
                        self.cfg.animation.camera_zoom = Self::clamp_zoom(self.camera_zoom_input);
                        if let Err(e) = self.cfg.save() {
                            tracing::error!("Failed to save config: {e}");
                        } else {
                            tracing::info!("Config saved");
                        }
            }

            ui.separator();

            ui.collapsing("Our Usage", |ui| {
                        ui.label("Live system usage (CPU %, Memory %)");
                        // Prepare plot points (Vec<[f64;2]>)
                        let cpu_vec: Vec<[f64; 2]> = self
                            .cpu_history
                            .iter()
                            .enumerate()
                            .map(|(i, v)| [i as f64, *v])
                            .collect();
                        let mem_vec: Vec<[f64; 2]> = self
                            .mem_history
                            .iter()
                            .enumerate()
                            .map(|(i, v)| [i as f64, *v])
                            .collect();

                        Plot::new("usage_plot").height(180.0).show(ui, |plot_ui| {
                            plot_ui.line(Line::new(cpu_vec).name("CPU %"));
                            plot_ui.line(Line::new(mem_vec).name("Memory %"));
                        });

                        ui.label("Notes: Live preview and usage graph for diagnostics. In Flatpak builds, access to system metrics may be limited.");
            });
        });
        ctx.request_repaint();
    }
}
