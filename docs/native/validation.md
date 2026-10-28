# Native validation — 2026-10-27

OpenETV 0.4.0, macOS / Apple Silicon:

- Native unit and integration tests cover formats, atomic saves, project restore, calculations, clipboard editing, undo, protected result axes, contextual help and result-view switching.
- 375 distinct frozen synthetic cases check numerical compatibility in Rust and through the executable's batch interface. No Python environment is used.
- Optional private ECU import fixtures were exercised locally; their data is not published.
- Cargo fmt and Clippy with warnings denied passed.
- The five workflow pages, nine Manual topics, and the working-principle view were rendered with the native harness. The light map was inspected alongside dark views. Map text contrast is tested throughout the colour scale.

The numerical implementation and its defaults are unchanged. Legacy code remains available in Git history, outside the working tree.

Windows/Linux runtime and ECU/dyno behaviour have not been validated. macOS packages are ad-hoc signed, not Apple-notarized.
