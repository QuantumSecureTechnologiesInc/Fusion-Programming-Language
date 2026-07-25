> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 9: Error Handling

Fusion’s commitment to reliability extends to error handling. Errors are a fact of life in software, so Fusion provides a number of features for handling situations in which something goes wrong.

In many languages, error handling is difficult or obscure. Fusion groups errors into two major categories: **recoverable** and **unrecoverable** errors.
- **Recoverable errors**: Usually temporary or expected (e.g., file not found). Handled with `Result<T, E>`.
- **Unrecoverable errors**: Symptoms of bugs (e.g., accessing an array past its end). Handled with `panic!`.

Fusion doesn't have exceptions. Instead, it has the type `Result<T, E>` for recoverable errors and the `panic!` macro that stops execution when the program encounters an unrecoverable error.

---

## 9.1 Unrecoverable Errors with `panic!`

Sometimes, bad things happen in your code, and there’s nothing you can do about it. In these cases, Fusion has the `panic!` macro.

When the `panic!` macro executes, your program will print a failure message, unwind and clean up the stack, and then quit.

```fusion
fn main() {
    panic!("crash and burn")
}
```text

Running this:

```text
thread 'main' panicked at 'crash and burn', src/main.fu:2:5
```text

### 9.1.1 Using a panic! Backtrace

When a `panic!` call comes from a library deep in your code, you need to know *where* it originated. You can set the `FUSION_BACKTRACE` environment variable to get a backtrace of strictly what led to the error.

```bash
FUSION_BACKTRACE=1 fusion run
```text

This will print the stack trace, showing the sequence of function calls that led to the panic.

---

## 9.2 Recoverable Errors with `Result`

Most errors aren’t serious enough to require the program to stop entirely. Often, when a function fails, it’s for a reason that you can easily interpret and respond to.

Recall the `Result` enum from Chapter 2:

```fusion
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```text

- `T` is the type of the value that will be returned in a success case.
- `E` is the type of the error that will be returned in a failure case.

### 9.2.1 Matching on Result

Let's look at opening a file:

```fusion
use std::fs::File

fn main() {
    let f = File::open("hello.txt")

    let f = match f {
        Ok(file) => file,
        Err(error) => {
            println!("Problem opening the file: {:?}", error)
            panic!("Could not open file")
        },
    }
}
```text

### 9.2.2 Matching on Different Errors

We can take different actions depending on the error type:

```fusion
use std::fs::File
use std::io::ErrorKind

fn main() {
    let f = File::open("hello.txt")

    let f = match f {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Problem creating the file: {:?}", e),
            },
            other_error => panic!("Problem opening the file: {:?}", other_error),
        },
    }
}
```text

### 9.2.3 Shortcuts for Panic on Error: `unwrap` and `expect`

Using `match` can be verbose. The `Result<T, E>` type has many helper methods.

**`unwrap()`**:
If the Result value is the `Ok` variant, `unwrap` will return the value inside `Ok`. If the Result is the `Err` variant, `unwrap` will call `panic!` for us.

```fusion
let f = File::open("hello.txt").unwrap()
```text

**`expect(msg)`**:
Similar to `unwrap`, but lets us choose the panic error message. This helps significantly with debugging.

```fusion
let f = File::open("hello.txt").expect("Failed to open hello.txt")
```text

### 9.2.4 Propagating Errors

When you’re writing a function whose implementation calls something that might fail, instead of handling the error within this function, you can return the error to the calling code so that it can decide what to do. This is known as **propagating** the error.

```fusion
use std::fs::File
use std::io::{self, Read}

fn read_username_from_file() -> Result<String, io::Error> {
    let f = File::open("hello.txt")

    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e),
    }

    let mut s = String::new()

    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}
```text

### 9.2.5 The `?` Operator

This pattern of propagating errors is so common that Fusion provides the `?` operator (try operator) to make this much more concise.

```fusion
use std::fs::File
use std::io
use std::io::Read

fn read_username_from_file() -> Result<String, io::Error> {
    let mut f = File::open("hello.txt")?
    let mut s = String::new()
    f.read_to_string(&mut s)?
    Ok(s)
}
```text

The `?` placed after a `Result` value works like this:
- If the value is `Ok(x)`, the expression evaluates to `x` and the program continues.
- If the value is `Err(e)`, the whole function returns `Err(e)` immediately.

We can chain these calls:

```fusion
fn read_username_from_file() -> Result<String, io::Error> {
    let mut s = String::new()
    File::open("hello.txt")?.read_to_string(&mut s)?
    Ok(s)
}
```text

**Note**: The `?` operator can only be used in functions that return `Result` (or `Option`). You cannot use it in `main` (unless `main` returns `Result`).

```fusion
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("hello.txt")?
    Ok(())
}
```text

---

## 9.3 To `panic!` or Not to `panic!`

How do you decide when you should `panic!` and when you should return `Result`?

### 9.3.1 Guidelines

1. **Examples and Prototypes**: It is fine to use `unwrap` and `expect`.
2. **Tests**: Tests should panic if they fail (`unwrap` is fine).
3. **Libraries**: Libraries should generally **never** panic. Always return `Result` so the user can decide how to handle the failure.
4. **Contract Violations**: If a function receives input that violates its requirements (e.g., index out of bounds), panic is acceptable (this indicates a bug in the caller).

### 9.3.2 Defining Custom Error Types

It's common to define custom error types for your crate.

```fusion

#[derive(Debug)]

pub enum MyError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Custom(String),
}

// Implement Display and Error traits (Chapter 10)
```text

---

## 9.4 Summary

Fusion’s error handling features are designed to help you write more robust code.
- `panic!` signals program bugs or unrecoverable states.
- `Result` is for expected failures.
- Utilizing `?` makes error propagation clean and readable.

In the next chapter, we’ll move away from errors and into the world of generics, traits, and lifetimes—the tools that give Fusion its powerful abstraction capabilities.

---

## 9.5 Exercises

1. **Improved Guessing Game**: Update the guessing game from Chapter 2. Instead of crashing on non-number input, prompt the user specifically with "Please enter a number" without clearing the screen, using `match`.
2. **File Reader**: Write a function that takes a filename and prints its content. Use the `?` operator. If the file doesn't exist, print a friendly "File not found" message instead of a stack trace.
3. **Validation**: Create a function `div(a: i32, b: i32) -> Result<i32, String>`. It should return `Err("division by zero")` if `b` is 0, otherwise `Ok(a / b)`.

---

[Next: Chapter 10 - Generic Types, Traits, and Lifetimes →](./chapter-10-generics.md)