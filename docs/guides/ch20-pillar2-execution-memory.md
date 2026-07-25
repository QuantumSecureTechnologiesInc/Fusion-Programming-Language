# Chapter 20: Pillar 2 — The Execution Model & Memory (The Engine)

> How Fusion v2.0 Vortex turns source code into running programs — and manages the memory they consume.

---

## Execution Strategy

A language's soul is its computational foundation, but its **engine** is the execution model: how source code becomes machine instructions, how memory is allocated and freed, and how concurrent work is scheduled. Fusion v2.0 Vortex supports multiple execution targets, giving you the right tool for every deployment scenario.

### Compiler Pipeline

Fusion compiles through a multi-phase pipeline:

```
Source Code (.fu)
    │
    ▼
┌─────────┐
│   Lex   │  Tokenization — break source into tokens
└────┬────┘
     ▼
┌─────────┐
│  Parse  │  Syntax analysis — build AST (Abstract Syntax Tree)
└────┬────┘
     ▼
┌─────────┐
│  Sema   │  Semantic analysis — type checking, borrow checking, name resolution
└────┬────┘
     ▼
┌─────────┐
│ Codegen │  Code generation — emit target-specific IR
└────┬────┘
     │
     ├──────────┬──────────┐
     ▼          ▼          ▼
┌─────────┐ ┌──────┐ ┌────────┐
│  LLVM   │ │ WASM │ │ Bytecode│
│  Native │ │      │ │   VM   │
└─────────┘ └──────┘ └────────┘
```

Each phase can be invoked independently:

```bash
# Lex only (token stream)
fuc lex input.fu --output tokens.json

# Parse only (AST)
fuc parse input.fu --output ast.json

# Type check (semantic analysis)
fuc check input.fu

# Full compilation to native binary
fuc build input.fu -o output

# Compile to WASM
fuc build input.fu --target wasm32 -o output.wasm

# Compile to bytecode
fuc build input.fu --target bytecode -o output.fvc
```

### LLVM Native Backend

The primary backend compiles Fusion to LLVM IR, then to native machine code via LLVM. This gives you:

- **Full performance** — LLVM's optimization passes (inlining, vectorization, loop unrolling, dead code elimination)
- **Platform-native code** — links against OS APIs, uses system calls directly
- **No runtime overhead** — no garbage collector, no interpreter, no VM

```bash
# Compile for current platform
fuc build input.fu -o myapp

# Compile with optimizations
fuc build input.fu -o myapp --release

# Compile with specific optimizations
fuc build input.fu -o myapp --opt-level 3
```

**Optimization levels:**

| Level | Description |
|---|---|
| `-O0` | No optimization (fastest compile, best for debugging) |
| `-O1` | Basic optimizations |
| `-O2` | Standard optimizations (default for `--release`) |
| `-O3` | Aggressive optimizations (may increase binary size) |
| `-Os` | Optimize for size |
| `-Oz` | Aggressive size optimization |

### WASM Backend

Compiles to WebAssembly for browser and sandboxed environments:

```bash
# Compile for WASM
fuc build input.fu --target wasm32 -o output.wasm

# Compile with WASI support
fuc build input.fu --target wasm32-wasi -o output.wasm

# Optimize WASM output
fuc build input.fu --target wasm32 -o output.wasm --wasm-opt
```

**WASM-specific considerations:**
- No direct system calls (use WASI interface)
- 32-bit address space
- No threading by default (use WASM threads proposal)
- Smaller binary sizes via `wasm-opt`
- Sandboxed execution — no access to host filesystem unless explicitly granted

### Bytecode VM

For rapid prototyping, scripting, and hot-reload scenarios:

```bash
# Compile to bytecode
fuc build input.fu --target bytecode -o output.fvc

# Run with bytecode VM
fuc run output.fvc

# Run source directly (implicit compile + execute)
fuc run input.fu
```

The bytecode VM is useful for:
- **Scripting** — embed Fusion as a scripting language
- **Hot reload** — reload bytecode without full recompilation
- **Debugging** — step through bytecode instructions
- **Embedded systems** — smaller footprint than native code

### How to Choose a Target

