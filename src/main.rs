//! OLED Wallpaper — interactive galaxy wallpaper
//!
//!   oled-wallpaper           # run forever
//!   oled-wallpaper --demo    # auto-close after 10s (default)
//!   oled-wallpaper --demo 30 # custom seconds

use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use glam::{Vec2, Vec3, Vec4};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tracing::info;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

use oled_wallpaper::config::Config;
use oled_wallpaper::init_tracing;
use oled_wallpaper::perf::PerfStats;
use oled_wallpaper::physics::{body::CelestialBody, orbit::Orbit, PhysicsSimulator};
use oled_wallpaper::renderer::camera::Camera;
use oled_wallpaper::widgets::WidgetSystem;

use glyphon::{
    Attrs, Buffer, Color as GlyphColor, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer,
};

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "oled-wallpaper",
    about = "Interactive galaxy wallpaper for OLED displays"
)]
struct Args {
    #[arg(long, default_missing_value = "10", num_args = 0..=1)]
    demo: Option<u64>,
}

// ─── Wallpaper pinning ────────────────────────────────────────────────────────

fn apply_wallpaper_hints(window: &winit::window::Window) {
    match window.window_handle().map(|h| h.as_raw()) {
        Ok(RawWindowHandle::Xlib(h)) => {
            #[cfg(feature = "x11")]
            pin_x11(h.window as u32);
        }
        Ok(RawWindowHandle::Xcb(h)) => {
            #[cfg(feature = "x11")]
            pin_x11(h.window.get());
        }
        Ok(RawWindowHandle::Wayland(_)) => info!("Wayland – compositor handles pinning"),
        _ => {}
    }
}

#[cfg(feature = "x11")]
fn pin_x11(window_id: u32) {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::*;
    let Ok((conn, _)) = x11rb::connect(None) else {
        return;
    };
    let intern = |n: &[u8]| {
        conn.intern_atom(false, n)
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| r.atom)
            .unwrap_or(0)
    };
    let wm_type = intern(b"_NET_WM_WINDOW_TYPE");
    let type_desk = intern(b"_NET_WM_WINDOW_TYPE_DESKTOP");
    let wm_state = intern(b"_NET_WM_STATE");
    let below = intern(b"_NET_WM_STATE_BELOW");
    let sticky = intern(b"_NET_WM_STATE_STICKY");
    conn.change_property(
        PropMode::REPLACE,
        window_id,
        wm_type,
        AtomEnum::ATOM,
        32,
        1,
        &type_desk.to_ne_bytes(),
    )
    .ok();
    let mut sb = Vec::new();
    sb.extend(below.to_ne_bytes());
    sb.extend(sticky.to_ne_bytes());
    conn.change_property(
        PropMode::REPLACE,
        window_id,
        wm_state,
        AtomEnum::ATOM,
        32,
        2,
        &sb,
    )
    .ok();
    conn.flush().ok();
}

// ─── Projection ───────────────────────────────────────────────────────────────
// 45° edge-on view of the galaxy plane + slow whole-plane yaw rotation.
// The yaw is driven by elapsed time so orbit lines never sit on the same pixel.

const VIEW_TILT: f32 = 45.0_f32; // degrees: edge-on-ish view of the orbital plane
                                 // Yaw speed: one full revolution every ~5 minutes (very gentle drift)
const YAW_SPEED: f32 = std::f32::consts::TAU / 300.0;

fn project(world: Vec3, plane_yaw: f32, view_half_h: f32, aspect: f32) -> Vec2 {
    // 1. Rotate entire plane around vertical (Y) axis — keeps orbits drifting
    let (sy, cy) = plane_yaw.sin_cos();
    let x2 = world.x * cy - world.z * sy;
    let z2 = world.x * sy + world.z * cy;

    // 2. Tilt 45° around X axis — edge-on galactic plane view
    let tilt = VIEW_TILT.to_radians();
    let y3 = world.y * tilt.cos() - z2 * tilt.sin();
    let z3 = world.y * tilt.sin() + z2 * tilt.cos();

    // 3. Mild perspective (adds sense of depth)
    let persp = 1.0 + z3 * 0.00035;
    let px = x2 * persp;
    let py = y3 * persp;

    Vec2::new(px / (view_half_h * aspect), py / view_half_h)
}

// ─── GPU types ────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
const QUAD: &[Vertex] = &[
    Vertex {
        position: [-1., -1.],
        uv: [-1., -1.],
    },
    Vertex {
        position: [1., -1.],
        uv: [1., -1.],
    },
    Vertex {
        position: [1., 1.],
        uv: [1., 1.],
    },
    Vertex {
        position: [-1., -1.],
        uv: [-1., -1.],
    },
    Vertex {
        position: [1., 1.],
        uv: [1., 1.],
    },
    Vertex {
        position: [-1., 1.],
        uv: [-1., 1.],
    },
];

/// Instance for the body/circle pipeline
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BodyInst {
    ndc_pos: [f32; 2],
    ndc_r: f32,
    glow: f32,
    color: [f32; 4],
}

/// Vertex for orbit lines and meteor trails
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVert {
    pos: [f32; 2],
    color: [f32; 4],
}

// ─── Pulse effects (right-click) ─────────────────────────────────────────────

const PULSE_SEGS: usize = 64;
const PULSE_DURATION: f32 = 1.5;

struct PulseEffect {
    origin: Vec2, // NDC position of right-click
    elapsed: f32,
    color: Vec4,
}

impl PulseEffect {
    fn new(origin: Vec2, rng: &mut SimpleRng) -> Self {
        let col = match rng.next() % 3 {
            0 => Vec4::new(0.30, 0.70, 1.00, 1.0), // blue
            1 => Vec4::new(0.75, 0.35, 1.00, 1.0), // violet
            _ => Vec4::new(1.00, 0.75, 0.25, 1.0), // gold
        };
        PulseEffect {
            origin,
            elapsed: 0.0,
            color: col,
        }
    }

