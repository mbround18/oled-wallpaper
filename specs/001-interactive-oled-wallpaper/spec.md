# Feature Specification: Interactive OLED Wallpaper

**Feature Branch**: `001-interactive-oled-wallpaper`

**Created**: 2026-08-14

**Status**: Draft

**Input**: User description: "i want to build a rust progream that exports as an applet or flatpack or something managable. The goal of this application is simple, become a ever gancing galaxy thats interactive with the user but is designed to be run on QD OLED or OLED panels to not have a static background. think like outter space with an orbit moving sun mobing planets letting the user on left click move it around ot right click send a pulse out just a really pleasant interactive desktop wall paper"

## User Scenarios & Testing

### User Story 1 - View Dynamic Galaxy Wallpaper (Priority: P1)

A user launches the application and sees a continuously animated galaxy scene displayed as their desktop wallpaper. The scene features orbital mechanics with planets moving around a sun, creating an engaging and dynamic visual that prevents static image burn-in on OLED displays.

**Why this priority**: This is the core value proposition. Without the animated galaxy display, the application has no purpose. This is essential for OLED burn-in prevention.

**Independent Test**: Can be tested by launching the application and verifying that animated celestial bodies (sun, planets) are visible and continuously moving on the desktop background for a minimum duration.

**Acceptance Scenarios**:

1. **Given** the application is installed and configured, **When** the user launches it, **Then** the wallpaper displays an animated galaxy scene with a sun and at least 2 orbiting planets visible across the entire monitor.
2. **Given** the wallpaper is running, **When** 30 seconds elapse, **Then** the planets have visibly moved along their orbital paths from their starting positions.
3. **Given** the application is running on an OLED panel, **When** observing the display over time, **Then** the animated content prevents static burn-in by ensuring no pixel remains unchanged for extended periods.

---

### User Story 2 - Interactive Panning with Mouse (Priority: P1)

A user can click and drag the wallpaper with their left mouse button to pan/move the galaxy scene around, allowing them to explore different parts of the space environment and maintain interaction with the desktop while work continues in the background.

**Why this priority**: Interactivity is a core requirement stated by the user. Left-click panning is the primary interaction mechanic for exploration.

**Independent Test**: Can be tested by clicking and dragging on the wallpaper, observing that the galaxy scene moves proportionally to the mouse movement in the expected direction.

**Acceptance Scenarios**:

1. **Given** the wallpaper is displayed, **When** the user clicks and holds the left mouse button and drags, **Then** the galaxy pans smoothly in the direction of the drag movement.
2. **Given** the user releases the left mouse button, **When** no input is given, **Then** the panning stops and the planets continue their orbital motion from the new position.
3. **Given** the user pans to the edge of the scene, **When** attempting to pan further, **Then** the scene either wraps around or gracefully bounds the pan limits.

---

### User Story 3 - Pulse Effect with Right-Click (Priority: P1)

A user can right-click anywhere on the wallpaper to generate a visual pulse effect (e.g., expanding wave, radiant glow, distortion) emanating from the click point. This provides satisfying visual feedback and tactile engagement with the desktop environment.

**Why this priority**: Right-click pulse is explicitly requested as a core interactive feature that provides pleasant visual feedback and engagement.

**Independent Test**: Can be tested by right-clicking on the wallpaper at various positions and verifying that a distinct visual pulse effect originates from the click coordinates and propagates outward.

**Acceptance Scenarios**:

1. **Given** the wallpaper is displayed, **When** the user right-clicks at any location on the screen, **Then** a pulse effect visibly emanates from the click position.
2. **Given** a pulse effect is triggered, **When** observing the effect, **Then** the pulse radiates outward smoothly and completes its animation within 1-2 seconds.
3. **Given** the user right-clicks multiple times in quick succession, **When** multiple pulses are triggered, **Then** all pulses animate simultaneously without visual artifacts or performance degradation.

---

### User Story 4 - Deployable as Flatpak/Applet (Priority: P1)

A user can install and launch the application as a managed package (Flatpak, AppImage, or system applet) on their Linux desktop environment. The application integrates seamlessly with the desktop wallpaper system without requiring manual compilation or complex setup.

**Why this priority**: Deployment format is explicitly required. Without a manageable installation method, the application is not usable by end-users.

**Independent Test**: Can be tested by installing the application via the intended package format (Flatpak/AppImage), launching it through standard desktop app launchers, and verifying it displays as the wallpaper without errors.

**Acceptance Scenarios**:

1. **Given** a Flatpak/AppImage is provided, **When** a user installs it via their package manager or file manager, **Then** the installation completes successfully with no errors.
2. **Given** the application is installed, **When** the user launches it from their desktop app launcher or system menu, **Then** it displays immediately as the desktop wallpaper.
3. **Given** the application is running, **When** the user opens other applications or performs system tasks, **Then** the wallpaper remains visible in the background and continues animating.

---

### User Story 5 – Cosmic Ambiance Effects (Priority: P2)

