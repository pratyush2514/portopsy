#!/bin/sh
set -eu

REPO="pratyush2514/portopsy"
INSTALL_DIR="/usr/local/bin"

say() { printf '%s\n' "portopsy: $*"; }

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) say "unsupported architecture: $ARCH"; exit 1 ;;
esac

if [ "$OS" != "linux" ]; then
  say "unsupported operating system: $OS"
  say "use a Linux or WSL2 environment"
  exit 1
fi

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

BASE_URL="https://github.com/${REPO}/releases/latest/download"
ARCHIVE=""

for candidate in \
  "portopsy-${ARCH}-unknown-linux-gnu.tar.gz" \
  "portopsy-${ARCH}-linux.tar.gz" \
  "portopsy_${ARCH}_linux.tar.gz" \
  "portopsy-${ARCH}.tar.gz"; do
  if curl -fsSL "$BASE_URL/$candidate" -o "$TMP_DIR/$candidate"; then
    ARCHIVE="$TMP_DIR/$candidate"
    break
  fi
done

if [ -z "$ARCHIVE" ]; then
  say "could not find a release for Linux/$ARCH"
  say "check the latest assets at https://github.com/${REPO}/releases"
  exit 1
fi

if ! tar -xzf "$ARCHIVE" -C "$TMP_DIR"; then
  say "downloaded release is not a valid tar.gz archive"
  exit 1
fi

BINARY=$(find "$TMP_DIR" -type f -name portopsy -print -quit)
if [ -z "$BINARY" ]; then
  say "release archive does not contain a portopsy binary"
  exit 1
fi

if [ -w "$INSTALL_DIR" ]; then
  cp "$BINARY" "$INSTALL_DIR/portopsy"
else
  say "elevated permissions required to install to $INSTALL_DIR"
  sudo cp "$BINARY" "$INSTALL_DIR/portopsy"
fi

chmod +x "$INSTALL_DIR/portopsy"
say "installed to $INSTALL_DIR/portopsy"