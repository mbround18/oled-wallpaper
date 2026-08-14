# Phase 0 Research: Interactive OLED Wallpaper

**Date**: 2026-08-14 | **Feature**: Interactive OLED Wallpaper

## Overview

This document consolidates technical research and design decisions for the Interactive OLED Wallpaper application. No blocking clarifications remain—all critical decisions have been made based on requirements analysis and industry best practices.

## Key Research Findings

### 1. Graphics Rendering Architecture

**Decision**: Use **wgpu** as the primary graphics abstraction layer with Rust bindings.

**Rationale**:
- Wgpu provides cross-platform abstraction (Vulkan, Metal, DX12, GL) with excellent Linux/X11 support
- Lightweight and performance-focused; suitable for real-time wallpaper rendering
- Active Rust ecosystem support; integrates well with winit for window management
- Enables mobile-friendly architecture without major refactoring (future consideration)

**Alternatives Considered**:
- **Bevy Engine**: Full game engine; overkill for a simple wallpaper, adds unnecessary complexity and memory overhead
- **Raw OpenGL**: Lower-level control, but wgpu provides similar performance with better maintainability
- **glium or gl-rs**: Legacy OpenGL bindings; wgpu is the modern Rust alternative

**Dependencies**:
- `wgpu` (graphics API abstraction)
- `winit` (window and event handling for X11/Wayland)
- `glam` (math library for vectors, matrices, orbital calculations)

### 2. Orbital Mechanics Simulation

**Decision**: Implement **Kepler orbit simulation** with simplified elliptical paths.

**Rationale**:
- Provides visually authentic planetary motion without full N-body physics
- Computationally efficient (O(n) for n planets) vs. N-body complexity
- User-configurable parameters: orbital period, eccentricity, semi-major axis
- Prevents static image burn-in by ensuring continuous smooth motion

**Implementation Approach**:
- Parametric equations for elliptical orbits: position as function of time
- Store orbit parameters per planet (period, eccentricity, inclination)
- Update positions each frame based on elapsed time
- Planets never return to exact same pixel position within hours of runtime

**Burn-In Prevention**:
- Animation speed ensures pixel variance over 8+ hours (success criteria SC-006)
- Optional screen-space dithering for additional burn-in prevention (future enhancement)

### 3. Desktop Wallpaper Integration

**Decision**: Use **X11 window properties** for X11 systems with fallback to standard fullscreen window.

**Rationale**:
- Native X11 integration via EWMH (Extended Window Manager Hints)
- Can set window as desktop wallpaper by setting `_NET_WM_WINDOW_TYPE` to `_NET_WM_WINDOW_TYPE_DESKTOP`
- Wayland support via layer-shell protocol (through smithay ecosystem)
- Allows seamless background operation while other windows overlay

**Alternatives Considered**:
- Direct framebuffer manipulation: lower-level, harder to maintain across display servers
- Wallpaper plugin systems: tightly coupled to specific desktop environments (GNOME, KDE)
- Fullscreen window: simpler but user would need to minimize wallpaper to see other apps

**Implementation Strategy**:
- Detect display server (X11 vs. Wayland) at startup
- Set appropriate window properties for each server
- Handle display disconnections and reconfiguration gracefully

### 4. Input Event Handling

**Decision**: Use **winit event loop** with OS-native mouse event processing.

**Rationale**:
- Winit provides unified event handling for X11 and Wayland
- <16ms response time achievable with immediate event processing (success criteria SC-005)
- Supports mouse button and motion events without manual polling
- Integrates seamlessly with wgpu rendering loop

**Input Contracts**:
- Left-click drag: Pan camera in direction of drag (continuous update)
- Right-click: Trigger pulse effect at click coordinates (discrete event)
- No input blocking: Application ignores input when overlapped by other windows

### 5. Configuration Management

**Decision**: Use **TOML configuration file** stored in `~/.config/oled-wallpaper/config.toml`.

