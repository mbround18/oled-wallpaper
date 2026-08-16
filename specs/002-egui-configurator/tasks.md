# Tasks: 002-egui-configurator

**Feature**: Egui Configurator (`002-egui-configurator`)
**Input**: specs/002-egui-configurator/{spec.md,plan.md,data-model.md,research.md,quickstart.md}

## Phase 1: Setup (Shared Infrastructure)

- [X] T001 Update Cargo.toml to add a new binary `oled-config` and dependencies: `eframe`, `egui`, `serde`, `serde_derive`, `toml`, `directories`, `assert_cmd` (dev), `tempfile` (dev)
- [X] T002 Create binary entrypoint at src/bin/oled-config.rs with basic eframe app skeleton and `--headless` CLI flag handling
- [X] T003 Create config module scaffold at src/config/mod.rs and src/config/config.rs implementing Config struct (serde) and load/save function signatures
- [X] T004 Add tests scaffolding directories: tests/unit/, tests/integration/, tests/e2e/ and a CI script .github/workflows/e2e.yml placeholder

## Phase 2: Foundational (Blocking Prerequisites)

- [X] T005 Implement load_config(path: &Path) -> Result<Config> in src/config/config.rs with TOML parsing and schema validation (atomic read semantics)
- [X] T006 Implement save_config(path: &Path, config: &Config) -> Result<()> in src/config/config.rs using atomic write (temp file + rename)
- [X] T007 Implement preset serialization helpers in src/config/presets.rs with export_preset() and import_preset()
- [X] T008 Implement CLI headless apply in src/bin/oled-config.rs: `--headless --apply <preset> --config-dir <dir>` that writes config and exits (used by E2E tests)
- [X] T009 Write unit tests for config load/save and preset import/export in tests/unit/test_config.rs
- [X] T010 Document config schema and file location in specs/002-egui-configurator/contracts/configuration-contract.md (ensure atomics and validation noted)

> Checkpoint: Foundational tasks complete — blocks user stories until done

---

## Phase 3: User Story 1 - Launch and Edit Settings (Priority: P1) 🎯 MVP

**Goal**: Provide a runnable GUI that opens, shows animation controls (planet_speed, camera_zoom, colors), and saves to `$XDG_CONFIG_HOME/oled-wallpaper/config.toml`.

**Independent Test**: Launch GUI, change `planet_speed` to 2.0, click Save, then run wallpaper in demo mode with same config-dir and verify wallpaper logs or demo shows updated speed.

### Tests (TDD-first)
- [ ] T011 [P] [US1] Unit tests for config validation of `planet_speed` and `camera_zoom` in tests/unit/test_validation.rs
- [ ] T012 [P] [US1] Integration test for headless apply flow in tests/integration/test_headless_apply.rs using `assert_cmd` to run `oled-config --headless --apply "test" --config-dir /tmp/test-config`

### Implementation
- [X] T013 [US1] Implement UI form for animation settings at src/configurator/ui.rs (planet_speed, camera_zoom, color pickers) and wire to Config struct
- [X] T014 [US1] Implement Save button behavior to call save_config with path override support (use XDG_CONFIG_HOME or --config-dir) in src/configurator/ui.rs
- [X] T015 [US1] Wire eframe app in src/bin/oled-config.rs to launch the GUI (non-headless path) and load initial config on startup
- [X] T016 [US1] Add integration test that runs wallpaper in `--demo 5 --config-dir /tmp/test-config` and asserts logs contain applied planet_speed value (tests/e2e/e2e_us1_apply.sh)
- [X] T017 [P] [US1] Add UI unit tests (where possible) for small helpers (e.g., normalize_color(), clamp_zoom()) in tests/unit/test_ui_helpers.rs
- [X] T018 [US1] Add README entry docs/quickstart-configurator.md with steps to launch GUI and save config (link from quickstart.md)
- [X] T019 [P] [US1] Add "Our Usage" live graphs section (CPU & Memory) in src/configurator/ui.rs using sysinfo + egui::plot


> Checkpoint: US1 complete — GUI saves config and wallpaper can read it

---

## Phase 4: User Story 2 - Widget Management (Priority: P1)

**Goal**: Enable/disable overlay widgets, set float mode, and reposition widgets via preview area; persisted to config and visible in wallpaper demo.

**Independent Test**: Disable clock widget in GUI, save, run wallpaper with demo config-dir, verify clock not present.

### Tests (TDD-first)
- [X] T019 [P] [US2] Unit tests for widget config serialization in tests/unit/test_widgets.rs
- [X] T020 [P] [US2] Integration test that toggles widget in headless mode: `oled-config --headless --apply "widget-off" --config-dir /tmp/test-config` and wallpaper `--demo` asserts absence of widget in logs or screenshot diff

