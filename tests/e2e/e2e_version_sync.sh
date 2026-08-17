#!/usr/bin/env bash
# Exercises scripts/version-sync against a throwaway fixture tree so it
# never touches the real repo's Cargo.toml / metainfo.xml.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/version-sync"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

mkdir -p "$TMPDIR/packaging/flatpak"

write_fixture() {
  local cargo_version="$1" metainfo_version="$2"
  cat > "$TMPDIR/Cargo.toml" <<EOF
[package]
name = "oled-wallpaper"
version = "$cargo_version"
edition = "2021"
EOF
  cat > "$TMPDIR/packaging/flatpak/ninja.boop.OledWallpaper.metainfo.xml" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <releases>
    <release version="$metainfo_version" date="2026-01-01">
      <description><p>fixture</p></description>
    </release>
  </releases>
</component>
EOF
}

# 1. In sync -> check succeeds
write_fixture "1.2.3" "1.2.3"
if ! "$SCRIPT" --root "$TMPDIR" >/dev/null; then
  echo "FAIL: expected check to pass when versions match"
  exit 1
fi
echo "PASS: in-sync versions -> check exits 0"

# 2. Drifted -> check fails
write_fixture "1.2.3" "1.0.0"
if "$SCRIPT" --root "$TMPDIR" >/dev/null 2>&1; then
  echo "FAIL: expected check to fail on drift"
  exit 1
fi
echo "PASS: drifted metainfo -> check exits non-zero"

# 3. --fix corrects the drift, and a follow-up check passes
"$SCRIPT" --root "$TMPDIR" --fix >/dev/null
if ! "$SCRIPT" --root "$TMPDIR" >/dev/null; then
  echo "FAIL: expected check to pass after --fix"
  exit 1
fi
if ! grep -q 'version="1.2.3"' "$TMPDIR/packaging/flatpak/ninja.boop.OledWallpaper.metainfo.xml"; then
  echo "FAIL: --fix did not rewrite metainfo.xml version"
  exit 1
fi
echo "PASS: --fix rewrites metainfo.xml to match Cargo.toml"

# 4. --tag matching Cargo.toml passes
write_fixture "2.0.0" "2.0.0"
if ! "$SCRIPT" --root "$TMPDIR" --tag v2.0.0 >/dev/null; then
  echo "FAIL: expected matching --tag to pass"
  exit 1
fi
echo "PASS: matching tag -> check exits 0"

# 5. --tag mismatching Cargo.toml fails
if "$SCRIPT" --root "$TMPDIR" --tag v9.9.9 >/dev/null 2>&1; then
  echo "FAIL: expected mismatching --tag to fail"
  exit 1
fi
echo "PASS: mismatching tag -> check exits non-zero"

echo "E2E version-sync OK"
