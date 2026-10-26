> **Archived browser interface manual.** OpenETV 0.3.0 uses a native Rust window. Read the current [native manual](../../rust/MANUAL.md), also embedded in the application. The pages below describe the previous interface.

# OpenETV — manual

How to use the app, one page per step of the window. The pages describe what
you see and click; how the app is built is one folder up
([`docs/plan.md`](../plan.md)).

## The five steps

- [Getting started](getting-started.md) — installing, the window, and a first
  map in five minutes.
- [1 · Engine torque](engine-torque.md) — the table of what the engine
  delivers: editing it, loading one, what its shape means.
- [2 · Torque request](torque-request.md) — what the rider asks for, and the
  generator that builds it from the engine table.
- [3 · Calculation](calculation.md) — the breakpoints of the map, **RPM Calc
  Method** and **Max Torque Tolerance**.
- [4 · ETV map](etv-map.md) — the result, what its marks mean, and the three
  fixes.
- [5 · Export](export.md) — CSV, Excel and Mectronik `.dss`, and why a map
  belongs on the dyno first.

## Around the steps

- [Files](files.md) — which files can be read and written, and how a table has
  to look.
- [Appearance](appearance.md) — light, dark or as the system, the accent
  colour, and the **?** marks.

## Writing a page

- One Markdown file per step, named after it (`torque-request.md`), in English
  like the rest of the app.
- Screenshots go to `images/`, named `<page>-<what>.png`, and are taken with
  the tool that takes them all the same way, from the real window:
  `.venv/bin/python tools/screenshots.py` (WebKit, light theme, **?** marks
  off, two device pixels per CSS pixel). A new picture is one more function
  there, not a picture taken by hand.
- Use the labels exactly as the window shows them (**Zero-gas fix**, **Max
  Torque Tolerance**), in bold.
- Start with what the step is for, then what you see, then the details and the
  limits.
- Indent nested lists and paragraphs inside a list item by **4 spaces** — the
  app renders the pages with Python-Markdown, which needs it.
- Write file contents and commands as a block indented by 4 spaces, not
  between fences (` ``` `): the app does not render fences, the text would run
  together. `test_manual.py` fails on a line that opens one.
- A heading is an anchor: the **?** marks open a page at one
  (`data-help="etv-map#fix-flat-spots"`). Renaming a heading breaks its mark —
  `test_manual.py` fails then.
