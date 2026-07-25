#!/bin/bash
echo "Uninstalling Fusion v2.0 Vortex..."
rm -rf "$HOME/.fusion"
cargo uninstall fuc fusion 2>/dev/null
echo "Removed. Restart your terminal."
