# Chapter 38: Build Cache & Incremental Compilation

When you change one Rust function, why does your Python build break? In polyglot systems, the build graph is a tangled web of cross-language dependencies, and the cache is the only thing standing between you and a 45-minute CI cycle. This chapter teaches you to tame it.

## Why Build Cache Matters More in Polyglot

In a single-language project, incremental compilation is well-understood. Rust's `cargo` has `cargo check` and `cargo build` with excellent caching. Python doesn't compile at all. But when Rust FFI functions are called from Python, changing the Rust side forces a full Python rebuild because the shared library changed. The cache becomes your bottleneck or your savior.

### The Polyglot Build Graph

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Rust Core   │────▶│  C FFI shim  │────▶│  Python     │
│  (libcore)   │     │  (libffi)    │     │  bindings   │
└─────────────┘     └──────────────┘     └─────────────┘
       │                                       │
       ▼                                       ▼
  ┌─────────┐                            ┌──────────┐
  │ Tests   │                            │ Tests    │
  └─────────┘                            └──────────┘
```

Every node in this graph is a cache boundary. When you modify `libcore`, the FFI shim must rebuild, and the Python bindings must regenerate. Without proper cache isolation, a single `cargo check` triggers a full rebuild cascade.

### Cache Invalidation Strategies

**File-hash based**: The simplest strategy. Hash every input file and rebuild only when hashes change. Tools like Bazel use this aggressively.

```python
# Fusion.toml: cache strategy configuration
[build.cache]
strategy = "content-hash"
invalidate_on = ["*.rs", "*.py", "*.toml"]
# Don't invalidate on: *.md, *.txt, LICENSE
```

**Graph-based**: Track which outputs depend on which inputs. Only rebuild downstream nodes. This is what Bazel, Buck2, and Gradle do.

```python
# Bazel BUILD file: explicit dependency graph
rust_library(
    name = "core",
    srcs = ["src/lib.rs"],
    deps = [":common"],
)

py_library(
    name = "bindings",
    srcs = ["bindings.py"],
    deps = [":core"],  # Python rebuilds when :core changes
)
```

**Timestamp-based**: Rebuild if any dependency is newer than the output. Simple but fragile — clock skew, CI environments, and Docker layer caching all break this.

**Hybrid**: Content-hash for source files, timestamp for generated files, explicit invalidation for config changes. This is the real-world sweet spot.

## Bazel & Gradle Remote Cache

### Bazel: The Polyglot Build System

Bazel was designed for polyglot builds (Google's internal system, Blaze, handles C++, Java, Go, Python, Rust in one build graph). Its remote cache is the key to fast CI.

```python
# WORKSPACE
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# Enable remote cache
# .bazelrc
build --remote_cache=grpc://cache.example.com:9092
build --remote_header=x-buildbuddy-api-key=YOUR_KEY
build --bes_results_url=https://app.buildbuddy.io/invocation/

# Cache configuration
build --disk_cache=~/.cache/bazel-disk
build --repository_cache=~/.cache/bazel-repo
```

**Remote cache hit rates in polyglot projects**:
- Same commit, different developer: ~90% cache hit
- Same developer, incremental changes: ~60-75% cache hit
- Cross-platform (linux→macos): ~40-50% cache hit

```python
# BUILD file: Rust → Python FFI
rust_binary(
    name = "native_lib",
    srcs = ["src/lib.rs"],
    crate_type = "cdylib",
)

genrule(
    name = "python_bindings",
    srcs = [":native_lib"],
    outs = ["bindings.py"],
    cmd = "$(location :generate_bindings) --input $(SRCS) --output $(OUTS)",
    tools = [":generate_bindings"],
)
```

### Gradle Remote Cache for JVM + Native

```groovy
// build.gradle
buildCache {
    local {
        enabled = true
        directory = new File(rootDir, '.gradle/build-cache')
        removeUnusedEntriesAfterDays = 30
    }
    remote(HttpBuildCache) {
        url = 'https://cache.example.com/cache/'
        credentials {
            username = project.findProperty('cacheUser') ?: ''
            password = project.findProperty('cachePass') ?: ''
        }
        push = System.getenv('CI') != null
        allowInsecureProtocol = false
    }
}
```

### Cache-Aware Task Design

The key insight: **design your build tasks so cache keys are deterministic and fine-grained**.

```python
# Bad: one big task that rebuilds everything
def build_all():
    build_rust()
    generate_python_bindings()  # Rebuilds even if Rust didn't change
    build_documentation()

