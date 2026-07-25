> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 10: Generic Types, Traits, and Lifetimes

Every programming language has tools for handling the duplication of concepts. In Fusion, one such tool is **generics**: abstract stand-ins for concrete types or other properties.

In this chapter, we're going to dive into three advanced but essential features that work together:
1. **Generics**: Define functions/structs that can operate on many types.
2. **Traits**: Define behavior shared by different types (similar to interfaces).
3. **Lifetimes**: Ensure that references verify scoping rules (validating that borrowed data is still valid).

These three features are the "heart" of Fusion's compile-time safety and expressiveness system.

---

## 10.1 Generic Data Types

Generics allow us to reuse code.

Consider a function that finds the largest number in a list of `i32`:

```fusion
fn largest_i32(list: &[i32]) -> i32 {
    let mut largest = list[0]
    for &item in list {
        if item > largest {
            largest = item
        }
    }
    largest
}
```text

If we wanted to find the largest `char`, we'd have to duplicate this function. Instead, we can use a **generic type parameter**.

### 10.1.1 Functions with Generics

```fusion
fn largest<T>(list: &[T]) -> T {
    let mut largest = list[0]
    for &item in list {
        if item > largest { // Error!
            largest = item
        }
    }
    largest
}
```text

The syntax `<T>` declares a generic type parameter named `T`. We read this as: "the function `largest` is generic over type `T`".

However, the code above won't compile yet. The `>` operator isn't defined for *every* possible type `T`. To fix this, we'll need **traits** (Section 10.2).

### 10.1.2 Struct Definitions with Generics

We can define structs to hold values of any type.

```fusion
struct Point<T> {
    x: T,
    y: T,
}

fn main() {
    let integer = Point { x: 5, y: 10 }
    let float = Point { x: 1.0, y: 4.0 }
    // let mixed = Point { x: 5, y: 4.0 } // Error! Types must match T
}
```text

If we want different types, we can use distinct generic parameters:

```fusion
struct Point<T, U> {
    x: T,
    y: U,
}
```text

### 10.1.3 Enum Definitions with Generics

We’ve already seen this with `Option<T>` and `Result<T, E>`.

```fusion
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```text

### 10.1.4 Performance of Generics

You might wonder if there is a runtime cost to using generics. The answer is **no**.

Fusion performs **monomorphization** at compile time. It looks at all the places where generic code is called and generates specific code for the concrete types used.

If you use `Option<i32>` and `Option<f64>`, Fusion compiles two distinct definitions: `Option_i32` and `Option_f64`. This results in highly optimized machine code with zero runtime overhead (static dispatch).

---

## 10.2 Traits: Defining Shared Behavior

A **trait** defines functionality a particular type has and can share with other types. We can use traits to define shared behavior in an abstract way. Traits are similar to *interfaces* in other languages, with some differences.

### 10.2.1 Defining a Trait

Let's say we have multiple types that hold text: `NewsArticle` and `Tweet`. We want to summarize them.

```fusion
pub trait Summary {
    fn summarize(&self) -> String
}
```text

This trait declares a method signature that implementing types must provide.

### 10.2.2 Implementing a Trait on a Type

```fusion
pub struct NewsArticle {
    pub headline: String,
    pub location: String,
    pub author: String,
    pub content: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}, by {} ({})", self.headline, self.author, self.location)
    }
}

pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub retweet: bool,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}
```text

Now we can call `.summarize()` on instances of `NewsArticle` or `Tweet`.

### 10.2.3 Default Implementations

We can provide a default behavior for some methods.

```fusion
pub trait Summary {
    fn summarize(&self) -> String {
        String::from("(Read more...)")
    }
}
```text

### 10.2.4 Traits as Parameters

Now we can define functions that accept *any* type that implements a trait.

```fusion
pub fn notify(item: &impl Summary) {
    println("Breaking news! {}", item.summarize())
}
```text

This is syntax sugar for **Trait Bounds**:

```fusion
pub fn notify<T: Summary>(item: &T) {
    println("Breaking news! {}", item.summarize())
}
```text

The syntax `T: Summary` means "any type T that implements the Summary trait".

### 10.2.5 Multiple Trait Bounds

We can specify more than one trait bound using the `+` syntax.

```fusion
pub fn notify<T: Summary + Display>(item: &T) { ... }
```text

### 10.2.6 Where Clauses

