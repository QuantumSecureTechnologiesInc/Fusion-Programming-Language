> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 5: Using Classes and Structs

In Chapter 3, we discussed primitive types and simple compound types like tuples and arrays. While useful, they can only express simple data relationships. When building complex applications, you need to model real-world concepts with custom data structures.

Fusion provides **structs** (and their alias **classes**) as the primary way to create custom data types. If you're coming from an object-oriented language, a `struct` is similar to a class's data attributes. If you're coming from C, it's like a `struct`.

In this chapter, we will cover:
- Defining and instantiating structs
- Tuple structs and unit-like structs
- Defining methods and associated functions
- The relationship between `struct` and `class` keywords in Fusion

---

## 5.1 Defining and Instantiating Structs

A struct allows you to name and package together multiple related values that make up a meaningful group. Each part of a struct is called a **field**.

### 5.1.1 Defining a Struct

We define a struct using the `struct` keyword (or `class`—they are identical in Fusion, but `struct` is idiomatic for data-only types). Inside the curly braces, we define the names and types of the fields.

```fusion
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}
```text

This definition creates a blueprint. It tells the compiler what a `User` looks like, but doesn't create any actual data yet.

### 5.1.2 Instantiating a Struct

To use a struct, we create an **instance** of it by stating the name and then adding curly braces containing key: value pairs.

```fusion
fn main() {
    let user1 = User {
        email: String::from("someone@example.com"),
        username: String::from("someusername123"),
        active: true,
        sign_in_count: 1,
    }
}
```text

The order of fields doesn't matter.

### 5.1.3 Accessing Fields

We use **dot notation** to read specific values from a struct instance.

```fusion
    println!("User email: {}", user1.email)
```text

If the instance is mutable, we can change a value by using the dot notation and assigning into a particular field.

```fusion
fn main() {
    let mut user1 = User {
        email: String::from("someone@example.com"),
        username: String::from("someusername123"),
        active: true,
        sign_in_count: 1,
    }

    user1.email = String::from("another@example.com")
}
```text

**Note**: The entire instance must be mutable. Fusion doesn't allow you to mark only certain fields as mutable.

### 5.1.4 Field Init Shorthand

When variables have the same names as the fields, usage can be repetitive:

```fusion
fn build_user(email: String, username: String) -> User {
    User {
        email: email,       // Repetitive
        username: username, // Repetitive
        active: true,
        sign_in_count: 1,
    }
}
```text

Fusion provides the **field init shorthand** syntax:

```fusion
fn build_user(email: String, username: String) -> User {
    User {
        email,    // Much cleaner!
        username,
        active: true,
        sign_in_count: 1,
    }
}
```text

### 5.1.5 Struct Update Syntax

Often, you want to create a new instance of a struct that includes most of the values from another instance, but changes some. You can use **struct update syntax**:

```fusion
fn main() {
    let user1 = build_user(String::from("a@b.com"), String::from("user1"))

    let user2 = User {
        email: String::from("another@example.com"),
        ..user1 // "Fill the rest of the fields from user1"
    }
}
```text

Note that the struct update syntax *moves* data. `user1.username` is moved into `user2`. If `user1` contained fields implementing `Copy` (like `bool` or `u64`), those would be copied. But since `String` is not `Copy`, `user1` can no longer be used as a whole after this operation (though `user1.email` is still valid).

---

## 5.2 Tuple Structs and Unit-Like Structs

### 5.2.1 Tuple Structs

You can define structs that look like tuples, called **tuple structs**. They have the added meaning of the struct name but don't name their fields.

```fusion
struct Color(i32, i32, i32)
struct Point(i32, i32, i32)

fn main() {
    let black = Color(0, 0, 0)
    let origin = Point(0, 0, 0)

    // Access by index
    let r = black.0
}
```text

Even though `black` and `origin` are made of the same types (`i32` x 3), they are **different types**. A function taking a `Color` parameter cannot accept a `Point`.

### 5.2.2 Unit-Like Structs

You can define structs without any fields! These are called **unit-like structs** because they behave similarly to `()`, the unit type. They are useful when you need to implement a trait on some type but don't have any data to store.

```fusion
struct TypeCheckPass
```text

---

## 5.3 Methods

Methods are functions that are defined within the context of a struct (or enum or trait object). Their first parameter is always `self`, which represents the instance of the struct the method is being called on.

### 5.3.1 Defining Methods

Let's define a `Rectangle` struct and methods for it.

```fusion
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}
```text

The `impl` (implementation) block tells the compiler "these functions belong to Rectangle".

- **`&self`**: This is short for `self: &Self`. It means the method borrows the instance immutably.
- **`&mut self`**: Use this if the method needs to change the instance.
- **`self`**: Takes ownership of the instance (consumes it). Used for transforming something into something else and preventing the original from being used.

### 5.3.2 Calling Methods

```fusion
fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    let rect2 = Rectangle { width: 10, height: 40 };

    println!("The area is {}", rect1.area())

    if rect1.can_hold(&rect2) {
        println!("Rect1 can hold Rect2")
    }
}
```text

Fusion simplifies method calling with **automatic referencing and dereferencing**. You don't need to write `(&rect1).area()`. Fusion automatically adds `&`, `&mut`, or `*` so that `object` matches the signature of the method.

---

## 5.4 Associated Functions

All functions defined within an `impl` block are called **associated functions**. However, not all of them are methods. If a function does **not** take `self` as a first parameter, it acts like a static method in other languages.

These are often used for **constructors**.

```fusion
impl Rectangle {
    fn new(width: u32, height: u32) -> Rectangle {
        Rectangle { width, height }
    }

    fn square(size: u32) -> Rectangle {
        Rectangle { width: size, height: size }
    }
}
```text

To call this, we use the `::` syntax with the struct name:

```fusion
let sq = Rectangle::square(3)
```text

---

## 5.5 `struct` vs `class`

Fusion supports both keywords:
- `struct`
- `class`

In Fusion, **they appear identical**. Both define custom data types with fields and methods.
- Typically, use `struct` for "Plain Old Data" (POD) types, simple data holders, or when memory layout is important.
- Use `class` when modelling higher-level objects, especially those utilizing inheritance-like patterns (via Traits) or large subsystems, just to signal intent to the reader.

Wait, does Fusion have inheritance? No. Fusion uses **Traits** (Chapter 10) for polymorphism, similar to Rust and Haskell, rather than class-based inheritance like Java or C++. The `class` keyword is strictly syntactic sugar alias for `struct` to make Python/C++ developers feel at home, but under the hood, they behave exactly the same in Fusion v1.0.

---

## 5.6 Summary

- **Structs** let you create custom types meaningful to your domain.
- **Instance fields** hold the specific data.
- **Methods** define behavior associated with that data.
- **Associated functions** let you namespace functionality (like constructors) under types.

Structs are the backbone of data modeling in Fusion. In the next chapter, we'll look at **Enums**, which let you define a type by enumerating its possible variants—another powerful tool for your type-system toolbox.

---

## 5.7 Exercises

1. **Person Struct**: Define a `Person` struct with `name`, `age`, and `address`. Add a method `is_adult(&self)` returning true if age >= 18.
2. **Constructor**: Implement a `new` associated function for `Person`.
3. **Tuple Structs**: Create a `Vector3` tuple struct (x, y, z) and implement a method to calculate its magnitude.

---

[Next: Chapter 6 - Enums and Pattern Matching →](./chapter-06-enums.md)