# Chapter 12: Tooling

> Language Server Protocol, formatter, package manager, test framework, profiler, linter, CLI commands, VS Code extension, and configuration

---

## Language Server Protocol (LSP)

Fusion provides a full-featured language server for IDE integration.

### Features

- **Autocomplete**: Context-aware code completion
- **Go to Definition**: Jump to function/struct definitions
- **Find References**: Find all usages of a symbol
- **Hover Documentation**: Show docs on hover
- **Diagnostics**: Real-time error checking
- **Code Actions**: Quick fixes and refactoring
- **Inlay Hints**: Type and return value hints
- **Signature Help**: Function parameter hints
- **Code Lens**: Run/test inline buttons
- **Semantic Highlighting**: Advanced syntax coloring
- **Workspace Symbols**: Search project-wide symbols
- **Call Hierarchy**: Navigate call chains
- **Type Hierarchy**: Navigate type relationships

### IDE Setup

```bash
# The LSP server is included with the Fusion toolchain
fuc lsp --stdio

# Or configure your IDE to use it
# VS Code: Install the Fusion extension
# Neovim: Use nvim-lspconfig with fusion_lsp
# Emacs: Use eglot or lsp-mode
# Sublime: Use LSP-fusion
```

### LSP Configuration

```json
// .fusion-lsp.json
{
  "compiler": {
    "path": "fuc",
    "args": ["--vortex"]
  },
  "diagnostics": {
    "enable": true,
    "lintLevel": "warning",
    "debounceMs": 250
  },
  "completion": {
    "enableSnippets": true,
    "autoImport": true,
    "triggerChars": [".", "->", "::"]
  },
  "inlayHints": {
    "typeHints": true,
    "parameterHints": true,
    "chainingHints": true
  },
  "semanticHighlighting": {
    "enable": true
  }
}
```

---

## Fusion CLI Commands (`fuc`)

The `fuc` compiler is the primary command-line tool for Fusion projects.

### Complete Command Reference

| Command | Description |
|---------|-------------|
| `fuc build` | Compile a Fusion program |
| `fuc run` | Compile and execute a program |
| `fuc check` | Run type checking without emitting |
| `fuc fmt` | Format source code |
| `fuc lint` | Lint source code |
| `fuc test` | Run tests |
| `fuc bench` | Run benchmarks |
| `fuc doc` | Generate documentation |
| `fuc clean` | Remove build artifacts |
| `fuc new` | Create a new project |
| `fuc init` | Initialize a project in existing directory |
| `fuc build --lib` | Compile as a library |
| `fuc build --release` | Release build with optimizations |
| `fuc build --debug` | Debug build with symbols |
| `fuc build --wasm` | Compile to WebAssembly |
| `fuc profile` | Profile a program |
| `fuc debug` | Run with debugger |
| `fuc lsp` | Start language server |

### `fuc build`

```bash
# Basic build
fuc build src/main.fu

# Release build
fuc build --release src/main.fu

# Debug build
fuc build --debug src/main.fu

# Build with optimization level
fuc build --opt-level 2 src/main.fu

# Build as library
fuc build --lib src/lib.fu

# Build to WebAssembly
fuc build --wasm src/main.fu

# Build with Vortex borrow checking
fuc build --vortex src/main.fu

# Build with target triple
fuc build --target x86_64-pc-windows-msvc src/main.fu

# Emit LLVM IR
fuc build --emit-llvm src/main.fu

# Build with custom output
fuc build -o bin/myapp src/main.fu

# Build with custom config
fuc build --config release.toml src/main.fu
```

### `fuc run`

```bash
# Run a program
fuc run src/main.fu

# Run with arguments
fuc run src/main.fu -- arg1 arg2 arg3

# Run in release mode
fuc run --release src/main.fu

# Run with Vortex checking
fuc run --vortex src/main.fu

# Run with environment variable
FUSION_LOG=debug fuc run src/main.fu
```

### `fuc check`

```bash
# Type-check without emitting
fuc check src/main.fu

# Check entire project
fuc check src/

# Check with strict mode
fuc check --strict src/main.fu

# Check only specific module
fuc check src/quantum/factor.fu
```

### `fuc fmt`

