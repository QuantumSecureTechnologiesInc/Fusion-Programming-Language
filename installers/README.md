# Fusion v2.0 Vortex — Package Manager & Installer Reference

Fusion is available across all major platforms through multiple package managers and installers.

---

## Quick Install

| Platform | Command |
|----------|---------|
| **Windows** | `winget install QSTech.Fusion` |
| **macOS** | `brew install fusion-lang` |
| **Linux (deb)** | `sudo dpkg -i fusion-lang-2.0.0-amd64.deb` |
| **Linux (rpm)** | `sudo rpm -i fusion-lang-2.0.0-1.x86_64.rpm` |
| **Linux (snap)** | `sudo snap install fusion-lang` |
| **Any (shell)** | `curl -sSL https://fusion-lang.org/install.sh \| bash` |

---

## Windows

### winget (Windows Package Manager)
```powershell
winget install QSTech.Fusion
```
- **Package ID:** `QSTech.Fusion`
- **Architectures:** x64, arm64
- **Scope:** Machine-wide (requires admin)
- **File associations:** `.fu`, `.fusion`
- Manifests: [`installers/windows/winget/manifests/`](windows/winget/manifests/)

### NSIS Installer (.exe)
```
Fusion-v2.0-Vortex-x64-Setup.exe
```
- Full wizard with component selection (compiler, stdlib, docs)
- Registers in Add/Remove Programs
- Adds to system PATH
- Start Menu shortcuts
- Silent install: `Fusion-v2.0-Vortex-x64-Setup.exe /S`
- Build: `makensis installers/windows/Fusion-Setup.nsi`

### MSI Installer (Enterprise)
```
Fusion-v2.0-Vortex-x64.msi
```
- WiX-based MSI for enterprise deployment
- Supports Group Policy (GPO) deployment
- Standard MSI properties for SCCM/Intune
- Silent install: `msiexec /i Fusion-v2.0-Vortex-x64.msi /quiet`
- Build: `wix build -arch x64 installers/windows/fusion.wxs`
- Config: [`installers/windows/fusion.wxs`](windows/fusion.wxs)

### PowerShell Installer (from source)
```powershell
# From source
.\installers\windows\install.ps1

# Specific version
.\installers\windows\install.ps1 -Version 2.0.0

# Custom directory
.\installers\windows\install.ps1 -InstallDir C:\Fusion

# Remote one-liner
irm https://fusion-lang.org/install.ps1 | iex
```

---

## macOS

### Homebrew
```bash
brew install fusion-lang
```
- **Tap:** `QuantumSecureTechnologiesInc/fusion-lang`
- Includes compiler (fuc), CLI (fusion), stdlib, and shell completions
- Formula: [`installers/macos/homebrew/fusion-lang.rb`](macos/homebrew/fusion-lang.rb)

### PKG Installer
```
Fusion-v2.0.0.pkg
```
- Standard Apple installer package
- Install: `sudo installer -pkg Fusion-v2.0.0.pkg -target /`
- Silent: `sudo installer -pkg Fusion-v2.0.0.pkg -target / -allowUntrusted`

### DMG Disk Image
```
Fusion-v2.0.0.dmg
```
- Contains PKG installer + standalone binaries
- Drag-to-install via Applications symlink
- Build: `hdiutil create -volname "Fusion v2.0" -srcfolder staging -ov -format UDZO Fusion.dmg`

### Shell Installer
```bash
curl -sSL https://fusion-lang.org/install.sh | bash
# Or from source:
./installers/macos/install.sh
```
- Checks for Xcode CLI Tools, Rust, LLVM
- Configures `~/.zshrc` PATH

### Xcode CLI Tools
Required for building Fusion from source on macOS.
```bash
xcode-select --install
```
The shell installer (`installers/macos/install.sh`) will automatically check and prompt for installation.

---

## Linux

### Debian/Ubuntu (.deb)
```bash
# Download and install
wget https://github.com/.../releases/download/v2.0.0/fusion-lang-2.0.0-amd64.deb
sudo dpkg -i fusion-lang-2.0.0-amd64.deb

# Or install dependencies
sudo apt-get install -f
```
- **Package:** `fusion-lang`
- **Architectures:** amd64, arm64
- **Installs to:** `/opt/fusion/`
- **Provides:** `fuc`, `fusion` binaries + stdlib
- Files: [`installers/linux/debian/`](linux/debian/)

### Fedora/RHEL (.rpm)
```bash
sudo rpm -i fusion-lang-2.0.0-1.x86_64.rpm
# Or from source RPM
rpmbuild -ba installers/linux/rpm/fusion-lang.spec
```
- **Package:** `fusion-lang`
- **Architectures:** x86_64, aarch64
- **Installs to:** `/opt/fusion/`
- Spec: [`installers/linux/rpm/fusion-lang.spec`](linux/rpm/fusion-lang.spec)

### Snap
```bash
sudo snap install fusion-lang
fusion init my_project
```
- **Confinement:** strict
- **Base:** core22
- **Plugs:** home, network, removable-media
- Config: [`installers/linux/snap/snapcraft.yaml`](linux/snap/snapcraft.yaml)

