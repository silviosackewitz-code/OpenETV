# Native validation — 2026-10-27

## OpenETV 0.4.1 — macOS / Apple Silicon

- 45 native tests passed, including individual breakpoint editing, insertion, removal, duplicate prevention, undo, project round trips and generator/calculation agreement.
- A 16-row synthetic map verifies that the last row is visible and editable below the former 420-pixel clipping boundary, before the chart.
- All 375 frozen synthetic numerical cases still match in Rust and the batch interface. Calculation code and defaults are unchanged.
- The five workflow pages, nine Manual topics, working-principle view, light map and tall map were rendered with the native harness. Changed help pages and breakpoint layout were visually inspected.
- Cargo fmt and Clippy with warnings denied passed.

Windows/Linux runtime and ECU/dyno behaviour have not been validated. macOS packages are ad-hoc signed, not Apple-notarized.

The packaged 0.4.1 executable passes all 375 batch cases; its ad-hoc signature and DMG integrity pass. The bundle installed at `Developer/OpenETV_App/OpenETV.app` reports 0.4.1 and matches the packaged executable byte for byte.

The existing 0.4.0 window had unsaved changes and did not respond to automated save actions. Its process and original bundle were preserved. The update takes effect on the next launch; actual macOS interaction with 0.4.1 remains unchecked. Native harness interaction and rendered screenshots passed.

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
