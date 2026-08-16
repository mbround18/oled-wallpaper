# Contract: Configuration & Integration (Configurator ↔ Wallpaper)

## Purpose
Define the canonical configuration format, file location, and integration expectations so the egui configurator and the wallpaper interoperate reliably across Flatpak and non-Flatpak installs.

## Config Location
Primary: `$XDG_CONFIG_HOME/oled-wallpaper/config.toml` (default fallback: `~/.config/oled-wallpaper/config.toml`)

## File Format
TOML document. Schema versioned via top-level `version` integer. Backwards-compatible additive changes only.

## Write Semantics
- Configurator MUST write atomically: write to temporary file then rename to final path to avoid partial reads by wallpaper.
- When writing from Flatpak, ensure manifest grants write access to the config path or instruct user to export config via import/export UI.

## Read Semantics (Wallpaper)
- Wallpaper MUST attempt to read config at startup and optionally watch for file changes.
- On parse error: wallpaper MUST log descriptive error and continue with default config.
- On missing fields: wallpaper MUST use defaults for missing keys and log a warning.

## Live-Reload Behavior
- Wallpaper MAY implement inotify/file-notify watch to pick up changes; not required for v1. If implemented, reload must be debounced (e.g., 250ms) to avoid thrashing.

## Preset Import/Export
- Export: Configurator writes a valid TOML file with `name` and `config` keys; UI offers file save dialog.
- Import: Configurator validates incoming TOML and rejects invalid schemas.

## CLI Contract (Headless Mode)
Provide a small CLI for automation:
- `oled-config --headless --apply <preset-name> --config-dir <dir>`: writes config to specified dir and exits with 0 on success
- `oled-config --export-preset <name> --out /path/to/file.toml`

## Flatpak Notes
- Flatpak manifest must allow access to `$XDG_CONFIG_HOME` (or `home` filesystem) or expose an export/import workflow.
- Recommend documenting required permissions in README and quickstart.

## Test Contracts
- Unit tests must cover load/save/validate paths
- Integration tests must use temp config directories (not user home) to avoid clobbering user data
- E2E tests must be able to run with `--config-dir` override to point wallpaper and configurator at a shared temp directory

