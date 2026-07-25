> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 4: Understanding Memory Safety

Memory safety is one of Fusion's defining features. This chapter introduces **ownership**, **borrowing**, and **lifetimes** the mechanisms that eliminate entire categories of bugs at compile time, without garbage collection overhead.

If you've programmed in C or C++, you've likely encountered memory bugs: dangling pointers, double frees, buffer overflows, and data races. These vulnerabilities cost billions of dollars annually and remain the primary entry point for security exploits. If you've used garbage-collected languages like Python or Java, you've sacrificed performance and predictability for safety.

Fusion offers a third path: compile-time memory safety with zero runtime cost. The ownership system guarantees that memory is correctly managed, that references are always valid, and that data races are impossible all verified before your program ever runs.

This chapter is foundational. Master these concepts, and the rest of Fusion becomes significantly easier.

---

## 4.1 The Ownership System

Ownership is Fusion's approach to memory management. Rather than relying on garbage collection (slow, unpredictable) or manual memory management (error-prone), Fusion uses a set of compile-time rules that track which part of your code "owns" each piece of data.

### 4.1.1 What Is Ownership?

Every value in Fusion has exactly one **owner** a variable that holds the value and is responsible for cleaning it up. When the owner goes out of scope, the value is automatically deallocated.

Think of ownership like physical possession: if you own a book, you're responsible for it. You can lend it to someone, but ultimately you decide when to dispose of it. And importantly, the book can only be in one place at a time.

### 4.1.2 The Three Rules of Ownership

Fusion's ownership system follows three simple rules:

1. **Each value has exactly one owner at any given time**
2. **When the owner goes out of scope, the value is dropped (deallocated)**
3. **Ownership can be transferred (moved), but not implicitly copied**

Let's see these rules in action:

```fusion
fn main() {
    let s1 = String::from("hello")  // s1 owns the string
    let s2 = s1                      // Ownership moves to s2
    // println("{}", s1)  // Error! s1 no longer owns the string
    println("{}", s2)     // Works: s2 is the owner

}  // s2 goes out of scope, string is deallocated

```text

When we assign `s1` to `s2`, we don't copy the string we *move* ownership. After the move, `s1` is no longer valid. This prevents the dangerous situation of two variables pointing to the same memory, where freeing one would leave the other dangling.

### 4.1.3 Why This Matters: Preventing Double-Free

Consider this C code:

```c
char* s1 = malloc(100);
strcpy(s1, "hello");
char* s2 = s1;  // Both point to same memory
free(s1);       // Memory freed
free(s2);       // DOUBLE FREE - undefined behaviour!

```text

This is undefined behaviour that can crash your program, corrupt memory, or create security vulnerabilities. In Fusion, this scenario is impossible. The type system prevents `s2` from being used after `s1` is freed because the move makes the relationship explicit.

### 4.1.4 The `Copy` Trait: When Moving Doesn't Happen

For simple types that live entirely on the stack (integers, floats, booleans, characters), copying is cheap and safe. These types implement the `Copy` trait, which means assignment creates a copy rather than a move:

```fusion
fn main() {
    let x = 5     // x owns the value 5
    let y = x     // y gets a COPY; x is still valid
    println("{}", x)  // Works!
    println("{}", y)  // Also works!

}

```text

Types that implement `Copy`:

- All integer types (`i8`, `i16`, `i32`, `i64`, `i128`, `u8`, etc.)
- All floating-point types (`f32`, `f64`)
- Booleans (`bool`)
- Characters (`char`)
- Tuples containing only `Copy` types
- Fixed-size arrays of `Copy` types

Types that do **not** implement `Copy` (and therefore move):

- `String`
- `Vec<T>`
- `HashMap<K, V>`
- Any type that owns heap-allocated data
- Any type with a custom `Drop` implementation

### 4.1.5 Scope and Cleanup

In Fusion, values are automatically cleaned up when they go out of scope. There's no need to call `free()` or `delete` manually.

```fusion
fn main() {
    {
        let s = String::from("hello")
        // s is valid from here...
        println("{}", s)
    }  // ...until here. s goes out of scope; memory is freed.
    // println("{}", s)  // Error! s no longer exists

}

```text

This is called **RAII** (Resource Acquisition Is Initialization)—a design pattern where resources are tied to variable lifetime. Files close automatically. Network connections terminate. Locks release. No forgetting, no leaks.

### 4.1.6 Ownership and Functions

Passing a value to a function transfers ownership just like assignment:

```fusion
fn take_ownership(s: String) {
    println("I now own: {}", s)
}  // s is dropped here

fn main() {
    let my_string = String::from("hello")
    take_ownership(my_string)      // Ownership moves to the function
    // println("{}", my_string)    // Error! my_string is no longer valid
}

```text

If you want to use a value after passing it to a function, you have two options:

1. **Clone** the value (creates a deep copy)
2. **Borrow** the value (next section)

Returning values from functions also transfers ownership:

```fusion
fn create_string() -> String {
    let s = String::from("hello")
    s  // Ownership transfers to the caller
}

fn main() {
    let s = create_string()  // s now owns the string
    println("{}", s)
}

```text

---

## 4.2 References and Borrowing

Moving ownership every time you want to use a value would be cumbersome. Borrowing lets you use a value without taking ownership like lending a book rather than giving it away.

### 4.2.1 What Is a Reference?

A **reference** is a pointer to a value that doesn't own it. References are created with the `&` operator:

```fusion
fn main() {
    let s1 = String::from("hello")
    let len = calculate_length(&s1)  // Pass a reference
    println("The length of '{}' is {}", s1, len)  // s1 still valid!

}

fn calculate_length(s: &String) -> int {
    s.len()
}  // s goes out of scope, but doesn't drop the String (doesn't own it)

```text

The function `calculate_length` takes `&String` a reference to a String. It can read the string's contents but doesn't own it and won't deallocate it.

### 4.2.2 Immutable References

By default, references are **immutable**. You can read the borrowed value but not modify it:

```fusion
fn main() {
    let s = String::from("hello")
    let r = &s
    // r.push_str(" world")  // Error! Cannot mutate through immutable reference
    println("{}", r)  // Reading is fine

}

```text

You can have **multiple immutable references** to the same value simultaneously:

```fusion
fn main() {
    let s = String::from("hello")
    let r1 = &s
    let r2 = &s
    let r3 = &s

    println("{}, {}, {}", r1, r2, r3)  // All valid

}

```text

This is safe because none of these references can modify the data. Multiple readers cause no conflicts.

### 4.2.3 Mutable References

If you need to modify borrowed data, use a **mutable reference** with `&mut`:

```fusion
fn main() {
    let mut s = String::from("hello")
    change(&mut s)

    println("{}", s)  // "hello, world"

}

fn change(s: &mut String) {
    s.push_str(", world")
}

```text

Critical rule: **You can have only ONE mutable reference to a value at a time**:

```fusion
fn main() {
    let mut s = String::from("hello")
    let r1 = &mut s
    // let r2 = &mut s  // Error! Cannot borrow `s` as mutable more than once

    println("{}", r1)

}

```text

This restriction prevents **data races** at compile time. A data race occurs when:

1. Two or more pointers access the same data simultaneously
2. At least one is writing
3. There's no synchronisation

By allowing only one mutable reference, Fusion guarantees that no two pieces of code can modify the same data simultaneously.

### 4.2.4 The Borrowing Rules

Fusion enforces two fundamental rules at compile time:

1. **At any given time, you may have EITHER**:
   
   - Any number of immutable references (`&T`), OR
   - Exactly one mutable reference (`&mut T`)

2. **References must always be valid** (no dangling pointers)

These rules are checked by the **borrow checker**, a component of the Fusion compiler that analyses reference usage.

```fusion
fn main() {
    let mut s = String::from("hello")
    let r1 = &s     // Immutable borrow
    let r2 = &s     // Another immutable borrow - OK
    // let r3 = &mut s  // Error! Cannot borrow as mutable while immutable borrows exist

    println("{} and {}", r1, r2)
    // r1 and r2 are no longer used after this point

    let r3 = &mut s  // Now OK! Previous borrows have ended
    println("{}", r3)

}

```text

Notice that the borrow checker is smart about *when* references are used, not just when they're declared. This is called **Non-Lexical Lifetimes (NLL)**—borrows end when the reference is last used, not when the variable goes out of scope.

### 4.2.5 Dangling References

A dangling reference points to memory that has been freed. Fusion prevents this:

```fusion
fn dangle() -> &String {
    let s = String::from("hello")
    &s  // Error! `s` will be dropped when this function returns
}       // Returning a reference to freed memory is forbidden

```text

The compiler produces a clear error:

```text
error: cannot return reference to local variable `s`
  --> src/main.fu:3:5
   |
3  |     &s
   |     ^^ returns a reference to data owned by the current function

```text

The solution is to return the owned value, transferring ownership to the caller:

