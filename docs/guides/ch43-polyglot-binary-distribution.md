# Chapter 43: Cross-Platform Binary Distribution

You've built a Rust library with Python bindings. It works on your Linux laptop. Now ship it to users on macOS ARM, Windows, and Linux ARM64. Welcome to the binary distribution nightmare — this chapter makes it survivable.

## The Distribution Matrix

Every polyglot binary faces a 4×3 matrix of targets:

```
Platform × Architecture Matrix:

                  x86_64          ARM64           ARMv7
Linux            ✅ Primary      ✅ Required     ⚠️ Optional
macOS            ✅ Required     ✅ Primary      ❌ N/A
Windows          ✅ Required     ⚠️ Emerging     ❌ N/A

Runtime × Build System Matrix:

                  Rust            Python          Go
Linux x86_64     cargo build     pip wheel       go build
Linux ARM64      cross-compile   cross-compile   GOARM64=1
macOS ARM64      cargo build     pip wheel       go build
Windows x86_64   cargo build     pip wheel       go build
```

### Target Triple Reference

```
x86_64-unknown-linux-gnu      — Linux x86_64 (glibc)
x86_64-unknown-linux-musl     — Linux x86_64 (musl, static)
aarch64-unknown-linux-gnu     — Linux ARM64 (glibc)
aarch64-apple-darwin           — macOS ARM64 (Apple Silicon)
x86_64-pc-windows-msvc        — Windows x86_64 (MSVC)
x86_64-pc-windows-gnu         — Windows x86_64 (MinGW)
```

## Packaging for Multiple Platforms

### Python Wheel Cross-Compilation

```yaml
# .github/workflows/release-wheels.yml
name: Build Python Wheels

on:
  push:
    tags: ['v*']

jobs:
  build-wheels:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ['3.9', '3.10', '3.11', '3.12']
    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}

      - name: Install build tools
        run: pip install maturin setuptools-rust wheel

      - name: Build wheel
        run: maturin build --release --interpreter python${{ matrix.python-version }}

      - name: Upload wheel
        uses: actions/upload-artifact@v4
        with:
          name: wheel-${{ matrix.os }}-py${{ matrix.python-version }}
          path: target/wheels/*.whl
```

### Rust Cross-Compilation Setup

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-linux-gnu-gcc"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.aarch64-apple-darwin]
linker = "aarch64-apple-darwin21.3-clang"

[target.x86_64-pc-windows-msvc]
linker = "x86_64-pc-windows-msvc-linker"
```

```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-msvc

# Cross-compile for Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Cross-compile for macOS ARM64
cargo build --release --target aarch64-apple-darwin
```

### Go Cross-Compilation

```bash
# Go makes cross-compilation trivial
GOOS=linux GOARCH=amd64 go build -o bin/server-linux-amd64
GOOS=linux GOARCH=arm64 go build -o bin/server-linux-arm64
GOOS=darwin GOARCH=arm64 go build -o bin/server-darwin-arm64
GOOS=windows GOARCH=amd64 go build -o bin/server-windows-amd64.exe
```

### Multi-Platform Release Script

```bash
#!/bin/bash
# release.sh — Build for all platforms
set -euo pipefail

VERSION=$1
PLATFORMS=(
    "linux:amd64"
    "linux:arm64"
    "darwin:arm64"
    "windows:amd64"
)

for platform in "${PLATFORMS[@]}"; do
    IFS=':' read -r os arch <<< "$platform"
    output="bin/fusion-${os}-${arch}"
    [ "$os" = "windows" ] && output+=".exe"

    echo "Building for ${os}/${arch}..."
    GOOS=$os GOARCH=$arch go build -ldflags "-s -w -X main.version=${VERSION}" \
        -o "$output" ./cmd/fusion
done