```bash
# Format a file
fuc fmt src/main.fu

# Format a directory
fuc fmt src/

# Check formatting without modifying
fuc fmt --check src/

# Format with specific style
fuc fmt --style=standard src/main.fu

# Format with custom config
fuc fmt --config .fusionfmt.toml src/
```

### `fuc lint`

```bash
# Lint a file
fuc lint src/main.fu

# Lint a directory
fuc lint src/

# Lint with specific rules
fuc lint --rules=security,performance src/

# Fix auto-fixable issues
fuc lint --fix src/

# Lint with custom config
fuc lint --config .fusionlint.toml src/

# Output as JSON (for CI)
fuc lint --format=json src/
```

### `fuc test`

```bash
# Run all tests
fuc test

# Run specific test
fuc test test_addition

# Run tests with verbose output
fuc test --verbose

# Run tests in parallel
fuc test --parallel

# Run tests with coverage
fuc test --coverage

# Run tests matching a pattern
fuc test --filter "math::"

# Run tests in a specific file
fuc test --file tests/math.fu
```

### `fuc bench`

```bash
# Run all benchmarks
fuc bench

# Run specific benchmark
fuc bench bench_matrix_multiply

# Run benchmarks for N seconds
fuc bench --duration 10s

# Output results as JSON
fuc bench --format=json
```

### `fuc doc`

```bash
# Generate documentation
fuc doc src/

# Generate and open in browser
fuc doc --open src/

# Generate with private items
fuc doc --private src/

# Generate to specific directory
fuc doc --output docs/ src/
```

### `fuc profile`

```bash
# Profile a program
fuc profile src/main.fu

# Profile with CPU and memory
fuc profile --cpu --memory src/main.fu

# Generate flame graph
fuc profile --flame-graph src/main.fu

# Profile for specific duration
fuc profile --duration 5s src/main.fu
```

### `fuc debug`

```bash
# Run with debugger
fuc debug src/main.fu

# Attach to running process
fuc debug --attach <pid>

# Start debug server (for IDE)
fuc debug --server

# Debug with specific breakpoint
fuc debug --breakpoint src/main.fu:42
```

### `fuc new` and `fuc init`

```bash
# Create new project
fuc new my_project

# Create new project with template
fuc new my_project --template cli
fuc new my_project --template web
fuc new my_project --template lib
fuc new my_project --template wasm

# Initialize in existing directory
cd existing_project
fuc init

# Init with specific name
fuc init --name my_project
```

### Other Commands

```bash
# Clean build artifacts
fuc clean

# Show version
fuc --version

# Show help
fuc --help
fuc build --help

# Show compiler information
fuc info
fuc info --targets
fuc info --features
```

### Compiler Flags Reference

| Flag | Description |
|------|-------------|
| `-o <path>` | Set output file path |
| `--opt-level <0-3>` | Optimization level |
| `--target <triple>` | Target triple |
| `--emit-llvm` | Emit LLVM IR |
| `--emit-bin` | Emit linked executable (default) |
| `--lib` | Compile as library |
| `--parse-only` | Parse only |
| `--sema-only` | Semantic analysis only |
| `--debug` | Include debug info |
| `--no-debug` | Exclude debug info |
| `--vortex` | Enable Vortex borrow checking |
| `--link-lib <name>` | Link external library |
| `--lib-path <path>` | Library search path |
| `--release` | Enable release optimizations |
| `--wasm` | Compile to WebAssembly |
| `--config <file>` | Use custom config file |

### Target Triples

| Triple | Platform |
|--------|----------|
| `x86_64-pc-windows-msvc` | Windows x64 |
| `x86_64-unknown-linux-gnu` | Linux x64 |
| `x86_64-apple-darwin` | macOS x64 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 |
| `aarch64-apple-darwin` | macOS ARM64 |
| `wasm32-unknown-unknown` | WebAssembly |
| `wasm32-wasi` | WebAssembly with WASI |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `FUSION_HOME` | Fusion installation directory |
| `FUSION_TARGET` | Default target triple |
| `FUSION_OPT_LEVEL` | Default optimization level |
| `FUSION_DEBUG` | Enable debug mode |
| `FUSION_LOG` | Log level (error, warn, info, debug, trace) |

---

## VS Code Extension

The official Fusion VS Code extension provides rich IDE support.

### Installation

