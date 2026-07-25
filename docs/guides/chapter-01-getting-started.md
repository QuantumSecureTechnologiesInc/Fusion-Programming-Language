> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 1: Getting Started

Welcome to *The Fusion v2.0 Vortex Programming Language*! This book will teach you everything you need to know to become a proficient Fusion developer, from writing your first program to building sophisticated applications that leverage classical computing, artificial intelligence, and quantum computing all from a single, unified codebase.

Fusion represents a new paradigm in programming language design. While most languages specialise in one domain systems programming, data science, or web development Fusion is built from the ground up to handle the computational challenges of tomorrow. It combines the ergonomics and readability of Python with the performance and memory safety of Rust, whilst adding native support for tensors, quantum circuits, and post-quantum cryptography.

This chapter will guide you through:

- Understanding what makes Fusion unique
- Installing the Fusion toolchain on your system
- Writing, compiling, and running your first Fusion program
- Using the package manager to create and manage projects

---

## 1.1 What is Fusion?

Fusion is a statically-typed, compiled programming language designed for the **Tri-brid computing era**—a world where classical processors, AI accelerators, and quantum computers work together to solve problems that no single computing paradigm could address alone.

### 1.1.1 The Tri-brid Computing Philosophy

Modern computing is evolving beyond the traditional CPU-centric model. Consider these trends:

**Classical Computing** remains essential for sequential logic, system-level programming, I/O operations, and business logic. Traditional processors excel at branching, precise arithmetic, and memory-intensive operations.

**AI and Machine Learning** have revolutionised pattern recognition, optimisation, and decision-making. Tensor operations the mathematical foundation of deep learning require specialised hardware like GPUs and TPUs for efficient execution.

**Quantum Computing** offers exponential speedups for specific problems: cryptography, molecular simulation, optimisation, and sampling. Quantum algorithms like Shor's (factoring) and Grover's (search) have no classical equivalents that match their theoretical performance.

Fusion recognises that the future belongs to applications that can seamlessly orchestrate all three paradigms. Rather than forcing developers to switch between Python (for ML), Rust (for systems), and Qiskit (for quantum), Fusion provides a unified language where:

- **Classical code** compiles to highly optimised LLVM IR
- **Tensor operations** automatically offload to GPUs via CUDA or Metal
- **Quantum circuits** execute on simulators or real quantum hardware (IBM Quantum, AWS Braket)

### 1.1.2 Why Fusion Exists

The creators of Fusion identified several pain points in existing languages:

1. **Fragmentation**: Building a quantum-enhanced ML application might require Python, C++, Rust, and multiple SDKs each with different build systems, type systems, and idioms.

2. **Security**: Quantum computers threaten current cryptographic standards. Few languages offer post-quantum cryptography as a first-class feature.

3. **Performance**: Dynamic languages sacrifice speed for convenience. Compiled languages often sacrifice productivity for performance.

4. **Safety**: Memory bugs remain the leading cause of security vulnerabilities in systems code.

Fusion addresses each of these:

| Challenge     | Fusion's Solution                                         |
|:------------- |:--------------------------------------------------------- |
| Fragmentation | One language, three paradigms                             |
| Security      | Built-in post-quantum cryptography (ML-KEM, ML-DSA)       |
| Performance   | LLVM-backed compilation, zero-cost abstractions           |
| Safety        | Ownership system, borrow checker, compile-time guarantees |

### 1.1.3 Who Should Use Fusion?

Fusion is designed for:

- **Systems programmers** who want Rust-like safety without the steep learning curve
- **Machine learning engineers** seeking native tensor support with GPU acceleration
- **Quantum computing researchers** who need a typed, safe environment for circuit design
- **Security professionals** building quantum-resistant applications
- **Full-stack developers** who want one language from kernel to cloud

If you're curious about any of these domains or want a language that will remain relevant as computing evolves Fusion is for you.

---

## 1.2 Installation

Fusion provides pre-built toolchains for all major operating systems. The installation process takes about five minutes and leaves you with everything needed to build and run Fusion programs.

### 1.2.1 System Requirements

**Minimum Requirements**:

- CPU: Any 64-bit processor (x86-64, ARM64, or RISC-V)
- RAM: 4 GB
- Storage: 2 GB free space
- Operating System: Windows 10+, macOS 12+, or Linux with kernel 5.15+

**Recommended for AI/ML Development**:

- CPU: 8+ cores, preferably with AVX-512 support
- RAM: 32 GB
- GPU: NVIDIA RTX 3000+ series (for CUDA acceleration)
- Storage: 50 GB SSD (for model weights and datasets)

**For Quantum Development**:

- An IBM Quantum or AWS Braket account (for hardware access)
- 32 GB RAM (for local simulation of 20+ qubits)

