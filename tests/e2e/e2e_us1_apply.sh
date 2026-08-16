#!/usr/bin/env bash
set -euo pipefail
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT
# Run headless configurator to write config
cargo run --bin oled-config --quiet -- --headless --apply "test" --config-dir "$TMPDIR/config.toml"
# Confirm config exists and contains planet_speed = 2.0
if ! grep -q "planet_speed.*2.0" "$TMPDIR/config.toml"; then
  echo "Config apply failed"
  exit 2
fi
echo "E2E US1 apply OK"
