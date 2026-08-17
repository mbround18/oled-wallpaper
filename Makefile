FLATPAK_MANIFEST := packaging/flatpak/ninja.boop.OledWallpaper.yml
APP_ID := ninja.boop.OledWallpaper
BUILD_DIR := build-dir
DIST_DIR := dist
FLATHUB_REPO := https://flathub.org/repo/flathub.flatpakrepo
RUNTIME_REF := org.freedesktop.Platform//25.08

.PHONY: help all build build-flatpak build-local bundle flatpak-repo \
	install install-flatpak install-local install-bundle \
	uninstall release enable-autostart enable-systemd \
	clean clean-cache clean-all

.DEFAULT_GOAL := help

## ---- Build -----------------------------------------------------------

all: build ## Alias for 'build'

build: ## Build: uses flatpak-builder if available, else a local cargo release build
	@if command -v flatpak-builder >/dev/null 2>&1; then \
		echo "flatpak-builder detected -> building Flatpak..."; \
		$(MAKE) build-flatpak; \
	else \
		echo "flatpak-builder not found -> building local release binary..."; \
		$(MAKE) build-local; \
	fi

build-flatpak: ## Build + install the Flatpak locally via flatpak-builder (dev loop)
	@echo "-> Ensuring Flatpak runtimes are available (may prompt)..."
	flatpak install --user --noninteractive flathub org.freedesktop.Platform//25.08 \
		org.freedesktop.Sdk//25.08 org.freedesktop.Sdk.Extension.rust-stable//25.08 2>/dev/null || true
	@echo "-> Running flatpak-builder"
	flatpak-builder --user --install --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	@echo "-> Flatpak build+install complete"

build-local: ## Build a local release binary with cargo (no Flatpak)
	@echo "-> cargo build --release"
	cargo build --release
	@echo "-> local build complete (target/release/oled-wallpaper)"

## ---- Package / distribute --------------------------------------------

bundle: ## Build a distributable .flatpak bundle file into dist/
	@echo "-> Building Flatpak repo and bundle into $(DIST_DIR)"
	@mkdir -p $(DIST_DIR)/repo
	flatpak-builder --repo=$(DIST_DIR)/repo --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	flatpak build-bundle --runtime-repo=$(FLATHUB_REPO) $(DIST_DIR)/repo $(DIST_DIR)/$(APP_ID).flatpak $(APP_ID)
	@echo "-> Bundle written to $(DIST_DIR)/$(APP_ID).flatpak"

flatpak-repo: ## Export the Flatpak build to a local OSTree repo in dist/repo
	@echo "-> Exporting flatpak repo to $(DIST_DIR)/repo"
	@mkdir -p $(DIST_DIR)/repo
	flatpak-builder --repo=$(DIST_DIR)/repo --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	@echo "-> Repo ready in $(DIST_DIR)/repo"

release: bundle install-bundle ## Build a .flatpak bundle and install it locally

## ---- Install / uninstall ----------------------------------------------

install: ## Install: uses Flatpak if available, else 'cargo install'
	@if command -v flatpak-builder >/dev/null 2>&1; then \
		$(MAKE) install-flatpak; \
	else \
		$(MAKE) install-local; \
	fi

install-flatpak: ## Build + install the Flatpak via flatpak-builder
	@echo "-> Installing Flatpak (build+install via flatpak-builder)"
	flatpak-builder --user --install --force-clean $(BUILD_DIR) $(FLATPAK_MANIFEST)
	@echo "-> Installed Flatpak: run with 'flatpak run $(APP_ID)'"

install-local: ## Install the binary locally with 'cargo install'
	@echo "-> Installing binary locally with 'cargo install --path . --force'"
	cargo install --path . --force
	@echo "-> Installed binary to ~/.cargo/bin"

install-bundle: ## Install the .flatpak bundle from dist/ (sets up Flathub remote + runtime)
	@echo "-> Installing bundle with runtime auto-setup"
	flatpak remote-add --if-not-exists --user flathub $(FLATHUB_REPO)
	flatpak install --user --noninteractive --assumeyes flathub $(RUNTIME_REF)
	@# Purge any stale .removed entries that cause "Directory not empty" on reinstall
	@rm -rf "$(HOME)/.local/share/flatpak/.removed/$(APP_ID)"* 2>/dev/null || true
	flatpak install --user --noninteractive --reinstall --assumeyes $(DIST_DIR)/$(APP_ID).flatpak
	@echo "-> Installed $(APP_ID) from bundle"

uninstall: ## Uninstall the Flatpak app
	@echo "-> Attempting to uninstall Flatpak app $(APP_ID)"
	flatpak uninstall --user -y $(APP_ID) || true

## ---- Autostart ----------------------------------------------------------

enable-autostart: ## Install the autostart .desktop entry to ~/.config/autostart
	@echo "-> Installing autostart .desktop to ~/.config/autostart (no system changes are made in CI)"
	@mkdir -p $(HOME)/.config/autostart
	@cp packaging/autostart/ninja.boop.OledWallpaper.desktop $(HOME)/.config/autostart/ninja.boop.OledWallpaper.desktop || true
	@echo "-> Autostart entry installed to $(HOME)/.config/autostart/ninja.boop.OledWallpaper.desktop"

enable-systemd: ## Enable the systemd --user service (requires a systemd user session)
	@echo "-> Enabling systemd --user unit (requires systemd user session)"
	systemctl --user daemon-reload || true
	systemctl --user enable --now ninja.boop.OledWallpaper.service || true
	@echo "-> Attempted to enable ninja.boop.OledWallpaper.service (check with 'systemctl --user status')"

## ---- Housekeeping ---------------------------------------------------------

clean: ## Remove build artifacts (build-dir, target, dist)
	rm -rf $(BUILD_DIR) target $(DIST_DIR)
	@echo "-> cleaned build artifacts ($(BUILD_DIR), target, $(DIST_DIR))"

clean-cache: ## Remove the flatpak-builder cache (.flatpak-builder) — can be 100+GB
	rm -rf .flatpak-builder
	@echo "-> cleaned .flatpak-builder cache"

clean-all: clean clean-cache ## Remove build artifacts AND the flatpak-builder cache

help: ## Show this help
	@echo "Usage: make [target]"
	@echo ""
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
