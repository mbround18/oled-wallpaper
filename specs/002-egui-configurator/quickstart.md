# Quickstart: Egui Configurator

## Prerequisites
- Linux desktop
- Rust 1.75+ (for dev)
- Optional: Flatpak for packaging/testing

## Build
cd /home/mbruno/development/oled-wallpaper
cargo build --release --bin oled-config

## Run
From terminal (dev):

```
# Launch configurator GUI
cargo run --bin oled-config --release

# Headless apply a preset (CI/E2E)
target/release/oled-config --headless --apply "Night Sky" --config-dir /tmp/test-config
```

## E2E Validation (CI-friendly)
1. Prepare temp config dir and a known preset TOML
2. Run wallpaper in demo: `target/release/oled-wallpaper --demo 10 --config-dir /tmp/test-config`
3. Run configurator headless to write config to /tmp/test-config
4. Wallpaper reads config and logs applied settings
5. CI asserts wallpaper logs contain expected settings or captures a screenshot for pixel checks

## Troubleshooting
- If Flatpak sandbox prevents writing to config, run with `--filesystem=home` during development or adjust manifest to allow XDG config access.

