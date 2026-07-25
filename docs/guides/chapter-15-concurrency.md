> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 15: Fearless Concurrency

Concurrency is the ability for different parts of a program to execute out-of-order or in partial order, without affecting the final outcome. Parallelism is the ability to execute these parts at the same time (e.g., on multi-core processors).

In many languages, writing concurrent code is subtle and error-prone (race conditions, deadlocks). Fusion, however, leverages its ownership and type system to solve many of these problems at **compile time**. We call this **Fearless Concurrency**.

In this chapter, we will cover:
- Creating threads to run code simultaneously.
- Message passing concurrency (sending data between threads).
- Shared-state concurrency (mutexes and locks).
- The `Sync` and `Send` traits: Fusion's concurrency extensions.

---

## 15.1 Using Threads to Run Code Simultaneously

In most modern operating systems, an executed program’s code is run in a **process**, and the operating system manages multiple **threads** within that process.

### 15.1.1 Creating a New Thread

We can create a new thread with `std::thread::spawn`.

```fusion
use std::thread
use std::time::Duration

fn main() {
    thread::spawn(|| {
        for i in 1..10 {
            println("hi number {} from the spawned thread!", i)
            thread::sleep(Duration::from_millis(1))
        }
    })

    for i in 1..5 {
        println("hi number {} from the main thread!", i)
        thread::sleep(Duration::from_millis(1))
    }
}
```text

Note that when the main thread completes, the program exits, even if the spawned thread is still running.

### 15.1.2 Waiting for All Threads to Finish Using `join` Handles

The `spawn` function returns a `JoinHandle`. We can call `join()` on it to wait for the thread to finish.

```fusion
fn main() {
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println("spawned thread: {}", i)
            thread::sleep(Duration::from_millis(1))
        }
    })

    for i in 1..5 {
        println("main thread: {}", i)
        thread::sleep(Duration::from_millis(1))
    }

    handle.join().unwrap()
}
```text

Now the program waits for the spawned thread to complete.

### 15.1.3 Using `move` Closures with Threads

We often want to use data from the main thread in the spawned thread. We must use the `move` keyword to transfer ownership of the data to the thread.

```fusion
fn main() {
    let v = vec![1, 2, 3]

    let handle = thread::spawn(move || {
        println!("Here's a vector: {:?}", v)
    })

    // println!("{:?}", v) // Error! v was moved to the thread.

    handle.join().unwrap()
}
```text

This prevents race conditions where the main thread drops `v` while the spawned thread is still trying to read it.

---

## 15.2 Using Message Passing to Transfer Data Between Threads

One increasingly popular approach to ensuring safe concurrency is **message passing**, where threads or actors communicate by sending each other messages containing data. As the Go motto says: "Do not communicate by sharing memory; instead, share memory by communicating."

Fusion implements this via **channels**.

### 15.2.1 Creating a Channel

A channel has two halves: a transmitter (`tx`) and a receiver (`rx`).

```fusion
use std::sync::mpsc // "multiple producer, single consumer"
use std::thread

fn main() {
    let (tx, rx) = mpsc::channel()

    thread::spawn(move || {
        let val = String::from("hi")
        tx.send(val).unwrap()
        // val is moved and cannot be used here anymore
    })

    let received = rx.recv().unwrap()
    println!("Got: {}", received)
}
```text

### 15.2.2 Sending Multiple Values

```fusion
fn main() {
    let (tx, rx) = mpsc::channel()

    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ]

        for val in vals {
            tx.send(val).unwrap()
            thread::sleep(Duration::from_millis(200)) // Simulation work
        }
    })

    // Treat the receiver as an iterator
    for received in rx {
        println!("Got: {}", received)
    }
}
```text

### 15.2.3 Creating Multiple Producers by Cloning the Transmitter

The `mpsc` acronym stands for **Multiple Producer**, Single Consumer. We can clone the transmitter to have multiple threads sending to the same receiver.

```fusion
let (tx, rx) = mpsc::channel()

let tx1 = tx.clone()
thread::spawn(move || {
    // send from tx1
})

thread::spawn(move || {
    // send from tx (original)
})
```text

