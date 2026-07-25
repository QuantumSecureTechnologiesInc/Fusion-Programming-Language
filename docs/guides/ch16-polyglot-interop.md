# Chapter 16: Polyglot Interoperability

> Formal Interoperability Protocol, FFI, Polyglot API, Data Type Mapping, Shared Memory, Foreign Value Proxies, Cross-Language Concurrency, Guest/Host Semantics

---

## Formal Interoperability Protocol

Fusion defines a formal protocol for interacting with external languages. The protocol establishes three contract layers:

1. **ABI Contract** — memory layout, calling conventions, and stack frame expectations
2. **Type Contract** — bidirectional type mapping between Fusion and the target language
3. **Lifecycle Contract** — object ownership, reference counting, and garbage-collection bridge

Every FFI boundary in Fusion must declare which contract layers are active:

```fusion
#[ffi(
    abi = "cdecl",
    types = "explicit",
    lifecycle = "shared"
)]
extern "python" {
    fn add(a: int, b: int) -> int;
}
```

---

## FFI with Calling Conventions

Fusion supports multiple calling conventions for foreign functions:

```fusion
// Cdecl — standard C calling convention
extern "cdecl" {
    fn c_printf(fmt: *const u8, ...) -> int;
}

// Stdcall — Windows API calling convention
extern "stdcall" {
    fn WinMain(hInstance: u64, hPrevInstance: u64, lpCmdLine: *const u8, nCmdShow: int) -> int;
}

// System V AMD64 — Linux/macOS ABI
extern "sysv" {
    fn pthread_create(
        thread: *mut u64,
        attr: *const u8,
        start_routine: fn(*mut u8) -> *mut u8,
        arg: *mut u8
    ) -> int;
}

// Fastcall — register-based calling
extern "fastcall" {
    fn high_perf_func(a: i32, b: i32, c: i32, d: i32) -> i64;
}
```

### Linking External Libraries

```fusion
// Link a shared library
#[link(name = "m")]
extern "cdecl" {
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn sqrt(x: f64) -> f64;
}

// Link a static library
#[link(name = "sqlite3", kind = "static")]
extern "cdecl" {
    fn sqlite3_open(path: *const u8, ppDb: *mut *mut u8) -> int;
    fn sqlite3_close(pDb: *mut u8) -> int;
}

// Link with full path
#[link(name = "libfoo.so", path = "/opt/libs")]
extern "cdecl" {
    fn foo_process(data: *mut u8, len: u64) -> int;
}
```

---

## Polyglot API

The Polyglot API provides three core operations: `import`, `export`, and `eval`.

### Import

Bring functions, types, and values from foreign languages into Fusion:

```fusion
// Import a Python module
#[polyglot::import(lang = "python")]
mod numpy as np;

// Import a specific function
#[polyglot::import(lang = "python", from = "math")]
fn factorial(n: int) -> int;

// Import a Rust crate
#[polyglot::import(lang = "rust")]
mod serde as json_serde;

// Import with type mapping
#[polyglot::import(lang = "java", class = "java.util.HashMap")]
struct JavaHashMap<K, V> {
    fn new() -> JavaHashMap<K, V>;
    fn put(key: K, value: V) -> V;
    fn get(key: K) -> Option<V>;
    fn size() -> int;
}
```

### Export

Expose Fusion functions to foreign languages:

```fusion
// Export to Python
#[polyglot::export(lang = "python")]
pub fn compute_distance(x1: float, y1: float, x2: float, y2: float) -> float {
    let dx = x2 - x1;
    let dy = y2 - y1;
    return (dx * dx + dy * dy).sqrt();
}

// Export to Rust
#[polyglot::export(lang = "rust")]
pub fn parse_config(path: string) -> Result<Config, string> {
    let content = std::fs::read_to_string(path)?;
    return Config::from_toml(content);
}

// Export to JavaScript
#[polyglot::export(lang = "javascript", name = "processData")]
pub fn js_process_data(data: Vec<int>) -> Vec<int> {
    return data.iter().map(|x| x * 2).collect();
}
```

