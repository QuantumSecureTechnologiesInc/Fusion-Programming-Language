# Fusion v2.0 Vortex Programming Language: User Guide

**Version**: v0.2.0-beta.1 (Bridge Connected)
**Date**: January 28, 2026
**Status**: Early Development – Core Compiler Functional
**Publisher**: Quantum Secure Technologies Inc.

---

## ⚠️ DOCUMENTATION STATUS NOTE

This guide describes features across the full development roadmap. Many features listed here are **aspirational or partially implemented**. The project is in early development.

| Status | Meaning |
| :--- | :--- |
| [IMPLEMENTED] | Working code exists and has been tested |
| [PARTIAL] | Some code exists; not fully functional |
| [ASPIRATIONAL] | Documented as a goal; no working implementation yet |

**What actually works today:**
- Compiler frontend: lexer, parser, type checker, bytecode VM (Rust-based)
- Self-hosting compiler framework (scaffold in `.fu` files)
- Quantum circuit builder and local simulator
- Basic tensor operations and autodiff
- Standard library: collections, strings, IO, option/result, file I/O
- PQC hybrid crypto via C FFI bindings
- Basic LSP implementation
- Flux-Resolve package manager (basic)

**What does NOT work yet:**
- Quantum hardware backends (IBM, AWS Braket)
- Native LLM implementations (Llama 3, Mistral, BERT)
- GPU/CUDA integration
- AI-powered scheduler (Cortex/Supernova)
- Kubernetes/FaaS/Enterprise deployment
- LLVM native backend (bytecode VM only)
- WebAssembly compilation target
- Zero-knowledge proofs
- Visual Compiler

---

## Introduction

Fusion v2.0 Vortex is the world's **first self-hosting, quantum-native, AI-integrated systems programming language** that combines the ease of Python with the performance of Rust, while adding native support for **Quantum Computing**, **AI/ML**, and **Enterprise Infrastructure**.

### What Makes Fusion v2.0 Vortex Unique

- **[PARTIAL] Self-Hosting Compiler**: Framework exists in `.fu` files; primary compiler is Rust-based
- **[PARTIAL] Unified Stack**: Classical and quantum code in one language (basic quantum simulator works)
- **[ASPIRATIONAL] 250+ Built-in Crates**: Comprehensive ecosystem across 6 archetypes (many crates are stubs)
- **[PARTIAL] Vortex Entropy Engine**: Basic PQC entropy via C FFI bindings
- **[ASPIRATIONAL] Supernova Runtime v3.0**: Automatic CPU/GPU/QPU dispatch (basic task scheduler exists)
- **[ASPIRATIONAL] Multi-Backend**: LLVM for native execution, WebAssembly for web deployment (bytecode VM only)
- **[ASPIRATIONAL] Production-Ready**: Full enterprise tooling (K8s, FaaS, Security, Telemetry)

---

## Getting Started

### Installation

````bash

# Install Fusion toolchain

./install.sh

# Verify installation

fusion --version
```text

### Your First Program

```fusion
fn main():
    print("Hello, Fusion v1.0!")
```text

**Compile and Run**:

```bash
fusion build main.fu
./main
```text

---

## Core Language Features

### 1. Variables and Types

Fusion supports both type inference and explicit typing:

```fusion
let x = 10              // Inferred: int
let y: float = 3.14     // Explicit: float
let name = "Fusion"     // Inferred: string
let mut counter = 0     // Mutable variable
```text

### 2. Control Flow

**Conditionals**:

```fusion
if x > 5:
    print("Greater than 5")
elif x == 5:
    print("Exactly 5")
else:
    print("Less than 5")
```text

**Loops**:

```fusion
// Range-based for loop
for i in 0..10:
    print(i)

// While loop
while counter < 100:
    counter += 1

// Iterators
let numbers = [1, 2, 3, 4, 5]
for num in numbers:
    print(num * 2)
```text

### 3. Functions

```fusion
fn add(a: int, b: int) -> int:
    return a + b

fn greet(name: string):
    print("Hello, " + name + "!")

// Generic functions
fn identity<T>(value: T) -> T:
    return value
```text

### 4. Classes and Structs

```fusion
class Point:
    x: float
    y: float

    fn new(x: float, y: float) -> Point:
        return Point { x, y }

    fn distance(self) -> float:
        return sqrt(self.x * self.x + self.y * self.y)
```text

---

## Advanced Features

### Quantum Computing [PARTIAL]

Fusion has **native quantum support** with a local simulator. Hardware backends (IBM, AWS) are NOT implemented:

```fusion
import quantum.circuits
import quantum.backends.ibm

fn quantum_hello():
    // Create a qubit in superposition
    let q = Qubit::new()
    h(q)  // Hadamard gate

    // Measure
    let result = measure(q)
    print("Quantum result: " + result)