---

## 15.3 Shared-State Concurrency

What if passed messages isn't enough? What if we want multiple threads to access the *same* memory location?

This is like sharing memory by communicating... in reverse.

### 15.3.1 Using Mutexes to Allow Access to Data from One Thread at a Time

**Mutex** is short for *mutual exclusion*. A mutex allows only one thread to access some data at any given time. To access the data, a thread must first signal that it wants access by asking for the mutex's **lock**.

```fusion
use std::sync::Mutex

fn main() {
    let m = Mutex::new(5)

    {
        let mut num = m.lock().unwrap()
        *num = 6
    } // Lock is automatically released here

    println("m = {:?}", m)
}
```text

`m.lock()` blocks the thread until the lock is acquired. It returns a smart pointer (`MutexGuard`) wrapped in a `LockResult`. The guard implements `Deref` to point to the inner data, and `Drop` to release the lock automatically.

### 15.3.2 Shared Ownership with Multiple Threads (`Arc<T>`)

Standard `Rc<T>` is not safe to share across threads. For multithreaded reference counting, we use **`Arc<T>`** (Atomic Reference Counted).

```fusion
use std::sync::{Arc, Mutex}
use std::thread

fn main() {
    let counter = Arc::new(Mutex::new(0))
    let mut handles = vec![]

    for _ in 0..10 {
        let counter = Arc::clone(&counter)
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap()

            *num += 1
        })
        handles.push(handle)
    }

    for handle in handles {
        handle.join().unwrap()
    }

    println!("Result: {}", *counter.lock().unwrap()) // 10
}
```text

This pattern—`Arc<Mutex<T>>`—is very common in Fusion for shared mutable state.

---

## 15.4 Extensible Concurrency with the `Sync` and `Send` Traits

Fusion's concurrency features are not just magical language keywords; they are largely defined by the standard library and two marker traits: `std::marker::Sync` and `std::marker::Send`.

### 15.4.1 Allowing Transference of Ownership Between Threads with `Send`

The `Send` trait indicates that ownership of values of the type implementing `Send` can be transferred between threads.
- Almost all Fusion types are `Send` (i32, String, etc.).
- `Rc<T>` is **not** `Send` (because checking reference counts isn't atomic).
- Raw pointers are not `Send`.

### 15.4.2 Allowing Access from Multiple Threads with `Sync`

The `Sync` trait indicates that it is safe for the type implementing `Sync` to be referenced from multiple threads.
- Generically, `T` is `Sync` if `&T` (an immutable reference to `T`) is `Send`.
- `Mutex<T>` is `Sync`.
- `RefCell<T>` is **not** `Sync`.

### 15.4.3 Implementing Send and Sync Manually Is Unsafe

Because `Send` and `Sync` are marker traits that inform the compiler about concurrency guarantees, implementing them manually involves "unsafe" code (checking invariants that the compiler can't verify). In 99.9% of cases, you should rely on the automatic derivation provided by Fusion when your struct is composed of Send/Sync parts.

---

## 15.5 Summary

Fusion provides high-level tools for concurrency that avoid common pitfalls.
- **Threads** for parallelism.
- **Channels** (`mpsc`) for message passing.
- **Mutexes** and **Arc** for shared state.
- **Type System** (`Send`/`Sync`) to prevent data races at compile time.

You can write multithreaded code fearlessly: the compiler will catch race conditions before they ever run.

In the next chapter, we will see one of the unique features of Fusion's tri-brid nature: **Tensor Types and AI/ML** (Chapter 16).

---

## 15.6 Exercises

1. **Parallel Map**: implement a function `pmap(vec, closure)` that applies the closure to each element of the vector using threads, creating a new vector in parallel.
2. **Deadlock Creation**: Write a program that intentionally deadlocks using two Mutexes, and analyze why it happens.
3. **Chat Server**: A primitive chat server where a main thread listens for messages from multiple client threads (simulated) and broadcasts them.

---

[Next: Chapter 16 - Tensor Types and AI/ML →](./chapter-16-tensors.md)