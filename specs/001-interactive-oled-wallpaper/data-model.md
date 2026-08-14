# Data Model: Interactive OLED Wallpaper

**Date**: 2026-08-14 | **Feature**: Interactive OLED Wallpaper | **Spec**: [spec.md](spec.md)

## Core Entities

### CelestialBody

Represents a single celestial object (sun or planet) in the galaxy scene.

**Attributes**:
- `id`: Unique identifier (String, e.g., "sun", "planet_0", "planet_1")
- `position`: Current 3D position in world space (Vec3: x, y, z floats)
- `velocity`: Current velocity vector (Vec3: x, y, z floats)
- `radius`: Visual size/radius (float, pixels)
- `color`: RGBA color value (Vec4: r, g, b, a; values 0.0-1.0)
- `mass`: Physical mass for orbit calculations (float, relative units)
- `is_static`: Boolean flag (true for sun; false for planets)

**Key Methods**:
- `update_position(delta_time: f32)`: Calculate new position based on velocity and elapsed time
- `get_screen_coordinates(camera: &Camera) -> Vec2`: Transform world position to screen coordinates
- `intersects_point(point: Vec2, screen_radius: f32) -> bool`: Check if click/pulse affects this body

**Validation Rules**:
- `radius > 0.0`: Visual size must be positive
- `mass > 0.0`: All bodies must have positive mass
- `color.w > 0.0`: Alpha must be positive (not fully transparent)
- `id.len() > 0`: ID must not be empty
- `is_static` MUST be true for exactly one body (the sun)

**State Transitions**:
- **Created**: Instantiated with initial position, velocity, color
- **Updating**: Position updates every frame via `update_position()`
- **Rendering**: Position passed to renderer each frame
- **Destroyed**: Removed from scene (sun never destroyed; planets persist throughout app lifetime)

---

### Orbit

Mathematical description of a celestial body's orbital path.

**Attributes**:
- `body_id`: Reference to orbiting CelestialBody (String)
- `parent_id`: Reference to parent body being orbited (String, typically "sun")
- `semi_major_axis`: Half the longest diameter of ellipse (float, world space units)
- `eccentricity`: Orbital ellipse elongation (float, 0.0-1.0; 0=circle, 0.999=highly elliptical)
- `inclination`: Orbital plane tilt in radians (float, 0.0-2π)
- `argument_of_periapsis`: Orientation of ellipse in radians (float, 0.0-2π)
- `mean_anomaly_at_epoch`: Starting position in orbit at time t=0 (float, radians)
- `orbital_period`: Time for one complete orbit (float, seconds)

**Key Methods**:
- `get_position_at_time(time: f32) -> Vec3`: Calculate orbital body position using Kepler equations
- `get_velocity_at_time(time: f32) -> Vec3`: Calculate orbital velocity (derivative of position)
- `is_valid() -> bool`: Validate orbital parameters are physically reasonable

