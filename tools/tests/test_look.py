"""The look is the one of Race Analysis: its files, unchanged, and no colour
anywhere else.

`tools/sync_look.py` copies them and writes `look.json`. A copy edited here
would be overwritten by the next sync, and the two apps would drift apart
before that — so this fails when a copy no longer matches. The other tests
are the ones Race Analysis keeps its own pages in line with
(`tools/tests/test_theme.py` there): no source but the theme names a colour,
every role a source uses exists, every icon a page names exists. What the
theme itself promises — contrast, the derived accent — is tested where it is
written.
"""

import hashlib
import json
import re
from pathlib import Path

from etvlib import server

LIB = Path(server.__file__).parent
MANIFEST = json.loads((LIB / "look.json").read_text(encoding="utf-8"))
THEME = LIB / "theme.css"

#: Sources that carry **data** colours, not the look: the value ramp of the
#: heatmaps encodes a number, as the ramp of the track view does in Race
#: Analysis. It is drawn on the scope, which is dark in every theme. Add to
#: this list only for data.
DATA_COLOURS = {"etv/ramp.js"}

_COLOUR = re.compile(r"#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3,4})(?![\w-])"
                     r"|\b(?:rgba?|hsla?)\(")


def _own_sources() -> list[Path]:
    """The pages, stylesheets and modules written here — not the copies."""
    found = [p for pattern in ("*.html", "*.css", "*.js") for p in LIB.rglob(pattern)]
    return sorted(p for p in found if p.relative_to(LIB).as_posix() not in MANIFEST["files"])


def test_every_copied_file_is_as_it_was_fetched():
    changed = [name for name, digest in MANIFEST["files"].items()
               if hashlib.sha256((LIB / name).read_bytes()).hexdigest() != digest]
    assert not changed, "change these in Race Analysis and run tools/sync_look.py: " + ", ".join(changed)
    assert re.fullmatch(r"[0-9a-f]{40}", MANIFEST["commit"])


def test_no_source_but_the_theme_names_a_colour():
    ids = {i for page in LIB.rglob("*.html") for i in re.findall(r'\bid="([^"]+)"', page.read_text(encoding="utf-8"))}
    found = [f"{src.relative_to(LIB)}:{number}: {match[0]}"
             for src in _own_sources() if src.relative_to(LIB).as_posix() not in DATA_COLOURS
             for number, line in enumerate(src.read_text(encoding="utf-8").splitlines(), 1)
             for match in _COLOUR.finditer(line)
             if not (match[0].startswith("#") and match[0][1:] in ids)]   # `$("#cAdd")` is an id, not a colour
    assert _own_sources() and not found, "use a role of theme.css instead:\n" + "\n".join(found)


def test_every_role_a_source_uses_exists():
    roles = set(re.findall(r"(--[\w-]+)\s*:", THEME.read_text(encoding="utf-8")))
    texts = {src: src.read_text(encoding="utf-8") for src in _own_sources()}
    # A module may set a custom property of its own for one element; those are known too.
    own = {name for text in texts.values()
           for pair in re.findall(r"""(--[\w-]+)\s*:|setProperty\(["'](--[\w-]+)""", text) for name in pair if name}
    missing = [f"{src.relative_to(LIB)}: {name}" for src, text in texts.items()
               for name in sorted(set(re.findall(r"""(?:var|getPropertyValue|themeColour)\(["']?(--[\w-]+)""", text))
                                  - roles - own)]
    assert not missing, "not defined in theme.css:\n" + "\n".join(missing)


def test_every_icon_a_page_names_exists():
    known = set(re.findall(r"^  (\w+): '", (LIB / "icons.js").read_text(encoding="utf-8"), re.M))
    assert len(known) >= 10
    used = {(src.relative_to(LIB).as_posix(), name) for src in _own_sources()
            for names in re.findall(r"""data-icon="([^"]+)"|\bicon\(["'](\w+)["']""", src.read_text(encoding="utf-8"))
            for name in names if name}
    assert not [u for u in used if u[1] not in known]
