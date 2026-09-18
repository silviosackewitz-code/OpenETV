"""Takes the pictures of the manual from the real window.

    .venv/bin/python tools/screenshots.py            # all of them
    .venv/bin/python tools/screenshots.py etv-map    # only these

Every picture is taken the same way — WebKit (the engine the window uses on
macOS), the light theme, the **?** marks off, two device pixels per CSS pixel
— so the manual does not end up with a patchwork of screenshots taken by hand
at different sizes and in different themes. A new picture is one more function
here, marked `@shot`.

The preferences of the run go to a folder of their own, so nobody's own
`~/.openetv` is touched.
"""

from __future__ import annotations

import os
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IMAGES = ROOT / "docs" / "manual" / "images"

#: The window in the pictures: width and height in CSS pixels, and the scale.
#: Two device pixels per CSS pixel, as on the screens the app is used on.
WINDOW = (1280, 820, 2)

#: name → what it does, filled by `@shot`.
SHOTS: dict[str, Callable] = {}


def shot(name: str):
    def register(function: Callable) -> Callable:
        SHOTS[name] = function
        return function
    return register


def take(page, name: str, part=None) -> None:
    """*part* (a locator) or the whole window, as `images/<name>.png`."""
    IMAGES.mkdir(parents=True, exist_ok=True)
    (part or page).screenshot(path=str(IMAGES / f"{name}.png"))
    print(f"  {name}.png")


def step(page, name: str):
    page.locator(f'#steps [data-step="{name}"]').click()
    return page.locator(f'main > section[data-step="{name}"]')


def calculate(page) -> None:
    step(page, "calculation")
    page.get_by_role("button", name="Calculate ETV map").click()
    page.wait_for_selector("#resultGrid td")


@shot("getting-started-window")
def _window(page):
    step(page, "engine")
    take(page, "getting-started-window")


@shot("engine-torque-table")
def _engine(page):
    section = step(page, "engine")
    take(page, "engine-torque-table", section)


@shot("torque-request-generator")
def _request(page):
    section = step(page, "request")
    section.locator("#generator > summary").click()
    page.locator("#genShape").select_option("corner_exit")
    # The curve is drawn from the server's numbers: wait for it.
    page.wait_for_function("""() => { const c = document.getElementById('genCurve');
        const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data; let lit = 0;
        for (let i = 0; i < d.length; i += 4) if (d[i] + d[i + 1] + d[i + 2] > 380) lit++; return lit > 500; }""")
    take(page, "torque-request-generator", section.locator("#generator"))


@shot("calculation-settings")
def _calculation(page):
    section = step(page, "calculation")
    take(page, "calculation-settings", section)


@shot("etv-map-result")
def _result(page):
    calculate(page)
    page.get_by_role("button", name="Zero-gas fix", exact=True).click()
    page.wait_for_selector(".notice")
    page.locator(".notice .btn").click()                     # the notice would cover the map
    take(page, "etv-map-result", step(page, "result"))


@shot("export-formats")
def _export(page):
    calculate(page)
    take(page, "export-formats", step(page, "export"))


@shot("appearance-dialog")
def _appearance(page):
    step(page, "engine")
    page.get_by_role("button", name="Appearance").click()
    page.wait_for_selector("#look[open]")
    take(page, "appearance-dialog", page.locator("#look"))
    page.locator("#look").get_by_role("button", name="Close").click()


def main(argv: list[str]) -> int:
    sys.path.insert(0, str(ROOT / "tools"))
    wanted = argv or sorted(SHOTS)
    unknown = [name for name in wanted if name not in SHOTS]
    if unknown:
        print(f"No such picture: {', '.join(unknown)}\nThere are: {', '.join(sorted(SHOTS))}")
        return 2

    with tempfile.TemporaryDirectory() as home:
        os.environ["OPENETV_HOME"] = home                    # not the user's own preferences
        from playwright.sync_api import sync_playwright  # noqa: PLC0415 — a development dependency

        from etvlib import prefs, server  # noqa: PLC0415 — after OPENETV_HOME is set

        prefs.PREFS_FILE = Path(home) / "prefs.json"
        prefs.set_prefs({"theme": "light", "accent": prefs.DEFAULT_ACCENT, "help_marks": False})
        httpd, url = server.serve()
        width, height, scale = WINDOW
        with sync_playwright() as p:
            browser = p.webkit.launch()
            context = browser.new_context(viewport={"width": width, "height": height},
                                          device_scale_factor=scale)
            page = context.new_page()
            page.goto(url)
            page.wait_for_selector('section[data-step="engine"] .grid td')
            for name in wanted:
                SHOTS[name](page)
                # Every picture starts from a window as it was opened.
                page.reload()
                page.wait_for_selector('section[data-step="engine"] .grid td')
            context.close()
            browser.close()
        httpd.shutdown()
    print(f"{len(wanted)} picture(s) in {IMAGES.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
