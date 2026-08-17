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

# --- .dmg ---
DMG_FILE=$(find "$BUNDLE_DIR/dmg" -maxdepth 1 -name "*.dmg" 2>/dev/null | head -1)
if [ -n "$DMG_FILE" ]; then
  cp "$DMG_FILE" "$DIST/${PRODUCT_NAME}-${VERSION}-aarch64.dmg"
  ARTIFACTS+=("$DIST/${PRODUCT_NAME}-${VERSION}-aarch64.dmg")
fi

# --- .app (inside dmg, extract if needed) ---
APP_DIR=$(find "$BUNDLE_DIR/macos" -maxdepth 1 -name "*.app" -type d 2>/dev/null | head -1)
if [ -n "$APP_DIR" ]; then
  cp -R "$APP_DIR" "$DIST/${PRODUCT_NAME}.app"
  ARTIFACTS+=("$DIST/${PRODUCT_NAME}.app")
fi

# --- Binary ---
if [ -f "$BINARY" ]; then
  cp "$BINARY" "$DIST/$MAIN_BINARY"
  chmod +x "$DIST/$MAIN_BINARY"
  ARTIFACTS+=("$DIST/$MAIN_BINARY")
fi

# --- Checksums ---
for artifact in "${ARTIFACTS[@]}"; do
  if [ -f "$artifact" ]; then
    shasum -a 256 "$artifact" > "${artifact}.sha256"
  fi
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
echo "  $(uname -m)"
echo ""
echo "Version:"
echo "  $VERSION"
