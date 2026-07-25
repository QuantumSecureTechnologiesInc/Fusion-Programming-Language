# Chapter 37: Cognitive Load & Team Onboarding

Polyglot systems don't just require knowing multiple languages. They require understanding how those languages interact, where the boundaries are, and what can go wrong when data crosses them. This chapter reduces that cognitive burden.

## The Polyglot Learning Curve

### Why Polyglot Is Harder Than Single-Language

Single-language teams face one learning curve. Polyglot teams face N curves plus the interactions between them:

```
Cognitive Load Model for Polyglot Systems:

1. Language Syntax (N languages)
   └── Each requires dedicated study time

2. Language Idioms (N languages)
   └── "Pythonic" ≠ "Rusty" ≠ "Go-like"

3. Type System Interactions
   └── How Rust's ownership maps to Python's GC
   └── How Go's interfaces map to Java's classes

4. FFI Mechanics
   └── Memory ownership across boundaries
   └── Error propagation across languages

5. Toolchain Management
   └── N build systems, N linters, N formatters

Total cognitive load: O(N²) where N = number of languages
```

### Realistic Timeline for Team Adoption

**Month 1-2: Survival**
- Team can read all languages
- Can make simple changes with guidance
- FFI boundaries are "magic boxes"

**Month 3-4: Competence**
- Team can write in all languages independently
- Understands FFI mechanics at a high level
- Can debug simple cross-language issues

**Month 5-8: Proficiency**
- Team optimizes across boundaries
- Can design new FFI interfaces
- Understands performance implications

**Month 9+: Mastery**
- Team innovates with polyglot patterns
- Creates abstractions that simplify complexity
- Mentors others effectively

**Key insight**: Don't expect production-ready polyglot developers in less than 6 months.

### Common Failure Modes

**1. The Expert Bottleneck**
```
Symptom: One person understands all the languages; others are blocked
Cause: No investment in cross-training
Fix: Pair programming across languages, rotate who touches what
```

**2. The "Just Rewrite It" Trap**
```
Symptom: Team wants to rewrite everything in "the best" language
Cause: Frustration with complexity, not actual performance issues
Fix: Profile first, prove the bottleneck, then optimize
```

**3. The Boundary Explosion**
```
Symptom: Every new feature adds 3 new FFI calls
Cause: No architectural boundaries, ad-hoc interop
Fix: Establish FFI design patterns, batch operations
```

**4. The Documentation Desert**
```
Symptom: "How does this data get from Python to Rust?" is unanswerable
Cause: No documentation at boundaries
Fix: Document every FFI function with examples and diagrams
```

## Code-Switching Cheat Sheet

The same algorithm implemented in each language, showing where they differ and where they're similar.

### Map/Filter/Reduce Comparison

```python
# Python
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# Map: Apply function to each element
squared = list(map(lambda x: x ** 2, numbers))

# Filter: Keep elements matching predicate
evens = list(filter(lambda x: x % 2 == 0, numbers))

# Reduce: Combine elements
from functools import reduce
total = reduce(lambda acc, x: acc + x, numbers, 0)

# Or use list comprehensions (more Pythonic)
squared = [x ** 2 for x in numbers]
evens = [x for x in numbers if x % 2 == 0]
```

```javascript
// JavaScript
const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Map
const squared = numbers.map(x => x ** 2);

// Filter
const evens = numbers.filter(x => x % 2 === 0);

// Reduce
const total = numbers.reduce((acc, x) => acc + x, 0);
```

```rust
// Rust
let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Map
let squared: Vec<i32> = numbers.iter().map(|&x| x.pow(2)).collect();

// Filter
let evens: Vec<&i32> = numbers.iter().filter(|&&x| x % 2 == 0).collect();

// Reduce
let total: i32 = numbers.iter().fold(0, |acc, &x| acc + x);
```

```java
// Java
int[] numbers = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};

// Map
int[] squared = Arrays.stream(numbers).map(x -> x * x).toArray();

// Filter
int[] evens = Arrays.stream(numbers).filter(x -> x % 2 == 0).toArray();

// Reduce
int total = Arrays.stream(numbers).reduce(0, (acc, x) -> acc + x);
```

