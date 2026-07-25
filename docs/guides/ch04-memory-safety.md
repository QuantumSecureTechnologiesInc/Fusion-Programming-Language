# Chapter 4: Memory Safety

> Ownership, borrowing, and the Vortex safety engine

---

## Ownership Model (Move Semantics)

Fusion uses an **ownership model** where each value has exactly one owner. When the owner goes out of scope or is reassigned, the value is dropped (freed). Values are **moved** by default — not copied.

### Basic Ownership

```fusion
fn main() -> int {
    let s1: string = "hello";
    let s2: string = s1;  // s1 is moved to s2

    // s1 is no longer valid here
    // println(s1);  // ERROR: use of moved value

    println(s2);  // OK: s2 owns the string
    return 0;
}
```

### Ownership Transfer

```fusion
fn consume(s: string) {
    println("Consumed: %s", s);
    // s is dropped at end of this function
}

fn main() -> int {
    let message: string = "hello";
    consume(message);       // message is moved into consume

    // message is no longer valid
    // println(message);  // ERROR: use of moved value

    return 0;
}
```

### Return and Ownership

```fusion
fn create_greeting(name: string) -> string {
    return "Hello, " + name + "!";
    // name is dropped here, but the new string is returned
}

fn main() -> int {
    let greeting: string = create_greeting("Fusion");
    println(greeting);  // OK: greeting owns the returned string
    return 0;
}
```

---

## Borrowing (&T, &mut T)

Borrowing lets you access a value without taking ownership. There are two kinds:

- **Shared borrow (`&T`)**: Multiple readers allowed simultaneously
- **Exclusive borrow (`&mut T`)**: Single writer, no other access allowed

### Shared Borrows

```fusion
fn calculate_length(s: &string) -> int {
    return s.len();
}

fn main() -> int {
    let name: string = "Fusion";
    let len: int = calculate_length(&name);  // Borrow name

    // name is still valid (only borrowed, not moved)
    println("%s has length %d", name, len);

    // Multiple shared borrows are OK
    let r1: &string = &name;
    let r2: &string = &name;
    println("%s, %s", r1, r2);

    return 0;
}
```

### Exclusive Borrows

```fusion
fn add_world(s: &mut string) {
    *s = *s + " world";
}

fn main() -> int {
    let mut greeting: string = "hello";
    add_world(&mut greeting);  // Mutably borrow
    println(greeting);  // "hello world"

    // Only one mutable borrow at a time
    let r1: &mut int = &mut x;
    // let r2: &mut int = &mut x;  // ERROR: already mutably borrowed
    // println(r1);  // r1 still alive here

    return 0;
}
```

### Borrowing Rules

```fusion
fn main() -> int {
    let mut data: string = "hello";

    // Rule 1: Multiple shared borrows OK
    let r1: &string = &data;
    let r2: &string = &data;
    println("%s %s", r1, r2);

    // Rule 2: Can't have mutable borrow while shared borrows exist
    // let r3: &mut string = &mut data;  // ERROR: data is already borrowed

    // Rule 3: Shared borrows end when last use is done
    println(r1);  // Last use of r1
    println(r2);  // Last use of r2
    // Now we can mutably borrow
    let r3: &mut string = &mut data;
    *r3 = "world";
    println(r3);

    return 0;
}
```

---

## The Vortex Safety Engine

The Vortex safety engine is Fusion's compile-time borrow checker. It uses **entropic flow analysis** to prevent data races, use-after-free, and other memory safety violations.

### How Vortex Works

```
Source Code
    ↓
Lexer → Tokens
    ↓
Parser → AST
    ↓
Semantic Analysis → Typed AST
    ↓
[VORTEX BORROW CHECKER] → Safety Validation
    ↓
IR Lowering → Intermediate Representation
    ↓
Code Generation → Native Binary / WASM
```

### Permission States

Vortex tracks each variable through four permission states:

| State | Description |
|-------|-------------|
| **Intact** | Value is in standard local scope |
| **Shared Borrowed** | Value is immutably borrowed (multiple readers OK) |
| **Exclusive Borrowed** | Value is mutably borrowed (single writer) |
| **Dissipated** | Value has been consumed or moved |

### Collision Detection

When two conflicting access patterns overlap, Vortex detects an **entropic collision**:

