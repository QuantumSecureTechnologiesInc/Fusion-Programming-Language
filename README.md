<div align="center">

<img src="docs/favicon-32x32.png" alt="Fusion Logo" width="120" height="120">

# Fusion v2.0 Vortex

### The Polyglot Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-2.0.0-green.svg)](ChangeLog.md)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](#installation)
[![Build](https://img.shields.io/badge/Build-Passing-brightgreen.svg)](#building)
[![PQC](https://img.shields.io/badge/Security-Post--Quantum-purple.svg)](#post-quantum-cryptography)

**A modern, general-purpose, polyglot systems programming language with post-quantum cryptography, quantum computing, blockchain, and 16 advanced programming language theory features.**

---

[Installation](#installation) | [Quick Start](#quick-start) | [Features](#features) | [Documentation](#documentation) | [Examples](#examples) | [Contributing](#contributing)

</div>

---

## Why Fusion?

Fusion is not just another programming language — it's a **polyglot systems language** built for the post-quantum era. It combines the safety of Rust, the expressiveness of Python, the performance of C++, and the interoperability of GraalVM, all while enforcing **50/50 hybrid post-quantum cryptography** by default.

### Key Differentiators

| Feature | Fusion | Rust | Python | Go |
|---------|--------|------|--------|-----|
| **Post-Quantum Crypto** | 50/50 hybrid enforced | Manual | Manual | Manual |
| **Quantum Computing** | Built-in simulator + 5 backends | No | Qiskit (external) | No |
| **Blockchain** | 31 modules built-in | No | No | No |
| **Polyglot Interop** | Native FFI to C/Python/JS/Java/Rust | Limited FFI | C extension only | cgo |
| **16 Advanced PLT Features** | Effects, Linear Types, TCO, Actors, etc. | Partial | No | No |
| **Type System** | Static + Gradual + Refinement + Dependent | Static only | Dynamic | Static only |
| **Memory Model** | Ownership + Linear Types + GC | Ownership only | GC only | GC only |

---

## Installation

### Quick Install (Recommended)

#### Windows
```powershell
# PowerShell (Run as Administrator)
.\installers\windows\install.ps1

# Or using winget
winget install QuantumSecureTechnologiesInc.Fusion
```

#### Linux
```bash
# Ubuntu/Debian
sudo dpkg -i installers/linux/debian/fusion-lang.deb

# Fedora/RHEL
sudo rpm -i installers/linux/rpm/fusion-lang.rpm

# Universal
bash installers/linux/install.sh
```

#### macOS
```bash
# Homebrew
brew install fusion-lang

# Or using the installer
bash installers/macos/install.sh
```

### Native Fusion Installer
```bash
# Requires a working Fusion compiler
fusion run installers/windows/install.fu   # Windows
fusion run installers/linux/install.fu     # Linux
fusion run installers/macos/install.fu     # macOS
```

### From Source
```bash
git clone https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language.git
cd Fusion-Programming-Language
cargo build --release
```

---

## Quick Start

### Hello World

```fusion
fn main() -> void {
    println("Hello, Fusion v2.0 Vortex!");
}
```

```bash
fusion init hello
cd hello
fusion run
```

### With Post-Quantum Crypto

```fusion
fn main() -> void {
    // Generate hybrid PQC keypair
    let keypair = pqc_hybrid_keygen();
    
    // Sign with 50/50 hybrid (Ed25519 + ML-DSA-65)
    let signature = pqc_hybrid_sign(keypair, "Hello, Quantum-Safe World!");
    
    // Verify both classical AND post-quantum signatures
    let valid = pqc_hybrid_verify(keypair, "Hello, Quantum-Safe World!", signature);
    println("Signature valid: " + valid);
}
```

### With Quantum Computing

```fusion
fn main() -> void {
    // Create a 2-qubit quantum circuit
    let circuit = quantum_circuit(2);
    
    // Apply Hadamard to qubit 0
    circuit.h(0);
    
    // Apply CNOT (entangle qubits 0 and 1)
    circuit.cx(0, 1);
    
    // Measure all qubits
    let result = circuit.measure();
    println("Measurement: " + result);
}
```

### With Blockchain

```fusion
fn main() -> void {
    // Create a new blockchain with Proof of Work
    let chain = chain_new(4);
    
    // Add a transaction
    let tx = tx_new("alice", "bob", 100.0, 0.01);
    
    // Mine a block
    let block = block_new(chain_height(chain) + 1, block_hash(chain_latest(chain)), "tx_data");
    let mined = pow_mine(block, pow_new(4));
    
    // Add to chain
    let updated_chain = chain_add_block(chain, block_data(mined));
    println("Chain length: " + chain_length(updated_chain));
}
```

---

## Features

### Core Language (Pillar 1: The Soul)

<details>
<summary><b>Turing Complete</b></summary>

- **Sequential Execution** — Statements run in order
- **Conditional Branching** — `if`/`else`/`match` with pattern matching
- **Iteration** — `while`, `for-in`, `break`, `continue`
- **Recursion** — Full recursive function support with guaranteed TCO
</details>

<details>
<summary><b>Data Types</b></summary>

- **Primitives**: `int`, `float`, `bool`, `string`, `char`, `void`
- **Composite**: Arrays `[T; N]`, Structs, Enums (unit/tuple/struct variants)
- **Collections**: Vector, HashMap, HashSet, LinkedList
- **Variables**: `let` (immutable), `let mut` (mutable), `const` (compile-time)
</details>

<details>
<summary><b>Operators</b></summary>

- **Arithmetic**: `+`, `-`, `*`, `/`, `%`
- **Comparison**: `==`, `!=`, `<`, `>`, `<=`, `>=`
- **Logical**: `&&`, `||`, `!`
- **Bitwise**: `&`, `|`, `^`, `<<`, `>>`
- Full precedence table with left-to-right associativity
</details>

<details>
<summary><b>Abstraction</b></summary>

- **Functions** with typed parameters and return values
- **Methods** via `impl` blocks
- **Closures** with capture semantics
- **First-class functions**
- **Traits** for polymorphism
</details>

### Execution Model (Pillar 2: The Engine)

<details>
<summary><b>Multi-Target Compilation</b></summary>

- **LLVM Backend** — Native x86_64/ARM compilation
- **WASM Backend** — WebAssembly for browser/edge
- **Bytecode VM** — Stack-based interpreter
- **Full Pipeline**: Lex → Parse → Sema → Borrow Check → Vortex → IR → SSA → Optimize → Codegen
</details>

<details>
<summary><b>Memory Management</b></summary>

- **Ownership System** — Rust-style move semantics
- **Borrow Checker** — Compile-time reference safety
- **Linear Types** — Exact-once usage for resources
- **Vortex Safety Engine** — Permission state machine
- **Garbage Collection** — For polyglot objects
</details>

<details>
<summary><b>Concurrency</b></summary>

- **Threads** — OS-level threading
- **Fibers** — Green threads with cooperative scheduling
- **Async/Await** — Non-blocking I/O
- **Channels** — Message passing
- **Mutex/RwLock** — Shared state synchronization
- **Atomics** — Lock-free operations
- **Supernova Runtime** — CPU/GPU/QPU dispatch
- **Cortex Scheduler** — AI-powered workload routing
</details>

### Safety & Error Handling (Pillar 3: The Airbags)

<details>
<summary><b>Type System</b></summary>

- **Static Typing** with full inference
- **Gradual Typing** — Mix static and dynamic
- **Refinement Types** — `{x: Int | x > 0}`
- **Dependent Types** — Types depending on values
- **Linear Types** — Resource protocol enforcement
</details>

<details>
<summary><b>Error Handling</b></summary>

- **Result<T, E>** — Recoverable errors with pattern matching
- **Option<T>** — Null safety without null pointers
- **panic/abort** — Unrecoverable errors
- **RAII** — Deterministic resource cleanup
- **try/catch** — Exception-like patterns
- **Assert macros** — Runtime verification
</details>

### Advanced PLT Features (16 Total)

| Feature | Description |
|---------|-------------|
| **Algebraic Effects** | Side-effect tracking with handlers |
| **Linear Types** | Resource protocol enforcement |
| **Dependent Types** | Types depending on values |
| **Refinement Types** | Logical predicates on types |
| **Gradual Typing** | Static + dynamic hybrid |
| **Guaranteed TCO** | Tail-call optimization |
| **Continuations** | First-class control flow |
| **Capability Security** | Object-capability model |
| **Multiple Dispatch** | Method resolution by all args |
| **Effect Polymorphism** | Generic effect signatures |
| **Formal Verification** | Compile-time proof hooks |
| **Partial Evaluation** | Multi-stage programming |
| **Actor Model** | Built-in actors + supervision |
| **Custom Allocators** | Per-type memory allocation |
| **Unsafe Provenance** | Proof requirements for unsafe |
| **Blockchain** | 31 built-in modules |

### Blockchain Development (31 Modules)

<details>
<summary><b>Core & Consensus</b></summary>

- Blocks, Chains, Transactions, Merkle Trees
- Proof of Work (PoW) with difficulty adjustment
- Proof of Stake (PoS) with weighted selection
- Delegated Proof of Stake (DPoS) with voting
- PBFT three-phase consensus
</details>

<details>
<summary><b>Tokens & Contracts</b></summary>

- ERC-20 Fungible Tokens
- ERC-721 NFTs
- ERC-1155 Multi-Token
- Smart Contracts (deploy, call, upgrade)
- Custom Token Standards
</details>

<details>
<summary><b>DeFi & Privacy</b></summary>

- AMM Liquidity Pools (x*y=k)
- Lending/Borrowing
- Stealth Addresses
- Confidential Transactions
- Shielded Pools
- Zero-Knowledge Proofs
</details>

<details>
<summary><b>Governance & Infrastructure</b></summary>

- On-chain Proposals & Voting
- DAOs with Treasury
- Staking & Rewards
- Cross-chain Bridges
- Layer 2 (State Channels, Rollups)
- Oracles
</details>

### Polyglot Interoperability (Pillar 5: Social Skills)

<details>
<summary><b>Language Bridges</b></summary>

- **Python** — Full FFI with type marshaling
- **JavaScript** — V8/QuickJS integration
- **Java** — JNI-style interface
- **Rust** — C FFI bridge with type mapping
- **C/C++** — Direct C ABI calls
</details>

<details>
<summary><b>Interop Features</b></summary>

- **Polyglot API** — `import`, `export`, `eval` across languages
- **Shared Memory** — Zero-copy cross-language data transfer
- **Foreign Proxies** — Transparent access to foreign objects
- **Type Mapping** — Automatic conversion between languages
- **Cross-Language Concurrency** — Thread pools and async bridges
</details>

### Standard Library

<details>
<summary><b>Core Modules</b></summary>

- **I/O** — `println`, `print`, `read_line`, `read_int`
- **Filesystem** — Read, write, append, exists, mkdir, rm
- **Strings** — Length, compare, contains, trim, replace, split
- **Math** — sqrt, sin, cos, pow, log, exp, random, constants
- **Collections** — Vector, HashMap, HashSet, LinkedList
- **Error Handling** — Result, Option, assert, panic, abort
- **JSON** — Serialize/deserialize
- **HTTP** — Request/response
- **Networking** — TCP/UDP
</details>

### Toolchain (Pillar 6: Assembly Line)

| Tool | Command | Description |
|------|---------|-------------|
| **Compiler** | `fuc` | Full compilation pipeline |
| **CLI** | `fusion` | 17+ commands (init, build, run, test, fmt, lint, etc.) |
| **VS Code Extension** | — | Syntax highlighting, completion, hover, diagnostics |
| **Package Manager** | `fusion add/remove` | Forge with cross-language deps |
| **Formatter** | `fusion fmt` | Auto-formatting |
| **Linter** | `fusion lint` | Policy enforcement |
| **Documentation** | `fusion doc` | Generate API docs |
| **Testing** | `fusion test` | Unit + integration tests |

---

## Project Structure

```
Fusion v2.0 Vortex/
├── src/                    # Fusion source code (80+ .fu files)
│   ├── blockchain/         # 31 blockchain modules
│   ├── compiler/           # 19 compiler modules
│   ├── effects/            # Algebraic effects
│   ├── types/              # Linear, dependent, refinement, gradual
│   ├── tco/                # TCO + staging
│   ├── control/            # Continuations, coroutines
│   ├── security/           # Capabilities, provenance
│   ├── dispatch/           # Multiple dispatch
│   ├── actors/             # Actor model
│   ├── safety/             # Memory/type safety
│   ├── meta/               # Generics, macros, reflection
│   ├── concurrency/        # Threads, mutex, channels, async
│   ├── modules/            # Module system
│   ├── observability/      # Logging, metrics, profiling
│   ├── portability/        # Platform, serialization
│   ├── interop/            # Polyglot protocol
│   ├── integration/        # 30 cross-feature functions
│   ├── quantum/            # Quantum computing
│   ├── ml/                 # Machine learning
│   ├── ai/                 # AI inference
│   ├── net/                # HTTP, gRPC, TLS, WebSocket
│   ├── cloud/              # K8s, FaaS
│   ├── mobile/             # iOS, Android
│   ├── web/                # Web runtime
│   ├── runtime/            # Supernova, Cortex, Intent, HAFT
│   └── tests/              # Integration tests
├── stdlib/                 # 22+ standard library modules
├── crates/                 # Rust compiler crates
├── runtime/                # C runtime + Rust crates
├── tools/                  # CLI, VS Code, Forge
├── docs/                   # 27+ guide chapters + spec
├── installers/             # Platform installers
├── grammar/                # ANTLR4 grammar
├── examples/               # Example programs
└── Source Files/           # Reference implementations
```

---

## Documentation

| Chapter | Topic |
|---------|-------|
| [Ch 1: Getting Started](docs/guides/ch01-getting-started.md) | Installation, Hello World |
| [Ch 2: Syntax](docs/guides/ch02-syntax.md) | Variables, types, operators, control flow |
| [Ch 3: Structs & Enums](docs/guides/ch03-structs-enums.md) | Data structures, pattern matching |
| [Ch 4: Memory Safety](docs/guides/ch04-memory-safety.md) | Ownership, borrowing, Vortex |
| [Ch 5: Generics](docs/guides/ch05-generics.md) | Generic types, trait bounds |
| [Ch 6: Standard Library](docs/guides/ch06-standard-library.md) | I/O, strings, collections |
| [Ch 7: Post-Quantum Crypto](docs/guides/ch07-post-quantum-crypto.md) | ML-KEM-768, ML-DSA-65 |
| [Ch 8: Quantum Computing](docs/guides/ch08-quantum-computing.md) | Gates, circuits, backends |
| [Ch 9: Machine Learning](docs/guides/ch09-machine-learning.md) | Tensors, neural networks |
| [Ch 10: Concurrency](docs/guides/ch10-concurrency.md) | Threads, async, actors |
| [Ch 11: WebAssembly](docs/guides/ch11-webassembly.md) | WASM compilation |
| [Ch 12: Tooling](docs/guides/ch12-tooling.md) | CLI, formatter, linter |
| [Ch 13: Advanced](docs/guides/ch13-advanced.md) | 16 PLT features |
| [Ch 14: Examples](docs/guides/ch14-examples.md) | Complete code examples |
| [Ch 15: Reference](docs/guides/ch15-reference.md) | API reference |
| [Ch 16: Polyglot Interop](docs/guides/ch16-polyglot-interop.md) | FFI, type mapping |
| [Ch 17: Fusion.toml](docs/guides/ch17-fusion-toml.md) | Configuration |
| [Ch 18: Compiler Features](docs/guides/ch18-compiler-features.md) | Feature toggle, witnesses |
| [Pillar 1-7](docs/guides/ch19-pillar1-*.md) | The 7 Pillars of Fusion |
| [Ch 26: Blockchain](docs/guides/ch26-blockchain.md) | 31 blockchain modules |
| [Ch 27: Null Handling](docs/guides/ch27-null-handling.md) | Option/Result types |

**Full Reference**: [FUSION_LANGUAGE_SPEC.md](docs/reference/FUSION_LANGUAGE_SPEC.md) (92KB comprehensive specification)

---

## Examples

<details>
<summary><b>Hello World</b></summary>

```fusion
fn main() -> void {
    println("Hello, Fusion v2.0 Vortex!");
}
```
</details>

<details>
<summary><b>Structs & Pattern Matching</b></summary>

```fusion
struct Point {
    x: int,
    y: int,
}

fn classify(p: Point) -> string {
    match p.x {
        0 => "on y-axis",
        _ => "general point",
    }
}

fn main() -> void {
    let p = Point { x: 3, y: 4 };
    println(classify(p));
}
```
</details>

<details>
<summary><b>Post-Quantum Cryptography</b></summary>

```fusion
fn main() -> void {
    let keypair = pqc_hybrid_keygen();
    let message = "Quantum-safe message";
    let signature = pqc_hybrid_sign(keypair, message);
    let valid = pqc_hybrid_verify(keypair, message, signature);
    println("Verified: " + valid);
}
```
</details>

<details>
<summary><b>Quantum Circuit</b></summary>

```fusion
fn main() -> void {
    let circuit = quantum_circuit(2);
    circuit.h(0);
    circuit.cx(0, 1);
    let result = circuit.measure();
    println("Bell state: " + result);
}
```
</details>

<details>
<summary><b>Blockchain</b></summary>

```fusion
fn main() -> void {
    let chain = chain_new(4);
    let tx = tx_new("alice", "bob", 100.0, 0.01);
    let block = pow_mine(block_new(chain_height(chain) + 1, block_hash(chain_latest(chain)), "tx"), pow_new(4));
    let updated = chain_add_block(chain, block_data(block));
    println("Chain length: " + chain_length(updated));
}
```
</details>

<details>
<summary><b>AI Model Inference</b></summary>

```fusion
fn main() -> void {
    let model = fusion_llm_load_model("llama3");
    let response = fusion_llm_generate(model, "What is quantum computing?", 100);
    println(response);
}
```
</details>

<details>
<summary><b>Blockchain Token</b></summary>

```fusion
fn main() -> void {
    let token = erc20_new("Fusion Token", "FUS", 1000000.0, "owner");
    let transferred = erc20_transfer(token, "owner", "alice", 100.0);
    println("Alice balance: " + erc20_balance_of(transferred, "alice"));
}
```
</details>

---

## Building

### Prerequisites

- Rust toolchain ([rustup](https://rustup.rs/))
- LLVM (for native compilation)
- CMake (optional, for C runtime)

### Build Commands

```bash
# Debug build
fusion build

# Release build
fusion build --release

# Build for WASM
fusion build --target wasm

# Run tests
fusion test

# Format code
fusion fmt

# Lint code
fusion lint
```

### Environment Variables

```bash
# Development mode (allow cargo fallback)
export FUSION_FLUX_ENABLED=false
export ALLOW_CARGO_FALLBACK=true

# Production mode (strict enforcement)
export FUSION_STRICT_MODE=true
```

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/guides/CONTRIBUTING.md).

### Development Setup

```bash
git clone https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language.git
cd Fusion-Programming-Language
cargo build
cargo test
```

### Building Installers

```bash
# Windows (NSIS)
makensis installers/windows/Fusion-Setup.nsi

# Linux (Debian)
dpkg-deb --build installers/linux/debian fusion-lang.deb

# macOS
pkgbuild --root installers/macos/root --identifier com.quantumsecure.fusion Fusion.pkg
```

---

## Community

- **GitHub**: [QuantumSecureTechnologiesInc/Fusion-Programming-Language](https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language)
- **Issues**: [Report a bug](https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language/issues)
- **Discussions**: [Join the conversation](https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language/discussions)

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Built with post-quantum security for the quantum computing era**

[<img src="https://img.shields.io/badge/Fusion-v2.0.0-purple?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZD0iTTEyIDJMMyAxMmg4bDYtMTJaIiBmaWxsPSIjRkZGRkZGIi8+PC9zdmc+" alt="Fusion">](https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language)

</div>