    fn update(&mut self, dt: f32) {
        self.elapsed += dt;
    }
    fn alive(&self) -> bool {
        self.elapsed < PULSE_DURATION
    }

    /// Build screen-space ring LineVerts for this frame.
    fn line_verts(&self, aspect: f32) -> Vec<LineVert> {
        let p = (self.elapsed / PULSE_DURATION).clamp(0., 1.);
        let r = p.sqrt() * 0.55; // fast expand, then decelerate
        let a = (1.0 - p) * (1.0 - (p * 3.0 - 0.2).max(0.)) * 0.82;
        if a < 0.005 {
            return vec![];
        }
        let c = self.color;
        (0..=PULSE_SEGS)
            .map(|i| {
                let theta = (i as f32 / PULSE_SEGS as f32) * std::f32::consts::TAU;
                LineVert {
                    pos: [
                        self.origin.x + theta.cos() * r / aspect,
                        self.origin.y + theta.sin() * r,
                    ],
                    color: [c.x, c.y, c.z, a],
                }
            })
            .collect()
    }
}

// ─── Meteors ─────────────────────────────────────────────────────────────────

struct Meteor {
    pos: Vec3,     // world position (in the orbital plane, z=0)
    vel: Vec3,     // world velocity
    lifetime: f32, // seconds remaining
    max_life: f32,
    color: Vec4,
}

const METEOR_TRAIL: usize = 12; // trail segments per meteor
const METEOR_SPAWN_INTERVAL: f32 = 12.0; // seconds between spawns
const METEOR_SPEED: f32 = 600.0; // world units/sec

impl Meteor {
    fn spawn(rng: &mut SimpleRng) -> Self {
        // Start off-screen edge, random angle across sky
        let angle = rng.f32() * std::f32::consts::TAU;
        let dist = 920.0 + rng.f32() * 200.0;
        let pos = Vec3::new(
            dist * angle.cos(),
            dist * angle.sin(),
            rng.f32_range(-30., 30.),
        );
        // Aim loosely at center with some scatter
        let aim_angle = angle + std::f32::consts::PI + (rng.f32() - 0.5) * 0.6;
        let vel = Vec3::new(aim_angle.cos(), aim_angle.sin(), 0.) * METEOR_SPEED;
        let max_life = 2.0 + rng.f32() * 1.5;
        // Meteors are white-blue to light yellow
        let hue = rng.f32();
        let color = if hue < 0.5 {
            Vec4::new(0.85, 0.90, 1.0, 1.0) // ice blue
        } else {
            Vec4::new(1.0, 0.92, 0.70, 1.0) // warm white
        };
        Meteor {
            pos,
            vel,
            lifetime: max_life,
            max_life,
            color,
        }
    }

    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.lifetime -= dt;
    }
    fn alive(&self) -> bool {
        self.lifetime > 0.0
    }
}

// ─── Alien ship ──────────────────────────────────────────────────────────────

struct AlienShip {
    pos: Vec3,
    vel: Vec3,
    wobble: f32, // phase for vertical oscillation
    lifetime: f32,
    blink: f32, // blink timer
    visible: bool,
}

const ALIEN_INTERVAL: f32 = 180.0; // seconds between appearances
const ALIEN_SPEED: f32 = 350.0;

impl AlienShip {
    fn spawn(rng: &mut SimpleRng) -> Self {
        let angle = rng.f32() * std::f32::consts::TAU;
        let dist = 950.0;
        let pos = Vec3::new(dist * angle.cos(), dist * angle.sin(), 0.);
        let aim = angle + std::f32::consts::PI + (rng.f32() - 0.5) * 0.3;
        let vel = Vec3::new(aim.cos(), aim.sin(), 0.) * ALIEN_SPEED;
        AlienShip {
            pos,
            vel,
            wobble: 0.,
            lifetime: 12.0,
            blink: 0.,
            visible: true,
        }
    }
    fn update(&mut self, dt: f32) {
        self.wobble += dt * 3.5;
        self.lifetime -= dt;
        self.blink += dt;
        // Blink every 0.4s
        if self.blink > 0.4 {
            self.blink = 0.;
            self.visible = !self.visible;
        }
        // Wobbly path: slight sinusoidal drift perpendicular to travel
        let perp = Vec3::new(-self.vel.y, self.vel.x, 0.).normalize_or_zero();
        self.pos += (self.vel + perp * self.wobble.sin() * 40.) * dt;
    }
    fn alive(&self) -> bool {
        self.lifetime > 0.
    }
}

// ─── Minimal LCG RNG (no deps) ───────────────────────────────────────────────

struct SimpleRng(u64);
impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e3779b97f4a7c15)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn f32(&mut self) -> f32 {
        (self.next() & 0xFFFFFF) as f32 / 0xFFFFFF as f32
    }
    fn f32_range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.f32() * (hi - lo)
    }
}

// ─── Background starfield ────────────────────────────────────────────────────
// Fixed stars at large radius — they rotate with plane_yaw, never truly static.

struct Star {
    pos: Vec3, // world space, radius 1400–2800
    vel: Vec3, // slow drift velocity (0.3–1.8 wu/s)
    bright: f32,
    twinkle: f32,
}

fn gen_stars(rng: &mut SimpleRng, count: usize) -> Vec<Star> {
    (0..count)
        .map(|_| {
            let theta = rng.f32() * std::f32::consts::TAU;
            let phi = (rng.f32() * 2.0 - 1.0).acos();
            let r = rng.f32_range(1400., 2800.);
            // Drift direction: random unit vector, very slow
            let vt = rng.f32() * std::f32::consts::TAU;
            let vp = (rng.f32() * 2.0 - 1.0).acos();
            let spd = rng.f32_range(0.3, 1.8);
            Star {
                pos: Vec3::new(
                    r * phi.sin() * theta.cos(),
                    r * phi.sin() * theta.sin(),
                    r * phi.cos(),
                ),
                vel: Vec3::new(
                    spd * vp.sin() * vt.cos(),
                    spd * vp.sin() * vt.sin(),
                    spd * vp.cos(),
                ),
                bright: rng.f32_range(0.04, 0.20),
                twinkle: rng.f32() * std::f32::consts::TAU,
            }
        })
        .collect()
}

