# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller build spec for OpenETV.

Build with:  pyinstaller OpenETV.spec
Output:      dist/OpenETV.app (macOS) or dist/OpenETV/ (Windows/Linux)

Streamlit apps can't be frozen as a plain script: `streamlit run app.py` is a
CLI command, not something PyInstaller's import analysis can follow. Instead
we freeze run_app.py, which starts Streamlit's CLI programmatically and
points it at a bundled copy of app.py (see run_app.py for details).

We use onedir + collect_all(...) rather than --onefile: Streamlit ships a
sizeable static web frontend as package data, which --onefile would have to
re-extract to a temp directory on every single launch. onedir keeps startup
fast; on macOS the BUNDLE() step still wraps everything into a single
double-clickable OpenETV.app.
"""
import sys

from PyInstaller.utils.hooks import collect_all

datas = [
    ("app.py", "."),
    ("dss.py", "."),
    ("sample_data", "sample_data"),
]
binaries = []
hiddenimports = []

for pkg in ("streamlit", "altair", "pyarrow"):
    pkg_datas, pkg_binaries, pkg_hiddenimports = collect_all(pkg)
    datas += pkg_datas
    binaries += pkg_binaries
    hiddenimports += pkg_hiddenimports

# collect_all() also pulls in pyarrow's (large) internal test suite as hidden
# imports; it isn't needed at runtime.
hiddenimports = [h for h in hiddenimports if not h.startswith("pyarrow.tests")]
datas = [d for d in datas if "pyarrow/tests" not in d[0] and "pyarrow\\tests" not in d[0]]

a = Analysis(
    ["run_app.py"],
    pathex=[],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=["pyarrow.tests"],
    noarchive=False,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name="OpenETV",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name="OpenETV",
)

if sys.platform == "darwin":
    app = BUNDLE(
        coll,
        name="OpenETV.app",
        icon=None,
        bundle_identifier="app.openetv.desktop",
        info_plist={
            "NSHighResolutionCapable": "True",
            "CFBundleShortVersionString": "0.1.0",
            "CFBundleName": "OpenETV",
        },
    )