### Eval

Evaluate foreign code at runtime:

```fusion
use std::polyglot;

fn main() -> int {
    // Evaluate Python code
    let result = polyglot::eval("python", "
import numpy as np
arr = np.array([1, 2, 3, 4, 5])
print(arr.mean())
arr.mean()
    ");
    println("Mean: %f", result.as_float());

    // Evaluate JavaScript code
    let json_str = polyglot::eval("javascript", "
JSON.stringify({ name: 'fusion', version: 2.0 })
    ");
    println("JSON: %s", json_str.as_string());

    // Evaluate Rust code at build time
    let computed = polyglot::eval("rust", "
        let x: i64 = (1..=100).sum();
        x.to_string()
    ");
    println("Sum 1-100: %s", computed.as_string());

    return 0;
}
```

---

## Data Type Mapping

Fusion provides bidirectional type mapping for each supported language.

### Fusion ↔ Python

| Fusion Type | Python Type | Notes |
|-------------|-------------|-------|
| `int` | `int` | Arbitrary precision in Python |
| `float` | `float` | Python float is f64 |
| `bool` | `bool` | Direct mapping |
| `string` | `str` | UTF-8 ↔ Python str |
| `bytes` | `bytes` | Direct mapping |
| `Vec<T>` | `list` | Copies on boundary |
| `HashMap<K,V>` | `dict` | Copies on boundary |
| `Option<T>` | `T \| None` | Automatic wrapping |
| `Result<T,E>` | `T \| Exception` | Catches Python exceptions |
| `Tensor` | `numpy.ndarray` | Zero-copy when possible |

```fusion
use std::polyglot;

#[polyglot::import(lang = "python")]
mod pandas as pd;

fn process_csv(path: string) -> Vec<Vec<string>> {
    // Fusion string → Python str automatically
    let df = pd::read_csv(path);

    // Python list of lists → Fusion Vec<Vec<string>>
    let data: Vec<Vec<string>> = polyglot::cast(df::to_list());

    return data;
}
```

### Fusion ↔ Rust

| Fusion Type | Rust Type | Notes |
|-------------|-----------|-------|
| `int` | `i64` | Direct |
| `float` | `f64` | Direct |
| `bool` | `bool` | Direct |
| `string` | `String` | UTF-8 owned |
| `&string` | `&str` | Borrowed |
| `Vec<T>` | `Vec<T>` | Direct |
| `HashMap<K,V>` | `HashMap<K,V>` | Direct |
| `Option<T>` | `Option<T>` | Direct |
| `Result<T,E>` | `Result<T,E>` | Direct |
| `struct S {}` | `struct S {}` | Field-compatible |
| `enum E { A, B }` | `enum E { A, B }` | Tagged union |

```fusion
#[polyglot::import(lang = "rust")]
mod serde_json;

fn serialize_user(user: User) -> string {
    // Fusion struct → Rust struct (zero-copy)
    let json = serde_json::to_string(&user)?;
    return json;
}
```

### Fusion ↔ JavaScript

| Fusion Type | JavaScript Type | Notes |
|-------------|----------------|-------|
| `int` | `number` | JS number is f64 |
| `float` | `number` | Direct |
| `bool` | `boolean` | Direct |
| `string` | `string` | UTF-8 ↔ JS string |
| `Vec<T>` | `Array` | Copies on boundary |
| `HashMap<K,V>` | `Object` | Converts keys to strings |
| `Option<T>` | `T \| undefined` | Maps None → undefined |
| `Result<T,E>` | `T` | Throws on Err |
| `Tensor` | `TypedArray` | Shares buffer |

### Fusion ↔ Java

