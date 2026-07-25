# Chapter 24: Pillar 6 — The Developer Lifecycle & Tooling (The Assembly Line)

> How Fusion v2.0 Vortex turns individual developers into productive teams — formatting, documentation, packages, builds, debugging, testing, and observability.

---

## Introduction

A language can be theoretically perfect and practically unusable. If there is no formatter, codebases diverge into stylistic chaos. If there are no tests, regressions ship silently. If there is no profiler, performance bottlenecks become guesswork. Pillar 6 is the **assembly line** — the全套 tooling that transforms raw source code into reliable, documented, tested, and observable software.

Fusion v2.0 Vortex treats developer tooling not as an afterthought bolted onto a language, but as a first-class pillar of the language design itself. Every tool is built by the same team that builds the compiler, ships in the same toolchain, and shares the same configuration. The result is a development experience where formatting, linting, testing, profiling, and documentation generation all *just work* out of the box.

---

## Formatting & Conventions

### Official Style Guide

Fusion v2.0 Vortex has a single canonical style, enforced by the built-in formatter. The style guide is not a suggestion — it is a machine-enforced contract. If `fusion fmt` reformats your code, the reformatted version is correct.

**Core principles:**

| Principle | Rule |
|-----------|------|
| Indentation | 4 spaces, no tabs |
| Max line length | 100 characters (soft limit) |
| Brace style | K&R (opening brace on same line) |
| Naming: functions | `snake_case` |
| Naming: types | `PascalCase` |
| Naming: constants | `SCREAMING_SNAKE_CASE` |
| Naming: modules | `snake_case` |
| Naming: variables | `snake_case` |
| Naming: parameters | `snake_case` |
| Naming: lifetimes | `'snake_case` |
| Naming: traits | `PascalCase` (often `AdjectiveNoun`) |

### Auto-Formatter (`fusion fmt`)

The formatter is deterministic, fast, and idempotent — running it twice produces the same output. It respects no-comment boundaries and preserves doc comments.

```bash
# Format a single file
fusion fmt src/main.fu

# Format all files in the project
fusion fmt .

# Format and overwrite in-place
fusion fmt --write src/

# Check formatting without modifying (CI use)
fusion fmt --check .

# Format with custom config
fusion fmt --config fusion.toml src/
```

**What the formatter handles:**

- Indentation normalization
- Trailing comma insertion on multiline expressions
- Blank line normalization (one blank line between items, no trailing blanks)
- Import/use statement ordering (alphabetical within groups)
- Match arm alignment
- Function signature line breaking when exceeding line length

**What the formatter does NOT change:**

- Comments (including their position)
- Doc comments (preserves original formatting within `///` blocks)
- String literals
- Attributes and macros

### Linting (`fusion lint`)

The linter catches common mistakes and enforces project-wide conventions beyond what the formatter handles.

```bash
# Lint the entire project
fusion lint

# Lint with specific rules enabled
fusion lint --enable=unused_imports,shadowing

# Lint with warnings as errors (CI)
fusion lint --deny-warnings

# Auto-fix lint issues where possible
fusion lint --fix

# Generate lint configuration
fusion lint --init
```

**Built-in lint rules:**

| Category | Rules |
|----------|-------|
| Correctness | `unused_imports`, `unused_variables`, `dead_code`, `unreachable_code` |
| Style | `snake_case_functions`, `pascal_case_types`, `redundant_clone` |
| Safety | `unsafe_in_safe`, `unwrap_in_test_only`, `panic_in_library` |
| Performance | `unnecessary_allocation`, `repeated_field_access`, `collect_before_iter` |
| Complexity | `too_many_arguments`, `deeply_nested`, `large_enum_variant` |
| Quantum | `unmeasured_qubit`, `classical_control_leak`, `entanglement_scope` |

**Custom lint configuration (Fusion.toml):**

```toml
[lint]
deny = ["unused_imports", "dead_code"]
warn = ["snake_case_functions", "redundant_clone"]
allow = ["complex_functions"]   # suppress specific rule

[lint.per_file_overrides]
"tests/**/*.fu" = { allow = ["unwrap_in_test_only"] }
"benches/**/*.fu" = { allow = ["too_many_arguments"] }
```

### Naming Conventions

Fusion enforces naming conventions at the language level through lint warnings, not just formatter rules:

```fusion
// CORRECT
struct UserProfile {                    // PascalCase for types
    user_name: String,                  // snake_case for fields
    login_count: i64,
}

fn calculate_tax(income: f64) -> f64 {  // snake_case for functions
    let max_rate: f64 = 0.37;          // snake_case for variables
    const TAX_CEILING: f64 = 500000.0; // SCREAMING_SNAKE_CASE for constants
    income * max_rate
}

mod network::tcp {                      // snake_case for modules
    pub fn connect(host: &str) -> Connection { ... }
}

trait Serializable { }                  // PascalCase for traits
```

### Code Organization Rules

The `fusion lint` tool enforces structural conventions:

```fusion
// File: src/lib.rs equivalent — src/lib.fu
// 1. Module declarations (top of file)
mod config;
mod network;
mod models;

// 2. Standard library imports
use std::collections::HashMap;
use std::io::{self, Read, Write};

// 3. Crate-level imports
use crate::config::Settings;
use crate::models::User;

// 4. Constants
const MAX_CONNECTIONS: usize = 100;

// 5. Type definitions
pub struct Server {
    addr: String,
    connections: Vec<Connection>,
}

// 6. Implementation blocks
impl Server {
    pub fn new(addr: String) -> Self { ... }
    pub fn listen(&self) -> Result<(), Error> { ... }
}

// 7. Free functions
fn parse_header(data: &[u8]) -> Result<Header, Error> { ... }

// 8. Tests (at bottom of file)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() { ... }
}
```

---

## Built-in Documentation

### Doc Comment Syntax

Fusion supports two doc comment styles:

**Triple-slash (preferred for single items):**

```fusion
/// Compute the factorial of a non-negative integer.
///
/// # Arguments
///
/// * `n` - A non-negative integer. Values above 20 may overflow for `i32`.
///
/// # Returns
///
/// The factorial of `n` as an `i64`.
///
/// # Examples
///
/// ```
/// let result = factorial(5);
/// assert(result == 120);
/// ```
///
/// # Panics
///
/// This function panics if `n` is negative.
fn factorial(n: i32) -> i64 {
    assert(n >= 0, "factorial requires non-negative input");
    if n <= 1 { 1 } else { n as i64 * factorial(n - 1) }
}
```

**Block doc comments (preferred for modules and complex items):**

```fusion
/**
 * The network module provides TCP and UDP networking primitives.
 *
 * This module is the foundation for all network communication in
 * Fusion applications. It wraps platform-specific socket APIs
 * behind a safe, cross-platform interface.
 *
 * # Quick Start
 *
 * ```
 * use network::tcp;
 *
 * let conn = tcp::connect("127.0.0.1:8080")?;
 * conn.send(b"Hello, server!")?;
 * ```
 */
mod network { ... }
```

### Documentation Generation (`fusion doc`)

```bash
# Generate HTML documentation for the project
fusion doc

# Generate docs and open in browser
fusion doc --open

# Generate docs for a specific package
fusion doc --package my-lib

# Generate docs including private items
fusion doc --document-private

# Generate docs with custom output directory
fusion doc --output ./target/docs

# Serve docs locally for preview
fusion doc --serve --port 8080
```

**Documentation output format:**

```
target/doc/
├── index.html           # Crate root documentation
├── all-items.html       # Alphabetical index
├── struct/
│   └── UserProfile.html
├── enum/
│   └── Error.html
├── fn/
│   └── factorial.html
├── trait/
│   └── Serializable.html
├── module/
│   ├── network.html
│   └── network/
│       ├── tcp.html
│       └── udp.html
└── src/
    └── lib.fu.html      # Source with doc links
```

### API Documentation Format

Fusion enforces a structured documentation format for public API items:

```fusion
/// Short one-line summary ending with a period.
///
/// Extended description (optional). Can span multiple paragraphs.
/// Use markdown for formatting: **bold**, *italic*, `code`.
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of the return value.
///
/// # Errors
///
/// Returns `Err(Error::NotFound)` if the resource does not exist.
/// Returns `Err(Error::PermissionDenied)` if access is denied.
///
/// # Panics
///
/// Panics if `value` is negative.
///
/// # Safety
///
/// The caller must ensure `ptr` points to valid memory of at least
/// `len` bytes.
///
/// # Examples
///
/// ```
/// let result = process(&data, 42);
/// assert(result.is_ok());
/// ```
///
/// # See Also
///
/// * [`related_function`] - A related operation
/// * [`SimilarStruct`] - A related type
fn process(data: &[u8], value: i64) -> Result<Vec<u8>, Error> { ... }
```

### Code Examples in Docs

Doc comments can contain runnable code examples. The test harness compiles and executes them:

```fusion
/// Adds two numbers together.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// let result = add(2, 3);
/// assert(result == 5);
/// ```
///
/// With variables:
///
/// ```
/// let a = 10;
/// let b = 20;
/// let sum = add(a, b);
/// assert(sum == 30);
/// ```
///
/// Compile-time error (won't compile):
///
/// ```compile_fail
/// let x: String = add(1, 2);  // Error: type mismatch
/// ```
///
/// Hidden lines (not shown in rendered docs):
///
/// ```
/// # // Setup code hidden from reader
/// # let config = load_config();
/// let result = process(&config);
/// # assert(result.is_ok());
/// ```
fn add(a: i32, b: i32) -> i32 { a + b }
```

### Cross-References

Doc comments support rich cross-references:

```fusion
/// Compute the hash of `data` using the algorithm from [`HashAlgorithm::Sha3`].
///
/// For the legacy algorithm, see [`HashAlgorithm::Sha2`].
/// For streaming computation, see [`StreamingHash`].
///
/// This function delegates to [`hash_internal`] which performs the
/// actual computation. See [`hash_benchmarks`] for performance data.
///
/// [`HashAlgorithm::Sha3`]: enum.HashAlgorithm.html#variant.Sha3
/// [`HashAlgorithm::Sha2`]: enum.HashAlgorithm.html#variant.Sha2
/// [`hash_internal`]: fn.hash_internal.html
/// [`hash_benchmarks`]: https://benchmarks.example.com/hash
fn hash(data: &[u8]) -> Hash { ... }
```

---

## Package Manager

### Fusion.toml Configuration

Every Fusion project is defined by a `Fusion.toml` at the root:

```toml
[package]
name = "my-application"
version = "0.5.0"
edition = "2026"
authors = ["Alice <alice@example.com>"]
description = "A demonstration application"
license = "MIT OR Apache-2.0"
repository = "https://github.com/alice/my-application"
homepage = "https://example.com"
documentation = "https://docs.example.com"
readme = "README.md"
keywords = ["web", "async", "demo"]
categories = ["web-programming", "asynchronous"]

