# Flatpak integration notes

To ensure the wallpaper autostarts when installed as a Flatpak:

- Include packaging/autostart/oled-wallpaper.desktop in the Flatpak bundle (copy to /app/share/applications/).
- Add a post-install step in the Flatpak manifest which installs the .desktop into the host autostart area or uses the xdg-autorun helper.
- Flatpak must allow access to the user config dir so installed autostart entries are visible to the host. In the build manifest add finish-args such as:
  "finish-args": [
    "--filesystem=xdg-config",
    "--share=network"
  ]

For systemd-based distributions, installers can enable the user unit after installing the binary:

  systemctl --user enable --now oled-wallpaper.service

Adjust ExecStart path to point to the installed binary location for each packaging target (deb/rpm/flatpak).
