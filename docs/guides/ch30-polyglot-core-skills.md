# Part 3: Core Skills & Interoperability

> *"A language is a dialect with an army and a navy."* — Max Weinreich.
> A polyglot developer is a monoglot with a plan and a compiler flag.

---

## Table of Contents

1. [Environment Setup](#1-environment-setup)
2. [Interoperability — The Glue](#2-interoperability--the-glue)
3. [Build Tools & Project Management](#3-build-tools--project-management)
4. [Testing & CI/CD](#4-testing--cicd)
5. [Data Interchange Rosetta Stone](#5-data-interchange-rosetta-stone)
6. [Code Examples](#6-code-examples)

---

## 1. Environment Setup

### 1.1 Multi-Language Development Environment

A polyglot project demands disciplined toolchain management. The moment you have five languages in one repository, "works on my machine" stops being a joke and starts being a bug report.

**Core principle:** Every toolchain is version-pinned and reproducible. No global installs that drift between machines.

#### Minimum Viable Polyglot Setup

```
┌─────────────────────────────────────────────────┐
│              Development Machine                 │
├─────────────────────────────────────────────────┤
│  Fusion    │ fusion-compiler 2.x.x  (required)  │
│  Rust      │ rustup + cargo       (via rustup)   │
│  Python    │ pyenv + uv           (via pyenv)    │
│  Node.js   │ nvm                 (via nvm)      │
│  Java      │ sdkman              (via sdkman)    │
│  Go        │ go install           (official)     │
│  C/C++     │ system gcc/clang     (via package)  │
└─────────────────────────────────────────────────┘
```

### 1.2 Toolchain Management

Each language has a version manager. Use them all.

| Language | Version Manager | Install | Pin Version |
|----------|----------------|---------|-------------|
| Fusion | `fusionup` (built-in) | `curl -fsSL https://fusion-lang.dev/install.sh \| sh` | `FUSION_VERSION=2.1.0` in `.fusion-version` |
| Rust | `rustup` | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | `rustup override set 1.80.0` |
| Python | `pyenv` | `curl https://pyenv.run \| bash` | `.python-version` file |
| Node.js | `nvm` | `curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh \| bash` | `.nvmrc` file |
| Java | `sdkman` | `curl -s "https://get.sdkman.io" \| bash` | `.sdkmanrc` file |
| Go | Official installer | `https://go.dev/dl/` | `go.mod` specifies `go 1.22` |

**Workflow:** When entering a project directory, each tool reads its version file and auto-switches:

```bash
# Project .tool-versions (asdf) or individual files
# fusion-version  2.1.0
# rust-toolchain  1.80.0
# .python-version 3.12
# .nvmrc          20
# .sdkmanrc       java=21.0.3-tem
```

With `asdf` or `mise` (formerly `rtx`), a single `.tool-versions` file handles everything:

```bash
# Install mise
curl https://mise.run | sh

# Install all project toolchains at once
mise install

# Each shell session auto-activates correct versions
mise activate pwsh >> $PROFILE
```

### 1.3 IDE Configuration for Polyglot Projects

**VS Code** (recommended for polyglot):

```jsonc
// .vscode/settings.json
{
  // Fusion
  "fusion.path": "${env:HOME}/.fusion/bin/fusion",
  "fusion.formatOnSave": true,

  // Rust
  "rust-analyzer.linkedProjects": ["./crates/*/Cargo.toml"],
  "rust-analyzer.check.command": "clippy",

  // Python
  "python.defaultInterpreterPath": ".venv/bin/python",
  "python.analysis.typeCheckingMode": "strict",

  // JavaScript/TypeScript
  "typescript.tsdk": "node_modules/typescript/lib",

  // Java
  "java.jdt.ls.java.home": "${env:JAVA_HOME}",

  // Go
  "go.gopath": "${env:GOPATH}",
  "go.lintTool": "golangci-lint",

  // Universal
  "files.associations": {
    "*.fusion": "fusion",
    "Fusion.toml": "toml"
  },
  "search.exclude": {
    "**/target": true,
    "**/node_modules": true,
    "**/__pycache__": true,
    "**/build": true
  }
}
```

**Essential extensions for polyglot work:**

| Extension | Purpose |
|-----------|---------|
| `tamasfe.even-better-toml` | Fusion.toml editing |
| `rust-lang.rust-analyzer` | Rust IDE features |
| `ms-python.python` | Python language support |
| `charliermarsh.ruff` | Python linting (replaces flake8+isort+black) |
| `dbaeumer.vscode-eslint` | JavaScript/TypeScript linting |
| `redhat.java` | Java language support |
| `golang.go` | Go language support |
| `ms-vscode.cpptools` | C/C++ for FFI work |

### 1.4 Container-Based Development

For teams and CI, containers guarantee identical environments.

```dockerfile
# Dockerfile.polyglot
FROM rust:1.80-bookworm AS rust-base

# Layer: Python
RUN apt-get update && apt-get install -y \
    software-properties-common \
    && add-apt-repository ppa:deadsnakes/ppa \
    && apt-get install -y python3.12 python3.12-venv python3-pip

# Layer: Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Layer: Java
RUN apt-get install -y openjdk-21-jdk

# Layer: Go
RUN wget -q https://go.dev/dl/go1.22.4.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.22.4.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"

# Layer: Fusion
RUN curl -fsSL https://fusion-lang.dev/install.sh | sh
ENV PATH="/root/.fusion/bin:${PATH}"

WORKDIR /workspace
COPY . .

# Verify all toolchains
RUN fusion --version && cargo --version && python3 --version \
    && node --version && java --version && go version
```

```yaml
# docker-compose.yml
services:
  dev:
    build:
      context: .
      dockerfile: Dockerfile.polyglot
    volumes:
      - .:/workspace
      - cargo-cache:/usr/local/cargo/registry
      - pip-cache:/root/.cache/pip
    environment:
      - FUSION_ENV=development
    command: fusion run dev
volumes:
  cargo-cache:
  pip-cache:
```

---

## 2. Interoperability — The Glue

Interoperability is the defining feature of polyglot programming. It is the reason you chose multiple languages instead of writing everything in the one that is "good enough." The question is not *whether* to interoperate but *how* to do it without losing type safety, performance, or your sanity.

### 2.1 Same-Platform Interop (JVM/.NET Style)

#### Fusion ↔ Rust via FFI

Fusion's FFI layer exposes a `@ffi` attribute for declaring foreign function bindings. Rust is the most natural partner because both languages share similar memory models.

```fusion
// bindings/fusion_to_rust.fusion

// Declare external Rust functions
@ffi("libmath_rust.so", "add_integers")
extern fn rust_add(a: Int64, b: Int64) -> Int64

@ffi("libmath_rust.so", "multiply_floats")
extern fn rust_multiply(a: Float64, b: Float64) -> Float64

// Rust function returning a Fusion-owned string
@ffi("libstring_rust.so", "process_string")
extern fn rust_process(input: &String) -> String

fn main():
    let result = rust_add(42, 58)
    println("Rust says: {result}")  // 100

    let product = rust_multiply(3.14, 2.0)
    println("Pi doubled: {product}")  // 6.28
```

```rust
// src/lib.rs — the Rust side
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn add_integers(a: i64, b: i64) -> i64 {
    a + b
}

#[no_mangle]
pub extern "C" fn multiply_floats(a: f64, b: f64) -> f64 {
    a * b
}

#[no_mangle]
pub extern "C" fn process_string(input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let rust_str = c_str.to_str().unwrap_or("");
    let processed = format!("PROCESSED: {}", rust_str.to_uppercase());
    CString::new(processed).unwrap().into_raw()
}
```

**Build the Rust shared library:**

```toml
# crates/math_rust/Cargo.toml
[package]
name = "math_rust"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
```

```bash
cargo build --release --manifest-path crates/math_rust/Cargo.toml
# Output: target/release/libmath_rust.so (Linux) / libmath_rust.dylib (macOS) / math_rust.dll (Windows)
```

#### Shared Memory Between Fusion and C

Fusion can map C memory directly, enabling zero-copy data sharing:

```fusion
// shared_memory.fusion

@ffi("libc.so.6", "malloc")
extern fn c_malloc(size: UInt64) -> *mut Byte

@ffi("libc.so.6", "free")
extern fn c_free(ptr: *mut Byte)

// Allocate a buffer in C memory, write to it from Fusion
fn shared_buffer_example():
    let buf_size: UInt64 = 1024
    let ptr = c_malloc(buf_size)

    // Wrap the raw pointer as a Fusion slice
    let fusion_slice = unsafe Slice<Byte>(ptr, buf_size)

    // Write data into C-allocated memory
    fusion_slice[0] = 0x48  // 'H'
    fusion_slice[1] = 0x69  // 'i'

    // The data is now accessible from both Fusion and C
    c_free(ptr)
```

#### Calling Python from Fusion

Python integration uses the embedded interpreter model:

```fusion
// python_bridge.fusion

@python
import numpy as np
import json

extern fn python_analyze(data: Array<Float64>) -> Dict<String, Any>:
    """Call Python's numpy for numerical analysis."""
    let arr = np.array(data)
    return {
        "mean": float(np.mean(arr)),
        "std": float(np.std(arr)),
        "min": float(np.min(arr)),
        "max": float(np.max(arr)),
        "median": float(np.median(arr))
    }

fn main():
    let measurements = [23.1, 24.5, 22.8, 25.0, 23.9, 24.1, 22.5]
    let stats = python_analyze(measurements)
    println("Mean: {stats["mean"]}, Std: {stats["std"]}")
```

#### Calling JavaScript from Fusion

JavaScript runs via the V8 embedding API:

```fusion
// js_bridge.fusion

@javascript
extern fn js_transform_json(input: String) -> String:
    """Use JavaScript's JSON.parse/stringify for transformations."""
    let obj = JSON.parse(input)
    obj["processed"] = true
    obj["timestamp"] = Date.now()
    return JSON.stringify(obj)

@javascript
extern fn js_regex_extract(pattern: String, text: String) -> Array<String>:
    """Leverage JavaScript's regex engine."""
    let re = new RegExp(pattern, "g")
    let matches = text.match(re)
    return matches ? Array.from(matches) : []

fn main():
    let raw = '{"name": "sensor_01", "value": 42.5}'
    let transformed = js_transform_json(raw)
    println(transformed)
```

#### Calling Java from Fusion

Java interop goes through JNI:

```fusion
// java_bridge.fusion

@java("com.example.MathUtils")
extern class JavaMathUtils:
    @static_method
    extern fn fibonacci(n: Int64) -> Int64

    @static_method
    extern fn prime_sieve(limit: Int64) -> Array<Int64>

fn main():
    let fib_10 = JavaMathUtils.fibonacci(10)
    println("Fibonacci(10) = {fib_10}")  // 55

    let primes = JavaMathUtils.prime_sieve(100)
    println("Primes under 100: {primes.count()}")  // 25
```

### 2.2 Cross-Platform Interop

#### GraalVM-Style Polyglot API

GraalVM proved that a single VM can host multiple languages with shared memory. Fusion's polyglot runtime follows similar principles:

```
┌──────────────────────────────────────────┐
│           Fusion Polyglot Runtime        │
├──────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  Fusion   │  │  Python  │  │  JS    │ │
│  │  (host)   │  │  (guest) │  │ (guest)│ │
│  └────┬─────┘  └────┬─────┘  └───┬────┘ │
│       │              │            │       │
│       └──────────────┴────────────┘       │
│              Shared Value Layer           │
│       (immutable values, zero-copy)       │
└──────────────────────────────────────────┘
```

```fusion
// polyglot_context.fusion

// Create an isolated context for each guest language
let python_ctx = PolyglotContext("python")
let js_ctx = PolyglotContext("javascript")

// Execute code in guest languages
python_ctx.execute("""
import pandas as pd
df = pd.read_csv('data.csv')
result = df.groupby('category').sum()
""")

// Share results without serialization
let fusion_result = python_ctx.get("result")  // Zero-copy reference

// Pass Fusion data into JavaScript
js_ctx.set("fusion_data", fusion_result)
js_ctx.execute("""
const chart = generateChart(fusion_data);
""")
```

#### WebAssembly as Universal Runtime

WebAssembly (Wasm) is the universal compilation target. Any language that compiles to Wasm can interoperate with any other:

```fusion
// wasm_interop.fusion

// Load a Wasm module compiled from Rust
@wasm("./target/wasm32-unknown-unknown/release/image_processor.wasm")
extern module ImageProcessor:
    extern fn resize(image_data: &ByteSlice, width: UInt32, height: UInt32) -> ByteSlice
    extern fn apply_filter(image_data: &ByteSlice, filter: &String) -> ByteSlice

// Load a Wasm module compiled from Go
@wasm("./target/wasm/wasm_shared.wasm")
extern module TextProcessor:
    extern fn tokenize(text: &String) -> Array<String>
    extern fn sentiment(text: &String) -> Float64

fn process_image_and_analyze_caption(path: String) -> Dict:
    let raw = File.read_bytes(path)
    let resized = ImageProcessor.resize(raw, 224, 224)
    let filtered = ImageProcessor.apply_filter(resized, "grayscale")

    let caption = TextProcessor.tokenize("A photo of a cat")
    let sentiment = TextProcessor.sentiment("A beautiful sunset")

    return {
        "processed_image": filtered,
        "tokens": caption,
        "sentiment_score": sentiment
    }
```

#### Foreign Function Interface (FFI) Deep Dive

FFI is the workhorse of polyglot interop. Understanding its mechanics prevents the most painful category of bugs.

**FFI lifecycle:**

```
Fusion code → Marshal arguments → Call foreign symbol → Marshal return → Fusion code
                 ↑                    ↑                      ↑
            Convert Fusion      ABI convention         Convert foreign
            types to C ABI      (cdecl, stdcall,       types back to
            types               sysv, etc.)            Fusion types
```

**Calling conventions:**

| Convention | Platform | Notes |
|------------|----------|-------|
| `cdecl` | x86 Linux/macOS | Caller cleans stack. Default for C. |
| `stdcall` | x86 Windows | Callee cleans stack. Win32 API default. |
| `sysv` | x86_64 Linux | Register-based. Up to 6 integer args in registers. |
| `aarch64` | ARM64 | Register-based. Up to 8 integer args in registers. |
| `wasm` | WebAssembly | Stack-based. All args on the stack. |

**Type marshaling across boundaries:**

| Fusion Type | C Type | Rust Type | Python Type | Go Type |
|-------------|--------|-----------|-------------|---------|
| `Int8` | `int8_t` | `i8` | `int` | `int8` |
| `Int16` | `int16_t` | `i16` | `int` | `int16` |
| `Int32` | `int32_t` | `i32` | `int` | `int32` |
| `Int64` | `int64_t` | `i64` | `int` | `int64` |
| `Float32` | `float` | `f32` | `float` | `float32` |
| `Float64` | `double` | `f64` | `float` | `float64` |
| `Bool` | `_Bool` | `bool` | `bool` | `bool` |
| `String` | `const char*` | `*const c_char` | `str` | `*C.char` |
| `&ByteSlice` | `void* + size_t` | `(*const u8, usize)` | `bytes` | `(unsafe.Pointer, uintptr)` |
| `null` | `NULL` | `Option::None` | `None` | `nil` |

**Critical safety rules:**

1. **Never pass Fusion-managed pointers to foreign code that stores them.** The garbage collector can relocate the object.
2. **Always use `@ffi_owned` for memory returned by foreign code** — Fusion will free it.
3. **String conversion allocates.** Prefer passing raw byte buffers when performance matters.

```fusion
// SAFE: String passed by value (copied to C string, freed after call)
@ffi("lib.so", "process")
extern fn safe_call(input: String) -> String

// UNSAFE: Raw pointer may be invalid after GC cycle
@ffi("lib.so", "unsafe_process")
extern fn unsafe_call(input: *mut Byte) -> *mut Byte  // Don't do this

// SAFE: Bounded lifetime via scope
extern fn with_scope(input: &String) -> String:
    // `input` is valid only within this call
    // Fusion guarantees the pointer is stable during the extern call
```

### 2.3 Data Serialization for Interop

When FFI is not viable (different processes, different machines, different runtimes), serialization is the bridge.

#### JSON/YAML (Easy but Slow)

```
Use when:  Human readability matters, schema is flexible, data is small (<1MB)
Avoid:     High-frequency serialization, binary data, strict type requirements

Fusion:
    let json_str = Json.stringify(sensor_data)
    let parsed = Json.parse<SensorReading>(json_str)

Python:
    import json
    data = json.loads(fusion_output)

JavaScript:
    const data = JSON.parse(fusionOutput);
```

**Performance characteristics:**

| Format | Parse (ms/MB) | Serialize (ms/MB) | Human Readable | Schema Evolution |
|--------|---------------|-------------------|----------------|------------------|
| JSON | ~15 | ~10 | Yes | Additive only |
| YAML | ~50 | ~30 | Yes | Additive only |

#### Protobuf (Schema Evolution)

```
Use when:  Schema must evolve, cross-language, moderate performance needs
Avoid:     When you need human-readable wire format, very small messages

# schema.proto
syntax = "proto3";
package sensor;

message Reading {
  string device_id = 1;
  double temperature = 2;
  double humidity = 3;
  int64 timestamp = 4;
  map<string, string> metadata = 5;
}

message Batch {
  repeated Reading readings = 1;
}
```

```
Fusion:
    let reading = Sensor.Reading {
        device_id: "sensor_01",
        temperature: 23.5,
        humidity: 65.0,
        timestamp: now().unix(),
    }
    let bytes = Proto.encode(reading)

Python:
    reading = sensor.Reading()
    reading.ParseFromString(fusion_bytes)
```

**Schema evolution rules:**

1. Never reuse field numbers.
2. New fields must have default values.
3. `repeated` fields are always backward compatible.
4. `oneof` fields are backward compatible but `map` fields are not.

#### FlatBuffers/Cap'n Proto (Zero-Copy)

```
Use when:  Performance is critical, data is large, random access needed
Avoid:     When simplicity is more important than performance

FlatBuffers:
    - Access fields without parsing/deserialization
    - Memory-mapped I/O friendly
    - ~0 parse time (data is already in the right format)

Cap'n Proto:
    - Similar zero-copy semantics
    - In-memory format IS the wire format
    - No encode/decode step at all
```

```fbs
# FlatBuffers schema (schema.fbs)
table SensorReading {
    device_id: string;
    temperature: double;
    humidity: double;
    timestamp: long;
}

root_type SensorReading;
```

**Performance comparison (1M records, 100 bytes each):**

| Format | Serialize | Deserialize | Random Access | Size on Wire |
|--------|-----------|-------------|---------------|--------------|
| JSON | 850ms | 1200ms | No | 120MB |
| Protobuf | 120ms | 180ms | No | 45MB |
| FlatBuffers | 95ms | ~0ms (mmap) | Yes | 52MB |
| Cap'n Proto | 90ms | ~0ms (mmap) | Yes | 48MB |
| MessagePack | 200ms | 250ms | No | 55MB |

#### MessagePack (Binary JSON)

```
Use when:  You want JSON's simplicity but 2x smaller + 3x faster
Avoid:     When you need schema evolution or zero-copy access

Fusion:
    let packed = MsgPack.encode(data)
    let unpacked = MsgPack.decode<SensorData>(packed)

Python:
    import msgpack
    data = msgpack.unpackb(fusion_packed, raw=False)
```

#### When to Use Each Format

```
                        ┌─────────────────┐
                        │ Need human       │
                        │ readability?     │
                        └────────┬────────┘
                           Yes /   \ No
                              /     \
                     ┌───────┐     ┌────────────────┐
                     │ JSON  │     │ Need schema     │
                     │  or   │     │ evolution?      │
                     │ YAML  │     └───────┬────────┘
                     └───────┘         Yes / \ No
                                        /     \
                              ┌─────────┐   ┌──────────────┐
                              │Protobuf │   │ Need zero-    │
                              │         │   │ copy access?  │
                              └─────────┘   └──────┬──────-'
                                              Yes / \ No
                                                 /     \
                                        ┌──────────┐ ┌────────────┐
                                        │FlatBuffers│ │MessagePack │
                                        │Cap'n Proto│ │  or JSON   │
                                        └──────────┘ └────────────┘
```

---

## 3. Build Tools & Project Management

### 3.1 Fusion.toml as Polyglot Manifest

`Fusion.toml` is the single source of truth for a polyglot project. It declares every language, every dependency, and every build target.

```toml
# Fusion.toml
[project]
name = "sensor-platform"
version = "2.1.0"
edition = "2024"
authors = ["Team Vortex"]

# --- Fusion core ---
[dependencies]
fusion-std = "2.1"
fusion-async = "2.1"
fusion-ffi = "2.1"
fusion-json = "2.1"

# --- Rust crates ---
[rust.dependencies]
image-processor = { path = "./crates/image-processor", version = "0.3" }
crypto-bindings = { git = "https://github.com/example/crypto-bindings", branch = "main" }

[rust.profile.release]
opt-level = 3
lto = true
codegen-units = 1

# --- Python packages ---
[python.dependencies]
numpy = ">=1.26"
pandas = ">=2.1"
scikit-learn = ">=1.4"
pymupdf = ">=1.23"

[python.virtualenv]
path = ".venv"
python = "3.12"

# --- Node.js packages ---
[node.dependencies]
express = "^4.18"
sharp = "^0.33"
@tensorflow/tfjs-node = "^4.15"

[node.devDependencies]
typescript = "^5.3"
@types/express = "^4.17"

# --- Java/Gradle ---
[java.dependencies]
com.google.protobuf:protobuf-java = "4.25.3"
org.apache.kafka:kafka-clients = "3.7.0"

# --- Go modules ---
[go.dependencies]
github.com/gin-gonic/gin = "v1.9"
github.com/redis/go-redis/v9 = "v9.5"

# --- Shared build targets ---
[[targets]]
name = "api-server"
kind = "binary"
lang = "fusion"
entry = "src/main.fusion"
deps = ["rust:image-processor", "python:numpy", "node:express"]

[[targets]]
name = "data-pipeline"
kind = "binary"
lang = "fusion"
entry = "src/pipeline.fusion"
deps = ["python:pandas", "java:kafka-clients", "go:gin"]

[[targets]]
name = "ml-service"
kind = "library"
lang = "rust"
entry = "crates/ml-service/src/lib.rs"

# --- Build profiles ---
[profile.dev]
fusion-opt-level = 0
rust-debug = true
python-unbuffered = true

[profile.release]
fusion-opt-level = 3
rust-release = true
lto = true
```

### 3.2 Cross-Language Dependency Resolution

The **Forge** package manager resolves dependencies across all languages from a single `Fusion.toml`:

```bash
# Install all dependencies for all languages
forge install

# Install only Python + Rust dependencies
forge install --lang python,rust

# Add a new Rust crate
forge add rust:serde --features derive

# Add a new Python package
forge add python:fastapi

# Add a local Rust crate as dependency
forge add rust:./crates/my-crate --path

# Resolve conflicts (when two languages need incompatible versions)
forge doctor
# Output:
#   WARN: Python 3.12 requires pip>=24.0, but system pip is 23.3
#   WARN: Node.js 20.x has deprecation warnings for sharp@0.32
#   OK:   Rust toolchain 1.80.0 is compatible with all crates
#   OK:   Go 1.22.4 matches go.mod requirement
```

**Dependency graph visualization:**

```bash
forge graph
```

```
sensor-platform v2.1.0
+-- fusion-std 2.1
+-- fusion-async 2.1
+-- fusion-ffi 2.1
+-- fusion-json 2.1
+-- rust:image-processor 0.3 ---- rust:rayon 1.8
|                                +-- rust:image 0.25
|                                +-- rust:num 0.4
+-- rust:crypto-bindings (git) -- rust:openssl 0.10
+-- python:numpy 1.26
+-- python:pandas 2.1
|   +-- python:numpy >=1.26
+-- python:scikit-learn 1.4
|   +-- python:scipy >=1.11
+-- node:express 4.18
|   +-- node:body-parser 1.20
+-- java:protobuf-java 4.25.3
+-- go:gin 1.9
    +-- go:go-redis/v9 9.5
```

### 3.3 Build Caching Strategies

Cross-language builds benefit enormously from caching. Each language has its own cache, but a unified strategy avoids redundant work.

```toml
# Fusion.toml build cache configuration
[build.cache]
# Shared cache directory
base = ".forge/cache"

# Per-language cache settings
[rust.cache]
target = "target"
incremental = true
sccache = true  # Use sccache for shared compilation cache

[python.cache]
wheel = true
compile = false  # Don't cache .pyc in project

[node.cache]
npm = ".npm"
turbo = true     # Use Turborepo for monorepo caching

[java.cache]
gradle = ".gradle"
build_cache = true

[go.cache]
module_cache = true
build_cache = true

# Cache invalidation rules
[build.cache.invalidation]
# When Fusion.toml changes, invalidate all
fusion_toml = "all"
# When Cargo.lock changes, invalidate Rust only
cargo_lock = "rust"
# When requirements.txt changes, invalidate Python only
requirements = "python"
# When package-lock.json changes, invalidate Node only
package_lock = "node"
```

### 3.4 Incremental Compilation Across Languages

```
+------------------------------------------------------+
|              Unified Build Pipeline                   |
+------------------------------------------------------+
|                                                      |
|  1. Parse Fusion.toml -> dependency graph             |
|  2. Check cache fingerprints per language             |
|  3. Build in dependency order:                        |
|     +-----+     +------+     +------+               |
|     |Rust |---->|Fusion|---->| Node |               |
|     |(low)|     |(mid) |     |(high)|               |
|     +-----+     +------+     +------+               |
|         |            |            |                   |
|         v            v            v                   |
|     [cache hit] [rebuild]  [cache hit]               |
|                                                      |
|  4. Link shared libraries                            |
|  5. Generate polyglot bindings                        |
|  6. Run integration tests at boundaries               |
|                                                      |
+------------------------------------------------------+
```

```bash
# Build everything (parallel where possible)
forge build

# Build only changed languages
forge build --changed

# Build with verbose output
forge build --verbose

# Build specific target
forge build --target api-server

# Clean all caches
forge clean

# Clean only Python cache
forge clean --lang python

# Show what would be rebuilt
forge build --dry-run
```

---

## 4. Testing & CI/CD

### 4.1 Unit Testing Across Languages

Each language uses its native test runner. The polyglot test harness coordinates them.

```toml
# Fusion.toml test configuration
[test]
runner = "forge test"

[test.fusion]
command = "fusion test"
pattern = "tests/**/*.test.fusion"

[test.rust]
command = "cargo test --manifest-path crates/*/Cargo.toml"

[test.python]
command = "uv run pytest tests/python/ -v"

[test.node]
command = "npx jest tests/js/"

[test.java]
command = "./gradlew test"

[test.go]
command = "go test ./tests/go/..."
```

```bash
# Run all tests
forge test

# Run tests for specific language
forge test --lang rust
forge test --lang python

# Run tests matching a pattern
forge test --filter "integration"

# Run tests with coverage
forge test --coverage

# Run tests in parallel (each language in its own process)
forge test --parallel
```

### 4.2 Integration Testing at Boundaries

The most critical tests live at the seams between languages:

```fusion
// tests/ffi_integration.test.fusion

import fusion:testing.{Test, assert_eq, assert_raises}

@ffi("libmath_rust.so", "add_integers")
extern fn rust_add(a: Int64, b: Int64) -> Int64

@python
extern fn python_analyze(data: Array<Float64>) -> Dict<String, Any>

test "Rust FFI: integer addition across boundary":
    let result = rust_add(i64::MAX - 1, 1)
    assert_eq(result, i64::MAX)

test "Python FFI: numpy analysis returns correct shape":
    let data = [1.0, 2.0, 3.0, 4.0, 5.0]
    let stats = python_analyze(data)
    assert_eq(stats["mean"], 3.0)
    assert_eq(stats["count"], 5)

test "Roundtrip serialization: Fusion -> JSON -> Python -> JSON -> Fusion":
    let original = SensorReading {
        id: "sensor_01",
        value: 42.5,
        timestamp: 1721827200,
    }
    let json = Json.stringify(original)
    // Python validates the schema and returns it back
    let validated = python_validate_and_return(json)
    let roundtripped = Json.parse<SensorReading>(validated)
    assert_eq(original, roundtripped)

test "Memory safety: no use-after-free across FFI":
    for i in 0..1000:
        let data = generate_random_buffer(1024)
        let result = rust_process(data)
        assert_eq(result.len(), data.len())
        // If this doesn't crash, the FFI memory management is correct
```

### 4.3 Property-Based Testing

Generate random inputs to test invariants across language boundaries:

```fusion
// tests/properties.test.fusion

import fusion:testing.{PropertyTest, for_all, assert_eq}
import fusion:testing::generators as gen

@ffi("libmath_rust.so", "add_integers")
extern fn rust_add(a: Int64, b: Int64) -> Int64

@property_test
fn addition_is_commutative(a: Int64, b: Int64):
    for_all(gen.int64_range(-1000, 1000), gen.int64_range(-1000, 1000)):
        assert_eq(rust_add(a, b), rust_add(b, a))

@property_test
fn addition_identity(a: Int64):
    for_all(gen.int64_range(-1000, 1000)):
        assert_eq(rust_add(a, 0), a)

@property_test
fn serialization_roundtrip(data: Dict<String, Any>):
    for_all(gen.dict(gen.string(), gen.any_json_value())):
        let json = Json.stringify(data)
        let parsed = Json.parse<Dict<String, Any>>(json)
        assert_eq(data, parsed)
```

### 4.4 Mutation Testing

Verify that your tests actually catch bugs by introducing deliberate mutations:

```bash
# Rust: cargo-mutants
cargo install cargo-mutants
cargo mutants --timeout 120 --testing "crate::math"

# Python: mutmut
pip install mutmut
mutmut run --paths-to-mutate=src/python/
mutmut results

# JavaScript: Stryker
npx stryker run --mutate "src/**/*.js" --testRunner jest
```

**Mutation score targets:**

| Metric | Minimum | Good | Excellent |
|--------|---------|------|-----------|
| Line coverage | 70% | 85% | 95% |
| Mutation score | 50% | 70% | 85% |
| FFI boundary coverage | 90% | 95% | 100% |

### 4.5 CI/CD Pipeline Design

```yaml
# .github/workflows/polyglot-ci.yml
name: Polyglot CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  # --- Lint & Format (fast, parallel) ---
  lint:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        language: [fusion, rust, python, node, go]
    steps:
      - uses: actions/checkout@v4
      - name: Lint ${{ matrix.language }}
        run: forge lint --lang ${{ matrix.language }}

  # --- Build (sequential by dependency) ---
  build:
    needs: lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Restore cache
        uses: actions/cache@v4
        with:
          path: |
            .forge/cache
            target/
            node_modules/
            .gradle/
          key: polyglot-${{ runner.os }}-${{ hashFiles('**/Fusion.toml', '**/Cargo.lock', '**/package-lock.json') }}

      - name: Install toolchains
        run: |
          curl -fsSL https://fusion-lang.dev/install.sh | sh
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          pip install uv
          npm ci
          go mod download

      - name: Build all languages
        run: forge build

      - name: Build shared libraries
        run: |
          cargo build --release --manifest-path crates/*/Cargo.toml

  # --- Test (parallel per language) ---
  test:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run unit tests
        run: forge test --parallel

      - name: Run FFI integration tests
        run: forge test --filter "ffi" --verbose

      - name: Run property tests
        run: forge test --filter "property" --timeout 300

  # --- Coverage ---
  coverage:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Generate coverage reports
        run: |
          forge test --coverage --lang rust     # cargo-tarpaulin
          forge test --coverage --lang python   # coverage.py
          forge test --coverage --lang node     # c8

      - name: Merge coverage
        run: forge coverage merge --format lcov

      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: coverage/lcov.info

  # --- Deploy ---
  deploy:
    needs: coverage
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Deploy
        run: forge deploy --profile production
```

### 4.6 Test Coverage Reporting

```bash
# Generate unified coverage report across all languages
forge coverage report

# Output:
#   Language    Lines    Branches    Functions    FFI Boundary
#   ---------   -----    --------    ---------    ------------
#   Fusion      92%      87%         95%          100%
#   Rust        88%      82%         91%          100%
#   Python      79%      71%         85%          N/A
#   JavaScript  84%      78%         88%          N/A
#   Go          91%      85%         93%          N/A
#   Java        86%      80%         89%          N/A
#   ---------   -----    --------    ---------    ------------
#   Overall     87%      81%         90%          100%

# Enforce minimum coverage
forge coverage check --min-overall 80 --min-ffi 95
```

---

## 5. Data Interchange Rosetta Stone

### 5.1 Type Representation Across Languages

| Concept | Fusion | Python | JavaScript | Rust | Java | Go |
|---------|--------|--------|------------|------|------|-----|
| **Signed 8-bit** | `Int8` | `int` | `number` | `i8` | `byte` | `int8` |
| **Signed 16-bit** | `Int16` | `int` | `number` | `i16` | `short` | `int16` |
| **Signed 32-bit** | `Int32` | `int` | `number` | `i32` | `int` | `int32` |
| **Signed 64-bit** | `Int64` | `int` | `BigInt` | `i64` | `long` | `int64` |
| **Unsigned 8-bit** | `UInt8` | `int` | `number` | `u8` | `unsigned byte` | `uint8` |
| **Float 32-bit** | `Float32` | `float` | `number` | `f32` | `float` | `float32` |
| **Float 64-bit** | `Float64` | `float` | `number` | `f64` | `double` | `float64` |
| **Boolean** | `Bool` | `bool` | `boolean` | `bool` | `boolean` | `bool` |
| **Character** | `Char` | `str` (len 1) | `string` (len 1) | `char` | `char` | `rune` |
| **String** | `String` | `str` | `string` | `String` | `String` | `string` |
| **Null** | `null` | `None` | `null` | `Option::None` | `null` | `nil` |
| **Optional** | `?T` | `Optional[T]` | `T | null` | `Option<T>` | `Optional<T>` | `*T` |
| **Error** | `Result<T,E>` | `Exception` | `Error` | `Result<T,E>` | `Exception` | `error` interface |
| **Fixed Array** | `[T; N]` | `tuple` | N/A | `[T; N]` | `T[]` | `[N]T` (Go 1.17+) |
| **Dynamic Array** | `Array<T>` | `list` | `Array` | `Vec<T>` | `ArrayList<T>` | `[]T` |
| **Map/Dict** | `Dict<K,V>` | `dict` | `Object` / `Map` | `HashMap<K,V>` | `HashMap<K,V>` | `map[K]V` |
| **Date** | `DateTime` | `datetime` | `Date` | `chrono::DateTime` | `Instant` | `time.Time` |
| **UUID** | `Uuid` | `uuid.UUID` | N/A (string) | `uuid::Uuid` | `UUID` | `uuid.UUID` |
| **Tuple** | `(A, B)` | `tuple` | N/A | `(A, B)` | `record` (Java 16+) | N/A (use struct) |
| **Void** | `()` | `None` (return) | `void` / `undefined` | `()` | `void` | N/A |

### 5.2 Serialization Format Comparison

| Feature | JSON | YAML | Protobuf | FlatBuffers | Cap'n Proto | MessagePack |
|---------|------|------|----------|-------------|-------------|-------------|
| **Human readable** | Yes | Yes | No | No | No | No |
| **Schema required** | No | No | Yes | Yes | Yes | No |
| **Schema evolution** | Weak | Weak | Strong | Strong | Strong | N/A |
| **Zero-copy** | No | No | No | Yes | Yes | No |
| **Streaming** | Yes | No | Yes | Yes | Yes | Yes |
| **Compression** | Gzip | Gzip | Built-in | N/A | N/A | Gzip |
| **Worst-case size** | Large | Largest | Medium | Medium | Medium | Medium |
| **Parse speed** | Slow | Slowest | Fast | Instant | Instant | Medium |
| **Ecosystem support** | Universal | Universal | Wide | Growing | Growing | Wide |
| **Best for** | APIs, configs | Configs, docs | Services, storage | Game engines, mmap | Embedded, IPC | Redis, msg queues |

### 5.3 Common Data Corruption Pitfalls

```
PITFALL 1: Integer overflow at language boundaries
-----------------------------------------------
Fusion Int64: 9223372036854775807  (max)
Python int:   9223372036854775807  (same -- arbitrary precision)
JS number:    9223372036854775807  -> 9007199254740992  (WRONG! JS numbers are 53-bit floats)
Rust i64:     9223372036854775807  (correct)

SOLUTION: Use BigInt in JS, or validate ranges at the FFI boundary.

PITFALL 2: Floating-point representation
-----------------------------------------
0.1 + 0.2 = 0.30000000000000004  (IEEE 754)

This is consistent across ALL languages. But:
- Python's Decimal(0.1) + Decimal(0.2) = Decimal("0.3")  (arbitrary precision)
- Fusion's Decimal type also avoids this

SOLUTION: Use Decimal/BigDecimal for money, or compare with epsilon.

PITFALL 3: String encoding mismatch
-------------------------------------
Fusion:  UTF-8 (always)
Python:  UTF-8 (default) or configurable
JS:      UTF-16 internally, UTF-8 on I/O
Java:    UTF-16 internally, configurable on I/O
Go:      UTF-8 (always)
Rust:    UTF-8 (always)

SOLUTION: Normalize to UTF-8 at every boundary. Never assume encoding.

PITFALL 4: Date/time representation
-------------------------------------
Fusion:   DateTime (timezone-aware)
Python:   datetime (naive or timezone-aware)
JS:       Date (always UTC internally, displayed locally)
Java:     Instant (UTC) or LocalDateTime (no timezone)
Go:       time.Time (always timezone-aware)

SOLUTION: Use UTC timestamps (Unix epoch) for interop. Convert to local only for display.

PITFALL 5: Null vs None vs undefined vs nil
----------------------------------------------
Fusion:    null
Python:    None
JS:        null | undefined  (TWO nulls!)
Java:      null (primitives can't be null)
Go:        nil (interfaces, pointers, maps, slices, channels)
Rust:      Option::None (no null by design)

SOLUTION: Map all to Option<T>/Nullable<T> at boundaries. Never trust "truthy" checks.
```

### 5.4 Safe Conversion Patterns

```fusion
// safe_conversions.fusion

// Pattern 1: Range-checked integer conversion
fn safe_i64_to_i32(value: Int64) -> Result<Int32, OverflowError>:
    if value > Int32::MAX or value < Int32::MIN:
        return Err(OverflowError("Int64 value {value} overflows Int32"))
    return Ok(value as Int32)

// Pattern 2: Checked float-to-int
fn safe_float_to_int(value: Float64) -> Result<Int64, ConversionError>:
    if value.is_nan():
        return Err(ConversionError("Cannot convert NaN to integer"))
    if value > Int64::MAX as Float64:
        return Err(ConversionError("Value {value} exceeds Int64 range"))
    if value < Int64::MIN as Float64:
        return Err(ConversionError("Value {value} below Int64 range"))
    return Ok(value as Int64)

// Pattern 3: String validation at boundaries
fn validate_utf8(input: &ByteSlice) -> Result<String, EncodingError>:
    match String::from_utf8(input.to_vec()):
        Ok(s) => Ok(s),
        Err(e) => Err(EncodingError("Invalid UTF-8 at byte {e.valid_up_to()}"))

// Pattern 4: Date normalization to UTC
fn normalize_date(input: DateTime) -> Int64:
    // Always convert to UTC epoch seconds for interop
    return input.to_utc().unix_timestamp()

// Pattern 5: Null-safe optional chaining
fn safe_dict_get(dict: &Dict<String, Any>, key: &String) -> Optional<String>:
    match dict.get(key):
        Some(value) => value.as_string(),
        None => None

// Pattern 6: Collection size validation
fn validate_array_size<T>(data: Array<T>, max_size: UInt64) -> Result<Array<T>, SizeError>:
    if data.len() as UInt64 > max_size:
        return Err(SizeError("Array size {data.len()} exceeds max {max_size}"))
    return Ok(data)
```

---

## 6. Code Examples

### 6.1 Complete FFI Example: Fusion Calling Python

**Project structure:**

```
fusion-python-ffi/
+-- Fusion.toml
+-- src/
|   +-- main.fusion
+-- python/
|   +-- __init__.py
|   +-- analyzer.py
|   +-- requirements.txt
+-- tests/
    +-- test_ffi.fusion
```

```toml
# Fusion.toml
[project]
name = "fusion-python-ffi"
version = "1.0.0"

[python.dependencies]
numpy = ">=1.26"
pandas = ">=2.1"
scikit-learn = ">=1.4"

[python.virtualenv]
path = ".venv"
python = "3.12"
```

```python
# python/analyzer.py
import numpy as np
from typing import List, Dict, Any


def compute_statistics(data: List[float]) -> Dict[str, Any]:
    """Compute comprehensive statistics for a dataset."""
    arr = np.array(data)
    return {
        "count": int(len(arr)),
        "mean": float(np.mean(arr)),
        "std": float(np.std(arr)),
        "min": float(np.min(arr)),
        "max": float(np.max(arr)),
        "median": float(np.median(arr)),
        "q25": float(np.percentile(arr, 25)),
        "q75": float(np.percentile(arr, 75)),
        "skewness": float(_skewness(arr)),
        "kurtosis": float(_kurtosis(arr)),
    }


def _skewness(arr: np.ndarray) -> float:
    n = len(arr)
    if n < 3:
        return 0.0
    mean = np.mean(arr)
    std = np.std(arr, ddof=1)
    if std == 0:
        return 0.0
    return float(np.mean(((arr - mean) / std) ** 3) * (n / ((n - 1) * (n - 2))))


def _kurtosis(arr: np.ndarray) -> float:
    n = len(arr)
    if n < 4:
        return 0.0
    mean = np.mean(arr)
    std = np.std(arr, ddof=1)
    if std == 0:
        return 0.0
    m4 = np.mean(((arr - mean) / std) ** 4)
    return float(m4 - 3.0)


def detect_outliers(data: List[float], method: str = "iqr") -> List[int]:
    """Detect outliers and return their indices."""
    arr = np.array(data)
    if method == "iqr":
        q25, q75 = np.percentile(arr, [25, 75])
        iqr = q75 - q25
        lower = q25 - 1.5 * iqr
        upper = q75 + 1.5 * iqr
        return [int(i) for i in np.where((arr < lower) | (arr > upper))[0]]
    elif method == "zscore":
        mean, std = np.mean(arr), np.std(arr)
        if std == 0:
            return []
        zscores = np.abs((arr - mean) / std)
        return [int(i) for i in np.where(zscores > 3)[0]]
    else:
        raise ValueError(f"Unknown method: {method}")


def linear_regression(x: List[float], y: List[float]) -> Dict[str, float]:
    """Simple linear regression: y = mx + b."""
    x_arr = np.array(x)
    y_arr = np.array(y)
    n = len(x_arr)
    if n < 2:
        return {"slope": 0.0, "intercept": 0.0, "r_squared": 0.0}
    slope = (n * np.sum(x_arr * y_arr) - np.sum(x_arr) * np.sum(y_arr)) / \
            (n * np.sum(x_arr**2) - np.sum(x_arr)**2)
    intercept = np.mean(y_arr) - slope * np.mean(x_arr)
    y_pred = slope * x_arr + intercept
    ss_res = np.sum((y_arr - y_pred) ** 2)
    ss_tot = np.sum((y_arr - np.mean(y_arr)) ** 2)
    r_squared = 1 - (ss_res / ss_tot) if ss_tot != 0 else 0.0
    return {"slope": float(slope), "intercept": float(intercept), "r_squared": float(r_squared)}
```

```fusion
// src/main.fusion
import fusion:io.{println, eprintln}
import fusion:convert.{ToString}

// Declare Python functions
@python("python/analyzer.py")
extern module Analyzer:
    extern fn compute_statistics(data: Array<Float64>) -> Dict<String, Any>
    extern fn detect_outliers(data: Array<Float64>, method: String) -> Array<Int64>
    extern fn linear_regression(x: Array<Float64>, y: Array<Float64>) -> Dict<String, Float64>

fn main():
    // Generate sample data
    let temperatures = [
        22.1, 23.5, 21.8, 25.0, 23.9, 24.1, 22.5,
        23.2, 24.8, 22.9, 23.7, 24.3, 22.0, 23.4,
        25.2, 21.5, 24.0, 23.6, 22.8, 24.5
    ]

    // Compute statistics using Python/numpy
    let stats = Analyzer.compute_statistics(temperatures)
    println("=== Temperature Statistics ===")
    println("Count:    {stats["count"]}")
    println("Mean:     {stats["mean"]:.2f}°C")
    println("Std Dev:  {stats["std"]:.2f}°C")
    println("Range:    [{stats["min"]:.1f}, {stats["max"]:.1f}]")
    println("Median:   {stats["median"]:.2f}°C")
    println("IQR:      [{stats["q25"]:.1f}, {stats["q75"]:.1f}]")

    // Detect outliers
    let outlier_indices = Analyzer.detect_outliers(temperatures, "iqr")
    if outlier_indices.len() > 0:
        println("\nOutliers detected at indices: {outlier_indices}")
        for idx in outlier_indices:
            println("  [{idx}] = {temperatures[idx as UInt64]}°C")
    else:
        println("\nNo outliers detected.")

    // Linear regression: time vs temperature
    let time_points = Array.range(0.0, temperatures.len() as Float64, 1.0)
    let regression = Analyzer.linear_regression(time_points, temperatures)
    println("\n=== Linear Regression ===")
    println("Slope:     {regression["slope"]:.4f} °C/period")
    println("Intercept: {regression["intercept"]:.2f} °C")
    println("R²:        {regression["r_squared"]:.4f}")
```

```bash
# Setup and run
forge install
forge run
```

**Expected output:**

```
=== Temperature Statistics ===
Count:    20
Mean:     23.32°C
Std Dev:  0.98°C
Range:    [21.5, 25.2]
Median:   23.45°C
IQR:      [22.6, 24.2]

No outliers detected.

=== Linear Regression ===
Slope:     0.0234 °C/period
Intercept: 23.10 °C
R²:        0.0842
```

### 6.2 Complete FFI Example: Fusion Calling Rust

**Project structure:**

```
fusion-rust-ffi/
+-- Fusion.toml
+-- crates/
|   +-- image-processor/
|   |   +-- Cargo.toml
|   |   +-- src/lib.rs
|   +-- crypto-engine/
|       +-- Cargo.toml
|       +-- src/lib.rs
+-- src/
|   +-- main.fusion
+-- tests/
    +-- test_ffi.fusion
```

```toml
# crates/image-processor/Cargo.toml
[package]
name = "image-processor"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
image = "0.25"
rayon = "1.8"
```

```rust
// crates/image-processor/src/lib.rs
use image::{DynamicImage, RgbaImage, Rgba};
use rayon::prelude::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;

#[repr(C)]
pub struct ProcessedImage {
    pub data: *mut u8,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
}

#[no_mangle]
pub extern "C" fn image_resize(
    data: *const u8,
    data_len: usize,
    target_width: u32,
    target_height: u32,
) -> ProcessedImage {
    let input_slice = unsafe { slice::from_raw_parts(data, data_len) };
    let img = image::load_from_memory(input_slice).expect("Failed to decode image");
    let resized = img.resize_exact(target_width, target_height, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();
    let raw = rgba.into_raw();

    let mut output = ProcessedImage {
        data: Box::into_raw(raw.into_boxed_slice()) as *mut u8,
        width,
        height,
        channels: 4,
    };
    output
}

#[no_mangle]
pub extern "C" fn image_grayscale(
    data: *const u8,
    data_len: usize,
) -> ProcessedImage {
    let input_slice = unsafe { slice::from_raw_parts(data, data_len) };
    let img = image::load_from_memory(input_slice).expect("Failed to decode image");
    let gray = img.grayscale();
    let rgba = gray.to_rgba8();
    let (width, height) = rgba.dimensions();
    let raw = rgba.into_raw();

    ProcessedImage {
        data: Box::into_raw(raw.into_boxed_slice()) as *mut u8,
        width,
        height,
        channels: 4,
    }
}

#[no_mangle]
pub extern "C" fn image_free(img: ProcessedImage) {
    if !img.data.is_null() {
        unsafe {
            let _ = Box::from_raw(slice::from_raw_parts_mut(
                img.data,
                (img.width * img.height * img.channels) as usize,
            ));
        }
    }
}

#[no_mangle]
pub extern "C" fn image_histogram(
    data: *const u8,
    data_len: usize,
) -> *mut u32 {
    let input_slice = unsafe { slice::from_raw_parts(data, data_len) };
    let img = image::load_from_memory(input_slice).expect("Failed to decode image");
    let rgba = img.to_rgba8();
    let pixels: Vec<Rgba<u8>> = rgba.pixels().cloned().collect();

    // Parallel histogram computation
    let histograms: Vec<[u32; 256]> = pixels
        .par_chunks(1024)
        .map(|chunk| {
            let mut hist = [0u32; 256];
            for pixel in chunk {
                hist[pixel[0] as usize] += 1; // Red channel
            }
            hist
        })
        .reduce(|| [0u32; 256], |mut acc, h| {
            for i in 0..256 {
                acc[i] += h[i];
            }
            acc
        });

    let boxed = histograms[0].to_vec().into_boxed_slice();
    Box::into_raw(boxed) as *mut u32
}
```

```fusion
// src/main.fusion
import fusion:io.{println, eprintln}
import fusion:file.{File}

// Declare Rust FFI bindings
@ffi("libimage_processor.so", "image_resize")
extern fn rust_image_resize(
    data: &ByteSlice,
    data_len: UInt64,
    target_width: UInt32,
    target_height: UInt32,
) -> FfiPointer

@ffi("libimage_processor.so", "image_grayscale")
extern fn rust_image_grayscale(data: &ByteSlice, data_len: UInt64) -> FfiPointer

@ffi("libimage_processor.so", "image_free")
extern fn rust_image_free(img: FfiPointer)

@ffi("libimage_processor.so", "image_histogram")
extern fn rust_image_histogram(data: &ByteSlice, data_len: UInt64) -> FfiPointer

struct RustImage:
    pointer: FfiPointer
    width: UInt32
    height: UInt32
    channels: UInt32

    fn free(self):
        rust_image_free(self.pointer)

fn main():
    let image_path = "test_image.png"
    let image_data = File.read_bytes(image_path)
        match:
            Ok(d) => d,
            Err(e) => { eprintln("Failed to read image: {e}"); return }

    println("Original image: {image_data.len()} bytes")

    // Resize
    let resized = rust_image_resize(image_data.ptr(), image_data.len() as UInt64, 800, 600)
    println("Resized: pointer={resized}")

    // Grayscale
    let gray = rust_image_grayscale(image_data.ptr(), image_data.len() as UInt64)
    println("Grayscale: pointer={gray}")

    // Histogram (parallel computation in Rust)
    let hist_ptr = rust_image_histogram(image_data.ptr(), image_data.len() as UInt64)
    let histogram = unsafe Slice<UInt32>(hist_ptr as *mut UInt32, 256)
    let max_val = histogram.max() as Float64
    println("\n=== Red Channel Histogram (normalized) ===")
    for i in (0..256).step_by(16):
        let bar_len = (histogram[i as UInt64] as Float64 / max_val * 40.0) as UInt64
        let bar = "█" * bar_len
        println("  [{i:3d}] {bar}")

    // Cleanup
    rust_image_free(resized)
    rust_image_free(gray)
```

### 6.3 Complete FFI Example: Fusion Calling JavaScript

**Project structure:**

```
fusion-js-ffi/
+-- Fusion.toml
+-- src/
|   +-- main.fusion
+-- js/
|   +-- package.json
|   +-- transformer.js
|   +-- validator.js
+-- tests/
    +-- test_ffi.fusion
```

```json
// js/package.json
{
  "name": "fusion-js-bridge",
  "version": "1.0.0",
  "dependencies": {
    "lodash": "^4.17.21",
    "ajv": "^8.12.0",
    "jsonata": "^2.0.3"
  }
}
```

```javascript
// js/transformer.js
const _ = require('lodash');
const Jsonata = require('jsonata');

/**
 * Deep merge two objects (Fusion calls this)
 */
function deepMerge(base, override) {
    return _.merge({}, base, override);
}

/**
 * Apply a JSONata transformation
 */
function jsonataTransform(data, expression) {
    try {
        const expr = Jsonata(expression);
        return { success: true, result: expr.evaluate(data) };
    } catch (e) {
        return { success: false, error: e.message };
    }
}

/**
 * Flatten a nested object to dot-notation keys
 */
function flattenObject(obj, prefix = '') {
    return _.transform(obj, (result, value, key) => {
        const newKey = prefix ? `${prefix}.${key}` : key;
        if (_.isPlainObject(value)) {
            Object.assign(result, flattenObject(value, newKey));
        } else {
            result[newKey] = value;
        }
    }, {});
}

/**
 * Group array of objects by a key
 */
function groupByKey(array, key) {
    return _.groupBy(array, key);
}

/**
 * Sort array of objects by multiple keys
 */
function multiSort(array, keys) {
    return _.orderBy(array, keys.map(k => k.field), keys.map(k => k.direction || 'asc'));
}

/**
 * Validate JSON against a JSON Schema
 */
const Ajv = require('ajv');
function validateSchema(data, schema) {
    const ajv = new Ajv({ allErrors: true });
    const validate = ajv.compile(schema);
    const valid = validate(data);
    return {
        valid,
        errors: valid ? null : validate.errors
    };
}

module.exports = {
    deepMerge,
    jsonataTransform,
    flattenObject,
    groupByKey,
    multiSort,
    validateSchema
};
```

```fusion
// src/main.fusion
import fusion:io.{println, eprintln}
import fusion:convert.{ToString}

// Declare JavaScript FFI bindings
@javascript("js/transformer.js")
extern module Transformer:
    extern fn deepMerge(base: Dict<String, Any>, override: Dict<String, Any>) -> Dict<String, Any>
    extern fn jsonataTransform(data: Any, expression: String) -> Dict<String, Any>
    extern fn flattenObject(obj: Dict<String, Any>) -> Dict<String, Any>
    extern fn groupByKey(array: Array<Dict<String, Any>>, key: String) -> Dict<String, Array<Dict<String, Any>>>
    extern fn multiSort(array: Array<Dict<String, Any>>, keys: Array<Dict<String, Any>>) -> Array<Dict<String, Any>>
    extern fn validateSchema(data: Any, schema: Dict<String, Any>) -> Dict<String, Any>

fn main():
    // Example 1: Deep merge configuration
    let base_config = {
        "database": {
            "host": "localhost",
            "port": 5432,
            "pool": { "min": 5, "max": 20 }
        },
        "logging": { "level": "info" }
    }

    let override_config = {
        "database": {
            "host": "production.db.example.com",
            "pool": { "max": 100 }
        },
        "features": { "dark_mode": true }
    }

    let merged = Transformer.deepMerge(base_config, override_config)
    println("=== Merged Configuration ===")
    println("Database host: {merged["database"]["host"]}")
    println("Pool max:      {merged["database"]["pool"]["max"]}")
    println("Logging level: {merged["logging"]["level"]}")
    println("Dark mode:     {merged["features"]["dark_mode"]}")

    // Example 2: JSONata transformation
    let sensor_data = {
        "sensors": [
            { "id": "temp_01", "type": "temperature", "value": 23.5 },
            { "id": "hum_01", "type": "humidity", "value": 65.0 },
            { "id": "temp_02", "type": "temperature", "value": 24.1 },
            { "id": "pres_01", "type": "pressure", "value": 1013.25 }
        ]
    }

    let temp_query = "sensors[type='temperature'].value"
    let result = Transformer.jsonataTransform(sensor_data, temp_query)
    if result["success"] == true:
        println("\n=== JSONata Query Result ===")
        println("Temperature sensors: {result["result"]}")
    else:
        eprintln("JSONata error: {result["error"]}")

    // Example 3: Flatten nested objects
    let nested = {
        "server": {
            "address": "0.0.0.0",
            "ssl": { "enabled": true, "cert_path": "/etc/ssl/cert.pem" }
        }
    }
    let flat = Transformer.flattenObject(nested)
    println("\n=== Flattened Object ===")
    for key, value in flat:
        println("  {key} = {value}")

    // Example 4: Group and sort
    let employees = [
        { "name": "Alice", "dept": "Engineering", "salary": 120000 },
        { "name": "Bob", "dept": "Marketing", "salary": 95000 },
        { "name": "Carol", "dept": "Engineering", "salary": 135000 },
        { "name": "Dave", "dept": "Marketing", "salary": 110000 },
        { "name": "Eve", "dept": "Engineering", "salary": 145000 },
    ]

    let grouped = Transformer.groupByKey(employees, "dept")
    println("\n=== Employees by Department ===")
    for dept, members in grouped:
        println("  {dept}:")
        for m in members:
            println("    - {m["name"]} (${m["salary"]})")

    let sorted = Transformer.multiSort(employees, [
        { "field": "dept", "direction": "asc" },
        { "field": "salary", "direction": "desc" }
    ])
    println("\n=== Sorted by Department, then Salary ===")
    for e in sorted:
        println("  {e["dept"]:12s} {e["name"]:8s} ${e["salary"]}")

    // Example 5: Schema validation
    let user_schema = {
        "type": "object",
        "properties": {
            "name": { "type": "string", "minLength": 1 },
            "age": { "type": "integer", "minimum": 0, "maximum": 150 },
            "email": { "type": "string", "format": "email" }
        },
        "required": ["name", "age", "email"]
    }

    let valid_user = { "name": "Alice", "age": 30, "email": "alice@example.com" }
    let invalid_user = { "name": "", "age": -5, "email": "not-an-email" }

    let v1 = Transformer.validateSchema(valid_user, user_schema)
    let v2 = Transformer.validateSchema(invalid_user, user_schema)
    println("\n=== Schema Validation ===")
    println("Valid user:   {v1["valid"]}")
    println("Invalid user: {v2["valid"]}")
    if v2["valid"] == false:
        println("Errors: {v2["errors"]}")
```

### 6.4 Cross-Language Test Suite

```fusion
// tests/cross_language_integration.test.fusion

import fusion:testing.{Test, assert_eq, assert_near, assert_raises, TestSuite}
import fusion:testing::generators as gen

// --- FFI Bindings ---
@ffi("libimage_processor.so", "image_resize")
extern fn rust_resize(data: &ByteSlice, len: UInt64, w: UInt32, h: UInt32) -> FfiPointer

@ffi("libimage_processor.so", "image_free")
extern fn rust_free(img: FfiPointer)

@python("python/analyzer.py")
extern module PythonAnalyzer:
    extern fn compute_statistics(data: Array<Float64>) -> Dict<String, Any>
    extern fn detect_outliers(data: Array<Float64>, method: String) -> Array<Int64>

@javascript("js/transformer.js")
extern module JSTransformer:
    extern fn flattenObject(obj: Dict<String, Any>) -> Dict<String, Any>
    extern fn validateSchema(data: Any, schema: Dict<String, Any>) -> Dict<String, Any>

// --- Test Suite ---
let suite = TestSuite("Cross-Language Integration")

// Test: Rust FFI boundary type safety
suite.test("Rust FFI: resize preserves pixel count"):
    let data = File.read_bytes("test_image.png").unwrap()
    let img = rust_resize(data.ptr(), data.len() as UInt64, 100, 100)
    // 100x100 RGBA = 40000 bytes
    assert_eq(img.size(), 40000)
    rust_free(img)

// Test: Python FFI numerical correctness
suite.test("Python FFI: statistics match manual calculation"):
    let data = [1.0, 2.0, 3.0, 4.0, 5.0]
    let stats = PythonAnalyzer.compute_statistics(data)
    assert_near(stats["mean"] as Float64, 3.0, 0.0001)
    assert_near(stats["std"] as Float64, 1.5811, 0.001)

// Test: JavaScript FFI data transformation
suite.test("JS FFI: flatten produces correct keys"):
    let nested = { "a": { "b": { "c": 1 } } }
    let flat = JSTransformer.flattenObject(nested)
    assert_eq(flat["a.b.c"], 1)

// Test: Roundtrip across all three languages
suite.test("Roundtrip: Fusion -> Python -> JS -> Fusion"):
    // Fusion generates data
    let original = Array.range(0, 100).map(|i| i as Float64 * 0.1)

    // Python analyzes it
    let stats = PythonAnalyzer.compute_statistics(original)
    let std_dev = stats["std"] as Float64

    // JavaScript validates it
    let validation = JSTransformer.validateSchema(
        stats,
        { "type": "object", "properties": { "std": { "type": "number" } } }
    )
    assert_eq(validation["valid"], true)

    // Fusion receives the validated result
    assert_near(std_dev, 29.0115, 0.01)

// Test: Error handling across boundaries
suite.test("Error propagation: Python exception -> Fusion Result"):
    // Pass invalid data type to Python (should trigger error handling)
    let result = PythonAnalyzer.compute_statistics([])
    // Empty array should return zeros, not crash
    assert_eq(result["count"], 0)

// Test: Performance under load
suite.test("Performance: 10K iterations across FFI boundary"):
    let start = fusion::time::now()
    for _ in 0..10000:
        let data = Array.range(0.0, 100.0, 1.0)
        PythonAnalyzer.compute_statistics(data)
    let elapsed = fusion::time::now() - start
    // Should complete in under 5 seconds
    assert(elapsed.as_secs_f64() < 5.0)
    println("  10K FFI calls completed in {elapsed.as_millis()}ms")
```

---

## Summary

This chapter covered the complete polyglot developer toolkit:

1. **Environment Setup** — Version-pinned toolchains via `mise`/`asdf`, container-based reproducibility, and IDE configuration for 7+ languages.

2. **Interoperability** — FFI as the primary mechanism, with type marshaling tables, calling conventions, and safety rules. WebAssembly as the universal bridge. GraalVM-style shared-memory polyglot contexts.

3. **Build Tools** — `Fusion.toml` as the single manifest. Forge as the cross-language package manager. Build caching and incremental compilation strategies.

4. **Testing** — Unit tests per language, integration tests at FFI boundaries, property-based testing for invariant checking, and mutation testing for test quality. CI/CD pipelines that build and test across all languages.

5. **Data Interchange** — Complete type mapping across Fusion/Python/JS/Rust/Java/Go. Serialization format comparison. Common corruption pitfalls and safe conversion patterns.

6. **Code Examples** — Three complete FFI examples (Python, Rust, JavaScript) with project structures, build instructions, and a cross-language test suite.

The next chapter covers advanced topics: distributed polyglot systems, performance optimization across boundaries, and production deployment patterns.
