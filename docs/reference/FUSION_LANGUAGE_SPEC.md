# Fusion Language Specification v2.0 — Vortex Edition

> **Status**: Draft v2.0  
> **Last Updated**: 2026-07-24  
> **Author**: Fusion Language Team

This document is the definitive reference for the Fusion programming language as implemented in the Vortex compiler. It covers lexical structure, type system, expressions, statements, declarations, memory safety, concurrency, quantum computing, post-quantum cryptography, the standard library, and compiler flags.

---

## Table of Contents

1. [Lexical Structure](#1-lexical-structure)
2. [Type System](#2-type-system)
3. [Expressions](#3-expressions)
4. [Statements](#4-statements)
5. [Declarations](#5-declarations)
6. [Memory Safety](#6-memory-safety)
7. [Concurrency](#7-concurrency)
8. [Quantum Computing](#8-quantum-computing)
9. [Post-Quantum Cryptography](#9-post-quantum-cryptography)
10. [Standard Library](#10-standard-library)
11. [Compiler Flags](#11-compiler-flags)

---

## 1. Lexical Structure

### 1.1 Source Files

Fusion source files use the `.fu` extension. The compiler processes files as UTF-8 encoded text. Each file implicitly declares a module with the name of the file (without extension). A file may contain a trailing newline; blank lines are permitted anywhere.

### 1.2 Keywords

Fusion has 44 keywords, organized into categories:

#### Core Keywords

| Keyword     | Category   | Description                                      |
|-------------|------------|--------------------------------------------------|
| `fn`        | Function   | Declares a function                              |
| `let`       | Binding    | Declares an immutable binding                    |
| `mut`       | Binding    | Marks a binding as mutable                       |
| `return`    | Control    | Returns a value from a function                  |
| `if`        | Control    | Conditional branching                            |
| `else`      | Control    | Alternative branch                               |
| `while`     | Control    | Loop with precondition                           |
| `for`       | Control    | Iterator-based loop                              |
| `in`        | Control    | Range or collection iteration                    |
| `break`     | Control    | Exits a loop                                     |
| `continue`  | Control    | Skips to next loop iteration                     |
| `match`     | Control    | Pattern matching                                 |
| `fn`        | Function   | Function literal                                 |
| `struct`    | Type       | Declares a struct                                |
| `enum`      | Type       | Declares an enum                                 |
| `trait`     | Type       | Declares a trait                                 |
| `impl`      | Type       | Implements a trait or inherent methods           |
| `type`      | Type       | Type alias                                       |
| `self`      | Type       | Refers to the current instance                   |
| `Self`      | Type       | Refers to the implementing type                  |
| `pub`       | Visibility | Public visibility                                |
| `mod`       | Module     | Declares a module                                |
| `use`       | Module     | Imports items                                    |
| `import`    | Module     | Imports external crates or modules               |
| `as`        | Type       | Type ascription / renaming                       |
| `const`     | Binding    | Compile-time constant                            |
| `static`    | Binding    | Static variable with fixed address               |
| `extern`    | FFI        | Foreign function interface                       |
| `unsafe`    | Safety     | Unsafe code block                                |
| `async`     | Async      | Asynchronous function                            |
| `await`     | Async      | Awaits an async future                           |
| `yield`     | Control    | Yields control from a fiber                      |
| `loop`      | Control    | Infinite loop                                   |
| `ref`       | Binding    | Reference binding                               |
| `move`      | Ownership  | Explicitly moves a value                         |
| `where`     | Generics   | Generic type constraints                         |
| `super`     | Module     | Refers to the parent module                      |
| `crate`     | Module     | Refers to the current crate root                 |
| `self`      | Module     | Refers to the current module                     |

#### Quantum Keywords

| Keyword            | Category   | Description                                  |
|--------------------|------------|----------------------------------------------|
| `qubit`            | Quantum    | Declares a qubit register                    |
| `qcircuit`         | Quantum    | Declares a quantum circuit                   |
| `gate`             | Quantum    | Declares a quantum gate operation            |
| `measure`          | Quantum    | Measures a qubit                             |
| `entangle`         | Quantum    | Creates entanglement between qubits          |
| `qfn`              | Quantum    | Declares a quantum function                  |

#### Post-Quantum Cryptography Keywords

| Keyword            | Category   | Description                                  |
|--------------------|------------|----------------------------------------------|
| `pqc`              | Crypto     | Post-quantum cryptographic context           |
| `hybrid`           | Crypto     | Hybrid classical/post-quantum mode           |

#### Additional Keywords

| Keyword      | Category   | Description                                    |
|--------------|------------|------------------------------------------------|
| `true`       | Literal    | Boolean true                                   |
| `false`      | Literal    | Boolean false                                  |
| `null`       | Literal    | Null reference                                 |
| `undefined`  | Literal    | Undefined value                                |

### 1.3 Identifiers

Identifiers start with a letter (`a-z`, `A-Z`) or underscore (`_`), followed by any number of letters, digits, or underscores. Identifiers are case-sensitive.

```
identifier ::= ( letter | '_' ) ( letter | digit | '_' )*
letter    ::= 'a'..'z' | 'A'..'Z' | '_'
digit     ::= '0'..'9'
```

**Examples:**
```fu
let x = 42;
let _private = "hidden";
let camelCase = true;
let SCREAMING_SNAKE = 100;
let α = 3.14;          // Unicode letters are permitted
let 不能以数字开头 = false; // Unicode letters are permitted
```

A single underscore `_` is a special identifier that acts as a wildcard (unused variable, catch-all pattern).

### 1.4 Literals

#### Integer Literals

Integer literals are `i64` by default. Suffixes select the type:

```fu
let decimal    = 42;          // i64
let hex        = 0xFF;        // i64
let octal      = 0o77;        // i64
let binary     = 0b1010_0101; // i64
let thousand   = 1_000_000;   // underscores for readability

let byte: u8   = 255_u8;
let long: i64  = 9_007_199_254_740_991_i64;
let unsigned   = 42u32;
```

**Type suffixes:** `_u8`, `_u16`, `_u32`, `_u64`, `_i8`, `_i16`, `_i32`, `_i64`, `_isize`, `_usize`

#### Float Literals

Float literals are `f64` by default:

```fu
let pi      = 3.14159;      // f64
let precise = 2.71828_f32;  // f32
let sci     = 1.5e10;       // scientific notation
let hex_f   = 0x1.0p4;      // hexadecimal float (16.0)
let under   = 0.5_f64;
```

#### String Literals

Strings are UTF-8 encoded and immutable. Escape sequences:

```fu
let s1 = "hello, world";
let s2 = "line1\nline2";         // newline
let s3 = "tab\there";            // tab
let s4 = "quote: \"hello\"";     // escaped quote
let s5 = "backslash: \\";        // backslash
let s6 = "null: \0";             // null byte
let s7 = "unicode: \u{1F600}";   // unicode escape

// Multi-line strings (raw)
let raw = r#"This is a "raw" string
with backslashes \n preserved."#;

// Byte string
let bytes: [u8; 5] = b"hello";
```

#### Character Literals

Characters are Unicode scalar values enclosed in single quotes:

```fu
let c: char = 'a';
let nl: char = '\n';
let unicode: char = '\u{03B1}';  // Greek alpha
let escaped: char = '\'';        // escaped single quote
```

#### Boolean Literals

```fu
let t = true;
let f = false;
```

#### Tuple Literals

```fu
let pair = (1, "hello");
let triple = (1, 2.0, true);
let unit = ();  // zero-element tuple
```

#### Array Literals

```fu
let arr = [1, 2, 3, 4, 5];
let zeros = [0; 10];  // 10 zeros
let mixed = [1, 2, 3] as [i64; 3];
```

### 1.5 Operators and Precedence

Operators are listed from highest to lowest precedence:

| Precedence | Operator | Associativity | Description                      |
|-----------|----------|---------------|----------------------------------|
| 1 (highest) | `()`  | Left          | Function call                    |
| 1         | `[]`     | Left          | Indexing / subscript             |
| 1         | `.`      | Left          | Field/method access              |
| 1         | `->`     | Left          | Return type annotation           |
| 2         | `!`      | Right         | Logical NOT / bitwise NOT        |
| 2         | `-`      | Right         | Unary negation                   |
| 2         | `*`      | Right         | Dereference                      |
| 2         | `&`      | Right         | Address-of / borrow              |
| 2         | `@`      | Right         | Quantum measurement              |
| 3         | `**`     | Right         | Exponentiation                   |
| 4         | `*`      | Left          | Multiplication                   |
| 4         | `/`      | Left          | Division                         |
| 4         | `%`      | Left          | Modulo                           |
| 5         | `+`      | Left          | Addition                         |
| 5         | `-`      | Left          | Subtraction                      |
| 6         | `<<`     | Left          | Left shift                       |
| 6         | `>>`     | Left          | Right shift                      |
| 7         | `&`      | Left          | Bitwise AND                      |
| 8         | `^`      | Left          | Bitwise XOR                      |
| 9         | `\|`     | Left          | Bitwise OR                       |
| 10        | `==`     | None          | Equality                         |
| 10        | `!=`     | None          | Inequality                       |
| 10        | `<`      | None          | Less than                        |
| 10        | `>`      | None          | Greater than                     |
| 10        | `<=`     | None          | Less or equal                    |
| 10        | `>=`     | None          | Greater or equal                 |
| 10        | `<:`     | None          | Subtype / trait bound            |
| 11        | `&&`     | Left          | Logical AND                      |
| 12        | `\|\|`   | Left          | Logical OR                       |
| 13        | `..`     | None          | Range (inclusive)                |
| 13        | `..=`    | None          | Range (inclusive, explicit)      |
| 13        | `...`    | None          | Range (exclusive)                |
| 14        | `=`      | Right         | Assignment                       |
| 14        | `+=`     | Right         | Add-assign                       |
| 14        | `-=`     | Right         | Subtract-assign                  |
| 14        | `*=`     | Right         | Multiply-assign                  |
| 14        | `/=`     | Right         | Divide-assign                    |
| 14        | `%=`     | Right         | Modulo-assign                    |
| 14        | `&=`     | Right         | AND-assign                       |
| 14        | `\|=`    | Right         | OR-assign                        |
| 14        | `^=`     | Right         | XOR-assign                       |
| 14        | `<<=`    | Right         | Left-shift-assign                |
| 14        | `>>=`    | Right         | Right-shift-assign               |
| 14        | `**=`    | Right         | Exponent-assign                  |
| 15        | `=>`     | Left          | Closure body / match arm         |
| 16 (lowest) | `,`   | Left          | Separator                        |

**Operator examples:**

```fu
// Arithmetic
let a = 2 + 3 * 4;        // 14 (mul binds tighter)
let b = (2 + 3) * 4;      // 20 (parens override)
let c = 2 ** 10;           // 1024 (exponentiation)
let d = -x;                // unary negation

// Bitwise
let flags = 0b1100 | 0b1010;   // 0b1110
let mask  = flags & 0xFF;      // lower byte
let shifted = 1 << 8;          // 256

// Comparison
let eq = x == y;
let ne = x != y;
let lt = x < y;

// Logical
let result = a && b || !c;

// Assignment
x += 1;
y *= 2;

// Range
for i in 0..10 { }
for i in 0..=10 { }
```

### 1.6 Comments

```fu
// Line comment — from // to end of line

/* Block comment
   can span
   multiple lines */

/* Block comments /* can be nested */ in Fusion */

// Doc comments appear on the next item
/// This documents the following item.
/** Multi-line
    doc comment. */
```

**Doc comments** (`///` and `/** */`) are captured by the compiler and stored in the crate metadata. They support Markdown formatting and special tags:

```fu
/// Computes the factorial of `n`.
///
/// # Arguments
/// * `n` - A non-negative integer
///
/// # Returns
/// The factorial of `n` as `u64`.
///
/// # Panics
/// Panics if `n > 20` (overflow).
///
/// # Examples
/// ```
/// let f = factorial(5);
/// assert!(f == 120);
/// ```
fn factorial(n: u64) -> u64 { ... }
```

### 1.7 Attributes

Attributes provide metadata to the compiler and runtime. They appear in square brackets `[]` before declarations:

```fu
// Simple attribute
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

// Attribute with arguments
#[cfg(target_os = "linux")]
fn linux_only() { }

// Intent attribute — describes program intent to the compiler
#[intent(security = "encryption")]
fn encrypt_data(data: &[u8], key: &[u8]) -> Vec<u8> { }

// Attribute with nested configuration
#[cfg(feature = "quantum")]
#[quantum(num_qubits = 8)]
fn quantum_algorithm() { }

// Multiple attributes
#[test]
#[should_panic(expected = "overflow")]
fn test_overflow() {
    let _ = u8::MAX + 1;
}

// Inner attributes (inside modules)
#![allow(dead_code)]
#![cfg_attr(test, warn(unused))]
```

**Built-in attributes:**

| Attribute          | Description                                    |
|--------------------|------------------------------------------------|
| `#[derive(...)]`  | Auto-derive trait implementations              |
| `#[cfg(...)]`      | Conditional compilation                        |
| `#[test]`          | Marks a test function                          |
| `#[bench]`         | Marks a benchmark function                     |
| `#[should_panic]`  | Test expects a panic                           |
| `#[ignore]`        | Skip this test                                 |
| `#[intent(...)]`   | Semantic intent annotation for optimization    |
| `#[unsafe]`        | Marks a block as intentionally unsafe          |
| `#[inline]`        | Function inlining hint                         |
| `#[cold]`          | Marks unlikely code path                       |
| `#[must_use]`      | Return value must be used                      |
| `#[allow(...)]`    | Suppress a specific lint                       |
| `#[warn(...)]`     | Set a lint to warning level                    |
| `#[deny(...)]`     | Set a lint to error level                      |
| `#[quantum(...)]`  | Quantum-specific annotations                   |

---

## 2. Type System

Fusion uses a structural type system with nominal typing for structs, enums, and traits. All types are resolved at compile time.

### 2.1 Primitive Types

#### Boolean

```fu
let t: bool = true;
let f: bool = false;
```

`bool` is 1 byte in memory. It does not support arithmetic operations.

#### Integer Types

| Type   | Size   | Range                              |
|--------|--------|-------------------------------------|
| `u8`   | 1 byte | 0 to 255                           |
| `u16`  | 2 bytes| 0 to 65,535                        |
| `u32`  | 4 bytes| 0 to 4,294,967,295                 |
| `u64`  | 8 bytes| 0 to 18,446,744,073,709,551,615   |
| `u128` | 16 bytes| 0 to 2^128 - 1                   |
| `i8`   | 1 byte | -128 to 127                        |
| `i16`  | 2 bytes| -32,768 to 32,767                 |
| `i32`  | 4 bytes| -2^31 to 2^31 - 1                |
| `i64`  | 8 bytes| -2^63 to 2^63 - 1                |
| `i128` | 16 bytes| -2^127 to 2^127 - 1              |
| `usize`| pointer-sized| 0 to 2^64 - 1 (on 64-bit) |
| `isize`| pointer-sized| -2^63 to 2^63 - 1 (on 64-bit) |

```fu
let a: u32 = 42;
let b: i64 = -100;
let c: usize = std::mem::size_of::<i32>();  // 4

// Arithmetic with type inference
let x = 10u8;          // u8
let y = 20u16;         // u16
// let z = x + y;      // ERROR: mismatched types
let z = x as u16 + y;  // OK: explicit cast
```

**Checked arithmetic:** Fusion wraps integer overflow by default in debug mode and wraps in release mode. Use `checked_*`, `saturating_*`, `wrapping_*`, `overflowing_*` for explicit behavior:

```fu
let a: u8 = 255;
let b = a.checked_add(1);    // None
let c = a.saturating_add(1); // 255
let d = a.wrapping_add(1);   // 0
let (e, overflow) = a.overflowing_add(1); // (0, true)
```

#### Float Types

| Type   | Size    | Precision |
|--------|---------|-----------|
| `f32`  | 4 bytes | ~7 decimal digits |
| `f64`  | 8 bytes | ~15 decimal digits |

```fu
let pi: f64 = 3.141592653589793;
let e: f32 = 2.71828;

// Special values
let inf = f64::INFINITY;
let nan = f64::NAN;
let neg_inf = f64::NEG_INFINITY;

// NaN comparisons (all false)
let result = f64::NAN == f64::NAN; // false
let check = f64::NAN.is_nan();     // true
```

#### Character

```fu
let c: char = 'A';
let emoji: char = '😀';
let alpha: char = 'α';

// char is always a Unicode scalar value (4 bytes)
let size = std::mem::size_of::<char>(); // 4
```

#### String

Strings are heap-allocated, UTF-8 encoded, and immutable by default. They are not `Copy`.

```fu
let s1: string = "hello, world";
let s2: string = String::from("owned string");
let s3: string = format!("{} + {}", "foo", "bar");

// String slicing (returns a string slice `&str`)
let slice: &str = &s2[0..5];  // "hello"

// String methods
let upper = s1.to_uppercase();
let len = s1.len();           // byte length
let char_count = s1.chars().count();
let contains = s1.contains("world");
```

#### Void

`void` is the type of expressions that produce no value:

```fu
fn print_line(msg: string) -> void {
    println!("{}", msg);
}

// `void` is also written as `()` (unit type)
fn do_nothing() { }  // implicitly returns ()
```

### 2.2 Compound Types

#### Structs

Structs are nominal types with named fields:

```fu
struct Point {
    x: f64,
    y: f64,
}

struct Person {
    name: string,
    age: u32,
    email: string,
}

// Tuple structs
struct Color(u8, u8, u8);
struct Meters(f64);

// Unit structs
struct Marker;

// Generic structs
struct Vec<T> {
    data: *mut T,
    len: usize,
    cap: usize,
}

struct Pair<A, B> {
    first: A,
    second: B,
}

// Field shorthand initialization
let x = 10;
let y = 20;
let p = Point { x, y };  // shorthand for { x: x, y: y }

// Struct update syntax
let p2 = Point { x: 5.0, ..p };  // copy y from p
```

**Struct layout:** By default, fields are laid out in declaration order with padding for alignment. Use `#[repr(C)]` for C-compatible layout or `#[repr(packed)]` to eliminate padding:

```fu
#[repr(C)]
struct CCompatible {
    a: u8,
    b: u32,  // 3 bytes padding before this
    c: u8,
}

#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,  // no padding
    c: u8,
}
```

#### Enums

Enums are algebraic data types with variants:

```fu
enum Direction {
    North,
    South,
    East,
    West,
}

enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Enums with data
enum Shape {
    Circle(f64),                     // radius
    Rectangle(f64, f64),             // width, height
    Triangle { a: f64, b: f64, c: f64 },  // named fields
}

// Enums with methods
impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => 3.14159 * r * r,
            Shape::Rectangle(w, h) => w * h,
            Shape::Triangle { a, b, c } => {
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
        }
    }
}

// Discriminant values
enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    ServerError = 500,
}
```

#### Tuples

Tuples are fixed-length heterogeneous collections:

```fu
let t: (i32, f64, string) = (1, 2.0, "hello");

// Destructuring
let (x, y, z) = t;

// Indexing
let first = t.0;   // 1
let second = t.1;  // 2.0

// Unit type
let unit: () = ();

// Nested tuples
let nested: ((i32, i32), (f64, f64)) = ((1, 2), (3.0, 4.0));
```

#### Arrays

Arrays are fixed-length, stack-allocated sequences:

```fu
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let zeros: [f64; 10] = [0.0; 10];

// Length is a compile-time constant
let len = arr.len();  // 5

// Indexing (bounds-checked)
let first = arr[0];
// let bad = arr[10];  // runtime panic

// Slicing
let slice = &arr[1..4];  // [2, 3, 4]

// Array methods
let sum: i32 = arr.iter().sum();
let max = arr.iter().max();
let sorted = {
    let mut copy = arr;
    copy.sort();
    copy
};
```

#### Slices

Slices are dynamically-sized views into contiguous sequences:

```fu
fn sum_all(data: &[i32]) -> i32 {
    data.iter().sum()
}

let arr = [1, 2, 3, 4, 5];
let slice: &[i32] = &arr;
let sub: &[i32] = &arr[1..4];

// Mutable slices
fn double_all(data: &mut [i32]) {
    for item in data.iter_mut() {
        *item *= 2;
    }
}

let mut arr = [1, 2, 3, 4, 5];
double_all(&mut arr);

// Slice patterns
fn describe(data: &[i32]) {
    match data {
        [] => println!("empty"),
        [x] => println!("single: {}", x),
        [x, y] => println!("pair: {} and {}", x, y),
        [first, .., last] => println!("first: {}, last: {}", first, last),
    }
}
```

### 2.3 Pointer Types

#### Raw Pointers

Raw pointers are unsafe and have no ownership semantics:

```fu
let x = 42;
let ptr: *const i32 = &x as *const i32;   // immutable raw pointer
let mut y = 42;
let ptr_mut: *mut i32 = &mut y as *mut i32; // mutable raw pointer

// Dereferencing raw pointers is unsafe
unsafe {
    println!("{}", *ptr);
    *ptr_mut = 100;
}
```

#### References

References are borrowed pointers with lifetime guarantees:

```fu
let x = 42;
let r: &i32 = &x;        // immutable reference
let mut y = 42;
let r_mut: &mut i32 = &mut y; // mutable reference

// References cannot outlive their referent
{
    let r;
    {
        let x = 42;
        r = &x;  // ERROR: `x` does not live long enough
    }
}

// Shared references
fn count(data: &[i32]) -> usize {
    data.len()
}
```

#### Smart Pointers

```fu
use std::boxed::Box;
use std::rc::Rc;
use std::arc::Arc;

// Box — heap-allocated, single ownership
let b: Box<i32> = Box::new(42);
let deref: i32 = *b;

// Rc — reference-counted, single-threaded shared ownership
let shared: Rc<string> = Rc::new("hello".into());
let clone = Rc::clone(&shared);

// Arc — atomically reference-counted, thread-safe shared ownership
let thread_safe: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
```

### 2.4 Function Types

Functions have first-class types:

```fu
// Function type: fn(i32) -> i32
let double: fn(i32) -> i32 = |x| x * 2;

// Function type: fn(f64, f64) -> f64
let add: fn(f64, f64) -> f64 = |a, b| a + b;

// Higher-order function
fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

let result = apply(double, 5);  // 10

// Functions with closures
fn make_adder(n: i32) -> fn(i32) -> i32 {
    move |x| x + n
}

let add5 = make_adder(5);
let result = add5(10);  // 15
```

### 2.5 Closure Types

Closures are anonymous functions that capture their environment:

```fu
// Unnamed closure type: |T| -> U
let square = |x: i32| -> i32 { x * x };

// Inferred types
let add = |a, b| a + b;  // inferred as fn(i32, i32) -> i32

// Closures that capture environment
let factor = 3;
let multiply = |x| x * factor;  // captures `factor`

// Mutable captures
let mut count = 0;
let increment = || {
    count += 1;
    count
};
increment();  // 1
increment();  // 2

// Closure trait bounds
fn apply_fn<F: Fn(i32) -> i32>(f: &F, x: i32) -> i32 {
    f(x)
}

// Three closure traits:
// Fn   — immutable capture, can be called multiple times
// FnMut — mutable capture, can be called multiple times
// FnOnce — takes ownership, can be called once

fn call_twice<F: Fn()>(f: F) {
    f();
    f();
}
```

### 2.6 Generic Types

```fu
// Generic function
fn first<T>(slice: &[T]) -> Option<&T> {
    if slice.is_empty() {
        Option::None
    } else {
        Option::Some(&slice[0])
    }
}

// Generic struct
struct Container<T> {
    value: T,
}

// Generic enum (built-in)
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Generic with trait bounds
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut max = &list[0];
    for item in &list[1..] {
        if item > max {
            max = item;
        }
    }
    max
}

// Multiple trait bounds
fn display_and_clone<T: Display + Clone>(item: &T) -> T {
    println!("{}", item);
    item.clone()
}

// where clause
fn complex<T, U>(t: &T, u: &U) -> string
where
    T: Display + Debug,
    U: Clone + Into<string>,
{
    format!("{}: {}", t, u.clone().into())
}

// Const generics
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

let m: Matrix<f64, 3, 3> = Matrix { data: [[0.0; 3]; 3] };
```

### 2.7 Quantum Types

Fusion provides first-class quantum types for hybrid quantum-classical programming:

```fu
use quantum::*;

// Qubit — a single quantum bit
let q: Qubit = Qubit::new();           // |0⟩ state
let q1: Qubit = Qubit::new();
let q2: Qubit = Qubit::new();

// QuantumCircuit — a sequence of quantum gates
let circuit: QuantumCircuit = QuantumCircuit::new(3); // 3 qubits

// QuantumState — represents a quantum state vector
let state: QuantumState = QuantumState::zero(2);  // |00⟩

// QuantumRegister — a register of qubits
let reg: QubitRegister = QubitRegister::new(8);  // 8-qubit register

// Gate operations
fn quantum_algorithm() {
    let mut circuit = QuantumCircuit::new(2);

    circuit.h(0);           // Hadamard on qubit 0
    circuit.cx(0, 1);       // CNOT: control=0, target=1
    circuit.measure([0, 1]); // measure both qubits

    let result: MeasurementResult = circuit.run();
    println!("Measured: {:?}", result.bits());
}
```

**Quantum type details:**

| Type               | Description                                    |
|--------------------|------------------------------------------------|
| `Qubit`            | Single qubit, initialized to \|0⟩              |
| `QuantumCircuit`   | Ordered sequence of gate operations            |
| `QuantumState`     | State vector representation                    |
| `QubitRegister`    | Fixed-size qubit register                      |
| `Gate`             | Single-qubit or multi-qubit gate               |
| `MeasurementResult`| Result of measuring qubits                     |
| `QubitPair`        | Entangled pair of qubits                       |
| `DensityMatrix`    | Density matrix for mixed states                |

### 2.8 Tensor Types

```fu
use tensor::*;

// Tensor<T, RANK> — typed N-dimensional array
let t: Tensor<f64, 1> = Tensor::from_vec(vec![1.0, 2.0, 3.0]);
let mat: Tensor<f64, 2> = Tensor::from_shape((3, 3));
let cube: Tensor<f64, 3> = Tensor::from_shape((2, 2, 2));

// Tensor operations
let sum = t1 + t2;            // element-wise addition
let product = t1 * t2;        // element-wise multiplication
let dot = t1.dot(&t2);        // dot product
let matmul = m1.matmul(&m2);  // matrix multiplication
let reshaped = mat.reshape((9, 1));

// Tensor with compile-time rank
let vector: Tensor<f64, 1> = Tensor::zeros([3]);
let matrix: Tensor<f64, 2> = Tensor::zeros([3, 3]);

// Neural network operations
let dense = layer.dense(&weights, &bias);  // linear layer
let activated = dense.relu();               // activation
let softmax = activated.softmax();          // softmax
```

**Tensor rank type:**

```fu
// Rank is part of the type — mismatched ranks are compile errors
fn matmul<const M: usize, const N: usize, const K: usize>(
    a: &Tensor<f64, 2>,  // M x K
    b: &Tensor<f64, 2>,  // K x N
) -> Tensor<f64, 2> {    // M x N
    // compile-time checked dimensions
}
```

### 2.9 Hybrid Types

Fusion provides hybrid types that bridge classical and quantum computation:

```fu
use hybrid::*;

// ClassicalValue — a classical bit or integer
enum ClassicalValue {
    Bit(bool),
    Integer(i64),
    Float(f64),
    String(string),
}

// QuantumValue — a quantum measurement outcome
enum QuantumValue {
    Zero,
    One,
    Superposition(f64, f64),  // (probability_0, probability_1)
    Entangled(Qubit, Qubit),
}

// TensorValue — a tensor of any rank
enum TensorValue {
    Scalar(f64),
    Vector(Vec<f64>),
    Matrix(Vec<Vec<f64>>),
    Tensor(Tensor<f64, ?>),  // dynamic rank
}

// Hybrid computation
fn hybrid_algorithm() {
    // Classical preprocessing
    let input: ClassicalValue = ClassicalValue::Integer(42);

    // Quantum processing
    let q = Qubit::new();
    let q = q.h().cx(q2);  // create superposition + entanglement

    // Measurement
    let result: QuantumValue = q.measure();

    // Classical post-processing
    match result {
        QuantumValue::Zero => println!("Got |0⟩"),
        QuantumValue::One => println!("Got |1⟩"),
        QuantumValue::Superposition(p0, p1) => {
            println!("Amplitudes: |0⟩={}, |1⟩={}", p0, p1);
        }
        _ => {}
    }
}
```

### 2.10 Type Aliases

```fu
type Kilometers = f64;
type Meter = f64;
type Result<T> = std::result::Result<T, FusionError>;
type Callback = fn(i32) -> i32;

// Complex type aliases
type Matrix4x4 = [[f64; 4]; 4];
type HashMap<K, V> = std::collections::HashMap<K, V, std::collections::hash_map::RandomState>;
```

### 2.11 Type Inference

Fusion has a powerful type inference engine. Types are inferred from context:

```fu
// Inferred from literal suffix
let x = 42;           // i64
let y = 3.14;         // f64
let s = "hello";      // &str

// Inferred from usage
let mut v = Vec::new();
v.push(1);            // v is Vec<i32>

// Inferred from return type
fn parse(s: &str) -> i32 {
    s.parse().unwrap()  // parse knows to return i32
}

// Inferred from function signature
fn double(x: i32) -> i32 { x * 2 }
let result = double(5);  // result is i32

// Turbofish for ambiguous cases
let v = "42".parse::<i32>().unwrap();
```

---

## 3. Expressions

### 3.1 Literals

All literals are expressions:

```fu
42;             // i64 literal
3.14;           // f64 literal
true;           // bool literal
'a';            // char literal
"hello";        // string literal
[1, 2, 3];     // array literal
(1, "two");     // tuple literal
```

### 3.2 Variables

```fu
let x = 42;          // immutable
let mut y = 42;      // mutable
y += 1;              // OK
// x += 1;           // ERROR: cannot assign to immutable

// Shadowing
let x = 5;
let x = x + 1;       // new binding, shadows previous
let x = x * 2;       // 12
```

### 3.3 Function Calls

```fu
fn add(a: i32, b: i32) -> i32 { a + b }

// Positional arguments
let sum = add(3, 4);

// Named arguments (with attribute)
#[named]
fn greet(name: string, greeting: string) -> string {
    format!("{}, {}!", greeting, name)
}
let msg = greet(name: "Alice", greeting: "Hello");

// Method calls (dot notation)
let s = "hello world";
let upper = s.to_uppercase();
let len = s.len();

// Chained method calls
let result = "Hello, World!"
    .chars()
    .filter(|c| c.is_alphabetic())
    .collect::<string>();
```

### 3.4 Binary Operators

```fu
// Arithmetic
let a = 10 + 3;      // 13
let b = 10 - 3;      // 7
let c = 10 * 3;      // 30
let d = 10 / 3;      // 3 (integer division)
let e = 10 % 3;      // 1
let f = 2 ** 10;     // 1024

// Float arithmetic
let g = 10.0 / 3.0;  // 3.333...

// Bitwise
let h = 0b1100 & 0b1010;  // 0b1000
let i = 0b1100 | 0b1010;  // 0b1110
let j = 0b1100 ^ 0b1010;  // 0b0110
let k = 1 << 4;            // 16
let l = 16 >> 2;           // 4

// Comparison
let m = 5 == 5;      // true
let n = 5 != 3;      // true
let o = 5 > 3;       // true
let p = 5 < 3;       // false
let q = 5 >= 5;      // true
let r = 5 <= 3;      // false

// Logical
let s = true && false;  // false
let t = true || false;  // true
let u = !true;          // false
```

### 3.5 Unary Operators

```fu
let x = -42;           // negation
let y = !true;         // logical NOT
let z = !0u8;          // bitwise NOT (0xFF)
let ptr = &x;          // reference
let val = *ptr;        // dereference
```

### 3.6 Match Expressions

Match is the primary control flow expression:

```fu
// Basic match
let x = 5;
let description = match x {
    0 => "zero",
    1 => "one",
    2..=9 => "single digit",
    _ => "other",
};

// Match with destructuring
let point = (3, 4);
let quadrant = match point {
    (0, 0) => "origin",
    (x, 0) => "on x-axis",
    (0, y) => "on y-axis",
    (x, y) if x > 0 && y > 0 => "quadrant I",
    (x, y) if x < 0 && y > 0 => "quadrant II",
    (x, y) if x < 0 && y < 0 => "quadrant III",
    _ => "quadrant IV",
};

// Match with enum
let shape = Shape::Circle(5.0);
let area = match shape {
    Shape::Circle(r) => 3.14159 * r * r,
    Shape::Rectangle(w, h) => w * h,
    Shape::Triangle { a, b, c } => {
        let s = (a + b + c) / 2.0;
        (s * (s - a) * (s - b) * (s - c)).sqrt()
    }
};

// Match with guards
let num = 42;
let classification = match num {
    n if n % 2 == 0 => "even",
    n if n % 3 == 0 => "divisible by 3",
    n if n > 100 => "large",
    _ => "other",
};

// Match with bindings
let message = match parse_input() {
    Ok(data) => format!("Got: {}", data),
    Err(ParseError::InvalidSyntax { line, col }) => {
        format!("Syntax error at {}:{}", line, col)
    }
    Err(e) => format!("Error: {}", e),
};

// Exhaustive matching
enum Color { Red, Green, Blue }
let name = match Color::Red {
    Color::Red => "red",
    Color::Green => "green",
    Color::Blue => "blue",
    // compiler enforces all variants handled
};

// Match as expression (returns a value)
let x = if condition { 1 } else { 2 };
```

### 3.7 Closures and Lambda Expressions

```fu
// Closure with explicit types
let add = |a: i32, b: i32| -> i32 { a + b };

// Closure with inferred types
let multiply = |a, b| a * b;

// Closure capturing environment
let factor = 10;
let scale = |x| x * factor;

// Closure with multiple statements
let process = |x: i32| {
    let doubled = x * 2;
    let incremented = doubled + 1;
    incremented
};

// Closure as function argument
let numbers = [1, 2, 3, 4, 5];
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);

// Closures with different capture modes
let mut data = vec![1, 2, 3];

// Fn — borrows immutably
let print_len = || println!("len: {}", data.len());

// FnMut — borrows mutably
let push = || data.push(4);

// FnOnce — takes ownership
let consume = || {
    let owned = data;
    println!("consumed: {:?}", owned);
};

// Moving ownership into closure
let name = String::from("Fusion");
let greet = move || {
    println!("Hello, {}!", name);
};
// name is no longer accessible here
```

### 3.8 Array and Slice Expressions

```fu
// Array literal
let arr = [1, 2, 3, 4, 5];

// Array repeat expression
let zeros = [0; 10];

// Array indexing (bounds-checked)
let first = arr[0];
// let bad = arr[10];  // runtime panic

// Array slicing
let slice = &arr[1..4];     // [2, 3, 4]
let from = &arr[2..];       // [3, 4, 5]
let to = &arr[..3];         // [1, 2, 3]

// Array methods
let len = arr.len();
let reversed = arr.iter().rev().collect::<Vec<&i32>>();
let sorted = {
    let mut copy = arr;
    copy.sort();
    copy
};

// Multi-dimensional arrays
let matrix: [[i32; 3]; 3] = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9],
];
let elem = matrix[1][2];  // 6

// Array as function parameter
fn sum_array(arr: &[i32]) -> i32 {
    arr.iter().sum()
}
```

### 3.9 Struct Literal Expressions

```fu
struct Point { x: f64, y: f64 }

// Full literal
let p = Point { x: 1.0, y: 2.0 };

// Field shorthand
let x = 1.0;
let y = 2.0;
let p = Point { x, y };

// Struct update syntax
let p2 = Point { x: 5.0, ..p };

// Struct with methods
impl Point {
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn origin() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
}

// Tuple struct literal
struct Color(u8, u8, u8);
let red = Color(255, 0, 0);

// Unit struct literal
struct Marker;
let m = Marker;
```

### 3.10 Method Calls (Dot Notation)

```fu
struct Calculator {
    value: f64,
}

impl Calculator {
    fn new(initial: f64) -> Self {
        Calculator { value: initial }
    }

    fn add(&mut self, x: f64) -> &mut Self {
        self.value += x;
        self
    }

    fn multiply(&mut self, x: f64) -> &mut Self {
        self.value *= x;
        self
    }

    fn result(&self) -> f64 {
        self.value
    }
}

// Chained method calls
let result = Calculator::new(1.0)
    .add(2.0)
    .multiply(3.0)
    .result();  // 9.0

// Method on built-in types
let s = "Hello, World!";
let upper = s.to_uppercase();
let has_world = s.contains("World");
let chars: Vec<char> = s.chars().collect();
```

### 3.11 Type Ascription

```fu
// Explicit type annotation
let x: i32 = 42;
let y: f64 = 3.14;

// As expression (type cast)
let int_val = 3.14 as i32;         // 3
let float_val = 42 as f64;         // 42.0
let byte_val = 256 as u8;          // 0 (wrapping)

// Safe conversion with try_into
let big: i64 = 1000;
let small: Result<u8, _> = big.try_into();

// Reference type conversion
let s: &str = "hello";
let owned: &String = &s.to_string();
```

---

## 4. Statements

### 4.1 Variable Declarations

```fu
// Immutable binding
let x = 42;

// Mutable binding
let mut y = 42;
y += 1;

// Type annotation
let z: i32 = 42;

// Destructuring
let (a, b, c) = (1, 2, 3);
let Point { x, y } = point;

// Array destructuring
let [first, second, ..rest] = [1, 2, 3, 4, 5];

// Shadowing
let x = 5;
let x = x + 1;  // new binding

// Pattern destructuring
let Ok(value) = parse("42") else {
    panic!("parse failed");
};
```

### 4.2 Assignments

```fu
let mut x = 42;
x = 100;              // simple assignment
x += 10;              // add-assign
x -= 5;               // subtract-assign
x *= 2;               // multiply-assign
x /= 3;               // divide-assign
x %= 7;               // modulo-assign
x &= 0xFF;            // AND-assign
x |= 0x10;            // OR-assign
x ^= 0x01;            // XOR-assign
x <<= 2;              // left-shift-assign
x >>= 1;              // right-shift-assign

// Destructuring assignment
let mut a = 1;
let mut b = 2;
(a, b) = (b, a);  // swap

// Field assignment
let mut p = Point { x: 0.0, y: 0.0 };
p.x = 5.0;
p.y = 10.0;
```

### 4.3 If/Else

```fu
// Basic if
if condition {
    do_something();
}

// If-else
if x > 0 {
    println!("positive");
} else {
    println!("non-positive");
}

// If-else-if chain
if x > 0 {
    println!("positive");
} else if x < 0 {
    println!("negative");
} else {
    println!("zero");
}

// If as expression
let sign = if x > 0 { 1 } else if x < 0 { -1 } else { 0 };

// If with let-else
let Some(value) = option else {
    return Err("no value");
};

// Nested if
if a > 0 {
    if b > 0 {
        println!("both positive");
    } else {
        println!("a positive, b non-positive");
    }
}
```

### 4.4 While Loops

```fu
// Basic while
let mut i = 0;
while i < 10 {
    println!("{}", i);
    i += 1;
}

// While with break
let mut sum = 0;
let mut n = 1;
while sum < 1000 {
    sum += n;
    n += 1;
}
// n is now the smallest integer where sum >= 1000

// While with continue
let mut evens = Vec::new();
let mut i = 0;
while i < 20 {
    i += 1;
    if i % 2 != 0 {
        continue;
    }
    evens.push(i);
}
```

### 4.5 For-In Loops

```fu
// Iterate over range
for i in 0..10 {
    println!("{}", i);
}

// Inclusive range
for i in 0..=10 {
    println!("{}", i);
}

// Iterate over collection
let fruits = ["apple", "banana", "cherry"];
for fruit in fruits {
    println!("{}", fruit);
}

// Iterate with index
for (i, fruit) in fruits.iter().enumerate() {
    println!("{}. {}", i + 1, fruit);
}

// Iterate over characters
for c in "hello".chars() {
    println!("{}", c);
}

// Iterate over key-value pairs
let map = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
for (key, value) in &map {
    println!("{}: {}", key, value);
}

// Iterate over lines
for line in input.lines() {
    process_line(line);
}

// Iterators with methods
let sum: i32 = (1..=100).sum();
let evens: Vec<i32> = (1..=100).filter(|x| x % 2 == 0).collect();
let doubled: Vec<i32> = (1..=10).map(|x| x * 2).collect();
```

### 4.6 Return Statements

```fu
// Implicit return (last expression)
fn add(a: i32, b: i32) -> i32 {
    a + b  // no semicolon = return value
}

// Explicit return
fn find_first(data: &[i32], target: i32) -> Option<usize> {
    for (i, &val) in data.iter().enumerate() {
        if val == target {
            return Some(i);
        }
    }
    None
}

// Early return for error handling
fn process(input: &str) -> Result<i32, Error> {
    let parsed = input.parse::<i32>()?;
    if parsed < 0 {
        return Err(Error::NegativeValue);
    }
    Ok(parsed * 2)
}
```

### 4.7 Break and Continue

```fu
// break exits the nearest loop
let mut n = 0;
loop {
    if n > 10 {
        break;
    }
    n += 1;
}

// break with value (from loop expression)
let result = loop {
    n += 1;
    if n > 100 {
        break n;  // returns n from the loop
    }
};

// break to labeled loop
'outer: for i in 0..10 {
    for j in 0..10 {
        if i * j > 50 {
            break 'outer;
        }
    }
}

// continue skips to next iteration
for i in 0..10 {
    if i % 3 == 0 {
        continue;  // skip multiples of 3
    }
    println!("{}", i);
}

// continue to labeled loop
'outer: for i in 0..5 {
    'inner: for j in 0..5 {
        if j == 3 {
            continue 'outer;  // skip to next i
        }
        println!("{}: {}", i, j);
    }
}
```

### 4.8 Match Statements

Match can be used as a statement (when the result is discarded):

```fu
match command {
    "start" => start_process(),
    "stop" => stop_process(),
    "restart" => {
        stop_process();
        start_process();
    }
    cmd => unknown_command(cmd),
}

// Match with exhaustiveness checking
enum Event {
    Click { x: i32, y: i32 },
    KeyPress(char),
    Scroll(i32),
    Resize(i32, i32),
}

match event {
    Event::Click { x, y } => handle_click(x, y),
    Event::KeyPress(c) => handle_key(c),
    Event::Scroll(delta) => handle_scroll(delta),
    Event::Resize(w, h) => handle_resize(w, h),
    // compiler error if any variant missing
}
```

### 4.9 Expression Statements

Any expression followed by a semicolon becomes a statement (its value is discarded):

```fu
// Function call as statement
println!("hello");

// Assignment as statement
x = 5;

// Block as statement
{
    let temp = compute();
    use_temp(temp);
}

// Match as statement
match input {
    "quit" => return,
    "help" => print_help(),
    _ => println!("unknown"),
};
```

---

## 5. Declarations

### 5.1 Functions

```fu
// Basic function
fn greet(name: string) -> string {
    format!("Hello, {}!", name)
}

// Function with multiple parameters
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Function with no return value
fn print_line(msg: string) {
    println!("{}", msg);
}

// Function with default parameters (via overloading)
#[overload]
fn connect(host: string, port: u16 = 80, secure: bool = false) -> Connection { ... }

// Function with variadic arguments
fn printf(format: string, args: ...) { ... }

// Generic function
fn first<T>(slice: &[T]) -> Option<&T> {
    slice.first()
}

// Function with trait bounds
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    list.iter().max().unwrap()
}

// Closures as parameters
fn apply_to_all<F: Fn(i32) -> i32>(data: &mut [i32], f: F) {
    for item in data.iter_mut() {
        *item = f(*item);
    }
}

// Higher-order function returning closure
fn make_counter(start: i32) -> impl FnMut() -> i32 {
    let mut count = start;
    move || {
        let current = count;
        count += 1;
        current
    }
}

// Async function
async fn fetch_data(url: &str) -> Result<string, Error> {
    let response = http::get(url).await?;
    Ok(response.body())
}

// Quantum function
qfn grovers_search(database: &[bool], target: bool) -> usize {
    let n = database.len();
    let qubits = (n as f64).log2().ceil() as usize;
    let mut circuit = QuantumCircuit::new(qubits);

    // Apply Hadamard to all qubits
    for i in 0..qubits {
        circuit.h(i);
    }

    // Oracle + diffusion (repeated)
    let iterations = ((PI / 4.0) * (n as f64).sqrt()).floor() as usize;
    for _ in 0..iterations {
        oracle(&mut circuit, database, target);
        diffusion(&mut circuit);
    }

    // Measure
    circuit.measure_all()
}
```

### 5.2 External Functions (FFI)

```fu
// C FFI
extern "C" {
    fn strlen(s: *const u8) -> usize;
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn printf(format: *const u8, ...) -> i32;
}

// Rust FFI (via cdylib or staticlib)
extern "rust" {
    fn rust_function(x: i32) -> i32;
}

// Calling extern functions (unsafe)
unsafe {
    let len = strlen(b"hello\0".as_ptr());
    let ptr = malloc(100);
    // ... use ptr ...
    free(ptr);
}
```

### 5.3 Structs

```fu
// Named fields
struct Point {
    x: f64,
    y: f64,
}

// Tuple struct
struct Meters(f64);
struct Color(u8, u8, u8);

// Unit struct
struct Marker;

// Generic struct
struct Vec<T> {
    data: *mut T,
    len: usize,
    cap: usize,
}

// Struct with methods
impl Point {
    // Associated function (constructor)
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    // Method
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    // Mutable method
    fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }

    // Consuming method
    fn into_tuple(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

// Struct with trait implementation
impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

### 5.4 Enums

```fu
// Simple enum
enum Direction {
    North,
    South,
    East,
    West,
}

// Enum with data
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle { a: f64, b: f64, c: f64 },
}

// Enum with methods
impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => 3.14159 * r * r,
            Shape::Rectangle(w, h) => w * h,
            Shape::Triangle { a, b, c } => {
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
        }
    }
}

// Enum with trait implementation
impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::South => write!(f, "South"),
            Direction::East => write!(f, "East"),
            Direction::West => write!(f, "West"),
        }
    }
}

// Enum with discriminants
enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    ServerError = 500,
}
```

### 5.5 Traits

```fu
// Basic trait
trait Printable {
    fn print(&self);
}

// Trait with default implementation
trait Summary {
    fn summarize(&self) -> string;

    fn preview(&self) -> string {
        format!("{}...", &self.summarize()[..50])
    }
}

// Trait with generic methods
trait Convertible<T> {
    fn convert(&self) -> T;
}

// Trait with associated types
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Implementing traits
impl Printable for Point {
    fn print(&self) {
        println!("Point({}, {})", self.x, self.y);
    }
}

// Blanket implementation
impl<T: std::fmt::Display> Printable for T {
    fn print(&self) {
        println!("{}", self);
    }
}

// Trait bounds in functions
fn print_all<T: Printable>(items: &[T]) {
    for item in items {
        item.print();
    }
}

// Multiple trait bounds
fn display_and_debug<T: std::fmt::Display + std::fmt::Debug>(item: &T) {
    println!("Display: {}", item);
    println!("Debug: {:?}", item);
}

// where clause
fn complex_function<T, U>(t: &T, u: &U)
where
    T: std::fmt::Display + Clone,
    U: std::fmt::Debug + Into<string>,
{
    println!("{}", t);
    let s: string = u.into();
    println!("{}", s);
}
```

### 5.6 Impl Blocks

```fu
// Inherent impl (methods on a type)
impl Point {
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    fn origin() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
}

// Trait impl
impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// Generic impl
impl<T> Vec<T> {
    fn new() -> Vec<T> {
        Vec { data: ptr::null_mut(), len: 0, cap: 0 }
    }

    fn push(&mut self, item: T) { ... }
    fn pop(&mut self) -> Option<T> { ... }
    fn len(&self) -> usize { self.len }
}

// Impl with where clause
impl<T> Container<T>
where
    T: std::fmt::Display,
{
    fn print(&self) {
        println!("{}", self.value);
    }
}
```

### 5.7 Constants and Statics

```fu
// Compile-time constant
const MAX_SIZE: usize = 1024;
const PI: f64 = 3.141592653589793;
const GREETING: string = "Hello, World!";

// Static variable (has fixed address, can be mutable)
static mut COUNTER: i32 = 0;

fn increment() {
    unsafe {
        COUNTER += 1;
    }
}

// Const with computed values
const ARRAY_SIZE: usize = 10;
const SUM: i32 = {
    let mut total = 0;
    let mut i = 0;
    while i < ARRAY_SIZE {
        total += i as i32;
        i += 1;
    }
    total
};

// Static with complex initialization
static LOOKUP: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = (i * 2) as u8;
        i += 1;
    }
    table
};
```

### 5.8 Modules

```fu
// Module declaration
mod network {
    pub mod tcp {
        pub fn connect(addr: &str) -> Connection { ... }
    }

    pub mod udp {
        pub fn send(addr: &str, data: &[u8]) -> usize { ... }
    }
}

// Using modules
use network::tcp;
use network::udp::{send, Connection};

// Path-based access
let conn = network::tcp::connect("127.0.0.1:8080");

// External crate import
import std::collections::HashMap;
import std::io::{self, Read, Write};

// Renaming imports
use std::collections::HashMap as Map;
use network::tcp::connect as tcp_connect;

// Glob import
use std::collections::*;

// Crate root reference
use crate::utils;
use super::parent_module;
```

### 5.9 Use and Import Statements

```fu
// Single item
use std::io;

// Multiple items from same module
use std::io::{self, Read, Write, BufRead};

// Nested paths
use std::{io, fs, net};

// Alias
use std::collections::HashMap as Map;

// Re-export
pub use crate::internal::PublicType;

// External crate
import serde::{Serialize, Deserialize};

// Feature-gated import
#[cfg(feature = "quantum")]
import quantum::{Qubit, QuantumCircuit};
```

### 5.10 Visibility

```fu
// Private by default
struct InternalData {
    secret: i32,      // private
}

// Public
pub struct PublicData {
    pub value: i32,   // public field
    internal: i32,    // private field
}

// Public function
pub fn public_api() { }

// Private function
fn internal_helper() { }

// Public module with private items
pub mod api {
    pub fn public_function() { }
    fn private_function() { }  // not accessible outside module
}

// Public re-export
pub use internal::PublicType;

// Visibility modifiers
pub(crate) fn crate_visible() { }     // visible within crate
pub(super) fn parent_visible() { }    // visible to parent module
pub(in path) fn path_visible() { }    // visible within specified path
```

---

## 6. Memory Safety

### 6.1 Ownership Model

Every value in Fusion has exactly one owner. When the owner goes out of scope, the value is dropped (deallocated).

```fu
// Ownership transfer (move)
let s1 = String::from("hello");
let s2 = s1;  // s1 is moved to s2
// println!("{}", s1);  // ERROR: s1 is no longer valid

// Clone (explicit copy)
let s1 = String::from("hello");
let s2 = s1.clone();  // deep copy
println!("{} {}", s1, s2);  // OK: both valid

// Copy types (stack-allocated, implicitly copied)
let x: i32 = 42;
let y = x;  // x is copied, not moved
println!("{} {}", x, y);  // OK: both valid

// Function ownership
fn take_ownership(s: string) {
    println!("{}", s);
}  // s is dropped here

fn make_ownership() -> string {
    let s = String::from("hello");
    s  // ownership transferred to caller
}

let s = make_ownership();
take_ownership(s);  // s is moved into function
// println!("{}", s);  // ERROR: s was moved
```

### 6.2 Borrowing Rules

Fusion enforces the following borrowing rules at compile time:

1. At any given time, you can have EITHER one mutable reference OR any number of immutable references.
2. References must always be valid (no dangling references).

```fu
// Immutable borrowing
fn calculate_length(s: &string) -> usize {
    s.len()
}  // borrow ends here

let s = String::from("hello");
let len = calculate_length(&s);  // borrow s
println!("{} {}", s, len);       // OK: s is still valid

// Mutable borrowing
fn append_world(s: &mut string) {
    s.push_str(", world");
}

let mut s = String::from("hello");
append_world(&mut s);
println!("{}", s);  // "hello, world"

// Cannot have mutable and immutable references simultaneously
let r1 = &s;       // immutable borrow
let r2 = &s;       // immutable borrow (OK)
// let r3 = &mut s; // ERROR: cannot borrow as mutable while borrowed immutably
println!("{} {}", r1, r2);
// r1 and r2 are no longer used after this point
let r3 = &mut s;   // OK: r1 and r2 are no longer in use

// NLL (Non-Lexical Lifetimes) — borrows end at last use
let mut data = vec![1, 2, 3];
let first = &data[0];     // immutable borrow
println!("{}", first);     // last use of `first`
data.push(4);              // OK: first is no longer borrowed
```

### 6.3 Vortex Safety Engine

The Vortex Safety Engine is Fusion's compile-time safety verification system. It enforces:

1. **No null pointers** — All references are guaranteed to be non-null.
2. **No data races** — No two threads can access the same data with at least one write.
3. **No use-after-free** — References cannot outlive their referents.
4. **No double-free** — Each value is dropped exactly once.
5. **No buffer overflow** — Array indexing is bounds-checked.

```fu
// Vortex engine annotations
#[vortex(safety = "verified")]
fn safe_function(data: &[i32]) -> i32 {
    data.iter().sum()
}

// Unsafe blocks require vortex verification
unsafe {
    // The Vortex engine still checks what it can
    let ptr = raw_ptr;
    // Must prove: ptr is valid, aligned, and not aliased
    std::ptr::write(ptr, 42);
}

// Vortex proves absence of undefined behavior
#[vortex(prove = "no-alias")]
unsafe fn no_alias_function(a: *mut i32, b: *mut i32) {
    // Compiler proves *a and *b don't alias
    *a = 1;
    *b = 2;
}
```

### 6.4 Affine Types

Fusion uses affine type tracking for linear resource management:

```fu
// File handle is affine — can be used exactly once
let file = File::open("data.txt")?;
// file is consumed by read_to_string
let content = file.read_to_string()?;
// file is no longer valid

// Mutex lock is affine
let lock = Mutex::new(42);
let guard = lock.lock()?;
*guard += 1;
// guard is consumed when it goes out of scope

// Quantum qubit is affine (no-cloning theorem)
let q = Qubit::new();
// let q2 = q;  // ERROR: qubit cannot be copied
// q must be consumed by a gate operation
let q = q.h();  // consumed and returned
```

### 6.5 Move vs Copy Semantics

```fu
// Copy types (implement Copy trait) — stack-allocated, implicitly copied
// Primitives: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, char
// Tuples of Copy types
// Arrays of Copy types
// References

let x: i32 = 42;
let y = x;  // copied
println!("{} {}", x, y);  // OK

// Move types (do NOT implement Copy) — heap-allocated, moved on assignment
// String, Vec, Box, Arc, Mutex, File, etc.

let s1 = String::from("hello");
let s2 = s1;  // moved
// println!("{}", s1);  // ERROR

// Explicit clone
let s1 = String::from("hello");
let s2 = s1.clone();  // deep copy
println!("{} {}", s1, s2);  // OK

// Custom Copy implementation
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

let p1 = Point { x: 1.0, y: 2.0 };
let p2 = p1;  // copied (both valid)
println!("{:?} {:?}", p1, p2);  // OK
```

---

## 7. Concurrency

### 7.1 Fibers and Cooperative Scheduling

Fusion uses green threads (fibers) for cooperative multitasking:

```fu
use std::fiber;

// Spawn a fiber
let handle = fiber::spawn(|| {
    for i in 0..5 {
        println!("fiber: {}", i);
        fiber::yield_now();  // yield control
    }
});

// Fiber with return value
let handle = fiber::spawn(|| {
    let mut sum = 0;
    for i in 1..=100 {
        sum += i;
        fiber::yield_now();
    }
    sum
});

// Wait for fiber to complete
let result = handle.join();
println!("sum: {}", result);

// Fiber with message passing
let (tx, rx) = fiber::channel();

fiber::spawn(move || {
    for i in 0..5 {
        tx.send(i).unwrap();
        fiber::yield_now();
    }
});

while let Ok(value) = rx.recv() {
    println!("received: {}", value);
}

// Multiple fibers
let handles: Vec<_> = (0..5)
    .map(|i| {
        fiber::spawn(move || {
            println!("fiber {}", i);
            i * 2
        })
    })
    .collect();

let results: Vec<_> = handles.into_iter().map(|h| h.join()).collect();
```

### 7.2 Message Passing

```fu
use std::sync::mpsc;

// Channel for message passing
let (tx, rx) = mpsc::channel();

// Sender (cloneable)
let tx1 = tx.clone();
let tx2 = tx.clone();

// Spawn producers
fiber::spawn(move || {
    tx1.send("message from fiber 1").unwrap();
});

fiber::spawn(move || {
    tx2.send("message from fiber 2").unwrap();
});

// Drop original sender
drop(tx);

// Receive all messages
for msg in rx {
    println!("received: {}", msg);
}

// Typed channels
let (tx, rx) = mpsc::channel::<i32>();

// Bounded channels (backpressure)
let (tx, rx) = mpsc::sync_channel(100);  // buffer size 100

// Async channels
use std::sync::mpsc::async_channel;
let (tx, rx) = async_channel::unbounded();

async fn producer(tx: async_channel::Sender<i32>) {
    for i in 0..10 {
        tx.send(i).await.unwrap();
    }
}

async fn consumer(rx: async_channel::Receiver<i32>) {
    while let Ok(value) = rx.recv().await {
        println!("consumed: {}", value);
    }
}
```

### 7.3 Shared State

```fu
use std::sync::{Arc, Mutex, RwLock};

// Mutex — mutual exclusion lock
let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(fiber::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    }));
}

