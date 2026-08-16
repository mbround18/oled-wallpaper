#!/usr/bin/env bash
set -euo pipefail
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT
# Write config with show_clock=false via headless
cargo run --bin oled-config --quiet -- --headless --apply "widget-off" --config-dir "$TMPDIR/config.toml"
# Confirm config exists and show_clock=false
if ! grep -q "show_clock.*false" "$TMPDIR/config.toml"; then
  echo "Widget toggle failed"
  exit 2
fi
echo "E2E US2 widget toggle OK"
