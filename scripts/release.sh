#!/usr/bin/env bash
set -euo pipefail

# Release build + zip for local Seroost builds
# Usage: ./scripts/release.sh [VERSION]
# Output: releases/seroost-<VERSION>-<target>.zip

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VERSION="${1:-$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')}"
TARGET="$(rustc -vV | sed -n 's|host: ||p')"

RELEASE_DIR="$PROJECT_ROOT/releases"
BUILD_DIR="$PROJECT_ROOT/target/release"
ZIP_NAME="seroost-${VERSION}-${TARGET}"
ZIP_PATH="$RELEASE_DIR/${ZIP_NAME}.zip"

echo "Building Seroost v$VERSION for $TARGET ..."
cd "$PROJECT_ROOT"
cargo build --release

echo "Packaging release ..."
mkdir -p "$RELEASE_DIR"
rm -f "$ZIP_PATH"

# Create temp staging dir
STAGING="$(mktemp -d)"
trap "rm -rf '$STAGING'" EXIT

mkdir -p "$STAGING/$ZIP_NAME"
cp "$BUILD_DIR/seroost.exe" "$STAGING/$ZIP_NAME/seroost.exe" 2>/dev/null || \
cp "$BUILD_DIR/seroost"     "$STAGING/$ZIP_NAME/seroost"
cp "$PROJECT_ROOT/readme.md" "$STAGING/$ZIP_NAME/"
cp "$PROJECT_ROOT/CHANGELOG.md" "$STAGING/$ZIP_NAME/" 2>/dev/null || true

cd "$STAGING"
zip -r "$ZIP_PATH" "$ZIP_NAME"

echo ""
echo "Release created: $ZIP_PATH"
ls -lh "$ZIP_PATH"