for handle in handles {
    handle.join();
}

println!("result: {}", *counter.lock().unwrap());  // 10

// RwLock — multiple readers, single writer
let data = Arc::new(RwLock::new(vec![1, 2, 3]));

let mut handles = vec![];

// Readers
for _ in 0..5 {
    let data = Arc::clone(&data);
    handles.push(fiber::spawn(move || {
        let read = data.read().unwrap();
        println!("read: {:?}", *read);
    }));
}

// Writer
{
    let data = Arc::clone(&data);
    handles.push(fiber::spawn(move || {
        let mut write = data.write().unwrap();
        write.push(4);
    }));
}

for handle in handles {
    handle.join();
}

// Atomic types for simple shared state
use std::sync::atomic::{AtomicI64, AtomicBool, Ordering};

static COUNTER: AtomicI64 = AtomicI64::new(0);
static FLAG: AtomicBool = AtomicBool::new(false);

// Lock-free operations
COUNTER.fetch_add(1, Ordering::SeqCst);
let value = COUNTER.load(Ordering::SeqCst);
FLAG.store(true, Ordering::Release);
```

### 7.4 Async/Await

```fu
use std::async;

// Async function
async fn fetch_data(url: &str) -> Result<string, Error> {
    let response = http::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

// Async block
let result = async {
    let data = fetch_data("https://api.example.com").await?;
    process(data).await
}.await?;

// Async with select
async fn handle_request(req: Request) -> Response {
    select! {
        response = fetch_data(&req.url) => {
            Response::ok(response)
        }
        _ = timeout(Duration::from_secs(5)) => {
            Response::timeout()
        }
    }
}

// Async streams
async fn stream_data() -> impl Stream<Item = Data> {
    stream::iter(0..100)
        .then(|i| async move {
            Data::new(i).await
        })
        .filter(|data| data.is_valid())
}

// Async iterator
async fn process_stream() {
    let mut stream = stream_data().await;
    while let Some(data) = stream.next().await {
        process(data);
    }
}

// Spawn async task
let handle = async::spawn(async {
    let result = fetch_data("https://api.example.com").await?;
    Ok(result)
});

let data = handle.await?;

// Async with fibers
let handle = fiber::spawn(|| {
    // Fiber can run async code
    async_block().await
});
```

---

## 8. Quantum Computing

### 8.1 Qubit Primitives

```fu
use quantum::*;

// Create a qubit (initialized to |0⟩)
let q = Qubit::new();

// Create a qubit in |1⟩ state
let q = Qubit::one();

// Create a qubit in arbitrary state
let q = Qubit::new_with_state(1.0 / 2.0_f64.sqrt(), 1.0 / 2.0_f64.sqrt());

// Qubit register
let reg = QubitRegister::new(5);  // 5 qubits, all |0⟩

// Access individual qubits in register
let q0 = reg.get(0);
let q1 = reg.get(1);

// Qubit state inspection (non-destructive)
let state = q.state();  // returns QuantumState
let probs = q.probabilities();  // returns [prob_0, prob_1]

// Qubit is affine (no-cloning theorem enforced)
let q = Qubit::new();
// let q2 = q;  // ERROR: qubits cannot be copied
let q = q.h();  // consumed and returned
```

### 8.2 Gate Operations

```fu
use quantum::gates::*;

// Single-qubit gates
let q = Qubit::new();

// Hadamard gate (creates superposition)
let q = q.h();

// Pauli gates
let q = q.x();  // NOT gate
let q = q.y();  // Y gate
let q = q.z();  // Z gate (phase flip)

// Phase gates
let q = q.s();      // S gate (π/2 phase)
let q = q.t();      // T gate (π/4 phase)
let q = q.phase(θ); // arbitrary phase

// Rotation gates
let q = q.rx(θ);  // rotation around X axis
let q = q.ry(θ);  // rotation around Y axis
let q = q.rz(θ);  // rotation around Z axis

// Multi-qubit gates
let q1 = Qubit::new();
let q2 = Qubit::new();

// CNOT (Controlled-NOT)
let (q1, q2) = q1.cx(q2);

// CZ (Controlled-Z)
let (q1, q2) = q1.cz(q2);

// SWAP
let (q1, q2) = q1.swap(q2);

// Toffoli (CCNOT)
let (q1, q2, q3) = q1.ccx(q2, q3);

// Custom unitary gate
let gate = UnitaryGate::new(&[
    [1.0, 0.0],
    [0.0, complex(-1.0)],
]);
let q = q.apply(gate);
```

### 8.3 Circuit Construction

```fu
use quantum::*;

// Create a quantum circuit
let mut circuit = QuantumCircuit::new(3);  // 3-qubit circuit

// Add gates to circuit
circuit.h(0);              // Hadamard on qubit 0
circuit.cx(0, 1);          // CNOT: control=0, target=1
circuit.cx(1, 2);          // CNOT: control=1, target=2
circuit.measure([0, 1, 2]); // measure all qubits

// Run circuit
let result = circuit.run();
println!("Result: {:?}", result.bits());  // e.g., [1, 0, 1]

// Bell state preparation
fn bell_state() -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(2);
    circuit.h(0);
    circuit.cx(0, 1);
    circuit.measure([0, 1]);
    circuit
}