```text

**Supported Backends**:
* `quantum.backends.simulator` - Local simulation [IMPLEMENTED]
* `quantum.backends.ibm` - IBM Quantum Experience [ASPIRATIONAL]
* `quantum.backends.aws` - Amazon Braket [ASPIRATIONAL]

### AI & Machine Learning [PARTIAL]

Basic tensor operations and autodiff exist. Training, inference, and model implementations are NOT implemented:

```fusion
import ai.models.llama
import ai.training

fn train_model():
    // Load pre-trained model
    let model = Llama3::load("7b-chat")

    // Configure training
    let trainer = Trainer::new(model)
    trainer.set_learning_rate(0.0001)

    // Train
    trainer.fit("dataset.jsonl", epochs=3)

    // Save
    model.save("fine-tuned-model")
```text

**Available Models** [ASPIRATIONAL]:
* `ai.models.llama` - Llama 3 architecture (not implemented)
* `ai.models.mistral` - Mistral AI models (not implemented)
* `ai.models.bert` - BERT for NLP (not implemented)

**Serving Providers (Fusion.toml)** [ASPIRATIONAL]:
* `ollama` - Local inference runtime (not implemented)
* `qwen` - Qwen API (not implemented)
* `deepseek` - DeepSeek API (not implemented)
* `gpt-oss` - GPT-OSS compatible endpoints (not implemented)
* `mistral` - Mistral API (not implemented)
* `phi` - Microsoft Phi endpoints (not implemented)
* `gemma` - Gemma endpoints (not implemented)
* `openai` - OpenAI-compatible servers (not implemented)

Example configuration:

```toml
[ai]
provider = "ollama"

[ai.ollama]
base_url = "http://localhost:11434"
model = "llama3.1:8b"
```text

### Collections

```fusion
import std.collections

fn use_collections():
    // HashMap
    let mut scores = HashMap<string, int>::new()
    scores.insert("Alice", 100)
    scores.insert("Bob", 95)

    // HashSet
    let mut unique = HashSet<int>::new()
    unique.insert(1)
    unique.insert(2)

    // Iteration
    for (name, score) in scores:
        print(name + ": " + score)
```text

### Non‑Fusion Components (Interop Layer)

Fusion is a `.fu` language, but the toolchain intentionally includes **non‑Fusion components** that remain compatible and are invoked by the build driver:

- **C runtimes**: `runtime.c`, ARC runtime, entropic checker runtime
  Expose stable symbols like `panic` and allocation helpers.
- **Scripts**: packaging and bootstrap helpers (`install.sh`, PowerShell, Python)
  Used for toolchain distribution and sysroot setup.
- **Editor tooling**: VS Code extension + UI assets
  Kept as JS/TS and packaged separately by `fusion`.

These pieces are wired into the Fusion build flow and ship as part of the official toolchain.

- **Interop store**: target location `dist/lib/fusion/interop` (not yet built); exported via `FUSION_INTEROP`

---

## Module System

### Project Structure

```text
my-project/
├── main.fu
├── utils.fu
└── math/
    ├── mod.fu
    └── algebra.fu
```text

### Importing Modules

```fusion
// main.fu
import utils
import math.algebra

fn main():
    utils::helper()
    let result = algebra::solve(10)
```text

---

## Building and Deployment

### Compilation Targets

**Native (LLVM)**:

```bash
fusion build main.fu --release
```text

**WebAssembly**:

```bash
fusion build main.fu --target wasm -o app.wasm
```text

### Multi-File Projects

```bash
fusion build --project my-project/
```text

---

## IDE Support

Fusion includes a **Language Server Protocol (LSP)** for professional IDE integration:

* ✅ Real-time diagnostics
* ✅ Auto-completion
* ✅ Go-to-definition
* ✅ Inline documentation
* ✅ Code formatting

**VS Code Extension**:

```bash
code --install-extension fusion-language-1.0.0.vsix
```text

---

## Enterprise Features

### Cloud Deployment

```fusion
import fusion.faas

fn handler(request: Request) -> Response:
    return Response::ok("Hello from Fusion FaaS!")

// Deploy to Kubernetes
fusion deploy --k8s production
```text

### Telemetry

```fusion
import fusion.telemetry

fn monitored_operation():
    let span = telemetry::start_span("operation")
    // Your code here
    span.end()
```text

---

## Package Management

Fusion includes **Flux-Resolve**, a deterministic dependency manager:

```bash

# Install a package

fusion add fusion-http

# Update dependencies

fusion update

# Build with dependencies

fusion build
```text

---

## Next Steps

1. **Tutorials**: See `/docs/tutorials` for step-by-step guides
2. **Examples**: Browse `/examples` for real-world applications
3. **API Reference**: Visit `/docs/references` for complete API documentation
4. **Community**: Join our Discord and GitHub discussions

---

**Generated by**: Fusion v2.0 Vortex Toolchain
**Document Version**: 2.0.0
**Last Updated**: January 28, 2026
````
