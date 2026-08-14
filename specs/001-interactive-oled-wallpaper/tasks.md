# Tasks: Interactive OLED Wallpaper

**Input**: Design documents from `/specs/001-interactive-oled-wallpaper/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Test tasks included. Tests are OPTIONAL - only include them if explicitly requested. **Per user guidance, TDD approach implemented: Tests written FIRST, then implementation.**

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `- [ ] [ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown below use project structure from plan.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

**Checkpoint**: Basic Rust project scaffold ready for feature development

- [X] T001 Create project structure per implementation plan (`src/renderer/`, `src/physics/`, `src/input/`, `src/config/`, `src/wallpaper/`, `tests/unit/`, `tests/integration/`, `tests/contract/`)
- [X] T002 [P] Initialize Cargo.toml with core dependencies: `wgpu`, `winit`, `glam`, `toml`, `serde`
- [X] T003 [P] Configure Cargo.toml with release optimization flags for performance
- [X] T004 [P] Setup CI/CD: GitHub Actions workflow for running tests and building releases
- [X] T005 [P] Configure linting: `rustfmt` and `clippy` configuration files
- [X] T006 Create Flatpak manifest (org.example.OledWallpaper.yml) in repository root with core permissions
- [X] T007 Create AppImage build configuration (if applicable) for alternate distribution method

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**Checkpoint**: Foundation ready - all user stories can proceed in parallel after this phase

- [X] T008 Create logger/tracing setup in `src/lib.rs` with RUST_LOG environment variable support
- [X] T009 Create error handling types in `src/error.rs` with custom Error enum and Result type alias
- [X] T010 [P] Implement configuration loading framework in `src/config/mod.rs` (TOML parsing, default fallback, validation framework)
- [X] T011 [P] Implement math utilities in `src/math.rs` (vector operations, matrix transformations needed by rendering and physics)
- [X] T012 [P] Create display server detection in `src/wallpaper/mod.rs` (X11 vs Wayland detection with fallback)
- [X] T013 Create window manager initialization in `src/wallpaper/integration.rs` for both X11 EWMH and Wayland layer-shell
- [X] T014 [P] Create base Scene struct in `src/renderer/scene.rs` as container for all entities
- [X] T015 Setup test infrastructure in `tests/common.rs` with helper functions and mock objects
- [X] T016 Create main.rs application entry point with graceful shutdown handling

---

## Phase 3: User Story 1 - View Dynamic Galaxy Wallpaper (Priority: P1) 🎯 MVP

**Goal**: Render a continuously animated galaxy scene with sun and orbiting planets as desktop wallpaper

**Independent Test**: Launch application and verify animated galaxy displays with planets visibly moving after 30 seconds

### Contract Tests for User Story 1 ⚠️ TDD

> **CRITICAL: Write these tests FIRST, verify they FAIL before implementation**

- [X] T017 [P] Create contract test for rendering in `tests/contract/desktop_integration.rs`: verify wallpaper renders full-screen at 1920x1080 and 2560x1440 resolutions
- [X] T018 [P] Create contract test for OLED burn-in prevention in `tests/contract/desktop_integration.rs`: verify no pixel region remains unchanged for >15 minutes (simulation)
- [X] T019 [P] Create contract test for animation performance in `tests/contract/desktop_integration.rs`: verify ≥30 FPS rendering sustained over 60-second run

### Data Models for User Story 1 ⚠️ TDD

- [X] T020 [P] [US1] Create CelestialBody struct in `src/physics/body.rs` with attributes: id, position, velocity, radius, color, mass, is_static
- [X] T021 [P] [US1] Create Orbit struct in `src/physics/orbit.rs` with Kepler orbit parameters (semi_major_axis, eccentricity, inclination, orbital_period, etc.)
- [X] T022 [P] [US1] Implement CelestialBody validation in `src/physics/body.rs`: radius > 0, mass > 0, color alpha > 0, exactly one static sun body
- [X] T023 [P] [US1] Implement Orbit validation in `src/physics/orbit.rs`: eccentricity 0.0-0.99, semi_major_axis > 0, orbital_period > 0, Kepler's third law check

### Unit Tests for Physics (US1) ⚠️ TDD

- [X] T024 [P] [US1] Create unit tests in `tests/unit/physics_tests.rs`: test Kepler orbit position calculation at known times
- [X] T025 [P] [US1] Create unit tests in `tests/unit/physics_tests.rs`: test orbital velocity calculation
- [X] T026 [P] [US1] Create unit tests in `tests/unit/physics_tests.rs`: test celestial body state transitions and updates
- [X] T027 [P] [US1] Create unit tests in `tests/unit/physics_tests.rs`: test CelestialBody and Orbit validation rules with edge cases

### Physics Engine Implementation for User Story 1

- [X] T028 [US1] Implement Kepler orbit equations in `src/physics/orbit.rs`: `get_position_at_time()` using parametric ellipse formulas
- [X] T029 [US1] Implement orbital velocity calculation in `src/physics/orbit.rs`: `get_velocity_at_time()` as derivative of position
- [X] T030 [P] [US1] Implement CelestialBody update logic in `src/physics/body.rs`: `update_position(delta_time)` using velocity
- [X] T031 [US1] Create physics simulation coordinator in `src/physics/mod.rs`: `update_all_bodies()` orchestrating per-body updates

### Rendering Setup for User Story 1

- [X] T032 [P] [US1] Implement Camera/Viewport struct in `src/renderer/camera.rs` with position, zoom, viewport dimensions
- [X] T033 [P] [US1] Implement coordinate transformation methods in `src/renderer/camera.rs`: `screen_to_world()` and `world_to_screen()`
- [X] T034 [P] [US1] Create wgpu render pipeline in `src/renderer/mod.rs`: initialize GPU context, render pass, buffer management
- [X] T035 [P] [US1] Create shader files: `src/shaders/celestial_body.wgsl` (render circles for sun/planets with color)
- [X] T036 [US1] Implement celestial body rendering in `src/renderer/scene.rs`: render all bodies to screen space with transformations

### Integration Tests for Rendering (US1) ⚠️ TDD

- [X] T037 [P] [US1] Create integration test in `tests/integration/wallpaper_integration.rs`: render scene for 60 frames, verify FPS ≥30
- [X] T038 [P] [US1] Create integration test in `tests/integration/wallpaper_integration.rs`: verify planet positions update correctly each frame
- [X] T039 [US1] Create integration test in `tests/integration/wallpaper_integration.rs`: verify rendering pipeline doesn't crash with various planet counts

### Application Loop and Window Setup for User Story 1

- [X] T040 [US1] Implement event loop in `src/main.rs`: window creation, render loop, frame timing (target 60 FPS, cap to monitor refresh rate)
- [X] T041 [US1] Integrate Scene into main.rs: instantiate with sun and 2 planets, update each frame, pass to renderer
- [X] T042 [US1] Implement wallpaper window properties in `src/wallpaper/integration.rs`: set X11 EWMH and Wayland layer-shell attributes

### Configuration for User Story 1

- [X] T043 [US1] Create default animation configuration in `src/config/animation.rs`: sun color, planet colors, planet sizes, orbital speeds
- [X] T044 [US1] Implement configuration loading in `src/config/mod.rs`: read from `~/.config/oled-wallpaper/config.toml` with fallback to defaults
- [X] T045 [US1] Implement configuration validation in `src/config/animation.rs`: validate color ranges, speed ranges, array sizes

### Unit Tests for Configuration (US1) ⚠️ TDD

- [X] T046 [P] [US1] Create unit tests in `tests/unit/config_tests.rs`: valid TOML loads correctly
- [X] T047 [P] [US1] Create unit tests in `tests/unit/config_tests.rs`: invalid TOML falls back to defaults with warning
- [X] T048 [P] [US1] Create unit tests in `tests/unit/config_tests.rs`: out-of-range values clamped to valid ranges

### Manual Testing for User Story 1

- [X] T049 [US1] Run Scenario 1 from quickstart.md: launch app, observe animation for 10s, verify smooth rendering
- [X] T050 [US1] Run performance profiling: verify FPS ≥30, memory <100MB, no crashes over 5-minute run

**Checkpoint**: User Story 1 should be fully functional and independently testable. Wallpaper displays animated galaxy with planets orbiting sun.

---

## Phase 4: User Story 2 - Interactive Panning with Mouse (Priority: P1)

**Goal**: Enable users to pan the galaxy view with left-click drag interactions

**Independent Test**: Click-drag wallpaper left/right/up/down, verify scene pans in expected direction and continues animating

### Contract Tests for User Story 2 ⚠️ TDD

- [ ] T051 [P] Create contract test in `tests/contract/desktop_integration.rs`: left-click drag pans scene; verify <16ms latency (SC-005)
- [ ] T052 [P] Create contract test in `tests/contract/desktop_integration.rs`: panning stops immediately on mouse release
- [ ] T053 Create contract test in `tests/contract/desktop_integration.rs`: planets continue orbiting during panning

### Input Handling for User Story 2 ⚠️ TDD

- [ ] T054 [P] [US2] Create mouse input event handler in `src/input/mouse.rs`: listen for `MouseMoved` events with button 1 pressed
- [ ] T055 [P] [US2] Implement pan calculation in `src/input/mouse.rs`: track previous position, calculate delta, call `camera.pan_by(delta)`
- [ ] T056 [US2] Add input handling to main event loop in `src/main.rs`: forward mouse events to input handler, update camera state

### Camera Panning Logic for User Story 2

- [ ] T057 [P] [US2] Implement `Camera.pan_by()` in `src/renderer/camera.rs`: update camera position by delta
- [ ] T058 [P] [US2] Implement pan bounds enforcement in `src/renderer/camera.rs`: `clamp_pan_bounds()` to prevent panning too far
- [ ] T059 [US2] Implement pan wrapping option in `src/renderer/camera.rs`: optional scene wrapping at edges (alternative to clamping)

### Unit Tests for Input Handling (US2) ⚠️ TDD

- [ ] T060 [P] [US2] Create unit tests in `tests/unit/input_tests.rs`: mouse drag events calculate correct delta
- [ ] T061 [P] [US2] Create unit tests in `tests/unit/input_tests.rs`: camera pan updates position correctly
- [ ] T062 [P] [US2] Create unit tests in `tests/unit/input_tests.rs`: pan bounds enforcement works at scene edges

### Unit Tests for Camera (US2) ⚠️ TDD

- [ ] T063 [P] [US2] Create unit tests in `tests/unit/renderer_tests.rs`: coordinate transformations are correct
- [ ] T064 [P] [US2] Create unit tests in `tests/unit/renderer_tests.rs`: camera pan updates view matrix correctly

### Integration Tests for Panning (US2) ⚠️ TDD

- [ ] T065 [P] [US2] Create integration test in `tests/integration/wallpaper_integration.rs`: perform mouse drag, verify scene pans and planets still animate
- [ ] T066 [US2] Create integration test in `tests/integration/wallpaper_integration.rs`: rapid mouse movements don't cause stuttering or input lag

### Manual Testing for User Story 2

- [ ] T067 [US2] Run Scenario 2 from quickstart.md: click-drag left/right/up/down, verify responsive panning
- [ ] T068 [US2] Test pan at scene edges: verify graceful bound enforcement or wrapping behavior
- [ ] T069 [US2] Measure input latency: drag mouse, verify visual response <16ms

**Checkpoint**: User Stories 1 AND 2 should both work independently. Wallpaper displays animated galaxy with full panning capability.

---

## Phase 5: User Story 3 - Pulse Effect with Right-Click (Priority: P1)

**Goal**: Enable users to trigger visual pulse effects with right-click

**Independent Test**: Right-click wallpaper at various positions, verify pulse effects appear and animate smoothly for ~1.5 seconds

### Contract Tests for User Story 3 ⚠️ TDD

- [ ] T070 [P] Create contract test in `tests/contract/desktop_integration.rs`: right-click triggers pulse at click coordinates
- [ ] T071 [P] Create contract test in `tests/contract/desktop_integration.rs`: pulse animation completes within 1.5 seconds (SC-004)
- [ ] T072 Create contract test in `tests/contract/desktop_integration.rs`: multiple concurrent pulses render without artifacts

### Data Models for User Story 3 ⚠️ TDD

- [ ] T073 [P] [US3] Create PulseEffect struct in `src/renderer/effects.rs` with attributes: id, origin, elapsed_time, duration, max_radius, intensity, color, is_active
- [ ] T074 [P] [US3] Implement PulseEffect validation in `src/renderer/effects.rs`: duration > 0, max_radius > 0, intensity 0.0-1.0, elapsed_time ≤ duration

### Unit Tests for Pulse Effects (US3) ⚠️ TDD

- [ ] T075 [P] [US3] Create unit tests in `tests/unit/renderer_tests.rs`: pulse progress calculation (0.0 → 1.0)
- [ ] T076 [P] [US3] Create unit tests in `tests/unit/renderer_tests.rs`: pulse radius interpolation over time
- [ ] T076 [P] [US3] Create unit tests in `tests/unit/renderer_tests.rs`: pulse alpha fade calculation

### Input Handling for User Story 3

- [ ] T077 [P] [US3] Extend mouse input handler in `src/input/mouse.rs`: listen for `MouseButton(Button::Right, Action::Press)` events
- [ ] T078 [P] [US3] Implement pulse trigger in input handler: call `scene.trigger_pulse(click_position)` on right-click
- [ ] T079 [US3] Integrate right-click handling into main event loop in `src/main.rs`

### Pulse Rendering for User Story 3

- [ ] T080 [P] [US3] Create shader for pulse effects in `src/shaders/pulse.wgsl`: render expanding circle with alpha fade
- [ ] T081 [P] [US3] Implement pulse rendering in `src/renderer/effects.rs`: calculate current radius and alpha, render to screen
- [ ] T082 [US3] Integrate pulse rendering into main scene renderer in `src/renderer/scene.rs`: render all active pulses each frame

### Scene Integration for User Story 3

- [ ] T083 [US3] Add pulse effect pool to Scene in `src/renderer/scene.rs`: `pulse_effects: Vec<PulseEffect>`
- [ ] T084 [US3] Implement `Scene.trigger_pulse()` in `src/renderer/scene.rs`: create new PulseEffect at coordinates, add to pool
- [ ] T085 [US3] Implement pulse update and cleanup in `src/renderer/scene.rs`: update all pulses each frame, remove finished ones

### Unit Tests for Scene Pulse Management (US3) ⚠️ TDD

- [ ] T086 [P] [US3] Create unit tests in `tests/unit/renderer_tests.rs`: trigger_pulse creates effect at correct coordinates
- [ ] T087 [P] [US3] Create unit tests in `tests/unit/renderer_tests.rs`: finished pulses are removed from pool
- [ ] T088 [P] [US3] Create unit tests in `tests/unit/renderer_tests.rs`: multiple concurrent pulses coexist without interference

### Integration Tests for Pulse Effects (US3) ⚠️ TDD

- [ ] T089 [P] [US3] Create integration test in `tests/integration/wallpaper_integration.rs`: right-click triggers pulse; verify animation plays
- [ ] T090 [P] [US3] Create integration test in `tests/integration/wallpaper_integration.rs`: rapid right-clicks create multiple pulses; all animate correctly
- [ ] T091 [US3] Create integration test in `tests/integration/wallpaper_integration.rs`: pulse effects don't interfere with animation or panning

### Manual Testing for User Story 3

- [ ] T092 [US3] Run Scenario 3 from quickstart.md: right-click at screen center, verify pulse appears and fades in ~1.5s
- [ ] T093 [US3] Test rapid right-clicks: verify multiple pulses animate simultaneously without visual artifacts
- [ ] T094 [US3] Test pulse at screen edges: verify pulse renders correctly at all screen positions

**Checkpoint**: All three core features (animation, panning, pulse) should now work independently and together. MVP feature complete.

---

## Phase 6: User Story 4 - Deployable as Flatpak/Applet (Priority: P1)

**Goal**: Package application as distributable Flatpak and/or AppImage

**Independent Test**: Install via Flatpak, launch from app menu, verify wallpaper functions identically to source build

### Build Configuration for User Story 4

- [ ] T095 [US4] Finalize Flatpak manifest at `org.example.OledWallpaper.yml`: add finish-args for display, input, config directory permissions
- [ ] T096 [US4] Create Flatpak build script in `build-flatpak.sh`: orchestrates `flatpak-builder` commands
- [ ] T097 [US4] Test Flatpak build locally: `flatpak-builder --user build org.example.OledWallpaper.yml`
- [ ] T098 [US4] Create AppImage build configuration (optional): if building AppImage format alternative
- [ ] T099 [US4] Create release build in Cargo.toml: optimize for size and performance (`opt-level = 3`, `lto = true`)

### Deployment & Launch Testing for User Story 4

- [ ] T100 [US4] Install Flatpak app: `flatpak build-install build org.example.OledWallpaper`
- [ ] T101 [US4] Verify Flatpak installation: `flatpak list --app | grep OledWallpaper`
- [ ] T102 [US4] Launch via Flatpak: `flatpak run org.example.OledWallpaper` - verify wallpaper appears
- [ ] T103 [US4] Verify Flatpak permissions: all graphics/input/config access working without permission errors
- [ ] T104 [US4] Test uninstall: `flatpak uninstall org.example.OledWallpaper` - clean removal with no artifacts

### Manual Testing for User Story 4

- [ ] T105 [US4] Run Scenario 6 from quickstart.md: Flatpak deployment testing (build, install, launch, feature validation)
- [ ] T106 [US4] Measure performance under Flatpak: verify FPS, latency, memory footprint match source build

**Checkpoint**: Application is now packaged and distributable via Flatpak. Users can install without source compilation.

---

## Phase 7: Feature Completion - Configuration & Polish

**Purpose**: Complete FR-010 (configurable parameters), polish, and cross-cutting concerns

**Checkpoint**: All user stories complete with full configurability

- [ ] T107 [P] Complete configuration parameter support in `src/config/animation.rs`: planet_speed, planet_colors, planet_sizes, orbital_patterns, camera_zoom, pulse_color, pulse_intensity
- [ ] T108 [P] Implement per-planet configuration override in `src/config/animation.rs`: ability to customize eccentricity, inclination, period multipliers per planet
- [ ] T109 [P] Create default config generator in `src/config/mod.rs`: if config file missing, create ~/.config/oled-wallpaper/config.toml with sensible defaults
- [ ] T110 [US4] Document configuration file format in `docs/CONFIG.md`: TOML schema, all parameters, ranges, defaults
- [ ] T111 [P] Add logging for all critical paths in `src/`: startup, rendering, input handling, config loading
- [ ] T112 [P] Code cleanup and refactoring: remove TODO/FIXME comments, improve code clarity and documentation
- [ ] T113 [P] [P] Unit test coverage for edge cases: empty config, malformed TOML, extreme parameter values
- [ ] T114 Run full integration test suite: `cargo test --all` - all tests pass
- [ ] T115 Run formatter: `cargo fmt --all` - all code formatted correctly
- [ ] T116 Run linter: `cargo clippy --all` - all warnings addressed
- [ ] T117 Run quickstart.md Scenario 1-6 validation: all acceptance scenarios pass manually
- [ ] T118 Document build and installation in `README.md`: source build, Flatpak installation, quickstart usage
- [ ] T119 Create troubleshooting guide in `docs/TROUBLESHOOTING.md` based on edge cases encountered during development

---

## Phase 8: Final Validation & Release

**Purpose**: Last verification before v1 release

**Checkpoint**: Feature complete, tested, documented, ready for distribution

- [ ] T120 Performance profiling and optimization: run under `perf`, identify hotspots, optimize if needed
- [ ] T121 Memory leak detection: run under `valgrind` or `sanitizers`, verify no leaks
- [ ] T122 Long-run test: run application continuously for 8 hours, verify OLED burn-in prevention works
- [ ] T123 Multi-resolution testing: run at 1920x1080, 2560x1440, 3840x2160, verify rendering correct at all resolutions
- [ ] T124 Create release notes in `RELEASE_NOTES.md`: features, performance metrics, known limitations
- [ ] T125 Build final release binary: `cargo build --release`, sign if applicable
- [ ] T126 Create GitHub release: tag version v0.1.0, attach Flatpak manifest and release notes
- [ ] T127 Smoke test: install from release artifacts, run through all scenarios one final time

---

## Dependencies & Execution Order

### Phase Dependencies

```
Setup (Phase 1)
    ↓
Foundational (Phase 2) ← CRITICAL GATE
    ↓
    ├─→ User Story 1 (Phase 3) [P1]  ─→ US1 Complete
    ├─→ User Story 2 (Phase 4) [P1]  ─→ US2 Complete
    ├─→ User Story 3 (Phase 5) [P1]  ─→ US3 Complete
    └─→ User Story 4 (Phase 6) [P1]  ─→ US4 Complete
    ↓
Polish & Features (Phase 7)
    ↓
Final Validation (Phase 8) ← RELEASE GATE
```

### User Story Dependencies

| Story | Depends On | Can Start After |
|-------|-----------|-----------------|
| US1 | Foundational | Phase 2 complete |
| US2 | Foundational, US1 (optional integration) | Phase 2 complete (US1 recommended for testing context) |
| US3 | Foundational, US1 (optional integration) | Phase 2 complete (US1 recommended for testing context) |
| US4 | All prior stories | Phase 6 start (needs working binary) |

**Key**: US2 and US3 can start immediately after Foundational, even if US1 in progress. Each is independently testable.

### Parallel Opportunities

**Within Setup (Phase 1)**:
- T002, T003, T004, T005 all run in parallel (different cargo config aspects)

**Within Foundational (Phase 2)**:
- T010, T011, T012, T014, T015 all run in parallel (different modules)
- T008, T009 depend on T001 structure existing

**Within User Story 1**:
- All contract tests (T017-T019) can run in parallel
- All data model tasks (T020-T023) can run in parallel
- All physics unit tests (T024-T027) can run in parallel
- Rendering setup (T032-T035) can run in parallel
- T028-T031 are sequential (depend on earlier physics work)

**Between User Stories** (after Foundational):
- US1, US2, US3 can be worked on in parallel by different developers
- US4 only needs any working build (doesn't need all stories complete)

**Within Polish (Phase 7)**:
- T107, T108, T109, T111-T113 can run in parallel
- T114-T119 are sequential checks

### Execution Strategy for Different Team Sizes

**Solo Developer (Recommended MVP Path)**:
1. Complete Setup (Phase 1): 1 day
2. Complete Foundational (Phase 2): 1 day
3. Complete US1 (Phase 3): 2-3 days (core feature)
4. **STOP and validate** - wallpaper fully functional
5. Complete US2 (Phase 4): 1 day (add panning)
6. Complete US3 (Phase 5): 1 day (add pulse)
7. Complete US4 (Phase 6): 1 day (package)
8. Complete Polish + Validation (Phases 7-8): 1 day

**Total**: ~9-10 days for full v0.1.0 release

**Three Developers**:
1. All work together: Setup + Foundational (Phases 1-2)
2. Split parallel work:
   - Dev A: US1 (animation + rendering)
   - Dev B: US2 (panning input)
   - Dev C: US3 (pulse effects)
3. Reunite: US4 (packaging) + Polish + Validation

**Total**: ~5-6 days for full v0.1.0 release

---

## Implementation Strategy

### MVP First (Recommended for Validation)

**Stop after Phase 3 (User Story 1)**:

At this point:
- ✅ Dynamic galaxy renders with sun and planets
- ✅ Planets orbit smoothly (≥30 FPS)
- ✅ OLED burn-in prevention working
- ✅ Configuration supported
- ✅ Desktop wallpaper integration functional

**Minimum viable product is complete**. Can validate user satisfaction, gather feedback.

### Incremental Delivery

1. **Release v0.1.0** after Phase 3 + Polish + Validation:
   - Core animation feature complete
   - Source build only (Flatpak optional)

2. **Release v0.2.0** after Phase 4:
   - Add panning interactivity
   - Fully interactive now

3. **Release v0.3.0** after Phase 5:
   - Add pulse effects
   - All core features complete

4. **Release v1.0.0** after Phase 6:
   - Flatpak packaging complete
   - Ready for distribution via package managers

### Parallel Team Strategy (Multiple Developers)

1. **Weeks 1-2**: All team members complete Setup + Foundational together
   - Establish project baseline
   - Ensure everyone understands architecture

2. **Weeks 3-4**: Teams split and work in parallel:
   - **Team A** (2 devs): Complete US1 + US2 (rendering + input)
   - **Team B** (1 dev): Complete US3 (pulse effects)

3. **Week 5**: Reunite for integration testing and polish

4. **Week 6**: Complete packaging (US4), final validation, release

---

## Test Coverage Summary

### Per User Story

| Story | Contract Tests | Unit Tests | Integration Tests |
|-------|--------------|------------|-------------------|
| US1 | 3 (rendering, burn-in, perf) | 8 (physics, config) | 3 (render, animation, crash) |
| US2 | 3 (pan latency, release, animate) | 5 (input, camera) | 2 (drag, latency) |
| US3 | 3 (trigger, duration, multiple) | 4 (pulse calc, state) | 3 (pulse, concurrent, interference) |
| US4 | 0 (manual only) | 0 | 0 |
| **Total** | **9 contract** | **17 unit** | **8 integration** |

**Total Test Count**: 34 test tasks across TDD approach

### Test Execution

```bash
# Run all tests
cargo test --all

# Run specific story tests
cargo test us1  # US1 tests only
cargo test contract  # Contract tests only
cargo test --test wallpaper_integration  # Integration tests

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

---

## Summary

- **Total Tasks**: 127 actionable tasks
- **Setup**: 7 tasks
- **Foundational**: 9 tasks
- **User Story 1 (P1 - MVP)**: 34 tasks (17 tests, 17 implementation)
- **User Story 2 (P1 - Panning)**: 19 tasks (9 tests, 10 implementation)
- **User Story 3 (P1 - Pulse)**: 25 tasks (11 tests, 14 implementation)
- **User Story 4 (P1 - Deployment)**: 12 tasks (manual testing)
- **Polish & Features**: 13 tasks
- **Final Validation**: 8 tasks

**MVP Scope** (Phase 1-3 + Polish): ~60 tasks, ~1-2 weeks solo developer

**Full Feature** (All phases): ~127 tasks, ~9-10 days solo or 5-6 days with team

**TDD Emphasis**: 37 test tasks written FIRST before implementation tasks

**Parallel Ready**: 45+ tasks marked [P] for parallel execution across team or developer context switching
