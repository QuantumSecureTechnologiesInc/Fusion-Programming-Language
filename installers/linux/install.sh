#!/bin/bash
# ============================================================================
# Fusion v2.0 Vortex — Linux Shell Installer
# ============================================================================
# Usage:
#   curl -sSL https://fusion-lang.org/install.sh | bash
#   ./install.sh              # Install from local source
#   ./install.sh --version 2.0.0  # Install specific version
#   ./install.sh --prefix /usr/local  # Custom install prefix
# ============================================================================
set -euo pipefail

FUSION_VERSION="${VERSION:-2.0.0}"
INSTALL_PREFIX="${PREFIX:-/opt/fusion}"
GITHUB_BASE="https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}>>>${NC} $*"; }
ok()    { echo -e "${GREEN}>>>${NC} $*"; }
warn()  { echo -e "${YELLOW}>>>${NC} $*"; }
err()   { echo -e "${RED}>>>${NC} $*" >&2; exit 1; }

echo ""
echo -e "${CYAN}  =========================================${NC}"
echo -e "${CYAN}   Fusion v2.0 Vortex Installer (Linux)${NC}"
echo -e "${CYAN}  =========================================${NC}"
echo ""

# ---------------------------------------------------------------------------
# Step 1: Try downloading pre-built release
# ---------------------------------------------------------------------------
info "Checking for pre-built release v${FUSION_VERSION}..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

DOWNLOADED=false
for ARCHIVE in \
    "fusion-${FUSION_VERSION}-linux-x64.tar.gz" \
    "fusion-${FUSION_VERSION}-linux-amd64.tar.gz" \
    "fusion-${FUSION_VERSION}-linux.tar.gz"; do

    URL="${GITHUB_BASE}/releases/download/v${FUSION_VERSION}/${ARCHIVE}"
    if curl -sSfL --connect-timeout 10 -o "${TMPDIR}/${ARCHIVE}" "$URL" 2>/dev/null; then
        ok "Downloaded ${ARCHIVE}"
        DOWNLOADED=true
        break
    fi
done

# ---------------------------------------------------------------------------
# Step 2: Install from release or build from source
# ---------------------------------------------------------------------------
if [ "$DOWNLOADED" = true ]; then
    info "Extracting release..."
    mkdir -p "$INSTALL_PREFIX"
    tar -xzf "${TMPDIR}/${ARCHIVE}" -C "$INSTALL_PREFIX" --strip-components=1
    ok "Installed to $INSTALL_PREFIX"
else
    warn "No pre-built release found. Building from source..."

    # Check for Rust
    if ! command -v rustc &>/dev/null; then
        info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    ok "Rust: $(rustc --version)"

    # Install system dependencies
    info "Installing system dependencies..."
    if command -v apt-get &>/dev/null; then
        sudo apt-get update -qq
        sudo apt-get install -y -qq build-essential libssl-dev pkg-config cmake
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y gcc openssl-devel pkg-config cmake
    elif command -v yum &>/dev/null; then
        sudo yum install -y gcc openssl-devel pkg-config cmake
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm base-devel openssl pkg-config cmake
    elif command -v zypper &>/dev/null; then
        sudo zypper install -y gcc libopenssl-devel pkg-config cmake
    else
        warn "Unknown package manager. Ensure build-essential, libssl-dev, pkg-config, cmake are installed."
    fi

    # Find project root (where Cargo.toml lives)
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
    cd "$PROJECT_ROOT"

    # Build compiler
    info "Building fuc compiler..."
    LLVM_FLAGS=""
    if command -v llvm-config &>/dev/null; then
        LLVM_FLAGS="--features llvm"
    fi
    cargo build --release --path crates/fuc $LLVM_FLAGS

    # Build CLI
    info "Building fusion CLI..."
    cargo build --release --path tools/fusion-cli

    # Install
    info "Installing to $INSTALL_PREFIX..."
    sudo mkdir -p "$INSTALL_PREFIX/bin"
    sudo cp target/release/fuc "$INSTALL_PREFIX/bin/fuc"
    sudo cp target/release/fusion "$INSTALL_PREFIX/bin/fusion"
    sudo chmod 755 "$INSTALL_PREFIX/bin/fuc" "$INSTALL_PREFIX/bin/fusion"

    # Install stdlib
    sudo mkdir -p "$INSTALL_PREFIX/stdlib"
    sudo cp -r stdlib/* "$INSTALL_PREFIX/stdlib/"

    ok "Built and installed to $INSTALL_PREFIX"
fi

# ---------------------------------------------------------------------------
# Step 3: Configure PATH
# ---------------------------------------------------------------------------
info "Configuring shell PATH..."

SHELL_RC=""
for RC in "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile" "$HOME/.zshrc"; do
    if [ -f "$RC" ]; then
        SHELL_RC="$RC"
        break
    fi
done

if [ -z "$SHELL_RC" ]; then
    SHELL_RC="$HOME/.bashrc"
    touch "$SHELL_RC"
fi

# Add to profile.d for system-wide access
if [ -w /etc/profile.d/ ]; then
    sudo tee /etc/profile.d/fusion.sh > /dev/null << 'PROFILE'
# Fusion v2.0 Vortex
if [ -d /opt/fusion/bin ]; then
    export PATH="/opt/fusion/bin:$PATH"
fi
PROFILE
    sudo chmod 644 /etc/profile.d/fusion.sh
fi

# Add to user shell rc
if ! grep -q "$INSTALL_PREFIX/bin" "$SHELL_RC" 2>/dev/null; then
    echo "" >> "$SHELL_RC"
    echo "# Fusion v2.0 Vortex" >> "$SHELL_RC"
    echo "export PATH=\"$INSTALL_PREFIX/bin:\$PATH\"" >> "$SHELL_RC"
    ok "Added to $SHELL_RC"
fi

# Update current PATH for this session
export PATH="$INSTALL_PREFIX/bin:$PATH"

# ---------------------------------------------------------------------------
# Verify
# ---------------------------------------------------------------------------
echo ""
info "Verifying installation..."
if command -v fusion &>/dev/null; then
    ok "Fusion $(fusion --version 2>/dev/null || echo 'installed') ready!"
else
    warn "Run: source $SHELL_RC"
fi

echo ""
echo -e "${GREEN}  =========================================${NC}"
echo -e "${GREEN}   Installation Complete!${NC}"
echo -e "${GREEN}  =========================================${NC}"
echo ""
echo "  Installed to:  $INSTALL_PREFIX"
echo "  Quick start:   fusion init my_project && cd my_project && fusion run"
echo ""