// GHZ state
fn ghz_state(n: usize) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(n);
    circuit.h(0);
    for i in 0..n-1 {
        circuit.cx(i, i + 1);
    }
    circuit.measure_all();
    circuit
}

// Circuit composition
let mut circuit = QuantumCircuit::new(2);
circuit.append(bell_state());  // compose circuits
circuit.append(custom_circuit());

// Circuit optimization
let optimized = circuit.optimize();
println!("Original depth: {}", circuit.depth());
println!("Optimized depth: {}", optimized.depth());
```

### 8.4 Measurement

```fu
use quantum::*;

// Measure a single qubit
let q = Qubit::new().h();  // superposition
let result = q.measure();  // collapses to |0⟩ or |1⟩

// Measure multiple qubits
let reg = QubitRegister::new(3);
reg.h(0);
reg.cx(0, 1);
reg.cx(1, 2);

let results = reg.measure([0, 1, 2]);
println!("Measured: {:?}", results.bits());  // e.g., [0, 0, 0] or [1, 1, 1]

// Measure with probability
let prob_0 = q.probability(0);  // probability of measuring |0⟩
let prob_1 = q.probability(1);  // probability of measuring |1⟩

// Expectation value
let expectation = q.expectation(PauliZ);  // ⟨Z⟩

// Density matrix
let rho = q.density_matrix();
println!("Density matrix: {:?}", rho);

