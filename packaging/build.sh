#!/bin/bash
# Build on the destination platform. No Python runtime is bundled.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$here"
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' rust/Cargo.toml | head -n 1)"
[ -n "$version" ] || { echo "Missing Cargo package version" >&2; exit 1; }
target="${OPENETV_APP_DIR:-$(cd "$here/../.." && pwd)/OpenETV_App}"
build_dir="${CARGO_TARGET_DIR:-$here/rust/target}"
case "$build_dir" in /*) ;; *) build_dir="$here/$build_dir" ;; esac
cargo build --manifest-path rust/Cargo.toml --release --locked -j3 --target-dir "$build_dir"
[ -x "$build_dir/release/openetv" ] || { echo "Native binary missing" >&2; exit 1; }
work="$(mktemp -d "${TMPDIR:-/tmp}/openetv-build.XXXXXX")"
trap 'rm -rf "$work"' EXIT
mkdir -p "$target"
case "$(uname -s)" in
  Darwin)
    app="$work/OpenETV.app"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    cp "$build_dir/release/openetv" "$app/Contents/MacOS/OpenETV"
    cp packaging/icon/app-icon.icns "$app/Contents/Resources/app-icon.icns"
    cp LICENSE rust/MANUAL.md "$app/Contents/Resources/"
    cp -R rust/help "$app/Contents/Resources/help"
    cp -R rust/assets/look "$app/Contents/Resources/third-party-look"
    cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>OpenETV</string>
<key>CFBundleDisplayName</key><string>OpenETV</string>
<key>CFBundleIdentifier</key><string>org.openetv.desktop</string>
<key>CFBundleExecutable</key><string>OpenETV</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>CFBundleVersion</key><string>$version</string>
<key>CFBundleIconFile</key><string>app-icon.icns</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
    codesign --force --deep --sign - "$app"
    codesign --verify --deep --strict "$app"
    rm -rf "$target/OpenETV.app"
    mv "$app" "$target/OpenETV.app"
    echo "Built: $target/OpenETV.app"
    if [ "${OPENETV_SKIP_DMG:-0}" != 1 ]; then
      mkdir "$work/dmg"
      cp -R "$target/OpenETV.app" "$work/dmg/"
      ln -s /Applications "$work/dmg/Applications"
      hdiutil create -quiet -volname "OpenETV $version" -srcfolder "$work/dmg" -ov -format UDZO "$target/OpenETV-$version-macOS.dmg"
      echo "Built: $target/OpenETV-$version-macOS.dmg"
    fi
    ;;
  Linux)
    mkdir "$work/OpenETV"
    cp "$build_dir/release/openetv" "$work/OpenETV/OpenETV"
    cp rust/MANUAL.md LICENSE "$work/OpenETV/"
    cp -R rust/help "$work/OpenETV/help"
    cp -R rust/assets/look "$work/OpenETV/third-party-look"
    rm -rf "$target/OpenETV"
    mv "$work/OpenETV" "$target/OpenETV"
    tar -czf "$target/OpenETV-$version-Linux.tar.gz" -C "$target" OpenETV
    echo "Built: $target/OpenETV-$version-Linux.tar.gz"
    ;;
  *) echo "Unsupported platform: $(uname -s)" >&2; exit 1 ;;
esac
