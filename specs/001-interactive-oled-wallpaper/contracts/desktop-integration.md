# Contract: Desktop Wallpaper Integration

**Purpose**: Define the interface contract between the wallpaper application and the Linux desktop environment (X11/Wayland).

**Scope**: This contract specifies how the application interacts with window managers, displays, and user input at the OS level.

## Desktop Integration Contract

### Window Properties (X11 EWMH)

The application MUST set the following X11 window properties to integrate as a desktop wallpaper:

```
_NET_WM_WINDOW_TYPE = _NET_WM_WINDOW_TYPE_DESKTOP
_NET_WM_STRUT_PARTIAL = [0, 0, 0, 0, ...]  (optional: reserve space)
```

**Behavior Expected**:
- Window appears behind all regular application windows
- Not shown in taskbar or window switcher
- Receives input events that don't hit foreground windows
- Survives window manager restarts (or can be re-launched)

### Display Server Support

**X11**: Native EWMH support via xlib/xcb bindings
**Wayland**: Layer-shell protocol (wl_layer_shell) support

**Fallback Behavior**: If neither can be detected, run as borderless fullscreen window with no decorations

### Input Event Contract

**Left-Click Drag**:
- Event: `MouseMove` with pressed button 1
- Payload: `(screen_x: i32, screen_y: i32, delta_x: i32, delta_y: i32)`
- Response: Pan camera by delta (call `Scene.handle_mouse_pan(Vec2 { delta_x, delta_y })`)
- Latency: <16ms from OS event to visual update (SC-005)
- Behavior: Continuous updates while mouse button held; stops immediately on release

**Right-Click**:
- Event: `MouseButton(Button::Right, Action::Press)`
- Payload: `(screen_x: i32, screen_y: i32)`
- Response: Trigger pulse effect at screen coordinates (call `Scene.trigger_pulse(Vec2 { screen_x, screen_y })`)
- Latency: <100ms from click to visible pulse animation
- Behavior: Discrete event; each right-click creates one pulse

**Input Pass-Through**:
- Events for overlapping windows: Application must NOT consume events from other windows
- If window manager indicates another window has focus: Application may ignore or handle input as requested
- No keyboard input handling required for v1

### Display Configuration Contract

**Display Connected**:
- Event: Monitor connection detected
- Behavior: If single-monitor mode, automatically use connected monitor
- Response: Resize wallpaper window to match monitor resolution

**Display Disconnected**:
- Event: Monitor removed
- Behavior: If primary display disconnected, pause animation and display static notification (future v2 enhancement)
- Response: For v1, no specific handling required; application continues

**Resolution Change**:
- Event: Monitor resolution changed (e.g., dynamic scaling)
- Behavior: Wallpaper viewport MUST adjust to new resolution
- Response: Update Camera viewport dimensions; continue rendering at new size

**Display Sleep / Screen Lock**:
- Event: Display powered off or screen locked
- Behavior: Application may continue rendering (not required to pause)
- Implementation: OS handles whether rendering continues; app doesn't need specific handling

### Wallpaper Persistence Contract

**Application Launch**:
- User launches app via system launcher, command line, or script
- App MUST detect display server and set window properties
- App MUST display galaxy wallpaper within 500ms of launch (UX expectation)
- App MUST NOT block system startup or hang with errors

**Application Termination**:
- Graceful shutdown: User can close app; desktop reverts to previous wallpaper or solid color
- Crash recovery: If app crashes, system should handle cleanup (window manager responsibility)
- No cleanup artifacts: App MUST NOT leave temporary files, locks, or zombie processes

**Multi-App Coexistence**:
- Multiple windows may be open on top of wallpaper
- Wallpaper continues animating behind any open windows
- Pulse effects and panning work only when wallpaper window has no occluding windows (or user input reaches wallpaper)

---

## Configuration File Contract

**Location**: `~/.config/oled-wallpaper/config.toml`

**Format**: TOML text file

