# Quickstart: Interactive OLED Wallpaper

**Purpose**: Runnable validation scenarios that demonstrate the feature works end-to-end.

**Audience**: Developers and QA testers validating feature implementation.

---

## Prerequisites

- Linux system with X11 or Wayland display server
- Rust 1.75+ toolchain installed
- Git repository cloned to `/home/mbruno/development/oled-wallpaper`
- Optional: Flatpak tools installed (for deployment testing)

---

## Setup

### Build from Source

```bash
cd /home/mbruno/development/oled-wallpaper
cargo build --release
```

**Expected Output**:
- Compilation succeeds with no errors
- Binary created at `target/release/oled-wallpaper` (or similar)
- Build time: <2 minutes on typical hardware

### Verify Cargo.toml Dependencies

Check that `Cargo.toml` includes:
- `wgpu` - Graphics rendering
- `winit` - Window and event handling
- `glam` - Math library for orbital calculations
- `toml` - Configuration file parsing
- `tracing` - Logging (optional, for debugging)

Run:
```bash
cargo tree | grep -E "wgpu|winit|glam|toml"
```

**Expected Output**: All core dependencies listed

---

## Scenario 1: Launch Wallpaper and Verify Animation

**Objective**: Verify that the application starts successfully and renders an animated galaxy scene.

**Test Steps**:

1. **Launch the application**:
   ```bash
   target/release/oled-wallpaper
   ```
   
2. **Observe for 10 seconds**:
   - Wallpaper displays full-screen
   - Galaxy scene visible with sun and planets
   - Planets are moving (orbiting around sun)
   - No visual stuttering or freezing
   - Animation is smooth and continuous

3. **Check performance metrics** (optional):
   ```bash
   # In another terminal, monitor the process
   watch -n 0.5 'ps aux | grep oled-wallpaper'
   ```
   - CPU usage: 10-20% per core (expected)
   - Memory (RSS): <100 MB (within target SC-007)
   - Process remains stable (no crashes)

**Acceptance Criteria** (from FR-001, SC-001):
- ✅ Wallpaper displays full-screen on monitor
- ✅ At least one sun and 2 planets visible
- ✅ Planets have visibly moved after 30 seconds (SC-001: ≥30 FPS animation)
- ✅ No crashes or errors logged to stderr

---

## Scenario 2: Pan Galaxy with Left-Click Drag

**Objective**: Verify that left-click drag panning works smoothly and responsively.

**Test Steps**:

1. **Launch wallpaper** (as in Scenario 1)

2. **Perform left-click drag**:
   - Click and hold left mouse button at screen position (500, 500)
   - Drag to position (700, 500) - drag right ~200 pixels
   - Observe the galaxy scene panning to the left
   - Release mouse button

3. **Continue panning**:
   - Click-drag from (700, 500) to (700, 300) - drag up ~200 pixels
   - Observe scene panning downward
   - Release mouse button

4. **Verify responsiveness**:
   - Pan motion should feel immediate (<16ms latency, SC-005)
   - No input lag or delayed response
   - Animation continues smoothly during panning

**Acceptance Criteria** (from FR-004, SC-005):
- ✅ Scene pans in expected direction (drag right → scene moves right)
- ✅ Panning responds within 16ms (one frame at 60 FPS)
- ✅ Planets continue orbiting while panning
- ✅ Panning stops immediately when mouse is released

---

## Scenario 3: Trigger Pulse Effect with Right-Click

**Objective**: Verify that right-click pulse effects render and animate correctly.

**Test Steps**:

1. **Launch wallpaper**

2. **Right-click at screen center** (approximately 500, 500):
   - Observe a visual pulse effect appearing at click point
   - Pulse should radiate outward smoothly
   - Pulse effect fades out over ~1-2 seconds (SC-004)

3. **Perform multiple rapid right-clicks**:
   - Right-click at (300, 300), then (700, 500), then (500, 700)
   - Observe multiple pulse effects animating simultaneously
   - All pulses should render without visual artifacts
   - No performance degradation observed

4. **Verify pulse characteristics**:
   - Pulse is centered at exact click coordinates
   - Pulse expands in circular/radial pattern
   - Alpha (opacity) fades as pulse expands
   - Pulse completes within 1.5 seconds (SC-004)

