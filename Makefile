FLATPAK_MANIFEST := packaging/flatpak/ninja.boop.OledWallpaper.yml
APP_ID := ninja.boop.OledWallpaper
BUILD_DIR := build-dir
DIST_DIR := dist
FLATHUB_REPO := https://flathub.org/repo/flathub.flatpakrepo
RUNTIME_REF := org.freedesktop.Platform//24.08

.PHONY: all build build-flatpak build-local build-bundle flatpak install install-flatpak install-local install-bundle clean uninstall-flatpak

all: build

# Choose flatpak build when available, otherwise do a local cargo release build
build:
	@echo "=== Build: choose flatpak if available, else cargo release ==="
	@if command -v flatpak-builder >/dev/null 2>&1; then \
		echo "flatpak-builder detected -> building Flatpak..."; \
		$(MAKE) build-flatpak; \
	else \
		echo "flatpak-builder not found -> building local release binary..."; \
		$(MAKE) build-local; \
	fi

build-flatpak:
	@echo "-> Ensuring Flatpak runtimes are available (may prompt)..."
	flatpak install --user --noninteractive flathub org.freedesktop.Platform//24.08 \
		org.freedesktop.Sdk//24.08 org.freedesktop.Sdk.Extension.rust-stable//24.08 2>/dev/null || true
	@echo "-> Running flatpak-builder"
	flatpak-builder --user --install --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	@echo "-> Flatpak build+install complete"

build-local:
	@echo "-> cargo build --release"
	cargo build --release
	@echo "-> local build complete (target/release/oled-wallpaper)"

install:
	@echo "=== Install: prefer Flatpak install, fallback to cargo install ==="
	@if command -v flatpak-builder >/dev/null 2>&1; then \
		$(MAKE) install-flatpak; \
	else \
		$(MAKE) install-local; \
	fi

install-flatpak:
	@echo "-> Installing Flatpak (build+install via flatpak-builder)"
	flatpak-builder --user --install --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	@echo "-> Installed Flatpak: run with 'flatpak run $(APP_ID)'"

install-local:
	@echo "-> Installing binary locally with 'cargo install --path . --force'"
	cargo install --path . --force
	@echo "-> Installed binary to ~/.cargo/bin"

# Build a Flatpak bundle file (.flatpak) and repo export
build-bundle:
	@echo "-> Building Flatpak repo and bundle into $(DIST_DIR)"
	@mkdir -p $(DIST_DIR)/repo
	flatpak-builder --repo=$(DIST_DIR)/repo --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	flatpak build-bundle --runtime-repo=$(FLATHUB_REPO) $(DIST_DIR)/repo $(DIST_DIR)/$(APP_ID).flatpak $(APP_ID)
	@echo "-> Bundle written to $(DIST_DIR)/$(APP_ID).flatpak (or $(DIST_DIR)/repo/ if build failed)"

# Export the build repo for distribution (OSTree repo)
export-repo:
	@echo "-> Exporting flatpak repo to $(DIST_DIR)/repo"
	@mkdir -p $(DIST_DIR)/repo
	flatpak-builder --repo=$(DIST_DIR)/repo --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	@echo "-> Repo ready in $(DIST_DIR)/repo"

# Enable autostart by copying desktop file to the user's autostart directory
enable-autostart:
	@echo "-> Installing autostart .desktop to ~/.config/autostart (no system changes are made in CI)"
	@mkdir -p $(HOME)/.config/autostart
	@cp packaging/autostart/ninja.boop.OledWallpaper.desktop $(HOME)/.config/autostart/ninja.boop.OledWallpaper.desktop || true
	@echo "-> Autostart entry installed to $(HOME)/.config/autostart/ninja.boop.OledWallpaper.desktop"

# Enable systemd --user service (requires the user to run this locally)
enable-systemd:
	@echo "-> Enabling systemd --user unit (requires systemd user session)"
	systemctl --user daemon-reload || true
	systemctl --user enable --now ninja.boop.OledWallpaper.service || true
	@echo "-> Attempted to enable ninja.boop.OledWallpaper.service (check with 'systemctl --user status')"

uninstall-flatpak:
	@echo "-> Attempting to uninstall Flatpak app $(APP_ID)"
	flatpak uninstall --user -y $(APP_ID) || true

flatpak:
	@echo "=== Building Flatpak bundle (.flatpak) and installing to local Flatpak repo ==="
	@if ! command -v flatpak-builder >/dev/null 2>&1; then \
		echo "Error: flatpak-builder not found. Install flatpak-builder to use 'make flatpak'."; exit 2; \
	fi
	@echo "-> Building repo and bundle into $(DIST_DIR)"
	@mkdir -p $(DIST_DIR)/repo
	flatpak-builder --repo=$(DIST_DIR)/repo --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	flatpak build-bundle --runtime-repo=$(FLATHUB_REPO) $(DIST_DIR)/repo $(DIST_DIR)/$(APP_ID).flatpak $(APP_ID)
	@echo "-> Bundle written to $(DIST_DIR)/$(APP_ID).flatpak"
	@echo "-> Installing bundle to local flatpak (user)"
	$(MAKE) install-bundle
	@echo "-> Flatpak bundle built and installed (if supported)"

clean:
	rm -rf $(BUILD_DIR) target repo $(APP_ID).flatpak $(DIST_DIR) $(DIST_DIR)/repo $(DIST_DIR)/$(APP_ID).flatpak
	@echo "-> cleaned build artifacts (including $(DIST_DIR))"
install-bundle:
	@echo "-> Installing bundle with runtime auto-setup"
	flatpak remote-add --if-not-exists --user flathub $(FLATHUB_REPO)
	flatpak install --user --noninteractive --assumeyes flathub $(RUNTIME_REF)
	@# Purge any stale .removed entries that cause "Directory not empty" on reinstall
	@rm -rf "$(HOME)/.local/share/flatpak/.removed/$(APP_ID)"* 2>/dev/null || true
	flatpak install --user --noninteractive --reinstall --assumeyes $(DIST_DIR)/$(APP_ID).flatpak
	@echo "-> Installed $(APP_ID) from bundle"
