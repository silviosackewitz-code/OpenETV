# Native validation — 2026-10-26

Validated on macOS / Apple Silicon:

- 37 Rust tests passed, including optional local ECU fixtures, file-format round trips, clipboard editing, undo/redo, protected result axes, project validation, correction restoration and refusal to overwrite externally changed projects.
- 48 Python numerical-reference tests passed.
- 670 exact Rust/Python comparisons passed, including two local ECU files; no private values are embedded in fixtures or published diagnostics. The same comparator passed against the packaged release executable.
- Cargo fmt, Clippy with warnings denied, and Ruff passed.
- The five pages and working-principle view were exercised and rendered without a desktop window. Screenshots use bundled synthetic data.
- The native release app reports version 0.3.0. Its ad-hoc code signature and the generated DMG checksum verify successfully. The app was built, not installed.

The first GitHub run found formatting/strict-zip issues in the new reference script. Those were corrected; local Ruff now checks the whole repository and the native CI retains that check.

Not validated: OS file-dialog interactions and a visible native window, Windows/Linux runtime, ECU hardware or dyno behaviour. macOS packages are ad-hoc signed, not Apple-notarized. Engine-braking, cylinder cut, constant-power/KP and automatic multi-gear calculations remain outside this migration.

Release artifact: macOS arm64, OpenETV-0.3.0-macOS.dmg, SHA-256
`d332ae6b08f979c8fd270bf10feca6fcd564334c10199fa85f7d62ef07f0628f`.
The packaging shell script remains executable as before the migration.
