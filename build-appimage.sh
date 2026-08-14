#!/bin/bash
# AppImage build script for OLED Wallpaper

set -e

echo "Building OLED Wallpaper AppImage..."

# Ensure release binary is built
cargo build --release

# Create AppDir structure
mkdir -p AppDir/usr/bin
mkdir -p AppDir/usr/share/applications
mkdir -p AppDir/usr/share/pixmaps

# Copy binary
cp target/release/oled-wallpaper AppDir/usr/bin/

# Create desktop file
cat > AppDir/usr/share/applications/oled-wallpaper.desktop << 'DESKTOP'
[Desktop Entry]
Type=Application
Name=OLED Wallpaper
Comment=Interactive animated wallpaper for OLED displays
Exec=oled-wallpaper
Icon=oled-wallpaper
Categories=Graphics;
StartupNotify=true
DESKTOP

# Download AppImage tools if not present
if [ ! -f appimagetool-x86_64.AppImage ]; then
    echo "Downloading AppImage tools..."
    wget -q https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x appimagetool-x86_64.AppImage
fi

# Build AppImage
export ARCH=x86_64
./appimagetool-x86_64.AppImage AppDir oled-wallpaper.AppImage

echo "✅ AppImage built: oled-wallpaper.AppImage"