```go
// Go
numbers := []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10}

// Map (Go doesn't have a built-in map; use loop or library)
squared := make([]int, len(numbers))
for i, x := range numbers {
    squared[i] = x * x
}

// Filter
var evens []int
for _, x := range numbers {
    if x%2 == 0 {
        evens = append(evens, x)
    }
}

// Reduce
total := 0
for _, x := range numbers {
    total += x
}
```

```fusion
// Fusion
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// Map
let squared = numbers.map(|x| x ** 2)

// Filter
let evens = numbers.filter(|x| x % 2 == 0)

// Reduce
let total = numbers.reduce(0, |acc, x| acc + x)
```

### Key Differences Table

| Concept | Python | JavaScript | Rust | Java | Go | Fusion |
|---------|--------|------------|------|------|----|----|
| **Null handling** | `None` | `null/undefined` | `Option<T>` | `null` | `nil` | `Optional<T>` |
| **Error handling** | Exceptions | try/catch | `Result<T,E>` | Exceptions | error return | `Result<T,E>` |
| **Async model** | async/await | async/await | async/await | CompletableFuture | goroutines | async/await |
| **Memory** | GC | GC | Ownership | GC | GC | GC |
| **Null safety** | Runtime | Runtime | Compile-time | Runtime | Runtime | Compile-time |
| **Generics** | Duck typing | N/A | Monomorphized | Type erasure | Type parameters | Type parameters |

## IDE Configuration

### VS Code Polyglot Setup

```json
// .vscode/settings.json
{
    // Language-specific settings
    "python.linting.enabled": true,
    "python.linting.pylintEnabled": true,
    "rust-analyzer.check.command": "clippy",
    
    // Polyglot-specific
    "files.associations": {
        "*.fusion": "fusion",
        "*.ffi": "ffi-spec"
    },
    
    // Formatting
    "[python]": {
        "editor.defaultFormatter": "ms-python.black-formatter",
        "editor.formatOnSave": true
    },
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer",
        "editor.formatOnSave": true
    },
    "[javascript]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },
    
    // Shared settings
    "editor.rulers": [80, 120],
    "files.trimTrailingWhitespace": true,
    "files.insertFinalNewline": true,
    
    // Fusion runtime integration
    "fusion.languageServer": {
        "enabled": true,
        "diagnostics": true,
        "inlayHints": true
    }
}
```

Recommended extensions:
- `ms-python.python` - Python support
- `rust-lang.rust-analyzer` - Rust support
- `esbenp.prettier-vscode` - JavaScript/TypeScript formatting
- `fusion-runtime.fusion-lang` - Fusion language support
- `streetsidesoftware.code-spell-checker` - Spell checking

### IntelliJ Polyglot Setup

IntelliJ Ultimate supports all languages natively:

1. **Python**: Enable Python plugin
2. **Rust**: Install Rust plugin
3. **JavaScript**: Enable JavaScript plugin
4. **Fusion**: Install Fusion plugin (if available)

Configure shared settings:
- **Editor > Code Style**: Set consistent indentation (4 spaces)
- **Plugins**: Enable "Interlaced" for polyglot navigation
- **Version Control**: Configure for polyglot projects

### Language Server Configuration

Fusion Language Server provides IDE features across all languages:

```json
// .fusion/langserver.json
{
    "languages": ["python", "rust", "javascript", "go"],
    "features": {
        "completion": true,
        "hover": true,
        "diagnostics": true,
        "crossLanguageRefactoring": true,
        "ffiDocumentation": true
    },
    "boundaryAnalysis": {
        "enabled": true,
        "showTypeConversions": true,
        "highlightMemoryOwnership": true
    }
}
```

### Autocomplete Across Languages

Fusion IDE integration enables cross-language autocomplete:

```python
# In Python file, typing rust_function( shows:
rust_function(
    data: List[Dict[str, Any]],  # Python type hint
    // Translates to: &[HashMap<String, Value>]  // Rust type hint
) -> Dict[str, Any]
// Translates to: HashMap<String, Value>
```

## Naming Conventions

### Interop Boundary Function Prefixes

