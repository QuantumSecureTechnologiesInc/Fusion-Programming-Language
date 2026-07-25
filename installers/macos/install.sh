#!/bin/bash
set -e

FUSION_VERSION="2.0.0"
INSTALL_DIR="$HOME/.fusion"

echo "=== Fusion v2.0 Vortex Installer (macOS) ==="
echo ""

# Step 1: Check Xcode
echo "[1/6] Checking prerequisites..."
if ! xcode-select -p &> /dev/null; then
    echo "  Installing Xcode Command Line Tools..."
    xcode-select --install
    echo "  Please complete the installation and re-run this script."
    exit 1
fi

# Step 2: Check Rust
echo "[2/6] Checking Rust..."
if ! command -v rustc &> /dev/null; then
    echo "  Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "  Rust: $(rustc --version)"

# Step 3: Check LLVM
echo ""
echo "[3/6] Checking LLVM..."
if ! command -v llvm-config &> /dev/null; then
    echo "  Installing LLVM via Homebrew..."
    if ! command -v brew &> /dev/null; then
        echo "  Installing Homebrew first..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    brew install llvm
fi
echo "  LLVM: $(llvm-config --version)"

# Step 4: Build
echo ""
echo "[4/6] Building Fusion..."
cargo install --path crates/fuc --features llvm
cargo install --path tools/fusion-cli

# Step 5: Install stdlib
echo ""
echo "[5/6] Installing standard library..."
mkdir -p "$INSTALL_DIR/stdlib"
cp -r stdlib/* "$INSTALL_DIR/stdlib/"

# Step 6: Shell config
echo ""
echo "[6/6] Configuring shell..."
if ! grep -q "$HOME/.cargo/bin" ~/.zshrc 2>/dev/null; then
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
    echo "  Added to ~/.zshrc"
fi

# Verify
echo ""
if command -v fusion &> /dev/null; then
    echo "  Fusion $(fusion --version) installed!"
else
    echo "  Run: source ~/.zshrc"
fi

echo ""
echo "=== Done ==="
echo "Quick start: fusion init my_project && cd my_project && fusion run"