| Fusion Type | Java Type | Notes |
|-------------|-----------|-------|
| `int` | `long` | Direct |
| `float` | `double` | Direct |
| `bool` | `boolean` | Direct |
| `string` | `String` | UTF-8 ↔ Java String |
| `Vec<T>` | `ArrayList<T>` | Copies on boundary |
| `HashMap<K,V>` | `HashMap<K,V>` | Direct |
| `Option<T>` | `T \| null` | Maps None → null |

---

## Shared Memory and Pass-by-Reference

Fusion supports zero-copy data sharing across language boundaries.

### SharedBuffer

```fusion
use std::interop::SharedBuffer;

fn main() -> int {
    // Allocate shared memory visible to all languages
    let buf = SharedBuffer::new(1024 * 1024); // 1MB

    // Write from Fusion
    buf.write_f64(0, 3.14159);
    buf.write_bytes(8, b"hello");

    // Pass to Python — zero copy
    let py_result = polyglot::eval("python", "
import struct
value = struct.unpack('d', shared_mem[0:8])[0]
text = shared_mem[8:13].decode('utf-8')
value + 1.0
    ");

    // Read result back
    let result = py_result.as_float();
    println("Python computed: %f", result);

    return 0;
}
```

### Pass-by-Reference

```fusion
// Pass Fusion data by reference to avoid copying
#[polyglot::export(lang = "python", ref = true)]
pub fn get_large_array() -> Vec<f64> {
    return (0..1_000_000).map(|i| i as f64 * 0.1).collect();
}

// Python can read the array without copying
// #[polyglot::import(lang = "python")]
// fn process(data: &[f64]) -> f64;  // Python receives a memoryview
```

### Reference Counting Bridge

```fusion
use std::interop::ArcHandle;

// Shared ownership across languages
struct Database {
    connection: ArcHandle<Connection>,
}

impl Database {
    pub fn new(conn: Connection) -> Database {
        Database {
            connection: ArcHandle::new(conn),
        }
    }

    // Clone the handle — ref count increments
    // Python also holds a reference — both sides keep it alive
    pub fn share_to_python(&self) -> ArcHandle<Connection> {
        return self.connection.clone();
    }
}
```

---

## Foreign Value Proxies

Foreign value proxies wrap non-Fusion objects so they can be used idiomatically in Fusion code.

```fusion
use std::polyglot::ForeignProxy;

// Python object proxy
struct PyProxy {
    handle: ForeignProxy,
}

impl PyProxy {
    pub fn getattr(&self, name: string) -> PyProxy {
        return PyProxy {
            handle: self.handle.call_method("getattr", &[name.into()]),
        };
    }

    pub fn call(&self, args: Vec<PyProxy>) -> PyProxy {
        return PyProxy {
            handle: self.handle.call_method("call", &args.iter().map(|a| a.handle.clone()).collect()),
        };
    }

    pub fn to_int(&self) -> int {
        return self.handle.to_i64();
    }

    pub fn to_string(&self) -> string {
        return self.handle.to_string();
    }

    pub fn to_float(&self) -> float {
        return self.handle.to_f64();
    }
}

// Usage
fn main() -> int {
    let np = polyglot::import_module("numpy");
    let arr = np.getattr("array").call(vec![
        PyProxy::from_vec(vec![1.0, 2.0, 3.0])
    ]);
    let mean = arr.getattr("mean").call(vec![]);

    println("Mean: %f", mean.to_float());
    return 0;
}
```

---

## Cross-Language Concurrency

Fusion enables concurrent execution across foreign language runtimes.

### Python Thread Pool

```fusion
use std::interop::ThreadPool;

fn main() -> int {
    // Create a Python thread pool with 4 workers
    let pool = ThreadPool::new("python", 4);

    // Submit tasks to the pool
    let mut handles = Vec::new();
    for i in 0..10 {
        let h = pool.submit(move || {
            polyglot::eval("python", &format!("
import time
time.sleep(0.1)
{}
            ", i * 10))
        });
        handles.push(h);
    }

    // Collect results
    for h in handles {
        let result = h.await();
        println("Result: %s", result.as_string());
    }

    return 0;
}
```