**Rationale**:
- Human-readable format for user-adjustable animation parameters
- Standard location on Linux ($XDG_CONFIG_HOME or ~/.config)
- Supports all required parameter types: floats (speed, scale), colors (RGB), integers (sizes)
- Validated at startup; invalid configs fall back to defaults

**Configurable Parameters**:
- `planet_speed`: Orbital speed multiplier (default: 1.0)
- `planet_colors`: Array of RGB tuples for each planet
- `planet_sizes`: Array of relative sizes for each planet
- `orbital_patterns`: Array of orbital eccentricity/inclination values
- `camera_zoom`: Default zoom level (default: 1.0)

### 6. Performance Optimization

**Decision**: Target **≥30 FPS minimum** (success criteria SC-001) with 60 FPS as optimal.

**Key Optimizations**:
- Single-pass rendering: combine all celestial bodies into one draw call
- Instanced rendering for planets (if >10 bodies; simplify for initial launch)
- Frame rate capping: limit to monitor refresh rate to reduce power consumption
- Continuous animation: no pause/resume logic required
- Memory pooling for pulse effect particles

**Expected Performance**:
- GTX 960 (mid-range 2015 GPU): 60+ FPS at 1080p
- Integrated graphics (modern mobile-class): 30+ FPS at 1080p
- Memory footprint: <100 MB for binary + data

### 7. Testing Strategy (TDD Emphasis)

**Decision**: **Test-First Development** (TDD) per user guidance: "ride tdd while we can"

**Testing Pyramid**:
```
           /\
          /  \        Contract Tests (desktop integration, wallpaper behavior)
         /----\       
        /      \      Integration Tests (scene rendering, physics + rendering together)
       /--------\
      /          \    Unit Tests (orbit calculations, input handling, config parsing)
     /____________\
```

**Test-First Workflow**:
1. Write test case (RED)
2. Get user approval on expected behavior
3. Test fails (RED)
4. Implement minimal code to pass test (GREEN)
5. Refactor for clarity (REFACTOR)

**Key Test Areas** (written before implementation):
- **Orbit Physics**: Planet positions match expected Kepler equations at known times
- **Input Handling**: Pan events update camera correctly; pulse events trigger at click point
- **Rendering**: Scene renders without crashes at multiple resolutions
- **Config Loading**: Valid configs load successfully; invalid configs error gracefully
- **Wallpaper Integration**: Window integrates with desktop; other windows overlay correctly

### 8. Deployment Strategy

**Decision**: **Flatpak** as primary distribution format (Flatseal sandboxing optional).

**Rationale**:
- Simplifies dependency management across Linux distributions
- Standard installation via GNOME Software, KDE Discover
- User-friendly installation experience (<2 minutes per success criteria SC-003)
- Sandboxing available (wallpaper needs monitor/input access; sockets required)

**Alternatives Considered**:
- **AppImage**: Self-contained binary; fewer dependencies, but larger file size
- **deb/rpm**: Distribution-specific; requires maintaining multiple packages
- **Snap**: Heavy containerization; may impact performance

**Flatpak Manifest Considerations**:
- Requires access to: display server, input devices, user home directory
- Runtime: org.freedesktop.Platform 23.08 (standard Linux base)
- Permissions: graphics, input, read-only home access

---

## Decision Tracking

| Decision | Owner | Status | Recorded |
|----------|-------|--------|----------|
| Graphics library: wgpu | Design | Approved | ✓ |
| Orbital mechanics: Kepler orbits | Design | Approved | ✓ |
| Wallpaper integration: X11/Wayland native | Design | Approved | ✓ |
| Input handling: winit events | Design | Approved | ✓ |
| Configuration: TOML files | Design | Approved | ✓ |
| Performance target: 30+ FPS | Design | Approved | ✓ |
| Testing: TDD (test-first) | User guidance | Approved | ✓ |
| Deployment: Flatpak primary | Design | Approved | ✓ |

---

## No Further Clarifications Required

All technical decisions are finalized. No blocking research items remain. Ready for Phase 1 design (data model, contracts, quickstart).