// Partial trace
let rho_a = rho.partial_trace([1]);  // trace out qubit 1
```

### 8.5 Hybrid Quantum-Classical Programming

```fu
use quantum::*;
use hybrid::*;

// Hybrid algorithm: Variational Quantum Eigensolver (VQE)
fn vqe(hamiltonian: &Hamiltonian, initial_params: &[f64]) -> f64 {
    let mut params = initial_params.to_vec();
    let mut best_energy = f64::INFINITY;

    for iteration in 0..100 {
        // Classical: optimize parameters
        let energy = evaluate_energy(&params, hamiltonian);

        if energy < best_energy {
            best_energy = energy;
            println!("Iteration {}: energy = {}", iteration, energy);
        }

        // Quantum: prepare state and measure
        let circuit = build_ansatz(&params);
        let measured = circuit.run();

        // Classical: compute gradients
        let gradients = compute_gradients(&params, hamiltonian);

        // Classical: update parameters
        for (i, grad) in gradients.iter().enumerate() {
            params[i] -= 0.01 * grad;
        }
    }

    best_energy
}

// Hybrid computation with ClassicalValue and QuantumValue
fn hybrid_function(input: ClassicalValue) -> QuantumValue {
    match input {
        ClassicalValue::Integer(n) => {
            // Prepare quantum state based on classical input
            let mut circuit = QuantumCircuit::new(n as usize);
            for i in 0..n as usize {
                circuit.h(i);
            }
            let result = circuit.run();
            QuantumValue::from measurement(result)
        }
        ClassicalValue::Bit(b) => {
            let mut q = Qubit::new();
            if b {
                q = q.x();
            }
            QuantumValue::from q.measure()
        }
        _ => QuantumValue::Zero,
    }
}
```

---

## 9. Post-Quantum Cryptography

### 9.1 Hybrid Key Exchange

Fusion uses hybrid key exchange combining classical and post-quantum algorithms:

```fu
use crypto::pqc::*;