// ─── Oort cloud ───────────────────────────────────────────────────────────────
// Ring of icy bodies at the system edge, very slowly drifting. No Kepler needed.

struct OortBody {
    angle: f32,   // current angular position (radians)
    ang_vel: f32, // radians/second (very slow)
    radius: f32,  // distance from barycenter (1000–1400wu)
    z: f32,       // vertical scatter (±120wu for spherical cloud feel)
    size: f32,    // visual radius (1.5–4wu)
    bright: f32,  // 0.3–0.9
}

fn gen_oort(rng: &mut SimpleRng, count: usize) -> Vec<OortBody> {
    (0..count)
        .map(|_| OortBody {
            angle: rng.f32() * std::f32::consts::TAU,
            ang_vel: rng.f32_range(0.0008, 0.003) * if rng.f32() > 0.5 { 1. } else { -1. },
            radius: rng.f32_range(980., 1380.),
            z: rng.f32_range(-120., 120.),
            size: rng.f32_range(1.5, 4.5),
            bright: rng.f32_range(0.3, 0.9),
        })
        .collect()
}

// ─── Cosmic rays ─────────────────────────────────────────────────────────────
// Near-instant very bright streaks — rarer and much faster than meteors.

struct CosmicRay {
    pos: Vec3,
    vel: Vec3,
    lifetime: f32,
    max_life: f32,
    trail: Vec<Vec3>,
}

const COSMIC_TRAIL: usize = 8;
const COSMIC_SPEED: f32 = 3200.;
const COSMIC_INTERVAL: f32 = 45.; // average seconds between events

impl CosmicRay {
    fn spawn(rng: &mut SimpleRng) -> Self {
        let angle = rng.f32() * std::f32::consts::TAU;
        let dist = 1100.;
        let pos = Vec3::new(
            dist * angle.cos(),
            dist * angle.sin(),
            rng.f32_range(-200., 200.),
        );
        let aim = angle + std::f32::consts::PI + (rng.f32() - 0.5) * 0.25;
        let vel = Vec3::new(aim.cos(), aim.sin(), 0.) * COSMIC_SPEED;
        CosmicRay {
            pos,
            vel,
            lifetime: 0.28,
            max_life: 0.28,
            trail: Vec::new(),
        }
    }
    fn update(&mut self, dt: f32) {
        self.trail.push(self.pos);
        if self.trail.len() > COSMIC_TRAIL {
            self.trail.remove(0);
        }
        self.pos += self.vel * dt;
        self.lifetime -= dt;
    }
    fn alive(&self) -> bool {
        self.lifetime > 0.
    }
}

// ─── Orbit sampling ──────────────────────────────────────────────────────────

const ORBIT_SEGS: usize = 96;

fn orbit_verts(
    orbit: &Orbit,
    plane_yaw: f32,
    view_half_h: f32,
    aspect: f32,
    pan: Vec2,
) -> Vec<LineVert> {
    (0..=ORBIT_SEGS)
        .map(|i| {
            let t = (i as f32 / ORBIT_SEGS as f32) * orbit.orbital_period;
            let wpos = orbit.get_position_at_time(t);
            let ndc = project(wpos, plane_yaw, view_half_h, aspect) + pan;
            let depth_frac = (wpos.y / orbit.semi_major_axis).clamp(-1., 1.);
            let alpha = 0.10 + 0.07 * (depth_frac * 0.5 + 0.5);
            LineVert {
                pos: ndc.to_array(),
                color: [0.40, 0.52, 0.70, alpha],
            }
        })
        .collect()
}

// ─── Shaders ─────────────────────────────────────────────────────────────────

const BODY_SHADER: &str = r#"
struct Inst { @location(2) ndc_pos:vec2<f32>, @location(3) ndc_r:f32, @location(4) glow:f32, @location(5) color:vec4<f32> }
struct VO   { @builtin(position) pos:vec4<f32>, @location(0) uv:vec2<f32>, @location(1) col:vec4<f32>, @location(2) glow:f32 }
@vertex fn vs(@location(0) p:vec2<f32>, @location(1) uv:vec2<f32>, inst:Inst) -> VO {
    var o:VO; o.pos=vec4<f32>(inst.ndc_pos+uv*inst.ndc_r,0.,1.); o.uv=uv; o.col=inst.color; o.glow=inst.glow; return o;
}
@fragment fn fs(in:VO)->@location(0) vec4<f32> {
    let d=length(in.uv); if d>1.0{discard;}
    let core=1.-smoothstep(0.45,0.90,d);
    let halo=pow(max(0.,1.-d),5.)*in.glow;
    return vec4<f32>(mix(in.col.rgb,in.col.rgb*1.4+0.08,halo*0.18), clamp(core+halo*0.25,0.,1.));
}
"#;

