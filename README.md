# OpenETV – Throttle Position Map Generator

A local desktop app that computes the throttle position a ride-by-wire ECU
drives to (ETV MAP / TPS Target) from an engine torque table (TORQUE DYNO:
RPM × Throttle → Torque) and a torque request (TORQUE TARGET: RPM × Pedal →
Target torque). It runs on your own machine and talks to nothing.

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
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

The app carries its own manual: **Manual** at the bottom left of the window,
or read it here — [docs/manual](docs/manual/README.md).

For testing and packaging: `.venv/bin/pip install -r requirements-dev.txt`,
then `.venv/bin/python -m pytest` and `.venv/bin/ruff check .` (the browser
tests need `.venv/bin/python -m playwright install webkit` once). Most people
need none of this — take a release instead, see **Installing** below.

The app is built like the sister project Race Analysis, and shares its look;
the plan, with a review of the calculation against real ECU data, is in
[docs/plan.md](docs/plan.md).

## Running from a checkout

```bash
PYTHONPATH=tools .venv/bin/python -m etvlib
```

A window of its own opens; `--browser` opens the page in the browser instead,
`--port 8766 --no-browser` only starts the local server. Nothing but
`127.0.0.1` is ever listened on, and every request from the page carries a key
of that run.

The window has the five steps in a sidebar: the engine torque table, the
torque request, the settings of the calculation, the ETV map and the export.
Tables are edited in place and take a paste from a spreadsheet.

## Features

**1) Engine Torque Table (TORQUE DYNO)**
- Editable in place, pasteable from a spreadsheet; negative values at TPS=0
  allowed (drag torque/friction), and a table cut at 0 Nm works as well
- Import via CSV, Excel, or Mectronik `.dss` export (`tools/etvlib/dss.py`)

**2) Demand Table (TORQUE TARGET)**
- Editable in place, RPM × Pedal → Target torque
- Import via CSV, Excel, or `.dss` (which may hold several tables, e.g. one
  per gear — you pick; a table that is in % rather than Nm is a throttle map,
  not a request, and says so instead of being calculated with)
- **Generate demand curve**: pick from example curves — cable-like feel
  (concave, n<1, mimics a 1:1 gas/throttle cable), linear, corner-exit
  precision (convex, n>1, keeps the low-to-mid pedal range fine for
  corner-exit throttle dosing while ramping up quickly once you commit to
  full power), or a fully custom S-curve with an adjustable fine-control
  zone boundary and transition sharpness. A live chart previews the curve
  shape as you change them. Both the gas breakpoints (fine near 0% to
  match real ECU exports) and the RPM breakpoints for the generated table are
  editable. By default the request is in absolute torque: full pedal asks for
  the engine's one maximum, and the same pedal means the same Nm at every RPM,
  capped where the engine has less — the torque stays where the rider holds
  the pedal while the revs rise. Scaling to the engine's maximum at each RPM
  instead is an option; the torque at a held pedal then follows the engine's
  torque curve again, as with a cable.

**3) Calculation (ETV MAP)**
- Editable RPM and pedal breakpoints for the output table (default: RPM axis
  of the demand table); both tables are read at the exact output RPM, the
  engine torque interpolated linearly between its RPM rows (rows without any
  torque, like a dummy row at 0 rpm, are left out), with a warning for RPM
  outside the engine table
- RPM Calc Method (threshold for the saturation case: first vs. last TPS
  breakpoint that reaches the max torque) and Max Torque Tolerance, matching
  the original tool

**4–5) Result & Post-processing**
- Warnings for saturated cells, values below the TPS=0 torque, and
  non-monotonic engine map rows
- Zero-gas fix: Pedal=0% → TPS=0%, then a straight ramp up to a chosen pedal
  breakpoint (default 20%, as in the original tool), so that no step is left
  behind the closed grip
- Flat-spot fix: a plateau, where TPS stands still while the pedal moves,
  becomes a straight ramp up to its last cell (as in the original tool)
- Remove dips: TPS monotonically non-decreasing over pedal for each RPM row

**6) Export**
- A calculated map is a first attempt: it moves a throttle, so check it on
  the dyno, pedal position by pedal position, before anyone rides it
- CSV, Excel, and Mectronik `.dss` (with editable table/axis paths and units
  for re-import into the ECU software)

## The .dss file format

`tools/etvlib/dss.py` reads and writes the Mectronik "DataSubset" XML format (see the
comments in the module). Round-tripping (import → export → import) produces
exactly identical values; tested against real ECU exports.

## Installing

Releases carry one file per system; nothing else has to be installed, Python
included.

- **macOS**: open `OpenETV-<version>-macOS.dmg` and drag OpenETV to
  Applications. The app is not signed, so macOS refuses the first start with a
  warning: right-click the app, choose **Open**, and confirm once.
- **Windows**: unpack `OpenETV-<version>-Windows.zip` anywhere and start
  `OpenETV\OpenETV.exe`; its `_internal` folder has to stay beside it.
  SmartScreen may warn about an unknown publisher (**More info → Run anyway**).
- **Linux**: unpack `OpenETV-<version>-Linux.tar.gz` and start
  `OpenETV/OpenETV`. Without a GUI toolkit for pywebview the page opens in the
  browser instead of a window of its own.

Theme and accent are kept in `~/.openetv/prefs.json`; nothing else is written
outside the files you save.

## Building

```bash
.venv/bin/pip install -r requirements-dev.txt
bash packaging/build.sh
```

On Windows: `powershell -ExecutionPolicy Bypass -File packaging\build.ps1`.
PyInstaller does not build across systems — build on the one you want to
target. The result goes **outside the repository**, to `../../OpenETV_App`
(`OPENETV_APP_DIR` names another place): the app, and the file a release
carries (`.dmg`, `.zip`, `.tar.gz`). The version is written once, in
`tools/etvlib/__init__.py`; the icons come from `packaging/icon/app-icon.svg`
(`packaging/icon/make_icon.py`).

A release is made by pushing a version tag: `.github/workflows/release.yml`
builds on macOS, Windows and Linux and attaches the three files to a draft
release. The tag has to name the version (`v0.2.0` for `0.2.0`).

## Not included (original tool features not yet reimplemented)

- Engine brake application (cylinder cut, torque-per-gear calculation,
  cut-pattern import)
- ETV Gain / KP (constant power) area for high RPM (chapter 3.2)
- Gearbox ratio import (Gearbox DataSubset)
- The original tool's software activation scheme (email-based `.req`/`.lic`
  file exchange) — this project has no such licensing/activation mechanism

## Third-party notices

The window's typeface (IBM Plex, OFL) and icons (Lucide, ISC) are the work of
others: see [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

## License

MIT — see [LICENSE](LICENSE). Contributions and improvements are welcome.
