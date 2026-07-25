# Chapter 1: Getting Started

> Your first steps with Fusion v2.0 Vortex — the post-quantum systems language

---

## What is Fusion?

Fusion is a **post-quantum systems programming language** designed for building secure, high-performance software that is resilient against both classical and quantum computing threats. It combines memory safety guarantees (via the Vortex borrow checker), native quantum computing support, integrated post-quantum cryptography primitives, and first-class machine learning capabilities into a single, cohesive language.

### Key Design Principles

- **Memory Safety by Default**: The Vortex borrow checker prevents data races, use-after-free, and null pointer dereferences at compile time using affine type tracking and entropic flow analysis.
- **Post-Quantum Ready**: Built-in hybrid cryptographic primitives (X25519 + ML-KEM-768, Ed25519 + ML-DSA-65) with a 50/50 enforcement policy ensure your code is quantum-safe from day one.
- **Quantum-Native**: First-class support for quantum circuits, gates, simulation, and hybrid quantum-classical programming without external dependencies.
- **ML Integrated**: Native tensor operations, automatic differentiation, neural network layers, and GPU acceleration for machine learning workloads.
- **Systems Performance**: Compiles to native code via LLVM IR or WebAssembly, with zero-cost abstractions and no garbage collector.
- **Expressive Syntax**: A modern, ergonomic syntax inspired by Rust and ML-family languages, with pattern matching, closures, traits, and generics.

### Who Should Use Fusion?

- **Security Engineers** building cryptographic systems that must withstand quantum attacks
- **Systems Programmers** who want memory safety without sacrificing performance
- **Quantum Computing Researchers** who need a real programming language (not just a DSL) for quantum algorithms
- **ML Engineers** who want to write training loops and model definitions in a type-safe language
- **Cloud/Infrastructure Developers** building distributed systems with built-in PQC transport

---

## Installation

### Prerequisites

Fusion requires a C compiler (MSVC on Windows, Clang/GCC on Linux/macOS) and Rust toolchain for building from source.

### Option 1: Install from Binary

Download the pre-built binary for your platform:

```bash
# Windows (PowerShell)
Invoke-WebRequest -Uri "https://releases.fusion-lang.org/fuc-v2.0.0-windows-x64.exe" -OutFile "fuc.exe"

# Linux/macOS
curl -fsSL https://releases.fusion-lang.org/install.sh | sh
```

### Option 2: Build from Source with Cargo

```bash
git clone https://github.com/quantumsecure/fusion-v2.0-vortex.git
cd fusion-v2.0-vortex
cargo install --path crates/fuc
```

### Option 3: Use the Install Script

```bash
# Linux/macOS
chmod +x install.sh
./install.sh

# Windows PowerShell
.\install.ps1
```

### Verify Installation

```bash
fuc --version
# Output: fuc 2.0.0 (vortex)

fuc --help
# Output: Usage: fuc [flags] <input.fu> [-o <output>]
```

### Environment Setup

Add the Fusion binary to your PATH:

```bash
# Windows (PowerShell) - Add to current session
$env:PATH += ";C:\Program Files\Fusion\bin"

# Linux/macOS - Add to .bashrc or .zshrc
export PATH="$HOME/.fusion/bin:$PATH"

# Verify PATH is set correctly
which fuc  # Should output path to fuc
```

### Compiler Flags

| Flag | Description |
|------|-------------|
| `-o <path>` | Set output file path |
| `--opt-level <0-3>` | Optimization level |
| `--target <triple>` | Target triple override |
| `--emit-llvm` | Emit textual LLVM IR |
| `--parse-only` | Parse only (no sema/codegen) |
| `--sema-only` | Semantic analysis only |
| `--emit-bin` | Emit linked executable |
| `--lib` | Compile as library |
| `--no-debug` | Disable DWARF debug info |
| `--link-lib <name>` | Link external library |
| `--lib-path <path>` | Library search path |

---

## Creating Your First Project

### Initialize a New Project

Use the `fusion init` command to scaffold a new project:

```bash
# Create and enter a new project
fusion init my_project
cd my_project
```

This generates the following structure:

```
my_project/
├── Fusion.toml          # Project configuration
├── src/
│   └── main.fu          # Entry point
├── tests/
│   └── test_main.fu     # Test file
└── README.md            # Project documentation
```

### Project Templates

Specify a template for different project types:

```bash
# Console application (default)
fusion init my_console_app --template console

# Library project
fusion init my_lib --template library

# WebAssembly project
fusion init my_wasm --template wasm

# Quantum computing project
fusion init my_quantum --template quantum
```

### Initialize in Existing Directory

```bash
# Initialize in current directory
fusion init --name my_project .
```

---

## Hello World

Create a file called `hello.fu`:

```fusion
fn main() -> int {
    println("Hello, World!");
    return 0;
}
```

Compile and run:

```bash
fuc hello.fu -o hello.exe
./hello.exe
# Output: Hello, World!
```

### Anatomy of a Fusion Program

Every Fusion program requires a `main` function as the entry point:

```fusion
fn main() -> int {
    // 'fn' declares a function
    // 'main' is the required entry point name
    // '-> int' specifies the return type
    // '{ ... }' is the function body
    return 0;  // Programs return 0 on success
}
```

**Code Breakdown:**
- `fn` - Keyword that declares a function
- `main` - Special function name; the program's entry point
- `()` - Parameter list (empty means no parameters)
- `-> int` - Return type annotation (returns an integer)
- `{ ... }` - Function body containing statements
- `return 0;` - Exit with success code (0 = success, non-zero = error)

### A More Interesting Example

```fusion
fn main() -> int {
    let name: string = "Fusion";
    let version: int = 2;

    println("Welcome to %s v%d!", name, version);

    // Variables are immutable by default
    let x: int = 10;
    let y: int = 20;
    let sum: int = x + y;
    println("%d + %d = %d", x, y, sum);

    // Mutable variables use 'mut'
    let mut counter: int = 0;
    counter = counter + 1;
    println("Counter: %d", counter);

    return 0;
}
```

---

## Project Structure

### Fusion.toml Configuration

Fusion projects use a `Fusion.toml` configuration file. Here's a complete reference:

```toml
[project]
name = "my_project"           # Project name (required)
version = "0.1.0"             # Semantic version (required)
edition = "2026"              # Language edition
authors = ["Your Name <you@example.com>"]
description = "A sample project"
license = "MIT"

[build]
entry = "src/main.fu"         # Entry point file
output = "build/"             # Output directory
target = "native"             # "native", "wasm32", or "wasm64"
debug = true                  # Include debug symbols

[dependencies]
# External crate dependencies
# stdlib is always available automatically
# Example: some_crate = "1.0.0"

[dev-dependencies]
# Dependencies only for tests
# test_helper = "0.1.0"

[profile.release]
opt-level = 3                 # Optimization level (0-3)
debug = false                 # Disable debug info
strip = true                  # Strip symbols from binary

[profile.debug]
opt-level = 0                 # No optimization for fast compile
debug = true                  # Full debug info
```

### Key Configuration Sections

**[project]** - Project metadata:
- `name`: Your project's identifier (used for builds and dependencies)
- `version`: Follows semantic versioning (major.minor.patch)
- `edition`: The Fusion language edition to use

**[build]** - Build configuration:
- `entry`: The main source file (default: `src/main.fu`)
- `output`: Where compiled binaries go (default: `build/`)
- `target`: Compilation target (`native` for local, `wasm32` for WebAssembly)

**[dependencies]** - External crates:
- Add dependencies with name and version
- Fusion's standard library is always available

**[profile.release]** and **[profile.debug]** - Build profiles:
- Optimize for speed vs compile time
- Control debug symbol inclusion

### Directory Layout

```
my_project/
├── Fusion.toml          # Project configuration
├── src/
│   ├── main.fu          # Entry point
│   ├── lib.fu           # Library module
│   ├── crypto.fu        # PQC module
│   └── quantum/
│       └── circuits.fu  # Quantum module
├── tests/
│   └── test_main.fu     # Tests
├── build/               # Build output
└── docs/                # Documentation
```

### Directory Layout

```
my_project/
├── Fusion.toml          # Project configuration
├── src/
│   ├── main.fu          # Entry point
│   ├── lib.fu           # Library module
│   ├── crypto.fu        # PQC module
│   └── quantum/
│       └── circuits.fu  # Quantum module
├── tests/
│   └── test_main.fu     # Tests
├── build/               # Build output
└── docs/                # Documentation
```

### Module System

Fusion uses a module system similar to Rust's. Each `.fu` file is a module:

```fusion
// src/main.fu
use std::io;
use crypto::pqc;  // Import from a submodule

mod crypto;  // Declare a module
mod quantum;

fn main() -> int {
    println("Project modules loaded!");
    return 0;
}
```

---

## Compilation Workflow

The Fusion compiler (`fuc`) follows a multi-stage pipeline:

```
Source Code (.fu)
    ↓
Lexer → Tokens
    ↓
Parser → Abstract Syntax Tree (AST)
    ↓
Semantic Analysis → Typed AST
    ↓
[Vortex Borrow Checker] → Safety Validation
    ↓
IR Lowering → Intermediate Representation
    ↓
Optimizer → Optimized IR
    ↓
Code Generation → Native Binary / WASM / LLVM IR
```

### Compilation Modes