| Scenario | Target | Why |
|---|---|---|
| Production server | LLVM native (`--release`) | Maximum performance |
| Browser application | WASM (`wasm32`) | Runs in browser sandbox |
| CLI tool | LLVM native | Direct OS interaction |
| Plugin system | Bytecode VM | Hot reload, sandboxing |
| Embedded device | LLVM native (cross-compile) | Minimal runtime |
| Scripting / automation | Bytecode VM | Fast startup, no compilation step |
| Testing | `fuc run` (implicit) | Quick iteration |
| Mobile app | WASM (via Capacitor/Ionic) | Cross-platform |

---

## Memory Management

Memory management is where Fusion earns its reputation as a systems language. It gives you **precise control** over allocation and deallocation while preventing the most dangerous classes of memory bugs at compile time.

### Stack vs Heap Allocation

#### Stack Allocation (Default)

Values live on the stack — fast, automatic, no allocation cost:

```fusion
fn compute() {
    let x: Int = 42;           // Stack
    let y: Float = 3.14;       // Stack
    let arr: [Int; 4] = [1, 2, 3, 4];  // Stack (fixed-size array)

    // All freed automatically when function returns
}
```

**Stack characteristics:**
- Allocated at function entry, freed at function exit
- LIFO order — no fragmentation
- Size must be known at compile time
- Extremely fast — just a pointer adjustment

#### Heap Allocation (When Needed)

Dynamic or large data lives on the heap:

```fusion
fn create_data() {
    let v: Vec<Int> = vec![1, 2, 3, 4, 5];  // Heap (dynamic size)
    let s: String = "hello".to_string();       // Heap (dynamic size)
    let b: Box<Int> = Box::new(42);            // Heap (explicit boxing)

    // Freed when variables go out of scope (RAII)
}
```

**Heap characteristics:**
- Allocated with `alloc`, freed with `dealloc`
- Size can be determined at runtime
- Supports dynamic growth (`Vec`, `String`)
- Slightly slower than stack (allocation + indirection)

### Ownership Model (Rust-Style)

Fusion uses **affine types** for ownership tracking. Every value has exactly one owner. When the owner goes out of scope, the value is dropped (freed).

```fusion
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;       // Ownership moves from s1 to s2

    // print(s1);       // Compile error! s1 is no longer valid
    print(s2);          // OK — s2 owns the string
}
```

**Ownership rules:**
1. Each value has exactly one owner
2. When the owner goes out of scope, the value is dropped
3. Assignment moves ownership (unless `Copy` type)
4. Ownership can be transferred via function calls

```fusion
fn takes_ownership(s: String) {
    print(s);
    // s is dropped here
}

fn makes_copy(x: Int) {
    print(x);
    // x is dropped, but original still valid (Int implements Copy)
}

fn main() {
    let s = String::from("hello");
    takes_ownership(s);          // Ownership moved
    // print(s);                 // Compile error!

    let x = 42;
    makes_copy(x);               // Value copied
    print(x);                    // OK — x still valid
}
```

### Borrowing Rules

Borrowing lets you reference a value without taking ownership:

#### Immutable References (`&T`)

```fusion
fn calculate_length(s: &String) -> usize {
    s.len()  // Can read, cannot modify
}

fn main() {
    let s = String::from("hello");
    let len = calculate_length(&s);  // Borrow s
    print("'{s}' has length {len}");
    // s is still valid — we only borrowed it
}
```

**Rules for `&T`:**
- Multiple immutable references allowed simultaneously
- Cannot modify the borrowed value
- Reference must not outlive the borrowed value

```fusion
fn main() {
    let s = String::from("hello");
    let r1 = &s;   // OK
    let r2 = &s;   // OK — multiple immutable borrows
    let r3 = &s;   // OK — still fine

    print("{r1} {r2} {r3}");
}
```

#### Mutable References (`&mut T`)

```fusion
fn push_world(s: &mut String) {
    s.push_str(", world!");
}

fn main() {
    let mut s = String::from("hello");
    push_world(&mut s);  // Mutable borrow
    print(s);             // "hello, world!"
}
```