### Flatpak
```bash
flatpak install com.quantumsecure.Fusion
flatpak run com.quantumsecure.Fusion
```
- **Runtime:** org.freedesktop.Platform 23.08
- Config: [`installers/linux/flatpak/com.quantumsecure.Fusion.yml`](linux/flatpak/com.quantumsecure.Fusion.yml)

### AppImage
```bash
chmod +x Fusion.AppImage
./Fusion.AppImage
```
- Portable, no installation required
- Config: [`installers/linux/appimage/AppImageBuilder.json`](linux/appimage/AppImageBuilder.json)

### Shell Installer
```bash
curl -sSL https://fusion-lang.org/install.sh | bash
# Or from source:
./installers/linux/install.sh
```
- Supports apt, dnf, yum, pacman, zypper
- Auto-detects architecture and distro
- Configures `/etc/profile.d/fusion.sh` + shell rc

---

## Building Installers

### Prerequisites
- Rust 1.80+ (`rustup update`)
- Pre-built release binaries: `cargo build --release`

### Windows
```bash
# NSIS installer (requires NSIS installed)
makensis installers/windows/Fusion-Setup.nsi

# MSI installer (requires WiX Toolset v3+)
wix build -arch x64 installers/windows/fusion.wxs -o Fusion.msi

# Winget manifest update
wingetcreate update QSTech.Fusion --version 2.0.0
```

### macOS
```bash
# Build both .pkg and .dmg
./installers/macos/build-pkg.sh

# PKG only
./installers/macos/build-pkg.sh --pkg-only

# Homebrew audit
brew audit --new fusion-lang
```

### Linux
```bash
# .deb package
mkdir -p build/deb/DEBIAN build/deb/opt/fusion/bin
cp target/release/{fuc,fusion} build/deb/opt/fusion/bin/
cp -r stdlib build/deb/opt/fusion/
sed 's/Version: .*/Version: 2.0.0/' installers/linux/debian/control > build/deb/DEBIAN/control
cp installers/linux/debian/{postinst,prerm,postrm} build/deb/DEBIAN/
dpkg-deb --build build/deb fusion-lang-2.0.0-amd64.deb

# .rpm package
rpmbuild -ba installers/linux/rpm/fusion-lang.spec

# Snap
cd installers/linux && snapcraft

# Flatpak
flatpak-builder build-dir installers/linux/flatpak/com.quantumsecure.Fusion.yml

# AppImage
appimagetool AppDir Fusion.AppImage
```

---

## CI/CD Automation

The release workflow (`.github/workflows/release-packages.yml`) automatically:

1. **Triggers** on tag push (`v*.*.*`) or manual dispatch
2. **Builds** all platform packages in parallel
3. **Publishes** a GitHub Release with all assets
4. **Submits** the winget manifest to `winget-pkgs`
5. **Uploads** source tarballs

### Triggering a Release
```bash
# Tag and push
git tag v2.0.0
git push origin v2.0.0

# Or via GitHub Actions UI: Run workflow with version=2.0.0
```

---

## Package Signing

Before publishing, sign all binaries:

```bash
# Windows (Authenticode)
signtool sign /f cert.pfx /p password /t http://timestamp.digicert.com Fusion-v2.0-Vortex-x64-Setup.exe

# macOS (codesign)
codesign --sign "Developer ID Application: QuantumSecure Technologies Inc" target/release/fuc

# Linux (GPG for .deb/.rpm)
debsigs --sign builder fusion-lang-2.0.0-amd64.deb
rpm --addsign fusion-lang-2.0.0-1.x86_64.rpm
```

---

## Directory Structure

```
installers/
├── README.md                              # This file
├── windows/
│   ├── Fusion-Setup.nsi                   # NSIS installer script
│   ├── fusion.wxs                         # WiX MSI installer
│   ├── install.ps1                        # PowerShell installer
│   ├── uninstall.ps1                      # PowerShell uninstaller
│   ├── favicon.ico                        # Installer icon
│   └── winget/
│       ├── fusion.yaml                    # Legacy manifest (deprecated)
│       └── manifests/q/QSTech/Fusion/2.0.0/
│           ├── QSTech.Fusion.yaml                 # Version manifest
│           ├── QSTech.Fusion.installer.yaml       # Installer manifest
│           └── QSTech.Fusion.locale.en-US.yaml    # Locale manifest
├── macos/
│   ├── install.sh                         # Shell installer
│   ├── uninstall.sh                       # Shell uninstaller
│   ├── distribution.xml                   # PKG distribution
│   ├── build-pkg.sh                       # PKG + DMG builder
│   ├── favicon.ico                        # Installer icon
│   └── homebrew/
│       └── fusion-lang.rb                 # Homebrew formula
└── linux/
    ├── install.sh                         # Shell installer
    ├── uninstall.sh                       # Shell uninstaller
    ├── debian/
    │   ├── control                        # Package metadata
    │   ├── postinst                       # Post-install script
    │   ├── prerm                          # Pre-remove script
    │   └── postrm                         # Post-remove script
    ├── rpm/
    │   └── fusion-lang.spec               # RPM spec file
    ├── snap/
    │   └── snapcraft.yaml                 # Snap package config
    ├── flatpak/
    │   └── com.quantumsecure.Fusion.yml   # Flatpak manifest
    └── appimage/
        └── AppImageBuilder.json           # AppImage config
```
