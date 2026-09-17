"""The window: the page of the local server in a pywebview window.

Without pywebview — or with `force_browser` — the same page opens in the
default browser, so there is a window in every case. What needs the window
itself is its save dialog (`App.save_dialog`): a page inside pywebview cannot
hand a download to anybody.
"""

from __future__ import annotations

import logging
import threading
import webbrowser
from pathlib import Path

from .server import serve

_log = logging.getLogger("etvlib.ui")


class App:
    """The window of one run of the app."""

    def __init__(self) -> None:
        #: The pywebview window while it is open; `None` in a browser and in tests.
        self.window = None

    def save_dialog(self, name: str, kind: str) -> Path | None:
        """Ask where to save a file called *name*; *kind* describes it
        ("CSV (*.csv)"). `None` when the user cancelled. Raises `LookupError`
        when there is no window to ask with."""
        if self.window is None:
            raise LookupError("no window")
        import webview  # noqa: PLC0415 — only there when the window is

        chosen = self.window.create_file_dialog(webview.FileDialog.SAVE, save_filename=name, file_types=(kind,))
        if not chosen:
            return None
        # A path, or a sequence holding one, depending on the platform.
        return Path(chosen if isinstance(chosen, str) else chosen[0])


app = App()


def show(port: int = 0, force_browser: bool = False) -> None:
    """Open the window and serve it until it is closed."""
    server, url = serve(port)
    print(f"OpenETV — {url}")

    webview = None
    if not force_browser:
        try:
            import webview  # noqa: PLC0415 — optional: without it the browser is the window
        except ImportError:
            _log.info("pywebview is not installed; opening the browser instead")

    if webview is not None:
        # text_select: a table of numbers is there to be selected and copied.
        app.window = webview.create_window("OpenETV", url, width=1240, height=820, min_size=(720, 520),
                                           text_select=True)
        webview.start()
        app.window = None
    else:
        webbrowser.open(url)
        try:
            threading.Event().wait()
        except KeyboardInterrupt:
            pass
    server.shutdown()
