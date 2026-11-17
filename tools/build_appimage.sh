#!/usr/bin/env bash
# Build a Linux AppImage of the shipping binary.
# usage: tools/build_appimage.sh [version] [out_dir]
#   version: name suffix (default: Cargo.toml version)
#   out_dir: where the AppImage goes (default: target/dist)
set -euo pipefail

# pinned tools, checked before use
APPIMAGETOOL_URL=https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage
APPIMAGETOOL_SHA256=ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0
RUNTIME_URL=https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64
RUNTIME_SHA256=2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d

APP=blocks
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-$(grep -m1 '^version' "$ROOT/Cargo.toml" | cut -d'"' -f2)}"
OUT_DIR="${2:-$ROOT/target/dist}"
mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"
cd "$ROOT"
TOOLS=target/appimage-tools
APPDIR="$OUT_DIR/AppDir"

fetch() { # url sha256 dest
    if ! echo "$2  $3" | sha256sum -c --status - 2>/dev/null; then
        curl -fsSL -o "$3" "$1"
        echo "$2  $3" | sha256sum -c -
    fi
}

mkdir -p "$TOOLS"
fetch "$APPIMAGETOOL_URL" "$APPIMAGETOOL_SHA256" "$TOOLS/appimagetool"
fetch "$RUNTIME_URL" "$RUNTIME_SHA256" "$TOOLS/runtime"
chmod +x "$TOOLS/appimagetool"

cargo build --profile ship --locked

# resources sit next to the binary, where the game looks for them
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps" "$APPDIR/usr/share/doc/$APP"
cp "target/ship/$APP" "$APPDIR/usr/bin/"
cp -r resources "$APPDIR/usr/bin/"
cp packaging/blocks.desktop "$APPDIR/usr/share/applications/"
cp packaging/blocks.desktop "$APPDIR/"
cp packaging/blocks.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/"
cp packaging/blocks.png "$APPDIR/"
cp LICENSE THIRD_PARTY_LICENSES.md "$APPDIR/usr/share/doc/$APP/"
ln -s "usr/bin/$APP" "$APPDIR/AppRun"

# extract-and-run: no FUSE needed
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "$TOOLS/appimagetool" --runtime-file "$TOOLS/runtime" \
    "$APPDIR" "$OUT_DIR/$APP-$VERSION-linux-x86_64.AppImage"
rm -rf "$APPDIR"
echo "built $OUT_DIR/$APP-$VERSION-linux-x86_64.AppImage"
