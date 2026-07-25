#!/bin/bash
echo "=== Fusion v2.0 Vortex Uninstaller ==="
echo ""

# Remove installation directory
echo "Removing ~/.fusion/..."
rm -rf "$HOME/.fusion"

# Uninstall cargo binaries
echo "Uninstalling binaries..."
cargo uninstall fuc 2>/dev/null || true
cargo uninstall fusion 2>/dev/null || true

# Remove from shell configs
CARGO_BIN="$HOME/.cargo/bin"
for RC_FILE in "$HOME/.bashrc" "$HOME/.profile" "$HOME/.bash_profile"; do
    if [ -f "$RC_FILE" ]; then
        sed -i "\|$CARGO_BIN|d" "$RC_FILE" 2>/dev/null || true
    fi
done

echo ""
echo "Fusion v2.0 Vortex has been uninstalled."
echo "Restart your terminal to apply PATH changes."