```
1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Fusion"
4. Install "Fusion Language Support"
```

### Features

| Feature | Description |
|---------|-------------|
| Syntax Highlighting | Full syntax support for `.fu` files |
| Code Completion | Context-aware autocomplete |
| Go to Definition | Jump to function/struct definitions |
| Find References | Find all usages across project |
| Hover Info | Documentation on hover |
| Diagnostics | Real-time error/warning display |
| Code Actions | Quick fixes and refactoring |
| Formatting | Auto-format on save |
| Debugging | Step-through debugger |
| Test Explorer | Run tests from sidebar |
| Inline Values | Show variable values during debug |
| Call Hierarchy | Navigate call chains |
| Type Hierarchy | Navigate type relationships |
| Inlay Hints | Type and parameter hints |
| Snippets | Common code patterns |

### Configuration

```json
// settings.json
{
  "fusion.path": "fuc",
  "fusion.args": ["--vortex"],
  "fusion.formatOnSave": true,
  "fusion.lintOnSave": true,
  "fusion.diagnosticSeverity": {
    "error": "Error",
    "warning": "Warning",
    "info": "Information",
    "hint": "Hint"
  },
  "fusion.inlayHints": {
    "enable": true,
    "typeHints": true,
    "parameterHints": true
  },
  "fusion.debug": {
    "externalConsole": false,
    "showDisassembly": false
  }
}
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Space` | Trigger completion |
| `F12` | Go to definition |
| `Shift+F12` | Find references |
| `Ctrl+Shift+F` | Search project symbols |
| `F5` | Start debugging |
| `F9` | Toggle breakpoint |
| `Ctrl+Shift+F5` | Run tests |
| `Ctrl+K Ctrl+F` | Format selection |
| `Ctrl+.` | Quick fix |

---

## Package Manager (Forge)

Forge is Fusion's package manager for managing dependencies.

### Creating a Package

```bash
# Initialize a new package
forge init my_package

# This creates:
# my_package/
# ├── Fusion.toml
# ├── src/
# │   ├── lib.fu
# │   └── main.fu
# └── tests/
#     └── test_lib.fu
```

### Adding Dependencies

```bash
# Add a dependency
forge add std_crypto

# Add a specific version
forge add std_quantum@1.0.0

# Add from git
forge add https://github.com/user/repo.git

# Add from local path
forge add ../my_local_package

# Add with features
forge add std_ml --features "gpu"

# Add as dev dependency
forge add test_framework --dev

# Add as build dependency
forge add codegen --build
```

### Removing Dependencies

```bash
# Remove a dependency
forge remove std_quantum

# Remove all unused dependencies
forge remove --unused
```

### Updating Dependencies

```bash
# Update all dependencies
forge update

# Update specific dependency
forge update std_crypto

# Update to latest compatible version
forge update --compatible
```

### Publishing Packages

```bash
# Login to Forge registry
forge login

# Publish your package
forge publish

# Publish with specific version
forge publish --version 1.0.0

# Dry run (check without publishing)
forge publish --dry-run

# Publish to private registry
forge publish --registry https://my-registry.example.com
```

### Searching Packages

```bash
# Search for packages
forge search quantum

# Search with filters
forge search --category crypto std

# Show package info
forge info std_ml

# Show package versions
forge versions std_ml
```

### Dependency Management

```toml
# Fusion.toml
[project]
name = "my_project"
version = "0.1.0"

[dependencies]
std_crypto = "1.0"
std_ml = "2.0"
std_quantum = "0.5"

[dev-dependencies]
test_framework = "1.0"

[build-dependencies]
build_tools = "1.0"
```

### Lock Files

```bash
# Generate lock file
forge lock

# Update dependencies
forge update

# Install from lock file
forge install

# Verify lock file integrity
forge verify
```

---

## Code Formatter

Fusion includes a built-in code formatter (`fuc fmt`).

### Basic Formatting

```bash
# Format a single file
fuc fmt src/main.fu

# Format all files in a directory
fuc fmt src/

# Check formatting without modifying
fuc fmt --check src/

# Format with specific style
fuc fmt --style=standard src/main.fu
```

### Formatting Rules