Use consistent prefixes to identify FFI functions:

```python
# Python side: x_ prefix for functions that cross boundaries
def x_process_batch(items: List[bytes]) -> List[bytes]:
    """Calls Rust batch processor."""
    return rust_ffi.process_batch(items)

def x_validate_schema(data: bytes) -> bool:
    """Calls Rust schema validator."""
    return rust_ffi.validate_schema(data)

# No prefix for internal Python functions
def prepare_data(raw: dict) -> dict:
    """Pure Python, no FFI."""
    return transform(raw)
```

```rust
// Rust side: #[no_mangle] functions exposed to FFI
#[no_mangle]
pub extern "C" fn x_process_batch(
    input: *const u8,
    len: usize,
    output: *mut *mut u8,
    output_len: *mut usize,
) -> i32 {
    // Implementation
}

// Internal Rust functions use normal naming
fn internal_process(item: &Item) -> Result<Item> {
    // No FFI prefix needed
}
```

### Type Naming Across Languages

Maintain consistency for the same conceptual type:

| Concept | Python | Rust | JavaScript | Java | Go |
|---------|--------|------|------------|------|----|
| **User ID** | `user_id: str` | `user_id: String` | `userId: string` | `String userId` | `UserID string` |
| **Timestamp** | `timestamp: datetime` | `timestamp: DateTime<Utc>` | `timestamp: Date` | `Instant timestamp` | `Timestamp time.Time` |
| **Optional** | `value: Optional[int]` | `value: Option<i32>` | `value?: number` | `Integer value` | `value *int` |
| **Result** | `raise ValueError()` | `Result<T, E>` | `throw new Error()` | `throws Exception` | `error` |

### Error Naming Conventions

```python
# Python: Prefix with Error
class ValidationError(Exception):
    """Raised when input validation fails."""
    pass

class FFIError(Exception):
    """Raised when FFI call fails."""
    pass
```

```rust
// Rust: Use descriptive error types
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FFIError {
    #[error("FFI call failed: {0}")]
    CallFailed(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

### Module/Package Naming

```
project/
├── src/
│   ├── python/
│   │   ├── core/           # Core Python logic
│   │   ├── ffi/            # FFI bridge functions
│   │   └── utils/          # Python utilities
│   ├── rust/
│   │   ├── src/
│   │   │   ├── core.rs     # Core Rust logic
│   │   │   ├── ffi.rs      # FFI entry points
│   │   │   └── utils.rs    # Rust utilities
│   │   └── Cargo.toml
│   └── fusion/
│       ├── pipeline.fusion # Fusion pipeline definitions
│       └── types.fusion    # Shared type definitions
├── tests/
│   ├── python/
│   ├── rust/
│   └── integration/        # Cross-language tests
└── docs/
    ├── api/                # API documentation
    ├── guides/             # Developer guides
    └── boundaries/         # FFI boundary documentation
```

## Documentation Standards

### Every Interop Function Needs Fuzzed Examples

"Thorough examples" means examples that cover edge cases, not just the happy path:

```python
def x_compress_data(data: bytes, level: int = 6) -> bytes:
    """
    Compress data using Rust's lz4 library.
    
    Args:
        data: Raw bytes to compress
        level: Compression level (1-16, default 6)
    
    Returns:
        Compressed bytes
    
    Examples:
        Basic compression:
        >>> x_compress_data(b"hello world")
        b'\\x04\\x22\\x4c\\x18\\x68\\x65\\x6c\\x6c\\x6f\\x20\\x77\\x6f\\x72\\x6c\\x64'
        
        Empty input:
        >>> x_compress_data(b"")
        b''
        
        Maximum compression:
        >>> x_compress_data(b"hello world" * 1000, level=16)
        b'...'  # Much smaller than input
        
        Invalid level (raises ValueError):
        >>> x_compress_data(b"test", level=0)
        ValueError: Compression level must be between 1 and 16
        
        None input (raises TypeError):
        >>> x_compress_data(None)
        TypeError: Data must be bytes, got NoneType
        
        Large data (tests memory handling):
        >>> x_compress_data(b"\\x00" * 10_000_000)
        b'...'  # Should complete without OOM
    """
