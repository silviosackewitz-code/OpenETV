#!/bin/bash
# Builds the standalone app for macOS or Linux.
#
#   bash packaging/build.sh
#
# Result, outside the repository — next to the folder `repo`, so that what is
# source and what is the finished app stay apart:
#   macOS  <Projects>/OpenETV_App/OpenETV.app  and  OpenETV-<version>-macOS.dmg
#   Linux  <Projects>/OpenETV_App/OpenETV/     and  OpenETV-<version>-Linux.tar.gz
# Another place: OPENETV_APP_DIR=/path bash packaging/build.sh
# The target computer needs no Python and no numpy afterwards.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$here"

python="$here/.venv/bin/python"
[ -x "$python" ] || { echo "No .venv. First: python3 -m venv .venv && .venv/bin/pip install -r requirements-dev.txt"; exit 1; }

# The repository lies in <Projects>/repo/OpenETV — the app two levels up.
target="${OPENETV_APP_DIR:-$(cd "$here/../.." && pwd)/OpenETV_App}"
work="$(mktemp -d "${TMPDIR:-/tmp}/openetv-build.XXXXXX")"
trap 'rm -rf "$work"' EXIT
mkdir -p "$target"
version="$("$python" -c "import re,pathlib; print(re.search(r'__version__ = \"([^\"]+)\"', pathlib.Path('tools/etvlib/__init__.py').read_text())[1])")"

# The pinned versions of requirements-dev.txt, so that a build does not change from day to day.
"$python" -m pip install --quiet -r requirements-dev.txt
"$python" -m PyInstaller packaging/openetv.spec --noconfirm --clean \
  --distpath "$work/dist" --workpath "$work/build"

if [ -d "$work/dist/OpenETV.app" ]; then
  # Only the old app is replaced, the target folder is not emptied.
  rm -rf "$target/OpenETV.app"
  mv "$work/dist/OpenETV.app" "$target/"
  # The disk image: the app beside a link to Applications — drag it over to install.
  dmg="$target/OpenETV-$version-macOS.dmg"
  mkdir "$work/dmg"
  cp -R "$target/OpenETV.app" "$work/dmg/"
  ln -s /Applications "$work/dmg/Applications"
  rm -f "$dmg"
  hdiutil create -quiet -volname "OpenETV $version" -srcfolder "$work/dmg" -ov -format UDZO "$dmg"
  echo "Done: $target/OpenETV.app ($(du -sh "$target/OpenETV.app" | cut -f1))"
  echo "      $dmg ($(du -sh "$dmg" | cut -f1))"
  echo "  Start: open '$target/OpenETV.app'"
else
  rm -rf "$target/OpenETV"
  mv "$work/dist/OpenETV" "$target/"
  archive="$target/OpenETV-$version-Linux.tar.gz"
  tar -czf "$archive" -C "$target" OpenETV
  echo "Done: $target/OpenETV/ and $archive"
  echo "  Start: '$target/OpenETV/OpenETV'"
fi