// Hybrid key exchange: X25519 + ML-KEM-768
let private_key = HybridPrivateKey::generate();
let public_key = private_key.public_key();

// Key exchange with peer
let peer_public_key = HybridPublicKey::from_bytes(peer_bytes)?;
let shared_secret = private_key.exchange(&peer_public_key)?;

// Shared secret properties:
// - Classical component: X25519 ECDH
// - PQC component: ML-KEM-768 (Kyber)
// - Combined with HKDF for forward secrecy

// Use shared secret
let encryption_key = hkdf::derive(
    &shared_secret,
    b"context",
    b"fusion-encryption-key",
    32,
)?;
```

### 9.2 Hybrid Signatures

```fu
use crypto::pqc::*;

// Hybrid signature: Ed25519 + ML-DSA-65
let private_key = HybridSignKey::generate();
let public_key = private_key.public_key();

// Sign a message
let message = b"Hello, post-quantum world!";
let signature = private_key.sign(message)?;

// Verify signature
let valid = public_key.verify(message, &signature)?;
assert!(valid);

// Signature contains both components:
// - Ed25519 classical signature
// - ML-DSA-65 (Dilithium) PQC signature
// Both must verify for the signature to be valid
```

### 9.3 50/50 Enforcement Policy

Fusion enforces a 50/50 policy for hybrid cryptography:

```fu
use crypto::pqc::*;

