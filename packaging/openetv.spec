# PyInstaller build plan for OpenETV.
#
#   pyinstaller packaging/openetv.spec --noconfirm
#
# Builds a standalone app: Python, numpy, openpyxl, pywebview and the window's
# pages are inside, nothing has to be installed on the target computer. Built
# per platform — PyInstaller does not build across platforms. The scripts in
# `packaging/` call this file.

import re
import sys
from pathlib import Path

ROOT = Path(SPECPATH).parent
TOOLS = ROOT / "tools"
LIB = TOOLS / "etvlib"

# The version is written once, in etvlib/__init__.py. macOS wants digits only
# in the bundle — "0.2.0-beta" becomes "0.2.0" there.
VERSION = re.search(r'__version__ = "([^"]+)"', (LIB / "__init__.py").read_text(encoding="utf-8"))[1]
VERSION_DIGITS = re.match(r"[\d.]+", VERSION)[0]

# The window is HTML, CSS and JS next to the code — they have to come along.
# Top level: the look and the shared scripts copied from Race Analysis
# (theme.css, fonts.css, common.js, …), served by etvlib/server.py.
pages = [(str(p), "etvlib") for pattern in ("*.js", "*.css") for p in LIB.glob(pattern)]
# The page and its modules.
pages += [(str(p), "etvlib/etv") for pattern in ("*.html", "*.css", "*.js") for p in (LIB / "etv").glob(pattern)]
# The shipped fonts (IBM Plex, OFL) with their licence.
pages += [(str(p), "etvlib/fonts") for p in (LIB / "fonts").glob("*") if p.is_file()]
# The tables the window starts with (etvlib/api.py, SAMPLE_DIRS).
pages += [(str(p), "etvlib/sample_data") for p in (ROOT / "sample_data").glob("*.csv")]
# The licences of what ships with the app.
pages += [(str(ROOT / name), ".") for name in ("LICENSE", "THIRD-PARTY-NOTICES.md")]

a = Analysis(
    [str(ROOT / "packaging" / "launcher.py")],
    pathex=[str(TOOLS)],
    binaries=[],
    datas=pages,
    hiddenimports=[],
    hookspath=[],
    runtime_hooks=[],
    # What is not needed stays out: numpy otherwise drags its tests along, and
    # nothing here draws with matplotlib or reads with pandas.
    excludes=["tkinter", "matplotlib", "scipy", "pandas", "pyarrow", "PIL", "pytest", "playwright",
              "numpy.f2py", "numpy.testing", "setuptools", "pip"],
    noarchive=False,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz, a.scripts, [],
    exclude_binaries=True,
    name="OpenETV",
    debug=False,
    strip=False,
    upx=False,
    console=False,          # no terminal window — the app has its own
    argv_emulation=False,
    target_arch=None,
    icon=str(ROOT / "packaging" / "icon" / ("app-icon.ico" if sys.platform == "win32" else "app-icon.icns")),
)

coll = COLLECT(exe, a.binaries, a.datas, strip=False, upx=False, name="OpenETV")

if sys.platform == "darwin":
    app = BUNDLE(
        coll,
        name="OpenETV.app",
        bundle_identifier="app.openetv.desktop",
        # packaging/icon/app-icon.svg is the source (packaging/icon/make_icon.py).
        icon=str(ROOT / "packaging" / "icon" / "app-icon.icns"),
        info_plist={
            "CFBundleName": "OpenETV",
            "CFBundleDisplayName": "OpenETV",
            "CFBundleShortVersionString": VERSION_DIGITS,
            "CFBundleVersion": VERSION_DIGITS,
            "CFBundleGetInfoString": f"OpenETV {VERSION}",
            "NSHighResolutionCapable": True,
            "LSMinimumSystemVersion": "11.0",
        },
    )
