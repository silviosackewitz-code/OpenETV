"""Make the app's icons from `app-icon.svg`.

    .venv/bin/python packaging/icon/make_icon.py

Writes `app-icon.png` (1024 × 1024, transparent outside the rounded square),
`app-icon.ico` (Windows) and, on macOS, `app-icon.icns`. All three are kept in
git, so a build needs none of this. The SVG is rendered by WebKit through
Playwright — a development dependency already, and nothing else here can draw
an SVG. An .ico may hold PNG images as they are (Windows Vista and later), so
it is written by hand: a header, one directory entry per size, the PNGs.
"""

from __future__ import annotations

import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
SVG = HERE / "app-icon.svg"
ICO_SIZES = (16, 32, 48, 64, 128, 256)


def render(page, size: int) -> bytes:
    page.set_viewport_size({"width": size, "height": size})
    page.set_content(f'<body style="margin:0;background:transparent">'
                     f'<img src="data:image/svg+xml;base64,{_b64(SVG.read_bytes())}" '
                     f'width="{size}" height="{size}" style="display:block"></body>')
    page.wait_for_function("document.images[0].complete && document.images[0].naturalWidth > 0")
    return page.screenshot(omit_background=True, clip={"x": 0, "y": 0, "width": size, "height": size})


def _b64(data: bytes) -> str:
    import base64  # noqa: PLC0415
    return base64.b64encode(data).decode("ascii")


def ico(images: dict[int, bytes]) -> bytes:
    header = struct.pack("<HHH", 0, 1, len(images))
    entries, blobs, offset = b"", b"", 6 + 16 * len(images)
    for size, png in images.items():
        # Width and height as one byte each, 0 standing for 256.
        entries += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(png), offset)
        blobs += png
        offset += len(png)
    return header + entries + blobs


def main() -> int:
    from playwright.sync_api import sync_playwright  # noqa: PLC0415 — a development dependency

    with sync_playwright() as p:
        browser = p.webkit.launch()
        page = browser.new_page(device_scale_factor=1)
        (HERE / "app-icon.png").write_bytes(render(page, 1024))
        (HERE / "app-icon.ico").write_bytes(ico({size: render(page, size) for size in ICO_SIZES}))
        browser.close()

    if sys.platform == "darwin" and shutil.which("iconutil"):
        with tempfile.TemporaryDirectory() as tmp:
            iconset = Path(tmp) / "AppIcon.iconset"
            iconset.mkdir()
            for size in (16, 32, 128, 256, 512):
                for scale, suffix in ((1, ""), (2, "@2x")):
                    subprocess.run(["sips", "-z", str(size * scale), str(size * scale), str(HERE / "app-icon.png"),
                                    "--out", str(iconset / f"icon_{size}x{size}{suffix}.png")],
                                   check=True, capture_output=True)
            subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(HERE / "app-icon.icns")], check=True)
    print("written:", ", ".join(sorted(p.name for p in HERE.glob("app-icon.*"))))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