```fusion
fn main() -> int {
    let mut data: Vec<int> = vec![1, 2, 3];

    // Shared borrow
    let ref1: &Vec<int> = &data;
    // Exclusive borrow — COLLISION!
    // let ref2: &mut Vec<int> = &mut data;

    // Fix: complete shared borrow before exclusive borrow
    println(ref1[0]);  // Last use of ref1
    let ref2: &mut Vec<int> = &mut data;  // OK now
    ref2.push(4);

    return 0;
}
```

### Vortex Error Messages

When Vortex detects a safety violation, it produces rich, descriptive error messages:

```
Entropic Collision Detected in my_resource
============================================================
Variable flow 'my_resource' suffered a permission stream intersection.

Analysis of Flow Collision:
  * The resource was already: exclusively borrowed by a mutable writer.
  * You attempted to access or borrow it at position 42-60

Remediation Advice:
    Fusion's Vortex Engine strictly forbids conflicting read/write access.
    Wrap the borrow blocks inside explicit scope boundaries using '{ ... }'
    to allow exclusive permission frames to exit before access.
============================================================
```

---

## Affine Types and Linear Types

Fusion's type system is based on **affine types** — values can be used at most once.

### Affine Types (Move Semantics)

```fusion
fn main() -> int {
    let s: string = "hello";

    // String is moved (affine — used once)
    let t: string = s;
    // s is now invalid

    println(t);
    return 0;
}
```

### Linear Types (Must Use Exactly Once)

Some types in Fusion are **linear** — they must be used exactly once:

```fusion
// A file handle that must be closed exactly once
struct File {
    handle: int,
}

impl File {
    fn open(path: string) -> File {
        return File { handle: 0 };  // Simplified
    }

    fn close(self) {
        println("Closing file handle %d", self.handle);
    }
}

fn main() -> int {
    let f: File = File::open("data.txt");
    f.close();  // Must be called exactly once
    // f is consumed by close()

    // Forgetting to close would be a compile error with linear types
    // let f2: File = File::open("data.txt");
    // // ERROR: f2 was never used

    return 0;
}
```

### Resource Patterns

```fusion
// RAII pattern: acquire in constructor, release in destructor
struct Lock {
    mutex: int,
}

impl Lock {
    fn acquire(mutex: int) -> Lock {
        println("Acquiring lock on mutex %d", mutex);
        return Lock { mutex };
    }

    fn release(self) {
        println("Releasing lock on mutex %d", self.mutex);
    }
}

fn main() -> int {
    let lock: Lock = Lock::acquire(1);
    // Do work under the lock
    println("Working under lock");
    lock.release();  // Release when done
    return 0;
}
```

---

## Copy vs Move Types

Fusion distinguishes between types that are copied (duplicated) and types that are moved (transferred).

### Copy Types (Copied on Use)

```fusion
// These types implement Copy semantics:
// - int (and other integer types)
// - bool
// - float
// - Pointers (*T)

fn main() -> int {
    let a: int = 42;
    let b: int = a;  // 'a' is copied, not moved

    // Both are still valid
    println("a=%d, b=%d", a, b);

    let x: bool = true;
    let y: bool = x;  // Copied
    println("x=%d, y=%d", x, y);

    return 0;
}
```

### Move Types (Moved on Use)

```fusion
// These types use move semantics:
// - string
// - Arrays [T; N]
// - Slices [T]
// - Structs (by default)
// - Closures

fn main() -> int {
    let s1: string = "hello";
    let s2: string = s1;  // s1 is moved

    // println(s1);  // ERROR: s1 was moved
    println(s2);  // OK

    let arr1: [int; 3] = [1, 2, 3];
    let arr2: [int; 3] = arr1;  // arr1 is moved

    // println(arr1[0]);  // ERROR: arr1 was moved
    println(arr2[0]);  // OK

    return 0;
}
```

### Explicit Clone

To keep the original value, use `.clone()`:

```fusion
fn main() -> int {
    let s1: string = "hello";
    let s2: string = s1.clone();  // Explicit copy

    // Both are valid
    println("%s %s", s1, s2);

    let arr1: [int; 3] = [1, 2, 3];
    let arr2: [int; 3] = arr1.clone();

    println("%d %d", arr1[0], arr2[0]);

    return 0;
}
```

---

## Memory Safety Guarantees

Fusion's Vortex engine guarantees:

### No Use-After-Free

```fusion
fn main() -> int {
    let s: string = "hello";
    let t: string = s;  // s moved to t
    // println(s);  // COMPILE ERROR: use of moved value
    println(t);
    return 0;
}
```

