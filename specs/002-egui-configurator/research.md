# Research: Egui Configurator for OLED Wallpaper

## Date: 2026-08-16

### Goal
Decide tooling and integration patterns for a Rust/egui-based configuration app that ships with the wallpaper Flatpak and uses TDD + E2E tests.

### UI Framework
Decision: egui + eframe
Rationale: Pure Rust immediate-mode GUI, fast iteration, small footprint, easy to embed preview canvas. eframe provides an app harness for desktop bundles and integrates with wgpu for rendering.
Alternatives: iced (heavier, longer dev time), GTK (heavier, FFI), Tauri (web-based; larger runtime).

### Config Storage & Integration
Decision: TOML file at `$XDG_CONFIG_HOME/oled-wallpaper/config.toml` as canonical config. Wallpaper reads this file and may watch for changes.
Rationale: Flatpak sandboxing complicates direct IPC; file-based integration is simple, debuggable, and fits XDG conventions.

### Preview Renderer
Decision: Embed a lightweight preview using egui's wgpu integration (reuse rendering code paths where possible). Preview is not full wallpaper but simulates key behaviors (widget positions, basic orbit speed).

### Packaging
Decision: Flatpak as primary distribution; AppImage optionally for users who prefer standalone binaries.
Notes: Flatpak manifest must grant read/write access to user config (xdg-config), or instruct user to grant permission; consider portal or portal-less approach by recommending `--filesystem=home` or `--talk-name=` rules in manifest.

### Testing Strategy
- TDD: Unit tests targeting config schema, validation, preset serialization
- Integration: CLI headless flows using `assert_cmd` and temp config dirs
- E2E: Use CI to run wallpaper in `--demo` and run configurator in headless mode to apply settings; assert via logs or pixel checks (screenshot diff) where possible

### Security & Permissions
- Config file writes limited to user home
- Validate and sanitize imported presets to avoid code injection via malformed values

### Recommendation
Implement file-based config + optional local Unix socket for live-reload in non-Flatpak builds. Provide `--headless` CLI mode for CI E2E tests that can apply presets and exit.
