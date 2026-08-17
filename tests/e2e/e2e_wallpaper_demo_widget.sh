#!/usr/bin/env bash
set -euo pipefail
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Build before HOME is redirected so rustup/cargo can resolve the toolchain.
cargo build --quiet

export HOME="$TMPDIR"
# Write config via headless configurator (uses HOME/.config/oled-wallpaper/config.toml)
target/debug/oled-config --headless --apply "widget-off"
# Run wallpaper in demo mode for 3s and capture logs
LOGFILE="$TMPDIR/wall.log"
RUST_LOG=info target/debug/oled-wallpaper --demo 3 2>&1 | tee "$LOGFILE"
# Assert widget absent was logged
if ! grep -q "Widget overlay absent" "$LOGFILE"; then
  echo "Widget still present in demo logs"
  exit 2
fi
# Also test widget-on path
target/debug/oled-config --headless --apply "widget-on"
RUST_LOG=info target/debug/oled-wallpaper --demo 2 2>&1 | tee "$LOGFILE"
if ! grep -q "Widget overlay present" "$LOGFILE"; then
  echo "Widget not present after enabling"
  exit 3
fi

echo "E2E wallpaper demo widget test OK"
