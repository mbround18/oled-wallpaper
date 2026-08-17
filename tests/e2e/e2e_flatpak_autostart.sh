#!/usr/bin/env bash
# Regression test for the silent autostart failure: the Flatpak sandbox
# previously had no --filesystem grant for ~/.config/autostart, so
# `set_autostart_enabled(true)` returned Ok(()) and the app showed
# "Autostart enabled", but the .desktop file was written into an ephemeral
# per-process overlay and never reached the real host path. Unit tests in
# tests/configurator_tests.rs can't catch this because they run natively
# (HOME pointed at a tempdir, no sandbox) — this script is the only test
# that runs the actual built Flatpak inside its sandbox and checks the real
# host filesystem side effect.
set -euo pipefail

APP_ID="ninja.boop.OledWallpaper"
MANIFEST="packaging/flatpak/ninja.boop.OledWallpaper.yml"
AUTOSTART_FILE="$HOME/.config/autostart/${APP_ID}.desktop"

if ! command -v flatpak-builder >/dev/null 2>&1; then
  echo "SKIP: flatpak-builder not installed"
  exit 0
fi

# Save/restore any pre-existing autostart file so this script is safe to run
# on a developer machine that already has the app's autostart entry set.
BACKUP=""
if [ -f "$AUTOSTART_FILE" ]; then
  BACKUP=$(mktemp)
  cp "$AUTOSTART_FILE" "$BACKUP"
fi
cleanup() {
  rm -f "$AUTOSTART_FILE"
  if [ -n "$BACKUP" ]; then
    mkdir -p "$(dirname "$AUTOSTART_FILE")"
    cp "$BACKUP" "$AUTOSTART_FILE"
    rm -f "$BACKUP"
  fi
}
trap cleanup EXIT

echo "-> Building Flatpak (offline, via cargo-sources.json)"
flatpak-builder --user --install --force-clean build-dir-e2e "$MANIFEST"

echo "-> Confirming the manifest actually grants xdg-config/autostart"
PERMS=$(flatpak info --user --show-permissions "$APP_ID" | grep '^filesystems=')
if ! echo "$PERMS" | grep -q 'xdg-config/autostart'; then
  echo "FAIL: manifest does not grant xdg-config/autostart — regression reintroduced"
  echo "  permissions: $PERMS"
  exit 1
fi
echo "PASS: sandbox has xdg-config/autostart permission"

rm -f "$AUTOSTART_FILE"

echo "-> Enabling autostart from inside the sandbox (same code path the GUI checkbox uses)"
flatpak run --command=oled-config "$APP_ID" --headless --set-autostart on

echo "-> Checking the REAL host autostart path (not the sandbox's private overlay)"
if [ ! -f "$AUTOSTART_FILE" ]; then
  echo "FAIL: $AUTOSTART_FILE was not written to the real host filesystem"
  exit 1
fi
if ! grep -q "^Exec=" "$AUTOSTART_FILE"; then
  echo "FAIL: autostart file exists but has no Exec= line"
  exit 1
fi
if ! grep -q "^Exec=flatpak run $APP_ID" "$AUTOSTART_FILE"; then
  echo "FAIL: Exec= line does not launch via flatpak"
  cat "$AUTOSTART_FILE"
  exit 1
fi
echo "PASS: autostart .desktop file written to real host path with correct Exec="

echo "-> Disabling autostart from inside the sandbox"
flatpak run --command=oled-config "$APP_ID" --headless --set-autostart off

if [ -f "$AUTOSTART_FILE" ]; then
  echo "FAIL: $AUTOSTART_FILE still exists after disabling"
  exit 1
fi
echo "PASS: autostart file removed from real host path"

echo "E2E flatpak autostart OK"
