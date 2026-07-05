#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET="${1:-$(rustc -vV | awk '/host:/ {print $2}')}"
BIN_NAME="peternak-aiai"
EXE_NAME="$BIN_NAME"

case "$TARGET" in
  *windows*|*msvc*|*gnu*)
    if [[ "$TARGET" == *"windows"* || "$TARGET" == *"pc-windows"* ]]; then
      EXE_NAME="$BIN_NAME.exe"
    fi
    ;;
esac

echo "Building $TARGET"
cargo build --release --target "$TARGET"

DIST="$ROOT/dist"
STAGE="$DIST/peternak-aiai-$TARGET"
rm -rf "$STAGE"
mkdir -p "$STAGE"

cp "$ROOT/target/$TARGET/release/$EXE_NAME" "$STAGE/$EXE_NAME"
cp "$ROOT/README.md" "$STAGE/README.md"

ARCHIVE="$DIST/peternak-aiai-$TARGET.tar.gz"
tar -czf "$ARCHIVE" -C "$STAGE" "$EXE_NAME" README.md

echo "Created $ARCHIVE"
