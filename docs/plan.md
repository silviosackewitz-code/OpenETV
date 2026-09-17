# Plan: OpenETV rebuilt on the structure and look of Race Analysis

Written 2026-09-18. Two parts: what a review of the calculation found (it
changes what has to be built), and the steps of the rebuild.

## What the app is for

A ride-by-wire ECU does not pass the grip to the throttle. It reads the grip
as a **torque request** and opens the throttle as far as the engine needs to
deliver that torque. Three tables are involved:

| Table | Axes | Value |
| --- | --- | --- |
| Engine torque (TORQUE DYNO) | RPM × throttle % | torque the engine delivers, Nm |
| Torque request (TORQUE TARGET) | RPM × grip % | torque the rider asks for, Nm |
| ETV map (the result) | RPM × grip % | throttle % the ECU drives to |

OpenETV computes the third from the first two: per RPM, the engine's torque
curve over throttle is read backwards — "which throttle gives this torque?".
What the rider feels is decided in the torque request; the ETV map only makes
the engine follow it.

**Why torque and not throttle:** with a cable, the same grip gives a different
torque at every RPM, and it keeps changing while the engine gains revs — the
rider has to correct for the engine. With a torque request the same grip can
mean the same torque at every RPM, and traction control, anti-wheelie and
engine-brake strategies all act on that one quantity instead of pulling at the
throttle each for itself.

## Review of the calculation (2026-09-18)

Checked against the manual of the original tool (pages 33–41) and against two
real exports of a Mectronik ECU (`.reference/`, local only, not in git).

### The core is right