# Good: fine-grained tasks with explicit dependencies
def build_rust_core():
    """Only rebuilds if src/*.rs or Cargo.toml changes."""
    cargo_build("libcore")

def generate_python_bindings():
    """Only rebuilds if FFI interface or generate.py changes."""
    run("python generate.py --ffi src/ffi.rs -o bindings.py")

def build_documentation():
    """Only rebuilds if docs/*.md changes."""
    run("mkdocs build")
```

## Separating Fast vs Slow Languages in CI

Not all languages are equal in build speed. Rust is slow to compile. Go is fast. Python barely compiles at all. Your CI pipeline should reflect this reality.

### The Speed Tiers

```
Tier 1 (Fast, <30s):    Python, JavaScript, Shell scripts
Tier 2 (Medium, 1-3min): Go, Java (incremental)
Tier 3 (Slow, 5-15min):  Rust (full build), C++ (full build)
Tier 4 (Very slow, 15min+): Rust (clean build), Native extensions
```

### CI Pipeline Design

```yaml
# .github/workflows/ci.yml
name: Polyglot CI

jobs:
  # Tier 1: Always run first, fast feedback
  lint-python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: ruff check src/
      - run: mypy src/
    timeout-minutes: 2

  lint-go:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: golangci-lint run ./...
    timeout-minutes: 3

  # Tier 2: Run after lints pass
  test-python:
    needs: [lint-python]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pytest tests/ --tb=short
    timeout-minutes: 5

  # Tier 3: Run in parallel with Tier 1-2
  build-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release
    timeout-minutes: 15

  # Tier 4: Only on merge to main
  integration:
    needs: [test-python, build-rust]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - run: make integration-test
    timeout-minutes: 30
```

### Language Isolation in Monorepos

```
myproject/
├── services/
│   ├── rust-core/          # Cargo workspace
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── python-api/         # Poetry/pip project
│   │   ├── pyproject.toml
│   │   └── src/
│   └── go-gateway/         # Go module
│       ├── go.mod
│       └── cmd/
├── shared/
│   ├── proto/              # gRPC definitions
│   └── schemas/            # JSON schemas
└── ci/
    ├── rust.Dockerfile
    ├── python.Dockerfile
    └── go.Dockerfile
```

Each language gets its own build context, its own cache, and its own CI job. Changes to `go-gateway/` don't trigger a Rust rebuild.

## Fusion.toml Build Cache Configuration

Fusion's build system tracks cross-language dependencies and manages cache invalidation automatically.

```toml
# Fusion.toml

[build]
# Enable content-hash based caching
cache_backend = "content-hash"

[build.cache.rust]
# Rust-specific cache configuration
incremental = true
# Maximum cache size (default 5GB)
max_size = "5GB"
# Cache directory
dir = ".fusion/cache/rust"
# Invalidate on these changes
watch = ["src/**/*.rs", "Cargo.toml", "Cargo.lock"]

[build.cache.python]
# Python doesn't compile, but FFI binding generation does
incremental = false
watch = ["**/*.py", "bindings.pyi", "pyproject.toml"]

[build.cache.go]
incremental = true
watch = ["**/*.go", "go.mod", "go.sum"]

[build.dependencies]
# Declare cross-language dependencies explicitly
rust → python = { trigger = "rebuild", invalidate = true }
rust → go = { trigger = "rebuild", invalidate = false }
proto → rust = { trigger = "regenerate", invalidate = true }
proto → python = { trigger = "regenerate", invalidate = true }
proto → go = { trigger = "regenerate", invalidate = true }

[build.cache.remote]
# Remote cache for CI
enabled = true
url = "grpc://cache.fusion.internal:9092"
# Allow local-only cache for development
local_only = false
```

### Cache Invalidation Rules

```toml
[build.cache.invalidation]
# Rules for when to invalidate caches
rules = [
    # Changing Cargo.toml invalidates all Rust builds
    { pattern = "Cargo.toml", invalidate = ["rust-core/**"] },

    # Changing proto files invalidates all language bindings
    { pattern = "shared/**/*.proto", invalidate = ["**"] },

    # Changing Python config only invalidates Python
    { pattern = "pyproject.toml", invalidate = ["python-api/**"] },

    # Fusion.toml changes invalidate everything
    { pattern = "Fusion.toml", invalidate = ["**"] },
]
```

### Running Cache Commands

```bash
# Check cache status
fusion cache status
# Output:
#   Rust: 142/156 targets cached (91% hit rate)
#   Python: 34/38 targets cached (89% hit rate)
#   Go: 67/70 targets cached (96% hit rate)

