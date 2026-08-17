#!/usr/bin/env bash
set -euo pipefail
TMPDIR=$(mktemp -d)
BG_PID=""
cleanup() {
  if [ -n "$BG_PID" ] && kill -0 "$BG_PID" 2>/dev/null; then
    kill "$BG_PID" 2>/dev/null || true
    wait "$BG_PID" 2>/dev/null || true
  fi
  rm -rf "$TMPDIR"
}
trap cleanup EXIT

# Build once up front, before HOME is redirected (rustup/cargo need the
# real HOME to resolve the toolchain). cargo run would also background the
# `cargo` wrapper process rather than the real binary, so $! wouldn't match
# the lock file's PID — invoke the built binary directly below instead.
cargo build --quiet
BIN="$(pwd)/target/debug/oled-wallpaper"

export HOME="$TMPDIR"

LOCK_FILE="$HOME/.config/oled-wallpaper/wallpaper.lock"
FIRST_LOG="$TMPDIR/first.log"
SECOND_LOG="$TMPDIR/second.log"

# Launch first instance in the background for a bounded demo run.
"$BIN" --demo 6 >"$FIRST_LOG" 2>&1 &
BG_PID=$!

# Poll for the lock file to appear (first run may take a moment to build/start).
waited=0
while [ ! -f "$LOCK_FILE" ]; do
  if ! kill -0 "$BG_PID" 2>/dev/null; then
    echo "First instance exited before creating lock file"
    cat "$FIRST_LOG" || true
    exit 2
  fi
  sleep 0.3
  waited=$((waited + 1))
  if [ "$waited" -ge 100 ]; then
    echo "Timed out waiting for lock file at $LOCK_FILE"
    exit 3
  fi
done

LOCK_PID=$(cat "$LOCK_FILE" | tr -d '[:space:]')
if [ "$LOCK_PID" != "$BG_PID" ]; then
  echo "Lock file PID ($LOCK_PID) does not match backgrounded process PID ($BG_PID)"
  exit 4
fi

# Attempt a second instance while the first is still running. The binary
# refuses to start a second instance (single-instance lock), printing a
# message that mentions the PID and returning promptly. Note: main() returns
# normally in this path rather than calling std::process::exit, so the
# process exit code is 0 — we assert on the "already running" message and
# on the fact that it returns quickly instead.
if ! timeout 5s "$BIN" --demo 1 >"$SECOND_LOG" 2>&1; then
  # timeout itself failing (non-zero from `timeout`) would only happen if
  # the second instance ran past its timeout, which should not occur.
  echo "Second instance did not return promptly (possibly opened a real window)"
  cat "$SECOND_LOG" || true
  exit 5
fi

if ! grep -qi "already running" "$SECOND_LOG"; then
  echo "Second instance did not report the wallpaper as already running"
  cat "$SECOND_LOG" || true
  exit 6
fi

if ! grep -q "$BG_PID" "$SECOND_LOG"; then
  echo "Second instance's message did not mention the first instance's PID ($BG_PID)"
  cat "$SECOND_LOG" || true
  exit 7
fi

if [ ! -f "$LOCK_FILE" ]; then
  echo "Lock file disappeared while first instance should still be running"
  exit 8
fi

# Wait for the first (backgrounded) instance to finish its bounded demo run.
wait "$BG_PID"
BG_PID=""

if [ -f "$LOCK_FILE" ]; then
  echo "Lock file was not cleaned up after first instance exited"
  exit 9
fi

echo "E2E wallpaper lock/restart OK"