const LINE_SHADER: &str = r#"
struct VO { @builtin(position) pos:vec4<f32>, @location(0) col:vec4<f32> }
@vertex fn vs(@location(0) pos:vec2<f32>, @location(1) col:vec4<f32>) -> VO {
    var o:VO; o.pos=vec4<f32>(pos,0.,1.); o.col=col; return o;
}
@fragment fn fs(in:VO)->@location(0) vec4<f32> { return in.col; }
"#;

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    init_tracing();
    let args = Args::parse();
    let demo_dur = args.demo.map(std::time::Duration::from_secs);

    // ── Window ─────────────────────────────────────────────────────────────
    let event_loop = EventLoop::new().expect("event loop");
    let monitor = event_loop.available_monitors().next();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("OLED Wallpaper")
            .with_fullscreen(Some(Fullscreen::Borderless(monitor)))
            .with_decorations(false)
            .build(&event_loop)
            .expect("window"),
    );
    let PhysicalSize { width, height } = window.inner_size();
    info!("Display: {}×{}", width, height);
    apply_wallpaper_hints(&window);

    // ── wgpu ───────────────────────────────────────────────────────────────
    let inst = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let surf = inst.create_surface(window.clone()).expect("surface");
    let adap = pollster::block_on(inst.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surf),
        ..Default::default()
    }))
    .expect("adapter");
    let (dev, queue) = pollster::block_on(adap.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("oled"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    ))
    .expect("device");
    let caps = surf.get_capabilities(&adap);
    let fmt = caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(caps.formats[0]);
    let mut cfg = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: fmt,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surf.configure(&dev, &cfg);

    // ── Glyphon text renderer (for --demo HUD overlay) ─────────────────────
    let mut font_system = FontSystem::new();
    let mut swash_cache = SwashCache::new();
    let mut text_atlas = TextAtlas::new(&dev, &queue, fmt);
    let mut text_renderer = TextRenderer::new(
        &mut text_atlas,
        &dev,
        wgpu::MultisampleState::default(),
        None,
    );
    let mut perf = PerfStats::new();

    // ── Build pipelines ────────────────────────────────────────────────────
    macro_rules! shader {
        ($src:expr) => {
            dev.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl($src.into()),
            })
        };
    }

    let body_sh = shader!(BODY_SHADER);
    let line_sh = shader!(LINE_SHADER);

    let alpha_blend = wgpu::BlendState::ALPHA_BLENDING;

    let body_inst_attrs = wgpu::vertex_attr_array![2=>Float32x2,3=>Float32,4=>Float32,5=>Float32x4];
    let body_pipe = dev.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("body"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &body_sh,
            entry_point: "vs",
            buffers: &[
                Vertex::desc(),
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<BodyInst>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &body_inst_attrs,
                },
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &body_sh,
            entry_point: "fs",
            targets: &[Some(wgpu::ColorTargetState {
                format: fmt,
                blend: Some(alpha_blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
    });

    let line_attrs = wgpu::vertex_attr_array![0=>Float32x2, 1=>Float32x4];
    let line_pipe = dev.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("line"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &line_sh,
            entry_point: "vs",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<LineVert>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &line_attrs,
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &line_sh,
            entry_point: "fs",
            targets: &[Some(wgpu::ColorTargetState {
                format: fmt,
                blend: Some(alpha_blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
    });

    // ── Static buffers ─────────────────────────────────────────────────────
    let quad_buf = dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("quad"),
        contents: bytemuck::cast_slice(QUAD),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // Body instances (generous upper bound: planets + stars + meteor heads + alien)
    // stars(250) + oort(120) + planets(6) + stars_bin(2) + nebulae(5) + meteors(8) + alien(1)
    const MAX_INSTS: u64 = 512;
    let body_ibuf = dev.create_buffer(&wgpu::BufferDescriptor {
        label: Some("body_inst"),
        size: MAX_INSTS * std::mem::size_of::<BodyInst>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Line verts: orbits + meteor trails
    // orbits + meteor trails + oort outline + cosmic ray trails + pulse rings (max 8 concurrent)
    const MAX_LINE_VERTS: u64 = 7 * (ORBIT_SEGS as u64 + 1)
        + 64 * (METEOR_TRAIL as u64 + 1)
        + 256
        + 64 * (COSMIC_TRAIL as u64 + 1)
        + 8 * (PULSE_SEGS as u64 + 1);
    let line_buf = dev.create_buffer(&wgpu::BufferDescriptor {
        label: Some("lines"),
        size: MAX_LINE_VERTS * std::mem::size_of::<LineVert>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // ── Physics – binary star + planets ───────────────────────────────────
    // Binary stars orbit their shared barycenter (0,0,0).
    // Each has a small, fast orbit so the "sun" position is never static.
    const BINARY_RADIUS: f32 = 38.0; // world units between each star and center
    const BINARY_PERIOD: f32 = 20.0; // seconds per orbit

    // ── Load config ────────────────────────────────────────────────────────
    let mut app_cfg = Config::load();
    let acfg = app_cfg.animation.clone();
    let speed = acfg.planet_speed; // orbital period divisor

    // Helper: override color from config if entry exists
    let pcolor = |i: usize, fallback: Vec4| -> Vec4 {
        acfg.planet_colors
            .get(i)
            .map(|&[r, g, b, a]| Vec4::new(r, g, b, a))
            .unwrap_or(fallback)
    };
    let psize =
        |i: usize, base: f32| -> f32 { base * acfg.planet_sizes.get(i).copied().unwrap_or(1.0) };

    // Planets orbit the barycenter (effectively the center of mass)
    // planet_speed scales orbital periods: speed=2 → twice as fast
    let planet_orbits: Vec<(&str, Orbit, f32, Vec4, f32)> = vec![
        (
            "mercury",
            Orbit::new(
                "mercury".into(),
                "bary".into(),
                140.,
                0.05,
                0.0,
                0.30,
                0.0,
                8.0 / speed,
            ),
            psize(0, 8.),
            pcolor(0, Vec4::new(0.70, 0.60, 0.50, 1.)),
            0.25,
        ),
        (
            "venus",
            Orbit::new(
                "venus".into(),
                "bary".into(),
                220.,
                0.01,
                0.1,
                1.20,
                1.0,
                18.0 / speed,
            ),
            psize(1, 14.),
            pcolor(1, Vec4::new(0.92, 0.76, 0.44, 1.)),
            0.35,
        ),
        (
            "earth",
            Orbit::new(
                "earth".into(),
                "bary".into(),
                315.,
                0.017,
                0.0,
                0.0,
                2.0,
                30.0 / speed,
            ),
            psize(2, 14.),
            pcolor(2, Vec4::new(0.18, 0.48, 0.90, 1.)),
            0.38,
        ),
        (
            "mars",
            Orbit::new(
                "mars".into(),
                "bary".into(),
                445.,
                0.09,
                0.03,
                0.8,
                3.5,
                52.0 / speed,
            ),
            psize(3, 10.),
            pcolor(3, Vec4::new(0.82, 0.32, 0.16, 1.)),
            0.28,
        ),
        (
            "jupiter",
            Orbit::new(
                "jupiter".into(),
                "bary".into(),
                610.,
                0.05,
                0.02,
                1.8,
                0.8,
                110.0 / speed,
            ),
            psize(4, 26.),
            pcolor(4, Vec4::new(0.80, 0.66, 0.48, 1.)),
            0.42,
        ),
        (
            "saturn",
            Orbit::new(
                "saturn".into(),
                "bary".into(),
                820.,
                0.06,
                0.04,
                0.5,
                5.5,
                185.0 / speed,
            ),
            psize(5, 21.),
            pcolor(5, Vec4::new(0.88, 0.78, 0.50, 1.)),
            0.38,
        ),
    ];

    let mut sim = PhysicsSimulator::new();
    // Barycenter anchor (static, never rendered)
    sim.add_body(CelestialBody::sun("bary".into(), Vec3::ZERO))
        .ok();

    for (id, ref orb, r, col, _) in &planet_orbits {
        let pos = orb.get_position_at_time(0.0);
        sim.add_body(CelestialBody::planet(id.to_string(), pos, *r, *col))
            .ok();
        sim.add_orbit(orb.clone()).ok();
    }

    // ── Dynamic state ──────────────────────────────────────────────────────
    let mut rng = SimpleRng::new(0xDEADBEEF_CAFEF00D);
    let mut meteors: Vec<Meteor> = Vec::new();
    let mut cosmic_rays: Vec<CosmicRay> = Vec::new();
    let mut alien: Option<AlienShip> = None;
    let mut next_meteor = METEOR_SPAWN_INTERVAL * rng.f32_range(0.5, 1.0);
    let mut next_alien = ALIEN_INTERVAL * rng.f32_range(0.8, 1.2);
    let mut next_cosmic = COSMIC_INTERVAL * rng.f32_range(0.6, 1.4);
    let mut meteor_history: Vec<Vec<Vec3>> = Vec::new();

    // ── Cosmic scene init ──────────────────────────────────────────────────
    let mut stars = gen_stars(&mut rng, 260);
    let mut oort = gen_oort(&mut rng, 120); // trail positions

    let _camera = Camera::new(width, height); // used by pan_ndc offset below
    let start = Instant::now();
    let mut last = Instant::now();

    // ── Input state ────────────────────────────────────────────────────────
    let mut pan_ndc: Vec2 = Vec2::ZERO; // accumulated NDC offset from panning
    let mut mouse_down: bool = false;
    let mut last_cursor: Vec2 = Vec2::ZERO; // previous cursor position in pixels
    let mut pulses: Vec<PulseEffect> = Vec::new();
    let mut widgets = WidgetSystem::new(&app_cfg.overlay, Vec2::new(width as f32, height as f32));
    let mut widget_dragged_since_press = false;

    // Scale: fit outermost orbit (820wu) in 88% of screen's short half-axis,
    // then apply camera_zoom from config (zoom > 1 = tighter view, < 1 = wider)
    let view_half_h = (820.0 / 0.88) / acfg.camera_zoom;

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => elwt.exit(),
                Event::WindowEvent {
                    event: WindowEvent::Resized(s),
                    ..
                } => {
                    cfg.width = s.width.max(1);
                    cfg.height = s.height.max(1);
                    surf.configure(&dev, &cfg);
                }

                // ── Left-click drag → pan or widget drag ───────────────────────
                Event::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            button: MouseButton::Left,
                            state,
                            ..
                        },
                    ..
                } => match state {
                    ElementState::Pressed => {
                        widget_dragged_since_press = false;
                        if app_cfg.overlay.widget_enabled && widgets.hit_test(last_cursor) {
                            widgets.begin_drag(last_cursor);
                            mouse_down = false;
                        } else {
                            mouse_down = true;
                        }
                    }
                    ElementState::Released => {
                        mouse_down = false;
                        if widgets.is_dragging() {
                            widgets.end_drag();
                            if widget_dragged_since_press {
                                app_cfg.overlay.widget_position = widgets
                                    .position_norm(Vec2::new(cfg.width as f32, cfg.height as f32));
                                let _ = app_cfg.save();
                            }
                        }
                    }
                },
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    let cur = Vec2::new(position.x as f32, position.y as f32);
                    if widgets.is_dragging() {
                        widgets.drag_to(cur, Vec2::new(cfg.width as f32, cfg.height as f32));
                        if (cur - last_cursor).length() > 0.25 {
                            widget_dragged_since_press = true;
                        }
                    } else if mouse_down {
                        let delta = cur - last_cursor;
                        // Convert pixel delta → NDC delta and accumulate
                        pan_ndc.x += delta.x / (cfg.width as f32 * 0.5);
                        pan_ndc.y -= delta.y / (cfg.height as f32 * 0.5); // y flipped
                    }
                    last_cursor = cur;
                }

                // ── Right-click → pulse ───────────────────────────────────────
                Event::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            button: MouseButton::Right,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    // Convert last known cursor pos to NDC
                    let ndc = Vec2::new(
                        last_cursor.x / (cfg.width as f32 * 0.5) - 1.0,
                        1.0 - last_cursor.y / (cfg.height as f32 * 0.5),
                    );
                    pulses.push({
                        let mut p = PulseEffect::new(ndc, &mut rng);
                        // Override colour from config if set
                        if let Some([r, g, b, a]) = acfg.pulse_color {
                            p.color = Vec4::new(r, g, b, a);
                        }
                        // Scale alpha by pulse_intensity
                        p.color.w = (p.color.w * acfg.pulse_intensity).clamp(0.05, 1.0);
                        p
                    });
                }
                Event::AboutToWait => {
                    if let Some(limit) = demo_dur {
                        if start.elapsed() >= limit {
                            elwt.exit();
                            return;
                        }
                    }

                    let now = Instant::now();
                    let dt = now.duration_since(last).as_secs_f32().min(0.05);
                    perf.update(dt);
                    last = now;
                    let t = start.elapsed().as_secs_f32();
                    let w = cfg.width as f32;
                    let h = cfg.height as f32;
                    let asp = w / h;
                    widgets.update(t, Vec2::new(w, h), &app_cfg.overlay);

                    // Slow whole-plane yaw — opposite direction to the largest body (Saturn),
                    // so nothing drifts predictably. Full revolution every ~5 min.
                    let plane_yaw = t * YAW_SPEED;

                    // Pan-aware projection closure — used everywhere in this frame
                    // Pan-aware projection closure — used everywhere in this frame
                    let proj =
                        |w: Vec3| -> Vec2 { project(w, plane_yaw, view_half_h, asp) + pan_ndc };

                    // ── Physics ────────────────────────────────────────────
                    sim.update_all_bodies(dt, t);

                    // ── Spawn / update meteors ─────────────────────────────
                    next_meteor -= dt;
                    if next_meteor <= 0. {
                        meteors.push(Meteor::spawn(&mut rng));
                        meteor_history.push(Vec::new());
                        next_meteor = METEOR_SPAWN_INTERVAL * rng.f32_range(0.8, 1.5);
                    }
                    let mut dead = Vec::new();
                    for (i, m) in meteors.iter_mut().enumerate() {
                        if let Some(hist) = meteor_history.get_mut(i) {
                            hist.push(m.pos);
                            if hist.len() > METEOR_TRAIL {
                                hist.remove(0);
                            }
                        }
                        m.update(dt);
                        if !m.alive() {
                            dead.push(i);
                        }
                    }
                    for i in dead.into_iter().rev() {
                        meteors.remove(i);
                        meteor_history.remove(i);
                    }

                    // ── Spawn / update alien ship ──────────────────────────
                    next_alien -= dt;
                    if next_alien <= 0. && alien.is_none() {
                        info!("👽 Alien ship spotted!");
                        alien = Some(AlienShip::spawn(&mut rng));
                        next_alien = ALIEN_INTERVAL * rng.f32_range(0.9, 1.3);
                    }
                    if let Some(ref mut ship) = alien {
                        ship.update(dt);
                    }
                    if alien.as_ref().map(|s| !s.alive()).unwrap_or(false) {
                        alien = None;
                    }

                    // ── Spawn / update cosmic rays ─────────────────────────
                    next_cosmic -= dt;
                    if next_cosmic <= 0. {
                        cosmic_rays.push(CosmicRay::spawn(&mut rng));
                        next_cosmic = COSMIC_INTERVAL * rng.f32_range(0.6, 1.4);
                    }
                    cosmic_rays.retain_mut(|r| {
                        r.update(dt);
                        r.alive()
                    });

                    // ── Update pulses ──────────────────────────────────────
                    pulses.retain_mut(|p| {
                        p.update(dt);
                        p.alive()
                    });

                    // ── Update Oort cloud + star drift ────────────────────
                    for body in oort.iter_mut() {
                        body.angle += body.ang_vel * dt;
                    }
                    for star in stars.iter_mut() {
                        star.pos += star.vel * dt;
                    }

                    // ── Build body instances ───────────────────────────────
                    let mut body_insts: Vec<BodyInst> = Vec::new();

                    // Binary stars: orbit the barycenter at ±BINARY_RADIUS
                    let star_angle = t * std::f32::consts::TAU / BINARY_PERIOD;
                    let star_a_pos = Vec3::new(
                        BINARY_RADIUS * star_angle.cos(),
                        BINARY_RADIUS * star_angle.sin(),
                        0.,
                    );
                    let star_b_pos = Vec3::new(
                        -BINARY_RADIUS * star_angle.cos(),
                        -BINARY_RADIUS * star_angle.sin(),
                        0.,
                    );

                    for (pos, color, glow) in [
                        (
                            star_a_pos,
                            {
                                let [r, g, b, a] = acfg.star_a_color;
                                Vec4::new(r, g, b, a)
                            },
                            2.2f32,
                        ),
                        (
                            star_b_pos,
                            {
                                let [r, g, b, a] = acfg.star_b_color;
                                Vec4::new(r, g, b, a)
                            },
                            1.8f32,
                        ),
                    ] {
                        let ndc = proj(pos);
                        let z3 = pos.y * VIEW_TILT.to_radians().sin();
                        let persp = 1.0 + z3 * 0.00035;
                        body_insts.push(BodyInst {
                            ndc_pos: ndc.to_array(),
                            ndc_r: 36.0 * persp / (view_half_h * asp),
                            glow,
                            color: color.to_array(),
                        });
                    }

                    // Planets
                    for (id, _orb, radius, color, glow) in &planet_orbits {
                        let pw = sim.get_body(id).map(|b| b.position).unwrap_or(Vec3::ZERO);
                        let ndc = proj(pw);
                        let z3 = pw.y * VIEW_TILT.to_radians().sin();
                        let persp = 1.0 + z3 * 0.00035;
                        body_insts.push(BodyInst {
                            ndc_pos: ndc.to_array(),
                            ndc_r: radius * persp / (view_half_h * asp),
                            glow: *glow,
                            color: color.to_array(),
                        });
                    }

                    // Meteor heads
                    for m in &meteors {
                        let ndc = proj(m.pos);
                        let life_frac = (m.lifetime / m.max_life).clamp(0., 1.);
                        body_insts.push(BodyInst {
                            ndc_pos: ndc.to_array(),
                            ndc_r: 5.0 / (view_half_h * asp),
                            glow: 0.8,
                            color: [m.color.x, m.color.y, m.color.z, life_frac],
                        });
                    }

                    // Alien ship (blinks green)
                    if let Some(ref ship) = alien {
                        if ship.visible {
                            let ndc = proj(ship.pos);
                            let pulse = (t * 8.).sin() * 0.3 + 0.7;
                            body_insts.push(BodyInst {
                                ndc_pos: ndc.to_array(),
                                ndc_r: 8.0 / (view_half_h * asp),
                                glow: 1.5,
                                color: [0.2 * pulse, 1.0 * pulse, 0.4 * pulse, 0.95],
                            });
                        }
                    }

                    // ── Oort cloud ─────────────────────────────────────────
                    for ob in &oort {
                        let wp =
                            Vec3::new(ob.radius * ob.angle.cos(), ob.radius * ob.angle.sin(), ob.z);
                        let ndc = proj(wp);
                        // Icy blue-white, dim
                        let brightness = ob.bright * (0.7 + 0.3 * (t * 0.8 + ob.angle).sin());
                        body_insts.push(BodyInst {
                            ndc_pos: ndc.to_array(),
                            ndc_r: ob.size / (view_half_h * asp),
                            glow: 0.15,
                            color: [
                                0.72 * brightness,
                                0.82 * brightness,
                                1.0 * brightness,
                                brightness * 0.85,
                            ],
                        });
                    }

                    // ── Background starfield ───────────────────────────────
                    for star in &stars {
                        let ndc =
                            project(star.pos, plane_yaw * 0.15, view_half_h, asp) + pan_ndc * 0.08; // rotate slower for parallax
                        let twinkle = 0.7 + 0.3 * (t * 2.3 + star.twinkle).sin();
                        let b = star.bright * twinkle;
                        body_insts.push(BodyInst {
                            ndc_pos: ndc.to_array(),
                            ndc_r: 2.5 / (view_half_h * asp),
                            glow: 0.05,
                            color: [b * 0.90, b * 0.92, b, b],
                        });
                    }

                    // ── Build line verts ───────────────────────────────────
                    let mut line_verts: Vec<LineVert> = Vec::new();
                    let mut line_strips: Vec<(u32, u32)> = Vec::new();

                    // Orbit rings
                    for (_id, orb, ..) in &planet_orbits {
                        let start_i = line_verts.len() as u32;
                        let verts = orbit_verts(orb, plane_yaw, view_half_h, asp, pan_ndc);
                        let count = verts.len() as u32;
                        line_verts.extend(verts);
                        line_strips.push((start_i, count));
                    }

                    // Binary star orbit (tiny ellipse visible at center)
                    {
                        let start_i = line_verts.len() as u32;
                        let n = 64u32;
                        for i in 0..=n {
                            let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
                            let wp = Vec3::new(
                                BINARY_RADIUS * angle.cos(),
                                BINARY_RADIUS * angle.sin(),
                                0.,
                            );
                            let ndc = proj(wp);
                            line_verts.push(LineVert {
                                pos: ndc.to_array(),
                                color: [0.80, 0.70, 0.30, 0.18],
                            });
                        }
                        line_strips.push((start_i, n + 1));
                    }

                    // Meteor trails
                    for (i, m) in meteors.iter().enumerate() {
                        if let Some(hist) = meteor_history.get(i) {
                            if hist.len() < 2 {
                                continue;
                            }
                            let start_i = line_verts.len() as u32;
                            let n = hist.len();
                            for (j, &hp) in hist.iter().enumerate() {
                                let ndc = proj(hp);
                                let alpha = (j as f32 / n as f32)
                                    * 0.55
                                    * (m.lifetime / m.max_life).clamp(0., 1.);
                                line_verts.push(LineVert {
                                    pos: ndc.to_array(),
                                    color: [m.color.x, m.color.y, m.color.z, alpha],
                                });
                            }
                            // Current head position
                            let ndc = proj(m.pos);
                            let alpha = 0.9 * (m.lifetime / m.max_life).clamp(0., 1.);
                            line_verts.push(LineVert {
                                pos: ndc.to_array(),
                                color: [m.color.x, m.color.y, m.color.z, alpha],
                            });
                            line_strips.push((start_i, (hist.len() + 1) as u32));
                        }
                    }

                    // Alien trail (short)
                    if let Some(ref ship) = alien {
                        let start_i = line_verts.len() as u32;
                        let ndc = proj(ship.pos);
                        let pulse = (t * 6.).sin() * 0.2 + 0.5;
                        line_verts.push(LineVert {
                            pos: [ndc.x - 0.012, ndc.y],
                            color: [0.1, pulse, 0.3, 0.4],
                        });
                        line_verts.push(LineVert {
                            pos: ndc.to_array(),
                            color: [0.2, 1.0, 0.4, 0.0],
                        });
                        line_strips.push((start_i, 2));
                    }

                    // Cosmic ray trails (bright white-blue, sharp)
                    for ray in &cosmic_rays {
                        if ray.trail.len() < 2 {
                            continue;
                        }
                        let start_i = line_verts.len() as u32;
                        let n = ray.trail.len();
                        for (j, &hp) in ray.trail.iter().enumerate() {
                            let ndc = proj(hp);
                            let alpha = (j as f32 / n as f32)
                                * 0.9
                                * (ray.lifetime / ray.max_life).clamp(0., 1.);
                            line_verts.push(LineVert {
                                pos: ndc.to_array(),
                                color: [0.85, 0.92, 1.0, alpha],
                            });
                        }
                        let ndc = proj(ray.pos);
                        line_verts.push(LineVert {
                            pos: ndc.to_array(),
                            color: [1.0, 1.0, 1.0, 0.95 * (ray.lifetime / ray.max_life)],
                        });
                        line_strips.push((start_i, (ray.trail.len() + 1) as u32));
                    }

                    // Pulse rings (right-click ripples, screen-space circles)
                    for pulse in &pulses {
                        let verts = pulse.line_verts(asp);
                        if verts.is_empty() {
                            continue;
                        }
                        let start_i = line_verts.len() as u32;
                        let count = verts.len() as u32;
                        line_verts.extend(verts);
                        line_strips.push((start_i, count));
                    }

                    // Upload
                    if !body_insts.is_empty() {
                        queue.write_buffer(&body_ibuf, 0, bytemuck::cast_slice(&body_insts));
                    }
                    if !line_verts.is_empty() {
                        queue.write_buffer(&line_buf, 0, bytemuck::cast_slice(&line_verts));
                    }

                    // ── Prepare text overlays (HUD + widgets) ───────────────
                    let show_hud = demo_dur.is_some() || app_cfg.overlay.show_hud;
                    let show_widgets = app_cfg.overlay.widget_enabled
                        && (app_cfg.overlay.show_clock || app_cfg.overlay.show_calendar);

                    let mut hud_pos: Option<(f32, f32)> = None;
                    let mut widget_style: Option<(f32, f32, GlyphColor)> = None;
                    let mut areas: Vec<TextArea> = Vec::new();

                    let hud_buf_opt = if show_hud {
                        let hud = perf.hud_text();
                        let font_size = (cfg.height as f32 * 0.018).clamp(14.0, 28.0);
                        let line_h = font_size * 1.35;
                        let mut buf =
                            Buffer::new(&mut font_system, Metrics::new(font_size, line_h));
                        let pad = font_size * 0.8;
                        let block_h = line_h * 3.5;
                        let top = (cfg.height as f32 - block_h - pad).max(0.0);
                        let right_w = 320.0_f32;
                        let left = (cfg.width as f32 - right_w - pad).max(0.0);
                        buf.set_size(&mut font_system, right_w, block_h + 10.0);
                        buf.set_text(
                            &mut font_system,
                            &hud,
                            Attrs::new().family(Family::Monospace),
                            Shaping::Basic,
                        );
                        buf.shape_until_scroll(&mut font_system);
                        hud_pos = Some((left, top));
                        Some(buf)
                    } else {
                        None
                    };

                    let widget_buf_opt = if show_widgets {
                        let text = widgets.text(&app_cfg.overlay);
                        if !text.is_empty() {
                            let font_size = ((cfg.height as f32 * 0.024)
                                * app_cfg.overlay.widget_font_scale)
                                .clamp(12.0, 48.0);
                            let line_h = font_size * 1.28;
                            let mut buf =
                                Buffer::new(&mut font_system, Metrics::new(font_size, line_h));
                            buf.set_size(&mut font_system, 360.0, 130.0);
                            buf.set_text(
                                &mut font_system,
                                &text,
                                Attrs::new().family(Family::SansSerif),
                                Shaping::Basic,
                            );
                            buf.shape_until_scroll(&mut font_system);
                            let wp = widgets.position_px();
                            let [wr, wg, wb, wa] = app_cfg.overlay.widget_color;
                            let col = GlyphColor::rgba(
                                (wr.clamp(0.0, 1.0) * 255.0) as u8,
                                (wg.clamp(0.0, 1.0) * 255.0) as u8,
                                (wb.clamp(0.0, 1.0) * 255.0) as u8,
                                (wa.clamp(0.0, 1.0) * 255.0) as u8,
                            );
                            widget_style = Some((wp.x, wp.y, col));
                            Some(buf)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let (Some(ref b), Some((left, top))) = (&hud_buf_opt, hud_pos) {
                        areas.push(TextArea {
                            buffer: b,
                            left,
                            top,
                            scale: 1.0,
                            bounds: TextBounds {
                                left: left as i32,
                                top: top as i32,
                                right: cfg.width as i32,
                                bottom: cfg.height as i32,
                            },
                            default_color: GlyphColor::rgb(160, 200, 255),
                        });
                    }

                    if let (Some(ref b), Some((left, top, col))) = (&widget_buf_opt, widget_style) {
                        areas.push(TextArea {
                            buffer: b,
                            left,
                            top,
                            scale: 1.0,
                            bounds: TextBounds {
                                left: 0,
                                top: 0,
                                right: cfg.width as i32,
                                bottom: cfg.height as i32,
                            },
                            default_color: col,
                        });
                    }

                    if !areas.is_empty() {
                        let _ = text_renderer.prepare(
                            &dev,
                            &queue,
                            &mut font_system,
                            &mut text_atlas,
                            Resolution {
                                width: cfg.width,
                                height: cfg.height,
                            },
                            areas,
                            &mut swash_cache,
                        );
                    }

                    // ── Render ─────────────────────────────────────────────
                    let frame = match surf.get_current_texture() {
                        Ok(f) => f,
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            surf.configure(&dev, &cfg);
                            return;
                        }
                        Err(e) => {
                            tracing::error!("{e}");
                            return;
                        }
                    };
                    let view_tex = frame.texture.create_view(&Default::default());
                    let mut enc = dev.create_command_encoder(&Default::default());
                    {
                        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("frame"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view_tex,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.,
                                        g: 0.,
                                        b: 0.,
                                        a: 1.,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        // 1. Orbit lines + meteor trails (behind everything)
                        if !line_verts.is_empty() {
                            pass.set_pipeline(&line_pipe);
                            pass.set_vertex_buffer(0, line_buf.slice(..));
                            for (start, count) in &line_strips {
                                pass.draw(*start..*start + count, 0..1);
                            }
                        }

                        // 2. Stars + planets + meteors + alien (on top)
                        if !body_insts.is_empty() {
                            pass.set_pipeline(&body_pipe);
                            pass.set_vertex_buffer(0, quad_buf.slice(..));
                            pass.set_vertex_buffer(1, body_ibuf.slice(..));
                            pass.draw(0..6, 0..body_insts.len() as u32);
                        }

                        // 3. Text overlays (HUD and/or widgets)
                        if show_hud || show_widgets {
                            let _ = text_renderer.render(&text_atlas, &mut pass);
                        }
                    }
                    queue.submit([enc.finish()]);
                    frame.present();
                }
                _ => {}
            }
        })
        .expect("event loop");
}
