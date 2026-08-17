#!/usr/bin/env bash
set -euo pipefail
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

PRESET_PATH="$TMPDIR/roundtrip.toml"
CONFIG_PATH="$TMPDIR/config.toml"

# Export a preset with a custom name via headless mode
cargo run --bin oled-config --quiet -- --headless --export-preset "$PRESET_PATH" --preset-name "roundtrip-test"

# Confirm the preset file was created and contains the custom name and the
# distinguishable non-default planet_speed value.
if [ ! -f "$PRESET_PATH" ]; then
  echo "Preset export failed: file not created"
  exit 2
fi
if ! grep -q 'name = "roundtrip-test"' "$PRESET_PATH"; then
  echo "Preset export failed: missing preset name"
  exit 2
fi
if ! grep -q "planet_speed.*2.0" "$PRESET_PATH"; then
  echo "Preset export failed: missing planet_speed = 2.0"
  exit 2
fi

# Import the preset into a fresh config dir and confirm the applied config
# reflects the preset's values (not just Config::default(), whose
# planet_speed is 1.0).
cargo run --bin oled-config --quiet -- --headless --import-preset "$PRESET_PATH" --config-dir "$CONFIG_PATH"

if [ ! -f "$CONFIG_PATH" ]; then
  echo "Preset import failed: config not written"
  exit 2
fi
if ! grep -q "planet_speed.*2.0" "$CONFIG_PATH"; then
  echo "Preset import failed: planet_speed not applied from preset"
  exit 2
fi

# Importing a nonexistent preset file must fail (non-zero exit).
if cargo run --bin oled-config --quiet -- --headless --import-preset "$TMPDIR/does-not-exist.toml" --config-dir "$TMPDIR/other-config.toml" 2>/dev/null; then
  echo "FAIL: expected error importing nonexistent preset"
  exit 2
fi

echo "E2E preset roundtrip OK"
