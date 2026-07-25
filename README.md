<p align="center">
  <img src="assets/logo.png" alt="Fusion Logo" width="120" height="120">
</p>

<h1 align="center">Fusion v2.0 Vortex</h1>

<h3 align="center">The Polyglot Programming Language</h3>

<p align="center">
  <a href="https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language/releases/tag/v2.0.0"><img src="https://img.shields.io/badge/Version-2.0.0-brightgreen?style=for-the-badge" alt="Version"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="License"></a>
  <a href="#installation"><img src="https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey?style=for-the-badge" alt="Platform"></a>
  <a href="https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language/actions"><img src="https://img.shields.io/badge/Build-Passing-brightgreen?style=for-the-badge" alt="Build"></a>
  <a href="#post-quantum-cryptography"><img src="https://img.shields.io/badge/Security-Post--Quantum-purple?style=for-the-badge" alt="PQC"></a>
</p>

<p align="center">
  <b>A modern, general-purpose, polyglot systems programming language</b><br>
  with post-quantum cryptography, quantum computing, blockchain,<br>
  and 16 advanced programming language theory features.
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#features">Features</a> •
  <a href="#documentation">Documentation</a> •
  <a href="#examples">Examples</a> •
  <a href="#contributing">Contributing</a>
</p>

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

<table>
<tr>
<td><b>Windows</b></td>
<td>

```powershell
# PowerShell (Run as Administrator)
.\installers\windows\install.ps1

# Or using winget
winget install QuantumSecureTechnologiesInc.Fusion
```

</td>
</tr>
<tr>
<td><b>Linux</b></td>
<td>

```bash
# Ubuntu/Debian
sudo dpkg -i installers/linux/debian/fusion-lang.deb

# Fedora/RHEL
sudo rpm -i installers/linux/rpm/fusion-lang.rpm

# Universal
bash installers/linux/install.sh
```

</td>
</tr>
<tr>
<td><b>macOS</b></td>
<td>

```bash
# Homebrew
brew install fusion-lang

# Or using the installer
bash installers/macos/install.sh
```

</td>
</tr>
</table>

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

### Post-Quantum Crypto

```fusion
fn main() -> void {
    let keypair = pqc_hybrid_keygen();
    let signature = pqc_hybrid_sign(keypair, "Hello, Quantum-Safe World!");
    let valid = pqc_hybrid_verify(keypair, "Hello, Quantum-Safe World!", signature);
    println("Signature valid: " + valid);
}
```

### Quantum Computing

```fusion
fn main() -> void {
    let circuit = quantum_circuit(2);
    circuit.h(0);
    circuit.cx(0, 1);
    let result = circuit.measure();
    println("Bell state: " + result);
}
```

### Blockchain

