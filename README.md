# OLED Wallpaper

A living animated galaxy, rendered straight to your Linux desktop background — built specifically for OLED and AMOLED screens.

True-black space between the stars means those pixels are actually **off**, so you get a beautiful animated wallpaper without the burn-in risk of a static image or a bright looping video.

- 🌌 Binary star system with real gravitational dynamics and Kepler orbits
- ☄️ Oort cloud debris, meteor showers, and drifting comets
- 🖱️ Left-drag to pan the view, right-click to send a ripple through the star field
- 🕶️ Slow scene drift + pixel-shifting to actively guard against burn-in
- 🕐 Optional clock/weather widget overlay
- 🖥️ Works on both X11 and Wayland

## Install

### Flatpak (recommended)

```bash
flatpak install flathub org.freedesktop.Platform//25.08   # one-time runtime setup
make build          # builds a Flatpak if flatpak-builder is installed
```

Or grab a pre-built `.flatpak` bundle from the [Releases](../../releases) page and install it directly:

```bash
flatpak install --user oled-wallpaper-*.flatpak
```

### Build from source (cargo)

Requires a recent stable [Rust toolchain](https://rustup.rs).

```bash
cargo build --release
cargo install --path . --force   # installs to ~/.cargo/bin
```

## Running it

Start the wallpaper:

```bash
oled-wallpaper
```

Try it out for 10 seconds without committing to anything:

```bash
oled-wallpaper --demo        # auto-closes after 10s
oled-wallpaper --demo 30     # or pick your own duration
```

### Run it automatically at login

Pick whichever fits your setup:

```bash
make enable-autostart   # XDG autostart .desktop entry
make enable-systemd     # systemd --user service
```

### Configuring

A small GUI configurator lets you tweak animation speed, widgets, and presets without editing files by hand:

```bash
oled-config
```

See [`docs/quickstart-configurator.md`](docs/quickstart-configurator.md) for details, including headless/scripted usage.

## Uninstalling

```bash
make uninstall-flatpak     # if installed via Flatpak
cargo uninstall oled-wallpaper   # if installed via cargo
```

## License

Dual-licensed under MIT or Apache-2.0, at your option.

## Contributing

Want to help build this? See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the dev setup, test suite, and release process.
