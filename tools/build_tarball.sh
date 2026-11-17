#!/usr/bin/env bash
# Build a Linux .tar.gz of the shipping binary.
# usage: tools/build_tarball.sh [version] [out_dir]
#   version: name suffix (default: Cargo.toml version)
#   out_dir: where the tarball goes (default: target/dist)
set -euo pipefail

APP=blocks
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-$(grep -m1 '^version' "$ROOT/Cargo.toml" | cut -d'"' -f2)}"
OUT_DIR="${2:-$ROOT/target/dist}"
mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"
cd "$ROOT"
NAME="$APP-$VERSION-linux-x86_64"

cargo build --profile ship --locked

# resources sit next to the binary, where the game looks for them
rm -rf "${OUT_DIR:?}/$NAME"
mkdir -p "$OUT_DIR/$NAME"
cp "target/ship/$APP" "$OUT_DIR/$NAME/"
cp -r resources README.md LICENSE THIRD_PARTY_LICENSES.md "$OUT_DIR/$NAME/"
tar -C "$OUT_DIR" -czf "$OUT_DIR/$NAME.tar.gz" "$NAME"
rm -rf "${OUT_DIR:?}/$NAME"
echo "built $OUT_DIR/$NAME.tar.gz"