# Clean stale cache entries
fusion cache clean --stale 7d

# Force rebuild (ignore cache)
fusion build --no-cache

# View cache hits/misses for last build
fusion build --verbose 2>&1 | grep cache
# [cache] hit: src/lib.rs → libcore.so (hash: a1b2c3)
# [cache] miss: bindings.py (dependency changed: libcore.so)
```

## Incremental Compilation

Incremental compilation reuses results from previous builds. It's critical for developer productivity — changing one function shouldn't recompile the entire project.

### Rust Incremental Compilation

```toml
# Cargo.toml
[profile.dev]
incremental = true
# Uses .cargo/incremental/ for cache storage
# Can grow to 1-3GB per project

[profile.release]
incremental = true  # Also works in release builds
codegen-units = 256  # More parallelism for incremental builds
```

**When incremental compilation helps**:
- Small changes to function bodies: ~2-5x faster
- Adding a new function: ~1.5-3x faster
- Changing type definitions: ~1-1.5x faster (many things depend on types)

**When it doesn't help**:
- Changing public API signatures: triggers full rebuild of dependents
- Changing `mod.rs` or module structure: triggers recompilation
- Clean build: no cache to reuse

### Python FFI Binding Generation

Python doesn't compile, but generating FFI bindings from Rust interfaces does. Fusion tracks this as an incremental build step.

```python
# Generate Python bindings from Rust FFI
# This is the "compilation" step for Python FFI

import hashlib
from pathlib import Path

def should_regenerate_bindings(ffi_source: Path, bindings_output: Path) -> bool:
    """Check if bindings need regeneration based on content hash."""
    if not bindings_output.exists():
        return True

    # Hash the FFI interface (not implementation)
    ffi_hash = hashlib.sha256()
    for line in ffi_source.read_text().splitlines():
        # Only hash #[no_mangle], pub extern, and type definitions
        if any(marker in line for marker in ['#[no_mangle]', 'pub extern', 'pub type']):
            ffi_hash.update(line.encode())

    # Compare with stored hash
    hash_file = bindings_output.with_suffix('.hash')
    if not hash_file.exists():
        return True

    old_hash = hash_file.read_text().strip()
    return ffi_hash.hexdigest() != old_hash
```

### Go Incremental Builds

Go has excellent incremental compilation by default. The module cache and build cache make most builds near-instantaneous.

```bash
# Go's build cache location
go env GOCACHE
# → /home/user/.cache/go-build

# Clear cache if builds are wrong
go clean -cache

# Force rebuild of specific package
go build -a ./cmd/server

# Check what's cached
go build -v ./cmd/server 2>&1 | grep -E "^#|cached"
```

## Cache Invalidation Patterns

### The Cache Invalidation Problem

There are only two hard things in computer science: cache invalidation and naming things. In polyglot systems, cache invalidation is even harder because changes in one language can invalidate caches in another.

### The Invalidation Cascade

```
Change in: src/lib.rs
  ↓ (content hash changes)
Rust library rebuilds
  ↓ (shared library .so changes)
Python FFI bindings invalid
  ↓ (bindings regenerated)
Python tests invalid
  ↓ (tests use new bindings)
Integration tests invalid
  ↓
Full rebuild cascade: 3-15 minutes
```

### Strategies to Minimize Cascade Impact

**1. Interface-only caching**: Cache the FFI interface separately from the implementation. Only regenerate bindings when the interface changes.

```python
# .fusion/cache-key.py
def compute_ffi_interface_hash(rust_src: Path) -> str:
    """Hash only the FFI-visible interface, not implementation."""
    h = hashlib.sha256()
    for path in sorted(rust_src.rglob("*.rs")):
        content = path.read_text()
        for line in content.splitlines():
            # Only interface markers matter for cache key
            if line.strip().startswith('#[no_mangle]'):
                h.update(line.encode())
            elif 'pub extern "C"' in line:
                h.update(line.encode())
            elif line.strip().startswith('pub type '):
                h.update(line.encode())
    return h.hexdigest()
```

**2. Parallel independent builds**: Build Rust, Python, and Go in parallel when they don't depend on each other.

```yaml
# .github/workflows/parallel-build.yml
jobs:
  build:
    strategy:
      matrix:
        language: [rust, python, go]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build ${{ matrix.language }}
        run: fusion build --lang ${{ matrix.language }}