**Acceptance Criteria** (from FR-005, SC-004):
- ✅ Pulse effect visible immediately after right-click
- ✅ Pulse radiates from click point outward
- ✅ Pulse completes and fades within 1.5 seconds
- ✅ Multiple concurrent pulses render without artifacts
- ✅ Animation performance maintained during pulses

---

## Scenario 4: Configuration Loading and Customization

**Objective**: Verify that animation parameters can be customized via config file.

**Test Steps**:

1. **Create config file**:
   ```bash
   mkdir -p ~/.config/oled-wallpaper/
   cat > ~/.config/oled-wallpaper/config.toml << 'EOF'
   [animation]
   planet_speed = 2.0
   
   [colors]
   sun_color = [1.0, 1.0, 0.0, 1.0]  # Yellow sun
   planet_colors = [
     [0.2, 0.8, 1.0, 1.0],  # Cyan planet 1
     [1.0, 0.2, 0.8, 1.0],  # Magenta planet 2
   ]
   EOF
   ```

2. **Launch wallpaper**:
   ```bash
   target/release/oled-wallpaper
   ```

3. **Observe configuration applied**:
   - Sun color changed to yellow (or close to it)
   - Planet colors changed to cyan and magenta
   - Planets orbiting 2x faster than default (planet_speed = 2.0)
   - Animation remains smooth at faster speed

4. **Modify config and verify reload** (if reload supported):
   - Edit config file to change `planet_speed = 0.5`
   - If auto-reload supported: Observe animation speed decrease
   - If manual restart required: Stop and restart app; observe new speed

**Acceptance Criteria** (from FR-010, related to success criteria):
- ✅ Config file loads without errors
- ✅ Configuration parameters applied to scene
- ✅ Animation speed multiplier works (planets faster/slower)
- ✅ Color values applied to sun and planets
- ✅ Invalid config handled gracefully (defaults used, warning logged)

---

## Scenario 5: Wallpaper Desktop Integration

**Objective**: Verify that wallpaper integrates with desktop and other applications work on top.

**Test Steps**:

1. **Launch wallpaper**:
   ```bash
   target/release/oled-wallpaper &
   ```
   (Note: Use `&` to run in background)

2. **Open other applications**:
   - Open file manager (e.g., `nautilus`, `dolphin`, or `thunar`)
   - Open terminal window
   - Open web browser or text editor

3. **Verify wallpaper behavior**:
   - Galaxy wallpaper visible behind all open windows
   - Wallpaper continues animating
   - Planets move smoothly behind application windows
   - Pulse effects work when clicking on wallpaper (not on other windows)

4. **Test window manager integration**:
   - Minimize all windows
   - Wallpaper should fill entire screen
   - Maximize a window again
   - Wallpaper remains in background

5. **Close wallpaper**:
   - Close wallpaper application
   - Desktop should revert to previous wallpaper (or solid color)
   - No errors or artifacts left behind

**Acceptance Criteria** (from FR-009, SC-002):
- ✅ Wallpaper appears behind all regular application windows
- ✅ Other applications function normally (no input capture)
- ✅ Wallpaper continues animating with other windows open
- ✅ Wallpaper seamlessly integrates with desktop environment
- ✅ No system errors or warnings when wallpaper is running

---

## Scenario 6: Flatpak Deployment Testing

**Objective**: Verify that the application can be installed and launched via Flatpak.

**Prerequisites**:
- Flatpak runtime installed: `sudo apt install flatpak` (Debian/Ubuntu)
- Freedesktop runtime: `flatpak install flathub org.freedesktop.Platform/x86_64/23.08`

**Test Steps**:

1. **Build Flatpak manifest** (if not already present):
   - Verify `ninja.boop.OledWallpaper.yml` exists in repo root
   - Build with: `flatpak-builder --user --install build ninja.boop.OledWallpaper.yml`
   - Build time: 2-5 minutes
   - Build should complete without errors

2. **Launch via Flatpak**:
   ```bash
   flatpak run ninja.boop.OledWallpaper
   ```