// 50/50 policy: both classical and PQC must be present
// This ensures security against both classical and quantum attacks

#[policy(hybrid = "50/50")]
struct SecureChannel {
    classical_key: X25519Key,
    pqc_key: MLKEM768Key,
}

impl SecureChannel {
    fn new() -> Result<Self, Error> {
        // Both keys must be generated
        let classical = X25519Key::generate()?;
        let pqc = MLKEM768Key::generate()?;

        Ok(SecureChannel {
            classical_key: classical,
            pqc_key: pqc,
        })
    }

    fn exchange(&self, peer: &SecureChannel) -> Result<SharedSecret, Error> {
        // Both exchanges must succeed
        let classical_secret = self.classical_key.exchange(&peer.classical_key)?;
        let pqc_secret = self.pqc_key.exchange(&peer.pqc_key)?;

        // Combine secrets
        let combined = hkdf::combine(&classical_secret, &pqc_secret)?;
        Ok(combined)
    }
}

// Policy enforcement at compile time
#[enforce_hybrid]
fn secure_function() {
    // Compiler ensures both classical and PQC are used
}
```

### 9.4 NeuralSeal PQC

Fusion includes NeuralSeal, a neural-network-enhanced PQC scheme:

```fu
use crypto::pqc::neuralseal::*;

// NeuralSeal key generation
let keypair = NeuralSeal::generate_keypair()?;

// NeuralSeal encryption
let plaintext = b"secret data";
let ciphertext = keypair.public_key().encrypt(plaintext)?;

// NeuralSeal decryption
let decrypted = keypair.private_key().decrypt(&ciphertext)?;
assert_eq!(&decrypted, plaintext);