```

### Data Structure Documentation at Boundaries

Every data structure that crosses a boundary needs documentation:

```python
# boundary-docs/record.json
"""
Record: Core data structure passed between Python and Rust

Schema:
{
    "id": "string (UUID format)",
    "name": "string (non-empty)",
    "value": "number (float64)",
    "tags": "array of strings",
    "metadata": "object (arbitrary key-value pairs)",
    "created_at": "string (ISO 8601 timestamp)"
}

Python example:
    record = {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "sensor-1",
        "value": 42.5,
        "tags": ["temperature", "indoor"],
        "metadata": {"location": "building-A"},
        "created_at": "2024-01-15T10:30:00Z"
    }

Rust equivalent:
    struct Record {
        id: String,      // UUID
        name: String,    // Non-empty
        value: f64,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
        created_at: DateTime<Utc>,
    }

Validation:
    - id: Must be valid UUID v4
    - name: Must be non-empty, max 256 chars
    - value: Must be finite (not NaN or Inf)
    - tags: Max 32 tags, each max 64 chars
    - metadata: Max 64 keys, values max 1024 chars
"""
```

### Cross-Language API Documentation

```markdown
# FFI API Documentation

## x_compress_data

**Purpose**: Compress data using lz4 algorithm
**Direction**: Python → Rust → Python
**Performance**: ~100MB/s compression, ~500MB/s decompression

### Parameters

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| data | bytes | Yes | - | Raw data to compress |
| level | int | No | 6 | Compression level (1-16) |

### Returns

- **Type**: bytes
- **Description**: Compressed data
- **Empty input**: Returns empty bytes

### Errors

| Error | Cause | Recovery |
|-------|-------|----------|
| ValueError | Invalid compression level | Check level is 1-16 |
| FFIError | Rust library not loaded | Check installation |
| MemoryError | Input too large | Process in chunks |

### Usage Examples

```python
# Simple usage
compressed = x_compress_data(b"hello world")

# With options
compressed = x_compress_data(data, level=9)

# Error handling
try:
    compressed = x_compress_data(data)
except ValueError as e:
    print(f"Invalid input: {e}")
```

### Performance Notes

- First call has ~10ms overhead (FFI initialization)
- Subsequent calls: ~100ns per KB
- Memory usage: 2x input size during compression
- Thread-safe: Yes
```

### README Conventions for Polyglot Projects

Every polyglot project README should include:

```markdown
# Project Name

## Languages Used

| Language | Version | Purpose |
|----------|---------|---------|
| Python | 3.11+ | API layer, data processing |
| Rust | 1.70+ | Core computation, performance |
| JavaScript | 18+ | Frontend, WebSocket |
| Fusion | 2.0 | Pipeline orchestration |

## Prerequisites

- Python 3.11+ with pip
- Rust 1.70+ with cargo
- Node.js 18+ with npm
- Fusion Runtime 2.0+

## Quick Start

```bash
# Install all dependencies
make install

# Run tests
make test

# Start development server
make dev
```

## Architecture

[Diagram showing language boundaries]

## Development Guide

### Adding a New FFI Function

1. Define function in `rust/src/ffi.rs`
2. Add Python wrapper in `python/ffi/`
3. Document in `docs/boundaries/`
4. Add tests in `tests/integration/`

### Running Specific Language Tests

```bash
make test-python
make test-rust
make test-integration
```

## Performance

See [performance guide](docs/guides/performance.md) for optimization tips.

## Troubleshooting

See [troubleshooting guide](docs/guides/troubleshooting.md) for common issues.
```

## Summary

- **Plan for 6+ months** to reach production-ready polyglot proficiency
- **Use the code-switching cheat sheet** to understand language differences
- **Configure IDEs** for polyglot support from day one
- **Follow naming conventions** consistently across languages
- **Document every FFI function** with thorough examples
- **Include data structure schemas** at boundaries
- **Write cross-language API docs** that explain the full flow

The investment in onboarding documentation pays dividends: teams that understand the system can debug it, optimize it, and extend it without expert help.

← [Chapter 36: Performance Profiling Across Language Boundaries](ch36-polyglot-profiling.md)