### 1.2.2 Installing on Linux and macOS

Open a terminal and run the official installation script:

```bash
curl -fsSL https://sh.fusion-lang.org | sh

```text

This script downloads the latest stable release and installs it to `~/.fusion`. It also adds the Fusion binaries to your PATH by modifying your shell configuration file (`.bashrc`, `.zshrc`, or similar).

After installation, restart your shell or run:

```bash
source ~/.fusion/env

```text

Verify the installation:

```bash
fusion --version

```text

You should see output like:

```text
fusion 1.0.0 (stable)

```text

### 1.2.3 Installing on Windows

Download the Windows installer from [fusion-lang.org/install](https://fusion-lang.org/install) and run it. The installer:

1. Installs the Fusion toolchain to `C:\Program Files\Fusion`
2. Adds Fusion to your system PATH
3. Optionally installs VS Code integration

Alternatively, use Windows Subsystem for Linux (WSL2) and follow the Linux instructions.

After installation, open a new PowerShell or Command Prompt window and verify:

```powershell
fusion --version

```text

### 1.2.4 Installing with Package Managers

**macOS (Homebrew)**:

```bash
brew install fusion-lang

```text

**Linux (Arch)**:

```bash
pacman -S fusion

```text

**Linux (Fedora)**:

```bash
dnf install fusion-lang

```text

### 1.2.5 Updating Fusion

To update to the latest version:

```bash
fusion self update

```text

This downloads the latest release and replaces your current installation whilst preserving configuration.

### 1.2.6 IDE Setup

**Visual Studio Code** is the recommended editor. Install the official extension:

```bash
code --install-extension fusion-lang.fusion-language

```text

The extension provides:

- Syntax highlighting
- Real-time error detection
- Code completion and IntelliSense
- Go-to-definition and find-references
- Integrated debugging
- Built-in formatter

**Other Editors**: Fusion includes a Language Server Protocol (LSP) implementation that works with any LSP-compatible editor, including Vim, Neovim, Emacs, and JetBrains IDEs.

---

## 1.3 Hello, World!

With the toolchain installed, let's write and run your first Fusion program. The venerable "Hello, World!" tradition continues in Fusion, though with a twist that hints at the language's philosophy.

### 1.3.1 Creating a Project

Fusion uses a project-based structure. Create a new project with:

```bash
fusion new hello_world
cd hello_world

```text

This creates the following structure:

```text
hello_world/
├── fusion.toml        # Project manifest
├── src/
│   └── main.fu        # Entry point
└── tests/
    └── integration.fu # Test file

```text

The `fusion.toml` file defines your project's metadata and dependencies:

```toml
[package]
name = "hello_world"
version = "0.1.0"
edition = "2025"
authors = ["Your Name <you@example.com>"]

[dependencies]

# Add dependencies here

```text

### 1.3.2 Writing the Program

Open `src/main.fu` in your editor. You'll see a template:

```fusion
fn main() {
    println("Hello, Fusion!")
}

```text

Let's examine this line by line:

**`fn main()`**: The `fn` keyword declares a function. Every Fusion executable must have a `main` function—this is where execution begins. The parentheses hold parameters (none in this case).

**`{` and `}`**: Curly braces delimit the function body. Fusion uses explicit blocks rather than significant whitespace.

**`println("Hello, Fusion!")`**: This calls the built-in `println` function, which prints text followed by a newline. Strings in Fusion are enclosed in double quotes.

Notice: **no semicolons are required**. Fusion treats newlines as statement terminators. You may add semicolons if you prefer—they're optional and have no effect.

### 1.3.3 Building and Running

From your project directory, compile and run with a single command:

```bash
fusion run

```text

You should see:

```text
   Compiling hello_world v0.1.0
    Finished dev [unoptimised + debuginfo] in 0.23s
     Running `target/debug/hello_world`
Hello, Fusion!

```text

Congratulations! You've just compiled and executed your first Fusion program.

### 1.3.4 Understanding the Build Process

When you run `fusion run`, several things happen:

1. **Parsing**: The source file is parsed into an Abstract Syntax Tree (AST)
2. **Type Checking**: The compiler verifies that types match and ownership rules are satisfied
3. **IR Generation**: The AST is lowered to Fusion Intermediate Representation
4. **Optimisation**: Fifty-plus optimisation passes refine the IR
5. **Code Generation**: LLVM generates platform-specific machine code
6. **Linking**: The linker produces an executable binary

The result is a native executable comparable in performance to C or Rust.

### 1.3.5 Build Modes

**Development Build** (default):

```bash
fusion build

```text

Fast compilation, includes debug symbols, minimal optimisation. Use during development.

**Release Build**:

```bash
fusion build --release

```text

Slower compilation, aggressive optimisation (LTO, inlining), smaller binaries. Use for deployment.

**WebAssembly Target**:

```bash
fusion build --target wasm32-unknown-unknown

```text

Outputs a `.wasm` file for browser or edge deployment.

---

## 1.4 How Fusion Code Works

Before moving forward, let's understand the fundamentals of how Fusion programmes are structured and executed.

### 1.4.1 Functions

Functions are the building blocks of Fusion programs. A function declaration consists of:

```fusion
fn function_name(parameter1: Type1, parameter2: Type2) -> ReturnType {
    // body
}

```text

For example:

```fusion
fn add(a: int, b: int) -> int {
    a + b
}

fn main() {
    let result = add(5, 3)
    println("5 + 3 = {}", result)
}

```text

Key points:

- Parameters require type annotations
- The return type follows `->` (omit for functions returning nothing)
- The last expression in a function is implicitly returned
- Use `return` for early exit

### 1.4.2 Variables and Mutability

Fusion variables are **immutable by default**—once assigned, they cannot change:

```fusion
let x = 10
x = 20  // Error: cannot assign to immutable variable `x`

```text

To create a mutable variable, use `mut`:

```fusion
let mut x = 10
x = 20  // OK

```text

This design prevents accidental modification and makes code easier to reason about. When you see `mut`, you know the value will change—it's an explicit signal in the code.

### 1.4.3 Type Inference

Fusion has powerful type inference. You often don't need explicit type annotations:

```fusion
let name = "Fusion"      // Inferred: String
let count = 42           // Inferred: int
let pi = 3.14159         // Inferred: float
let enabled = true       // Inferred: bool

```text

But you can always specify types explicitly:

```fusion
let name: String = "Fusion"
let count: i32 = 42

```text

### 1.4.4 Comments

```fusion
// Single-line comment

/*

* Multi-line
* comment
  */

/// Documentation comment (for functions, types, etc.)
fn documented_function() {
    // Implementation
}

```text

Documentation comments (starting with `///` or `/** */`) are extracted by the `fusion doc` command to generate API documentation.

---

## 1.5 The Fusion Ecosystem

Fusion is more than just a language—it's a complete development ecosystem.

### 1.5.1 The Fusion CLI

The `fusion` command provides everything you need:

| Command             | Description                       |
|:------------------- |:--------------------------------- |
| `fusion new <name>` | Create a new project              |
| `fusion build`      | Compile the project               |
| `fusion run`        | Build and execute                 |
| `fusion test`       | Run tests                         |
| `fusion check`      | Check for errors without building |
| `fusion fmt`        | Format source code                |
| `fusion clippy`     | Run lints                         |
| `fusion doc`        | Generate documentation            |
| `fusion add <pkg>`  | Add a dependency                  |
| `fusion publish`    | Publish to the registry           |

### 1.5.2 The Package Registry

Fusion packages (called *crates*) are hosted at [registry.fusion-lang.org](https://registry.fusion-lang.org). With 140+ crates covering AI, quantum computing, networking, security, and more, you can generally find a high-quality solution for any problem.

Add a dependency to your project:

```bash
fusion add serde

```text

This modifies `fusion.toml`:

```toml
[dependencies]
serde = "1.0"

```text

### 1.5.3 The Standard Library

Fusion ships with a comprehensive standard library including:

- **Collections**: `Vec`, `HashMap`, `HashSet`, `BinaryHeap`
- **I/O**: File operations, networking, streams
- **Concurrency**: Threads, channels, async/await
- **Text**: String processing, regular expressions
- **Time**: Dates, durations, timers

Unlike languages that require third-party packages for basic tasks, Fusion's standard library is batteries-included.

---

## 1.6 Summary

In this chapter, you learned:

- **Fusion's philosophy**: A unified language for classical, AI, and quantum computing
- **Installation**: How to set up the Fusion toolchain on your system
- **Hello World**: Creating, building, and running your first project
- **Fundamentals**: Functions, variables, mutability, and type inference
- **Ecosystem**: CLI commands, package management, and the standard library

You're now ready to write real Fusion code. In the next chapter, we'll build a more substantial program a number guessing game that introduces control flow, user input, and random number generation.

---

## 1.7 Exercises

1. **Hello, You**: Modify `main.fu` to print your name instead of "Fusion".

2. **Temperature Converter**: Write a function that converts Celsius to Fahrenheit. The formula is `F = C × 9/5 + 32`.

3. **Multiple Files**: Create a second source file and call a function from it in `main.fu`.

4. **Explore Documentation**: Run `fusion doc --std --open` to explore the standard library documentation in your browser.

---

[Next: Chapter 2 - Programming a Guessing Game →](./chapter-02-guessing-game.md)
