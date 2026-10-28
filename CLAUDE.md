# OpenETV - working conventions

OpenETV is a Rust/egui desktop application. Only the native application is maintained. Frozen synthetic regression data preserves the verified numerical behaviour.

## Language and publication

App code, comments, documentation and commits are English. Conversation with the maintainer is German.

One logical change per commit; every commit must pass its relevant checks. Do not push without asking unless the maintainer already explicitly authorized that push in the current session. Never push a version tag without explicit authorization: it starts the three-platform draft release workflow.

Nothing from the maintainer's books goes into the public repository: no text, paraphrases, example numbers, figures or commit-message material. Private notes and real ECU exports remain in ignored `.reference/`. The original freely distributed tool manual may be referenced. Before publishing, inspect the outgoing diff and history for private content.

Calculation changes affect an ECU throttle target. Keep changes to numerical behaviour separate from refactoring, cover them with tests and obtain the maintainer's decision before changing established behaviour or defaults.

## Native structure

- `rust/src/model.rs`: table validation, engine interpolation, inversion, generation and result adjustments.
- `rust/src/formats.rs`: CSV, XLSX, DSS; preserve units and atomic saves.
- `rust/src/app.rs`, `grid.rs`: five-step window, editing, charts, stale-result protection and undo.
- `rust/src/look/`, `rust/assets/look/`: copied RaceAnalysis/RaceDash themes, fonts and icons. Preserve provenance and licences. Make shared appearance changes upstream and copy them deliberately; do not create independent colour definitions in product pages.
- `rust/MANUAL.md`: embedded native manual through `rust/src/manual.rs`. Every app change must update its manual text when behaviour changes.
- `rust/NATIVE-MIGRATION.md`: native migration scope and public-source rationale.
- `rust/tests/fixtures/`: public synthetic numerical baselines and shared palette reference. No private ECU data.
- Help follows the concise task/operating/technical-reference conventions in `docs/manual-writing.md`.

Keep implementation small, use established helpers and avoid additional frameworks. Tables in Nm and throttle maps in percent are distinct inputs. Never silently reinterpret units or change an axis to make an import fit. Stale results must be recalculated before export. Restored results retain their corrections only after verifying the saved baseline against the current inputs.

## Checks

Run `cargo fmt --manifest-path rust/Cargo.toml --check`, `cargo clippy --manifest-path rust/Cargo.toml --locked --all-targets -j3 -- -D warnings`, and `cargo test --manifest-path rust/Cargo.toml --locked -j3`.

Numerical baselines run as native integration tests. Optional private ECU fixtures must stay private, including failure diagnostics.

UI changes require the native headless interaction tests and rendered screenshots. Check actual OS interaction before a public release; headless tests do not prove file-dialog or window-manager behaviour. Test with `OPENETV_HOME` pointing to a scratch directory to preserve personal preferences.

## Packaging

Version is in `rust/Cargo.toml`. `bash packaging/build.sh` builds macOS/Linux; `packaging/build.ps1` builds Windows. Default output remains `../../OpenETV_App`; override with `OPENETV_APP_DIR`. These scripts package only and must not install or replace an installed application. The macOS output is an ad-hoc-signed `.app` and DMG; Windows uses ZIP and Linux tar.gz. No Python runtime, local HTTP server or browser is needed.

Before delivery, run the packaged executable in batch mode, check its five steps through the native harness and file round trips, and record what was actually verified. A successful compile is not a GUI check. Do not claim cross-platform runtime validation from a macOS build.