### Implementation
- [X] T021 [US2] Implement widget controls UI in src/configurator/widgets.rs (toggle, x/y position inputs, float_mode checkbox)
- [X] T022 [US2] Implement preview rendering in src/configurator/preview.rs that simulates widget positions and float-mode animation (lightweight; reuse egui painting)
- [X] T023 [US2] Persist widget positions to Config and ensure save/load uses these fields (src/config/config.rs + src/config/presets.rs)

> Checkpoint: US2 complete — Widget toggles and positions persisted and previewed

---

## Phase 5: User Story 3 - Presets & Import/Export (Priority: P2)

**Goal**: Save named presets, import/export TOML, and apply presets immediately via UI and headless CLI

**Independent Test**: Create preset "Night Sky" in GUI, export, delete locally, import, apply, and run wallpaper demo to verify applied preset

### Tests (TDD-first)
- [ ] T024 [P] [US3] Unit tests for preset round-trip serialization in tests/unit/test_presets.rs
- [ ] T025 [P] [US3] Integration test for export/import via CLI in tests/integration/test_preset_cli.rs

### Implementation
- [ ] T026 [US3] Add preset UI in src/configurator/presets.rs (create, list, apply, export, import)
- [ ] T027 [US3] Implement export/import handlers in src/config/presets.rs using atomic write and validation
- [ ] T028 [US3] Wire preset apply to live-update config file path used by wallpaper demo (or fail gracefully if live-reload unsupported)

> Checkpoint: US3 complete — Preset save/import/export works for CI and users

---

## Phase 6: User Story 4 - Accessibility & Validation (Priority: P2)

**Goal**: Validate inputs (ranges), show inline errors in UI, prevent save on invalid values

**Independent Test**: Enter planet_speed=-1 in UI, attempt Save — UI shows validation error and prevents writing config

### Tests (TDD-first)
- [ ] T029 [P] [US4] Unit tests for validation logic in tests/unit/test_validation_ranges.rs

### Implementation
- [ ] T030 [US4] Implement validation helpers in src/config/validation.rs and integrate with UI controls to show inline errors and disable Save when invalid

> Checkpoint: US4 complete — UI prevents invalid saves and shows helpful messages

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T031 [P] Add Flatpak manifest or update existing manifest to include `oled-config` binary and required filesystem permissions (manifest ninja.boop.OledWallpaper.yml)
- [ ] T032 [P] Add CI E2E workflow: .github/workflows/e2e.yml runs wallpaper `--demo` and headless configurator to perform two E2E scenarios (US1 apply, US2 widget toggle) and asserts logs/screenshot diffs
- [ ] T033 [P] Documentation: update README.md with Configurator usage, presets, and Flatpak packaging notes
- [ ] T034 [P] Run quickstart.md validation steps and fix any issues found

---

## Dependencies & Execution Order

- Phase 1 (T001-T004) must run first to prepare project structure
- Phase 2 (T005-T010) foundational tasks BLOCK user story implementation until complete
- User Story phases (T011-T030) depend on Phase 2 completion and are organized by priority (P1 → P2)
- Polish tasks (T031-T034) run after core stories or in parallel where safe

## Parallel Opportunities

- Tasks marked [P] (T002, T011, T012, T017, T019, T020, T024, T025, T029, T031-T034) can run in parallel
- Unit tests, model implementations, and UI helper tasks are good parallel work
- Different user stories (US1, US2, US3, US4) can be implemented in parallel after foundational phase

## Independent Test Criteria (per story)

- US1: GUI can save `planet_speed` and wallpaper `--demo` reflects new speed (logs or measured behavior)
- US2: GUI can disable clock widget and wallpaper `--demo` shows no clock
- US3: Preset export/import round-trip restores named preset and applies config
- US4: Invalid inputs prevented from saving; validation tests pass

## Suggested MVP Scope

- Deliver US1 (Launch & Edit Settings) as MVP
- Foundation (config load/save + headless CLI) + US1 provides a usable product for users and CI

## Format Validation

- All tasks follow the checklist format `- [ ] T### [P?] [US?] Description with file path`


---

Generated: /home/mbruno/development/oled-wallpaper/specs/002-egui-configurator/tasks.md
Total tasks: 34
Tasks per story/phase:
- Phase 1 (Setup): 4
- Phase 2 (Foundational): 6
- US1 (P1): 8
- US2 (P1): 5
- US3 (P2): 6
- US4 (P2): 2
- Polish & Cross-cutting: 4

MVP: User Story 1 (Phase 3) — implement GUI save/load and headless apply

