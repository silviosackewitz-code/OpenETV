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
  `.reference/` (local only) and skip without it.
- Lint: `.venv/bin/ruff check .` (also in CI, `tests.yml`).
- Chained commands that commit after tests: use `set -o pipefail`.

## Where things are
- The calculation: `tools/etvlib/` — numpy only, no pandas, no Streamlit, no
  window. `core.py` (inverting the engine table, post-processing), `curves.py`
  (generating a torque request), `tables.py` (a table, CSV and Excel),
  `dss.py` (Mectronik DataSubset files).
- The window: still `app.py` (Streamlit), a thin layer over `etvlib`. It is
  replaced in steps 3–6 of the plan.
- `.reference/` is not in git: the manual of the original tool and two real
  ECU exports. `torque demand.dss` there holds **throttle in %**, not torque —
  see the review in `docs/plan.md`.

## Trying the app
`.venv/bin/python -m streamlit run app.py`, or the configuration
`openetv-streamlit` in `.claude/launch.json`.