A user views a richly layered space scene where distant cosmic phenomena fill the background: a faint ring of icy debris at the edge of the solar system, softly glowing nebulae in the far distance, a field of dim background stars that rotate with the scene, and rare instantaneous bright streaks of cosmic rays crossing the display. These effects deepen the sense of being in outer space without overwhelming the primary solar system view.

**Why this priority**: These effects elevate the visual quality and reinforce OLED burn-in prevention by adding subtle pixel-level movement throughout the entire screen, but they are not essential to the core interactive experience.

**Independent Test**: Can be tested by observing the scene over a 5-minute period and confirming that background stars rotate with the plane, the Oort cloud ring is faintly visible at the scene boundary, at least one nebula region is visible, and a cosmic ray streak occurs at least once.

**Acceptance Scenarios**:

1. **Given** the wallpaper is running, **When** the user observes the scene, **Then** approximately 250 dim background stars are visible, rotating with the galactic plane yaw.
2. **Given** the wallpaper is running, **When** the user observes the outer edge of the scene, **Then** a faint ring of ~120 slowly drifting icy bodies (the Oort cloud) is visible.
3. **Given** the wallpaper is running over time, **When** a nebula region is visible, **Then** it appears as a large soft-glowing color region (purple, blue, pink, or teal) at very low opacity that does not obscure foreground objects.
4. **Given** the wallpaper is running, **When** a cosmic ray event fires, **Then** a sharp, near-instantaneous bright streak crosses the screen and fades within 0.3 seconds.
5. **Given** all cosmic ambiance effects are active, **When** monitoring frame rate, **Then** frame rate does not drop below 30 FPS.

---

### User Story 6 – Overlay Widget System (Priority: P2)

A user can optionally enable floating widgets—such as a clock or calendar—displayed directly over the wallpaper. The user can drag each widget to a preferred screen position, or enable a "float mode" that slowly moves the widget across the screen along a smooth path, ensuring no pixel beneath the widget stays lit in the same pattern for long. Widget positions and settings are saved automatically so they persist across application restarts.

**Why this priority**: Widgets add practical utility to the decorative wallpaper experience, and float mode directly contributes to OLED burn-in prevention for widget pixels. However, widgets are an optional enhancement and not core to the wallpaper concept.

**Independent Test**: Can be tested by enabling the clock widget in the config, verifying it displays the current time, dragging it to a new position, restarting the app and confirming the position persisted, and enabling float mode to confirm the widget visibly drifts over a 60-second period.

**Acceptance Scenarios**:

1. **Given** the clock widget is enabled in config, **When** the wallpaper is running, **Then** a clock displaying the current time is visible in the configured format (12h or 24h, with or without seconds).
2. **Given** the clock widget is displayed, **When** the user reads the displayed time, **Then** it is accurate to within 1 second of system time.
3. **Given** the calendar widget is enabled, **When** the wallpaper is running, **Then** the current date (or full month view, depending on config) is displayed as an overlay widget.
4. **Given** a widget is displayed, **When** the user clicks and drags it, **Then** the widget moves to the new position and remains there until moved again.
5. **Given** float mode is enabled for a widget, **When** 60 seconds have elapsed, **Then** the widget's screen position has changed by at least 50 pixels from its position at the start of that period.
6. **Given** the user repositions or configures a widget, **When** the application is restarted, **Then** the widget appears at the saved position with the saved settings.

---

### Edge Cases

- What happens if the user opens a full-screen application? (Wallpaper may be hidden; should resume when returning to desktop)
- How does the application behave when the monitor is put to sleep or display is locked?
- What happens if the user clicks/right-clicks on UI elements of running applications? (Input should pass through or be ignored)
- How does performance scale if the user has multiple monitors?
- What happens during window manager crashes or display configuration changes?
- How does the pulse effect interact with semi-transparent windows?

## Requirements

### Functional Requirements

- **FR-001**: System MUST render a continuously animated galaxy scene as the desktop wallpaper
- **FR-002**: System MUST include at least one sun/central star object and at least 2 orbiting planets with simulated orbital mechanics
- **FR-003**: System MUST maintain animation performance (smooth motion without stuttering) on typical desktop hardware
- **FR-004**: Users MUST be able to pan the galaxy view by clicking and dragging with the left mouse button
- **FR-005**: Users MUST be able to trigger a visual pulse effect by right-clicking anywhere on the wallpaper
- **FR-006**: System MUST be deployable via Flatpak or AppImage format for easy installation on Linux systems
- **FR-007**: System MUST display the wallpaper across the full monitor resolution(s)
- **FR-008**: System MUST prevent static burn-in on OLED panels by ensuring no pixel region remains completely static for extended periods
- **FR-009**: System MUST handle window manager interactions gracefully (e.g., allow other applications to run on top)
- **FR-010**: System MUST provide configurable animation parameters including planet orbital speed, colors, sizes, orbital patterns, and zoom/pan scale
- **FR-011**: System MUST render a background starfield of ~250 distant stars that rotate with the scene's galactic plane yaw
- **FR-012**: System MUST render an Oort cloud consisting of ~120 icy bodies positioned at the edge of the solar system, slowly drifting with slight vertical scatter for a spherical feel
- **FR-013**: System MUST periodically display nebula clouds as large soft-glowing color regions (purple, blue, pink, or teal) at very low opacity
- **FR-014**: System MUST display rare cosmic ray events as fast, bright streaks that are rarer than meteor showers and fade within 0.3 seconds
- **FR-015**: System MUST support optional overlay widgets (clock, calendar) with configurable display format and screen position
- **FR-016**: Widgets MUST support a "float mode" that slowly drifts the widget's position along a smooth path to prevent static burn-in under the widget

