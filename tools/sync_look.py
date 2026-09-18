"""Fetch the look of Race Analysis: its theme, fonts, icons and the shared scripts.

    .venv/bin/python tools/sync_look.py            # copy, write tools/etvlib/look.json
    .venv/bin/python tools/sync_look.py --check    # only say what differs

OpenETV looks like Race Analysis because it uses the same files, unchanged —
and links them at the same addresses (`/theme.css`, `/common.js`, …), so that
it could become a window of that app one day. The files are copied, not
shared: OpenETV is a project of its own and has to build without the other
checkout. **Never edit a copied file here**: change it in Race Analysis and
sync; `test_look.py` fails when a copy no longer matches `look.json`.

Taken from what is *committed* there (`git show HEAD:…`), never from the
working tree — somebody may be in the middle of a change. The other checkout
is `../Race_Analysis`, or `RACE_ANALYSIS=/path`.
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "tools" / "etvlib"
MANIFEST = TARGET / "look.json"
SOURCE_DIR = "tools/racelib"

#: What is copied, as it is named in `tools/racelib/` — and here.
FILES = ("theme.css", "fonts.css", "theme.py", "theme.js", "prefs.js", "common.js", "icons.js", "notify.js", "help.js",
         "fonts/IBMPlexMono-Medium.woff2", "fonts/IBMPlexMono-Regular.woff2",
         "fonts/IBMPlexMono-SemiBold.woff2", "fonts/IBMPlexSans-Bold.woff2",
         "fonts/IBMPlexSans-Medium.woff2", "fonts/IBMPlexSans-Regular.woff2",
         "fonts/IBMPlexSans-SemiBold.woff2", "fonts/OFL.txt")


def _git(source: Path, *args: str) -> bytes:
    return subprocess.run(["git", "-C", str(source), *args], check=True, capture_output=True).stdout


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main(argv: list[str]) -> int:
    check = "--check" in argv
    source = Path(os.environ.get("RACE_ANALYSIS") or ROOT.parent / "Race_Analysis")
    if not (source / ".git").exists():
        print(f"No checkout of Race Analysis at {source} (set RACE_ANALYSIS=/path).")
        return 2
    commit = _git(source, "rev-parse", "HEAD").decode().strip()

    differs, hashes = [], {}
    for name in FILES:
        data = _git(source, "show", f"HEAD:{SOURCE_DIR}/{name}")
        hashes[name] = sha256(data)
        here = TARGET / name
        if not here.is_file() or here.read_bytes() != data:
            differs.append(name)
            if not check:
                here.parent.mkdir(parents=True, exist_ok=True)
                here.write_bytes(data)

    if check:
        print(f"Race Analysis {commit[:7]}: " + ("the look is up to date." if not differs
                                                 else "differs — " + ", ".join(differs)))
        return 1 if differs else 0
    MANIFEST.write_text(json.dumps({"source": "Race Analysis, tools/racelib", "commit": commit,
                                    "files": hashes}, indent=2) + "\n", encoding="utf-8")
    print(f"Race Analysis {commit[:7]}: {len(differs)} of {len(FILES)} files changed"
          + (": " + ", ".join(differs) if differs else "."))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
