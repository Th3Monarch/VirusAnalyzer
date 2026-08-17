#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG_JSON="$ROOT/package.json"
TAURI_CONF="$ROOT/src-tauri/tauri.conf.json"

VERSION=$(node -p "require('$PKG_JSON').version")
PRODUCT_NAME=$(node -p "require('$TAURI_CONF').productName")
MAIN_BINARY=$(node -p "require('$TAURI_CONF').mainBinaryName // require('$TAURI_CONF').productName")

DIST="$ROOT/dist"
BINARY="$ROOT/src-tauri/target/release/$MAIN_BINARY"
BUNDLE_DIR="$ROOT/src-tauri/target/release/bundle"

mkdir -p "$DIST"

ARTIFACTS=()

# --- .deb ---
DEB_FILE=$(find "$BUNDLE_DIR/deb" -maxdepth 1 -name "*.deb" 2>/dev/null | head -1)
if [ -n "$DEB_FILE" ]; then
  cp "$DEB_FILE" "$DIST/${PRODUCT_NAME}_${VERSION}_amd64.deb"
  ARTIFACTS+=("$DIST/${PRODUCT_NAME}_${VERSION}_amd64.deb")
fi

# --- AppImage ---
APPIMAGE_FILE=$(find "$BUNDLE_DIR/appimage" -maxdepth 1 -name "*.AppImage" 2>/dev/null | head -1)
if [ -n "$APPIMAGE_FILE" ]; then
  cp "$APPIMAGE_FILE" "$DIST/${PRODUCT_NAME}-${VERSION}-amd64.AppImage"
  chmod +x "$DIST/${PRODUCT_NAME}-${VERSION}-amd64.AppImage"
  ARTIFACTS+=("$DIST/${PRODUCT_NAME}-${VERSION}-amd64.AppImage")
fi

# --- Binary ---
if [ -f "$BINARY" ]; then
  cp "$BINARY" "$DIST/$MAIN_BINARY"
  chmod +x "$DIST/$MAIN_BINARY"
  ARTIFACTS+=("$DIST/$MAIN_BINARY")
fi

# --- Checksums ---
for artifact in "${ARTIFACTS[@]}"; do
  sha256sum "$artifact" > "${artifact}.sha256"
done

echo ""
echo "BUILD SUCCESSFUL"
echo ""
echo "Artifacts:"
for artifact in "${ARTIFACTS[@]}"; do
  echo "  $artifact"
done
echo ""
echo "Architecture:"
echo "  x86_64-unknown-linux-gnu"
echo ""
echo "Version:"
echo "  $VERSION"