### JavaScript Event Loop Bridge

```fusion
use std::polyglot::JsRuntime;

fn main() -> int {
    // Initialize JS runtime with event loop
    let js = JsRuntime::new();

    // Schedule async work
    let future = js.eval_async("
        new Promise((resolve) => {
            setTimeout(() => resolve(42), 100);
        })
    ");

    // Fusion coroutine awaits the JS promise
    let result = future.await();
    println("JS resolved: %d", result.as_int());

    return 0;
}
```

### Java Virtual Machine Bridge

```fusion
use std::polyglot::JvmHandle;

fn main() -> int {
    // Get or create JVM instance
    let jvm = JvmHandle::get_or_create();

    // Run Java code in a separate thread
    let handle = jvm.spawn_thread(|| {
        let thread = polyglot::eval("java", "
            Thread.currentThread().getName()
        ");
        thread.as_string()
    });

    let thread_name = handle.await();
    println("Java thread: %s", thread_name);
    return 0;
}
```

---

## Guest/Host Semantics

Fusion distinguishes between the **host** language (Fusion) and **guest** languages (Python, JS, etc.).

### Ownership Rules

| Operation | Host → Guest | Guest → Host |
|-----------|-------------|-------------|
| Value types | Copied | Copied |
| Reference types | Shared (ref counted) | Copied (unless `ref=true`) |
| Mutable refs | Moved (invalidated in host) | Copy-on-write |
| Functions | Exported (name-registered) | Imported (lazy-loaded) |

### Error Propagation

```fusion
// Guest errors become Fusion Result types
let result: Result<int, string> = polyglot::eval("python", "
    raise ValueError('invalid input')
");

match result {
    Ok(val) => println("Value: %d", val),
    Err(msg) => println("Python error: %s", msg),
}
```

### Cleanup Hooks

```fusion
// Register cleanup for foreign objects
use std::interop::ForeignHandle;

struct PyModel {
    handle: ForeignHandle,
}

impl Drop for PyModel {
    fn drop(&mut self) {
        // Called when PyModel goes out of scope
        // Releases the Python reference
        self.handle.release();
    }
}
```

---

## Complete Example: Calling Python from Fusion

```fusion
use std::polyglot;

// Import Python modules
#[polyglot::import(lang = "python")]
mod numpy as np;

#[polyglot::import(lang = "python")]
mod sklearn::linear_model as lm;

// Export a Fusion struct to Python
#[polyglot::export(lang = "python")]
pub struct DataPoint {
    pub x: float,
    pub y: float,
    pub label: int,
}

fn main() -> int {
    // Generate training data in Fusion
    let mut data: Vec<DataPoint> = Vec::new();
    for i in 0..100 {
        data.push(DataPoint {
            x: i as float * 0.1,
            y: i as float * 0.2 + 1.0,
            label: if i % 2 == 0 { 0 } else { 1 },
        });
    }

    // Convert to numpy arrays
    let x_arr = np::array(data.iter().map(|d| d.x).collect::<Vec<f64>>());
    let y_arr = np::array(data.iter().map(|d| d.y).collect::<Vec<f64>>());

    // Train a model using scikit-learn
    let model = lm::LinearRegression::new();
    model.fit(x_arr, y_arr);

    // Make predictions
    let test_x = np::array(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let predictions = model.predict(test_x);

    println("Predictions: %s", predictions.to_string());
    return 0;
}
```

---

## Complete Example: Calling Rust from Fusion

