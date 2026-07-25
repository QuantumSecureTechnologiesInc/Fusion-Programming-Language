# Fusion v2.0 Vortex

A modern, general-purpose, polyglot systems programming language with post-quantum cryptography, quantum computing, blockchain, and 16 advanced PLT features.

## Quick Install

### Windows
```powershell
# PowerShell (Run as Administrator)
.\installers\windows\install.ps1
```

### Linux
```bash
bash installers/linux/install.sh
```

### macOS
```bash
bash installers/macos/install.sh
```

### Native Fusion Installer
```bash
# Requires a working Fusion compiler
fusion run installers/windows/install.fu   # Windows
fusion run installers/linux/install.fu     # Linux
fusion run installers/macos/install.fu     # macOS
```

## Quick Start

```bash
# Create a new project
fusion init my_project
cd my_project

# Build and run
fusion build
fusion run

# Run tests
fusion test
```

## Directory Structure

```
Fusion v2.0 Vortex/
├── src/                    # Main compiler source code
├── stdlib/                 # Standard library implementation
├── crates/                 # Rust crate dependencies
├── runtime/                # Runtime system
├── tools/                  # Development tools
├── examples/               # Example programs and tests
│   └── tests/              # Test .fu files
├── grammar/                # Language grammar definitions
├── docs/                   # Documentation
├── Source Files/           # Source file archives
├── build/                  # Build artifacts
│   ├── artifacts/          # Compiled executables
│   └── intermediates/      # Intermediate build files
├── scripts/                # Build and test scripts
├── Cargo.toml              # Rust package manifest
├── Cargo.lock              # Dependency lock file
├── Fusion.toml             # Fusion project configuration
├── README.md               # This file
├── ChangeLog.md            # Version history
├── LICENSE                 # License information
├── BUILD_POLICY.md         # Build policy documentation
└── SIMPLIFIED_PARTS.md     # Simplified parts documentation
```

## Build Instructions

### Prerequisites

- Rust toolchain (rustup)
- CMake (optional, for C integration)
- Git

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build with CMake (if applicable)
mkdir build && cd build
cmake ..
cmake --build .
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --package fusion-test

# Run integration tests
./scripts/run_all_tests.ps1
```

## Development

### Project Structure

- **src/**: Core compiler implementation including lexer, parser, type checker, and code generator
- **stdlib/**: Standard library modules (collections, I/O, etc.)
- **runtime/**: Runtime support for garbage collection and memory management
- **tools/**: Development tools like the formatter and LSP server

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests to ensure nothing is broken
5. Submit a pull request

## License

See [LICENSE](LICENSE) for details.
