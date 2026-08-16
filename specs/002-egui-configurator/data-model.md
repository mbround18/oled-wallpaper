# Data Model: Egui Configurator (specs/002-egui-configurator)

## Configuration (root)
TOML structure persisted to `$XDG_CONFIG_HOME/oled-wallpaper/config.toml`.

Fields:
- `version` (int): Config schema version
- `animation` (table): animation settings
  - `planet_speed` (float, 0.1-5.0)
  - `camera_zoom` (float, 0.1-10.0)
- `widgets` (table): widget settings
  - `clock` (table): { enabled: bool, x: int, y: int, float_mode: bool }
  - `calendar` (table): same shape as clock
- `presets` (array of tables): named preset objects
  - `name` (string)
  - `config` (table): full config snapshot

## Types & Validation
- Colors: [r,g,b,a] arrays of floats 0.0-1.0
- Sizes: positive integers (pixels)
- Positions: integers within viewport bounds (validated in UI)
- Preset names: non-empty, ASCII-safe

## Methods
- `load_config(path) -> Result<Config>`: loads and validates schema; on failure returns descriptive errors
- `save_config(path, &Config) -> Result<()>`: atomically writes new config (write temp -> rename)
- `apply_preset(&mut Config, preset_name) -> Result<()>`
- `export_preset(path, &Preset) -> Result<()>`
- `import_preset(path) -> Result<Preset>`: validates and returns

## State Transitions
- Default config created on first run
- User edits fields in UI (transient state) -> validation -> save -> persisted
- Preset CRUD: create -> apply -> delete

