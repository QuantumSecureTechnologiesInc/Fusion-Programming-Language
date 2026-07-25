# Fusion v2.0 Vortex Installers

## Quick Install

### Windows
```powershell
# PowerShell (Run as Administrator)
.\installers\windows\install.ps1
```

### Linux
```bash
bash installers/linux/install.sh
```

### macOS
```bash
bash installers/macos/install.sh
```

## Prerequisites
- Rust toolchain (auto-installed if missing)
- LLVM (auto-installed on macOS via Homebrew)
- Build tools (gcc/make on Linux, Xcode CLI on macOS)

## What Gets Installed
- `fuc` compiler binary
- `fusion` CLI tool
- Standard library (~/.fusion/stdlib/)
- PATH configuration

## Uninstall
- Windows: `installers/windows/uninstall.ps1`
- Linux: `bash installers/linux/uninstall.sh`
- macOS: `bash installers/macos/uninstall.sh`
