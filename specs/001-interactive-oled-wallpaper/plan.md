# Implementation Plan: Interactive OLED Wallpaper

**Branch**: `001-interactive-oled-wallpaper` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-interactive-oled-wallpaper/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Build an interactive desktop wallpaper application in Rust that renders an animated galaxy with orbital mechanics, supports real-time mouse interaction (panning and pulse effects), and is optimized for OLED displays to prevent burn-in. Deploy as a Flatpak/AppImage on Linux. The application prioritizes visual smoothness (≥30 FPS), responsive input handling (<16ms), low memory footprint (<200 MB), and user-configurable animation parameters.

## Technical Context

**Language/Version**: Rust 1.75+ (User requirement: "i want to build a rust program")

**Primary Dependencies**: Graphics rendering library (wgpu, bevy, or custom OpenGL), window management (winit), and input handling. Flatpak/AppImage distribution tools. TDD framework (cargo test with test-first approach per user guidance).

**Storage**: Configuration file-based (JSON/TOML for animation parameters); no persistent database required.

**Testing**: `cargo test` with unit and integration tests; TDD approach (tests written before implementation per user guidance "ensure part of our plan includes we want to ride tdd while we can").

**Target Platform**: Linux desktop with X11 or Wayland display server. Primary use case: QD OLED or OLED panels, but compatible with standard displays.

**Project Type**: Desktop application (cross-platform compiled binary, but Linux-focused for v1).

**Performance Goals**: ≥30 FPS animation rendering; <16ms input response time; 8+ hours OLED burn-in prevention; <200 MB memory footprint.

**Constraints**: Single monitor support v1; must integrate with desktop wallpaper system; input must not interfere with other desktop applications.

**Scale/Scope**: Single-window full-screen wallpaper; render ~3-10 celestial bodies; support mouse input at desktop resolution up to 4K.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Constitution Status**: Template-only (placeholders not yet filled). No active governance constraints apply at this stage.

**Assessment**: No violations detected. Feature is compatible with standard software engineering practices:
- ✅ Rust ecosystem well-suited for desktop applications
- ✅ TDD approach explicitly requested by user ("ride tdd while we can")
- ✅ Clear testing strategy via `cargo test`
- ✅ Single responsibility (wallpaper only, no extraneous features)
- ✅ Linux-native deployment via established packaging tools

**Gate Result**: PASS - Proceed to Phase 0 research

## Project Structure

### Documentation (this feature)

```text
specs/001-interactive-oled-wallpaper/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Application entry point, wallpaper window setup
├── renderer/            # Graphics rendering (orbital animation, pulse effects)
│   ├── mod.rs
│   ├── scene.rs         # Galaxy scene management, celestial body rendering
│   ├── camera.rs        # Viewport/panning logic
│   └── effects.rs       # Pulse effect rendering
├── physics/             # Orbital mechanics simulation
│   ├── mod.rs
│   ├── orbit.rs         # Orbital path calculations
│   └── body.rs          # Celestial body state and motion
├── input/               # Mouse input handling
│   ├── mod.rs
│   └── mouse.rs         # Pan and pulse event processing
├── config/              # Configuration management
│   ├── mod.rs
│   └── animation.rs     # Animation parameter loading/validation
└── wallpaper/           # Desktop wallpaper integration
    ├── mod.rs
    └── integration.rs   # X11/Wayland wallpaper setup

tests/
├── unit/                # Unit tests for each module
│   ├── physics_tests.rs
│   ├── renderer_tests.rs
│   ├── input_tests.rs
│   └── config_tests.rs
├── integration/         # Integration tests (scene rendering, full pipeline)
│   └── wallpaper_integration.rs
└── contract/            # Contract validation tests
    └── desktop_integration.rs
```

**Structure Decision**: Single-project monolithic structure with modular organization. All code in `src/` with clear separation by domain (rendering, physics, input, config). Tests co-located by type (unit/ integration/ contract/) in tests/ directory. This structure supports TDD development where tests are written before implementation.

## Complexity Tracking

No complexity violations to track. Technical decisions are straightforward and justified by feature requirements.

---

## Phase 0: Research (COMPLETED)

**Output**: `research.md` ✅

