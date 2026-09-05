# OpenETV – Throttle Position Map Generator

Local Streamlit app that computes the correct throttle position (ETV MAP /
TPS Target) from an engine torque table (TORQUE DYNO: RPM × Throttle →
Torque) and a rider demand table (TORQUE TARGET: RPM × Pedal → Target torque).

Modeled after the ETV Builder workflow of the original "EGEA Bike Torque
Tool", whose manual served as a reference for the complete original feature
set (including engine brake/cylinder cut, which this reimplementation does
not yet cover) – as well as the principles for torque tables and gas map
design described in "A Practical Guide to Race Motorbike Electronics"
(chapters 2–3).

## In memory of Stéphane Egea

The original "EGEA Bike Torque Tool", on which this project is built, was
developed by Stéphane Egea (EGEA Engineering). He has passed away, and his
tool has not been maintained since. This reimplementation was built on his
freely documented approach and is intended as a continuation of his work in
his spirit – with thanks for the ideas and the care with which he made them
accessible.

## Setup

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## Running

```bash
source venv/bin/activate
streamlit run app.py
```

The browser opens automatically at `http://localhost:8501`.

## Features

**1) Engine Torque Table (TORQUE DYNO)**
- Editable, negative values at TPS=0 allowed (drag torque/friction)
- Import via CSV, Excel, or Mectronik `.dss` export (`dss.py`)

**2) Demand Table (TORQUE TARGET)**
- Editable, RPM × Pedal → Target torque
- Import via CSV, Excel, or `.dss` (with a selection dropdown if a file
  contains multiple tables, e.g. one per gear)
- **Generate demand curve**: pick from example curves — cable-like feel
  (concave, n<1, mimics a 1:1 gas/throttle cable), linear, corner-exit
  precision (convex, n>1, keeps the low-to-mid pedal range fine for
  corner-exit throttle dosing while ramping up quickly once you commit to
  full power), or a fully custom S-curve with an adjustable fine-control
  zone boundary and transition sharpness. A live chart previews the curve
  shape as you move the sliders. Both the gas breakpoints (fine near 0% to
  match real ECU exports) and the RPM breakpoints for the generated table are
  editable; max torque is linearly interpolated from the engine table for any
  RPM not falling exactly on one of its breakpoints.

**3) Calculation (ETV MAP)**
- Editable RPM and pedal breakpoints for the output table (default: RPM axis
  of the demand table); the engine map row is internally rounded to the
  nearest existing RPM breakpoint of the engine table (no RPM interpolation
  in the engine map, as in the original tool), with a warning when rounding
  occurs
- RPM Calc Method (threshold for the saturation case: first vs. last TPS
  breakpoint that reaches the max torque) and Max Torque Tolerance, matching
  the original tool

**4–5) Result & Post-processing**
- Warnings for saturated cells, values below the TPS=0 torque, and
  non-monotonic engine map rows
- Zero-gas fix (Pedal=0% → TPS=0%) and flat-spot fix (TPS monotonically
  non-decreasing over pedal for each RPM row)

**6) Export**
- CSV, Excel, and Mectronik `.dss` (with editable table/axis paths and units
  for re-import into the ECU software)

## The .dss file format

`dss.py` reads and writes the Mectronik "DataSubset" XML format (see the
comments in the module). Round-tripping (import → export → import) produces
exactly identical values; tested against real ECU exports.

## Building a standalone desktop app

A PyInstaller spec (`OpenETV.spec`) is included to build a double-clickable
app that bundles Python and all dependencies — no separate install needed by
the end user. PyInstaller does not cross-compile: build on the OS you want to
target.

```bash
source venv/bin/activate
pip install pyinstaller
pyinstaller OpenETV.spec --noconfirm
```

- **macOS**: produces `dist/OpenETV.app` — double-click to run.
- **Windows**: produces `dist/OpenETV/OpenETV.exe` (plus its `_internal`
  folder, which must stay next to it) — double-click `OpenETV.exe` to run.
- **Linux**: produces `dist/OpenETV/OpenETV` (plus `_internal`) — run
  `./OpenETV` from a terminal, or mark it executable and launch from a file
  manager (behavior varies by distro/desktop environment).

A GitHub Actions workflow (`.github/workflows/build.yml`) is included that
builds all three platforms automatically on real Windows/Linux/macOS
runners, in case this project is ever pushed to a GitHub repo and automated
builds become useful — it is not required for local builds.

## Not included (original tool features not yet reimplemented)

- Engine brake application (cylinder cut, torque-per-gear calculation,
  cut-pattern import)
- ETV Gain / KP (constant power) area for high RPM (chapter 3.2)
- Gearbox ratio import (Gearbox DataSubset)
- The original tool's software activation scheme (email-based `.req`/`.lic`
  file exchange) — this project has no such licensing/activation mechanism

## License

MIT — see [LICENSE](LICENSE). Contributions and improvements are welcome.
