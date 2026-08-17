#!/usr/bin/env bash
# Regression test for the single-instance lock and the cooperative restart
# signal (src/runtime.rs). The lock used to store a PID and check
# /proc/{pid} to decide if an old instance was still alive — meaningless
# under Flatpak, where every `flatpak run` gets its own private PID
# namespace and the launched process almost always lands on PID 2. A
# wallpaper started via the autostart entry's `flatpak run` would write "2"
# to the lock file; the Configurator (a *different* sandbox instance, or
# native) would check /proc/2 in ITS OWN namespace, where PID 2 is some
# unrelated, always-alive process (kthreadd on a bare host) — so the check
# always reported "still running," permanently, even after the real
# process was long dead. That was the "auto locked, never starts again"
# bug. The fix: advisory `flock` (namespace-independent, released by the
# kernel on fd close regardless of how the process exits) plus a
# cooperative restart-request file instead of `kill -TERM <pid>` (which
# can't cross the PID namespace boundary either).
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
RESTART_SIGNAL="$HOME/.config/oled-wallpaper/restart.signal"
FIRST_LOG="$TMPDIR/first.log"
SECOND_LOG="$TMPDIR/second.log"
THIRD_LOG="$TMPDIR/third.log"

# The lock is now advisory (flock), not a PID in the file — check actual
# lock state with the same primitive the app uses, not file presence.
is_locked() {
  ! flock -n -x "$LOCK_FILE" -c 'true' 2>/dev/null
}

# Launch first instance in the background for a bounded demo run (long
# enough that it won't finish naturally before we exercise the restart
# signal below).
"$BIN" --demo 20 >"$FIRST_LOG" 2>&1 &
BG_PID=$!

# Poll for the lock file to appear (first run may take a moment to start).
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

# Give it a moment past file-creation to actually take the flock.
waited=0
while ! is_locked; do
  sleep 0.1
  waited=$((waited + 1))
  if [ "$waited" -ge 50 ]; then
    echo "Lock file exists but was never actually flocked"
    exit 4
  fi
done

LOCK_PID=$(cat "$LOCK_FILE" | tr -d '[:space:]')
if [ "$LOCK_PID" != "$BG_PID" ]; then
  echo "Lock file PID ($LOCK_PID) does not match backgrounded process PID ($BG_PID)"
  exit 5
fi

# Attempt a second instance while the first is still running. The binary
# refuses to start a second instance (single-instance lock), printing a
# message that mentions the PID and returning promptly. Note: main() returns
# normally in this path rather than calling std::process::exit, so the
# process exit code is 0 — we assert on the "already running" message and
# on the fact that it returns quickly instead.
if ! timeout 5s "$BIN" --demo 1 >"$SECOND_LOG" 2>&1; then
  echo "Second instance did not return promptly (possibly opened a real window)"
  cat "$SECOND_LOG" || true
  exit 6
fi

if ! grep -qi "already running" "$SECOND_LOG"; then
  echo "Second instance did not report the wallpaper as already running"
  cat "$SECOND_LOG" || true
  exit 7
fi

if ! grep -q "$BG_PID" "$SECOND_LOG"; then
  echo "Second instance's message did not mention the first instance's PID ($BG_PID)"
  cat "$SECOND_LOG" || true
  exit 8
fi

if ! is_locked; then
  echo "Lock was released while first instance should still be running"
  exit 9
fi

# Exercise the actual restart mechanism: write the same cooperative signal
# file runtime::request_restart() writes, and confirm the running instance
# notices it (polled once per frame) and exits well before its 20s --demo
# timer would have — proving the fix, not just a natural timeout exit.
: > "$RESTART_SIGNAL"

waited=0
while kill -0 "$BG_PID" 2>/dev/null; do
  sleep 0.2
  waited=$((waited + 1))
  if [ "$waited" -ge 50 ]; then
    echo "First instance did not exit within 10s of the restart signal"
    exit 10
  fi
done
wait "$BG_PID" 2>/dev/null || true
BG_PID=""

if is_locked; then
  echo "Lock was not released after the signaled instance exited"
  exit 11
fi

if [ -f "$RESTART_SIGNAL" ]; then
  echo "Restart signal file was not consumed by the exiting instance"
  exit 12
fi

# Prove the fix end-to-end: a brand new instance must be able to start
# immediately — this is exactly the scenario that used to be "auto locked,
# never starts again" under Flatpak.
if ! timeout 5s "$BIN" --demo 2 >"$THIRD_LOG" 2>&1; then
  echo "Third instance failed to run after the lock was released"
  cat "$THIRD_LOG" || true
  exit 13
fi
if grep -qi "already running" "$THIRD_LOG"; then
  echo "Third instance wrongly refused to start — lock was not really released"
  cat "$THIRD_LOG" || true
  exit 14
fi

echo "E2E wallpaper lock/restart OK"
