"""The user manual: every page reachable, every link and every "?" mark alive.

The window's help marks open the manual at a heading (`data-help="etv-map#
zero-gas-fix"`). A renamed heading would leave a mark pointing at nothing, and
nothing in the window would show it — so it is checked here.
"""

import re
from pathlib import Path

import pytest

from etvlib import manual

MANUAL = manual.manual_dir()
ETV = Path(manual.__file__).parent / "etv"
PAGES = sorted(p.stem for p in MANUAL.glob("*.md"))
IMAGES = MANUAL / "images"

#: A link in Markdown: `[text](target)`.
_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
#: An image in Markdown: `![text](images/x.png)`.
_IMAGE = re.compile(r"!\[[^\]]*\]\(([^)]+)\)")


def _text(page: str) -> str:
    return (MANUAL / f"{page}.md").read_text(encoding="utf-8")


def _anchors(page: str) -> set[str]:
    """The ids Python-Markdown gives the headings of *page*."""
    return set(re.findall(r'<h[23] id="([^"]+)"', manual.render_page(page)))


def test_the_manual_is_found_and_has_its_pages():
    assert MANUAL is not None and len(PAGES) >= 8
    assert "README" in PAGES


def test_the_index_lists_every_page_and_only_pages_that_exist():
    listed = {target[:-3] for target in _LINK.findall(_text("README"))
              if target.endswith(".md") and not target.startswith("../")}
    assert listed == set(PAGES) - {"README"}


@pytest.mark.parametrize("page", PAGES)
def test_every_link_of_a_page_leads_somewhere(page):
    for target in _LINK.findall(_text(page)):
        if target.startswith(("http://", "https://", "images/")):
            continue
        name, _, anchor = target.partition("#")
        if name.startswith("../"):                                    # out of the manual, e.g. ../plan.md
            assert (MANUAL.parent / name[3:]).exists(), f"{page}: {target}"
            continue
        assert not name or name.endswith(".md"), f"{page}: {target}"
        where = name[:-3] if name else page
        assert where in PAGES, f"{page}: {target}"
        assert not anchor or anchor in _anchors(where), f"{page}: {target}"


@pytest.mark.parametrize("page", PAGES)
def test_the_pictures_of_a_page_are_there(page):
    for target in _IMAGE.findall(_text(page)):
        assert target.startswith("images/"), f"{page}: {target}"
        assert (MANUAL / target).is_file(), f"{page}: {target}"


def test_no_picture_lies_around_unused():
    used = {target[len("images/"):] for page in PAGES for target in _IMAGE.findall(_text(page))}
    there = {p.name for p in IMAGES.iterdir() if p.suffix != ".md"}
    assert there == used, f"unused: {sorted(there - used)}"


def test_every_help_mark_of_the_window_opens_a_page_that_exists():
    marks = {mark for src in ETV.glob("*.html")
             for mark in re.findall(r'data-help="([^"]+)"', src.read_text(encoding="utf-8"))}
    assert len(marks) >= 5, "no help marks in the window?"
    for mark in marks:
        page, _, anchor = mark.partition("#")
        assert page in PAGES, mark
        assert not anchor or anchor in _anchors(page), mark


def test_no_page_uses_fences_the_renderer_does_not_understand():
    # The app renders with Python-Markdown without `fenced_code`: a ``` block
    # would run together into one paragraph. Indent by four spaces instead.
    # A fence opens a line; the writing guide names one inline, which is fine.
    for page in PAGES:
        assert not re.search(r"^\s*```", _text(page), re.M), page


def test_a_rendered_page_links_as_the_app_serves_it():
    html = manual.render_page("etv-map")
    assert '<a class="back" href="/manual/">← Manual</a>' in html
    assert 'href="/manual/calculation#rpm-calc-method"' in html
    assert 'src="/manual/images/etv-map-result.png"' in html
    assert "<title>4 · ETV map — OpenETV</title>" in html
    assert 'href="/theme.css"' in html                                 # the manual is in the app's look
    # The index has no back link to itself.
    assert 'class="back"' not in manual.render_page("")


@pytest.mark.parametrize("page", ["nothing", "../etvlib/server", "images/x", "READ ME", ""])
def test_what_is_no_page_is_refused(page):
    if page == "":
        assert manual.render_page(page)                                # "" is the index
        return
    with pytest.raises(manual.ManualError):
        manual.render_page(page)


def test_an_image_is_read_by_name_only():
    data, kind = manual.image("etv-map-result.png")
    assert kind == "image/png" and data[:4] == b"\x89PNG"
    for wrong in ("../../LICENSE", "x.png", "etv-map-result.PNG.txt", ""):
        with pytest.raises(manual.ManualError):
            manual.image(wrong)
