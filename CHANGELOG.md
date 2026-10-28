# Changelog

## 0.5.0

- Add a native 3D surface view for engine torque, torque request and ETV maps.
- Rotate, tilt and zoom with mouse or labelled controls; inspect point values on hover and reset the view.
- Preserve actual breakpoint spacing and table colour scales; keep 3D viewing read-only.

## 0.4.1

- Edit generator and output breakpoints individually, with labelled add/remove controls and units.
- Arrange calculation settings in labelled rows.
- Show every map row without a fixed-height inner scrollbar; tall maps use page scrolling.

## 0.4.0

- Remove the former HTML/JavaScript/Python application, tooling, packaging and archived manual. Build and test only Rust.
- Preserve 375 distinct synthetic numerical regression cases as native test data.
- Colour editable map cells, show a value legend and chosen-cell outline, and keep throttle colours on a fixed 0–100% scale.
- Give navigation clearer spacing; keep the request map visible ahead of optional generator and preview controls.
- Switch the result chart between throttle and torque comparison.
- Add nine concise offline Manual topics and contextual help, following the Race Analysis writing conventions.

## 0.3.0

- Replace the Python/browser desktop runtime with a Rust/egui native application while retaining the Python calculation as a numerical reference.
- Keep the five-step torque-table workflow, editable grids, request generators, calculation settings, result adjustments and CSV/XLSX/DSS import/export.
- Recover a modelled torque request from an existing ETV map.
- Add complete `.openetv` session files, 40-action undo/redo and recalculation requirements for stale results and baseline verification when restoring projects.
- Show heatmaps, available-torque overlays and requested/modelled torque comparisons.
- Reuse RaceAnalysis/RaceDash Dark, Light, Industrial and Terminal themes with system following, two font families and interface scaling.
- Embed the native manual, font assets and sample data; build platform packages without a Python runtime or local server.

Engine braking, cylinder cut, constant-power/KP areas, ETV gain, gearbox import and automatic multi-gear processing remain outside the implemented scope.
