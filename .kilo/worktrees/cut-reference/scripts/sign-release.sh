#!/bin/bash
# Sign release files for Tauri updater
# Run this script to generate signatures for your release files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT_NODE="$(cygpath -w "$PROJECT_ROOT" 2>/dev/null || printf '%s' "$PROJECT_ROOT")"
RELEASE_DIR="$PROJECT_ROOT/release-signing"
RELEASE_DIR_NODE="$(cygpath -w "$RELEASE_DIR" 2>/dev/null || printf '%s' "$RELEASE_DIR")"
PRIVATE_KEY_PATH="${TAURI_SIGNING_PRIVATE_KEY_PATH:-$HOME/.tauri/whisprtypr.key}"
VERSION="${VERSION:-$(PROJECT_ROOT_NODE="$PROJECT_ROOT_NODE" node -p "require(process.env.PROJECT_ROOT_NODE + '/src-tauri/tauri.conf.json').version")}"
REPOSITORY="${GITHUB_REPOSITORY:-johuniq/whisprtypr}"
PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}"
WINDOWS_SETUP="$PROJECT_ROOT/src-tauri/target/release/bundle/nsis/WhisprTypr_${VERSION}_x64-setup.exe"
WINDOWS_MSI="$PROJECT_ROOT/src-tauri/target/release/bundle/msi/WhisprTypr_${VERSION}_x64_en-US.msi"

# Check if private key exists
if [ ! -f "$PRIVATE_KEY_PATH" ]; then
    echo "Error: Private key not found at $PRIVATE_KEY_PATH"
    echo "Generate one with: cargo tauri signer generate -w ~/.tauri/whisprtypr.key"
    exit 1
fi

# Check build artifacts exist
if [ ! -f "$WINDOWS_SETUP" ]; then
    echo "Error: Windows setup artifact not found at $WINDOWS_SETUP"
    echo "Build it first with: cargo tauri build"
    exit 1
fi

if [ ! -f "$WINDOWS_MSI" ]; then
    echo "Error: Windows MSI artifact not found at $WINDOWS_MSI"
    echo "Build it first with: cargo tauri build"
    exit 1
fi

# Create directory for signing
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"
cp "$WINDOWS_SETUP" "$RELEASE_DIR/"
cp "$WINDOWS_MSI" "$RELEASE_DIR/"
cd "$RELEASE_DIR"

echo "=== WhisprTypr Release Signing Script ==="
echo ""

echo "Release files:"
ls -la

echo ""
echo "=== Signing files and generating latest.json ==="
echo ""

TAURI_SIGNING_PRIVATE_KEY="$(cat "$PRIVATE_KEY_PATH")" \
TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$PRIVATE_KEY_PASSWORD" \
VERSION="$VERSION" \
GITHUB_REPOSITORY="$REPOSITORY" \
node "$PROJECT_ROOT_NODE/scripts/generate-latest-json.mjs" "$RELEASE_DIR_NODE" "$RELEASE_DIR_NODE/latest.json"

echo "Generated latest.json with signatures:"
cat latest.json

echo ""
echo "=== Next Steps ==="
echo "1. Upload 'latest.json' to your GitHub release v$VERSION"
echo "2. Go to: https://github.com/$REPOSITORY/releases/edit/v$VERSION"
echo "3. Attach the file: $RELEASE_DIR/latest.json"
echo ""
echo "Files are in: $RELEASE_DIR/"