```fusion
use std::polyglot;

// Import a Rust crate
#[polyglot::import(lang = "rust")]
mod serde_json;

#[polyglot::import(lang = "rust")]
mod reqwest;

// Define a struct compatible with Rust's serde
#[polyglot::export(lang = "rust", derive = ["Serialize", "Deserialize"])]
pub struct ApiResponse {
    pub status: int,
    pub body: string,
    pub headers: HashMap<string, string>,
}

fn fetch_api(url: string) -> Result<ApiResponse, string> {
    // Call Rust's reqwest from Fusion
    let response = reqwest::blocking::get(url)
        .map_err(|e| e.to_string())?;

    let status = response.status().as_u16() as int;
    let body = response.text().map_err(|e| e.to_string())?;

    // Deserialize JSON using Rust's serde
    let parsed: ApiResponse = serde_json::from_str(&body)
        .map_err(|e| e.to_string())?;

    return Ok(ApiResponse {
        status,
        body,
        headers: HashMap::new(),
    });
}

fn main() -> int {
    match fetch_api("https://api.example.com/data") {
        Ok(resp) => {
            println("Status: %d", resp.status);
            println("Body: %s", resp.body);
        }
        Err(e) => println("Error: %s", e),
    }
    return 0;
}
```

---

## Complete Example: Calling JavaScript from Fusion

```fusion
use std::polyglot;

// Initialize a JS runtime
let js_runtime = polyglot::JsRuntime::new();

fn main() -> int {
    // Define a JS function from Fusion
    js_runtime.eval("
        function fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
    ");

    // Call JS function from Fusion
    for i in 0..20 {
        let result = js_runtime.call("fibonacci", &[i.into()]);
        println("fib(%d) = %d", i, result.as_int());
    }

    // Use JS for JSON processing
    let data = js_runtime.eval("
        const items = [
            { name: 'Alice', age: 30 },
            { name: 'Bob', age: 25 },
            { name: 'Charlie', age: 35 }
        ];
        items.filter(i => i.age > 28).map(i => i.name);
    ");

    let names: Vec<string> = polyglot::cast(data);
    println("Adults: %s", names.join(", "));

    // Use JS crypto for hashing
    let hash = js_runtime.eval("
        const crypto = require('crypto');
        crypto.createHash('sha256').update('hello world').digest('hex');
    ");
    println("SHA-256: %s", hash.as_string());

    return 0;
}
```

---

## Complete Example: Calling Java from Fusion

```fusion
use std::polyglot;

// Import Java classes
#[polyglot::import(lang = "java", class = "java.util.concurrent.ForkJoinPool")]
struct ForkJoinPool {
    fn new(parallelism: int) -> ForkJoinPool;
    fn submit<T>(task: T) -> ForkJoinTask<T>;
    fn shutdown() -> ();
}

#[polyglot::import(lang = "java", class = "java.util.stream.IntStream")]
struct IntStream {
    fn range(start: int, end_exclusive: int) -> IntStream;
    fn sum() -> int;
    fn map<T>(mapper: fn(int) -> T) -> Stream<T>;
    fn collect<T>(collector: Collector<T>) -> T;
}

#[polyglot::import(lang = "java", class = "java.nio.file.Files")]
struct Files {
    fn readAllLines(path: String) -> Vec<String>;
    fn write(path: String, lines: Vec<String>) -> ();
}

fn main() -> int {
    // Use Java's IntStream for parallel computation
    let sum = IntStream::range(1, 1_000_001)
        .parallel()
        .sum();

    println("Sum 1 to 1M: %d", sum);

    // Use Java NIO for file operations
    let lines = Files::readAllLines("data/input.txt".to_string());
    let processed: Vec<String> = lines.iter()
        .map(|l| l.to_uppercase())
        .collect();

    Files::write(
        "data/output.txt".to_string(),
        processed
    );

    // Use ForkJoinPool for parallel processing
    let pool = ForkJoinPool::new(8);
    // ... submit tasks and collect results

    pool.shutdown();
    return 0;
}
```

---

## Configuration in Fusion.toml

Configure polyglot interoperability in your project manifest.

