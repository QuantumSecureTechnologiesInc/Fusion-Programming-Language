> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 7: Packages, Crates, and Modules

As your code ecosystem grows, organizing it becomes critical. You need to split code into multiple files, group related functionality, and control visibility (what is public vs. private).

Fusion provides a powerful module system to handle this:

- **Packages**: A feature of Fusion's package manager (Cargo equivalent) that lets you build, test, and share crates.
- **Crates**: A tree of modules that produces a library or executable.
- **Modules** and **use**: Let you control the organization, scope, and privacy of paths.
- **Paths**: A way of naming an item, such as a struct, function, or module.

This chapter maps out these features so you can structure your projects effectively.

---

## 7.1 Packages and Crates

### 7.1.1 Crates

A **crate** is the smallest unit of code that the Fusion compiler considers at a time.
- **Binary Crate**: compiled to an executable. Must have a `main` function.
- **Library Crate**: doesn't have a `main` function (it defines functionality for other projects to share).

A crate can contain modules, which may be defined in other files that get compiled with the crate.

### 7.1.2 Packages

A **package** is a bundle of one or more crates that provides a set of functionality. A package contains a `fusion.toml` file that describes how to build those crates.

A package can contain:
- Use `src/main.fu` to define a binary crate.
- Use `src/lib.fu` to define a library crate.
- Or both (a library and a binary tool that uses it).

---

## 7.2 Defining Modules to Control Scope

**Modules** let us organize code within a crate into groups for readability and easy reuse. Modules also control the **privacy** of items—whether an item can be used by outside code (*public*) or is an internal implementation detail (*private*).

### 7.2.1 A Module Example

Let's write a library crate that provides functionality for a restaurant. We'll define the signatures of functions but leave their bodies empty for now.

Open `src/lib.fu` and create a `front_of_house` module:

```fusion
mod front_of_house {
    mod hosting {
        fn add_to_waitlist() {}
        fn seat_at_table() {}
    }

    mod serving {
        fn take_order() {}
        fn serve_order() {}
        fn take_payment() {}
    }
}
```text

We define a module with the `mod` keyword. Modules can be nested inside other modules.

### 7.2.2 The Module Tree

The structure of our crate looks like this:

```text
crate
 └── front_of_house
     ├── hosting
     │   ├── add_to_waitlist
     │   └── seat_at_table
     └── serving
         ├── take_order
         ├── serve_order
         └── take_payment
```text

This tree shows how some modules nest inside one another (like `hosting` nests inside `front_of_house`). The tree commands scope: you can't just call `add_to_waitlist` from anywhere.

---

## 7.3 Paths for Referring to Items

To show Fusion where to find an item in a module tree, we use a **path**.

- **Absolute path**: starts from a crate root by using a literal crate name or a literal `crate`.
- **Relative path**: starts from the current module and uses `self`, `super`, or an identifier in the current module.

Paths are separated by `::`.

```fusion
pub fn eat_at_restaurant() {
    // Absolute path
    crate::front_of_house::hosting::add_to_waitlist()

    // Relative path
    front_of_house::hosting::add_to_waitlist()
}
```text

### 7.3.1 Private by Default

If you try to compile the code above, it will fail!
`error: module 'hosting' is private`.

In Fusion, **all items (functions, methods, structs, enums, modules, and constants) are private by default**.
- Items in a parent module cannot use private items inside child modules.
- But items in child modules can use items in their ancestor modules.

To make an item accessible, we use the `pub` keyword.

```fusion
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

pub fn eat_at_restaurant() {
    // This now works!
    crate::front_of_house::hosting::add_to_waitlist()
}
```text

### 7.3.2 Starting Relative Paths with `super`

We can construct relative paths that begin in the parent module by using `super`. This is like `..` in a filesystem.

```fusion
fn deliver_order() {}

mod back_of_house {
    fn fix_incorrect_order() {
        cook_order()
        super::deliver_order() // Go up one level to find deliver_order
    }

    fn cook_order() {}
}
```text

### 7.3.3 Making Structs and Enums Public

If we use `pub` before a struct definition, we make the struct public. However, **the struct's fields will still be private**. We can make each field public or not on a case-by-case basis.

```fusion
mod back_of_house {
    pub struct Breakfast {
        pub toast: String,      // Public field
        seasonal_fruit: String, // Private field
    }

    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("peaches"),
            }
        }
    }
}
```text

Because `seasonal_fruit` is private, users cannot construct a `Breakfast` directly (they can't set that field). They strictly *must* use the `Breakfast::summer` constructor.

In contrast, if we make an **enum** public, **all of its variants become public**.

```fusion
pub enum Appetizer {
    Soup,
    Salad,
}
```text

---

## 7.4 Bringing Paths into Scope with `use`

Typing `crate::front_of_house::hosting::add_to_waitlist` every time is tedious. We can bring a path into scope once and then call the items as if they’re local items with the `use` keyword.

```fusion
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist()
    hosting::add_to_waitlist()
}
```text

### 7.4.1 Idiomatic `use` Paths

- **Functions**: It's idiomatic to bring the *parent module* into scope (`use crate::front_of_house::hosting`) rather than the function itself. This makes it clear that the function isn't locally defined.
- **Structs/Enums**: It's idiomatic to bring the item itself into scope.

```fusion
use std::collections::HashMap

fn main() {
    let mut map = HashMap::new()
}
```text

### 7.4.2 Renaming with `as`

If we bring two items with the same name into scope, there's a conflict. We can fix this with `as`.

```fusion
use std::fmt
use std::io

fn function1() -> fmt::Result { ... }
fn function2() -> io::Result<()> { ... }

// OR

use std::fmt::Result
use std::io::Result as IoResult

fn function1() -> Result { ... }
fn function2() -> IoResult<()> { ... }
```text

### 7.4.3 Re-exporting Names with `pub use`

When you bring a name into scope with `use`, the name available in the new scope is private. To enable code that calls your code to refer to that name as if it had been defined in that code’s scope, you can combine `pub` and `use`. This technique is called **re-exporting**.

```fusion
pub use crate::front_of_house::hosting
```text

---

## 7.5 Separating Modules into Different Files

As modules get large, you might want to move their definitions to separate files to make the code easier to navigate.

File `src/lib.fu`:

```fusion
mod front_of_house

pub use crate::front_of_house::hosting

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist()
}
```text

File `src/front_of_house.fu`:

```fusion
pub mod hosting
```text

File `src/front_of_house/hosting.fu`:

```fusion
pub fn add_to_waitlist() {}
```text

Fusion looks for module code in:
1. Inline (`mod foo { ... }`)
2. In `src/foo.fu`
3. In `src/foo/mod.fu` (older style, but supported)

---

## 7.6 Summary

Fusion provides a robust module system to manage complexity.
- **Packages** contain crates.
- **Crates** are trees of modules.
- **Modules** control organization and privacy.
- **Paths** refer to items.
- **`use`** brings paths into scope.
- **`pub`** makes items public (exposing them to other modules).

Mastering this system allows you to write clean, encapsulated, and reusable code.

---

## 7.7 Exercises

1. **Library Creation**: Create a new library named `fusion_math`.
2. **Modules**: Inside, create a module `arithmetic` with functions `add` and `subtract`. Make `add` public but keep `subtract` private.
3. **Nested Modules**: Create a `shapes` module with a sub-module `rectangle`. Define a public struct `Rectangle` inside it.
4. **Integration**: Create a `tests` module and write a test function that uses `super` to call your arithmetic functions.

---

[Next: Chapter 8 - Common Collections →](./chapter-08-collections.md)