Key decisions finalized:
- Graphics: wgpu + winit for cross-platform rendering
- Physics: Kepler orbital mechanics (O(n) complexity)
- Desktop Integration: X11 EWMH + Wayland layer-shell
- Input: winit event loop with <16ms target latency
- Config: TOML files in `~/.config/oled-wallpaper/`
- Performance: 30+ FPS minimum (60 FPS target)
- Testing: Test-First Development (TDD) per user guidance
- Deployment: Flatpak primary distribution

**Status**: Ready for Phase 1

---

## Phase 1: Design & Contracts (COMPLETED)

### Data Model

**Output**: `data-model.md` ✅

Entities defined with full contract specifications:
- **CelestialBody**: Sun and planets with position, velocity, visual properties
- **Orbit**: Kepler orbit equations for each planet
- **Camera**: Viewport with panning support
- **PulseEffect**: Temporary animations from right-click
- **AnimationConfig**: User-configurable parameters (speed, colors, sizes, patterns)
- **Scene**: Container coordinating all entities

All entities include:
- ✅ Attribute definitions with types and ranges
- ✅ Key methods and behaviors
- ✅ Validation rules
- ✅ State transitions
- ✅ Burn-in prevention mechanism
- ✅ Relationships and references

### Interface Contracts

**Output**: `contracts/desktop-integration.md` ✅

Desktop integration contract specifies:
- ✅ X11 EWMH window properties for desktop wallpaper
- ✅ Wayland layer-shell protocol support
- ✅ Input event handling (left-click pan, right-click pulse)
- ✅ Display configuration responses (resolution change, monitor connect/disconnect)
- ✅ Configuration file format (TOML) and validation rules
- ✅ Performance SLAs (30 FPS, <16ms latency, <200 MB memory)
- ✅ Error handling contracts
- ✅ Testing contract (acceptance scenarios mapped to test cases)

### Quickstart Validation

**Output**: `quickstart.md` ✅

Runnable validation scenarios demonstrating each requirement:
- ✅ Scenario 1: Launch and verify animation (FR-001, SC-001)
- ✅ Scenario 2: Pan with left-click (FR-004, SC-005)
- ✅ Scenario 3: Pulse with right-click (FR-005, SC-004)
- ✅ Scenario 4: Config customization (FR-010)
- ✅ Scenario 5: Desktop integration (FR-009, SC-002)
- ✅ Scenario 6: Flatpak deployment (FR-006, SC-003)
- ✅ Performance validation (SC-001, SC-005, SC-007)
- ✅ Troubleshooting guide

**Status**: Ready for Phase 2 (task generation with `/speckit-tasks`)

---

## TDD Emphasis

Per user guidance ("ensure part of our plan includes we want to ride tdd while we can"):

**Test-First Workflow Applied Throughout**:
- Quickstart scenarios are written as **acceptance test cases** that developers will implement against
- Data model includes **validation rules** that translate directly to unit tests
- Desktop integration contract includes **test cases** for each interaction
- Each feature requirement (FR-) has corresponding test scenarios
- Testing pyramid designed: unit → integration → contract tests

**Implementation will follow TDD strict discipline**:
1. Write test case (RED phase)
2. Verify test fails
3. Implement minimal code (GREEN phase)
4. Refactor for quality (REFACTOR phase)
5. All tests pass before feature considered complete

**Tests will be co-located** in `tests/` directory by test type (unit/, integration/, contract/) as specified in Project Structure.

---

## Summary of Deliverables

| Artifact | Path | Status | Purpose |
|----------|------|--------|---------|
| plan.md | specs/001-interactive-oled-wallpaper/plan.md | ✅ COMPLETE | This file - planning overview |
| research.md | specs/001-interactive-oled-wallpaper/research.md | ✅ COMPLETE | Technical decisions and rationale |
| data-model.md | specs/001-interactive-oled-wallpaper/data-model.md | ✅ COMPLETE | Entity definitions and contracts |
| contracts/desktop-integration.md | specs/001-interactive-oled-wallpaper/contracts/desktop-integration.md | ✅ COMPLETE | OS-level integration contract |
| quickstart.md | specs/001-interactive-oled-wallpaper/quickstart.md | ✅ COMPLETE | Runnable validation scenarios |

**Next Phase**: Run `/speckit-tasks` to generate `tasks.md` with implementation tasks ready for development.