echo "Build complete. Artifacts:"
ls -la bin/
```

## manylinux Wheels

The `manylinux` standard ensures Python wheels work across Linux distributions by bundling only old-enough glibc symbols.

### manylinux Standard

```
manylinux2014 (CentOS 7):  glibc >= 2.17
manylinux_2_28 (AlmaLinux 8): glibc >= 2.28
manylinux_2_31 (Debian 11): glibc >= 2.31

Compatibility:
manylinux2014 ⊃ manylinux_2_28 ⊃ manylinux_2_31

A manylinux2014 wheel works on ALL modern Linux distributions.
A manylinux_2_31 wheel ONLY works on Debian 11+, Ubuntu 22+.
```

### Building manylinux Wheels

```yaml
# .github/workflows/manylinux.yml
name: Build manylinux Wheels

jobs:
  build:
    runs-on: ubuntu-latest
    container: quay.io/pypa/manylinux_2_28_x86_64
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source $HOME/.cargo/env

      - name: Build manylinux wheel
        run: |
          pip install maturin
          maturin build --release --manylinux manylinux_2_28

      - name: Audit wheel
        run: |
          pip install auditwheel
          auditwheel repair target/wheels/*.whl --plat manylinux_2_28_x86_64
```

### Custom manylinux Build

```dockerfile
# manylinux.Dockerfile
FROM quay.io/pypa/manylinux_2_28_x86_64

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install build dependencies
RUN yum install -y gcc openssl-devel perl-IPC-Cmd

# Build for multiple Python versions
RUN for py in cp39 cp310 cp311 cp312; do \
        /opt/python/${py}/bin/pip install maturin && \
        /opt/python/${py}/bin/maturin build --release; \
    done

# Repair wheels for manylinux compatibility
RUN pip install auditwheel && \
    auditwheel repair target/wheels/*.whl --plat manylinux_2_28_x86_64
```

## JNI Libraries

JNI (Java Native Interface) libraries follow different distribution patterns than Python wheels.

### JNI Distribution Structure

```
fusion-native-1.0.0/
├── fusion-native-1.0.0.jar           # Java classes
├── fusion-native-1.0.0-linux-x86_64.jar    # Native library
├── fusion-native-1.0.0-linux-arm64.jar     # Native library
├── fusion-native-1.0.0-darwin-arm64.jar    # Native library
└── fusion-native-1.0.0-windows-x86_64.jar  # Native library
```

### Building JNI Libraries

```rust
// src/lib.rs — Rust library with JNI bindings
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[no_mangle]
pub extern "system" fn Java_com_fusion_NativeProcessor_process(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let input: String = env.get_string(&input).expect("Failed to get string").into();

    // Process input
    let result = format!("Processed: {}", input);

    // Return Java string
    env.new_string(result).expect("Failed to create string").into_raw()
}
```

```xml
<!-- pom.xml — Maven JNI distribution -->
<project>
    <groupId>com.fusion</groupId>
    <artifactId>fusion-native</artifactId>
    <version>1.0.0</version>
    <packaging>jar</packaging>

    <profiles>
        <profile>
            <id>linux-x86_64</id>
            <activation>
                <os>
                    <name>linux</name>
                    <arch>amd64</arch>
                </os>
            </activation>
            <dependencies>
                <dependency>
                    <groupId>com.fusion</groupId>
                    <artifactId>fusion-native-linux-x86_64</artifactId>
                    <version>${project.version}</version>
                </dependency>
            </dependencies>
        </profile>
    </profiles>
</project>
```

### Dynamic Loading with Platform Detection

```python
# dynamic_load.py — Platform-aware dynamic loading
import platform
import sys
from pathlib import Path
import ctypes

def get_platform_tag() -> str:
    """Get platform tag for library selection."""
    system = platform.system().lower()
    machine = platform.machine().lower()

    # Normalize architecture names
    arch_map = {
        "x86_64": "x86_64",
        "amd64": "x86_64",
        "aarch64": "arm64",
        "arm64": "arm64",
    }
    arch = arch_map.get(machine, machine)

    # Normalize OS names
    os_map = {
        "linux": "linux",
        "darwin": "macos",
        "windows": "windows",
    }
    os_name = os_map.get(system, system)

    return f"{os_name}-{arch}"

def load_native_library(name: str, version: str):
    """Load native library for current platform."""
    platform_tag = get_platform_tag()

    # Library file extensions
    ext_map = {
        "linux": ".so",
        "macos": ".dylib",
        "windows": ".dll",
    }
    ext = ext_map.get(platform_tag.split("-")[0], ".so")

    # Library filename patterns
    patterns = [
        f"lib{name}{ext}",
        f"lib{name}-{platform_tag}{ext}",
        f"{name}{ext}",
        f"{name}-{platform_tag}{ext}",
    ]

    # Search in standard locations
    search_paths = [
        Path(__file__).parent / "libs",
        Path(__file__).parent / "lib",
        Path("/usr/local/lib"),
        Path("/usr/lib"),
    ]

    for pattern in patterns:
        for search_path in search_paths:
            lib_path = search_path / pattern
            if lib_path.exists():
                return ctypes.CDLL(str(lib_path))

    raise OSError(
        f"Could not find native library {name} for platform {platform_tag}\n"
        f"Searched: {patterns}\n"
        f"In paths: {search_paths}"
    )

# Usage
native = load_native_library("fusion_core", "1.0.0")
result = native.process(b"input data")
```

## Binary Distribution Strategy

### Fusion.toml Binary Distribution

```toml
# Fusion.toml: binary distribution configuration
[distribution]
# Target platforms for binary distribution
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]

[distribution.python]
# Python wheel distribution
enabled = true
manylinux = "manylinux_2_28"
python_versions = ["3.9", "3.10", "3.11", "3.12"]
# Upload to PyPI
pypi_repository = "https://upload.pypi.org/legacy/"

[distribution.rust]
# Rust crate distribution
enabled = true
publish_to_crates_io = true

[distribution.go]
# Go module distribution (vendored)
enabled = true
vendor = true

[distribution.java]
# JNI library distribution
enabled = true
maven_repository = "https://repo.maven.apache.org/maven2"
artifact_group = "com.fusion"
artifact_name = "fusion-native"
```

### Release Automation

```yaml
# .github/workflows/release.yml
name: Release All Platforms

on:
  push:
    tags: ['v*']

jobs:
  build-matrix:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: fusion-linux-amd64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact: fusion-linux-arm64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: fusion-darwin-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: fusion-windows-amd64.exe
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        run: |
          rustup target add ${{ matrix.target }}
      - name: Build
        run: |
          cargo build --release --target ${{ matrix.target }}
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: target/${{ matrix.target }}/release/fusion*

  publish:
    needs: build-matrix
    runs-on: ubuntu-latest
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            fusion-linux-amd64/fusion*
            fusion-linux-arm64/fusion*
            fusion-darwin-arm64/fusion*
            fusion-windows-amd64.exe/fusion*
```

## Best Practices

1. **Use manylinux_2_28** for broad Linux compatibility
2. **Cross-compile from CI** — don't rely on native builders
3. **Test on all target platforms** — not just build
4. **Sign binaries** — GPG for Linux/macOS, Authenticode for Windows
5. **Provide checksums** — SHA-256 for verification
6. **Use semantic versioning** — users need predictable upgrades
7. **Document platform support** — be explicit about what works where
8. **Bundle dependencies** — don't require users to install system libraries

## Summary

Cross-platform binary distribution requires:
1. **Target matrix** — know what you need to build
2. **Cross-compilation** — build from a single CI environment
3. **manylinux** — for Python wheel compatibility
4. **JNI packaging** — platform-specific JARs
5. **Dynamic loading** — platform-aware library discovery
6. **Release automation** — CI builds for all targets

The goal: users install one command and it works on their platform.