**Structure**:
```toml
[animation]
planet_speed = 1.0              # float: 0.1-5.0
camera_zoom = 1.0               # float: 0.1-10.0

[colors]
sun_color = [1.0, 1.0, 0.8, 1.0]     # RGBA
planet_colors = [
  [0.4, 0.6, 1.0, 1.0],  # Planet 0
  [1.0, 0.6, 0.4, 1.0],  # Planet 1
  [0.7, 0.5, 1.0, 1.0],  # Planet 2
]
pulse_color = [0.3, 0.8, 1.0, 0.8]   # Light blue pulse

[sizes]
sun_radius = 20                  # pixels
planet_sizes = [15, 10, 12]     # relative to sun (pixels)

[orbits]
planet_speed_multipliers = [1.0, 1.5, 0.8]  # per-planet orbit period scaling
eccentricities = [0.1, 0.2, 0.05]           # orbital eccentricity per planet
```

**Validation Contract**:
- Missing keys: Use built-in defaults; emit warning to stderr
- Invalid TOML syntax: Log error; load defaults; continue with warning
- Out-of-range values: Clamp to valid ranges; log warning
- Invalid color format: Use default color; log error
- File not found: Create with defaults; log info message

**Reload Contract** (future v2):
- File watch: App MAY reload config on file change without restart
- Validation: Invalid reloaded config falls back to last known-good state
- User feedback: Log reload status to stderr or display toast notification

---

## Performance Contract

**Rendering Performance**:
- Minimum: 30 FPS on mid-range GPU (GTX 960, 2015)
- Target: 60 FPS on modern desktop GPU
- Measurement: Frames per second (FPS) during 60-second nominal operation with 3 planets

**Input Response Time**:
- Pan response: <16ms from mouse motion to visual update (SC-005)
- Pulse trigger: <100ms from right-click to visible effect
- Measurement: Milliseconds from OS event to rendered frame update

**Memory Footprint**:
- Steady state: <100 MB RAM during normal operation
- Peak: <200 MB RAM (including temporary allocations)
- Measurement: Resident set size (RSS) via `/proc/[pid]/status` or `ps` command

**CPU Usage**:
- Idle CPU: <5% per core (system-dependent)
- Typical CPU: 10-15% per core during animation at 60 FPS
- Measurement: CPU percentage via `top` or `ps` command

---

## Error Handling Contract

**Display Server Not Found**:
- Condition: Neither X11 nor Wayland can be detected
- Response: Attempt fullscreen window; log error to stderr
- User Experience: Wallpaper may appear as fullscreen window without desktop integration

**Config Load Failure**:
- Condition: File parsing error or invalid path
- Response: Use hardcoded defaults; log error to stderr with details
- User Experience: Wallpaper displays with default colors/speeds

**Rendering Failure**:
- Condition: GPU initialization fails or rendering crashes
- Response: Log error with details; exit gracefully with error code 1
- User Experience: App terminates; user can check logs or reinstall

**Input Event Overflow**:
- Condition: More input events than can be processed in one frame
- Response: Process queued events in FIFO order; drop oldest if queue full (limit ~1000 events)
- User Experience: Input may lag slightly under extreme rapid-click scenarios; no crashes

---

## Testing Contract

Each acceptance scenario from the feature spec MUST have a corresponding integration test:

| Requirement | Test Scenario | Expected Result |
|-------------|---------------|-----------------|
| FR-001: Render animated galaxy | Launch app, observe for 10s | Planets visible and moving; no freeze |
| FR-004: Pan with left-click drag | Click-drag wallpaper left; observe 100px pan | Scene pans; animation continues |
| FR-005: Pulse with right-click | Right-click at screen center (500, 500) | Pulse visible for ~1.5s at click point |
| FR-006: Flatpak deployment | Run `flatpak run ninja.boop.OledWallpaper` | App launches as wallpaper |
| FR-007: Full monitor display | Run at 1920x1080, 2560x1440, 3840x2160 | Wallpaper fills entire monitor |
| FR-008: OLED burn-in prevention | Run for 8 hours; observe pixel wear patterns | No static regions visible |
| FR-009: Window manager integration | Open file manager, browser over wallpaper | Apps display on top; wallpaper continues |