### No Data Races

```fusion
fn main() -> int {
    let mut data: string = "shared";

    // Can't have mutable and immutable borrows simultaneously
    let r1: &string = &data;
    // let r2: &mut string = &mut data;  // COMPILE ERROR
    println(r1);

    return 0;
}
```

### No Null Pointer Dereferences

Fusion uses `Option<T>` instead of null:

```fusion
fn find_value(key: string) -> Option<int> {
    if key == "exists" {
        return Some(42);
    }
    return None;
}

fn main() -> int {
    let result: Option<int> = find_value("exists");

    match result {
        Some(value) => {
            println("Found: %d", value);
        }
        None => {
            println("Not found");
        }
    }

    // Can't accidentally use None as an int
    // let x: int = result;  // COMPILE ERROR

    return 0;
}
```

### No Buffer Overflows

```fusion
fn main() -> int {
    let arr: [int; 3] = [1, 2, 3];

    // Safe indexing
    let val: int = arr[0];  // OK
    // let bad: int = arr[10];  // COMPILE ERROR or runtime check

    // Safe iteration
    for item in arr {
        println("%d", item);
    }

    return 0;
}
```

---

## Unsafe Blocks

For low-level operations that bypass safety checks, use `@unsafe`:

```fusion
@unsafe
@manual_memory
fn raw_pointer_operation(ptr: *int) -> int {
    // Manual pointer manipulation
    return *ptr;
}

fn main() -> int {
    let x: int = 42;
    let result: int = raw_pointer_operation(&x);
    println("Result: %d", result);
    return 0;
}
```

### When to Use Unsafe

- FFI (Foreign Function Interface) calls
- Hardware-level programming
- Performance-critical code where bounds checks are proven safe
- Interfacing with external libraries

### Safety Annotations

```fusion
@unsafe          // Marks code that bypasses safety checks
@manual_memory   // Declares explicit manual memory lifecycle
@borrowed        // Indicates borrowed reference with lifetime

fn dangerous(ptr: *int) -> int {
    return *ptr;
}
```

---

## Common Patterns and Anti-Patterns

### Good: Scoped Borrows

```fusion
fn main() -> int {
    let mut data: string = "hello";

    // Scoped borrow
    {
        let r: &string = &data;
        println(r);
    }  // r goes out of scope here

    // Now we can mutably borrow
    let r2: &mut string = &mut data;
    *r2 = "world";

    return 0;
}
```

### Bad: Long-lived Borrows

```fusion
fn main() -> int {
    let mut data: string = "hello";
    let r: &string = &data;  // Long-lived borrow

    // Can't mutate while r exists
    // data = "world";  // ERROR: data is borrowed

    println(r);
    return 0;
}
```

### Good: Return Ownership

```fusion
fn process(mut data: string) -> string {
    data = data + " processed";
    return data;  // Return ownership to caller
}

fn main() -> int {
    let data: string = "input";
    let result: string = process(data);
    println(result);
    return 0;
}
```

---

## Tips and Best Practices

1. **Minimize mutable borrows**: Use immutable borrows whenever possible.
2. **Use scoped borrows**: Contain borrows within `{ ... }` blocks to limit their lifetime.
3. **Return ownership**: When a function needs to give data back, return it.
4. **Use `clone()` sparingly**: Prefer moving values over cloning when possible.
5. **Trust the compiler**: Vortex error messages explain the problem and how to fix it.

---

## Common Mistakes and How to Fix Them

### Mistake 1: Using a Moved Value

```fusion
fn main() -> int {
    let s: string = "hello";
    let t: string = s;
    // println(s);  // ERROR: use of moved value

    // Fix: Use .clone() if you need both values
    let s: string = "hello";
    let t: string = s.clone();
    println("%s %s", s, t);

    return 0;
}
```

### Mistake 2: Mutable Borrow While Shared Borrow Exists

```fusion
fn main() -> int {
    let mut data: Vec<int> = vec![1, 2, 3];
    let r: &Vec<int> = &data;
    // let r2: &mut Vec<int> = &mut data;  // ERROR: data is already borrowed

    // Fix: Finish using shared borrow before mutable borrow
    println("%d", r[0]);  // Last use of r
    let r2: &mut Vec<int> = &mut data;  // OK now
    r2.push(4);

    return 0;
}
```

