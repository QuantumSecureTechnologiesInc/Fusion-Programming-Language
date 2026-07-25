#!/bin/bash
set -e

FUSION_VERSION="2.0.0"
INSTALL_DIR="$HOME/.fusion"

echo "=== Fusion v2.0 Vortex Installer (Linux) ==="
echo ""

# Step 1: Check Rust
echo "[1/6] Checking Rust..."
if ! command -v rustc &> /dev/null; then
    echo "  Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "  Rust: $(rustc --version)"

# Step 2: System dependencies
echo ""
echo "[2/6] Installing system dependencies..."
if command -v apt-get &> /dev/null; then
    sudo apt-get update -qq && sudo apt-get install -y -qq build-essential libssl-dev pkg-config cmake
elif command -v dnf &> /dev/null; then
    sudo dnf install -y gcc openssl-devel pkg-config cmake
elif command -v pacman &> /dev/null; then
    sudo pacman -S --noconfirm base-devel openssl pkg-config cmake
else
    echo "  Warning: Unknown distro. Ensure build-essential, libssl-dev, pkg-config are installed."
fi

# Step 3: Build compiler
echo ""
echo "[3/6] Building fuc compiler..."
cargo install --path crates/fuc --features llvm

# Step 4: Build CLI
echo ""
echo "[4/6] Building fusion CLI..."
cargo install --path tools/fusion-cli

# Step 5: Install stdlib
echo ""
echo "[5/6] Installing standard library..."
mkdir -p "$INSTALL_DIR/stdlib"
cp -r stdlib/* "$INSTALL_DIR/stdlib/"
echo "  Installed to $INSTALL_DIR/stdlib"

# Step 6: Shell config
echo ""
echo "[6/6] Configuring shell..."
CARGO_BIN="$HOME/.cargo/bin"

for RC_FILE in "$HOME/.bashrc" "$HOME/.profile" "$HOME/.bash_profile"; do
    if [ -f "$RC_FILE" ]; then
        if ! grep -q "$CARGO_BIN" "$RC_FILE" 2>/dev/null; then
            echo "export PATH=\"$CARGO_BIN:\$PATH\"" >> "$RC_FILE"
            echo "  Added to $RC_FILE"
        fi
    fi
done

# Verify
echo ""
echo "Verifying installation..."
if command -v fusion &> /dev/null; then
    echo "  Fusion $(fusion --version) installed successfully!"
else
    echo "  Run: source ~/.bashrc"
fi

echo ""
echo "=== Installation Complete ==="
echo "Quick start: fusion init my_project && cd my_project && fusion run"
