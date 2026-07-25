# Fusion v2.0 Vortex — Installer Formats

## Windows
| Format | File | Description |
|--------|------|-------------|
| .exe | Fusion-v2.0-Vortex-Setup.exe | NSIS installer with wizard |
| .msi | Fusion-v2.0-Vortex.msi | MSI installer for enterprise |
| winget | fusion-lang | Windows Package Manager |

## macOS
| Format | File | Description |
|--------|------|-------------|
| .pkg | Fusion.pkg | Apple installer package |
| .dmg | Fusion.dmg | Disk image with drag-to-install |
| .app | Fusion.app | Application bundle |
| Homebrew | brew install fusion-lang | Package manager |

## Linux
| Format | File | Description |
|--------|------|-------------|
| .deb | fusion-lang.deb | Debian/Ubuntu package |
| .rpm | fusion-lang.rpm | Fedora/RHEL package |
| .AppImage | Fusion.AppImage | Portable single-file |
| Flatpak | com.quantumsecure.Fusion | Universal sandbox |
| Snap | fusion-lang | Snap package |

## Building Installers

### Windows
```bash
# Build NSIS installer
makensis installers/windows/Fusion-Setup.nsi

# Build winget package
wingetcreate generate
```

### macOS
```bash
# Build .pkg
pkgbuild --root installers/macos/root --identifier com.quantumsecure.fusion --version 2.0.0 Fusion.pkg

# Build .dmg
hdiutil create -volname "Fusion v2.0" -srcfolder Fusion.app -ov -format UDZO Fusion.dmg
```

### Linux
```bash
# Build .deb
dpkg-deb --build installers/linux/debian fusion-lang.deb

# Build .rpm
rpmbuild -ba installers/linux/rpm/fusion-lang.spec

# Build AppImage
appimagetool AppDir Fusion.AppImage

# Build Snap
snapcraft

# Build Flatpak
flatpak-builder build-dir installers/linux/flatpak/com.quantumsecure.Fusion.yml
```