3. **Verify Flatpak launch** (same as Scenario 1):
   - Wallpaper displays
   - Animation smooth and responsive
   - All features (pan, pulse) work as expected

4. **Check installation** (optional):
   ```bash
   flatpak list --app
   ```
   - `ninja.boop.OledWallpaper` should be listed

**Acceptance Criteria** (from FR-006, SC-003):
- ✅ Flatpak build completes successfully
- ✅ Application installs without errors
- ✅ App launches within <2 minutes from user action (SC-003)
- ✅ All features functional via Flatpak
- ✅ Sandbox permissions allow display and input (no permission errors)

---

## Performance Validation

### Measure Frame Rate

```bash
# Option 1: Visual inspection (subjective)
# Run wallpaper and observe animation smoothness
# Smooth = ≥30 FPS; Choppy/stuttering = <30 FPS

# Option 2: Use FPS counter (if logging implemented)
RUST_LOG=debug target/release/oled-wallpaper 2>&1 | grep -i fps

# Option 3: Linux profiling tools
sudo perf stat target/release/oled-wallpaper
```

**Expected Results** (SC-001):
- Animation smooth and continuous (subjective)
- FPS counter shows ≥30 FPS (if implemented)
- CPU usage reasonable (<20% per core)

### Measure Input Latency

```bash
# Manual test: Click and observe immediate visual response
# Expected: Visual response within one frame (16ms at 60 FPS)
# Subjective test: Pan input feels responsive/reactive
```

**Expected Results** (SC-005):
- Left-click panning responds immediately (feels instant)
- Right-click pulse appears instantly
- No noticeable input lag

### Measure Memory Usage

```bash
# Terminal 1: Launch wallpaper
target/release/oled-wallpaper &

# Terminal 2: Monitor memory
watch -n 1 'ps aux | grep oled-wallpaper | grep -v grep | awk "{print \$6 \"KB RAM\"}"'
```

**Expected Results** (SC-007):
- RSS (resident set size) <100 MB
- Memory stable (not growing over time)
- VSZ (virtual memory) may be larger, but RSS is what counts

---

## Troubleshooting

### Wallpaper doesn't appear
- **Check**: Display server detection (X11 or Wayland?)
  ```bash
  echo $DISPLAY  # X11 should show ":0" or similar
  echo $WAYLAND_DISPLAY  # Wayland should show "wayland-0" or similar
  ```
- **Fix**: Ensure display server environment variables are set
- **Fallback**: App should run as fullscreen window if desktop integration fails

### No animation or static image
- **Check**: Rendering pipeline initialization
  ```bash
  RUST_LOG=debug target/release/oled-wallpaper 2>&1 | head -20
  ```
- **Fix**: Verify GPU drivers are up-to-date; OpenGL/Vulkan support working
- **Test**: Run benchmark (if available) to verify rendering capability

### Config file not loading
- **Check**: File location and format
  ```bash
  cat ~/.config/oled-wallpaper/config.toml
  ```
- **Fix**: Validate TOML syntax (use online validator or `toml-cli`)
- **Fix**: Ensure keys match expected names (case-sensitive)

### Input (panning/pulse) not working
- **Check**: Window manager is delivering input to wallpaper window
- **Fix**: Verify wallpaper window has focus (may need to click it first)
- **Test**: Check X11/Wayland event delivery with `xinput` (X11) or `wl-paste` (Wayland)

### Crashes or errors
- **Check**: System logs
  ```bash
  dmesg | tail -20  # Kernel messages
  journalctl -f     # System journal
  ```
- **Fix**: Update Rust toolchain: `rustup update`
- **Fix**: Rebuild clean: `cargo clean && cargo build --release`

---

## Success Summary

When all 6 scenarios pass, the feature is ready for the implementation phase (tasks.md and code development).

**Scenario Pass Checklist**:
- [ ] Scenario 1: Animation and rendering ✅
- [ ] Scenario 2: Left-click panning ✅
- [ ] Scenario 3: Right-click pulse effects ✅
- [ ] Scenario 4: Configuration customization ✅
- [ ] Scenario 5: Desktop integration ✅
- [ ] Scenario 6: Flatpak deployment ✅

All scenarios passing = **Feature specification is validated and ready for implementation**.
