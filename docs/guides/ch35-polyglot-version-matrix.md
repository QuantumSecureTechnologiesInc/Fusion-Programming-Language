# Chapter 35: The Polyglot Version Matrix

Managing one language's dependencies is hard. Managing five languages in one project is a discipline. This chapter covers version compatibility matrices, environment management strategies, and the dark art of resolving dependency hell across language boundaries.

---

## Compatibility Matrix

### Language Version Compatibility

The Fusion language is designed to interoperate with mainstream languages via FFI and IPC. Here's what you need on each side:

| Language | Minimum Version | Recommended | Notes |
|---|---|---|---|
| Fusion | 2.0 | 2.1+ | FFI stabilized in 2.0 |
| Python | 3.9 | 3.11+ | 3.8 EOL; 3.12+ has perf gains |
| JavaScript/Node.js | 18 LTS | 20 LTS | 16 is EOL; 22 is current |
| Rust | 1.70 | 1.75+ | MSRV for most crates |
| Java | 11 (LTS) | 17 (LTS) | 21 available; 8 is legacy |
| Go | 1.20 | 1.22+ | 1.21 changed toolchain mgmt |
| C/C++ | C17 / C++20 | C20 / C++23 | For native extensions |
| Ruby | 3.1 | 3.3+ | 2.7 is EOL |
| .NET | 6.0 | 8.0 LTS | 7.0 is STS, already EOL |

### Runtime Version Requirements

Each language brings its own runtime. When FFI is involved, the native library must match:

| FFI Pair | Requirement |
|---|---|
| Fusion → Python | Python development headers (`python3-config`) must match runtime version |
| Fusion → Node.js | Node-API version must match (N-API is stable across Node 8+) |
| Fusion → C | C11+ standard library; ABI must match (glibc version on Linux) |
| Rust → C | Same ABI requirements; `cc` crate handles compilation |
| Go → C | CGO requires matching C toolchain; cross-compilation is fragile |
| Java → C (JNI) | JNI version must match JVM; classpath and native lib paths must align |

### Library Version Conflicts

The most common polyglot pain: a native library needed by two languages at different versions.

**Example:** OpenSSL

```
Fusion HTTP client → OpenSSL 3.1
Python requests     → OpenSSL 3.0 (system)
Go net/http         → BoringSSL (static)
Node.js https       → OpenSSL 3.1 (bundled)
```

When these share a process (via FFI), only one OpenSSL can be loaded. The result: segfaults, symbol conflicts, or silently wrong behavior.

**Solution matrix:**

| Scenario | Strategy |
|---|---|
| FFI in same process | One version of the native lib; static-link the rest |
| Separate processes (IPC) | Each process uses its own version freely |
| Container per language | Docker isolation eliminates conflicts entirely |
| Shared native lib | Pin to the lowest common version; test thoroughly |

### Native Dependency Versions

| Library | Fusion ecosystem | Python ecosystem | Go ecosystem |
|---|---|---|---|
| OpenSSL | 3.0+ recommended | System (varies) | BoringSSL (static) or system |
| libcurl | 7.68+ | via `pycurl` | `net/http` (native) |
| zlib | 1.2.11+ | System | Static `compress/flate` |
| SQLite | 3.35+ | Bundled in Python | `modernc.org/sqlite` |
| protobuf | prost (latest) | protobuf (latest) | google.golang.org/protobuf |

**Rule:** Always document which native libraries your project depends on and their minimum versions. A `NATIVE_DEPS.md` file saves hours of debugging.

---

## Version Management Strategies

### asdf for Multi-Language Version Management

