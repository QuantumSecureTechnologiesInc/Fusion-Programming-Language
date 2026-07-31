#!/bin/bash
# ============================================================================
# Fusion v2.0 Vortex — macOS PKG & DMG Builder
# ============================================================================
# Prerequisites:
#   - Pre-built binaries in target/release/ (fuc, fusion)
#   - Xcode Command Line Tools
# Usage:
#   ./build-pkg.sh              # Build both .pkg and .dmg
#   ./build-pkg.sh --pkg-only   # Build .pkg only
#   ./build-pkg.sh --dmg-only   # Build .dmg only
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build/macos"
VERSION="${VERSION:-2.0.0}"
IDENTIFIER="com.quantumsecure.fusion"
INSTALL_PREFIX="/usr/local"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${YELLOW}>>>${NC} $*"; }
ok()    { echo -e "${GREEN}>>>${NC} $*"; }
err()   { echo -e "${RED}ERROR:${NC} $*" >&2; exit 1; }

BUILD_PKG=true
BUILD_DMG=true

for arg in "$@"; do
    case "$arg" in
        --pkg-only) BUILD_DMG=false ;;
        --dmg-only) BUILD_PKG=false ;;
        --help|-h)
            echo "Usage: $0 [--pkg-only|--dmg-only]"
            exit 0
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Check prerequisites
# ---------------------------------------------------------------------------
info "Checking prerequisites..."

if ! command -v xcode-select &>/dev/null; then
    err "Xcode Command Line Tools required. Run: xcode-select --install"
fi

if [ ! -f "$PROJECT_ROOT/target/release/fuc" ] || [ ! -f "$PROJECT_ROOT/target/release/fusion" ]; then
    err "Pre-built binaries not found. Run: cargo build --release --path crates/fuc && cargo build --release --path tools/fusion-cli"
fi

# ---------------------------------------------------------------------------
# Prepare payload directory
# ---------------------------------------------------------------------------
info "Preparing payload..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/root$INSTALL_PREFIX/bin"
mkdir -p "$BUILD_DIR/root$INSTALL_PREFIX/share/fusion/stdlib"
mkdir -p "$BUILD_DIR/root$INSTALL_PREFIX/share/fusion/docs"
mkdir -p "$BUILD_DIR/root/Library/LaunchDaemons"

# Copy binaries
cp "$PROJECT_ROOT/target/release/fuc" "$BUILD_DIR/root$INSTALL_PREFIX/bin/"
cp "$PROJECT_ROOT/target/release/fusion" "$BUILD_DIR/root$INSTALL_PREFIX/bin/"
chmod 755 "$BUILD_DIR/root$INSTALL_PREFIX/bin/fuc"
chmod 755 "$BUILD_DIR/root$INSTALL_PREFIX/bin/fusion"

# Copy standard library
cp -r "$PROJECT_ROOT/stdlib/"* "$BUILD_DIR/root$INSTALL_PREFIX/share/fusion/stdlib/"

