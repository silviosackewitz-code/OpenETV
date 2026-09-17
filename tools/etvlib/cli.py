"""Starting the app from a terminal: `python -m etvlib` (with `tools` on the path)."""

from __future__ import annotations

import argparse
import threading

from . import __version__
from .server import serve
from .ui import show


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="openetv", description="OpenETV — throttle position map generator")
    parser.add_argument("--version", action="version", version=f"OpenETV {__version__}")
    parser.add_argument("--port", type=int, default=0, help="port of the local server (default: a free one)")
    how = parser.add_mutually_exclusive_group()
    how.add_argument("--browser", action="store_true", help="open the page in the browser instead of a window")
    how.add_argument("--no-browser", action="store_true", help="start the server only, open nothing")
    args = parser.parse_args(argv)

    if not args.no_browser:
        show(args.port, force_browser=args.browser)
        return 0
    server, url = serve(args.port)
    print(f"OpenETV {__version__} — {url}")
    try:
        threading.Event().wait()
    except KeyboardInterrupt:
        pass
    server.shutdown()
    return 0