**Rules for `&mut T`:**
- Only ONE mutable reference at a time
- No immutable references while a mutable reference exists
- Cannot modify the borrowed value through an immutable reference while mutably borrowed

```fusion
fn main() {
    let mut s = String::from("hello");

    let r1 = &mut s;
    // let r2 = &mut s;          // Compile error! Two mutable borrows

    r1.push_str(" world");
    print(r1);

    let r1 = &s;                // OK — mutable borrow ended
    let r2 = &s;                // OK — multiple immutable borrows
    print("{r1} {r2}");
}
```

#### Borrowing in Practice

```fusion
struct TextBuffer {
    content: Vec<char>,
    cursor: usize,
}

impl TextBuffer {
    fn new() -> Self {
        Self {
            content: Vec::new(),
            cursor: 0,
        }
    }

    fn insert(&mut self, ch: char) {
        self.content.insert(self.cursor, ch);
        self.cursor += 1;
    }

    fn get_char(&self, index: usize) -> Option<&char> {
        self.content.get(index)
    }

    fn get_range(&self, start: usize, end: usize) -> &[char] {
        &self.content[start..end]
    }

    fn len(&self) -> usize {
        self.content.len()
    }
}

fn count_words(buffer: &TextBuffer) -> usize {
    // Read-only borrow — can call any &self method
    let text: String = buffer.get_range(0, buffer.len()).iter().collect();
    text.split_whitespace().count()
}

fn main() {
    let mut buf = TextBuffer::new();
    buf.insert('H');
    buf.insert('e');
    buf.insert('l');
    buf.insert('l');
    buf.insert('o');

    // Multiple immutable borrows
    let len = buf.len();
    let ch = buf.get_char(0);
    print("Length: {len}, First char: {ch:?}");

    // Mutable borrow (exclusive)
    buf.insert(' ');
    buf.insert('W');
    print(count_words(&buf)); // 1
}
```

### Linear Types for Resource Management

Linear types ensure resources are used exactly once — not twice, not zero times:

```fusion
// File handle as a linear type
struct FileHandle {
    fd: RawFd,
}

impl FileHandle {
    fn open(path: &str) -> Result<Self, IoError> {
        // ...
    }

    fn read_line(&self) -> Result<String, IoError> {
        // ...
    }

    fn close(self) -> Result<(), IoError> {  // Consumes self
        // ...
    }
}

fn process_file(path: &str) -> Result<(), IoError> {
    let file = FileHandle::open(path)?;   // FileHandle created
    let line = file.read_line()?;          // Still valid — borrowed
    // file is automatically closed when it goes out of scope

    // file.close()?;  // Optional explicit close

    Ok(())
    // file is dropped here — guaranteed
}
```

**Key properties:**
- Value must be used exactly once (moved, consumed, or dropped)
- Prevents use-after-free (value cannot be used after being consumed)
- Prevents resource leaks (value cannot be ignored)
- Compile-time enforcement — zero runtime cost

### Garbage Collection for Polyglot Objects

When interacting with garbage-collected languages (Python, JavaScript, Java), Fusion uses a hybrid approach:

```fusion
// Bridging to a GC-managed Python object
extern "polyglot" {
    fn py_import(module: &str, name: &str) -> PyObject;
}

fn call_python() {
    let np = py_import("numpy", "array");
    // np is a GC-managed reference
    // Fusion tracks it separately from owned Rust-style objects

    let result = np.call_method("tolist", &[]);
    // result is a new GC reference

    // GC references are dropped when they go out of scope
    // The GC handles the actual memory reclamation
}
```

### Manual `alloc` / `dealloc`

For maximum control, Fusion provides explicit allocation:

```fusion
use std::alloc::{alloc, dealloc, Layout};

fn manual_allocation() {
    let layout = Layout::array::<Int>(1024).unwrap();
    let ptr = unsafe { alloc(layout) } as *mut Int;

    if ptr.is_null() {
        panic!("Allocation failed");
    }

    // Initialize memory
    for i in 0..1024 {
        unsafe { ptr.add(i).write(i as Int); }
    }

    // Use memory
    let sum: Int = (0..1024)
        .map(|i| unsafe { ptr.add(i).read() })
        .sum();

    print("Sum: {sum}");

    // Free memory
    unsafe { dealloc(ptr as *mut u8, layout); }
}
```