[asdf](https://asdf-vm.com/) is a polyglot version manager. One tool, all languages.

```bash
# Install asdf
git clone https://github.com/asdf-vm/asdf.git ~/.asdf --branch v0.14.0

# Add plugins
asdf plugin add python
asdf plugin add nodejs
asdf plugin add rust
asdf plugin add golang
asdf plugin add java

# Set versions (per-project via .tool-versions)
echo "python 3.11.7" >> .tool-versions
echo "nodejs 20.11.0" >> .tool-versions
echo "rust 1.75.0" >> .tool-versions
echo "golang 1.22.0" >> .tool-versions
echo "java 17.0.9" >> .tool-versions

# Install all
asdf install

# Verify
asdf current
```

**`.tool-versions` file (project root):**

```bash
python 3.11.7
nodejs 20.11.0
rust 1.75.0
golang 1.22.0
java 17.0.9
```

**Pros:** Simple, shell-agnostic, per-project, supports all major languages.
**Cons:** No lockfile; relies on plugins being maintained; slow installs from source.

### Nix for Reproducible Environments

Nix provides hermetic, reproducible environments. Every developer gets identical versions.

```nix
# flake.nix
{
  description = "Fusion polyglot development environment";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  
  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      rustPkgs = import rust-overlay { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          # Python
          pkgs.python311
          pkgs.python311Packages.pytest
          pkgs.python311Packages.numpy
          
          # Node.js
          pkgs.nodejs_20
          pkgs.yarn
          
          # Rust
          (rustPkgs.rust-bin.stable."1.75.0".default.override {
            extensions = ["rust-src" "clippy"];
          })
          
          # Go
          pkgs.go_1_22
          
          # Java
          pkgs.jdk17
          
          # Native dependencies
          pkgs.openssl_3_1
          pkgs.zlib
          pkgs.pkg-config
          pkgs.cmake
        ];
        
        shellHook = ''
          echo "Fusion polyglot environment loaded"
          echo "Python: $(python3 --version)"
          echo "Node:   $(node --version)"
          echo "Rust:   $(rustc --version)"
          echo "Go:     $(go version)"
          echo "Java:   $(java -version 2>&1 | head -1)"
        '';
      };
    };
}
```

```bash
# Enter the environment
nix develop

# Or use direnv for automatic activation
echo "use flake" >> .envrc
direnv allow
```

**Pros:** Deterministic, reproducible, isolated, binary caches speed up installs.
**Cons:** Steep learning curve; Nix language is unusual; large disk usage for stores.

### Docker for Consistent Builds

Docker guarantees the same environment everywhere — the "works on my machine" killer.

```dockerfile
# Dockerfile for polyglot build environment
FROM rust:1.75-bookworm AS rust-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    python3.11-dev \
    nodejs npm \
    golang-go \
    openjdk-17-jdk \
    libssl-dev \
    libcurl4-openssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set up Python
RUN pip3 install --break-system-packages pytest numpy pydantic

# Set up Node.js
RUN npm install -g typescript @types/node

# Set up project
WORKDIR /app
COPY . .

# Build and test
RUN cargo build --release
RUN cargo test
RUN python3 -m pytest tests/python/
RUN node tests/js/test.js
RUN go test ./tests/go/...
```

```yaml
# docker-compose.yml
version: "3.9"

services:
  build:
    build: .
    volumes:
      - .:/app
    command: make test-all

  python-service:
    build:
      context: .
      dockerfile: Dockerfile.python
    ports:
      - "8000:8000"

  node-service:
    build:
      context: .
      dockerfile: Dockerfile.node
    ports:
      - "3000:3000"
```

**Pros:** Perfect isolation, reproducible, CI/CD native, language-agnostic.
**Cons:** Image size, layer caching complexity, learning curve for multi-stage builds.

### Version Pinning Strategies

| Strategy | Tool | When to use |
|---|---|---|
| Exact pinning | `asdf`, `.tool-versions` | Small teams, few languages |
| Lock files | `package-lock.json`, `Cargo.lock`, `go.sum` | Always — never commit without lockfiles |
| Hash pinning | Nix flakes, Home Manager | Maximum reproducibility |
| Docker image tags | `FROM python:3.11.7-slim` | Container builds |
| Version ranges (avoid) | `^1.0.0`, `>=2.0` | Only for libraries, never for apps |

**Golden rule for applications:** Pin exact versions. For libraries, use compatible ranges but test against the full range in CI.

---

## Dependency Hell Resolution

### Container Layering for Native Library Isolation

When different languages need different versions of the same native library, isolate them in separate containers and communicate via IPC.

```
┌──────────────────────┐     ┌──────────────────────┐
│  Fusion Container     │     │  Python Container     │
│  OpenSSL 3.1         │     │  OpenSSL 3.0          │
│  libcurl 7.88        │     │  requests 2.31        │
│  Fusion runtime      │     │  Python 3.11          │
└──────────┬───────────┘     └──────────┬───────────┘
           │                            │
           └──────────┬─────────────────┘
                      │
              gRPC / HTTP / Unix Socket
```

```yaml
# docker-compose.yml — isolated services
services:
  fusion-api:
    build: ./fusion-service
    environment:
      - DATABASE_URL=postgres://db:5432/app
    depends_on: [db]

  python-ml:
    build: ./python-ml-service
    environment:
      - MODEL_PATH=/models/latest
    depends_on: [db]
    # Different OpenSSL, different Python — no conflict

  gateway:
    build: ./gateway  # Fusion or Go reverse proxy
    ports: ["8080:8080"]
    depends_on: [fusion-api, python-ml]
```

### Static Linking vs Dynamic Linking Tradeoffs

| Approach | Pros | Cons |
|---|---|---|
| **Static linking** | Self-contained, no runtime surprises, reproducible | Larger binaries, slower startup, license concerns (GPL) |
| **Dynamic linking** | Smaller binaries, shared memory, hot-patching | DLL hell, version conflicts, "works on my machine" |

**Fusion-specific guidance:**

```toml
# Cargo.toml — prefer static linking for deployments
[target.'cfg(target_os = "linux")']
rustflags = ["-C", "target-feature=+crt-static"]

# Use static for distribution, dynamic for development
[profile.release]
prefer-dynamic = false
```

**When to use static:**
- CLI tools distributed as single binaries
- Embedded systems with no package manager
- Serverless functions (cold start matters)

**When to use dynamic:**
- Shared libraries loaded by multiple processes
- Systems where security patching must be fast
- Development environments where iteration speed matters

### Runtime Library Auditing

```bash
# Linux: check shared library dependencies
ldd ./fusion-binary

# Example output:
# linux-vdso.so.1 (0x00007ffd...)
# libssl.so.3 => /lib/x86_64-linux-gnu/libssl.so.3 (0x00007f...)
# libcrypto.so.3 => /lib/x86_64-linux-gnu/libcrypto.so.3
# libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
# /lib64/ld-linux-x86-64.so.2

# macOS: check dynamic library dependencies
otool -L ./fusion-binary

# Check for version mismatches
ldd ./fusion-binary | grep "=> not found"

# Find which package provides a library (Debian/Ubuntu)
dpkg -S /lib/x86_64-linux-gnu/libssl.so.3
# libssl3:amd64: /lib/x86_64-linux-gnu/libssl.so.3

# Audit for known vulnerabilities
cargo audit          # Rust
pip audit            # Python
npm audit            # Node.js
govulncheck ./...    # Go
```

```python
# Python: audit installed packages
import subprocess
result = subprocess.run(["pip", "audit", "--format", "json"], capture_output=True, text=True)
print(result.stdout)
```

```bash
# Fusion: check native dependencies at build time
# Build with dependency info
cargo build --release
# Check for symbol conflicts
nm -g ./target/release/fusion-binary | grep " U "  # undefined symbols
```

### Conflict Resolution Strategies

#### Strategy 1: Version Negotiation Protocol

When two services need the same library, negotiate versions at startup:

```fusion
// Fusion service — negotiate compatible library versions
struct LibraryVersion {
    name: String,
    min_version: String,
    max_version: String,
}

fn negotiate_version(
    required: &[LibraryVersion],
    available: &[(String, String)], // (name, version)
) -> Result<Vec<(String, String)>, VersionConflict> {
    let mut resolved = Vec::new();
    
    for req in required {
        let compat = available.iter()
            .find(|(name, ver)| name == &req.name && version_in_range(ver, &req.min_version, &req.max_version))
            .ok_or(VersionConflict::Unsatisfied(req.clone()))?;
        resolved.push(compat.clone());
    }
    
    Ok(resolved)
}
```

#### Strategy 2: Adapter Layer

Wrap incompatible versions behind a stable interface:

```go
// Go adapter for a C library with version conflicts
//
// #cgo CFLAGS: -I./compat/openssl-3.0/include
// #cgo LDFLAGS: -L./compat/openssl-3.0/lib -lssl -lcrypto
import "C"

// This Go package statically links OpenSSL 3.0,
// while the rest of the system uses 3.1
func VerifyCertificate(certPEM, caPEM []byte) (bool, error) {
    // Uses the statically-linked OpenSSL 3.0
    // ...
}
```

#### Strategy 3: Sidecar Process

Move the conflicting library to a separate process:

```
┌─────────────────┐         ┌─────────────────┐
│  Main Process    │  gRPC   │  Sidecar Process │
│  Fusion binary   │◄──────►│  Python + OpenSSL │
│  OpenSSL 3.1     │         │  3.0              │
└─────────────────┘         └─────────────────┘
```

```yaml
# Kubernetes sidecar pattern
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: fusion-api
    image: fusion-api:2.1
    ports: [{containerPort: 8080}]
  - name: python-sidecar
    image: python-ml:1.0
    ports: [{containerPort: 9090}]
  # Both containers share localhost — no version conflict
```

#### Strategy 4: Fat Binary (Multi-Version Bundling)

For critical dependencies, bundle multiple versions and select at runtime:

```fusion
// Fusion: bundle multiple native library versions
enum NativeLibVersion {
    OpenSSL3_0,
    OpenSSL3_1,
}

fn select_native_lib(version: NativeLibVersion) -> &'static Path {
    match version {
        NativeLibVersion::OpenSSL3_0 => Path::new("lib/openssl-3.0/libssl.so"),
        NativeLibVersion::OpenSSL3_1 => Path::new("lib/openssl-3.1/libssl.so"),
    }
}
```

---

## CI/CD for Polyglot Projects

### Multi-Language CI Pipeline

```yaml
# .github/workflows/polyglot-ci.yml
name: Polyglot CI

on: [push, pull_request]

jobs:
  fusion:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo clippy -- -D warnings

  python:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ["3.9", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}
      - run: pip install -r requirements.txt
      - run: pytest tests/python/

  node:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: [18, 20, 22]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node-version }}
      - run: npm ci
      - run: npm test

  go:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: "1.22"
      - run: go test ./...

  integration:
    needs: [fusion, python, node, go]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: docker compose up -d
      - run: docker compose run test-runner pytest tests/integration/
```

### Version Conflict Detection

```bash
#!/bin/bash
# scripts/check-version-conflicts.sh

echo "=== Checking native library versions ==="

# Check OpenSSL across languages
echo "System OpenSSL:"
openssl version

echo "Python OpenSSL:"
python3 -c "import ssl; print(ssl.OPENSSL_VERSION)"

echo "Node.js OpenSSL:"
node -e "console.log(process.versions.openssl)"

echo "Go crypto:"
go version -m ./binary 2>/dev/null | grep crypto

# Check for conflicts
SYSTEM_VER=$(openssl version | awk '{print $2}')
PYTHON_VER=$(python3 -c "import ssl; print(ssl.OPENSSL_VERSION)" | awk '{print $3}')
NODE_VER=$(node -e "console.log(process.versions.openssl)")

if [ "$SYSTEM_VER" != "$PYTHON_VER" ]; then
    echo "WARNING: System OpenSSL ($SYSTEM_VER) != Python OpenSSL ($PYTHON_VER)"
fi
```

---

## Quick Reference: Version Management Decision Tree

```
Need reproducible environments?
├─ Yes → Is your team familiar with Nix?
│   ├─ Yes → Nix flakes + direnv
│   └─ No → Docker Compose
└─ No → Do you use multiple languages per developer?
    ├─ Yes → asdf + .tool-versions
    └─ No → Language-native tools (pyenv, nvm, rustup)
```

---

## Summary

- Pin exact versions for applications; use compatible ranges only for libraries.
- asdf is the simplest polyglot version manager; Nix is the most reproducible.
- Docker isolates language runtimes and eliminates native library conflicts.
- Never commit lockfiles absent — they're your reproducibility guarantee.
- Audit native dependencies regularly with `ldd`, `otool`, and language-specific tools.
- When native libraries conflict, isolate via containers or sidecar processes.
- CI should test against the minimum and maximum supported versions of every language.
