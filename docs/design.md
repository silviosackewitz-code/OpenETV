# Design — how the window looks, and where that is decided

OpenETV looks like Race Analysis because it uses the same files: the theme
(`theme.css`), the fonts, the icons and the shared scripts are **copies** of
that app's, unchanged, served at the same addresses (`/theme.css`,
`/fonts.css`, `/common.js`, `/icons.js`, `/theme.js`, `/prefs.js`). The rules
of the look — roles, scales, components, icons, the scope that is dark in every
theme, how a canvas reads a colour — are written down once, in
`docs/design.md` of Race Analysis, and apply here as they stand.

What is particular to OpenETV:

- **Never edit a copied file here.** Change it in Race Analysis, commit there,
  and run `.venv/bin/python tools/sync_look.py` (`--check` only says what
  differs). It copies from what is committed there and writes
  `tools/etvlib/look.json` with the commit and a checksum per file;
  `test_look.py` fails when a copy no longer matches.
- A page links `/fonts.css`, then `/theme.css`, then its own stylesheet, which
  says where things are and how big — **never what colour**. `test_look.py`
  fails on a colour value outside the theme, on a role that does not exist and
  on an icon that does not exist; `test_server.py` checks that the page links
  the theme.
- Theme and accent are the user's choice, kept in `~/.openetv/prefs.json`
  (`etvlib/prefs.py`; `OPENETV_HOME` names another folder). The server writes
  them into the page it serves (`theme.page_look`), so the window never shows
  another look first.
- **Colours that are data:** the value ramp of a heatmap encodes a number, as
  the ramp of the track view does in Race Analysis. When the window gets one
  (docs/plan.md, step 5), it lives in one module, and `test_look.py` names that
  module as the exception.
