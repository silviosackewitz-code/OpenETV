# Native validation — 2026-10-27

OpenETV 0.4.0, macOS / Apple Silicon:

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
