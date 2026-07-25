> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 11: Writing Automated Tests

Tests are imperative for ensuring that your code behaves as expected. Unlike some languages where testing is an afterthought or requires external frameworks, Fusion has a first-class testing system built right into the language and the compiler.

In this chapter, we’ll discuss:
- How to write unit tests and integration tests.
- How to run tests using the `fusion test` command.
- Controlling test execution and output.

---

## 11.1 How to Write Tests

A test in Fusion is a function that’s annotated with the `#[test]` attribute. Attributes are metadata about pieces of code; we’ve used `#[derive(...)]` before.

### 11.1.1 The Anatomy of a Test Function

When you run `fusion test`, Fusion builds a test runner binary that executes the annotated functions and reports on whether each test function passes or fails.

Let's create a new library project called `adder`:

```bash
fusion new adder --lib
cd adder
```text

Open `src/lib.fu`. You'll see a generated test:

```fusion

#[cfg(test)]

mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2
        assert_eq!(result, 4)
    }
}
```text

Let's break it down:
1. `#[test]`: This attribute indicates this is a test function.
2. `assert_eq!`: A macro that asserts two values are equal. If they are not, it panics, causing the test to fail.

To run it:

```bash
fusion test
```text

### 11.1.2 Checking Results with Assert Macros

The `assert!` macro checks that a boolean condition is true.

```fusion

#[test]

fn larger_can_hold_smaller() {
    let larger = Rectangle { width: 8, height: 7 }
    let smaller = Rectangle { width: 5, height: 1 }

    assert!(larger.can_hold(&smaller))
}
```text

The `assert_eq!` and `assert_ne!` macros compare two arguments for equality or inequality. They print the values if the assertion fails, which is very helpful for debugging.

```fusion
pub fn add_two(a: i32) -> i32 {
    a + 2
}

#[test]

fn it_adds_two() {
    assert_eq!(4, add_two(2))
}
```text

### 11.1.3 Adding Custom Failure Messages

You can add a custom message as a second (or third) argument to the assert macros.

```fusion
assert!(
    result.contains("Carol"),
    "Greeting did not contain name, value was `{}`",
    result
)
```text

### 11.1.4 Checking for Panics with `should_panic`

Sometimes, testing that a function fails (panics) when given bad input is important for your API contracts.

```fusion
pub struct Guess {
    value: i32,
}

impl Guess {
    pub fn new(value: i32) -> Guess {
        if value < 1 || value > 100 {
            panic!("Guess value must be between 1 and 100, got {}.", value)
        }

        Guess { value }
    }
}

#[cfg(test)]

mod tests {
    use super::*

    #[test]
    #[should_panic(expected = "Guess value must be between 1 and 100")]
    fn greater_than_100() {
        Guess::new(200)
    }
}
```text

The `expected` parameter validates that the panic message contains the provided text.

### 11.1.5 Using `Result<T, E>` in Tests

Tests can also return `Result<(), E>`. This allows you to use the `?` operator in tests.

```fusion

#[test]

fn it_works() -> Result<(), String> {
    if 2 + 2 == 4 {
        Ok(())
    } else {
        Err(String::from("two plus two does not equal four"))
    }
}
```text

---

## 11.2 Controlling How Tests Are Run

`fusion test` compiles your code in test mode and runs the resulting test binary. You can control this behavior with command line flags.

### 11.2.1 Running Tests in Parallel or Consecutively

By default, tests run in parallel using threads. This is fast, but if your tests share state (like writing to the same file 'test.txt'), they might interfere with each other.

To run tests one at a time:

```bash
fusion test -- --test-threads=1
```text

### 11.2.2 Showing Function Output

By default, if a test passes, Fusion captures anything printed to stdout (like `println!`) and doesn't show it. If you want to see output even for passing tests:

```bash
fusion test -- --show-output
```text

### 11.2.3 Running a Subset of Tests

You can pass the name (or part of the name) of a test to run only that test.

```bash
fusion test it_works
```text

This will run any test function that has "it_works" in its name.

### 11.2.4 Ignoring Tests

Some tests are expensive (e.g., they take minutes to run). You can mark them as ignored:

```fusion

#[test]


#[ignore]

fn expensive_test() {
    // code that takes an hour to run
}
```text

It won't run by default. To run it explicitly:

```bash
fusion test -- --ignored
```text

---

## 11.3 Test Organization

The Fusion community thinks about tests in two main categories: unit tests and integration tests.

### 11.3.1 Unit Tests

**purpose**: Test each unit of code in isolation (often private functions).
**location**: In the same file as the source code, inside a `tests` module annotated with `#[cfg(test)]`.

The `#[cfg(test)]` attribute tells Fusion to compile and run this code *only* when you run `fusion test`, not when you run `fusion build`. This saves space in the resulting binary.

```fusion
// src/lib.fu

fn internal_adder(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]

mod tests {
    use super::* // Import the parent module's items

    #[test]
    fn internal() {
        assert_eq!(4, internal_adder(2, 2))
    }
}
```text

Since the tests are in the same file, they can test **private** functions.

### 11.3.2 Integration Tests

**purpose**: Test whether many parts of your library work together correctly using its public API.
**location**: In a separate `tests` directory.

Fusion treats each file in the `tests` directory as a separate crate.

Project structure:

```text
adder
├── src
│   └── lib.fu
└── tests
    └── integration_test.fu
```text

Inside `tests/integration_test.fu`:

```fusion
use adder

#[test]

fn it_adds_two() {
    assert_eq!(4, adder::add_two(2))
}
```text

We don't need `#[cfg(test)]` here; the `tests` folder is special. Also, integration tests can **only** call public functions.

---

## 11.4 Summary

Fusion’s testing features provide a way to specify how code should function to ensure it continues to work as you extend it.
- **Unit tests** live alongside code and check internals.
- **Integration tests** live in `tests/` and check the public API.
- `fusion test` is your primary tool for validation.

As you write larger projects, automated testing becomes not just helpful, but necessary. In the next chapter, we'll put everything we've learned so far—structs, error handling, generics, and testing—into practice by building a complete command-line tool.

---

## 11.5 Exercises

1. **Testing Math**: In your `fusion_math` library from Chapter 7, add unit tests for your `add` and `subtract` functions.
2. **Bug Hunt**: Intentionally introduce a bug in your code (e.g., change `+` to `-`). Run `fusion test` to see the failure. Fix it.
3. **TDD**: Write a test for a new function `multiply(a, b)` *before* you implement the function (Test Driven Development). Watch it fail to compile, then fail the test, then implement it to pass.

---

[Next: Chapter 12 - An I/O Project: Building a Command Line Program →](./chapter-12-io-project.md)