**Validation Rules**:
- `semi_major_axis > 0.0`: Orbit size must be positive
- `eccentricity >= 0.0 && eccentricity < 1.0`: Valid ellipse eccentricity (parabolic/hyperbolic excluded for wallpaper)
- `orbital_period > 0.0`: Orbit period must be positive
- `parent_id != body_id`: Cannot orbit itself
- `orbital_period` proportional to `semi_major_axis^1.5` (Kepler's third law validation)

**State Transitions**:
- **Created**: Instantiated with orbital parameters
- **Active**: Position calculated continuously during scene updates
- **Persisted**: Never removed; orbits persist throughout application lifetime

---

### Camera / Viewport

Represents the viewpoint into the galaxy scene; supports user panning and zoom.

**Attributes**:
- `position`: Camera center point in world space (Vec2: x, y)
- `zoom_level`: Magnification factor (float, 1.0=default; >1.0=zoomed in)
- `width`: Viewport width in pixels (u32, matches monitor width)
- `height`: Viewport height in pixels (u32, matches monitor height)
- `pan_offset`: Accumulated panning displacement (Vec2: x, y world space units)

**Key Methods**:
- `pan_by(delta: Vec2)`: Apply panning motion (from mouse drag)
- `set_zoom(level: f32)`: Update zoom level (future enhancement)
- `screen_to_world(screen_pos: Vec2) -> Vec3`: Convert screen coordinates to world space (for click detection)
- `world_to_screen(world_pos: Vec3) -> Vec2`: Convert world coordinates to screen space (for rendering)
- `get_view_matrix() -> Mat4`: Calculate view transformation for rendering pipeline
- `clamp_pan_bounds()`: Enforce pan limits (optional; wrapping alternative)

**Validation Rules**:
- `width > 0 && height > 0`: Viewport dimensions must be positive
- `zoom_level > 0.0`: Zoom must be positive
- `zoom_level` constrained to reasonable range (e.g., 0.1 to 10.0)

**State Transitions**:
- **Initialized**: Created with monitor resolution and default position
- **Panning**: `pan_offset` updated continuously during mouse drag
- **Stable**: Position held constant when no input (planets continue orbiting)
- **Reconfigured**: Viewport dimensions updated on monitor resize

---

### PulseEffect

Temporary visual animation triggered by user right-click.

**Attributes**:
- `id`: Unique effect identifier (String, auto-generated UUID or counter)
- `origin`: Screen center point of pulse (Vec2: x, y pixels)
- `elapsed_time`: Time since pulse creation (float, seconds)
- `duration`: Total animation duration (float, seconds; typically 1.5)
- `max_radius`: Maximum pulse radius (float, pixels)
- `intensity`: Animation intensity multiplier (float, 0.0-1.0)
- `color`: Pulse color RGBA (Vec4: r, g, b, a)
- `is_active`: Whether pulse is still animating (bool)

**Key Methods**:
- `update(delta_time: f32)`: Advance pulse animation state
- `get_progress() -> f32`: Return animation progress (0.0=start, 1.0=complete)
- `get_current_radius() -> f32`: Calculate pulse radius at current time (typically `max_radius * progress`)
- `get_alpha() -> f32`: Calculate fade-out alpha (1.0→0.0 over duration)
- `is_finished() -> bool`: Check if pulse animation is complete

**Validation Rules**:
- `duration > 0.0`: Animation must have positive duration
- `max_radius > 0.0`: Pulse must have positive radius
- `intensity >= 0.0 && intensity <= 1.0`: Intensity normalized to 0-1 range
- `elapsed_time <= duration`: Time cannot exceed total duration

**State Transitions**:
- **Created**: Triggered by user right-click; elapsed_time = 0, is_active = true
- **Animating**: elapsed_time increments each frame; visual radius and alpha change
- **Complete**: elapsed_time >= duration; marked is_active = false; removed from scene on next update
- **Destroyed**: Removed from pulse pool after completion

---

### AnimationConfig

User-configurable animation parameters loaded from TOML config file.

**Attributes**:
- `planet_speed`: Orbital speed multiplier (float, default 1.0, range 0.1-5.0)
- `planet_colors`: Array of planet RGBA colors (Vec<Vec4>, one per planet)
- `planet_sizes`: Array of relative planet radii (Vec<f32>, one per planet, default 1.0 per planet)
- `orbital_patterns`: Array of orbital parameter overrides (Vec<OrbitParams>)
  - `eccentricity`: Override eccentricity for planet (float, 0.0-0.99)
  - `inclination`: Override inclination for planet (float, radians)
  - `period_multiplier`: Scale orbital period (float, e.g., 2.0 = twice as slow)
- `camera_zoom`: Default camera zoom (float, default 1.0)
- `pulse_color`: Default pulse effect color (Vec4, default: light blue)
- `pulse_intensity`: Pulse animation intensity (float, default 1.0, range 0.1-2.0)

**Key Methods**:
- `load_from_file(path: &str) -> Result<AnimationConfig>`: Load from TOML file
- `validate() -> Result<()>`: Validate all parameters are in valid ranges
- `apply_to_scene(scene: &mut Scene)`: Update scene objects with config values

**Validation Rules**:
- `planet_speed > 0.0`: Speed multiplier must be positive
- Each color RGBA in range [0.0, 1.0]
- Each planet_size > 0.0
- `camera_zoom > 0.0`
- Arrays must match scene entity counts (or use defaults for missing entries)
- Invalid TOML syntax → load defaults; emit warning log

**State Transitions**:
- **Unloaded**: Before application startup
- **Loaded**: Read from disk during app initialization
- **Applied**: Parameters copied to scene entities
- **Active**: Config influences rendering throughout app lifetime

---

### Scene

Container for all visual objects and state.

**Attributes**:
- `celestial_bodies`: Collection of CelestialBody objects (Vec<CelestialBody>)
- `orbits`: Collection of Orbit calculations (Vec<Orbit>)
- `camera`: Active Camera/Viewport (Camera)
- `pulse_effects`: Active pulse animations (Vec<PulseEffect>)
- `config`: Current animation configuration (AnimationConfig)
- `time_elapsed`: Total time since app start (float, seconds; for orbit calculations)

**Key Methods**:
- `update(delta_time: f32)`: Update all scene state (orbits, camera, pulses)
- `render()`: Generate frame (delegates to renderer)
- `handle_mouse_pan(delta: Vec2)`: Process mouse drag input
- `trigger_pulse(screen_pos: Vec2)`: Create new pulse effect at click point
- `get_bodies_at_point(screen_pos: Vec2) -> Vec<&CelestialBody>`: Hit test for click events

**Validation Rules**:
- At least one CelestialBody (the sun) must exist
- At least one Orbit must exist (planet orbiting sun)
- Camera dimensions must match actual viewport
- No duplicate body IDs

**State Transitions**:
- **Initialized**: Scene created with sun and planets
- **Running**: Updated every frame with continuous animation
- **Paused**: (Not applicable for v1; always running)
- **Reconfigured**: Configuration reloaded on file change (future enhancement)

---

## Relationships

```
Scene
├── CelestialBody[] (sun + planets)
├── Orbit[] (one per planet, referencing parent and child bodies)
├── Camera (viewport/pan state)
├── PulseEffect[] (active pulse animations)
└── AnimationConfig (loaded parameters)

Orbit references:
  - parent_id → CelestialBody (the sun)
  - body_id → CelestialBody (the planet)

PulseEffect triggers from:
  - User right-click event → Scene.trigger_pulse() → Create new PulseEffect
```

---

## Burn-In Prevention Mechanism

**Core Principle**: Ensure no pixel region remains static for >15 minutes (within 8-hour observation period).

**Data Model Support**:
- **Orbit continuity**: Kepler orbits ensure planet positions never repeat pixel-perfectly
- **Camera pan**: User-initiated pans vary viewport position
- **Time-based variation**: All positions functions of elapsed time; no repeating sequences within 8 hours
- **Orbital periods**: Planets configured with non-integer-multiple periods (e.g., 60s, 100s, 160s) prevent synchronization

**Validation**: Scene validates that:
- `orbital_period` values are co-prime (no common divisors)
- At least one body has period >3600 seconds (1 hour)
- Camera pan can reach all scene regions

---

## Migration Notes

- **v1 Scope**: Single sun, 2-3 planets with fixed initial orbits
- **v2 Considerations**: Add planet creation API, dynamic configuration reload, multi-monitor support
- **Data Persistence**: Configuration only (TOML file); no save/restore of scene state
