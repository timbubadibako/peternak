#!/usr/bin/env sh
set -eu

APP="peternak-aiai"
VERSION="${PETERNAK_VERSION:-latest}"
BASE_URL="${PETERNAK_DOWNLOAD_BASE:-https://github.com/timbubadibako/peternak/releases/${VERSION}/download}"
INSTALL_DIR="${PETERNAK_INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS:$ARCH" in
  Linux:x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
  Darwin:x86_64) TARGET="x86_64-apple-darwin" ;;
  Darwin:arm64) TARGET="aarch64-apple-darwin" ;;
  *) echo "Unsupported platform: $OS $ARCH" >&2; exit 1 ;;
esac

ARCHIVE="$APP-$TARGET.tar.gz"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$INSTALL_DIR"
URL="${BASE_URL%/}/$ARCHIVE"

echo "Downloading $URL"
curl -fsSL "$URL" -o "$TMP_DIR/$ARCHIVE"
tar -xzf "$TMP_DIR/$ARCHIVE" -C "$TMP_DIR"
install -m 755 "$TMP_DIR/$APP" "$INSTALL_DIR/$APP"

echo "Installed $APP to $INSTALL_DIR/$APP"
echo "Make sure $INSTALL_DIR is in PATH."