# Copy documentation
cp "$PROJECT_ROOT"/docs/guides/*.md "$BUILD_DIR/root$INSTALL_PREFIX/share/fusion/docs/" 2>/dev/null || true

# Copy license
cp "$PROJECT_ROOT/LICENSE" "$BUILD_DIR/root$INSTALL_PREFIX/share/fusion/"

# ---------------------------------------------------------------------------
# Create a welcome/readme for the installer
# ---------------------------------------------------------------------------
mkdir -p "$BUILD_DIR/Resources"
cat > "$BUILD_DIR/Resources/welcome.html" << 'HTML'
<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Welcome</title>
<style>body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; padding: 20px; }</style>
</head>
<body>
<h2>Fusion v2.0 Vortex</h2>
<p>This installer will install the Fusion programming language on your Mac.</p>
<p>Fusion is a modern, polyglot systems programming language with post-quantum cryptography, quantum computing, blockchain, and advanced type system features.</p>
<p><strong>Installed components:</strong></p>
<ul>
  <li><code>fuc</code> — Fusion Compiler</li>
  <li><code>fusion</code> — Fusion CLI</li>
  <li>Standard Library</li>
  <li>Documentation</li>
</ul>
<p>Installation location: /usr/local/</p>
</body>
</html>
HTML

# ---------------------------------------------------------------------------
# Build .pkg
# ---------------------------------------------------------------------------
if [ "$BUILD_PKG" = true ]; then
    info "Building .pkg installer..."

    # Create component package
    pkgbuild \
        --root "$BUILD_DIR/root" \
        --identifier "$IDENTIFIER" \
        --version "$VERSION" \
        --install-location "/" \
        --scripts "$SCRIPT_DIR/pkg-scripts" \
        "$BUILD_DIR/Fusion.pkg" 2>/dev/null || \
    pkgbuild \
        --root "$BUILD_DIR/root" \
        --identifier "$IDENTIFIER" \
        --version "$VERSION" \
        --install-location "/" \
        "$BUILD_DIR/Fusion.pkg"

    # Create product archive with distribution
    if [ -f "$SCRIPT_DIR/distribution.xml" ]; then
        productbuild \
            --distribution "$SCRIPT_DIR/distribution.xml" \
            --resources "$BUILD_DIR/Resources" \
            --package-path "$BUILD_DIR" \
            "$BUILD_DIR/Fusion-v${VERSION}.pkg" 2>/dev/null || \
        cp "$BUILD_DIR/Fusion.pkg" "$BUILD_DIR/Fusion-v${VERSION}.pkg"
    else
        cp "$BUILD_DIR/Fusion.pkg" "$BUILD_DIR/Fusion-v${VERSION}.pkg"
    fi

    ok "Built: $BUILD_DIR/Fusion-v${VERSION}.pkg"
fi

# ---------------------------------------------------------------------------
# Build .dmg
# ---------------------------------------------------------------------------
if [ "$BUILD_DMG" = true ]; then
    info "Building .dmg disk image..."

    DMG_DIR="$BUILD_DIR/dmg-staging"
    rm -rf "$DMG_DIR"
    mkdir -p "$DMG_DIR"

    # Create Applications symlink for drag-to-install
    ln -s /Applications "$DMG_DIR/Applications"

    # Copy .pkg into the DMG
    if [ -f "$BUILD_DIR/Fusion-v${VERSION}.pkg" ]; then
        cp "$BUILD_DIR/Fusion-v${VERSION}.pkg" "$DMG_DIR/"
    fi

    # Also copy binaries directly for users who prefer manual install
    mkdir -p "$DMG_DIR/Fusion"
    cp "$PROJECT_ROOT/target/release/fuc" "$DMG_DIR/Fusion/"
    cp "$PROJECT_ROOT/target/release/fusion" "$DMG_DIR/Fusion/"
    cp -r "$PROJECT_ROOT/stdlib" "$DMG_DIR/Fusion/stdlib"

    DMG_PATH="$BUILD_DIR/Fusion-v${VERSION}.dmg"

    # Create the DMG
    hdiutil create \
        -volname "Fusion v${VERSION}" \
        -srcfolder "$DMG_DIR" \
        -ov \
        -format UDZO \
        -imagekey zlib-level=9 \
        "$DMG_PATH"

    ok "Built: $DMG_PATH"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
ok "Build complete!"
echo ""
echo "  Output files:"
[ "$BUILD_PKG" = true ] && echo "    .pkg:  $BUILD_DIR/Fusion-v${VERSION}.pkg"
[ "$BUILD_DMG" = true ] && echo "    .dmg:  $BUILD_DIR/Fusion-v${VERSION}.dmg"
echo ""
echo "  To install .pkg:  sudo installer -pkg Fusion-v${VERSION}.pkg -target /"
echo "  To install .dmg:  Open the .diskimage and drag Fusion to Applications"
echo ""
