"""The user manual inside the app.

The manual is Markdown in `docs/manual` — readable on GitHub and in any
editor. The app shows the same files as pages of its own: rendered here with
Python-Markdown, served at `/manual/<page>` by the local server. It works
offline and always matches the installed version, because the built app
carries the files (`packaging/openetv.spec`).

Built like the manual of Race Analysis (`racelib/manual.py`), without its
folder of file-format specifications.
"""

from __future__ import annotations

import html
import re
import sys
from pathlib import Path

import markdown

#: Where the manual is: next to the package in the built app, in docs/ when
#: running from the repository.
_CANDIDATES = (
    Path(__file__).resolve().parent.parent / "manual",
    Path(__file__).resolve().parents[2] / "docs" / "manual",
)
if getattr(sys, "_MEIPASS", None):
    _CANDIDATES = (Path(sys._MEIPASS) / "manual", *_CANDIDATES)

#: Page names: file names without `.md`, no folders.
PAGE_NAME = re.compile(r"[A-Za-z0-9_-]+")
IMAGE_NAME = re.compile(r"[A-Za-z0-9_.-]+\.(png|jpg|jpeg|gif|svg|webp)")
IMAGE_TYPES = {"png": "image/png", "jpg": "image/jpeg", "jpeg": "image/jpeg", "gif": "image/gif",
               "svg": "image/svg+xml", "webp": "image/webp"}

#: Python-Markdown extensions, loaded by name (the packaging spec lists them).
#: `toc` gives every heading an id, as GitHub does (`## Quick start` →
#: `id="quick-start"`): a help mark opens a page at the heading of its step
#: (`/manual/etv-map#fix-flat-spots`).
EXTENSIONS = ["tables", "sane_lists", "toc"]


class ManualError(LookupError):
    """No such page or image."""


def manual_dir() -> Path | None:
    return next((p for p in _CANDIDATES if (p / "README.md").is_file()), None)


def _source(page: str) -> Path:
    folder = manual_dir()
    if folder is None:
        raise ManualError("The manual is not installed")
    name = page or "README"
    if not PAGE_NAME.fullmatch(name):
        raise ManualError(f"No manual page {page!r}")
    path = folder / f"{name}.md"
    if not path.is_file():
        raise ManualError(f"No manual page {page!r}")
    return path


def _app_links(body: str) -> str:
    """Links and images as the app serves them.

    In the files a page links `other-page.md` and images are `images/x.png` —
    right on GitHub. In the app those are `/manual/other-page` and
    `/manual/images/x.png`."""
    body = re.sub(r'href="README\.md(#[^"]*)?"', lambda m: f'href="/manual/{m[1] or ""}"', body)
    body = re.sub(r'href="([A-Za-z0-9_-]+)\.md(#[^"]*)?"',
                  lambda m: f'href="/manual/{m[1]}{m[2] or ""}"', body)
    return re.sub(r'src="images/', 'src="/manual/images/', body)


def render_page(page: str) -> str:
    """A manual page as a complete HTML document."""
    body = _app_links(markdown.markdown(_source(page).read_text(encoding="utf-8"), extensions=EXTENSIONS))
    title = re.search(r"<h1[^>]*>(.*?)</h1>", body)
    title = re.sub(r"<[^>]+>", "", title[1]) if title else "Manual"
    index = "" if not page or page == "README" else '<a class="back" href="/manual/">← Manual</a>'
    return PAGE.format(title=html.escape(title), index=index, body=body)


def image(name: str) -> tuple[bytes, str]:
    """An image of the manual: its bytes and media type."""
    folder = manual_dir()
    match = IMAGE_NAME.fullmatch(name or "")
    path = folder / "images" / name if folder and match else None
    if path is None or not path.is_file():
        raise ManualError(f"No manual image {name!r}")
    return path.read_bytes(), IMAGE_TYPES[match[1].lower()]


PAGE = """<!doctype html>
<meta charset="utf-8">
<title>{title} — OpenETV</title>
<!-- IBM Plex, shipped with the app (THIRD-PARTY-NOTICES.md) — no network call. -->
<link rel="stylesheet" href="/fonts.css">
<link rel="stylesheet" href="/theme.css">
<style>
html,body{{height:auto}}
body{{font:15px/1.6 var(--sans);background:var(--bg)}}
main{{max-width:820px;margin:0 auto;padding:28px 28px 60px;background:var(--panel);min-height:100vh;
  border-left:1px solid var(--line);border-right:1px solid var(--line)}}
.back{{display:inline-block;margin-bottom:14px;font-size:13px}}
h1{{font-size:28px;line-height:1.2;margin:0 0 14px}}
h2{{font-size:20px;margin:34px 0 8px;padding-top:12px;border-top:1px solid var(--line)}}
h3{{font-size:16px;margin:24px 0 6px}}
a{{color:var(--accent)}}
code{{font-family:var(--mono);font-size:.88em;background:var(--code);padding:1px 5px;border-radius:4px}}
pre{{background:var(--code);padding:12px;border-radius:6px;overflow:auto}}
pre code{{background:none;padding:0}}
img{{max-width:100%;height:auto;border:1px solid var(--line);border-radius:6px;margin:6px 0}}
li{{margin:3px 0}}
table{{border-collapse:collapse}} td,th{{border:1px solid var(--line);padding:4px 8px}}
</style>
<main>
{index}
{body}
</main>
"""
