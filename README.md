# OpenETV - Throttle Position Map Generator

OpenETV is a local Rust desktop app for building a ride-by-wire throttle target map from an engine torque table and a rider torque request. It runs without Python, a browser or a local server. The calculation stays on your computer.

Engine torque is RPM × throttle (%) → torque (Nm). The request is RPM × grip (%) → torque (Nm). The resulting ETV map is RPM × grip (%) → throttle (%). These are different tables: a throttle-percent export must not be used as a torque request in Nm.

## In memory of Stéphane Egea

The original EGEA Bike Torque Tool was developed by Stéphane Egea of EGEA Engineering. This independent reimplementation follows the workflow documented in his freely distributed manual, with thanks for the ideas and care he made accessible. It does not reproduce the complete original tool.

## Using the app

The five sidebar steps are Engine torque, Torque request, Calculation, ETV map and Export. **Manual** opens short task and reference pages; **?** beside a page title opens help for that step. The same pages are available from [rust/MANUAL.md](rust/MANUAL.md).

- Import CSV, XLSX or Mectronik DSS tables; select a table from a multi-table file and edit cells directly.
- Generate a power-shaped or custom S-curve request, with explicit absolute-torque or per-RPM scaling. Recover a modelled torque request from an existing ETV map and the loaded engine table.
- Edit output breakpoints individually with add/remove controls; set RPM calculation threshold and maximum-torque tolerance. Inspect saturation and out-of-range warnings.
- Apply the low-grip zero-gas ramp, flat-spot correction or dip removal; reset to the raw calculation when needed.
- Switch between the editable **Table** and a native **3D map** on all three maps. Rotate, tilt, zoom and inspect point values; actual breakpoint spacing and map values are preserved.
- Read and edit coloured maps with a visible scale and chosen-cell outline. Every row is shown without a separate vertical table scrollbar; tall maps use page scrolling. Throttle maps use a fixed 0–100% scale; Nm maps use their value range. Switch between the throttle curve and requested versus modelled torque. A modelled comparison is not a measurement.
- Save and reopen JSON `.openetv` sessions containing the inputs, settings and result. Undo and redo retain the latest 40 actions. Changed inputs mark a result stale; loaded sessions verify their saved calculation baseline before retaining result corrections.
- Export CSV, XLSX or DSS with explicit destination and DSS table/axis paths and units.

A calculated map is a first attempt. Validate it on a dyno across grip positions and engine speeds before riding. OpenETV does not validate actuator hardware or other ECU control strategies.

Appearance follows Race Analysis and RaceDash: Dark, Light, Industrial and Terminal, plus system following; IBM Plex and JetBrains Mono; an adjustable interface scale. Industrial and Terminal keep their own signal colours. Preferences are saved in `~/.openetv/prefs.json`, preserving unrelated existing keys. `OPENETV_HOME` selects a separate preferences directory for testing.

## Running from source

Install a current stable Rust toolchain, then run:

```sh
cargo run --manifest-path rust/Cargo.toml --locked -j3
```

Linux builds need X11/Wayland and OpenGL development libraries; see the release workflow for the package list. Linux file dialogs use the desktop portal. No Python environment is needed to run the native app.

## Checking changes

```sh
cargo fmt --manifest-path rust/Cargo.toml --check
cargo clippy --manifest-path rust/Cargo.toml --locked --all-targets -j3 -- -D warnings
cargo test --manifest-path rust/Cargo.toml --locked -j3
cargo build --manifest-path rust/Cargo.toml --locked -j3
```

The native suite includes 375 frozen numerical cases from the previous implementation, using only synthetic inputs. Calculation, generation, reverse lookup and corrections are checked without a second runtime.

Original ECU fixtures in `.reference/` are private and optional; public CI uses sample and synthetic data. Never publish private fixtures or their values.

## Building and installing

Build on the destination platform:

```sh
bash packaging/build.sh
```

On Windows use `powershell -ExecutionPolicy Bypass -File packaging\build.ps1`. The scripts use the version in `rust/Cargo.toml` and write outside the repository to `../../OpenETV_App`. `OPENETV_APP_DIR` overrides that location; `CARGO_TARGET_DIR` overrides the Cargo build directory. `OPENETV_SKIP_DMG=1` skips disk-image creation on macOS.

- macOS: open the DMG and drag OpenETV to Applications. The build is signed ad hoc, not notarized with an Apple developer identity; macOS may require explicit first-launch approval.
- Windows: unpack the ZIP and run `OpenETV/OpenETV.exe`. SmartScreen may identify an unknown publisher.
- Linux: unpack the tar archive and run `OpenETV/OpenETV` in a graphical session with the required system libraries.

The scripts create packages without installing them. A matching version tag triggers three platform builds and a draft GitHub release; manual workflow dispatch builds artifacts only. Pushing a release tag requires a separate explicit decision.

## Limits and migration

Engine braking, cylinder cut, torque-per-gear and gearbox import, constant-power/KP areas and ETV gain processing, and automatic multi-gear processing are not implemented. A file may contain several tables, but calculations operate on the selected table.

Imports are limited to 16 MB and tables to 20,000 cells.

See [rust/NATIVE-MIGRATION.md](rust/NATIVE-MIGRATION.md) for the native migration and public research sources.

Local verification is recorded in [docs/native/validation.md](docs/native/validation.md); [native screenshots](docs/native/README.md) show the rebuilt interface.

## Third-party notices and licence

The bundled font and icon licences and provenance are in [rust/assets/look](rust/assets/look/PROVENANCE.md), including IBM Plex, JetBrains Mono and Lucide. Existing project notices remain in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

MIT - see [LICENSE](LICENSE).
