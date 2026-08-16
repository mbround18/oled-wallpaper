//! Performance statistics — zero extra dependencies, all from /proc.
//!
//! GPU metrics use /proc/self/fdinfo/ with DRM standard fields:
//!   drm-engine-render  → cumulative ns the render engine was active (delta → %)
//!   drm-memory-vram    → VRAM bytes currently allocated by this process
//!
//! Works vendor-agnostically across NVIDIA (open), AMD (amdgpu), Intel (i915/xe).

use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::time::Instant;

const FRAME_WINDOW: usize = 120; // frames for rolling FPS average
const METRICS_PERIOD: f32 = 1.0; // seconds between CPU/GPU refresh

pub struct PerfStats {
    // FPS
    frame_times: VecDeque<f32>,

    // CPU
    last_at: Instant,
    last_cpu_ticks: u64,
    cpu_percent: f32,

    // GPU (DRM /proc/self/fdinfo)
    last_render_ns: u64,
    gpu_percent: f32,
    vram_mb: f32,
}

impl Default for PerfStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfStats {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(FRAME_WINDOW),
            last_at: Instant::now(),
            last_cpu_ticks: cpu_ticks(),
            cpu_percent: 0.0,
            last_render_ns: drm_fdinfo().0,
            gpu_percent: 0.0,
            vram_mb: 0.0,
        }
    }

    /// Call once per frame with the frame delta time.
    pub fn update(&mut self, dt: f32) {
        // Rolling FPS
        self.frame_times.push_back(dt);
        if self.frame_times.len() > FRAME_WINDOW {
            self.frame_times.pop_front();
        }

        // CPU + GPU — refresh every METRICS_PERIOD
        let elapsed = self.last_at.elapsed().as_secs_f32();
        if elapsed >= METRICS_PERIOD {
            // ── CPU ──────────────────────────────────────────────────
            let now_ticks = cpu_ticks();
            let dcpu = now_ticks.saturating_sub(self.last_cpu_ticks);
            // CLK_TCK = 100 on virtually all modern Linux
            self.cpu_percent = (dcpu as f32 / 100.0) / elapsed * 100.0;
            self.last_cpu_ticks = now_ticks;

            // ── GPU (DRM) ─────────────────────────────────────────────
            let (render_ns, vram_bytes) = drm_fdinfo();

            let delta_ns = render_ns.saturating_sub(self.last_render_ns);
            let interval_ns = (elapsed * 1_000_000_000.0) as u64;
            self.gpu_percent = if interval_ns > 0 {
                (delta_ns as f32 / interval_ns as f32 * 100.0).min(100.0)
            } else {
                0.0
            };
            self.vram_mb = vram_bytes as f32 / (1024.0 * 1024.0);
            self.last_render_ns = render_ns;

            self.last_at = Instant::now();
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────

    pub fn fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let total: f32 = self.frame_times.iter().sum();
        self.frame_times.len() as f32 / total.max(f32::EPSILON)
    }

    pub fn cpu_percent(&self) -> f32 {
        self.cpu_percent
    }
    pub fn gpu_percent(&self) -> f32 {
        self.gpu_percent
    }
    pub fn vram_mb(&self) -> f32 {
        self.vram_mb
    }
    pub fn ram_mb(&self) -> f32 {
        rss_kb() as f32 / 1024.0
    }

    /// Multi-line HUD string shown in --demo overlay.
    pub fn hud_text(&self) -> String {
        let gpu_line = if self.vram_mb > 0.0 {
            format!(
                "GPU  {:4.1}%  VRAM {:5.1} MB",
                self.gpu_percent(),
                self.vram_mb()
            )
        } else {
            // GPU fd not yet open or driver doesn't expose drm fields
            "GPU  --   (no DRM fdinfo)".to_string()
        };
        format!(
            "FPS  {:5.1}\nCPU  {:4.1}%   RAM {:5.1} MB\n{}",
            self.fps(),
            self.cpu_percent(),
            self.ram_mb(),
            gpu_line,
        )
    }
}

// ─── /proc readers ───────────────────────────────────────────────────────────

/// Scan /proc/self/fdinfo/ for DRM fields.
/// Returns (cumulative_render_ns, total_vram_bytes).
fn drm_fdinfo() -> (u64, u64) {
    let Ok(entries) = std::fs::read_dir("/proc/self/fdinfo") else {
        return (0, 0);
    };

    let mut total_render_ns: u64 = 0;
    let mut total_vram: u64 = 0;

    for entry in entries.flatten() {
        let Ok(file) = std::fs::File::open(entry.path()) else {
            continue;
        };
        let reader = BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            // drm-engine-render: <ns>  (cumulative ns render engine was active)
            if let Some(rest) = line.strip_prefix("drm-engine-render:") {
                if let Ok(ns) = rest
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or(())
                {
                    total_render_ns += ns;
                }
            }
            // drm-memory-vram: <bytes>  (VRAM allocated to this fd)
            if let Some(rest) = line.strip_prefix("drm-memory-vram:") {
                if let Ok(bytes) = rest
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or(())
                {
                    total_vram += bytes;
                }
            }
        }
    }

    (total_render_ns, total_vram)
}

/// utime + stime from /proc/self/stat in CLK_TCK units.
fn cpu_ticks() -> u64 {
    let Ok(data) = std::fs::read_to_string("/proc/self/stat") else {
        return 0;
    };
    let f: Vec<&str> = data.split_whitespace().collect();
    let utime: u64 = f.get(13).and_then(|s| s.parse().ok()).unwrap_or(0);
    let stime: u64 = f.get(14).and_then(|s| s.parse().ok()).unwrap_or(0);
    utime + stime
}

/// VmRSS from /proc/self/status in kilobytes.
fn rss_kb() -> u64 {
    let Ok(data) = std::fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    data.lines()
        .find(|l| l.starts_with("VmRSS:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