### Mistake 3: Long-Lived Mutable Borrow

```fusion
fn main() -> int {
    let mut items: Vec<int> = vec![1, 2, 3];
    let mut borrowed: &mut Vec<int> = &mut items;
    borrowed.push(4);
    // items.push(5);  // ERROR: items is mutably borrowed

    // Fix: Use scoped borrows
    let mut items: Vec<int> = vec![1, 2, 3];
    {
        let borrowed: &mut Vec<int> = &mut items;
        borrowed.push(4);
    }  // borrowed goes out of scope
    items.push(5);  // OK now

    return 0;
}
```

### Mistake 4: Forgetting to Use Ownership

```fusion
fn process(data: Vec<int>) -> int {
    return data.len();
}

fn main() -> int {
    let nums: Vec<int> = vec![1, 2, 3];
    let len: int = process(nums);
    // println(nums.len());  // ERROR: nums was moved

    // Fix 1: Use a reference instead
    fn process_ref(data: &Vec<int>) -> int {
        return data.len();
    }
    let nums: Vec<int> = vec![1, 2, 3];
    let len: int = process_ref(&nums);
    println(nums.len());  // OK: nums still owned

    // Fix 2: Clone if you need both
    let nums: Vec<int> = vec![1, 2, 3];
    let len: int = process(nums.clone());
    println(nums.len());  // OK: nums was cloned

    return 0;
}
```

---

## Complete Example: Safe Memory Management

```fusion
struct Database {
    connection: *Connection,
    data: Vec<Record>,
}

impl Database {
    fn new(url: string) -> Database {
        let conn: *Connection = connect(url);
        return Database {
            connection: conn,
            data: Vec::new(),
        };
    }

    fn query(self: &mut Database, sql: string) -> Vec<Record> {
        // Execute query and return results
        let results: Vec<Record> = execute(self.connection, sql);
        self.data = results.clone();
        return results;
    }

    fn close(self) {
        // Ensure connection is properly closed
        disconnect(self.connection);
    }
}

fn process_data(db: &mut Database) -> int {
    let results: Vec<Record> = db.query("SELECT * FROM users");
    return results.len();
}

fn main() -> int {
    let mut db: Database = Database::new("localhost:5432");

    // Use mutable borrow for query
    let count: int = process_data(&mut db);

    // Query again after borrow ends
    let more: Vec<Record> = db.query("SELECT * FROM orders");

    println("Users: %d, Orders: %d", count, more.len());

    // Clean up
    db.close();

    return 0;
}
```

---

## Complete Example: Shared State

```fusion
use std::sync;

struct SharedCounter {
    count: sync::Mutex<int>,
    history: sync::RwLock<Vec<int>>,
}

impl SharedCounter {
    fn new() -> SharedCounter {
        return SharedCounter {
            count: sync::Mutex::new(0),
            history: sync::RwLock::new(Vec::new()),
        };
    }

    fn increment(self: &SharedCounter) {
        let mut val: sync::MutexGuard<int> = self.count.lock();
        *val = *val + 1;

        let mut hist: sync::RwLockWriteGuard<Vec<int>> = self.history.write();
        hist.push(*val);
    }

    fn get_count(self: &SharedCounter) -> int {
        let val: sync::MutexGuard<int> = self.count.lock();
        return *val;
    }

    fn get_history(self: &SharedCounter) -> Vec<int> {
        let hist: sync::RwLockReadGuard<Vec<int>> = self.history.read();
        return hist.clone();
    }
}

fn worker(counter: &SharedCounter, id: int) {
    for _ in 0..100 {
        counter.increment();
    }
    println("Worker %d completed", id);
}

fn main() -> int {
    let counter: SharedCounter = SharedCounter::new();

    // Multiple workers share the counter
    let workers: [async::Fiber] = [];
    for i in 0..4 {
        let c: &SharedCounter = &counter;
        workers.push(spawn worker(c, i));
    }

    for w in workers {
        w.join();
    }

    println("Final count: %d", counter.get_count());
    println("History length: %d", counter.get_history().len());

    return 0;
}
```

---

## Cross-References

- **Chapter 2**: Syntax for basic variable declarations
- **Chapter 3**: Structs and Enums for custom types
- **Chapter 5**: Generics for parameterized types
- **Chapter 10**: Concurrency for shared state patterns
- **Chapter 13**: Advanced for FFI and unsafe blocks