`Demand.Dry.Gear456` from the real ECU, read as throttle % and sent forward
through the real engine table, gives **torque in Nm ≈ grip in %** at every RPM
from 4000 to 11500 (grip 30 % → 28 Nm, 50 % → 50.5 Nm, 80 % → 81 Nm, capped at
the engine's maximum). So that map was built from a linear torque request of
1 Nm per % grip. Feeding exactly that request into OpenETV's inversion
reproduces the real map to **1–2 % throttle between 20 and 80 % grip**. The
inversion does what the original does.

### What is wrong or missing

1. **Engine rows that do not rise all the way are inverted wrongly.** The
   lookup is a binary search, which needs a sorted row. 8 of the 18 real rows
   are not sorted (4500 rpm: 85.3 Nm at 60 % throttle, 83.6 at 80 %, 84.3 at
   100 %). Identity check — request the engine's own torque, expect throttle =
   grip — is off by up to 50 % throttle in those rows. Today this is a
   warning; with real data it is the normal case.
   *Proposal:* take the first crossing coming from closed throttle (the
   smallest opening that delivers the torque); above the first peak the
   saturation rule decides, as now.

2. **Output RPM that the engine table lacks are snapped to the nearest row.**
   The real engine table has a dummy row at 0 rpm (all zero) and starts at
   4000; the real map has rows at 2000 and 3000. 2000 snaps to the dummy row:
   the whole 2000 rpm row of the result is 0 % throttle. The original never
   had the case (both tables share their breakpoints from the `.setup` file),
   so "as in the original" does not apply.
   *Proposal:* interpolate the engine row linearly between its neighbours,
   ignore all-zero rows, say so where a row lies outside the table.

3. **The zero-gas fix is half of the original's.** The original sets 0 % grip
   to 0 % throttle **and interpolates linearly up to the 20 % grip
   breakpoint** (manual p. 39; its picture shows 0, 0.7, 1.3, 1.9, 2.5 … 10.0).
   OpenETV only sets the first column. With the real tables that leaves a
   step of 6–16 % throttle between 0 and 2 % grip. The real ECU map has the
   ramp.

4. **The flat-spot fix does something else than the original's.** A flat spot
   is a plateau: throttle does not change while the grip moves ("gas release
   delay", p. 35). The original replaces the plateau by a linear ramp up to the
   last cell (p. 40: 32.5, 35.0, 37.5, 40.0, 42.5, 45.0). OpenETV takes the
   running maximum, which removes dips and **creates** plateaus. Both are
   useful; they are two functions.

5. **Units are not checked.** The user's own file `torque demand.dss` holds
   throttle in % (that is the finding above), not torque in Nm. Loaded as a
   torque request it is accepted silently and gives a plausible-looking,
   meaningless map.
   *Proposal:* read the unit; a `%` table offered as torque request is
   refused with a sentence that says what it is.

6. **The way back is missing**: torque request from an existing ETV map
   (original: "Calculate torque request from ETV map tab"). It is the natural
   start with a real ECU: load the map that is in the bike, see the torque
   request it means, reshape that, invert again. It also gives the true
   "cable" request (engine torque at throttle = grip), which the generator's
   fixed exponent 0.6 only imitates.

7. **The generator works against the idea it serves.** It scales the shape to
   the engine's maximum *at each RPM*, so at a fixed grip the torque follows
   the engine's torque curve over RPM — the very thing a torque request is
   there to remove (see "Why torque and not throttle"). The real ECU map is
   built the other way: the same Nm for the same grip at every RPM, with one
   maximum for the whole engine, capped where the engine has less.
   *Proposal:* the generator's default becomes absolute torque; "share of the
   maximum at each RPM" stays as an option and says what it does.

8. **Engine tables with and without negative torque.** The real engine table
   is cut at 0 Nm (at 8000 rpm it is 0 Nm up to 10 % throttle); OpenETV's
   sample data has negative torque at closed throttle, and that is where
   "0 Nm is not 0 % throttle" comes from. The first version of this review
   proposed to use only the positive part of a table.
   *Decided while fixing (2026-09-18): the calculation is not changed.* That
   proposal would be wrong: the original tool's manual says its algorithm
   follows the request through the complete table, negative torque included,
   and the negative values are what fixes *where* the torque crosses zero —
   cut them off, and the first part of the grip gives engine braking where
   torque was asked for. Both kinds of table are valid, with negative values
   (as the original tool's) and cut at 0 Nm (as the real ECU export); both
   need the zero-gas ramp at the end.

9. Small: one table per run although real files hold one per gear; grip ends
   at 98 % and RPM at 12000 in the real map, beyond the engine table — clamped
   without a word; `.DSS` in capitals is not recognised; `.xls` is handled in
   code but not offered.

10. Project: no tests, calculation inside the Streamlit script, 230 MB build
    (pyarrow 112 MB, Streamlit 29 MB, pandas 17 MB — Race Analysis is 46 MB),
    build output and a 401 MB venv inside the synced folder, version written
    in the spec, no icon, the built app opens a browser tab.

11. A calculated map is a first attempt: the engine table it comes from is a
    measurement with errors of its own. It has to be checked on the dyno, grip
    position by grip position, before it is ridden, and the app should say so
    where it exports.

**The result goes into an ECU that moves a throttle.** Points 1–4 and 7 change
numbers; each is its own commit with its own tests, after the refactoring, and
the maintainer decides the proposals first.

## The rebuild

Same construction as Race Analysis: a package that only calculates, a small
local server (127.0.0.1, token), pages made of ES modules, a pywebview window,
every colour in `theme.css`. Streamlit, altair, pandas and pyarrow go;
openpyxl stays for Excel.

```
tools/etvlib/      __init__.py (version), core.py, curves.py, dss.py, tables.py,
                   api.py, server.py, ui.py, cli.py, theme.css, fonts/, icons.js …
tools/etvlib/etv/  etv.html, etv.css, main.js, state.js, grid.js, heatmap.js, …
tools/tests/       test_core.py, test_dss.py, test_server.py, test_theme.py, js/
packaging/         launcher.py, openetv.spec, build.sh, build.ps1, icon/
docs/              plan.md, design.md, manual/
```

The look is copied from Race Analysis with a note of the commit it came from
(`tools/sync_look.sh` fetches it again; a test keeps colour values out of the
pages). Pages link `/fonts.css` and `/theme.css` exactly as there, so that
OpenETV could later become a window of Race Analysis: package into that
repository, one route, one button.

The window: a sidebar with the steps (Engine table, Torque request,
Calculation, Result, Export), the chosen step on the right, one primary
action. Tables are an editable grid in Plex Mono that takes a paste from
Excel; heatmap and curves are drawn on a canvas in the dark scope; warnings
are banners and the cells they mean are marked; a result older than its
inputs says so.

### Steps

One commit per step or part of a step; every commit passes the tests.

- [x] **1. Scaffold** — Ruff, pytest, pinned requirements, `.venv`,
  `CLAUDE.md`, a CI job for tests.
- [x] **2. The calculation as a package** — `etvlib` on numpy, without pandas
  and without Streamlit; tests, among them the ones above as known defects.
  The Streamlit app keeps running on the package, numbers unchanged.
  *(done: old and new compared bit for bit before the old code went — 709
  comparisons, 33 084 result cells, sample data and the real ECU tables, every
  preset of the generator, .dss output byte-identical. The app clicked through
  with Streamlit's AppTest. The rebuilt PyInstaller spec is not built yet.)*
- [x] **2b. Fix the calculation** *(done: 1 — identity on the rising
  part of all 18 real rows exact, was off by up to 50 % throttle; the real map
  over all rows from 4000 rpm now 1.2 % off on average; 3 — with the ramp to
  20 % grip the real map's first 20 % of grip are met to 1.4 % on average,
  5.3 % at worst, was 5.6 % and 16.6 %; 2 — 2000 and 3000 rpm of the real map
  take the engine's 4000 rpm row instead of the dummy row, which gave a whole
  row of 0 % throttle, and are named as outside the engine table; 4 — `flat_spot_fix` beside
  `monotonic_fix`, straight in grip as the original's pictures show; 5 — a `.dss` table that is
  not in Nm is refused with a sentence, the real "demand" tables among them;
  7 — the generator's default is absolute torque: from the real engine table,
  linear and 100 %, the whole chain meets the real map to 1.1 % throttle over
  grip 20–80 %, was 3.0 % with the share per RPM, and the torque at a held
  50 % grip is 50.7 Nm at every RPM, was 41.2 to 50.7; 8 — no change, see
  there; 11 — the export says that a calculated map belongs on the dyno
  first)* — review points 1–5, 7, 8 and 11, one commit
  each, after the maintainer's decision.
- [ ] **3. Server and API** — parse, generate, calculate, post-process,
  export; token and host check as in Race Analysis.
- [ ] **4. The look** — theme, fonts, icons, `sync_look.sh`, `test_theme.py`;
  theme and accent as preferences. Best after Race Analysis has pushed its
  phase 3.
- [ ] **5. The window** — page, modules, grid, heatmap, curve preview, export
  through the system's file dialog; JS tests and a WebKit click-through.
- [ ] **6. Remove Streamlit** — `app.py`, `run_app.py`, `Start App.command`;
  README rewritten.
- [ ] **7. Build and install** — `packaging/build.sh` / `build.ps1` to
  `../../OpenETV_App`, version from `__init__.py`, icon; macOS: `.app` in a
  DMG, Windows: zip, Linux: tar; GitHub builds on a version tag only and
  attaches the files to a release.
- [ ] **8. Manual** — `docs/manual/`, one page per step, shown under Help.
- [ ] **9. Features from the review** — the way back (point 6), one run over
  all gears (9), the throttle-at-maximum line (1).
  What the original tool has beyond that (its manual, read in full): table
  tools for the engine table (interpolate a row or a column; offset, multiply,
  fill the selected cells; a 3D view); in the torque request, marking the
  torque the engine has available, a KP area laid over a selection, and a gain
  table applied to a 1:1 request; and engine brake as a part of its own —
  friction torque, indicated torque per cylinder, torque by the number of
  cylinders firing, cut pattern and gearbox from `.dss`, torque at the wheel
  per gear, request and throttle in both directions there too.

### Decisions for the maintainer

- The proposals of review points 1–5, 7 and 8.
- Which book the README should name: the chapter numbers it cites may belong
  to the same author's second book rather than to the one it names.
- Signing on macOS: without an Apple developer account Gatekeeper warns at
  the first start (right click → Open). Proposal: ship unsigned first, explain
  it in the README.
- Pushing: `origin` is a public GitHub repository and `build.yml` builds on
  three systems at every push to `main`.
