#!/usr/bin/env bash
# Build a Windows .zip of the release binary. Run on Windows from Git Bash.
# usage: tools/build_windows_zip.sh [version] [out_dir]
#   version: name suffix (default: Cargo.toml version)
#   out_dir: where the zip goes (default: target/dist)
set -euo pipefail

APP=blocks
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-$(grep -m1 '^version' "$ROOT/Cargo.toml" | cut -d'"' -f2)}"
OUT_DIR="${2:-$ROOT/target/dist}"
mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"
cd "$ROOT"
NAME="$APP-$VERSION-windows-x86_64"

cargo build --release --locked

# resources sit next to the exe, where ggez looks for them
rm -rf "${OUT_DIR:?}/$NAME" "$OUT_DIR/$NAME.zip"
mkdir -p "$OUT_DIR/$NAME"
cp "target/release/$APP.exe" "$OUT_DIR/$NAME/"
cp -r resources README.md LICENSE THIRD_PARTY_LICENSES.md "$OUT_DIR/$NAME/"
(
    cd "$OUT_DIR"
    if command -v 7z >/dev/null; then
        7z a -tzip "$NAME.zip" "$NAME" >/dev/null
    else
        powershell.exe -NoProfile -Command "Compress-Archive -Path '$NAME' -DestinationPath '$NAME.zip'"
    fi
)
rm -rf "${OUT_DIR:?}/$NAME"
echo "built $OUT_DIR/$NAME.zip"
