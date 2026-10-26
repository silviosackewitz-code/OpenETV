# Changelog

## 0.3.0

- Replace the Python/browser desktop runtime with a Rust/egui native application while retaining the Python calculation as a numerical reference.
- Keep the five-step torque-table workflow, editable grids, request generators, calculation settings, result adjustments and CSV/XLSX/DSS import/export.
- Recover a modelled torque request from an existing ETV map.
- Add complete `.openetv` session files, 40-action undo/redo and recalculation requirements for stale results and baseline verification when restoring projects.
- Show heatmaps, available-torque overlays and requested/modelled torque comparisons.
- Reuse RaceAnalysis/RaceDash Dark, Light, Industrial and Terminal themes with system following, two font families and interface scaling.
- Embed the native manual, font assets and sample data; build platform packages without a Python runtime or local server.

Engine braking, cylinder cut, constant-power/KP areas, ETV gain, gearbox import and automatic multi-gear processing remain outside the implemented scope.
