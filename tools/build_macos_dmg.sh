#!/usr/bin/env bash
# Build a universal (Apple silicon + Intel) Blocks.app in a .dmg. Run on macOS.
# usage: tools/build_macos_dmg.sh [version] [out_dir]
#   version: name suffix and app version (default: Cargo.toml version)
#   out_dir: where the dmg goes (default: target/dist)
set -euo pipefail

APP=blocks
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-$(grep -m1 '^version' "$ROOT/Cargo.toml" | cut -d'"' -f2)}"
OUT_DIR="${2:-$ROOT/target/dist}"
mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"
cd "$ROOT"

NAME="$APP-$VERSION-macos-universal"
STAGING="$OUT_DIR/dmg"
BUNDLE="$STAGING/Blocks.app"
ICONSET="$OUT_DIR/blocks.iconset"

export MACOSX_DEPLOYMENT_TARGET=11.0
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --profile ship --locked --target aarch64-apple-darwin
cargo build --profile ship --locked --target x86_64-apple-darwin

rm -rf "$STAGING" "$ICONSET" "$OUT_DIR/$NAME.dmg"
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Resources"
lipo -create -output "$BUNDLE/Contents/MacOS/$APP" \
    "target/aarch64-apple-darwin/ship/$APP" \
    "target/x86_64-apple-darwin/ship/$APP"

# game loads resources from Contents/Resources, see bundle_resource_dir
cp -r resources/ "$BUNDLE/Contents/Resources/"
cp LICENSE THIRD_PARTY_LICENSES.md "$BUNDLE/Contents/Resources/"
sed "s|@VERSION@|$VERSION|g" packaging/Info.plist >"$BUNDLE/Contents/Info.plist"

mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
    sips -z $s $s packaging/blocks.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    sips -z $((s * 2)) $((s * 2)) packaging/blocks.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$BUNDLE/Contents/Resources/blocks.icns"
rm -rf "$ICONSET"

# ad-hoc signature; required to launch on Apple silicon
codesign --force --sign - "$BUNDLE"
codesign --verify --strict --verbose=2 "$BUNDLE"

ln -s /Applications "$STAGING/Applications"
hdiutil create -volname Blocks -srcfolder "$STAGING" -ov -format UDZO "$OUT_DIR/$NAME.dmg"
rm -rf "$STAGING"
echo "built $OUT_DIR/$NAME.dmg"