// NeuralSeal with neural network acceleration
let config = NeuralSealConfig {
    network_size: 1024,
    use_gpu: true,
    security_level: SecurityLevel::High,
};

let seal = NeuralSeal::new(config);

// NeuralSeal signature
let signature = seal.sign(&keypair.private_key(), message)?;
let valid = seal.verify(&keypair.public_key(), message, &signature)?;

// Hybrid NeuralSeal + ML-KEM
let hybrid_key = HybridNeuralSealKey::generate()?;
let shared = hybrid_key.exchange(&peer_key)?;
```

---

## 10. Standard Library

### 10.1 I/O

```fu
use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};

// Reading from stdin
let mut input = String::new();
io::stdin().read_line(&mut input)?;

// Writing to stdout
print!("no newline");
println!("with newline");
eprintln!("to stderr");

// File I/O
let content = std::fs::read_to_string("data.txt")?;
std::fs::write("output.txt", "hello")?;

// Buffered I/O
let file = std::fs::File::open("data.txt")?;
let reader = BufReader::new(file);
for line in reader.lines() {
    println!("{}", line?);
}

// Network I/O
use std::net::{TcpListener, TcpStream};

let listener = TcpListener::bind("127.0.0.1:8080")?;
for stream in listener.incoming() {
    let stream = stream?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let mut line = String::new();
    reader.read_line(&mut line)?;
    writer.write_all(b"Response")?;
    writer.flush()?;
}

// Path manipulation
use std::path::{Path, PathBuf};

let path = Path::new("/usr/local/bin");
let parent = path.parent();        // Some("/usr/local")
let ext = path.extension();        // None
let file_name = path.file_name();  // Some("bin")

let mut path_buf = PathBuf::new();
path_buf.push("/usr");
path_buf.push("local");
path_buf.push("bin");
```

### 10.2 Strings

```fu
// String creation
let s1: string = "hello".into();
let s2: string = String::from("world");
let s3: string = format!("{} {}", s1, s2);
let s4: string = "x".repeat(10);

// String operations
let upper = s1.to_uppercase();
let lower = s1.to_lowercase();
let trimmed = "  hello  ".trim();
let contains = s1.contains("ell");
let starts = s1.starts_with("he");
let ends = s1.ends_with("lo");

// String splitting
let parts: Vec<&str> = "a,b,c".split(',').collect();
let lines: Vec<&str> = "line1\nline2\nline3".lines().collect();

// String joining
let joined = vec!["a", "b", "c"].join(", ");

// String replacement
let replaced = "hello world".replace("world", "Fusion");

// String parsing
let num: i32 = "42".parse()?;
let float: f64 = "3.14".parse()?;

// String slicing
let s = "hello world";
let slice = &s[0..5];  // "hello"
```

### 10.3 Collections

```fu
use std::collections::{Vec, HashMap, HashSet, BTreeMap, BTreeSet, VecDeque};

// Vec — dynamic array
let mut v = Vec::new();
v.push(1);
v.push(2);
v.push(3);

let v = vec![1, 2, 3, 4, 5];
let first = v[0];
let len = v.len();
let contains = v.contains(&3);
let filtered: Vec<&i32> = v.iter().filter(|&&x| x > 2).collect();

// HashMap — hash map
let mut map = HashMap::new();
map.insert("key1", 1);
map.insert("key2", 2);

let value = map.get("key1");
let contains = map.contains_key("key1");
let len = map.len();

// HashMap from iterator
let map: HashMap<i32, bool> = vec![(1, true), (2, false)].into_iter().collect();

// HashSet — hash set
let mut set = HashSet::new();
set.insert(1);
set.insert(2);
set.insert(3);

let contains = set.contains(&1);
let intersection: HashSet<i32> = set1.intersection(&set2).cloned().collect();
let union: HashSet<i32> = set1.union(&set2).cloned().collect();

// BTreeMap — sorted map
let mut btree = BTreeMap::new();
btree.insert(3, "c");
btree.insert(1, "a");
btree.insert(2, "b");

// BTreeSet — sorted set
let mut btree_set = BTreeSet::new();
btree_set.insert(3);
btree_set.insert(1);
btree_set.insert(2);
// Iterates in order: 1, 2, 3

// VecDeque — double-ended queue
let mut deque = VecDeque::new();
deque.push_back(1);
deque.push_front(0);
deque.pop_back();  // removes from back
deque.pop_front(); // removes from front
```

### 10.4 Filesystem

```fu
use std::fs;
use std::path::Path;

// Read file
let content = fs::read_to_string("file.txt")?;
let bytes = fs::read("file.bin")?;

// Write file
fs::write("output.txt", "content")?;

// File operations
let metadata = fs::metadata("file.txt")?;
let is_file = metadata.is_file();
let is_dir = metadata.is_dir();
let size = metadata.len();

// Directory operations
fs::create_dir("new_dir")?;
fs::create_dir_all("a/b/c")?;
fs::remove_dir("dir")?;
fs::remove_dir_all("dir")?;

// File operations
fs::rename("old.txt", "new.txt")?;
fs::copy("src.txt", "dst.txt")?;
fs::remove_file("file.txt")?;

// Directory listing
for entry in fs::read_dir(".")? {
    let entry = entry?;
    let path = entry.path();
    let metadata = entry.metadata()?;
    println!("{}: {} bytes", path.display(), metadata.len());
}

// Path operations
let path = Path::new("/usr/local/bin/file.txt");
assert!(path.exists());
assert!(path.is_file());
assert!(path.is_absolute());

let canonical = path.canonicalize()?;
let with_ext = path.with_extension("rs");
let parent = path.parent();
let file_name = path.file_name();
```

### 10.5 Math

```fu
use std::math;

// Basic operations
let sum = math::add(2.0, 3.0);
let product = math::mul(4.0, 5.0);
let power = math::pow(2.0, 10.0);
let sqrt = math::sqrt(144.0);
let abs = math::abs(-42.0);

// Trigonometry
let sin = math::sin(math::PI / 2.0);
let cos = math::cos(0.0);
let tan = math::tan(math::PI / 4.0);
let asin = math::asin(1.0);

// Logarithms
let ln = math::ln(math::E);
let log2 = math::log2(8.0);
let log10 = math::log10(1000.0);

// Constants
let pi = math::PI;
let e = math::E;
let inf = math::INFINITY;
let nan = math::NAN;

// Random numbers
use std::random;

let r: f64 = random::random();          // 0.0 to 1.0
let r: i32 = random::random_range(1, 100);  // 1 to 99
let r: f64 = random::normal(0.0, 1.0);  // normal distribution

// Complex numbers
use std::complex::Complex;

let z1 = Complex::new(1.0, 2.0);  // 1 + 2i
let z2 = Complex::new(3.0, 4.0);  // 3 + 4i
let sum = z1 + z2;  // 4 + 6i
let product = z1 * z2;  // -5 + 10i
let magnitude = z1.abs();  // sqrt(5)
```

### 10.6 Cryptography

```fu
use std::crypto;

// Hashing
let hash = crypto::sha256(b"hello world");
let hash_hex = hash.to_hex();

// HMAC
let hmac = crypto::hmac_sha256(b"key", b"message");

// Symmetric encryption (AES-GCM)
let key = crypto::random_key(32)?;
let nonce = crypto::random_nonce()?;
let ciphertext = crypto::aes_gcm_encrypt(&key, &nonce, b"plaintext")?;
let plaintext = crypto::aes_gcm_decrypt(&key, &nonce, &ciphertext)?;

// Key exchange (X25519)
let private_key = crypto::x25519::PrivateKey::generate()?;
let public_key = private_key.public_key();
let shared_secret = private_key.exchange(&peer_public_key)?;

// Digital signatures (Ed25519)
let signing_key = crypto::ed25519::SigningKey::generate()?;
let verifying_key = signing_key.verifying_key();
let signature = signing_key.sign(b"message");
let valid = verifying_key.verify(b"message", &signature)?;

// Post-quantum cryptography
use crypto::pqc::*;

// ML-KEM (Kyber)
let kem_key = MLKEM768::generate_keypair()?;
let (ciphertext, shared_secret) = kem_key.public_key().encapsulate()?;
let shared_secret2 = kem_key.private_key().decapsulate(&ciphertext)?;

// ML-DSA (Dilithium)
let dsa_key = MLDSA65::generate_keypair()?;
let signature = dsa_key.sign(b"message")?;
let valid = dsa_key.verify(b"message", &signature)?;

// Random number generation
let random_bytes = crypto::random_bytes(32)?;
let random_u64 = crypto::random_u64();
```

### 10.7 Networking

```fu
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr};
use std::net::http::{Client, Server, Request, Response};

// TCP client
let mut stream = TcpStream::connect("127.0.0.1:8080")?;
stream.write_all(b"Hello")?;

let mut buffer = [0; 1024];
let bytes_read = stream.read(&mut buffer)?;

// TCP server
let listener = TcpListener::bind("0.0.0.0:8080")?;
for stream in listener.incoming() {
    let stream = stream?;
    std::thread::spawn(move || {
        handle_client(stream);
    });
}

// UDP
let socket = UdpSocket::bind("0.0.0.0:9000")?;
socket.send_to(b"hello", "127.0.0.1:9001")?;

let mut buffer = [0; 1024];
let (bytes_read, src) = socket.recv_from(&mut buffer)?;

// HTTP client
let client = Client::new();
let response = client.get("https://api.example.com/data")?;
let body = response.text()?;

// HTTP server
let server = Server::new("0.0.0.0:3000")?;
server.get("/api/data", |_req| {
    Response::ok("Hello, World!")
})?;

// WebSocket
use std::net::websocket;

let ws = websocket::connect("ws://localhost:8080/ws")?;
ws.send(websocket::Message::text("hello"))?;
let msg = ws.recv()?;
```

### 10.8 Quantum Standard Library

```fu
use std::quantum::*;

// Quantum state manipulation
let state = QuantumState::zero(3);           // |000⟩
let state = QuantumState::plus(3);           // |+++⟩
let state = QuantumState::bell();            // Bell state

// Gate library
use quantum::gates::{H, X, Y, Z, CNOT, CZ, SWAP, Toffoli};

// Circuit simulation
let simulator = StatevectorSimulator::new(3);
let result = simulator.run(&circuit);

// Density matrix simulation
let dm_simulator = DensityMatrixSimulator::new(3);

// Noise models
let noise = DepolarizingNoise::new(0.01);  // 1% depolarizing
let noisy_circuit = circuit.apply_noise(&noise);

// Quantum error correction
use quantum::qec::*;

let code = SurfaceCode::new(7);  // distance-7 surface code
let logical_qubit = code.encode(&physical_qubit);
let corrected = code.correct(&logical_qubit, &syndrome);
```

### 10.9 AI/ML Standard Library

```fu
use std::ai::*;

// Neural network
let mut model = NeuralNetwork::new();
model.add(Dense::new(784, 128, Activation::ReLU));
model.add(Dense::new(128, 64, Activation::ReLU));
model.add(Dense::new(64, 10, Activation::Softmax));

// Training
model.compile(Optimizer::Adam, Loss::CrossEntropy);
model.fit(&training_data, epochs=10, batch_size=32);

// Inference
let prediction = model.predict(&input);

// Tensor operations
use std::tensor::*;

let t = Tensor::from_vec(vec![1.0, 2.0, 3.0]);
let reshaped = t.reshape([3, 1]);
let matmul = t1.matmul(&t2);

// Pretrained models
let model = models::ResNet50::pretrained()?;
let embeddings = model.encode(&image)?;

// Quantum ML
use quantum::ml::*;

let qml_model = QuantumMLModel::new(4);  // 4 qubits
qml_model.add(VariationalLayer::new(4, 2));
let result = qml_model.forward(&input);
```

### 10.10 Runtime

```fu
use std::runtime;