```fusion
fn no_dangle() -> String {
    let s = String::from("hello")
    s  // Ownership transfers; no dangling reference
}

```text

---

## 4.3 The Slice Type

Slices let you reference a contiguous sequence of elements within a collection, without taking ownership of the entire collection.

### 4.3.1 String Slices

A **string slice** (`&str`) is a reference to a portion of a `String`:

```fusion
fn main() {
    let s = String::from("hello world")
    let hello = &s[0..5]   // Slice from index 0 to 5 (exclusive)
    let world = &s[6..11]  // Slice from index 6 to 11

    println("{}", hello)  // "hello"
    println("{}", world)  // "world"

}

```text

Slice syntax:

- `&s[start..end]` - from `start` to `end` (exclusive)
- `&s[..end]` - from beginning to `end`
- `&s[start..]` - from `start` to end
- `&s[..]` - entire string

### 4.3.2 Slices Prevent Bugs

Consider finding the first word in a string:

```fusion
fn first_word(s: &String) -> &str {
    let bytes = s.as_bytes()
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i]
        }
    }

    &s[..]

}

fn main() {
    let mut s = String::from("hello world")
    let word = first_word(&s)
    // s.clear()  // Error! Cannot mutate `s` while `word` references it

    println("First word: {}", word)

}

```text

The borrow checker prevents `s.clear()` because `word` holds an immutable reference to part of `s`. This catches bugs that would otherwise corrupt the slice reference.

### 4.3.3 Array Slices

Slices work on any contiguous memory, including arrays and vectors:

```fusion
fn main() {
    let a = [1, 2, 3, 4, 5]
    let slice = &a[1..3]  // [2, 3]
    println("{:?}", slice)

}

```text

The type of an array slice is `&[T]`, where `T` is the element type.

### 4.3.4 String Literals Are Slices

When you write a string literal, you get a `&str`:

```fusion
let s: &str = "Hello, world!"

```text

The bytes of the string are stored in the read-only section of the binary. `s` is a slice pointing to that location. This is why string literals are immutable and live for the entire program.

---

## 4.4 Lifetimes

Lifetimes are annotations that tell the compiler how long references are valid. They ensure that references never outlive the data they point to.

### 4.4.1 The Problem Lifetimes Solve

Consider this function that returns the longer of two strings:

```fusion
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

```text

This won't compile. The compiler asks: "Does the returned reference come from `x` or `y`? How long is it valid for?" Without knowing this, it can't ensure the reference remains valid.

### 4.4.2 Lifetime Annotation Syntax

Lifetimes are annotated with an apostrophe followed by a short name (conventionally `'a`, `'b`, etc.):

```fusion
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

```text

This signature says: "Given two string slices that both live for at least the lifetime `'a`, return a string slice that also lives for `'a`."

The compiler now knows the returned reference is valid as long as *both* input references are valid.

### 4.4.3 Lifetime Elision

In many cases, the compiler infers lifetimes, so you don't need to write them explicitly. These rules are called **lifetime elision rules**:

1. Each parameter gets its own lifetime
2. If there's exactly one input lifetime, it's assigned to all output lifetimes
3. If one input is `&self` or `&mut self`, that lifetime is assigned to outputs

This is why many functions don't require explicit lifetimes:

```fusion
// Written:
fn first_word(s: &str) -> &str { ... }

// Compiler infers:
fn first_word<'a>(s: &'a str) -> &'a str { ... }

```text

### 4.4.4 Lifetimes in Structs

If a struct holds references, you must annotate lifetimes:

```fusion
struct ImportantExcerpt<'a> {
    part: &'a str
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...")
    let first_sentence = novel.split('.').next().unwrap()
    let excerpt = ImportantExcerpt {
        part: first_sentence
    }

    println("Excerpt: {}", excerpt.part)

}

```text

This annotation means: "An `ImportantExcerpt` instance cannot outlive the reference in its `part` field."

### 4.4.5 The Static Lifetime

One special lifetime is `'static`, meaning the reference lives for the entire program:

```fusion
let s: &'static str = "I live forever!"

```text

String literals have the `'static` lifetime because they're embedded in the binary.

Use `'static` sparingly. Usually, you want references with shorter, more specific lifetimes.

---

## 4.5 Best Practices

Having understood ownership, borrowing, and lifetimes, here are best practices for writing safe, idiomatic Fusion code:

### 4.5.1 Prefer Borrowing Over Moving

If a function only needs to read data, take a reference:

```fusion
// Good: Takes reference, caller keeps ownership
fn print_length(s: &str) {
    println("Length: {}", s.len())
}

// Less ideal: Takes ownership unnecessarily
fn print_length_owned(s: String) {
    println("Length: {}", s.len())
}  // s is dropped here; caller can't use it

```text

### 4.5.2 Use Slices for Flexibility

Accept slices (`&str`, `&[T]`) rather than owned types (`String`, `Vec<T>`) when possible:

```fusion
// Good: Works with both String and &str
fn count_words(s: &str) -> int {
    s.split_whitespace().count()
}

// Less flexible: Requires String
fn count_words_owned(s: String) -> int {
    s.split_whitespace().count()
}

```text

### 4.5.3 Clone Deliberately

Cloning creates a deep copy. It's sometimes necessary, but should be intentional:

```fusion
let s1 = String::from("hello")
let s2 = s1.clone()  // Explicit deep copy

println("{}", s1)  // Works—s1 wasn't moved
println("{}", s2)

```text

Don't clone to "fix" borrow checker errors without understanding why. Often there's a better design.

### 4.5.4 Minimise Mutable State

Immutability is the default for good reason. Mutable state is harder to reason about:

```fusion
// Prefer this:
fn add(a: int, b: int) -> int {
    a + b
}

// Over mutation:
fn add_to(a: &mut int, b: int) {
    *a += b
}

```text

### 4.5.5 Keep Borrows Short

End borrows as soon as possible to avoid conflicts:

```fusion
fn main() {
    let mut data = vec![1, 2, 3]
    // Bad: Long-lived borrow
    let first = &data[0]
    // ... many lines of code ...
    // data.push(4)  // Error! `first` still active

    // Good: Use and release borrow immediately
    println("First: {}", data[0])
    data.push(4)  // Works!

}

```text

---

## 4.6 Common Patterns and Solutions

Here are solutions to common ownership challenges:

### 4.6.1 Returning Multiple Values

Use tuples to return ownership of multiple values:

```fusion
fn split_at(s: String, mid: int) -> (String, String) {
    let first = s[..mid].to_string()
    let second = s[mid..].to_string()
    (first, second)
}

```text

### 4.6.2 Interior Mutability

When you need mutation through a shared reference, use `RefCell`:

```fusion
use std::cell::RefCell

fn main() {
    let data = RefCell::new(5)
    *data.borrow_mut() += 1

    println("{}", data.borrow())  // 6

}

```text

`RefCell` moves borrow checking to runtime. Use sparingly.

### 4.6.3 Shared Ownership with `Rc`

When multiple parts of your code need ownership, use `Rc` (Reference Counted):

```fusion
use std::rc::Rc

fn main() {
    let data = Rc::new(String::from("shared"))
    let a = Rc::clone(&data)
    let b = Rc::clone(&data)

    println("{} {} {}", data, a, b)  // All valid

}  // Memory freed when last Rc is dropped

```text

---

## 4.7 Summary

This chapter covered Fusion's memory safety system:

| Concept                  | Purpose                                           |
|:------------------------ |:------------------------------------------------- |
| **Ownership**            | Every value has exactly one owner                 |
| **Move semantics**       | Assignment transfers ownership                    |
| **Borrowing**            | References let you use values without owning them |
| **Immutable references** | Multiple simultaneous readers allowed             |
| **Mutable references**   | Only one writer at a time                         |
| **Lifetimes**            | Ensure references remain valid                    |
| **Slices**               | References to portions of collections             |

Key takeaways:

1. **No garbage collector, no manual memory management**—the compiler handles it
2. **Data races are prevented at compile time** by the borrowing rules
3. **Dangling pointers are impossible** due to lifetime checking
4. **These guarantees have zero runtime cost**

Understanding ownership is the foundation of Fusion mastery. With these concepts internalised, you'll write safer code faster than in any other systems language.

---

## 4.8 Exercises

1. **Move and Clone**: Create a `String`, move it to a new variable, then try to use the original. Fix the error using `clone()`.

2. **Borrowing Practice**: Write a function that takes a `&String` and returns its length without taking ownership.

3. **Mutable Borrow**: Write a function that appends " world" to a mutable string reference.

4. **Slice Manipulation**: Write a function that takes a string slice and returns the first word (ending at the first space).

5. **Lifetime Annotations**: Write a function `longest_with_announcement` that takes two string slices and a message, prints the message, and returns the longer string.

---

[Next: Chapter 5 - Using Classes and Structs →](./chapter-05-structs.md)