**Use manual allocation only when:**
- You need precise control over allocation timing
- You're implementing a custom allocator
- You're interfacing with C code that expects manual memory management
- You're building a memory allocator or garbage collector

---

## Concurrency Model

Fusion provides a layered concurrency model, from low-level OS threads to high-level async/await, with the Supernova runtime bridging CPU, GPU, and quantum processing.

### Threads (OS-Level)

```fusion
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        print("Hello from another thread!");
        42
    });

    let result = handle.join().unwrap();
    print("Thread returned: {result}");
}
```

**Thread characteristics:**
- OS-managed scheduling
- True parallelism on multi-core systems
- Each thread has its own stack (~2MB default)
- Expensive to create (~50μs)
- Use for CPU-bound work that benefits from parallelism

### Fibers (Green Threads)

Lightweight cooperative threads managed by the Fusion runtime:

```fusion
use std::fiber;

fn main() {
    let f1 = fiber::spawn(|| {
        fiber::yield_now();  // Cooperatively yield
        print("Fiber 1 resumed");
    });

    let f2 = fiber::spawn(|| {
        print("Fiber 2 running");
        fiber::yield_now();
        print("Fiber 2 resumed");
    });

    f1.join();
    f2.join();
}
```

**Fiber characteristics:**
- Runtime-managed scheduling (~100ns switch cost)
- Thousands or millions can exist simultaneously
- Cooperative yielding (`fiber::yield_now()`)
- Useful for structured concurrency and concurrent I/O

### Async/Await Event Loop

For I/O-bound concurrency:

```fusion
use std::async;
use std::io;

async fn fetch_url(url: &str) -> Result<String, io::Error> {
    let response = async::http::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

async fn main() {
    let urls = vec![
        "https://api.example.com/data1",
        "https://api.example.com/data2",
        "https://api.example.com/data3",
    ];

    // Concurrent fetches — all run on the event loop
    let results: Vec<_> = async::join_all(
        urls.iter().map(|url| fetch_url(url))
    ).await;

    for (url, result) in urls.iter().zip(results) {
        match result {
            Ok(body) => print("Got {url}: {body} bytes"),
            Err(e) => print("Failed {url}: {e}"),
        }
    }
}
```

**Async characteristics:**
- Zero-cost abstraction (compiles to state machine)
- Single-threaded event loop (or multi-threaded via runtime)
- Ideal for network I/O, file I/O, database queries
- `.await` suspends the task until the future completes

### Channels for Message Passing

CSP-style communication between concurrent tasks:

```fusion
use std::channel;

fn main() {
    let (tx, rx) = channel::unbounded::<String>();

    // Producer
    let tx1 = tx.clone();
    thread::spawn(move || {
        for i in 0..5 {
            tx1.send(format!("Message {i}")).unwrap();
        }
    });

    // Producer 2
    let tx2 = tx.clone();
    thread::spawn(move || {
        for i in 5..10 {
            tx2.send(format!("Message {i}")).unwrap();
        }
    });

    // Consumer
    drop(tx);  // Close our copy so we know when all senders are done
    while let Ok(msg) = rx.recv() {
        print("Received: {msg}");
    }
    print("All senders dropped, channel closed");
}
```

**Channel types:**
- `unbounded()` — infinite buffer, backpressure-free
- `bounded(capacity)` — bounded buffer, blocks when full
- `oneshot()` — single value, single sender/receiver
- `broadcast()` — multiple consumers, each gets every message
- `watch()` — single producer, consumers see latest value

### Mutex / RwLock for Shared State

When message passing is too heavy:

```fusion
use std::sync::{Arc, Mutex, RwLock};

fn with_mutex() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = counter.clone();
        handles.push(thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    print("Final count: {}", *counter.lock().unwrap());
}

fn with_rwlock() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Multiple readers
    for _ in 0..5 {
        let data = data.clone();
        handles.push(thread::spawn(move || {
            let guard = data.read().unwrap();
            print("Read: {:?}", *guard);
        }));
    }

    // Single writer
    {
        let data = data.clone();
        handles.push(thread::spawn(move || {
            let mut guard = data.write().unwrap();
            guard.push(4);
            print("Wrote: {:?}", *guard);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
```

