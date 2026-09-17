# OpenETV — working conventions

The same conventions as Race Analysis (`../Race_Analysis/CLAUDE.md`); OpenETV
is being rebuilt on that project's structure and look. The plan, with the
review of the calculation it starts from, is [`docs/plan.md`](docs/plan.md) —
tick a step there when it is done.

## Language
- Everything that belongs to the app is written in **English**: code,
  identifiers, comments, docs, commit messages.
- Conversation with the maintainer is in German.

## Commits
- One logical change per commit. Every commit must pass the test suite.
- **Do not push without asking.** `origin` is a public GitHub repository, and
  `.github/workflows/build.yml` builds the app on three systems at every push
  to `main` (until step 7 of the plan changes that to version tags).
- **Nothing from the maintainer's books goes into the repository** — no text,
  no paraphrased passages, no example numbers, no figures, not in files and
  not in commit messages (the repository is public). Notes on them stay in
  `.reference/`, which git ignores. The freely distributed manual of the
  original tool may be referred to. Before a push, search what goes up:
  `git log -p origin/main..HEAD | grep -i book`.
- A change to what the calculation returns is never mixed with a
  refactoring: the result goes into an ECU that moves a throttle. Such a
  change gets its own commit and its own tests, and the maintainer decides it
  first.

## Checks
- Tests: `.venv/bin/python -m pytest`. Tests against real ECU exports need
  `.reference/` (local only) and skip without it. This includes the window's
  JavaScript (`tools/tests/js/`, needs Node) and a click-through in WebKit
  (`test_etv_browser.py`, needs `.venv/bin/python -m playwright install webkit`
  once). UI changes: run the browser tests and extend them.
- Trying something in a browser by script is not enough for the grid: without
  system focus no focus events fire, and a click places its caret *after* the
  focus — both were found only by the WebKit click-through with real keys.
- Lint: `.venv/bin/ruff check .` (also in CI, `tests.yml`).
- Chained commands that commit after tests: use `set -o pipefail`.

## Where things are
- The calculation: `tools/etvlib/` — numpy only, no pandas, no Streamlit, no
  window. `core.py` (inverting the engine table, post-processing), `curves.py`
  (generating a torque request), `tables.py` (a table, CSV and Excel),
  `dss.py` (Mectronik DataSubset files).
- The server: `etvlib/server.py` (127.0.0.1 only, host check, a token per
  run embedded in the page — built like `racelib/server.py`) and `etvlib/api.py`
  (the routes under `/api/`; the server keeps nothing between requests, the
  page holds the tables).
- The window: `etvlib/etv/` — `etv.html`, `etv.css` and one ES module per
  area, no build step. The rules are those of the analyzer in Race Analysis:
  **nothing runs on import** (each module wires itself in `init()`, `main.js`
  calls them in a fixed order), **shared state only in `state.js`**, and a
  module **says what changed** (`events.js`: `engine`, `request`, `settings`,
  `result`, `step`) instead of calling whoever shows it —
  `test_etv_import_graph.py` fails on an import ring. `tablemath.js` (what an
  editable table does to its numbers) and `ramp.js` (the heatmaps' value ramp,
  the one module that may name colours) touch neither page nor server, so Node
  tests them. `etvlib/ui.py` opens the page in a pywebview window, or in the
  browser without it; saving goes through the window's save dialog
  (`/api/save`), in a browser through its downloads.
- The look: `theme.css`, `fonts.css`, `fonts/`, `theme.py`, `theme.js`,
  `prefs.js`, `common.js`, `icons.js` in `tools/etvlib/` are **copies from Race
  Analysis — never edit them here**; `tools/sync_look.py` fetches them again
  (rules: `docs/design.md`). Pages name no colour, only roles of the theme.
  Preferences (theme, accent): `etvlib/prefs.py`, `~/.openetv/prefs.json`;
  try the app with `OPENETV_HOME=<folder>` so as not to write one's own.
- `app.py` (Streamlit) is the window of the first version, still there beside
  the new one until step 6 of the plan removes it.
- `.reference/` is not in git: the manual of the original tool and two real
  ECU exports. `torque demand.dss` there holds **throttle in %**, not torque —
  see the review in `docs/plan.md`.

## Trying the app
- The Streamlit window: `.venv/bin/python -m streamlit run app.py`
  (`openetv-streamlit` in `.claude/launch.json`).
- The new window: `PYTHONPATH=tools .venv/bin/python -m etvlib` (`--browser`
  opens the page in the browser instead); `--port 8766 --no-browser` starts
  the server only, at a fixed address
  (`openetv-server` in `.claude/launch.json`). A session started in another
  project does not see this `launch.json` — start the server in the
  background and open its address instead.
