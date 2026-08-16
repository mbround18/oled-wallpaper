# OLED Wallpaper v0.1.0 (Draft)

## Features
- Fullscreen OLED-first animated galaxy with pure-black background.
- Binary-star orbital system, rotating orbital plane, drifting starfield, Oort elements, meteors, cosmic rays, and pulse interactions.
- Mouse interactions: left-drag pan, right-click pulse.
- Optional demo mode (`--demo`) with auto-exit timer and bottom-right performance HUD.
- Linux display-server aware startup path (Wayland/X11 handle routing) with X11 desktop hint pinning.
- Flatpak packaging artifacts (`ninja.boop.OledWallpaper.yml`, desktop file, metainfo).

## Performance Notes
- Release build validated with `cargo build --release`.
- In-app HUD can report FPS, CPU, RAM, and DRM fdinfo GPU/VRAM counters when available.
- Visual motion is intentionally low-intensity and continuously drifting to reduce static-pixel persistence.

## Known Limitations
- Wayland desktop-wallpaper pinning differs across compositors; behavior may vary.
- Some final release-gate validations (8-hour soak, multi-resolution hardware pass, packaged smoke test) are still pending.
- GitHub release/tag publication is not automated in-repo and remains a manual step.
