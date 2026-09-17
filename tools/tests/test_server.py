"""The server: what it serves, and to whom."""

import http.client
import json
import re

import pytest

from etvlib import __version__, server


@pytest.fixture(scope="module")
def running():
    httpd, url = server.serve()
    yield httpd.server_address[1]
    httpd.shutdown()


def ask(port, method, path, body=None, token=True, host=None, headers=None):
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
    send = {"Host": host or f"127.0.0.1:{port}", **(headers or {})}
    if token:
        send[server.TOKEN_HEADER] = server.TOKEN
    data = None if body is None else (body if isinstance(body, bytes) else json.dumps(body).encode())
    connection.request(method, path, body=data, headers=send)
    answer = connection.getresponse()
    content = answer.read()
    connection.close()
    return answer.status, answer.getheader("Content-Type") or "", content


def test_the_page_carries_the_key_and_sends_it_to_this_server_only(running):
    status, kind, page = ask(running, "GET", "/", token=False)
    assert status == 200 and kind.startswith("text/html")
    text = page.decode()
    assert re.match(r"<!doctype html>\s*<script>", text)               # before anything else of the page
    assert json.dumps(server.TOKEN) in text and server.TOKEN_HEADER in text
    assert "url.origin!==location.origin" in text


def test_the_pages_modules_are_served_and_nothing_else_from_the_disk(running):
    status, kind, _ = ask(running, "GET", "/etv/main.js", token=False)
    assert status == 200 and kind.startswith("text/javascript")
    for path in ("/etv/../server.py", "/etv/..%2Fserver.py", "/etv/etv.html", "/etv/sub/main.js",
                 "/etv/missing.js", "/server.py", "/indexX", "/etvX"):
        assert ask(running, "GET", path)[0] == 404, path


def test_data_only_with_the_key(running):
    assert ask(running, "GET", "/api/about", token=False)[0] == 403
    assert ask(running, "POST", "/api/calculate", {}, token=False)[0] == 403
    status, _, content = ask(running, "GET", "/api/about")
    assert status == 200 and json.loads(content)["version"] == __version__


def test_only_requests_to_this_server_itself(running):
    # DNS rebinding: a foreign name that resolves to 127.0.0.1.
    for path in ("/", "/api/about"):
        assert ask(running, "GET", path, host=f"evil.example:{running}")[0] == 403
    assert ask(running, "POST", "/api/calculate", {}, host="evil.example")[0] == 403
    assert ask(running, "GET", "/", host=f"localhost:{running}")[0] == 200


def test_a_calculation_over_http(running):
    sample = json.loads(ask(running, "GET", "/api/sample")[2])
    status, kind, content = ask(running, "POST", "/api/calculate",
                                {"engine": sample["engine"], "request": sample["request"], "rpm_threshold": 7000})
    answer = json.loads(content)
    assert status == 200 and kind == "application/json" and answer["ok"]
    assert answer["table"]["values"][0][:3] == [4.1, 5.0, 6.7]


def test_a_post_always_answers_json_the_page_can_show(running):
    for path, body, says in (("/api/calculate", {}, "The engine torque table is missing."),
                             ("/api/nothing", {}, "unknown endpoint nothing"),
                             ("/api/calculate", b"[1, 2]", "The request is not a JSON object."),
                             ("/api/calculate", b"{not json", "The request is not JSON.")):
        status, _, content = ask(running, "POST", path, body)
        assert status == 200 and json.loads(content) == {"ok": False, "error": says}, path
    assert ask(running, "POST", "/somewhere", {})[0] == 404
    assert ask(running, "GET", "/api/nothing")[0] == 404


def test_a_body_is_not_read_beyond_its_limit(running):
    too_long = {"Content-Length": str(server._Handler.MAX_BODY + 1)}
    assert ask(running, "POST", "/api/calculate", b"{}", headers=too_long)[0] == 413
    assert ask(running, "POST", "/api/calculate", b"{}", headers={"Content-Length": "-5"})[0] == 400


def test_numbers_that_json_does_not_know_become_null():
    assert server._finite({"a": [1.0, float("nan"), (float("inf"),)]}) == {"a": [1.0, None, [None]]}
