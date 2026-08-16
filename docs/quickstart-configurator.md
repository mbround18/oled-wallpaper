# Quickstart: OLED Configurator (GUI)

Launch the configurator (dev):

```bash
cargo run --bin oled-config --release
```

Headless apply (CI):

```bash
target/release/oled-config --headless --apply "Night Sky" --config-dir /tmp/test-config
```

Usage section: the GUI includes an "Our Usage" collapsible with live CPU and Memory graphs for diagnostics. Note: Flatpak builds may restrict access to system metrics.