```toml
[project]
name = "polyglot_demo"
version = "1.0.0"

# Python configuration
[interop.python]
enabled = true
version = "3.11"
virtual_env = ".venv"
packages = ["numpy", "pandas", "scikit-learn"]
prelude = "import warnings; warnings.filterwarnings('ignore')"

# Rust configuration
[interop.rust]
enabled = true
edition = "2021"
crates = ["serde", "serde_json", "reqwest", "tokio"]

# JavaScript configuration
[interop.javascript]
enabled = true
engine = "v8"
node_modules = "node_modules"
npm_packages = ["lodash", "express", "ws"]

# Java configuration
[interop.java]
enabled = true
jdk_path = "/usr/lib/jvm/java-17"
class_path = ["libs/java-utils.jar"]
jvm_args = ["-Xmx2g", "-XX:+UseG1GC"]

# Shared memory configuration
[interop.shared_memory]
enabled = true
default_size = "16MB"
max_size = "1GB"

# Thread pool configuration
[interop.thread_pool]
python_workers = 4
rust_workers = 2
js_workers = 2
java_workers = 4
```

---

## Feature Integration Guide

Fusion's compiler-level features (Chapter 18) can be used across polyglot boundaries. This section covers how each of the 16 features integrates with foreign language interop.

### How Features Work Together

When a Fusion module declares features and also uses FFI, the compiler applies the feature transforms to the Fusion side of the boundary. Foreign code runs under its own language's semantics — features do not "leak" across the FFI boundary, but the interface between Fusion and foreign code must respect the Fusion side's constraints.

### Cross-Feature Interaction Patterns

| Feature | FFI Interaction | Constraint |
|---------|----------------|------------|
| Effects | Foreign calls can declare effects | Use `effect [ForeignCall]` for untracked side effects |
| LinearTypes | Foreign-owned resources treated as linear | Must explicitly drop or transfer ownership |
| CapabilitySecurity | Foreign functions require capability proof | Capabilities cannot cross FFI boundary; use proxies |
| DependentTypes | Type contracts enforce array lengths | FFI declarations must use compile-time-known sizes |
| RefinementTypes | Postconditions verified for foreign returns | Add `where` clauses on FFI function signatures |
| Continuations | Foreign calls cannot capture continuations | Use callback-style interop for async foreign code |
| TCO | No impact on FFI calls | FFI calls are never in tail position |
| Coroutines | Foreign calls block the coroutine | Use `spawn_blocking` for long-running foreign calls |
| Actors | Foreign calls from actor handlers block the actor | Use dedicated actor pools for FFI-heavy workloads |
| FormalVerification | Cannot prove properties of foreign code | Use `axiom` declarations for foreign function contracts |
| TypeProviders | Can invoke foreign code at compile time | Type providers can call Python/Rust at build time |
| EffectRegions | Foreign calls in isolated regions | Wrap FFI in `region_isolated { ... }` blocks |
| UnsafeProvenance | Cannot track provenance across FFI | Raw pointers from FFI are untagged |
| TaintTracking | Foreign returns are initially untainted | Apply explicit `sanitize()` after FFI returns |
| GradualTyping | Foreign values have dynamic types | Use `as Type` casts for foreign values |
| CapabilityGate | Foreign calls bypass capability gates | Use `CapabilityProxy` for safe foreign delegation |

### Code Example: Effects + Linear Types

```fusion
use std::polyglot;
use std::io::{File, Write};

module data_pipeline;

uses: [Effects, LinearTypes];

#[polyglot::export(lang = "python")]
pub fn export_to_python(data: LinearBytes) -> Result<(), string>
    effect [PythonEval]
{
    // LinearTypes: data is consumed — Python gets ownership
    // Effects: this call may fail (PythonEval)
    let py_result = polyglot::eval("python", &format!(
        "import json; json.loads('{}')", data.to_string_lossy()
    ));

    match py_result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
    // data is consumed here — LinearTypes verified
}

#[polyglot::export(lang = "python")]
pub fn process_csv(path: string) -> Result<Vec<Vec<string>>, string>
    effect [FileRead, PythonEval]
{
    let file = File::open(path)?;                  // LinearTypes: file is linear
    let contents = file.read_to_string()?;          // file consumed here

    let result = polyglot::eval("python", &format!(
        "import csv; list(csv.reader('{}'.split('\\n')))", contents
    ))?;
    // contents consumed by Python — LinearTypes verified
    return Ok(polyglot::cast(result));
}
```

