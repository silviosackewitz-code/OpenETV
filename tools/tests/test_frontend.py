"""Tests for the window's JavaScript.

The page has no build step and no package.json; its pure modules — the ones
that touch neither the page nor the server — are tested with Node's built-in
test runner (`tools/tests/js/*.test.mjs`). This runs them from pytest. Skipped
when Node is not installed.
"""

import shutil
import subprocess
from pathlib import Path

import pytest

JS_TESTS = Path(__file__).with_name("js")
NODE = shutil.which("node")

pytestmark = pytest.mark.skipif(NODE is None, reason="Node.js is not installed")


@pytest.mark.parametrize("name", sorted(p.name for p in JS_TESTS.glob("*.test.mjs")))
def test_javascript(name):
    run = subprocess.run([NODE, "--test", str(JS_TESTS / name)], capture_output=True, text=True, timeout=120)
    assert run.returncode == 0, run.stdout + run.stderr
