# Contributing

Thanks for looking at the code! This covers dev setup, testing, packaging, and how releases are cut.

## Project layout

- `src/` — the wallpaper binary (`oled-wallpaper`) and the GUI configurator binary (`oled-config`), sharing the `oled_wallpaper` library crate.
- `tests/` — unit, integration, contract, and shell-based e2e tests.
- `packaging/` — Flatpak manifest/metadata, AppImage build script, systemd user unit, XDG autostart entry.
- `scripts/` — small standalone tooling (currently just `version-sync`, see below).
- `docs/` — user-facing docs referenced from the README.

## Dev setup

You need a stable Rust toolchain ([rustup.rs](https://rustup.rs)) and, optionally, `flatpak` + `flatpak-builder` if you're touching packaging.

```bash
cargo build                      # debug build
cargo run --bin oled-wallpaper -- --demo   # quick smoke test, auto-closes
cargo run --bin oled-config      # launch the configurator GUI
```

## Testing

```bash
cargo test --all                 # unit + integration + contract tests
cargo fmt --all -- --check       # formatting (rustfmt.toml)
cargo clippy --all --all-targets -- -D warnings   # lint, warnings are errors
```

Shell-based e2e tests live in `tests/e2e/` and run headless (they need `xvfb-run` on a machine without a display, e.g. CI):

```bash
chmod +x tests/e2e/*.sh
xvfb-run -a ./tests/e2e/e2e_us1_apply.sh
xvfb-run -a ./tests/e2e/e2e_us2_widget.sh
xvfb-run -a ./tests/e2e/e2e_wallpaper_demo_widget.sh
./tests/e2e/e2e_version_sync.sh   # no display needed
```

All of the above run in CI on every push/PR — see `.github/workflows/ci.yml` and `.github/workflows/e2e.yml`.

## Building packages locally

```bash
make build            # Flatpak if flatpak-builder is present, else a local cargo release build
make build-bundle      # produces dist/ninja.boop.OledWallpaper.flatpak
./packaging/appimage/build-appimage.sh   # AppImage (needs network to fetch appimagetool once)
```

## Versioning

`Cargo.toml`'s `[package].version` is the **single source of truth** for the project version. Everything else that embeds a version number (currently just the Flatpak `metainfo.xml` release entry) is kept in sync by `scripts/version-sync`:

```bash
./scripts/version-sync           # check mode — fails if anything has drifted
./scripts/version-sync --fix     # rewrites out-of-sync files to match Cargo.toml
```

When bumping the version for a release: edit `Cargo.toml`, run `scripts/version-sync --fix`, add a new `<release>` entry to `packaging/flatpak/ninja.boop.OledWallpaper.metainfo.xml` with your changelog, commit, then tag.

The script is tested by `tests/e2e/e2e_version_sync.sh`, which runs it against a throwaway fixture tree (never the real repo files) to check both the "in sync" and "drifted" paths, `--fix`, and `--tag` verification.

## Cutting a release

Releases are fully automated by `.github/workflows/release.yml`, triggered on pushing a tag matching `v*.*.*`:

1. Bump `version` in `Cargo.toml`.
2. Run `./scripts/version-sync --fix` and update the metainfo changelog entry.
3. Commit the version bump.
4. Tag and push: `git tag v1.2.3 && git push origin v1.2.3`.

The workflow then:
- Verifies the tag matches `Cargo.toml` (and that `metainfo.xml` is in sync) via `scripts/version-sync --tag`.
- Runs the full test suite.
- Builds a release binary tarball and a Flatpak bundle.
- Publishes a GitHub Release with both artifacts attached and auto-generated release notes.

If the version check fails, nothing else runs — fix the drift and re-push the tag.
