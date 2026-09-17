"""The build: what the spec packs is what the app needs.

A file the server serves but the spec leaves out is a built app whose window
stays empty — and nothing in the source tree shows it. The build itself is
tried by hand (`bash packaging/build.sh`); this keeps its plan in line.
"""

import re
from pathlib import Path

import etvlib
from etvlib import server

ROOT = Path(__file__).resolve().parents[2]
SPEC = (ROOT / "packaging" / "openetv.spec").read_text(encoding="utf-8")


def _packed() -> set[str]:
    """What the spec's `pages` take along, as paths below `etvlib/` — by the
    spec's own patterns: a tuple of them (`for pattern in ("*.js", …)`) or the
    one in its `.glob("…")`."""
    lib, found = ROOT / "tools" / "etvlib", set()
    lines = re.findall(r'\(str\(p\), "etvlib(/?[\w/]*)"\) for (?:pattern in \(([^)]*)\) for )?p in [^\n]*?'
                       r'\.glob\((?:pattern|"([^"]+)")\)', SPEC)
    assert len(lines) == 4, "the way packaging/openetv.spec lists its pages changed; read it again here"
    for folder, several, one in lines:
        folder = folder.strip("/")
        source = ROOT / "sample_data" if folder == "sample_data" else lib / folder
        for pattern in re.findall(r'"([^"]+)"', several) or [one]:
            found.update(f"{folder}/{p.name}".lstrip("/") for p in source.glob(pattern) if p.is_file())
    return found


def test_the_version_is_written_once_and_the_spec_reads_it():
    assert """re.search(r'__version__ = "([^"]+)"',""" in SPEC             # it reads the line …
    assert etvlib.__version__ not in SPEC                                  # … and does not repeat it
    assert re.fullmatch(r"\d+\.\d+\.\d+(-[\w.]+)?", etvlib.__version__)
    for name in ("build.sh", "build.ps1"):
        assert etvlib.__version__ not in (ROOT / "packaging" / name).read_text(encoding="utf-8")


def test_everything_the_server_serves_is_packed():
    packed = _packed()
    wanted = {path.name for path, _ in server.SHARED.values()}
    wanted |= {f"etv/{p.name}" for p in server.ETV_DIR.iterdir() if p.suffix in (".html", ".css", ".js")}
    wanted |= {f"fonts/{p.name}" for p in server.FONTS_DIR.iterdir() if p.is_file()}
    wanted |= {"sample_data/engine_torque_map.csv", "sample_data/demand_map.csv"}
    assert len(wanted) > 30
    assert not wanted - packed, f"served, but not in packaging/openetv.spec: {sorted(wanted - packed)}"


def test_the_icons_and_the_licences_are_there():
    for name in ("app-icon.svg", "app-icon.png", "app-icon.icns", "app-icon.ico"):
        assert (ROOT / "packaging" / "icon" / name).stat().st_size > 1000, name
    assert (ROOT / "packaging" / "icon" / "app-icon.ico").read_bytes()[:4] == b"\x00\x00\x01\x00"
    assert '"LICENSE", "THIRD-PARTY-NOTICES.md"' in SPEC


def test_the_build_leaves_out_what_the_app_does_not_use():
    excluded = re.search(r"excludes=\[(.*?)\]", SPEC, re.S)[1]
    for package in ("pandas", "pyarrow", "matplotlib", "playwright", "pytest"):
        assert f'"{package}"' in excluded
    assert "streamlit" not in SPEC.lower()