**Locking strategies:**
- `Mutex<T>` — exclusive access, blocks when locked
- `RwLock<T>` — multiple readers OR single writer
- `TryLock` — non-blocking attempt to acquire
- `TimedLock` — acquire with timeout

### Atomics for Lock-Free Operations

```fusion
use std::sync::atomic::{AtomicInt, Ordering};

fn lock_free_counter() {
    let counter = AtomicInt::new(0);
    let mut handles = vec![];

    for _ in 0..100 {
        let counter = &counter;
        handles.push(thread::spawn(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    print("Final count: {}", counter.load(Ordering::SeqCst));
}
```

**Atomic ordering:**
- `Relaxed` — no ordering guarantees (fastest)
- `Acquire` — subsequent reads see writes before the release
- `Release` — prior reads/writes visible to the acquire
- `AcqRel` — combines acquire and release
- `SeqCst` — total sequential consistency (safest, slowest)

### Supernova Runtime

Fusion's unified runtime for heterogeneous compute:

```fusion
use std::supernova::{Runtime, Device, Kernel};

fn main() {
    let runtime = Runtime::new();

    // Discover available devices
    let devices = runtime.devices();
    for device in &devices {
        print("Device: {device.name()} ({device.type()})");
    }

    // CPU compute
    let cpu_result = runtime.execute(Device::CPU, || {
        // CPU-bound work
        (0..1_000_000).sum::<Int>()
    });

    // GPU compute (if available)
    if runtime.has_device(Device::GPU) {
        let gpu_result = runtime.execute(Device::GPU, || {
            // GPU kernel launch
            Kernel::new("matrix_multiply")
                .arg(&matrix_a)
                .arg(&matrix_b)
                .arg(&mut result)
                .dispatch([1024, 1024, 1])
                .await
        });
    }

    // Quantum compute (if available)
    if runtime.has_device(Device::QPU) {
        let qpu_result = runtime.execute(Device::QPU, || {
            // Quantum circuit
            QuantumCircuit::new(8)
                .h(0)
                .cx(0, 1)
                .measure_all()
                .execute(1000)
        });
    }
}
```

### Cortex Scheduler

AI-powered work distribution:

```fusion
use std::cortex::Cortex;

fn main() {
    let cortex = Cortex::new();

    // Cortex automatically chooses the best device for each task
    let result = cortex.schedule(|task| {
        task.priority(High)
            .hint(Device::GPU)  // Suggestion, not requirement
            .execute(|| {
                // Work that Cortex will route optimally
                heavy_computation()
            })
    });

    // Batch scheduling — Cortex optimizes across all tasks
    let results = cortex.schedule_batch(vec![
        ("train_model", train_model_task),
        ("process_data", process_data_task),
        ("evaluate", evaluate_task),
    ]);
}
```

**Cortex capabilities:**
- Device-aware scheduling (CPU/GPU/QPU)
- Priority-based preemption
- Load balancing across devices
- Work stealing between threads
- Adaptive batching for ML workloads

### Data-Race Prevention via Vortex Borrow Checker

The Vortex borrow checker extends Rust's ownership model with additional safety guarantees:

```fusion
// Vortex ensures no data races at compile time
fn main() {
    let mut data = vec![1, 2, 3, 4, 5];

    // Vortex detects this as a data race and rejects it:
    // thread::spawn(|| {
    //     data.push(6);  // Mutable borrow on thread 1
    // });
    // data.push(7);     // Mutable borrow on main thread
    //                    // COMPILE ERROR: data race detected

    // Correct: use Arc<Mutex<>> or channels
    let data = Arc::new(Mutex::new(data));
    let data_clone = data.clone();

    thread::spawn(move || {
        data_clone.lock().unwrap().push(6);
    });

    data.lock().unwrap().push(7);
}
```