```fusion
// Before formatting (messy code):
fn   main()   ->   int   {
let   x:int=42;let y  :int=   100;
if(x>10){println("big");}else{println("small");}
return 0;}

// After formatting (clean code):
fn main() -> int {
    let x: int = 42;
    let y: int = 100;

    if x > 10 {
        println("big");
    } else {
        println("small");
    }

    return 0;
}
```

### Configuration

```toml
# fusion.toml
[format]
max_width = 100
tab_width = 4
use_tabs = false
chain_width = 80
single_line_if_else = true
```

---

## Test Framework

Fusion includes a built-in test framework.

### Writing Tests

```fusion
// tests/test_math.fu

#[test]
fn test_addition() {
    let result: int = add(2, 3);
    assert_eq(result, 5);
}

#[test]
fn test_subtraction() {
    let result: int = subtract(10, 4);
    assert_eq(result, 6);
}

#[test]
fn test_division_by_zero() {
    let result: Result<int, string> = safe_divide(10, 0);
    assert!(result.is_err());
}

#[test]
fn test_string_operations() {
    let s: string = "Hello, World!";
    assert_eq(s.len(), 13);
    assert!(s.contains("World"));
}

// Async test
#[test::async]
fn test_async_operation() -> Result<void, string> {
    let result = async_fetch("https://api.example.com").await?;
    assert_eq(result.status, 200);
    return Ok(());
}

// Test with setup/teardown
#[test]
fn test_with_fixture() {
    let fixture = setup_test_data();
    defer cleanup_test_data(fixture);

    let result = process(fixture);
    assert_eq(result.count, 42);
}
```

### Running Tests

```bash
# Run all tests
fuc test

# Run specific test
fuc test test_addition

# Run tests with output
fuc test --verbose

# Run tests in parallel
fuc test --parallel

# Generate test coverage
fuc test --coverage

# Run tests matching pattern
fuc test --filter "math::"

# Run tests in specific file
fuc test --file tests/math.fu

# Run tests with timeout
fuc test --timeout 30s

# Run ignored tests
fuc test --ignored
```

### Test Helpers

```fusion
// tests/common.fu

pub fn setup() {
    println("Setting up test environment");
}

pub fn teardown() {
    println("Tearing down test environment");
}

// Use in tests
#[test]
fn test_with_setup() {
    setup();
    defer teardown();

    assert_eq(1 + 1, 2);
}
```

### Test Configuration

```toml
# fusion.toml
[test]
parallel = true
verbose = true
coverage = true
timeout = "30s"
```

---

## Profiler

Fusion includes profiling tools for performance analysis.

### CPU Profiling

```bash
# Profile a program
fuc profile src/main.fu

# Profile with specific options
fuc profile --cpu --memory src/main.fu

# Generate flame graph
fuc profile --flame-graph src/main.fu

# Profile for specific duration
fuc profile --duration 5s src/main.fu
```

### Memory Profiling

```bash
# Track memory allocations
fuc profile --memory src/main.fu

# Detect memory leaks
fuc profile --leak-check src/main.fu

# Track allocation patterns
fuc profile --alloc-pattern src/main.fu
```

### In-Code Profiling

```fusion
use std::profiler;

fn main() -> int {
    profiler::start("main_work");

    let mut sum: int = 0;
    for i in 0..1000000 {
        sum = sum + i;
    }

    profiler::stop("main_work");
    profiler::report();

    return 0;
}
```

### Profiling Output

```
Profiling Report
===============

main_work: 45.2ms (100.0%)
  - Loop iterations: 1,000,000
  - Average per iteration: 0.000045ms
  - Memory allocated: 0 bytes

Total execution time: 45.2ms
Peak memory usage: 1.2MB
```

---

## Linter

Fusion includes a linter for code quality checks.

### Running the Linter

```bash
# Lint a file
fuc lint src/main.fu

# Lint all files
fuc lint src/

# Lint with specific rules
fuc lint --rules=security,performance src/

# Fix auto-fixable issues
fuc lint --fix src/

# Output as JSON (for CI)
fuc lint --format=json src/
```

### Lint Rules

```toml
# .fusionlint.toml
[rules]
# Security rules
unsafe_code = "warn"
unchecked_unwrap = "error"
sql_injection = "error"
hardcoded_secret = "error"

# Performance rules
unnecessary_clone = "warn"
redundant_allocation = "warn"
unbounded_loop = "warn"

# Style rules
naming_convention = "warn"
unused_import = "warn"
dead_code = "warn"
```

