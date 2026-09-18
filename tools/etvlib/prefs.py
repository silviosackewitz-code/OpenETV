"""The preferences of the window: which theme, which accent (`theme.py`).

Kept in `~/.openetv/prefs.json`. Not in the browser's `localStorage`: the
server takes a free port at every start, which makes every start a new origin.
The names, the checks and the answer a window gets are those of Race Analysis
(`racelib/library.py`, `api._prefs_answer`), so that the shared `prefs.js` and
`theme.js` work here unchanged.
"""

from __future__ import annotations

import json
import os
import tempfile
from pathlib import Path

from .theme import DEFAULT_ACCENT, DEFAULT_THEME, PRESETS, THEMES, accent_css, parse_accent

#: Where the preferences are kept: `~/.openetv`, or the folder `OPENETV_HOME`
#: names — for trying the app without touching one's own. A test puts another
#: file here.
PREFS_FILE = Path(os.environ.get("OPENETV_HOME") or Path.home() / ".openetv") / "prefs.json"

#: Every preference with its default. `help_marks`: the "?" marks at the
#: steps that open the manual there are shown (the **Help** switch).
PREFS = {"theme": DEFAULT_THEME, "accent": DEFAULT_ACCENT, "help_marks": True}


def _checked(key: str, value: object) -> object:
    """*value* as it is stored for *key*; `ValueError` when it does not fit."""
    if key == "theme":
        if value not in THEMES:
            raise ValueError(f"preference 'theme' must be one of {', '.join(THEMES)}")
        return value
    if key == "accent":
        return parse_accent(value)
    if not isinstance(value, bool):
        raise ValueError(f"preference {key!r} must be true or false")
    return value


def _stored() -> dict:
    try:
        stored = json.loads(PREFS_FILE.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    return stored if isinstance(stored, dict) else {}


def read_prefs() -> dict:
    """The preferences; a missing or wrong value reads as its default."""
    stored, prefs = _stored(), {}
    for key, default in PREFS.items():
        try:
            prefs[key] = _checked(key, stored[key])
        except (KeyError, ValueError):
            prefs[key] = default
    return prefs


def set_prefs(values: dict) -> dict:
    """Change some preferences and return all of them. Raises `ValueError` for
    an unknown name or a value that does not fit it; nothing is written then."""
    checked = {}
    for key, value in values.items():
        if key not in PREFS:
            raise ValueError(f"unknown preference {key!r}")
        checked[key] = _checked(key, value)
    if checked:
        PREFS_FILE.parent.mkdir(parents=True, exist_ok=True)
        # Written beside the file and moved over it: a crash in between leaves
        # the old preferences, not half a file.
        handle, temporary = tempfile.mkstemp(dir=PREFS_FILE.parent, suffix=".tmp")
        with os.fdopen(handle, "w", encoding="utf-8") as file:
            json.dump({**_stored(), **checked}, file, indent=2)
        os.replace(temporary, PREFS_FILE)
    return read_prefs()


def answer(prefs: dict) -> dict:
    """The preferences and what a window needs to show the look they name: the
    accent's rules, derived in one place (`theme.accent_css`), and the colours
    offered for it."""
    return {"ok": True, **prefs, "accent_css": accent_css(prefs["accent"]),
            "accent_presets": [{"name": name, "colour": colour} for name, colour in PRESETS]}