**Vortex borrow checker features:**
- **Thread-aware borrowing** — tracks borrows across thread boundaries
- **Lifetime analysis** — ensures references don't outlive their data
- **Conditional borrowing** — borrows that depend on runtime conditions
- **Flow-sensitive types** — type state changes based on control flow
- **Compile-time data race prevention** — zero runtime cost

---

## Portability

### Platform Abstraction

Fusion's standard library abstracts over OS differences:

```fusion
use std::fs;
use std::env;
use std::path::PathBuf;

fn platform_agnostic() {
    // Path handling
    let config_dir = if cfg!(windows) {
        PathBuf::from(env::var("APPDATA").unwrap())
    } else {
        PathBuf::from(env::var("HOME").unwrap())
            .join(".config")
    };

    // File operations
    let content = fs::read_to_string(config_dir.join("app.toml"))
        .unwrap_or_default();

    // Environment
    for (key, value) in env::vars() {
        if key.starts_with("MY_APP_") {
            print("{key} = {value}");
        }
    }
}
```

### Endianness Handling

```fusion
use std::mem;

fn main() {
    if mem::needs_endian_swap() {
        print("System is big-endian");
    } else {
        print("System is little-endian");
    }

    // Explicit byte order
    let value: u32 = 0x12345678;
    let little_bytes = value.to_le_bytes();
    let big_bytes = value.to_be_bytes();
    let native_bytes = value.to_ne_bytes();

    print("Little-endian: {little_bytes:?}");
    print("Big-endian: {big_bytes:?}");
    print("Native: {native_bytes:?}");
}
```

### Cross-Compilation

```bash
# List available targets
fuc target list

# Compile for different platform
fuc build input.fu --target x86_64-unknown-linux-gnu -o output-linux
fuc build input.fu --target aarch64-apple-darwin -o output-macos-arm
fuc build input.fu --target wasm32-unknown-unknown -o output.wasm

# Cross-compile with a target specification file
fuc build input.fu --target custom-target.json -o output
```

**Supported targets (partial list):**

| Target | Platform |
|---|---|
| `x86_64-pc-windows-msvc` | Windows x64 |
| `x86_64-unknown-linux-gnu` | Linux x64 |
| `aarch64-apple-darwin` | macOS ARM64 (Apple Silicon) |
| `wasm32-unknown-unknown` | WebAssembly |
| `wasm32-wasi` | WebAssembly + WASI |
| `riscv64gc-unknown-linux-gnu` | RISC-V 64-bit |
| `thumbv7em-none-eabihf` | ARM Cortex-M (embedded) |

---

## Code Examples

### Multi-Threaded Program

```fusion
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Stats {
    processed: usize,
    errors: usize,
    total_time: Duration,
}

fn worker(id: usize, data: Arc<Vec<Int>>, results: Arc<Mutex<Vec<Int>>>) {
    let start = Instant::now();
    let mut local_results = Vec::new();

    for &value in data.iter() {
        // Simulate processing
        let processed = value * value + id as Int;
        local_results.push(processed);
        thread::sleep(Duration::from_millis(1));
    }

    let mut results = results.lock().unwrap();
    results.extend(local_results);

    print("Worker {id} finished in {:?}", start.elapsed());
}

fn main() {
    let data: Arc<Vec<Int>> = Arc::new((0..1000).collect());
    let results: Arc<Mutex<Vec<Int>>> = Arc::new(Mutex::new(Vec::new()));

    let num_workers = 4;
    let chunk_size = data.len() / num_workers;

    let start = Instant::now();
    let mut handles = vec![];

    for id in 0..num_workers {
        let data = data.clone();
        let results = results.clone();

        handles.push(thread::spawn(move || {
            worker(id, data, results);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let results = results.lock().unwrap();
    let elapsed = start.elapsed();

    print("Processed {} items in {elapsed:?}", results.len());
    print("Average: {:.2}", results.iter().sum::<Int>() as Float / results.len() as Float);
}
```

### Async/Await Program

