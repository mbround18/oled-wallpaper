#!/bin/bash
set -e
echo "Building OLED Wallpaper Flatpak..."
# Install required runtime if not present
flatpak install --user --noninteractive flathub org.freedesktop.Platform//23.08 \
    org.freedesktop.Sdk//23.08 \
    org.freedesktop.Sdk.Extension.rust-stable//23.08 2>/dev/null || true
# Build
flatpak-builder --user --install --force-clean build-dir ninja.boop.OledWallpaper.yml
echo "✅ Built and installed: ninja.boop.OledWallpaper"
echo "Run with: flatpak run ninja.boop.OledWallpaper"
echo "Demo mode: flatpak run ninja.boop.OledWallpaper --demo 30"
