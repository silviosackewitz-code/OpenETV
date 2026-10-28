# Native validation — 2026-10-28

## OpenETV 0.5.0 — native 3D maps

- 48 native tests passed. New checks cover true breakpoint spacing, signed and constant values, single-row/column maps, drag rotation, wheel zoom, reset, read-only viewing and Table/3D switching on all three maps.
- All 375 frozen numerical cases still match in Rust and the packaged executable. Calculation code and defaults are unchanged.
- Native screenshots cover the engine, request and ETV surfaces, light-theme line/point/flat cases, and ten concise Manual topics. Axes, colour scales and help were visually inspected.
- Cargo fmt, Clippy with warnings denied, package signature and DMG integrity passed.
- The installed `Developer/OpenETV_App/OpenETV.app` reports 0.5.0 and matches the packaged executable byte for byte. After the previous session closed, the installed app was launched. Switching to the 3D map and the complete surface with all axes were visually checked in the actual macOS window. Rotation, zoom and reset were exercised in the native harness.

Artifact: `OpenETV-0.5.0-macOS.dmg`, SHA-256 `20f70a001d202d7e2dddb8379bb90354deb6dc2b2b05bd0709faa8bb844ffdc9`.

GitHub CI passed for `ccf0b3c`: https://github.com/silviosackewitz-code/OpenETV/actions/runs/40647331415

## OpenETV 0.4.1 — macOS / Apple Silicon

- 45 native tests passed, including individual breakpoint editing, insertion, removal, duplicate prevention, undo, project round trips and generator/calculation agreement.
- A 16-row synthetic map verifies that the last row is visible and editable below the former 420-pixel clipping boundary, before the chart.
- All 375 frozen synthetic numerical cases still match in Rust and the batch interface. Calculation code and defaults are unchanged.
- The five workflow pages, nine Manual topics, working-principle view, light map and tall map were rendered with the native harness. Changed help pages and breakpoint layout were visually inspected.
- Cargo fmt and Clippy with warnings denied passed.

Windows/Linux runtime and ECU/dyno behaviour have not been validated. macOS packages are ad-hoc signed, not Apple-notarized.

The packaged 0.4.1 executable passes all 375 batch cases; its ad-hoc signature and DMG integrity pass. The bundle installed at `Developer/OpenETV_App/OpenETV.app` reports 0.4.1 and matches the packaged executable byte for byte.

After the maintainer closed the previous app, the installed 0.4.1 bundle was launched. Startup, navigation to Calculation, individual breakpoint controls, wrapping and labelled settings were checked in the actual macOS window. Edit/add/remove, undo and calculation interactions passed in the native harness; these mutations were not repeated in the live session.

GitHub CI passed for `a5a0728`: https://github.com/silviosackewitz-code/OpenETV/actions/runs/40644056178

Artifact: `OpenETV-0.4.1-macOS.dmg`, SHA-256 `e3a98340fa09e7adedeea09f15391b8fa1e6b9fe030856ddfa5e03dc4bd631c9`.

## Previous verification — 0.4.0

- 43 native tests passed. Unit and integration tests cover formats, atomic saves, project restore, calculations, clipboard editing, undo, protected result axes, contextual help and result-view switching.
- 375 distinct frozen synthetic cases check numerical compatibility in Rust and through the executable's batch interface. No Python environment is used.
- Optional private ECU import fixtures were exercised locally; their data is not published.
- Cargo fmt and Clippy with warnings denied passed.
- The five workflow pages, nine Manual topics, and the working-principle view were rendered with the native harness. The light map was inspected alongside dark views. Map text contrast is tested throughout the colour scale.

The numerical implementation and its defaults are unchanged. Legacy code remains available in Git history, outside the working tree.

Windows/Linux runtime and ECU/dyno behaviour have not been validated. macOS packages are ad-hoc signed, not Apple-notarized.

The packaged release executable passes all 375 frozen cases through its batch interface. The installed `Developer/OpenETV_App/OpenETV.app` reports 0.4.0. Its signature and DMG integrity checks pass. Startup, contextual Manual, calculation, torque-comparison switching, undo and the unsaved-project close prompt were checked in the actual macOS window; the test session was discarded.

GitHub CI passed for code revision `39353a6`: https://github.com/silviosackewitz-code/OpenETV/actions/runs/40641597600

Artifact: `OpenETV-0.4.0-macOS.dmg`, SHA-256 `b3dc1f4b2cf93a973fa769b9431c9ad60429d44c5bca4c56e5f708ac308a72d7`.