For complex bounds, use a `where` clause:

```fusion
fn some_function<T, U>(t: &T, u: &U) -> i32
where
    T: Display + Clone,
    U: Clone + Debug,
{ ... }
```text

### 10.2.7 Fixing the `largest` Function

Now we can fix the `largest` function from section 10.1.1. We need to restrict `T` to types that can be compared. The standard library provides the `PartialOrd` trait for this. We also need `Copy` to move values out of the slice (because `largest = item` moves/copies).

```fusion
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut largest = list[0]
    for &item in list {
        if item > largest {
            largest = item
        }
    }
    largest
}
```text

---

## 10.3 Validating References with Lifetimes

Lifetimes are the final piece of the generics puzzle. They are distinct from the other two, but they use similar syntax.

**Lifetimes ensure that references are valid as long as we need them to be.**

Usually, Fusion infers lifetimes implicitly (lifetime elision). But when references interact in complex ways, Fusion needs our help to ensure memory safety.

### 10.3.1 Dangling References

The main aim of lifetimes is to prevent **dangling references**: pointers that reference data that has been cleaned up.

```fusion
fn main() {
    let r

    {
        let x = 5
        r = &x
    } // x is dropped here

    println("r: {}", r) // Error! r refers to invalid memory
}
```text

Fusion catches this at compile time using the **borrow checker**.

### 10.3.2 Generic Lifetimes in Functions

Let's look at this function which returns the longer of two string slices:

```fusion
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```text

This fails to compile!
`error: missing lifetime specifier`.

Fusion doesn't know whether the returned reference refers to `x` or `y`. It doesn't know how long `x` or `y` will live.

We annotate lifetimes using apostrophes: `'a`.

```fusion
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```text

**Interpretation**: "For some lifetime `'a`, the function takes two parameters, both of which live at least as long as `'a`, and returns a reference that also lives at least as long as `'a`."

Practically, `'a` becomes the *intersection* (overlap) (specifically the smaller) of the lifetimes of `x` and `y`.

### 10.3.3 Lifetime Syntax

- `&i32`        // a reference
- `&'a i32`     // a reference with an explicit lifetime
- `&'a mut i32` // a mutable reference with an explicit lifetime

### 10.3.4 Struct Definitions with Lifetimes

If a struct holds a reference, we **must** explicitly annotate the lifetime.

```fusion
struct ImportantExcerpt<'a> {
    part: &'a str,
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...")
    let first_sentence = novel.split('.').next().expect("Could not find a '.'")

    let i = ImportantExcerpt {
        part: first_sentence,
    }
}
```text

This guarantees `ImportantExcerpt` cannot outlive the reference it holds (`part`).

### 10.3.5 The Static Lifetime

One special lifetime is `'static`. It means the reference *can* live for the entire duration of the program. All string literals have the `'static` lifetime.

```fusion
let s: &'static str = "I have a static lifetime."
```text

### 10.3.6 Putting It All Together

Let's combine generics, trait bounds, and lifetimes in one function!

```fusion
use std::fmt::Display

fn longest_with_an_announcement<'a, T>(
    x: &'a str,
    y: &'a str,
    ann: T,
) -> &'a str
where
    T: Display,
{
    println("Announcement! {}", ann)
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```text

---

## 10.4 Summary

This was a dense chapter! We covered:
- **Generics**: Abstracting over types to write reusable code (`<T>`).
- **Traits**: Defining shared behavior and constraining generics (`T: Display`).
- **Lifetimes**: Making explicit guarantees about reference validity (`'a`).

These tools allow you to write code that is flexible, performant (zero-cost abstractions), and guaranteed safe from memory bugs.

In the next chapter, we'll learn how to write **automated tests** to ensure your logic is correct.

---

## 10.5 Exercises

1. **Generic Struct**: Create a `Pair<T>` struct with `x` and `y` fields. Implement a method `new(x: T, y: T) -> Pair<T>`.
2. **Trait Implementation**: Create a struct `Circle` and a struct `Square`. Define a trait `Area` with a method `area(&self) -> f64`. Implement the trait for both structs.
3. **Refactoring**: Take the `largest` function we wrote and refactor it to work on a slice of `&T` references instead of `Copy` types, handling strict lifetime requirements.

---

[Next: Chapter 11 - Writing Automated Tests →](./chapter-11-testing.md)