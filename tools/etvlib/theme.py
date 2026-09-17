"""The part of the look the user chooses: light or dark, and the accent colour.

`theme.css` holds the look. Two things in it are the user's: which of its two
themes applies (`theme`: follow the system, light, dark) and the accent
(`accent`, any colour). Both are view preferences (`library.read_prefs`).

One colour is not enough for an accent: a filled button needs a text colour
that can be read on it, the accent as text has to be readable on the panel it
stands on — in the light theme and in the dark one — and a filled control has
to stand out from that panel at all. `accent_roles` derives these from the one
chosen colour, so yellow, white or black are usable choices too.

The derivation lives here and only here. The server writes its result into
every page it serves (`page_look`), so a window starts in the right colours
instead of flashing the default first, and hands it to the open windows with
the preferences (`api.get_prefs`). `theme.css` carries the roles of
`DEFAULT_ACCENT`; `test_theme.py` checks that they are what this module
derives.
"""
from __future__ import annotations

import colorsys
import json
import re

#: `auto` follows the system.
THEMES = ("auto", "light", "dark")
DEFAULT_THEME = "auto"

#: A bright Ducati red (maintainer, 2026-09-17).
DEFAULT_ACCENT = "#d40000"
#: Offered beside the free choice, the default first.
PRESETS = (("Red", DEFAULT_ACCENT), ("Orange", "#f26a00"), ("Yellow", "#e6b800"),
           ("Green", "#3fa535"), ("Blue", "#0a5fd1"), ("Graphite", "#4a5568"))

#: The surfaces of `theme.css` the accent has to work on. As text it stands on
#: the panel and on the window; the one with less contrast decides.
_SURFACES = {"light": ("#ffffff", "#eef0f4"), "dark": ("#161b23", "#0f1319")}
#: Text on a filled accent control: white, or the dark ink.
_ON_ACCENT = ("#ffffff", "#101317")
#: WCAG: 4.5 for text, 3 for the outline of a control against its surroundings.
_TEXT_CONTRAST, _FILL_CONTRAST = 4.5, 3.0
_SOFT_ALPHA = {"light": .14, "dark": .16}

_HEX = re.compile(r"#?([0-9a-fA-F]{6}|[0-9a-fA-F]{3})")


def parse_accent(value: object) -> str:
    """*value* as `#rrggbb` in lower case; `ValueError` when it is no colour."""
    match = _HEX.fullmatch(value.strip()) if isinstance(value, str) else None
    if not match:
        raise ValueError("the accent must be a colour like #d40000")
    digits = match[1].lower()
    if len(digits) == 3:
        digits = "".join(2 * d for d in digits)
    return "#" + digits


def _rgb(colour: str) -> tuple[float, float, float]:
    return tuple(int(colour[i:i + 2], 16) / 255 for i in (1, 3, 5))


def _hex(rgb: tuple[float, float, float]) -> str:
    return "#" + "".join(f"{round(max(0.0, min(1.0, c)) * 255):02x}" for c in rgb)


def _luminance(colour: str) -> float:
    def linear(c: float) -> float:
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = (linear(c) for c in _rgb(colour))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a: str, b: str) -> float:
    """The WCAG contrast ratio of two colours, 1 (none) to 21."""
    high, low = sorted((_luminance(a), _luminance(b)), reverse=True)
    return (high + 0.05) / (low + 0.05)


def _towards_contrast(colour: str, surfaces: tuple[str, ...], wanted: float, lighter: bool) -> str:
    """*colour*, made lighter or darker in small steps until it has the
    *wanted* contrast to every surface. Hue and saturation stay, so the result
    still reads as the chosen colour; black or white is the end of the road."""
    hue, lightness, saturation = colorsys.rgb_to_hls(*_rgb(colour))
    step = .01 if lighter else -.01
    result = colour
    while min(contrast(result, s) for s in surfaces) < wanted and 0 <= lightness + step <= 1:
        lightness = round(lightness + step, 4)
        result = _hex(colorsys.hls_to_rgb(hue, lightness, saturation))
    return result


def accent_roles(accent: str, theme: str) -> dict[str, str]:
    """The accent roles of `theme.css` for *accent* in the light or dark theme."""
    surfaces = _SURFACES[theme]
    lighter = theme == "dark"
    fill = _towards_contrast(accent, surfaces[:1], _FILL_CONTRAST, lighter)
    text = _towards_contrast(accent, surfaces, _TEXT_CONTRAST, lighter)
    r, g, b = (round(c * 255) for c in _rgb(text))
    return {"--accent": text,
            "--accent-fill": fill,
            "--on-accent": max(_ON_ACCENT, key=lambda ink: contrast(ink, fill)),
            "--accent-soft": f"rgba({r},{g},{b},{_SOFT_ALPHA[theme]})"}


def accent_css(accent: str) -> str:
    """The rules that put *accent* in place of the default one — empty for the
    default itself. Same selectors as the dark blocks of `theme.css`, and
    placed after it, so they win."""
    if accent == DEFAULT_ACCENT:
        return ""
    light, dark = (";".join(f"{k}:{v}" for k, v in accent_roles(accent, t).items())
                   for t in ("light", "dark"))
    return (f":root{{{light}}}"
            f'@media (prefers-color-scheme:dark){{:root:not([data-theme="light"]){{{dark}}}}}'
            f':root[data-theme="dark"]{{{dark}}}')


#: The element the accent rules live in; `theme.js` replaces its text when the
#: accent changes while the window is open.
ACCENT_STYLE_ID = "accentRoles"
_THEME_LINK = re.compile(r'<link[^>]+href="/theme\.css"[^>]*>\n?')
_DOCTYPE = re.compile(r"\s*<!doctype[^>]*>\s*", re.IGNORECASE)


def page_look(html: str, prefs: dict) -> str:
    """*html* with the chosen look written into it, so the page never shows
    another one first: `data-theme` is set before anything is painted, the
    accent rules follow the link to `theme.css`."""
    if prefs["theme"] != DEFAULT_THEME:
        start = _DOCTYPE.match(html)
        at = start.end() if start else 0
        script = f"<script>document.documentElement.dataset.theme={json.dumps(prefs['theme'])}</script>\n"
        html = html[:at] + script + html[at:]
    link = _THEME_LINK.search(html)
    if link:
        style = f'<style id="{ACCENT_STYLE_ID}">{accent_css(prefs["accent"])}</style>\n'
        html = html[:link.end()] + style + html[link.end():]
    return html
