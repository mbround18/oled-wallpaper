# Flatpak integration notes

To ensure the wallpaper autostarts when installed as a Flatpak:

- Include packaging/autostart/ninja.boop.OledWallpaper.desktop in the Flatpak bundle (copy to /app/share/applications/).
- Add a post-install step in the Flatpak manifest which installs the .desktop into the host autostart area or uses the xdg-autorun helper.
- Flatpak must allow access to the user config dir so installed autostart entries are visible to the host. In the build manifest add finish-args such as:
  "finish-args": [
    "--filesystem=xdg-config",
    "--share=network"
  ]

Recommended installer steps (packagers should adapt to distro conventions):

- Desktop autostart: copy `packaging/autostart/oled-wallpaper.desktop` into `/usr/share/applications/` during packaging and, for user-scoped autostart, install into `~/.config/autostart/` on first-run or via a post-install script.

- Systemd user unit: package `packaging/systemd/user/oled-wallpaper.service` and provide a post-install action (or packaging hook) that runs:

  ```sh
  # As the target user (do not run automatically in global package installs)
  systemctl --user daemon-reload
  systemctl --user enable --now ninja.boop.OledWallpaper.service
  ```

- Flatpak specifics: include the .desktop file in the Flatpak (`/app/share/applications/`) and add `--filesystem=xdg-config` to `finish-args`. If wanting to create a host-autostart entry from Flatpak, use an installer helper on first-run with appropriate user consent.

Makefile targets

- `make build` — builds via Flatpak when `flatpak-builder` is available, otherwise does `cargo build --release`.
- `make build-bundle` — creates a flatpak bundle `ninja.boop.OledWallpaper.flatpak` and an OSTree `repo/` (if flatpak-builder present).
- `make install` — installs Flatpak when available, otherwise performs `cargo install --path . --force`.
- `make enable-autostart` — (local only) copy the autostart desktop to `~/.config/autostart` for the current user.
- `make enable-systemd` — (local only) attempt to enable the systemd user unit (requires user session and permission).

Safety notes

- The `enable-*` targets make changes to the current user's session (copy files, enable systemd units). They are intended for interactive use only and are NOT executed by CI.
- Flatpak cannot directly write into the host autostart locations without host integration; prefer a helper or packaging step outside the sandbox.