### Code Example: Capabilities + Actors

```fusion
use std::actor;

module secure_actor_system;

uses: [CapabilitySecurity, Actors];

pub struct DataWorker {
    db_cap: Capability<DatabaseRead>,
    cache_cap: Capability<CacheWrite>,
}

impl Actor for DataWorker {
    type Message = DataRequest;

    fn handle(&self, msg: DataRequest) -> DataResponse {
        // CapabilitySecurity: both capabilities are verified at compile time
        let cached = self.cache_cap.get(&msg.key);
        match cached {
            Some(data) => return DataResponse::Cached(data),
            None => {
                let data = self.db_cap.query(&msg.query)?;
                self.cache_cap.set(&msg.key, &data);
                return DataResponse::Fresh(data);
            }
        }
    }
}

fn spawn_worker_pool(count: int) -> Vec<ActorHandle<DataRequest>> {
    let mut handles = Vec::new();
    for _ in 0..count {
        let db_cap = acquire_capability(Authority::DatabaseRead).unwrap();
        let cache_cap = acquire_capability(Authority::CacheWrite).unwrap();
        let worker = DataWorker { db_cap, cache_cap };
        handles.push(actor::spawn(worker));
    }
    return handles;
}
```

### Code Example: Continuations + Coroutines

```fusion
module async_http_client;

uses: [Continuations, Coroutines];

pub fn fetch_all(urls: Vec<string>) -> Vec<Response>
    effect [NetworkRead]
{
    let mut responses = Vec::new();

    // Continuations: each fetch captures the rest of the computation
    // Coroutines: each fetch runs concurrently
    for url in urls {
        let response = spawn_coroutine(move || {
            // Continuations: if this were CPS, we could suspend here
            http_get(url)?
        });
        responses.push(response);
    }

    // Collect all coroutine results
    let mut results = Vec::new();
    for resp in responses {
        results.push(resp.await());
    }
    return results;
}

// Continuations allow early exit from within a coroutine
pub fn fetch_first_ok(urls: Vec<string>) -> Option<Response>
    effect [NetworkRead]
{
    let mut first_ok = None;

    reset {
        for url in urls {
            shift |k| {
                // If this fetch succeeds, invoke continuation
                // to stop processing further URLs
                spawn_coroutine(move || {
                    match http_get(url) {
                        Ok(resp) => {
                            first_ok = Some(resp);
                            k(());    // exit the loop early
                        }
                        Err(_) => {}  // continue to next URL
                    }
                });
            }
        }
    }

    return first_ok;
}
```

### Code Example: Dependent + Refinement Types

```fusion
module type_safe_buffer;

uses: [DependentTypes, RefinementTypes, Effects];

// DependentTypes: buffer type encodes its capacity
pub struct Buffer<n: usize> {
    data: [u8; n],                      // n is a compile-time value
    len: usize where len <= n,          // RefinementTypes: len is bounded
}

impl<n: usize> Buffer<n> {
    pub fn new() -> Buffer<n>
        where n > 0                     // RefinementTypes: n must be positive
    {
        Buffer {
            data: [0; n],
            len: 0,
        }
    }

    pub fn push(&mut self, byte: u8) -> Result<(), BufferFull>
        where self.len < n              // RefinementTypes: check before write
    {
        self.data[self.len] = byte;
        self.len += 1;
        return Ok(());
    }

    // DependentTypes: return type depends on input
    pub fn split_at(self, mid: usize) -> (Buffer<mid>, Buffer<n - mid>)
        where mid <= n
    {
        let left = Buffer {
            data: copy_slice(&self.data, 0, mid),
            len: mid,
        };
        let right = Buffer {
            data: copy_slice(&self.data, mid, n),
            len: n - mid,
        };
        return (left, right);
    }
}

// FFI integration with dependent types
#[polyglot::import(lang = "python")]
fn numpy_buffer(data: &[u8], length: usize) -> NumpyArray;

pub fn export_buffer(buf: Buffer<1024>) -> NumpyArray
    effect [PythonEval]
{
    // DependentTypes: the type system knows buf has exactly 1024 bytes
    return numpy_buffer(&buf.data, 1024);
}
```

