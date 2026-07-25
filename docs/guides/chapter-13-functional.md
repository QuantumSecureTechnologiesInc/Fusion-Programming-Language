> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 13: Functional Language Features: Iterators and Closures

Fusion’s design is heavily influenced by functional programming. Functional programming allows you to treat functions as values, pass them around, and process data streams without explicit loops.

In this chapter, we will discuss:
- **Closures**: Anonymous functions that can capture their environment.
- **Iterators**: A way of collecting a sequence of items and performing operations on them.

These features allow you to write concise, fast, and expressive code.

---

## 13.1 Closures: Anonymous Functions that Capture Their Environment

Closures are functions that are saved in a variable or passed as arguments to other functions. Unlike functions defined with `fn`, closures can capture values from the scope in which they are defined.

### 13.1.1 Defining a Closure

```fusion
fn main() {
    let add_one = |x: i32| -> i32 { x + 1 }

    let result = add_one(5)
    println!("Result: {}", result)
}
```text

Syntax:
- Pair of pipes `| |` for parameters (instead of `( )`).
- Optional type annotations (Fusion often infers them).
- Braces `{ }` for the body (optional if a single expression).

A more concise version:

```fusion
let add_one = |x| x + 1
```text

### 13.1.2 Capturing the Environment

This is the key difference between closures and functions.

```fusion
fn main() {
    let x = 4

    let equal_to_x = |z| z == x // captures 'x'

    let y = 4

    assert!(equal_to_x(y))
}
```text

If you tried to do this with `fn`, it would fail because `x` is not in the scope of the function.

### 13.1.3 Closure Traits: `Fn`, `FnMut`, and `FnOnce`

Closures capture values in three ways, which map directly to the three ways a function can take a parameter: owning, borrowing mutably, and borrowing immutably.

1. **`FnOnce`**: Consumes the variables it captures (moves them). Can be called only once.
2. **`FnMut`**: Borrows values mutably. Can be called multiple times and can change the environment.
3. **`Fn`**: Borrows values immutably. Can be called multiple times without side effects.

Fusion infers which trait to implement based on how the closure uses the captured values.

**Moving captured values**:
If you want to force the closure to take ownership of the values it uses, you can use the `move` keyword. This is useful when passing a closure to a new thread.

```fusion
let x = vec![1, 2, 3]
let equal_to_x = move |z| z == x
// println!("{:?}", x) // Error! x has been moved into the closure
```text

---

## 13.2 Processing a Series of Items with Iterators

An iterator is responsible for the logic of iterating over each item and determining when the sequence has finished.

### 13.2.1 The `Iterator` Trait

All iterators implement a trait named `Iterator`:

```fusion
pub trait Iterator {
    type Item
    fn next(&mut self) -> Option<Self::Item>
    // ... many default methods
}
```text

The heart of an iterator is the `next` method, which returns `Some(item)` or `None` (when finished).

### 13.2.2 Using an Iterator

```fusion
let v1 = vec![1, 2, 3]
let v1_iter = v1.iter()

for val in v1_iter {
    println("Got: {}", val)
}
```text

### 13.2.3 Iterator Adaptors

**Adaptors** are methods defined on the `Iterator` trait that produce a *new* iterator. They are lazy: they don't do anything until you consume the iterator.

**`map`**: Transforms each item.

```fusion
let v1: Vec<i32> = vec![1, 2, 3]
let v2: Vec<_> = v1.iter().map(|x| x + 1).collect()
// v2 is [2, 3, 4]
```text

**`filter`**: Filters items based on a predicate.

```fusion
let v1 = vec![1, 2, 3, 4]
let v2: Vec<_> = v1.into_iter().filter(|x| x % 2 == 0).collect()
// v2 is [2, 4]
```text

**`zip`**: Zips two iterators into pairs.

```fusion
let a = [1, 2]
let b = [3, 4]
let c: Vec<_> = a.iter().zip(b.iter()).collect()
// c is [(1, 3), (2, 4)]
```text

### 13.2.4 Consuming Adaptors

These methods call `next` and use up the iterator to produce a final value.

**`collect`**: Turns an iterator into a collection (like `Vec`).
**`sum`**: Sums all items.
**`fold`**: Reduces the iterator to a single value using a closure.

```fusion
let v1 = vec![1, 2, 3]
let total: i32 = v1.iter().sum() // 6
```text

---

## 13.3 Improving Our I/O Project

Let's use our new knowledge to improve the `minigrep` project from Chapter 12.

### 13.3.1 Removing `clone` Using an Iterator

In `Config::new`, we cloned the strings. We can instead take an iterator as ownership directly!

```fusion
impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next() // Skip program name

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        }

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file name"),
        }

        // ...
    }
}
```text

### 13.3.2 Making Code Clearer with Iterator Adaptors

Let's rewrite `search`:

**Old**:

```fusion
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new()
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line)
        }
    }
    results
}
```text

**New (Functional)**:

```fusion
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}
```text

To many Fusion developers, this functional style is clearer. It focuses on the *what* (filter by query) rather than the *how* (loop, check, push).

---

## 13.4 Performance: Loops vs. Iterators

You might think loops are faster than iterators because of the abstraction layers.

**Actually, iterators are often faster.**

Fusion's iterators are a **zero-cost abstraction**. They get compiled down to roughly the same code as a lower-level loop (and sometimes better, because the compiler can unroll loops and vectorize operations more easily when it knows the exact bounds, which iterators provide).

Don't fear using iterators for high-performance code!

---

## 13.5 Summary

Closures and iterators are Fusion features inspired by functional programming. They contribute to Fusion’s capability to write clearly expressed, high-level code that compiles to low-level machine code with fast runtime performance.

In the next chapter, we'll look at **Smart Pointers**, which unlock even more capabilities.

---

## 13.6 Exercises

1. **Custom Iterator**: implementing `Iterator` for a custom struct `Counter` that counts from 1 to 5.
2. **Filter/Map**: Use filter and map to find the sum of all squares of even numbers in a vector.
3. **Closures**: Write a `Cacher` struct that holds a closure and a generic result value. It should execute the closure only once and then return the cached value on subsequent calls (memoization).

---

[Next: Chapter 14 - Smart Pointers →](./chapter-14-smart-pointers.md)