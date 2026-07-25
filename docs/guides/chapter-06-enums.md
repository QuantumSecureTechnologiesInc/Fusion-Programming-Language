> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 6: Enums and Pattern Matching

Structs let you group related fields and data together. Usage of **enums** (enumerations) allows you to define a type by enumerating its possible *variants*.

Enums in Fusion are significantly more powerful than enums in languages like C, Java, or C#. In Fusion, enum variants can store data, essentially making them "Algebraic Data Types". This feature, combined with the `match` expression, makes handling complex data states safe and expressive.

In this chapter, we'll cover:
- Defining enums
- The `Option` enum
- The `match` control flow operator
- Concise control flow with `if let`

---

## 6.1 Defining an Enum

Let's look at a situation where we might want to express an IP address. Currently, two major standards are used: IPv4 and IPv6. We can express this concept with an enum:

```fusion
enum IpAddrKind {
    V4,
    V6,
}
```text

`IpAddrKind` is now a custom data type.

### 6.1.1 Enum Values

We can create instances of each of the two variants:

```fusion
let four = IpAddrKind::V4
let six = IpAddrKind::V6
```text

We can define a function that takes any `IpAddrKind`:

```fusion
fn route(ip_type: IpAddrKind) { }

route(IpAddrKind::V4)
route(IpAddrKind::V6)
```text

### 6.1.2 Storing Data Inside Variants

Pure enums are useful, but often we want to associate data with the variant. In Java or C++, you might create a struct with an "enum" field and a "data" field (and worry about which data field is valid for which enum).

Fusion solves this elegantly:

```fusion
enum IpAddr {
    V4(String),
    V6(String),
}

let home = IpAddr::V4(String::from("127.0.0.1"))
let loopback = IpAddr::V6(String::from("::1"))
```text

Each variant can contain different types or amounts of data:

```fusion
enum Message {
    Quit,                       // No data
    Move { x: i32, y: i32 },    // Named fields (like a struct)
    Write(String),              // Single String
    ChangeColor(i32, i32, i32), // Transformation to tuple
}
```text

This is incredibly powerful. A function accepting `Message` can handle any of these variants safely.

### 6.1.3 Defining Methods on Enums

Just like structs, you can define methods on enums using `impl`:

```fusion
impl Message {
    fn call(&self) {
        // Method body would likely match on self
    }
}

let m = Message::Write(String::from("hello"))
m.call()
```text

---

## 6.2 The `Option` Enum and Null Safety

Fusion does **not** have null.

In many languages, `null` is a value that means "no value". This leads to the "Billion Dollar Mistake": dereferencing a null value crashes the program.

Fusion captures the concept of "value present" vs "value absent" using the `Option<T>` enum, defined in the standard library as:

```fusion
enum Option<T> {
    Some(T),
    None,
}
```text

- `Some(T)` holds a value of type `T`.
- `None` represents the absence of a value.

### 6.2.1 Why Is Option Better Than Null?

Because `Option<T>` and `T` are different types, the compiler prevents you from using an `Option<T>` as if it were a valid value.

```fusion
let x: i8 = 5
let y: Option<i8> = Some(5)

// let sum = x + y  // Error! Cannot add i8 and Option<i8>
```text

You **must** convert the `Option<T>` to a `T` before you can perform `T` operations on it. This forces you to handle the `None` case explicitly. You can never accidentally assume a value exists when it might be null.

---

## 6.3 The `match` Control Flow Construct

`match` is an extremely powerful control flow operator that allows you to compare a value against a series of patterns and execute code based on which pattern matches. Think of it like a `switch` statement in C-like languages, but fully turbocharged.

```fusion
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}
```text

### 6.3.1 Patterns that Bind to Values

Match arms can bind parts of the values that verify the pattern. This is how we extract data from enum variants.

Example with `Option<i32>`:

```fusion
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}

let five = Some(5)
let six = plus_one(five)
let none = plus_one(None)
```text

In the `Some(i)` arm, `i` binds to the value contained in `Some`.

### 6.3.2 Matches Are Exhaustive

In Fusion, matches are **exhaustive**. You must handle every possible case.

```fusion
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        Some(i) => Some(i + 1),
    }
}
```text

**Compilation Error**: `pattern 'None' not covered`.

This protects you from forgetting to handle edge cases.

### 6.3.3 Catch-all Patterns and the `_` Placeholder

If you want to handle specific values and ignore all others, use `_`:

```fusion
let u8_value = 0u8
match u8_value {
    1 => println("one"),
    3 => println("three"),
    5 => println("five"),
    _ => (), // Do nothing for all measure values
}
```text

---

## 6.4 Concise Control Flow with `if let`

The `match` syntax can be verbose if you only care about *one* specific case. `if let` is syntax sugar for a `match` that runs code only if the value matches one pattern.

Using `match`:

```fusion
let config_max = Some(3u8)
match config_max {
    Some(max) => println!("The maximum is configured to be {}", max),
    _ => (),
}
```text

Using `if let`:

```fusion
let config_max = Some(3u8)
if let Some(max) = config_max {
    println!("The maximum is configured to be {}", max)
}
```text

You can also use `else`:

```fusion
// Imagine coin is Coin::Quarter(UsState::Alaska)
if let Coin::Quarter(state) = coin {
    println!("State quarter from {:?}!", state)
} else {
    count += 1
}
```text

---

## 6.5 Summary

Enums allow you to create custom types that can be one of a set of distinct variants.
- Fusion's enums are algebraic data types: variants can hold diverse data.
- The `Option<T>` enum prevents null-pointer errors by forcing explicit handling of missing values.
- `match` allows safe, exhaustive pattern matching.
- `if let` provides a concise way to handle single patterns.

Enums and structs are the building blocks of data in Fusion. Next, we'll see how to organize this data (and your code) using Fusion's Module System.

---

## 6.6 Exercises

1. **Calculator Enum**: Define an enum `Operation` with variants `Add`, `Subtract`, `Multiply`, `Divide`. Each should hold two `f64` values.
2. **Evaluate Function**: Write a function `evaluate(op: Operation) -> f64` that performs the math.
3. **Option Handling**: Write a function that takes `Option<String>` and returns the string length as `Option<usize>`. It should return `None` if the input is `None`.

---

[Next: Chapter 7 - Packages, Crates, and Modules →](./chapter-07-modules.md)