### Code Example: Full Integration Test

```fusion
// integration_test.fu — Demonstrates all 16 features in a single program
// (Minus the 5 hard incompatibilities, which are excluded)

module integration_demo;

uses: [
    Effects,              // Effect tracking for all side effects
    RefinementTypes,      // Type narrowing at branches
    Coroutines,           // Structured concurrency
    Actors,               // Message-passing concurrency
    TypeProviders,        // Compile-time type generation
    EffectRegions,        // Isolate effect scopes
    TaintTracking,        // Track data flow
];

// TypeProviders: generates types from external schema at compile time
type ProviderConfig = type_provider!("config_schema.json");

// Effects: function declares its side effects
pub fn process_order(order: Order, config: ProviderConfig)
    effect [DatabaseWrite, NotificationSend, NetworkRead]
{
    // RefinementTypes: order.status is narrowed
    let validated = match order.status {
        Status::Pending => validate_order(order)?,
        Status::AlreadyProcessed => return,
        _ => unreachable!(),
    };

    // TaintTracking: user input is tainted
    let sanitized_input = sanitize(validated.input);  // input becomes clean
    let tainted_url = validated.external_url;          // still tainted

    // EffectRegions: isolate network calls
    let product_data = region_isolated {
        // Only NetworkRead allowed in this region
        http_get(tainted_url)?     // TaintTracking: URL is sanitized by region
    };

    // Coroutines: process items concurrently
    let mut handles = Vec::new();
    for item in validated.items {
        handles.push(spawn_coroutine(move || {
            // Actors: delegate to worker actors
            let worker = acquire_actor::<InventoryWorker>();
            worker.send(InventoryCheck::new(item))
        }));
    }

    let mut results = Vec::new();
    for h in handles {
        results.push(h.await());
    }

    // Effects: database write
    persist_order(&validated, &results)?;

    // Actors: notify asynchronously
    let notifier = acquire_actor::<NotificationActor>();
    notifier.send(Notification::OrderComplete {
        order_id: validated.id,
        items: results.len(),
    });
}

// RefinementTypes: postcondition guarantees
pub fn safe_divide(a: int, b: int) -> int
    where b != 0                              // precondition
    -> result: int where result * b == a      // postcondition
{
    return a / b;
}

// TaintTracking: data flow through the system
pub fn sanitize_tainted_data(raw: Tainted<string>) -> Clean<string> {
    let filtered = raw.filter(|c| c.is_alphanumeric());
    return Clean::new(filtered);               // transitions from Tainted to Clean
}

// EffectRegions: isolate unsafe operations
pub fn managed_unsafe(data: &[u8]) -> Result<ProcessedData, Error>
    effect [CryptoOp]                          // crypto operations tracked
{
    let key = acquire_capability(Authority::CryptoKey)?;
    region_isolated {
        // Only CryptoOp effects allowed here
        let encrypted = encrypt(key, data)?;
        let hash = sha256(&encrypted);
        return Ok(ProcessedData { encrypted, hash });
    }
}
```

---

## Cross-References

- **Chapter 8**: Quantum Computing for quantum FFI
- **Chapter 9**: Machine Learning for Python/Julia ML interop
- **Chapter 11**: WebAssembly for WASM-based interop
- **Chapter 17**: Fusion.toml Configuration for full config reference
- **Chapter 18**: Compiler Features for feature details
- **Chapter 15**: Reference for complete API signatures