```fusion
fn main() -> void {
    let chain = chain_new(4);
    let tx = tx_new("alice", "bob", 100.0, 0.01);
    let block = pow_mine(block_new(chain_height(chain) + 1, block_hash(chain_latest(chain)), "tx"), pow_new(4));
    let updated = chain_add_block(chain, block_data(block));
    println("Chain length: " + chain_length(updated));
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

<table>
<tr><th>Feature</th><th>Description</th></tr>
<tr><td><b>Algebraic Effects</b></td><td>Side-effect tracking with handlers</td></tr>
<tr><td><b>Linear Types</b></td><td>Resource protocol enforcement</td></tr>
<tr><td><b>Dependent Types</b></td><td>Types depending on values</td></tr>
<tr><td><b>Refinement Types</b></td><td>Logical predicates on types</td></tr>
<tr><td><b>Gradual Typing</b></td><td>Static + dynamic hybrid</td></tr>
<tr><td><b>Guaranteed TCO</b></td><td>Tail-call optimization</td></tr>
<tr><td><b>Continuations</b></td><td>First-class control flow</td></tr>
<tr><td><b>Capability Security</b></td><td>Object-capability model</td></tr>
<tr><td><b>Multiple Dispatch</b></td><td>Method resolution by all args</td></tr>
<tr><td><b>Effect Polymorphism</b></td><td>Generic effect signatures</td></tr>
<tr><td><b>Formal Verification</b></td><td>Compile-time proof hooks</td></tr>
<tr><td><b>Partial Evaluation</b></td><td>Multi-stage programming</td></tr>
<tr><td><b>Actor Model</b></td><td>Built-in actors + supervision</td></tr>
<tr><td><b>Custom Allocators</b></td><td>Per-type memory allocation</td></tr>
<tr><td><b>Unsafe Provenance</b></td><td>Proof requirements for unsafe</td></tr>
<tr><td><b>Blockchain</b></td><td>31 built-in modules</td></tr>
</table>

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

<table>
<tr><th>Tool</th><th>Command</th><th>Description</th></tr>
<tr><td><b>Compiler</b></td><td><code>fuc</code></td><td>Full compilation pipeline</td></tr>
<tr><td><b>CLI</b></td><td><code>fusion</code></td><td>17+ commands (init, build, run, test, fmt, lint, etc.)</td></tr>
<tr><td><b>VS Code Extension</b></td><td>—</td><td>Syntax highlighting, completion, hover, diagnostics</td></tr>
<tr><td><b>Package Manager</b></td><td><code>fusion add/remove</code></td><td>Forge with cross-language deps</td></tr>
<tr><td><b>Formatter</b></td><td><code>fusion fmt</code></td><td>Auto-formatting</td></tr>
<tr><td><b>Linter</b></td><td><code>fusion lint</code></td><td>Policy enforcement</td></tr>
<tr><td><b>Documentation</b></td><td><code>fusion doc</code></td><td>Generate API docs</td></tr>
<tr><td><b>Testing</b></td><td><code>fusion test</code></td><td>Unit + integration tests</td></tr>
</table>

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

<table>
<tr><th>Chapter</th><th>Topic</th></tr>
<tr><td><a href="docs/guides/ch01-getting-started.md">Ch 1</a></td><td>Installation, Hello World</td></tr>
<tr><td><a href="docs/guides/ch02-syntax.md">Ch 2</a></td><td>Variables, types, operators, control flow</td></tr>
<tr><td><a href="docs/guides/ch03-structs-enums.md">Ch 3</a></td><td>Data structures, pattern matching</td></tr>
<tr><td><a href="docs/guides/ch04-memory-safety.md">Ch 4</a></td><td>Ownership, borrowing, Vortex</td></tr>
<tr><td><a href="docs/guides/ch05-generics.md">Ch 5</a></td><td>Generic types, trait bounds</td></tr>
<tr><td><a href="docs/guides/ch06-standard-library.md">Ch 6</a></td><td>I/O, strings, collections</td></tr>
<tr><td><a href="docs/guides/ch07-post-quantum-crypto.md">Ch 7</a></td><td>ML-KEM-768, ML-DSA-65</td></tr>
<tr><td><a href="docs/guides/ch08-quantum-computing.md">Ch 8</a></td><td>Gates, circuits, backends</td></tr>
<tr><td><a href="docs/guides/ch09-machine-learning.md">Ch 9</a></td><td>Tensors, neural networks</td></tr>
<tr><td><a href="docs/guides/ch10-concurrency.md">Ch 10</a></td><td>Threads, async, actors</td></tr>
<tr><td><a href="docs/guides/ch11-webassembly.md">Ch 11</a></td><td>WASM compilation</td></tr>
<tr><td><a href="docs/guides/ch12-tooling.md">Ch 12</a></td><td>CLI, formatter, linter</td></tr>
<tr><td><a href="docs/guides/ch13-advanced.md">Ch 13</a></td><td>16 PLT features</td></tr>
<tr><td><a href="docs/guides/ch14-examples.md">Ch 14</a></td><td>Complete code examples</td></tr>
<tr><td><a href="docs/guides/ch15-reference.md">Ch 15</a></td><td>API reference</td></tr>
<tr><td><a href="docs/guides/ch16-polyglot-interop.md">Ch 16</a></td><td>FFI, type mapping</td></tr>
<tr><td><a href="docs/guides/ch17-fusion-toml.md">Ch 17</a></td><td>Configuration</td></tr>
<tr><td><a href="docs/guides/ch18-compiler-features.md">Ch 18</a></td><td>Feature toggle, witnesses</td></tr>
<tr><td><a href="docs/guides/ch26-blockchain.md">Ch 26</a></td><td>Blockchain development</td></tr>
<tr><td><a href="docs/guides/ch27-null-handling.md">Ch 27</a></td><td>Option/Result types</td></tr>
</table>

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

<small>QuantumSecure Technologies LTD</small>

</div>