// Garbage collector information
let gc_stats = runtime::gc::stats();
println!("Allocated: {} bytes", gc_stats.allocated);
println!("Collections: {}", gc_stats.collections);

// Memory allocation
let layout = std::alloc::Layout::new::<[u8; 1024]>();
let ptr = unsafe { std::alloc::alloc(layout) };
unsafe { std::alloc::dealloc(ptr, layout); }

// Fiber runtime
let runtime = runtime::Runtime::new()?;
runtime.block_on(async {
    // async code
})?;

// Performance measurement
use std::time::Instant;

let start = Instant::now();
// ... work ...
let duration = start.elapsed();
println!("Elapsed: {:?}", duration);

// Profiling
use std::profiling;

profiling::start();
// ... work ...
profiling::stop();
profiling::report();

// Environment variables
let home = std::env::var("HOME")?;
let path = std::env::var("PATH")?;
std::env::set_var("MY_VAR", "value");

// Command line arguments
let args: Vec<string> = std::env::args().collect();
```

---

## 11. Compiler Flags

### 11.1 CLI Options

```
fusion [OPTIONS] <INPUT>

OPTIONS:
    -o, --output <FILE>        Output file path
    -c, --compile              Compile only (no link)
    -S, --assembly             Emit assembly code
    --emit <TYPE>              Emit specific output type
                               [llvm-ir, mir, hir, ast, tokens]
    -O, --optimize <LEVEL>     Optimization level [0, 1, 2, 3, s, z]
    -g, --debug-info           Include debug information
    --cfg <KEY[=VALUE]>        Set configuration flag
    --feature <FLAG>           Enable feature flag
    --target <TARGET>          Compilation target triple
    --edition <YEAR>           Language edition [2024, 2025, 2026]
    -W, --warn <LINT>          Set lint to warning
    -D, --deny <LINT>          Set lint to error
    -A, --allow <LINT>         Suppress lint
    --error-format <FMT>       Error output format [human, json, short]
    --color <WHEN>             Color output [always, never, auto]
    -V, --version              Print version
    -h, --help                 Print help
    -v, --verbose              Verbose output
    -q, --quiet                Suppress non-error output
    --no-default-features      Disable default features
    --extern <NAME=PATH>       Link external crate
    --crate-type <TYPE>        Crate type [bin, lib, dylib, cdylib, staticlib]
    --crate-name <NAME>        Crate name
    --out-dir <DIR>            Output directory
    --dep-info <FILE>          Write dependency information
    --json                     JSON output format
    --time-passes              Show timing for each pass
    --print <ITEM>             Print compiler information
    --explain <CODE>           Explain a compiler error code
    --error-index              Print error index
    --vortex-check             Run Vortex safety engine checks only
    --quantum-verify           Verify quantum circuit correctness
    --pqc-audit                Audit PQC implementation
```

### 11.2 Feature Flags

```
# Quantum computing support
--feature quantum
--feature quantum-sim
--feature quantum-hw

# Post-quantum cryptography
--feature pqc
--feature pqc-hybrid
--feature neuralseal

# AI/ML support
--feature ai
--feature tensor
--feature gpu

# Concurrency
--feature fibers
--feature async
--feature channels

# Safety
--feature vortex-strict
--feature vortex-prove
--feature no-unsafe

# Optimization
--feature lto           # Link-time optimization
--feature pgo           # Profile-guided optimization
--feature avx2          # AVX2 SIMD instructions
--feature avx512        # AVX-512 SIMD instructions
--feature neon          # ARM NEON SIMD instructions

# Debugging
--feature debug-alloc
--feature debug-gc
--feature debug-fibers

# Experimental
--feature const-generics
--feature const-fn
--feature effects
--feature generators
--feature try-blocks
```

### 11.3 Target Specifications

```
# Target triples
--target x86_64-unknown-linux-gnu
--target x86_64-unknown-linux-musl
--target x86_64-apple-darwin
--target aarch64-unknown-linux-gnu
--target aarch64-apple-darwin
--target wasm32-unknown-unknown
--target wasm32-wasi

# Custom target JSON
--target custom-target.json

# Target features
--target-feature +avx2
--target-feature +sse4.2
--target-feature +neon
--target-feature +vfpv4

# Target CPU
--target-cpu native
--target-cpu skylake
--target-cpu cortex-a72
```

### 11.4 Optimization Levels

| Level | Flag | Description |
|-------|------|-------------|
| 0     | `-O0` | No optimizations (fastest compilation) |
| 1     | `-O1` | Basic optimizations |
| 2     | `-O2` | Standard optimizations (default for release) |
| 3     | `-O3` | Aggressive optimizations |
| s     | `-Os` | Optimize for size |
| z     | `-Oz` | Optimize aggressively for size |

### 11.5 Compiler Output

```
# Compilation stages
--emit tokens           # Lexer output
--emit ast              # Abstract syntax tree
--emit hir              # High-level intermediate representation
--emit mir              # Mid-level intermediate representation (with Vortex checks)
--emit llvm-ir          # LLVM IR
--emit assembly         # Assembly code
--emit object           # Object file (default with -c)
--emit link             # Linked binary (default)

# Dependency tracking
--emit dep-info         # Write .d file for make
--emit link-deps        # Write link dependency info

# Metadata
--emit metadata         # Crate metadata
--emit metadata-sysroot # Sysroot metadata
```

### 11.6 Lint Configuration

```
# Built-in lints
-W unused-variables
-W unused-imports
-W dead-code
-W unreachable-code
-W missing-docs
-D unsafe-code
-D overflow
-D division-by-zero

# Vortex-specific lints
-W vortex borrow-check       # Borrow checker warnings
-D vortex use-after-free     # Use-after-free errors
-D vortex data-race          # Data race errors
-D vortex null-deref         # Null dereference errors
-D vortex buffer-overflow    # Buffer overflow errors

# Quantum lints
-W quantum state-collapse    # Unintended state collapse
-W quantum measurement-order # Measurement ordering issues
-D quantum no-cloning        # No-cloning theorem violations

# PQC lints
-W pqc weak-cipher           # Weak cipher suite
-D pqc classical-only        # Classical-only crypto (violation of 50/50)
-W pqc key-length            # Key length warnings
```

---

## Appendix A: Grammar Reference

The following is a simplified PEG-like grammar for Fusion:

```peg
# Program
program       = item* EOF
item          = fn_item | struct_item | enum_item | trait_item
              | impl_item | mod_item | use_item | const_item
              | static_item | type_item | extern_item

# Functions
fn_item       = "fn" IDENTIFIER generic_params? "(" params? ")" return_type?
                where_clause? block
extern_item   = "extern" string_literal? fn_item
params        = param ("," param)*
param         = pattern ":" type
return_type   = "->" type

# Types
type          = fn_type | ref_type | raw_ptr_type
              | path_type | tuple_type | array_type | slice_type
              | tensor_type | quantum_type
fn_type       = "fn" generic_params? "(" types? ")" return_type
ref_type      = "&" "mut"? type
raw_ptr_type  = "*" ("const" | "mut") type
path_type     = path ("::" IDENTIFIER)*
tuple_type    = "(" types? ")"
array_type    = "[" type ";" expr "]"
slice_type    = "[" type "]"
tensor_type   = "Tensor" "<" type "," NUMBER ">"
quantum_type  = "Qubit" | "QuantumCircuit" | "QuantumState"

# Patterns
pattern       = literal_pattern | identifier_pattern
              | tuple_pattern | struct_pattern | enum_pattern
              | slice_pattern | ref_pattern | mut_pattern
              | wildcard_pattern | or_pattern | range_pattern
literal_pattern = literal
identifier_pattern = "mut"? IDENTIFIER
tuple_pattern = "(" patterns? ")"
struct_pattern = path "{" field_patterns? "}"
enum_pattern  = path ("(" patterns? ")" | "{" field_patterns? "}")
slice_pattern = "[" patterns? "]"
wildcard_pattern = "_"
or_pattern    = pattern "|" pattern
range_pattern = literal ("..=" | "...") literal

# Expressions
expr          = literal | identifier | path | tuple_expr
              | array_expr | struct_expr | block_expr
              | if_expr | match_expr | while_expr | for_expr
              | loop_expr | fn_expr | closure_expr
              | call_expr | method_expr | field_expr
              | index_expr | binary_expr | unary_expr
              | as_expr | await_expr | yield_expr

binary_expr   = expr op expr
unary_expr    = unary_op expr
call_expr     = expr "(" args? ")"
method_expr   = expr "." IDENTIFIER ("(" args? ")")?
field_expr    = expr "." IDENTIFIER
index_expr    = expr "[" expr "]"

# Statements
stmt          = let_stmt | expr_stmt | semi_stmt | item
let_stmt      = "let" "mut"? pattern (":" type)? "=" expr ";"
expr_stmt     = expr ";"
semi_stmt     = ";"
```

---

## Appendix B: Operator Precedence (Visual Reference)

```
Highest
  ()
  []
  .
  ->

  !
  -
  *
  &
  @

  **

  *  /  %

  +  -

  <<  >>

  &

  ^

  |

  ==  !=  <  >  <=  >=  <:

  &&

  ||

  ..  ..=  ...

  =  +=  -=  *=  /=  %=  &=  |=  ^=  <<=  >>=  **=

  =>

Lowest
```

---

## Appendix C: Error Codes

Fusion uses structured error codes for compiler diagnostics:

| Code | Category | Description |
|------|----------|-------------|
| E0001 | Parse | Unexpected token |
| E0002 | Parse | Missing semicolon |
| E0003 | Parse | Invalid expression |
| E1001 | Type | Type mismatch |
| E1002 | Type | Cannot find value |
| E1003 | Type | Missing trait implementation |
| E1004 | Type | Type annotations needed |
| E2001 | Borrow | Cannot borrow as mutable |
| E2002 | Borrow | Cannot borrow as immutable |
| E2003 | Borrow | Reference does not live long enough |
| E2004 | Borrow | Use of moved value |
| E3001 | Vortex | Use-after-free detected |
| E3002 | Vortex | Data race detected |
| E3003 | Vortex | Null pointer dereference |
| E3004 | Vortex | Buffer overflow |
| E4001 | Quantum | No-cloning violation |
| E4002 | Quantum | Measurement ordering violation |
| E4003 | Quantum | Qubit already measured |
| E5001 | PQC | Classical-only cryptography |
| E5002 | PQC | Weak cipher suite |
| E5003 | PQC | Key length insufficient |

---

## Appendix D: Built-in Traits

| Trait | Description |
|-------|-------------|
| `Copy` | Implicit copy semantics |
| `Clone` | Explicit deep copy |
| `Debug` | Debug formatting (`{:?}`) |
| `Display` | User-facing formatting (`{}`) |
| `Hash` | Hash computation |
| `Eq` / `PartialEq` | Equality comparison |
| `Ord` / `PartialOrd` | Ordering comparison |
| `From` / `Into` | Type conversion |
| `TryFrom` / `TryInto` | Fallible type conversion |
| `Default` | Default value |
| `Iterator` | Iterator protocol |
| `IntoIterator` | Conversion to iterator |
| `Drop` | Cleanup on scope exit |
| `Add` / `Sub` / `Mul` / `Div` | Arithmetic operations |
| `Index` / `IndexMut` | Indexing operations |
| `Deref` / `DerefMut` | Smart pointer dereferencing |
| `Fn` / `FnMut` / `FnOnce` | Callable types |
| `Sized` | Compile-time sized |
| `Send` | Thread-safe transfer |
| `Sync` | Thread-safe sharing |
| `Unpin` | Safe to move after pinning |

---

*End of Fusion Language Specification v2.0 — Vortex Edition*