### Common Lint Issues

```fusion
// Before linting:
let x: int = 42;
let y: int = x.clone();  // Warning: unnecessary clone for Copy type
let z: &int = &x;

// After linting:
let x: int = 42;
let y: int = x;  // Fixed: Copy type, no clone needed
let z: &int = &x;
```

### Custom Lint Rules

```fusion
// Define custom lint rule
#[lint_rule(
    name = "no_hardcoded_secrets",
    level = "error",
    description = "Detects hardcoded secrets in code"
)]
fn check_no_hardcoded_secrets(ast: &Ast) -> Vec<LintDiagnostic> {
    let mut diagnostics: Vec<LintDiagnostic> = Vec::new();

    for node in ast.nodes() {
        if let Some(string_literal) = node.as_string() {
            if string_literal.contains("password") ||
               string_literal.contains("secret") ||
               string_literal.contains("api_key") {
                diagnostics.push(LintDiagnostic {
                    span: node.span(),
                    message: "Possible hardcoded secret detected".to_string(),
                    level: LintLevel::Error,
                });
            }
        }
    }

    return diagnostics;
}
```

---

## Debugging

### Debug Build

```bash
# Compile with debug info
fuc build --debug src/main.fu

# Run with debugger
fuc debug src/main.fu

# Attach to running process
fuc debug --attach <pid>

# Start debug server
fuc debug --server

# Debug with breakpoint
fuc debug --breakpoint src/main.fu:42
```

### Debug Symbols

```fusion
fn main() -> int {
    let x: int = 42;
    let s: string = "Hello";

    // Set breakpoints
    debugger::breakpoint();

    println("Debugging: x=%d, s=%s", x, s);
    return 0;
}
```

### Logging

```fusion
use std::log;

fn main() -> int {
    log::set_level(log::Level::Debug);

    log::info("Application started");
    log::debug("Processing item: %d", 42);
    log::warn("Low memory");
    log::error("Connection failed");

    return 0;
}
```

---

## Configuration with `fusion config`

Fusion provides a global configuration system.

### View Configuration

```bash
# Show all config
fusion config

# Show specific key
fusion config build.opt_level

# Show with defaults
fusion config --show-defaults
```

### Set Configuration

```bash
# Set a config value
fusion config set build.opt_level 3

# Set for specific scope
fusion config set --global build.opt_level 3
fusion config set --project build.opt_level 2

# Unset a config value
fusion config unset build.opt_level
```

### Configuration File Locations

| File | Scope | Description |
|------|-------|-------------|
| `~/.fusion/config.toml` | Global | User-wide settings |
| `Fusion.toml` | Project | Project-specific settings |
| `.fusion/config.toml` | Local | Local overrides (gitignored) |

### Common Configuration Keys

```toml
# Build settings
build.opt_level = 3
build.lto = true
build.debug = false

# Format settings
format.max_width = 100
format.tab_width = 4

# Lint settings
lint.level = "warning"

# Test settings
test.parallel = true
test.coverage = false

# Registry settings
registry.url = "https://forge.fusion-lang.org"

# Telemetry
telemetry.enabled = true
```

---

## Tips and Best Practices

1. **Use the formatter**: Keep code consistent with `fuc fmt`.
2. **Run linter in CI**: Catch issues early in development.
3. **Write tests first**: Use TDD with the test framework.
4. **Profile before optimizing**: Use the profiler to find bottlenecks.
5. **Use Forge for dependencies**: Don't reinvent the wheel.
6. **Configure VS Code extension**: Get the best IDE experience.
7. **Use `fuc check` before build**: Catch type errors early.
8. **Use `fuc doc`**: Generate documentation for your API.
9. **Use `fusion config`**: Customize tool behavior.
10. **Use git hooks**: Run `fuc fmt --check` and `fuc lint` on pre-commit.

---

## Cross-References

- **Chapter 1**: Getting Started for toolchain installation
- **Chapter 6**: Standard Library for API usage
- **Chapter 13**: Advanced for compiler internals
- **Chapter 15**: Reference for complete tool documentation
- **Chapter 17**: Fusion.toml Configuration for project settings
