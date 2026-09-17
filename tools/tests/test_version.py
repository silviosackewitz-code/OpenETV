"""The version is written once, in `etvlib/__init__.py`."""

import re
from pathlib import Path

import etvlib

ROOT = Path(__file__).resolve().parents[2]


def test_the_build_spec_carries_the_package_version():
    spec = (ROOT / "OpenETV.spec").read_text(encoding="utf-8")
    assert re.search(r'"CFBundleShortVersionString": "([^"]+)"', spec)[1] == etvlib.__version__