```fusion
use std::async;
use std::io;
use std::time::{Duration, Instant};

async fn slow_operation(name: &str, duration_ms: u64) -> String {
    async::sleep(Duration::from_millis(duration_ms)).await;
    format!("{name} completed after {duration_ms}ms")
}

async fn fetch_data(url: &str) -> Result<String, io::Error> {
    let response = async::http::get(url).await?;
    Ok(response.text().await?)
}

async fn main() {
    let start = Instant::now();

    // Sequential — slow
    let r1 = slow_operation("Task 1", 100).await;
    let r2 = slow_operation("Task 2", 100).await;
    let r3 = slow_operation("Task 3", 100).await;
    print("Sequential: {start:?} — {r1}, {r2}, {r3}");

    // Concurrent — fast
    let start = Instant::now();
    let (r1, r2, r3) = async::join!(
        slow_operation("Task 1", 100),
        slow_operation("Task 2", 100),
        slow_operation("Task 3", 100),
    );
    print("Concurrent: {start:?} — {r1}, {r2}, {r3}");

    // Stream processing
    let mut stream = async::stream::from_iter(vec![
        "https://api.example.com/1",
        "https://api.example.com/2",
        "https://api.example.com/3",
    ]);

    while let Some(url) = stream.next().await {
        match fetch_data(url).await {
            Ok(data) => print("Got data from {url}: {data} bytes"),
            Err(e) => print("Error fetching {url}: {e}"),
        }
    }
}
```

### Channel-Based Communication

```fusion
use std::channel;
use std::thread;
use std::time::Duration;

enum Task {
    Process(String),
    Shutdown,
}

fn main() {
    let (task_tx, task_rx) = channel::bounded::<Task>(10);
    let (result_tx, result_rx) = channel::unbounded::<String>();

    // Start worker pool
    let num_workers = 3;
    let mut workers = vec![];

    for id in 0..num_workers {
        let task_rx = task_rx.clone();
        let result_tx = result_tx.clone();

        workers.push(thread::spawn(move || {
            loop {
                match task_rx.recv() {
                    Ok(Task::Process(data)) => {
                        // Simulate processing
                        thread::sleep(Duration::from_millis(50));
                        let result = format!("Worker {id} processed: {data}");
                        result_tx.send(result).unwrap();
                    }
                    Ok(Task::Shutdown) => {
                        print("Worker {id} shutting down");
                        break;
                    }
                    Err(_) => break,
                }
            }
        }));
    }

    // Drop our copies so channels close properly
    drop(task_rx);
    drop(result_tx);

    // Send tasks
    for i in 0..10 {
        task_tx.send(Task::Process(format!("item_{i}"))).unwrap();
    }

    // Send shutdown signals
    for _ in 0..num_workers {
        task_tx.send(Task::Shutdown).unwrap();
    }
    drop(task_tx);

    // Collect results
    let mut results = Vec::new();
    while let Ok(result) = result_rx.recv() {
        results.push(result);
    }

    // Wait for workers
    for worker in workers {
        worker.join().unwrap();
    }

    print("All {} results collected", results.len());
    for result in &results {
        print("  {result}");
    }
}
```

---

## Summary

Pillar 2 defines how Fusion v2.0 Vortex **executes** and **manages memory**:

- **Multiple backends**: LLVM native for performance, WASM for portability, Bytecode VM for scripting
- **Ownership model**: Rust-style affine types with the Vortex borrow checker for compile-time memory safety
- **Stack vs heap**: Automatic stack allocation for known-size data, explicit heap allocation when needed
- **Linear types**: Guarantee resources are used exactly once, preventing leaks and use-after-free
- **GC bridge**: Seamless interop with garbage-collected polyglot languages
- **Concurrency layers**: OS threads for parallelism, fibers for lightweight concurrency, async/await for I/O
- **Message passing**: CSP-style channels for safe concurrent communication
- **Shared state**: Mutex/RwLock for when channels are too heavy, atomics for lock-free operations
- **Supernova runtime**: Heterogeneous compute across CPU/GPU/QPU
- **Cortex scheduler**: AI-powered work distribution
- **Full portability**: Cross-compilation, endianness handling, platform abstraction

---

> **Next**: [Chapter 21 — Pillar 3: Safety, Reliability & Error Handling (The Airbags)](ch21-pillar3-safety-error-handling.md)