```

**3. Precomputed artifacts**: Store build artifacts in a shared location so CI can download them instead of rebuilding.

```toml
# Fusion.toml: artifact sharing
[build.artifacts]
enabled = true
backend = "s3"
bucket = "fusion-build-artifacts"
prefix = "v2.0/"
# Store Rust artifacts for 7 days
retention.rust = "7d"
# Store Python bindings for 30 days (they change less)
retention.python = "30d"
```

### Manual Cache Management

```bash
# View cache contents
fusion cache ls --lang rust
# Target: libcore.so (hash: a1b2c3, size: 4.2MB, age: 2h)
# Target: libffi.so (hash: d4e5f6, size: 1.1MB, age: 2h)

# Invalidate specific targets
fusion cache invalidate --target libcore.so

# Invalidate all targets from a specific source file
fusion cache invalidate --source src/lib.rs

# Export cache for CI
fusion cache export --output .fusion/cache.tar.gz

# Import cache from CI
fusion cache import --input .fusion/cache.tar.gz
```

## Measuring Cache Effectiveness

### Metrics That Matter

```
Cache Hit Rate:        85% (target: >80%)
Time Saved Per Build:  12.4 minutes (down from 18.2 min)
CI Cost Reduction:     $2,400/month (from $4,100)
Developer Wait Time:   3.8 min avg (down from 8.1 min)
```

### Monitoring Dashboard

```python
# .fusion/cache-monitor.py
"""Track cache hit rates over time."""
import json
from datetime import datetime

def log_cache_stats(stats: dict):
    """Append cache stats to metrics file."""
    stats["timestamp"] = datetime.utcnow().isoformat()
    with open(".fusion/cache-metrics.jsonl", "a") as f:
        f.write(json.dumps(stats) + "\n")

# Example usage in CI
log_cache_stats({
    "language": "rust",
    "total_targets": 156,
    "cache_hits": 142,
    "cache_misses": 14,
    "build_time_saved_seconds": 744,
    "remote_cache_enabled": True,
})
```

### Cache Efficiency by Language

```
Language   | Hit Rate | Avg Saved | Notes
-----------|----------|-----------|-------------------------------
Rust       | 82%      | 11.3 min  | Incremental helps most
Go         | 94%      | 2.1 min   | Fast by default
Python     | 78%      | 0.8 min   | Binding regeneration
Protobuf   | 91%      | 3.4 min   | Generated code caching
```

## Common Pitfalls

### 1. Cache Poisoning from Environment Variables

```bash
# BAD: Environment variables change build output
export CC=gcc-12  # Different from CI's gcc-13
cargo build  # Cache miss because compiler changed

# GOOD: Pin compiler in config
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
```

### 2. Over-Caching (Stale Dependencies)

```toml
# BAD: Never invalidating the cache
[build.cache]
max_age = "365d"

# GOOD: Periodic cache rotation
[build.cache]
max_age = "7d"
```

### 3. Under-Caching (Rebuilding Everything)

```yaml
# BAD: No cache configuration
steps:
  - run: cargo build
  - run: pytest

# GOOD: Explicit cache configuration
steps:
  - uses: Swatinem/rust-cache@v2
  - run: cargo build
  - uses: actions/setup-python@v5
  - run: pip install -r requirements.txt  # Cached by setup-python
  - run: pytest
```

### 4. Cache Thrashing

When cache size exceeds the limit and evicts frequently-needed entries, you get cache thrashing — worse than no cache at all.

```bash
# Monitor cache eviction rate
fusion cache stats --verbose
# [cache] eviction rate: 23% (too high)
# [recommendation] Increase max_size from 5GB to 8GB
```

## Best Practices Summary

1. **Isolate caches per language** — Rust cache shouldn't affect Python cache
2. **Use content-hash invalidation** — timestamps are unreliable in CI
3. **Track cross-language dependencies explicitly** — don't guess
4. **Separate fast and slow languages in CI** — lint Python while Rust compiles
5. **Monitor cache hit rates** — if they drop below 80%, investigate
6. **Pin toolchains** — environment differences break cache
7. **Periodic cache rotation** — stale caches waste space and cause subtle bugs
8. **Design for cacheability** — immutable inputs, deterministic outputs, explicit dependencies

Build cache is not a luxury in polyglot systems — it's the difference between a 3-minute CI cycle and a 45-minute one. Invest in it early and monitor it continuously.