[dependencies]
forge-std = "1.0"                    # Latest compatible version
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
quantum-sim = { git = "https://github.com/fusion-lang/quantum-sim", branch = "main" }
local-crate = { path = "../local-crate" }

[dev-dependencies]
assert_cmd = "2.0"
tempfile = "3.0"

[build-dependencies]
cc = "1.0"

[workspace]
members = ["crates/*"]
exclude = ["old-crates/*"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0
debug = true

[profile.bench]
opt-level = 3
debug = true

[features]
default = ["json", "logging"]
json = ["serde", "serde_json"]
logging = ["tracing"]
quantum = ["quantum-sim"]
gpu = ["cuda-sys"]
full = ["json", "logging", "quantum", "gpu"]
```

### Forge Commands

Forge is Fusion's package manager, analogous to Cargo for Rust:

```bash
# Create a new project
forge new my-project
forge new my-project --lib           # Library crate
forge new my-project --bin my-app    # Binary crate with custom name

# Build
forge build                          # Debug build
forge build --release                # Optimized release build
forge build --target wasm32          # WASM target
forge build --features quantum       # Enable specific features

# Run
forge run                            # Build and run
forge run -- arg1 arg2               # Pass arguments
forge run --release                  # Optimized run

# Test
forge test                           # Run all tests
forge test -- --filter=unit          # Run only unit tests
forge test -- --nocapture            # Show println! output
forge test -- --test-threads=4       # Parallel test execution

# Documentation
forge doc                            # Generate documentation
forge doc --open                     # Generate and open in browser

# Dependencies
forge add serde                      # Add latest version
forge add serde@1.0                  # Add specific version
forge add serde --features derive    # Add with features
forge remove serde                   # Remove dependency
forge update                         # Update all dependencies
forge update serde                   # Update specific dependency
forge tree                           # Show dependency tree
forge audit                          # Check for known vulnerabilities

# Publishing
forge login                          # Authenticate with registry
forge publish                        # Publish to registry
forge publish --dry-run              # Verify without publishing
forge yank 1.0.0                     # Yank a version

# Workspace
forge workspace add my-crate         # Add member to workspace
forge workspace list                 # List workspace members

# Benchmarks
forge bench                           # Run all benchmarks
forge bench -- --filter=hash         # Run specific benchmark

# Clean
forge clean                           # Remove build artifacts
forge clean --all                    # Remove target/ and caches
```

### Dependency Resolution

Forge uses a SAT-solver-based dependency resolver that guarantees reproducible builds:

```bash
# Show resolved dependency graph
forge tree

# Output:
# my-application v0.5.0
# ├── serde v1.0.193
# │   └── serde_derive v1.0.193 (proc-macro)
# │       └── syn v2.0.39
# │           ├── quote v1.0.33
# │           └── proc-macro2 v1.0.70
# ├── tokio v1.35.0
# │   ├── bytes v1.5.0
# │   ├── pin-project-lite v0.2.13
# │   └── parking_lot v0.12.1
# └── quantum-sim v0.3.1 (git+https://github.com/fusion-lang/quantum-sim)
#     └── num-complex v0.4.4

# Lock file ensures reproducibility
forge lock                            # Generate/update lock file
forge install                         # Install from lock file (CI)
```

**Conflict resolution strategies:**

| Strategy | Behavior |
|----------|----------|
| SemVer-compatible | Default. Picks highest compatible version |
| Exact match | `serde = "=1.0.193"` pins to exact version |
| Minimum | `serde = ">=1.0, <2.0"` range constraint |
| Prefer newer | Default tiebreaker when ranges overlap |
| Yanked versions | Never selected automatically |

### Version Management

Forge follows Semantic Versioning with Fusion-specific extensions:

```toml
# SemVer range syntax
[dependencies]
serde = "1.0"            # Compatible: >=1.0.0, <2.0.0
serde = "^1.0.3"         # Caret: >=1.0.3, <2.0.0
serde = "~1.0.3"         # Tilde: >=1.0.3, <1.1.0
serde = ">=1.0, <1.5"    # Range: explicit bounds
serde = "=1.0.193"       # Exact: pinned version

# Pre-release versions
my-lib = "0.5.0-beta.1"
my-lib = "0.5.0-rc.1"

# Edition-based compatibility
[package]
edition = "2026"          # Language edition compatibility
```

**Version compatibility matrix:**

| Change Type | Version Bump | Example |
|-------------|-------------|---------|
| Bug fix | Patch (0.0.x) | 1.0.0 → 1.0.1 |
| New feature (backward-compatible) | Minor (0.x.0) | 1.0.0 → 1.1.0 |
| Breaking API change | Major (x.0.0) | 1.0.0 → 2.0.0 |
| Language edition change | Major or standalone | edition = "2026" |

### Cross-Language Package Support

Forge supports packages written in other languages via FFI wrappers:

```toml
[dependencies]
# Native Fusion crate
my-lib = "1.0"

# C library via FFI binding
openssl-sys = { version = "0.9", lang = "c" }

# Python extension via PyO3-style binding
python-bridge = { version = "0.1", lang = "python" }

# WASM module (imported at runtime)
wasm-plugin = { version = "0.2", lang = "wasm" }

# Go library via CGo-style binding
go-bridge = { version = "0.3", lang = "go" }
```

```bash
# Build cross-language dependencies
forge build --fetch-native

# Verify FFI compatibility
forge verify-ffi

# Generate FFI bindings
forge ffi generate my-c-dep
```

---

## Build System

### Compilation Pipeline

The Fusion build system orchestrates the full compilation pipeline:

```
Source Files (.fu)
    │
    ▼
┌──────────────────┐
│   Dependency     │  Resolve & download crates from registry
│   Resolution     │
└────────┬─────────┘
         ▼
┌──────────────────┐
│   Macro          │  Expand procedural and declarative macros
│   Expansion      │
└────────┬─────────┘
         ▼
┌──────────────────┐
│   Lexing &       │  Tokenize source, parse into AST
│   Parsing        │
└────────┬─────────┘
         ▼
┌──────────────────┐
│   Semantic       │  Type check, borrow check, name resolution
│   Analysis       │
└────────┬─────────┘
         ▼
┌──────────────────┐
│   Optimization   │  LLVM optimization passes (release mode)
│   (optional)     │
└────────┬─────────┘
         ▼
┌──────────────────┐
│   Code           │  Generate target-specific code
│   Generation     │
└────────┬─────────┘
         ▼
┌──────────────────┐
│   Linking        │  Link libraries, produce final binary
│                  │
└──────────────────┘
```

```bash
# Full build pipeline
forge build

# Verbose output showing each phase
forge build --verbose

# Build with specific number of parallel jobs
forge build --jobs 8

# Dry-run (show what would be compiled)
forge build --dry-run
```

### Build Targets

Fusion supports multiple compilation targets:

```bash
# Native (current platform)
forge build --target native

# WebAssembly
forge build --target wasm32-unknown-unknown
forge build --target wasm32-wasi

# Embedded targets
forge build --target thumbv7em-none-eabihf   # ARM Cortex-M7
forge build --target riscv64gc-unknown-none   # RISC-V

# Cross-compilation
forge build --target aarch64-unknown-linux-gnu  # ARM64 Linux
forge build --target x86_64-pc-windows-msvc    # Windows x64

# List available targets
forge build --print-targets
```

**Target-specific configuration:**

```toml
# Fusion.toml
[target.'cfg(target_os = "linux")']
 linker = "clang"
 rustflags = ["-C", "target-cpu=native"]

[target.'cfg(target_os = "windows")']
 linker = "link.exe"

[target.'cfg(target_arch = "wasm32")']
 runner = "wasmtime"
```

### Optimization Levels

```toml
# Fusion.toml
[profile.dev]
opt-level = 0          # No optimization (fast compile)
debug = true           # Full debug symbols
incremental = true     # Incremental compilation

[profile.release]
opt-level = 3          # Maximum optimization
lto = "fat"            # Full link-time optimization
codegen-units = 1      # Single codegen unit (slower build, better optimization)
panic = "abort"        # Abort on panic (smaller binary)
strip = "symbols"      # Strip debug symbols

[profile.bench]
opt-level = 3          # Full optimization
debug = true           # Keep debug symbols for profiling
lto = "thin"           # Thin LTO (faster than fat)

[profile.test]
opt-level = 0          # Fast compilation for tests
debug = true           # Debug symbols for test output
```

**Command-line override:**

```bash
# Override optimization level
forge build --release --opt-level=2

# Override LTO
forge build --release --lto=off

# Build with specific target CPU features
forge build --release --target-cpu=haswell
```

### Feature Flags

Features enable conditional compilation across the dependency graph:

```toml
# In your Fusion.toml
[features]
default = ["std"]                    # Enabled by default
std = ["forge-std/std"]             # Standard library
no_std = []                          # No standard library (embedded)
json = ["serde", "serde_json"]      # Enable JSON support
quantum = ["quantum-sim"]           # Enable quantum simulation
gpu = ["cuda-sys", "gpu-kernels"]   # Enable GPU acceleration
unstable = []                        # Unstable APIs (nightly only)

[dependencies]
serde = { version = "1.0", optional = true }
serde_json = { version = "1.0", optional = true }
quantum-sim = { version = "0.3", optional = true }
cuda-sys = { version = "0.1", optional = true }
```

```rust
// In source code — conditional compilation
#[cfg(feature = "json")]
pub fn parse_json(input: &str) -> Result<Value, Error> { ... }

#[cfg(feature = "quantum")]
pub fn simulate(circuit: &QuantumCircuit) -> StateVector { ... }

#[cfg(not(feature = "std"))]
use core::alloc::{GlobalAlloc, Layout};

// Compile-time feature assertions
#[cfg(not(any(feature = "json", feature = "quantum")))]
compile_error!("Enable at least one of 'json' or 'quantum' features");
```

```bash
# Build with specific features
forge build --features "json,quantum"

# Build with all features
forge build --all-features

# Build with no default features
forge build --no-default-features

# List available features
forge build --print-features
```

### Incremental Compilation

Fusion caches intermediate compilation artifacts to speed up rebuilds:

```bash
# Incremental compilation is enabled by default in dev builds
forge build                           # Uses incremental cache

# Full clean rebuild
forge clean && forge build

# Disable incremental (useful for CI)
forge build --no-incremental

# Show incremental compilation stats
forge build --timings

# Output:
#   Item  Compiling  Optimizing  Codegen  Linking  Total
#   my-app  2.3s      0.0s       0.8s     0.4s     3.5s
#   (incremental: reused 42 items, recomputed 3 items)
```

### Cross-Compilation

```bash
# Install a cross-compilation toolchain
forge target add aarch64-unknown-linux-gnu

# Build for cross-compilation target
forge build --target aarch64-unknown-linux-gnu

# Use a cross-compilation linker
forge build --target aarch64-unknown-linux-gnu \
    --linker aarch64-linux-gnu-gcc

# Cross-compile with QEMU runner for testing
forge test --target aarch64-unknown-linux-gnu \
    --runner "qemu-aarch64-static"

# Docker-based cross-compilation
forge build --target x86_64-unknown-linux-musl \
    --cross-container "rustembedded/cross:x86_64-unknown-linux-musl"
```

---

## Debugging & Profiling

### Stack Traces on Crashes

Fusion produces rich stack traces on crashes by default:

```bash
# Run with debug symbols (default in dev builds)
forge run

# On panic or crash, output:
# thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5'
# stack backtrace:
#    0: 0x000055a1b2c3d4e5 - my_app::process_data at src/utils.fu:42
#    1: 0x000055a1b2c3d5a0 - my_app::main at src/main.fu:15
#    2: 0x000055a1b2c3d6b5 - fusion_rt::start at runtime/start.fu:100
#    3: 0x00007f8a1b2c3d4e - __libc_start_main
```

**Enhanced stack traces:**

```toml
# Fusion.toml
[debug]
backtrace = "full"          # Full backtraces on panic
color = "auto"              # Colorized output
pretty = true               # Human-readable formatting
```

```bash
# Environment variable overrides
RUST_BACKTRACE=1 forge run               # Full backtrace
RUST_BACKTRACE=full forge run            # Extra verbose backtrace
FUSION_COLOR=always forge run            # Force color output
```

### Debug Symbols

```bash
# Debug build (includes all symbols)
forge build

# Release build with debug symbols (for profiling)
forge build --release --debug-symbols

# Strip debug symbols from release binary
forge build --release --strip symbols

# Generate separate debug info file
forge build --release --debug-info split

# Inspect debug info
fusion-dbg info target/release/my-app
```

### External Profiler Hooks

Fusion exports profiling hooks compatible with standard tools:

```bash
# Linux perf
forge build --release
perf record -g ./target/release/my-app
perf report

# Valgrind (callgrind)
valgrind --tool=callgrind ./target/release/my-app
callgrind_annotate callgrind.out.*

# Intel VTune
vtune -collect hot ./target/release/my-app

# macOS Instruments
instruments -t "Time Profiler" ./target/release/my-app

# Google Performance Tools (gperftools)
CPUPROFILE=profile.out ./target/release/my-app
pprof ./target/release/my-app profile.out
```

### Built-in Profiler (`fusion profile`)

Fusion ships with a built-in sampling profiler:

```bash
# Profile a build/run
fusion profile forge run

# Profile with specific options
fusion profile --duration 30s --frequency 997Hz forge run --release

# Profile specific function
fusion profile --filter "process_data" forge run

# Generate flamegraph
fusion profile --flamegraph forge run

# Output: target/profile/flamegraph.svg

# Generate call tree
fusion profile --call-tree forge run

# Compare two profiles
fusion profile --baseline baseline.prof --current current.prof

# Export to common format
fusion profile --format pprof forge run
fusion profile --format json forge run
fusion profile --format chrome-trace forge run
```

**Profiler output example:**

```
Profiling forge run (30.0s, 997Hz)
──────────────────────────────────────────────────
Function                  Self Time    Total Time    Calls
──────────────────────────────────────────────────
process_data              12.3s (41%)  15.2s (51%)   1,247
  serialize               2.1s  (7%)   2.1s  (7%)    1,247
  hash                    0.8s  (3%)   0.8s  (3%)    1,247
main                       0.2s  (1%)  30.0s (100%)      1
network::send              5.4s (18%)   5.4s (18%)     892
──────────────────────────────────────────────────
Total:                    30.0s
```

### Memory Profiling

```bash
# Memory profiling with built-in tool
fusion profile --memory forge run

# Track allocations
fusion profile --memory --track-alloc forge run

# Detect memory leaks
fusion profile --leaks forge run

# Output:
# Memory Profile Summary
# ──────────────────────
# Peak RSS:        45.2 MB
# Total Allocs:    12,847
# Total Bytes:     234.5 MB
# Live at Exit:    44.8 MB (0.4 MB leaked)
#
# Top Allocations:
#   src/data.fu:42     12,345 allocations  180.2 MB  (Vec::resize)
#   src/net.fu:88        1,234 allocations   45.1 MB  (Buffer::new)
```

### CPU Profiling

```bash
# Quick CPU profile
fusion profile --cpu forge run --release

# With source annotation
fusion profile --cpu --annotate forge run --release

# Per-thread profiling
fusion profile --cpu --per-thread forge run --release

# Sampling configuration
fusion profile --cpu \
    --frequency 1000 \
    --stack-depth 64 \
    --duration 60s \
    forge run --release
```

---

## Testing Framework

### Unit Tests (`#[test]`)

Tests are defined inline using the `#[test]` attribute:

```fusion
// In src/math.fu
pub fn add(a: i32, b: i32) -> i32 { a + b }

pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        assert(add(2, 3) == 5);
    }

    #[test]
    fn test_add_negative() {
        assert(add(-1, -2) == -3);
    }

    #[test]
    fn test_add_zero() {
        assert(add(0, 42) == 42);
    }

    #[test]
    fn test_divide_normal() {
        let result = divide(10.0, 2.0);
        assert(result.is_ok());
        assert(result.unwrap() == 5.0);
    }

    #[test]
    fn test_divide_by_zero() {
        let result = divide(10.0, 0.0);
        assert(result.is_err());
        assert(result.unwrap_err() == "division by zero");
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_overflow_panics() {
        let x: i32 = i32::MAX;
        let _ = x + 1;  // panics in debug mode
    }

    #[test]
    #[ignore]  // Skip this test
    fn expensive_test() {
        // Long-running test that's skipped by default
        assert(true);
    }
}
```

### Integration Tests

Integration tests live in the `tests/` directory and test the public API:

```fusion
// tests/integration_test.fu
use my_library::{add, divide, Config};

#[test]
fn test_end_to_end() {
    let config = Config::default();
    let result = my_library::process(&config, "input.txt");
    assert(result.is_ok());
    assert(result.unwrap().len() > 0);
}

#[test]
fn test_with_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    std::fs::write(&path, "hello world").unwrap();

    let result = my_library::read_and_process(&path);
    assert(result.is_ok());
}

#[test]
fn test_concurrent_access() {
    let shared = Arc::new(Mutex::new(Vec::new()));
    let handles: Vec<_> = (0..10).map(|i| {
        let data = shared.clone();
        spawn(move || {
            data.lock().push(i);
        })
    }).collect();

    for h in handles { h.join().unwrap(); }
    assert(shared.lock().len() == 10);
}
```

**Test file organization:**

```
my-project/
├── src/
│   ├── lib.fu           # Library root
│   ├── math.fu          # Unit tests inline
│   └── net.fu           # Unit tests inline
├── tests/               # Integration tests
│   ├── integration.fu   # General integration tests
│   ├── api_test.fu      # API contract tests
│   └── common/
│       └── helpers.fu   # Shared test utilities
├── benches/             # Benchmarks
│   └── math_bench.fu
└── examples/            # Example programs
    └── hello.fu
```

### Benchmarks

```fusion
// benches/math_bench.fu
use test::benchmark;

#[bench]
fn bench_add(b: &mut Bencher) {
    b.iter(|| {
        for i in 0..1000 {
            add(i, i + 1);
        }
    });
}

#[bench]
fn bench_divide(b: &mut Bencher) {
    b.iter(|| {
        for i in 1..1000 {
            divide(i as f64, 3.14);
        }
    });
}

#[bench]
fn bench_alloc(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::new();
        for i in 0..10000 {
            v.push(i);
        }
        v
    });
    b.bytes = 10000 * std::mem::size_of::<i64>();
}
```

```bash
# Run benchmarks
forge bench

# Run specific benchmark
forge bench --bench math_bench

# Benchmark with custom settings
forge bench --warm-up 10 --iterations 1000

# Compare benchmarks
forge bench --baseline main
forge bench --compare
```

### Test Organization

**Test categories:**

| Category | Location | Purpose |
|----------|----------|---------|
| Unit tests | `#[cfg(test)]` in source files | Test individual functions/modules |
| Integration tests | `tests/*.fu` | Test public API surface |
| Doc tests | `///` code examples | Verify documentation examples |
| Benchmarks | `benches/*.fu` | Performance regression detection |
| Examples | `examples/*.fu` | Usage demonstrations |

**Test attributes:**

```fusion
#[test]                     // Basic test
#[test] #[ignore]           // Skipped by default
#[test] #[should_panic]     // Expected panic
#[test] #[should_panic(expected = "msg")]  // Expected panic with message
#[cfg(test)]                // Compile only in test mode
#[cfg(feature = "json")]    // Compile only with feature
```

### Test Running (`forge test`)

```bash
# Run all tests
forge test

# Run with output shown
forge test -- --nocapture

# Run specific test by name
forge test -- --filter test_add_positive

# Run tests matching pattern
forge test -- --filter "net::"

# Run tests in parallel (default)
forge test -- --test-threads 8

# Run tests sequentially
forge test -- --test-threads 1

# Run only unit tests
forge test --lib

# Run only integration tests
forge test --test integration_test

# Run tests with specific features
forge test --features "json,logging"

# List all tests without running
forge test -- --list

# Run tests and show compilation time
forge test --timings
```

### Coverage Reporting

```bash
# Generate test coverage report
forge coverage

# Generate HTML coverage report
forge coverage --html --open

# Generate lcov format (for CI)
forge coverage --lcov --output coverage.lcov

# Coverage with minimum threshold
forge coverage --minimum 80.0

# Per-file coverage
forge coverage --by-file

# Coverage for specific module
forge coverage --include "src/math.fu"

# Output:
# | File              | Lines   | Branches | Functions |
# |-------------------|---------|----------|-----------|
# | src/math.fu       | 95.2%   | 92.1%    | 100.0%    |
# | src/net.fu        | 87.3%   | 81.4%    | 90.0%     |
# | src/config.fu     | 78.1%   | 70.0%    | 85.0%     |
# |-------------------|---------|----------|-----------|
# | Total             | 88.5%   | 82.3%    | 92.1%     |
```

---

## Observability

### Structured Logging

Fusion provides structured logging via the `log` crate integration:

```fusion
use log::{trace, debug, info, warn, error};
use tracing_subscriber;

fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()                          // JSON output for log aggregators
        .init();

    info!(user_id = 42, action = "login", "User logged in");
    debug!(duration_ms = 23, "Request processed");
    warn!(retry_count = 3, "Retrying connection");
    error!(error = "timeout", "Failed to connect to database");
}
```

**Log levels:**

| Level | Purpose | Example |
|-------|---------|---------|
| `TRACE` | Extremely detailed, often noisy | Variable values in a loop |
| `DEBUG` | Debugging information | Function entry/exit, intermediate results |
| `INFO` | General application events | Server started, request processed |
| `WARN` | Unexpected but recoverable | Deprecated API used, retry attempt |
| `ERROR` | Failures requiring attention | Connection lost, operation failed |

### Metrics Collection

```fusion
use metrics::{counter, gauge, histogram};

fn process_request(req: &Request) -> Response {
    counter!("requests_total", "method" => req.method.clone()).increment(1);

    let timer = histogram!("request_duration_ms").measure();

    let response = handle(req);

    drop(timer);  // Records elapsed time

    gauge!("active_connections").increment(1.0);

    counter!("bytes_sent", "status" => response.status.to_string())
        .increment(response.body.len() as u64);

    response
}

fn main() {
    // Export metrics in Prometheus format
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener("0.0.0.0:9090")
        .install()
        .unwrap();
}
```

**Available metric types:**

| Type | Use Case | Example |
|------|----------|---------|
| `counter` | Monotonically increasing values | Requests processed, bytes sent |
| `gauge` | Values that go up and down | Active connections, queue size |
| `histogram` | Distribution of values | Request duration, response size |

### Distributed Tracing

```fusion
use tracing::{info_span, instrument};

#[instrument(skip(config), fields(request_id = %req.id))]
fn process_request(req: &Request, config: &Config) -> Response {
    let _span = info_span!("processing", phase = "validation").entered();

    validate(req)?;

    let _span = info_span!("processing", phase = "execution").entered();

    execute(req, config)
}

// Nested spans automatically create parent-child relationships
#[instrument]
fn authenticate(token: &str) -> Result<Session, AuthError> {
    let _span = info_span!("token_validation").entered();
    validate_token(token)?;

    let _span = info_span!("session_creation").entered();
    Session::new(token)
}
```

### OpenTelemetry Compatibility

```fusion
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;

fn init_telemetry() {
    let provider = TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_resource(opentelemetry::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "my-service"),
            opentelemetry::KeyValue::new("service.version", "0.5.0"),
        ]))
        .build();

    global::set_tracer_provider(provider);
}

// Traces are exported in OTLP format
// Compatible with Jaeger, Zipkin, Grafana Tempo, etc.
```

### Performance Monitoring

```bash
# Enable runtime performance monitoring
forge run --release --with-monitoring

# Monitor system metrics
fusion monitor --duration 60s

# Output:
# System Metrics (60s window)
# ───────────────────────────
# CPU Usage:        45.2% avg, 87.3% peak
# Memory Usage:     128.5 MB avg, 142.1 MB peak
# GC Pauses:        0 (no garbage collector)
# Thread Count:     8 active, 12 spawned
# IO Operations:    1,247 reads, 892 writes
# Network:          2.3 MB/s in, 1.8 MB/s out
# Open Files:       23
# Uptime:           60.0s
```

---

## Code Examples

### Complete Testing Example

```fusion
// src/lib.fu
pub struct Calculator {
    history: Vec<(String, f64)>,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator { history: Vec::new() }
    }

    pub fn add(&mut self, a: f64, b: f64) -> f64 {
        let result = a + b;
        self.history.push(("add".to_string(), result));
        result
    }

    pub fn divide(&mut self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            return Err("division by zero".to_string());
        }
        let result = a / b;
        self.history.push(("divide".to_string(), result));
        Ok(result)
    }

    pub fn last_result(&self) -> Option<f64> {
        self.history.last().map(|(_, v)| *v)
    }

    pub fn history(&self) -> &[(String, f64)] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_calculator_is_empty() {
        let calc = Calculator::new();
        assert(calc.history().len() == 0);
        assert(calc.last_result().is_none());
    }

    #[test]
    fn test_add() {
        let mut calc = Calculator::new();
        let result = calc.add(2.0, 3.0);
        assert(result == 5.0);
        assert(calc.last_result() == Some(5.0));
    }

    #[test]
    fn test_divide() {
        let mut calc = Calculator::new();
        let result = calc.divide(10.0, 3.0);
        assert(result.is_ok());
        assert((result.unwrap() - 3.3333333).abs() < 0.0001);
    }

    #[test]
    fn test_divide_by_zero() {
        let mut calc = Calculator::new();
        let result = calc.divide(10.0, 0.0);
        assert(result.is_err());
    }

    #[test]
    fn test_history_tracking() {
        let mut calc = Calculator::new();
        calc.add(1.0, 2.0);
        calc.divide(6.0, 3.0);
        calc.add(4.0, 5.0);

        assert(calc.history().len() == 3);
        assert(calc.history()[0].0 == "add");
        assert(calc.history()[1].0 == "divide");
        assert(calc.history()[2].1 == 9.0);
    }
}

// tests/calculator_integration.fu
use my_lib::Calculator;

#[test]
fn test_calculator_workflow() {
    let mut calc = Calculator::new();

    // Simulate a user workflow
    let sum = calc.add(100.0, 200.0);
    assert(sum == 300.0);

    let quotient = calc.divide(sum, 3.0);
    assert(quotient.is_ok());
    assert(quotient.unwrap() == 100.0);

    // Verify history
    assert(calc.history().len() == 2);
    assert(calc.last_result() == Some(100.0));
}

#[test]
fn test_concurrent_calculators() {
    use std::thread::spawn;
    use std::sync::{Arc, Mutex};

    let calc = Arc::new(Mutex::new(Calculator::new()));
    let handles: Vec<_> = (0..10).map(|i| {
        let c = calc.clone();
        spawn(move || {
            let mut calc = c.lock();
            calc.add(i as f64, 1.0);
        })
    }).collect();

    for h in handles { h.join().unwrap(); }
    assert(calc.lock().history().len() == 10);
}
```

### Profiling Workflow

```bash
# Step 1: Build with debug symbols in release mode
forge build --release --debug-symbols

# Step 2: Run baseline profile
fusion profile --cpu --flamegraph forge run --release
# → target/profile/baseline/flamegraph.svg

# Step 3: Identify hot function (from flamegraph)
# Found: process_data takes 41% of CPU time

# Step 4: Optimize the hot function
# (edit src/data.fu)

# Step 5: Run comparison profile
fusion profile --cpu --flamegraph forge run --release
# → target/profile/optimized/flamegraph.svg

# Step 6: Compare profiles
fusion profile --compare \
    --baseline target/profile/baseline/profile.prof \
    --current target/profile/optimized/profile.prof

# Output:
# Profile Comparison
# ──────────────────
# Baseline:  30.0s total, process_data: 12.3s (41%)
# Optimized: 18.5s total, process_data:  3.1s (17%)
# Improvement: 38.3% faster overall
#
# Hot function improvements:
#   process_data:  12.3s → 3.1s  (-74.8%)
#   serialize:      2.1s → 1.8s  (-14.3%)
#   hash:           0.8s → 0.8s  (no change)
```

### Documentation Generation

```bash
# Step 1: Write documentation
# (add doc comments to all public items)

# Step 2: Generate documentation
forge doc

# Step 3: Open in browser
forge doc --open

# Step 4: Check for documentation warnings
forge doc --check
# Output:
# warning: missing documentation for `internal_helper` in module `utils`
# warning: missing code example for `process` in struct `DataProcessor`

# Step 5: Generate documentation with private items (for internal reference)
forge doc --document-private

# Step 6: Serve docs locally for team review
forge doc --serve --port 8080
# → http://localhost:8080

# Step 7: Verify doc tests pass
forge test --doc
# running 42 doc tests
# test src/lib.fu - add (line 42) ... ok
# test src/lib.fu - divide (line 58) ... ok
# ...
# test result: ok. 42 passed; 0 failed
```

---

## Summary

Pillar 6 ensures that Fusion v2.0 Vortex is not just a language but a **complete development platform**. It provides:

- **Formatting & Conventions** — Enforced code style via `fusion fmt` and `fusion lint`, eliminating bikeshedding and ensuring consistency across teams
- **Built-in Documentation** — Doc comments with runnable code examples, cross-references, and `fusion doc` for generating browsable API documentation
- **Package Manager** — Forge with dependency resolution, version management, feature flags, and cross-language package support
- **Build System** — Multi-target compilation (native, WASM, embedded), optimization profiles, incremental compilation, and cross-compilation support
- **Debugging & Profiling** — Rich stack traces, built-in profiler with flamegraph output, memory profiling, and external tool compatibility
- **Testing Framework** — Unit tests, integration tests, doc tests, benchmarks, and coverage reporting — all integrated into `forge test`
- **Observability** — Structured logging, metrics collection, distributed tracing, and OpenTelemetry compatibility for production monitoring

Together, these tools form the **assembly line** that transforms raw source code into reliable, well-documented, thoroughly tested, and observable software. Without this pillar, even the most elegant language design would fail in production.

---

> **Next**: [Chapter 25 — Pillar 7: The Language's Lifecycle & Governance (The Constitution)](ch25-pillar7-language-lifecycle.md)