### Already Implemented Effects (as of initial build)

The following features are implemented in the codebase and captured here for specification completeness:

- **Binary star system**: Two stars orbit their shared gravitational center (barycenter) on a 20-second period, visible alongside the primary sun
- **45° edge-on galactic plane**: The scene is viewed from a 45° edge-on angle with the entire galactic plane rotating slowly (5-minute full revolution) to prevent burn-in
- **Meteor showers**: Meteors spawn approximately every 12 seconds with a 12-segment fading trail and randomized direction and speed
- **Easter egg alien ship**: Approximately every 3 minutes, a small alien ship appears, blinks green, travels a wobbly path across the scene, and is logged to the console
- **`--demo N` CLI flag**: Using the `clap` argument parser, the user can pass `--demo N` to auto-close the application after N seconds (useful for screenshots and testing)
- **OS-agnostic wallpaper pinning**: The application uses raw window handle detection to pin the window as a desktop wallpaper layer on X11, Wayland, and unknown display servers
- **Additive perspective projection**: Scene objects are projected using an additive blending perspective from the 45° view angle, giving bright objects a glowing appearance

### Key Entities

- **Celestial Bodies**: Visual objects representing the sun and planets with properties like position, velocity, size, color, and orbital parameters
- **Scene Camera/View**: The viewport into the galaxy that can be panned and positioned
- **Pulse Effect**: A temporary visual animation triggered at click coordinates with properties like radius, duration, intensity
- **Orbit Paths**: Mathematical descriptions of planetary orbits (potentially elliptical) defining motion patterns
- **Wallpaper Canvas**: The full-screen rendering surface for the desktop background

## Success Criteria

### Measurable Outcomes

- **SC-001**: Animation renders at ≥30 FPS on desktop systems with modest GPUs (GTX 960 or equivalent) to ensure smooth visual experience
- **SC-002**: Wallpaper integrates seamlessly with desktop environment—other applications can launch and function normally on top of it
- **SC-003**: User can install application via Flatpak/AppImage and launch within <2 minutes without technical knowledge
- **SC-004**: Pulse effect completes animation and becomes imperceptible within 1.5 seconds of user right-click
- **SC-005**: Panning input responds to mouse movement within <16ms (single frame at 60 FPS) to feel responsive
- **SC-006**: After 8+ hours of continuous display on OLED panel, no visible static image burn-in occurs in any screen region
- **SC-007**: Application memory footprint remains <200 MB during normal operation
- **SC-008**: 90% of users find the interactive experience pleasant and engaging (subjective, measured via feedback)
- **SC-009**: Cosmic ambiance effects (starfield, Oort cloud, nebulae, cosmic rays) render without the frame rate dropping below 30 FPS
- **SC-010**: In float mode, a widget's screen position changes by at least 50 pixels every 60 seconds, ensuring burn-in prevention for the widget region
- **SC-011**: The clock widget displays system time accurate to within 1 second at all times
- **SC-012**: All cosmic ambiance effects (Oort cloud, nebulae, cosmic rays, background starfield) contribute to pixel movement such that no screen region remains completely static for more than 10 minutes

## Assumptions

- **Target Platform**: Application targets Linux desktop environments with X11 or Wayland display servers; OLED panel support is primary use case but not strictly required for functionality
- **Rendering Capability**: Desktop systems have adequate GPU support for real-time 3D graphics (OpenGL 3.3 or Vulkan support assumed)
- **User Expertise**: End-users can install applications from package managers or app stores (low technical barrier assumed)
- **Scope Boundary**: Mobile platforms (Android/iOS) and Windows/macOS support are out of scope for v1
- **Configuration**: Application ships with sensible default animation parameters; advanced configuration is optional for v1
- **Input Method**: Primary input is mouse-based (left-click for pan, right-click for pulse); keyboard shortcuts are out of scope for v1
- **Wallpaper Persistence**: Application behavior for multi-monitor setups is unsupported in v1 (single monitor or unified display assumed)
- **Rendering Library**: Existing Rust graphics ecosystem (e.g., wgpu, bevy, or custom OpenGL bindings) will be evaluated during planning; no specific tech stack commitment at spec stage
