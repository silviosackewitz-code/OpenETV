"""Starting the app from a terminal: `python -m etvlib` (with `tools` on the path)."""

from __future__ import annotations

import argparse
import threading
import webbrowser

from . import __version__
from .server import serve


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="openetv", description="OpenETV — throttle position map generator")
    parser.add_argument("--version", action="version", version=f"OpenETV {__version__}")
    parser.add_argument("--port", type=int, default=0, help="port of the local server (default: a free one)")
    parser.add_argument("--no-browser", action="store_true", help="start the server only, open nothing")
    args = parser.parse_args(argv)

    server, url = serve(args.port)
    print(f"OpenETV {__version__} — {url}")
    if not args.no_browser:
        webbrowser.open(url)
    try:
        threading.Event().wait()
    except KeyboardInterrupt:
        pass
    server.shutdown()
    return 0
