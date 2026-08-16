# Feature Specification: Egui Configuration Tool

**Feature Branch**: `002-egui-configurator`

**Goal**: Provide a native GUI configuration application built with egui that ships with the Flatpak/AppImage and makes it easy for end users to configure the Interactive OLED Wallpaper (widgets, animation parameters, presets). The configurator must support Test-Driven Development (TDD) workflows and include end-to-end (E2E) validation scenarios.

## User Scenarios & Testing

### User Story 1 - Launch and Edit Settings (Priority: P1)
A user launches the Configurator app from the desktop menu and edits animation parameters (planet speed, colors, widget positions). Changes are saved and applied to the wallpaper via the shared configuration file.

Independent Test: Launch Configurator, change planet_speed to 2.0, save, then start wallpaper with demo flag and verify speed applied.

Acceptance:
- Configurator opens as a normal desktop app (Flatpak/AppImage).
- Settings are editable via labeled inputs and saved to `~/.config/oled-wallpaper/config.toml`.
- Wallpaper reads updated config on next launch or via live reload if supported.

### User Story 2 - Widget Management (Priority: P1)
User can enable/disable overlay widgets (clock, calendar), set float mode, and reposition widgets via a preview area.

Independent Test: Toggle clock widget off, save, start wallpaper with demo mode and verify clock absent.

Acceptance:
- Widget toggles persist in config file.
- Preview area reflects widget positions and float-mode animation.

### User Story 3 - Presets & Import/Export (Priority: P2)
Users can save named presets, import/export presets as TOML files, and apply presets immediately.

Independent Test: Create preset "Night Sky", export it, delete it locally, then import and verify preset exists and applies correctly.

Acceptance:
- Presets listed in UI with apply/export/delete actions.
- Export writes valid TOML; import validates and adds preset.

### User Story 4 - Accessibility & Validation (Priority: P2)
UI inputs must validate ranges and provide helpful inline error messages. Defaults are restored on invalid input.

Independent Test: Enter planet_speed = -1, attempt save — UI shows validation error and prevents save.

Acceptance:
- Inputs validate per constraints (see Data Model).
- Validation errors surfaced in UI and prevent invalid saves.

## Requirements
- FR-002-01: Configurator MUST be implemented in Rust using egui/eframe
- FR-002-02: MUST persist configuration to `~/.config/oled-wallpaper/config.toml` (TOML)
- FR-002-03: MUST allow enable/disable and position for widgets and support float mode
- FR-002-04: MUST include preset management (save/import/export)
- FR-002-05: MUST include unit tests (TDD), integration tests, and at least 2 E2E scenarios
- FR-002-06: MUST package as Flatpak/AppImage and be installable alongside the wallpaper
- FR-002-07: MUST provide a preview area that simulates wallpaper behavior for immediate feedback

## Constraints & Assumptions
- Flatpak sandboxing limits direct IPC; use shared config file as primary integration mechanism
- Target desktop Linux (X11/Wayland); no mobile/Windows/macOS support for v1
- Egui/eframe chosen for rapid Rust-native GUI development and small binary size
- Wallpaper watches config file for changes (live-reload optional but recommended)

## Success Criteria
- SC-002-01: Configurator launches and saves config; wallpaper applies changes on restart
- SC-002-02: At least two E2E tests that run in CI (headless or --demo flags) pass
- SC-002-03: UI validation prevents invalid values from being saved
- SC-002-04: Preset import/export round-trip works reliably

## Test Strategy (TDD & E2E)
- Unit tests: Validate config parsing, validation, preset serialization, and small UI logic helpers
- Integration tests: Run configurator CLI entrypoints (headless) to verify save/load flows
- E2E tests: Use `--demo N` mode for wallpaper and a headless CLI mode for configurator to apply settings, then assert wallpaper behavior (fps, presence/absence of widgets) via process logs or screenshot assertions

## Key Entities
- Configuration file (TOML)
- Preset (named collection of settings)
- Preview renderer (lightweight scene preview inside egui)
- Validation rules (per field)

## Implementation Notes
- Use `egui` + `eframe` for UI
- Use `config` crate or `toml` + `serde` for config load/save
- Provide a headless CLI mode: `oled-config --apply PRESET` for CI/E2E automation
- Prefer file-based integration to avoid Flatpak IPC restrictions


<!-- End of spec -->