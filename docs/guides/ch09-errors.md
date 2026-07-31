> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 9: Error Handling

Fusion distinguishes between *recoverable* and *unrecoverable* errors. Recoverable errors (file not found) should be reported so the caller can decide what to do. Unrecoverable errors (index out of bounds) are bugs and should stop the program.

---

## Unrecoverable Errors with `panic!`

When something goes badly wrong—a bug that shouldn't be recovered from—use `panic!`:

```fusion
fn main() {
    panic!("crash and burn")
}
```text

Output:

```text
thread 'main' panicked at 'crash and burn', src/main.fu:2:5
```text

### When to Panic

Panic when:
- A bug in the code is detected
- An invariant is violated
- Recovery is impossible or not meaningful

```fusion
fn divide(a: int, b: int) -> int {
    if b == 0 {
        panic!("division by zero")
    }
    a / b
}
```text

### Using `panic!` in Development

During development, `panic!` is useful for unfinished code:

```fusion
fn coming_soon() {
    panic!("not implemented yet")
}

// Or use the todo! macro
fn also_coming() {
    todo!("implement this later")
}
```text

---

## Recoverable Errors with `Result`

Most errors aren't severe enough to stop the program. The `Result` enum handles these:

```fusion
enum Result<T, E> {
    Ok(T),   // Success with value of type T
    Err(E),  // Failure with error of type E
}
```text

### Basic Usage

```fusion
use std::fs::File

fn main() {
    let file_result = File::open("hello.txt")

    let file = match file_result {
        Ok(f) => f,
        Err(error) => panic!("Problem opening file: {:?}", error),
    }
}
```text

### Matching on Different Errors

```fusion
use std::fs::File
use std::io::ErrorKind

fn main() {
    let file = match File::open("hello.txt") {
        Ok(f) => f,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Problem creating file: {:?}", e),
            },
            other_error => panic!("Problem opening file: {:?}", other_error),
        },
    }
}
```text

### `unwrap` and `expect`

For quick prototyping, `unwrap` extracts the value or panics:

```fusion
let file = File::open("hello.txt").unwrap()  // Panics if Err
```text

`expect` lets you specify the panic message:

```fusion
let file = File::open("hello.txt")
    .expect("hello.txt should be included with this project")
```text

In production code, prefer explicit error handling.

---

## Propagating Errors

Functions often pass errors to their callers:

```fusion
use std::fs::File
use std::io::{self, Read}

fn read_username_from_file() -> Result<String, io::Error> {
    let file_result = File::open("username.txt")

    let mut file = match file_result {
        Ok(f) => f,
        Err(e) => return Err(e),  // Return the error to caller
    }

    let mut username = String::new()

    match file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}
```text

### The `?` Operator

The `?` operator simplifies error propagation:

```fusion
fn read_username_from_file() -> Result<String, io::Error> {
    let mut file = File::open("username.txt")?  // Returns Err if failed
    let mut username = String::new()
    file.read_to_string(&mut username)?
    Ok(username)
}
```text

`?` does:
1. If `Ok`: Extract the value
2. If `Err`: Return the error from the current function

### Chaining with `?`

```fusion
fn read_username_from_file() -> Result<String, io::Error> {
    let mut username = String::new()
    File::open("username.txt")?.read_to_string(&mut username)?
    Ok(username)
}
```text

Or use the standard library:

```fusion
fn read_username_from_file() -> Result<String, io::Error> {
    std::fs::read_to_string("username.txt")
}
```text

### `?` in `main`

`main` can return `Result`:

```fusion
use std::error::Error

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("hello.txt")?
    Ok(())
}
```text

---

## Custom Error Types

For complex applications, define your own error types:

```fusion

#[derive(Debug)]

enum AppError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError { field: String, message: String },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AppError::ValidationError { field, message } => {
                write!(f, "Validation error on '{}': {}", field, message)
            }
        }
    }
}

// Convert from io::Error
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> AppError {
        AppError::IoError(error)
    }
}
```text

Now you can use `?` with automatic conversion:

```fusion
fn process_file() -> Result<(), AppError> {
    let content = std::fs::read_to_string("data.txt")?  // Converts io::Error to AppError
    // ... validate content
    Ok(())
}
```text

---

## Error Handling Best Practices

### When to `panic!`

✅ Panic when:
- The error indicates a bug (e.g., violated invariant)
- Continuing would be worse than stopping
- In examples and prototype code

❌ Don't panic when:
- The error is expected (e.g., file not found)
- The caller should decide how to handle it
- In library code (return `Result` instead)

### When to Return `Result`

✅ Return `Result` when:
- The error is expected and recoverable
- The caller might want to handle it differently
- Writing library code

### Guidelines

```fusion
// Library function: Return Result
pub fn parse_config(path: &str) -> Result<Config, ConfigError> {
    // ...
}

// Application code: Handle or propagate
fn main() -> Result<(), Box<dyn Error>> {
    let config = parse_config("config.toml")?
    // ...
    Ok(())
}

// Validation with panic for bugs
fn process_data(data: &[int]) {
    assert!(!data.is_empty(), "data cannot be empty")  // Bug if violated
    // ...
}
```text

---

## The `anyhow` and `thiserror` Crates

For real-world applications, these popular crates simplify error handling:

### `anyhow` for Applications

```fusion
use anyhow::{Context, Result}

fn main() -> Result<()> {
    let config = std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?

    println("Config: {}", config)
    Ok(())
}
```text

### `thiserror` for Libraries

```fusion
use thiserror::Error

#[derive(Error, Debug)]

pub enum DataError {
    #[error("file not found: {0}")]
    NotFound(String),

    #[error("invalid data format")]
    InvalidFormat,

    #[error("io error")]
    Io(#[from] std::io::Error),
}
```text

---

## Summary

| Scenario             | Approach                |
| :------------------- | :---------------------- |
| Bug in code          | `panic!`                |
| Expected failure     | `Result<T, E>`          |
| Quick prototype      | `unwrap()` / `expect()` |
| Pass error to caller | `?` operator            |
| Multiple error types | Custom error enum       |

This chapter covered:

- **`panic!`**: For unrecoverable errors (bugs)
- **`Result<T, E>`**: For recoverable errors
- **`match`**: Explicit error handling
- **`unwrap` and `expect`**: Quick extraction (panics on error)
- **`?` operator**: Concise error propagation
- **Custom errors**: Type-safe error handling
- **Best practices**: When to panic vs return `Result`

[Next: Chapter 10 - Generics, Traits, and Lifetimes →](./chapter-10-generics.md)