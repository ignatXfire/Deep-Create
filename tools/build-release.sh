#!/usr/bin/env bash
# Deep Create — one-command release build script.
# Sources the org-local toolchain, builds the full workspace in release mode,
# builds the UI, and packages everything into a portable zip.
#
# Usage: ./tools/build-release.sh [--skip-ui]
# Output: target/release/deep-create-<version>-windows-x64.zip

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CANDIDATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ORG_DIR="$(dirname "$CANDIDATE_DIR")"
PROJECT_ROOT="$(dirname "$(dirname "$ORG_DIR")")"

echo "=== Deep Create Release Build ==="
echo "Candidate: $CANDIDATE_DIR"
echo "Org:       $ORG_DIR"

# 1. Activate toolchain
echo "[1/6] Activating toolchain..."
source "$ORG_DIR/build-env.sh"

# Verify linker
LINK_PATH="$(which link 2>/dev/null || true)"
if [[ "$LINK_PATH" != *Hostx64* ]]; then
    echo "ERROR: MSVC link.exe not found in PATH (got: $LINK_PATH)"
    echo "  Ensure .local/toolchain/msvc/ and .local/toolchain/sdk/ are present."
    exit 1
fi
echo "  Linker: $LINK_PATH"

# 2. Run tests first — do not ship untested code
echo "[2/6] Running workspace tests..."
cd "$CANDIDATE_DIR"
TEST_OUTPUT=$(cargo test --workspace --offline 2>&1)
TEST_PASSED=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= passed)' | awk '{s+=$1}END{print s}')
TEST_FAILED=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= failed)' | awk '{s+=$1}END{print s}')
echo "  Tests: $TEST_PASSED passed, $TEST_FAILED failed"
if [[ "$TEST_FAILED" != "0" ]]; then
    echo "ERROR: Tests failed. Fix before release."
    exit 1
fi

# 3. Architecture checker
echo "[3/6] Architecture checker..."
node tools/check-architecture.mjs
echo "  Architecture: OK"

# 4. Release build (full workspace)
echo "[4/6] Building release (workspace)..."
cargo build --workspace --release --offline
echo "  Release build: OK"

# 5. UI build (unless --skip-ui)
if [[ "${1:-}" != "--skip-ui" ]]; then
    echo "[5/6] Building UI..."
    cd "$CANDIDATE_DIR"
    pnpm ui:build
    echo "  UI build: OK"
else
    echo "[5/6] Skipping UI build (--skip-ui)"
fi

# 6. Package
echo "[6/6] Packaging..."
VERSION=$(grep '^version' "$CANDIDATE_DIR/Cargo.toml" | head -1 | grep -oP '"[^"]+"' | tr -d '"')
RELEASE_DIR="$ORG_DIR/target/release"
PACKAGE_NAME="deep-create-${VERSION}-windows-x64"
PACKAGE_DIR="$RELEASE_DIR/$PACKAGE_NAME"

rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"

# Copy binaries
for exe in dc-desktop.exe; do
    if [[ -f "$RELEASE_DIR/$exe" ]]; then
        cp "$RELEASE_DIR/$exe" "$PACKAGE_DIR/"
        echo "  + $exe"
    fi
done

# Copy UI dist if built
if [[ -d "$CANDIDATE_DIR/ui/dist" ]]; then
    cp -r "$CANDIDATE_DIR/ui/dist" "$PACKAGE_DIR/ui/"
    echo "  + ui/dist/"
fi

# Copy docs
for doc in README.md LICENSE; do
    if [[ -f "$CANDIDATE_DIR/$doc" ]]; then
        cp "$CANDIDATE_DIR/$doc" "$PACKAGE_DIR/"
    fi
done

# Create zip
cd "$RELEASE_DIR"
if command -v zip &>/dev/null; then
    zip -r "${PACKAGE_NAME}.zip" "$PACKAGE_NAME/"
    echo "  Package: $RELEASE_DIR/${PACKAGE_NAME}.zip"
elif command -v 7z &>/dev/null; then
    7z a "${PACKAGE_NAME}.zip" "$PACKAGE_NAME/"
    echo "  Package: $RELEASE_DIR/${PACKAGE_NAME}.zip"
else
    echo "  WARNING: no zip/7z found, package directory: $PACKAGE_DIR"
fi

echo ""
echo "=== Release build complete ==="
echo "  Version: $VERSION"
echo "  Tests:   $TEST_PASSED passed, $TEST_FAILED failed"
echo "  Output:  $PACKAGE_DIR"
