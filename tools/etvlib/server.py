"""The local HTTP server behind the window.

It serves the page and its modules, checks host and token of every request,
and hands `/api/…` to `api.py`. It only listens on 127.0.0.1 and keeps nothing
between requests. Built like the server of Race Analysis (`racelib/server.py`),
so that OpenETV could become one of its windows.
"""

from __future__ import annotations

import json
import logging
import math
import re
import secrets
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

from . import api

#: The window's page and its modules (one ES module per area).
ETV_DIR = Path(__file__).with_name("etv")
PAGE = ETV_DIR / "etv.html"

#: Files that may be served from a folder, by extension.
ASSET_TYPES = {".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8",
               ".woff2": "font/woff2"}
_ASSET_NAME = re.compile(r"[A-Za-z0-9_-]+\.(?:" + "|".join(t[1:] for t in ASSET_TYPES) + ")")


def _pages() -> dict[str, Path]:
    """HTML pages, matched exactly against the path. A function, so that a
    test can put another page in `PAGE`."""
    return {"/": PAGE, "/index": PAGE}


_log = logging.getLogger("etvlib.server")

#: The key of this run of the program. The server listens on this computer
#: only, but any web page in a browser can send requests to 127.0.0.1. The
#: app's own page gets the key embedded and sends it as a header. A foreign
#: page cannot read it, and cannot set a header without the server's consent
#: (CORS).
TOKEN = secrets.token_urlsafe(32)
TOKEN_HEADER = "X-OpenETV-Token"


class _Handler(BaseHTTPRequestHandler):
    #: Largest request body accepted. The biggest real one is an Excel file in
    #: base64 — far below this.
    MAX_BODY = 16 * 2**20

    def log_message(self, *a):        # no access lines in the terminal
        pass

    def handle(self):
        # A window closed while its answer was on the way is no error of the app.
        try:
            super().handle()
        except (BrokenPipeError, ConnectionResetError):
            pass

    def _send(self, body: bytes, kind: str, code: int = 200):
        self.send_response(code)
        self.send_header("Content-Type", kind)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _json(self, data, code: int = 200):
        # NaN and Inf are not JSON — JSON.parse in the window would drop the whole answer.
        self._send(json.dumps(_finite(data), ensure_ascii=False).encode(), "application/json", code)

    def _error(self, code: int, message: str):
        self._json({"ok": False, "error": message}, code)

    def _asset(self, folder: Path, name: str):
        """A script or stylesheet from *folder* — and only from there: plain
        file names with a known extension, no subfolders, no `..`."""
        if not _ASSET_NAME.fullmatch(name):
            return self.send_error(404)
        path = folder / name
        if not path.is_file():
            return self.send_error(404)
        self._send(path.read_bytes(), ASSET_TYPES[path.suffix])

    def _host_ok(self) -> bool:
        """Only requests to exactly this server — protects against DNS rebinding."""
        host = (self.headers.get("Host") or "").lower()
        port = self.server.server_address[1]
        return host in (f"127.0.0.1:{port}", f"localhost:{port}")

    def _token_ok(self) -> bool:
        return secrets.compare_digest(self.headers.get(TOKEN_HEADER) or "", TOKEN)

    def do_GET(self):
        if not self._host_ok():
            return self._error(403, "forbidden host")
        parts = urlparse(self.path)
        path = parts.path
        # The page and its scripts are free — data only with the key.
        if path.startswith("/api/"):
            if not self._token_ok():
                return self._error(403, "missing or wrong token")
            route = api.GET.get(path[len("/api/"):])
            if route is None:
                return self.send_error(404)
            return self._answer(route, {k: v[0] for k, v in parse_qs(parts.query).items()}, error_code=400)
        pages = _pages()
        if path in pages:
            return self._send(_with_token(pages[path].read_text(encoding="utf-8")).encode(),
                              "text/html; charset=utf-8")
        if path.startswith("/etv/"):
            return self._asset(ETV_DIR, path[len("/etv/"):])
        self.send_error(404)

    def do_POST(self):
        if not self._host_ok():
            return self._error(403, "forbidden host")
        if not self._token_ok():
            return self._error(403, "missing or wrong token")
        try:
            length = int(self.headers.get("Content-Length") or 0)
        except ValueError:
            return self._error(400, "bad Content-Length")
        # Checked before reading: a negative length would read until the
        # connection closes, and nothing else limits what is read into memory.
        if length < 0:
            return self._error(400, "bad Content-Length")
        if length > self.MAX_BODY:
            return self._error(413, f"request body too large ({length} bytes)")
        path = urlparse(self.path).path
        if not path.startswith("/api/"):
            return self.send_error(404)
        try:
            body = json.loads(self.rfile.read(length) or b"{}")
        except ValueError:
            return self._json({"ok": False, "error": "The request is not JSON."})
        if not isinstance(body, dict):
            return self._json({"ok": False, "error": "The request is not a JSON object."})
        # Every `/api/` POST error, "no such endpoint" included, answers 200
        # with `{"ok": false}`: the page always reads a JSON body, never an
        # HTTP error page.
        route = api.POST.get(path[len("/api/"):])
        if route is None:
            return self._json({"ok": False, "error": f"unknown endpoint {path[len('/api/'):]}"})
        self._answer(route, body, error_code=200)

    def _answer(self, route, request: dict, error_code: int):
        try:
            self._json(route(request))
        except ValueError as error:             # what the user got wrong, as a sentence
            self._json({"ok": False, "error": str(error)}, error_code)
        except Exception as error:              # noqa: BLE001 — the window shall be able to show it
            _log.exception("%s %s failed", self.command, self.path)
            self._json({"ok": False, "error": str(error) or type(error).__name__},
                       500 if error_code != 200 else 200)


def _finite(value):
    """NaN and ±Inf replaced by `null`, recursively."""
    if isinstance(value, float):
        return value if math.isfinite(value) else None
    if isinstance(value, dict):
        return {k: _finite(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [_finite(v) for v in value]
    return value


def _with_token(html: str) -> str:
    """Put the key into the page and let `fetch` send it along — to this
    server only; a request to another origin does not need it."""
    token, header = json.dumps(TOKEN), json.dumps(TOKEN_HEADER)
    script = (f"<script>(()=>{{const T={token},H={header},f=window.fetch.bind(window);"
              "window.fetch=(u,o={})=>{const url=new URL(u instanceof Request?u.url:u,location.href);"
              "if(url.origin!==location.origin)return f(u,o);"
              "const h=new Headers(o.headers||(u instanceof Request?u.headers:undefined));"
              "h.set(H,T);return f(u,{...o,headers:h});};})();</script>\n")
    head = re.match(r"\s*<!doctype[^>]*>\s*", html, re.IGNORECASE)
    at = head.end() if head else 0
    return html[:at] + script + html[at:]


def serve(port: int = 0) -> tuple[ThreadingHTTPServer, str]:
    """Start the server in a thread of its own; returns it and its address.
    Port 0 takes a free one."""
    server = ThreadingHTTPServer(("127.0.0.1", port), _Handler)
    server.daemon_threads = True
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, f"http://127.0.0.1:{server.server_address[1]}/"
