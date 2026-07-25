> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 14: Smart Pointers

A **pointer** is a general concept for a variable that contains an address in memory.
**Smart pointers**, on the other hand, are data structures that act like a pointer but also have additional metadata and capabilities.

In Fusion, smart pointers often own the data they point to (unlike references, which only borrow data). String and Vec are technically smart pointers!

In this chapter, we'll look at:
- `Box<T>`: for allocating values on the heap.
- `Rc<T>`: a reference counting type that enables multiple ownership.
- `Ref<T>` and `RefMut<T>`: accessed through `RefCell<T>`, a type that enforces the borrowing rules at runtime instead of compile time (interior mutability).
- The `Deref` and `Drop` traits: the secret sauce behind smart pointers.

---

## 14.1 Using `Box<T>` to Point to Data on the Heap

The most straightforward smart pointer is a **box**. Boxes allow you to store data on the heap rather than the stack. What remains on the stack is the pointer to the heap data.

### 14.1.1 Using a Box

```fusion
fn main() {
    let b = Box::new(5)
    println!("b = {}", b)
}
```text

When `b` goes out of scope, the box is deallocated (both the pointer on the stack and the data on the heap).

### 14.1.2 Recursive Types with Boxes

A recursive type is a type that contains itself. Because Fusion needs to know how much space a type takes up at compile time, you can't define infinite recursion directly.

**Cons List (Lisp-style list)**:

```fusion
enum List {
    Cons(i32, List),
    Nil,
}
// Error: recursive type `List` has infinite size
```text

To fix this, we insert a `Box` (which has a known, fixed size: the size of a pointer).

```fusion
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use List::{Cons, Nil}

fn main() {
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))))
}
```text

---

## 14.2 Treating Smart Pointers Like Regular References with `Deref`

Implementing the `Deref` trait allows you to customize the behavior of the *dereference operator*, `*`.

```fusion
use std::ops::Deref

struct MyBox<T>(T)

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn main() {
    let x = 5
    let y = MyBox::new(x)

    assert_eq!(5, x)
    assert_eq!(5, *y) // Calls *(y.deref())
}
```text

### 14.2.1 Deref Coercion

Fusion performs **deref coercion**: it converts a reference to a type that implements `Deref` into a reference to the underlying type.

```fusion
fn hello(name: &str) {
    println!("Hello, {}!", name)
}

fn main() {
    let m = MyBox::new(String::from("Fusion"))

    // MyBox<String> -> String -> &str
    hello(&m)
}
```text

This makes smart pointers ergonomic to use.

---

## 14.3 Running Code on Cleanup with the `Drop` Trait

The `Drop` trait lets you customize what happens when a value is about to go out of scope.

```fusion
struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data)
    }
}

fn main() {
    let c = CustomSmartPointer { data: String::from("my stuff") }
    let d = CustomSmartPointer { data: String::from("other stuff") }
    println!("CustomSmartPointers created.")
}
```text

Output:

```text
CustomSmartPointers created.
Dropping CustomSmartPointer with data `other stuff`!
Dropping CustomSmartPointer with data `my stuff`!
```text

Note variables are dropped in reverse order of creation.

---

## 14.4 `Rc<T>`, the Reference Counted Smart Pointer

In the majority of cases, ownership is clear. But in graphs or usage lists, a value might have multiple owners. `Rc<T>` enables this.

Note: `Rc<T>` is for **single-threaded** scenarios only.

```fusion
use std::rc::Rc

enum List {
    Cons(i32, Rc<List>),
    Nil,
}

use List::{Cons, Nil}

fn main() {
    let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))))

    // b shares ownership of a
    // clone() here increases reference count, doesn't deep copy data
    let b = Cons(3, Rc::clone(&a))

    // c also shares ownership of a
    let c = Cons(4, Rc::clone(&a))
}
```text

The underlying data won't be dropped until the last `Rc` pointing to it is dropped.

---

## 14.5 `RefCell<T>` and the Interior Mutability Pattern

**Interior mutability** is a design pattern that allows you to mutate data even when there are immutable references to that data.

`RefCell<T>` enforces borrowing rules at **runtime** (panicking if violated) rather than compile time.

### 14.5.1 When to use RefCell

Usage scenario: You have a value inside an immutable structure, but you need to modify it for implementation details (e.g., caching, mocking).

```fusion
use std::cell::RefCell

trait Messenger {
    fn send(&self, msg: &str)
}

struct MockMessenger {
    // sent_messages is immutable from the outside, but mutable inside
    sent_messages: RefCell<Vec<String>>,
}

impl Messenger for MockMessenger {
    fn send(&self, msg: &str) {
        // .borrow_mut() gets a mutable reference
        self.sent_messages.borrow_mut().push(String::from(msg))
    }
}
```text

### 14.5.2 Runtime Borrow Checking

- `borrow()` returns `Ref<T>` (smart pointer).
- `borrow_mut()` returns `RefMut<T>`.

`RefCell` tracks how many `Ref` and `RefMut` are active. If you try to create two `RefMut`s at the same time, the program will panic.

---

## 14.6 Reference Cycles Can Leak Memory

It is possible to create memory leaks in Fusion if you use `Rc<T>` and `RefCell<T>` to create a reference cycle (A points to B, B points to A). Both reference counts will always be > 0.

To solve this, use `Weak<T>`. A **weak reference** does not express an ownership relationship (`strong_count` vs `weak_count`).

```fusion
use std::rc::{Rc, Weak}
use std::cell::RefCell

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}
```text

---

## 14.7 Summary

Smart pointers are a powerful tool in Fusion:
- `Box<T>`: Allocate on heap, single owner.
- `Rc<T>`: Multiple owners (ref counting).
- `RefCell<T>`: Interior mutability (runtime borrow checking).
- `Weak<T>`: Non-owning reference (break cycles).

This chapter completes the "core" Fusion v2.0 Vortex language education. The next chapters will dive into concurrency and advanced application development.

---

## 14.8 Exercises

1. **Deref**: Implement a `MyBox` pointer and verify `*` works.
2. **Tree**: Build a tree structure where nodes have children (Vec of Rc) and pointers back to parents (Weak).
3. **Mocking**: Use `RefCell` to create a mock object for testing a logger that counts how many times it was called.

---

[Next: Chapter 15 - Fearless Concurrency →](./chapter-15-concurrency.md)