```bash
# Full compilation to native binary
fuc src/main.fu -o myapp.exe

# Emit LLVM IR for inspection
fuc src/main.fu --emit-llvm -o output.ll

# Compile to WebAssembly
fuc src/main.fu --target wasm32-unknown-unknown -o output.wasm

# Parse only (syntax checking)
fuc src/main.fu --parse-only

# Semantic analysis only (type checking)
fuc src/main.fu --sema-only

# Compile as library (no main required)
fuc src/lib.fu --lib -o libmylib.a

# Linked executable with external PQC library
fuc src/main.fu --emit-bin --link-lib hypercycle_pqc --lib-path ./libs
```

### Optimization Levels

| Level | Description |
|-------|-------------|
| `-O0` | No optimization (default, fastest compile) |
| `-O1` | Basic optimizations |
| `-O2` | Standard optimizations |
| `-O3` | Aggressive optimizations |

---

## Running Programs

### Direct Execution

```bash
# Compile and run in one step
fuc run src/main.fu

# With arguments
fuc run src/main.fu -- --input data.csv --verbose
```

### Build and Run

```bash
# Build
fuc build --release

# Run the built binary
./build/myapp.exe
```

### Development Mode

```bash
# Watch mode (recompile on file changes)
fuc watch src/main.fu

# Run with Vortex borrow checking enabled
fuc run --vortex src/main.fu

# Run with verbose output
fuc run -v src/main.fu
```

---

## Tips for Beginners

1. **Start simple**: Write a `main` function that prints something, compile it, and verify it works.
2. **Use `println` freely**: It's the easiest way to debug and verify your code is working.
3. **Let the compiler help**: Fusion's error messages are designed to be helpful. Read them carefully.
4. **Immutable by default**: Use `let` for values that don't change. Only use `mut` when you need to.
5. **Check types**: Fusion is statically typed. If you see a type error, it's the compiler helping you avoid bugs.

---

## Quick Reference: All Commands

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `fuc` | Fusion compiler | `fuc src/main.fu` |
| `fuc run` | Compile and run in one step | `fuc run src/main.fu` |
| `fuc build` | Build the project | `fuc build --release` |
| `fuc test` | Run test suite | `fuc test` |
| `fuc init` | Initialize new project | `fusion init my_project` |
| `fuc clean` | Remove build artifacts | `fuc clean` |
| `fuc fmt` | Format source files | `fuc fmt src/` |
| `fuc lint` | Run linter | `fuc lint src/` |
| `fuc doc` | Generate documentation | `fuc doc` |
| `fuc check` | Type-check without building | `fuc check src/main.fu` |

### Compiler Flags

| Flag | Description | Example |
|------|-------------|---------|
| `-o <path>` | Set output file path | `fuc -o myapp.exe src/main.fu` |
| `--opt-level <0-3>` | Optimization level | `fuc --opt-level 2 src/main.fu` |
| `--target <triple>` | Target triple override | `fuc --target wasm32-unknown-unknown` |
| `--emit-llvm` | Emit textual LLVM IR | `fuc --emit-llvm src/main.fu` |
| `--parse-only` | Parse only (no sema/codegen) | `fuc --parse-only src/main.fu` |
| `--sema-only` | Semantic analysis only | `fuc --sema-only src/main.fu` |
| `--emit-bin` | Emit linked executable | `fuc --emit-bin src/main.fu` |
| `--lib` | Compile as library | `fuc --lib src/lib.fu` |
| `--no-debug` | Disable DWARF debug info | `fuc --no-debug src/main.fu` |
| `--link-lib <name>` | Link external library | `fuc --link-lib pqc src/main.fu` |
| `--lib-path <path>` | Library search path | `fuc --lib-path ./libs src/main.fu` |
| `--vortex` | Enable Vortex borrow checker | `fuc run --vortex src/main.fu` |
| `-v, --verbose` | Verbose output | `fuc -v src/main.fu` |

### Project Commands

| Command | Description | Example |
|---------|-------------|---------|
| `fusion init` | Create new project | `fusion init my_app` |
| `fusion init --template` | Use project template | `fusion init my_app --template wasm` |
| `fuc update` | Update dependencies | `fuc update` |
| `fuc add <crate>` | Add dependency | `fuc add crypto_pqc@1.0` |
| `fuc remove <crate>` | Remove dependency | `fuc remove crypto_pqc` |
| `fuc publish` | Publish to registry | `fuc publish` |

### Debugging Commands

| Command | Description | Example |
|---------|-------------|---------|
| `fuc run -v` | Run with verbose output | `fuc run -v src/main.fu` |
| `fuc watch` | Watch for file changes | `fuc watch src/main.fu` |
| `fuc debug` | Run with debugger support | `fuc debug src/main.fu` |

---

## Next Steps

Now that you have Fusion installed and can compile your first program, continue to:

- **Chapter 2: Syntax** — Learn about variables, types, operators, and control flow
- **Chapter 3: Structs and Enums** — Define custom types and pattern matching
- **Chapter 4: Memory Safety** — Understand the Vortex borrow checker
