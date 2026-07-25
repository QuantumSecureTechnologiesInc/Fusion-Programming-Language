# Chapter 23: Pillar 5 — Interoperability & the Polyglot Ecosystem

> Foreign function interface, shared runtime protocol, polyglot API, type mapping, shared memory, unified toolchain, and guest/host boundaries

---

## Foreign Function Interface (FFI)

Fusion's FFI enables direct calls to functions compiled in other languages — primarily C, but extensible to any language with C ABI support.

### extern fn Declarations

```fusion
// Import C functions
extern "cdecl" {
    fn printf(fmt: *const u8, ...) -> int;
    fn malloc(size: u64) -> *mut u8;
    fn free(ptr: *mut u8);
    fn strlen(s: *const u8) -> u64;
}

// Import functions from a specific library
#[link(name = "m")]
extern "cdecl" {
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
}

// Static linking
#[link(name = "sqlite3", kind = "static")]
extern "cdecl" {
    fn sqlite3_open(path: *const u8, ppDb: *mut *mut u8) -> int;
    fn sqlite3_exec(db: *mut u8, sql: *const u8, callback: *mut u8, arg: *mut u8, errmsg: *mut *mut u8) -> int;
    fn sqlite3_close(db: *mut u8) -> int;
}
```

### Calling Conventions

Fusion supports multiple calling conventions for foreign functions:

| Convention | Keyword | Usage |
|-----------|---------|-------|
| C default | `"cdecl"` | Standard C libraries (Linux/macOS) |
| Windows default | `"stdcall"` | Windows API calls |
| Register-based | `"fastcall"` | Performance-critical calls |
| System V AMD64 | `"sysv"` | Linux/macOS x86-64 ABI |
| ARM64 | `"aarch64"` | ARM platform calls |

```fusion
// Windows API
extern "stdcall" {
    fn CreateFileA(
        lpFileName: *const u8,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *mut u8,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: *mut u8,
    ) -> *mut u8;

    fn CloseHandle(hObject: *mut u8) -> bool;
}

// Performance-critical register calling
extern "fastcall" {
    fn simd_add(a: *const f32, b: *const f32, out: *mut f32, len: u64);
}

// Platform-specific
extern "sysv" {
    fn pthread_create(
        thread: *mut u64,
        attr: *const u8,
        start_routine: fn(*mut u8) -> *mut u8,
        arg: *mut u8,
    ) -> int;
}
```

### C ABI Compatibility

Fusion types map directly to C types for seamless interop:

| Fusion Type | C Type | Size |
|------------|--------|------|
| `i8` / `u8` | `int8_t` / `uint8_t` | 1 byte |
| `i16` / `u16` | `int16_t` / `uint16_t` | 2 bytes |
| `i32` / `u32` | `int32_t` / `uint32_t` | 4 bytes |
| `i64` / `u64` | `int64_t` / `uint64_t` | 8 bytes |
| `f32` | `float` | 4 bytes |
| `f64` | `double` | 8 bytes |
| `bool` | `_Bool` | 1 byte |
| `*const T` | `const T*` | pointer |
| `*mut T` | `T*` | pointer |
| `[T; N]` | `T[N]` | array |

```fusion
// Structs match C memory layout with #[repr(C)]
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

#[repr(C)]
struct Matrix {
    rows: u32,
    cols: u32,
    data: *mut f64,
}

extern "cdecl" {
    fn mat_mul(a: *const Matrix, b: *const Matrix, result: *mut Matrix) -> int;
    fn point_distance(a: *const Point, b: *const Point) -> f64;
}
```

### Type Marshaling

```fusion
use std::ffi::{CString, CStr};

// Fusion string ↔ C string
fn call_c_with_string(input: &string) -> string {
    let c_input = CString::new(input.as_bytes()).unwrap();
    let c_result = unsafe { strlen(c_input.as_ptr()) };

    // C string → Fusion string
    let c_output = CString::new("result from C").unwrap();
    return c_output.to_string_lossy().to_string();
}

// Fusion Vec ↔ C array
fn process_c_array(data: &[f64]) -> Vec<f64> {
    let result = unsafe {
        let mut output: Vec<f64> = Vec::with_capacity(data.len());
        c_process_array(data.as_ptr(), output.as_mut_ptr(), data.len() as u64);
        output.set_len(data.len());
        output
    };
    return result;
}
```

---

## Shared Runtime Protocol

Fusion implements a Truffle-style polyglot protocol that enables seamless communication between languages running in the same process.

### Polyglot Protocol

```fusion
use std::polyglot::Runtime;

fn main() -> int {
    // Initialize the polyglot runtime
    let runtime = Runtime::new();

    // Register languages
    runtime.register_language("python", PythonEngine::new());
    runtime.register_language("javascript", JsEngine::new());
    runtime.register_language("rust", RustEngine::new());

    // Execute code in any registered language
    let py_result = runtime.eval("python", "2 + 2");
    let js_result = runtime.eval("javascript", "2 + 2");

    println("Python: %v", py_result);
    println("JavaScript: %v", js_result);

    return 0;
}
```

### Foreign Object Access

```fusion
use std::polyglot::{ForeignObject, PolyglotValue};

// Create a Python dictionary from Fusion
let py_dict = polyglot::eval("python", "
d = {'name': 'Alice', 'age': 30}
d
");

// Read properties from the foreign object
let name = py_dict.getattr("name").as_string();
let age = py_dict.getattr("age").as_int();
println("Name: %s, Age: %d", name, age);

// Modify foreign objects
py_dict.setattr("email", "alice@example.com".into());
```

### Property Reads / Writes

```fusion
// JavaScript object manipulation
let js_obj = polyglot::eval("javascript", "
    ({ x: 10, y: 20, method: function() { return this.x + this.y; } })
");

// Read property
let x = js_obj.get_property("x").as_int();
println("x = %d", x);

// Write property
js_obj.set_property("z", 30.into());

// Call method
let sum = js_obj.call_method("method", &[]).as_int();
println("sum = %d", sum);
```

### Method Calls Across Languages

```fusion
// Call Python methods from Fusion
let numpy = polyglot::import_module("numpy");
let arr = numpy.call_method("array", &[vec![1.0, 2.0, 3.0].into()]);
let mean = arr.call_method("mean", &[]).as_float();
println("Mean: %f", mean);

// Call Rust functions from Fusion
let json = polyglot::import_module("serde_json");
let data = json.call_method("to_string", &[my_struct.into()]);
println("JSON: %s", data.as_string());

// Call Java methods from Fusion
let jvm = polyglot::get_jvm();
let string_utils = jvm.call_static("org.apache.commons.lang3.StringUtils", "capitalize", &["hello".into()]);
println("Capitalized: %s", string_utils.as_string());
```

---

## Polyglot API

### import_value / eval

```fusion
use std::polyglot;

// Import a value from Python
let np = polyglot::import_value("numpy");
let arr = np.getattr("array").call(&[vec![1.0, 2.0, 3.0].into()]);

// Import a specific function
let factorial = polyglot::import_value("math", "factorial");
let result = factorial.call(&[5.into()]);
println("5! = %d", result.as_int());

// Import from JavaScript
let fs = polyglot::import_value("fs");
let content = fs.call_method("readFileSync", &["config.json".into(), "utf8".into()]);
```

### export_value

```fusion
use std::polyglot;

// Export a Fusion function to Python
#[polyglot::export(lang = "python")]
pub fn fibonacci(n: int) -> int {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// Export a Fusion struct to JavaScript
#[polyglot::export(lang = "javascript")]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[polyglot::export(lang = "javascript")]
impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        return Vector3 { x, y, z };
    }

    pub fn length(&self) -> f64 {
        return (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
    }

    pub fn add(&self, other: &Vector3) -> Vector3 {
        return Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        };
    }
}

// Export to Rust
#[polyglot::export(lang = "rust")]
pub fn process_data(input: Vec<u8>) -> Vec<u8> {
    return input.iter().map(|b| b ^ 0xFF).collect();
}
```

### eval() for Executing Foreign Code

```fusion
use std::polyglot;

fn main() -> int {
    // Execute Python code
    let py_result = polyglot::eval("python", "
import json
data = {'users': [{'name': 'Alice', 'age': 30}, {'name': 'Bob', 'age': 25}]}
json.dumps(data)
    ");
    println("JSON: %s", py_result.as_string());

    // Execute JavaScript code
    let js_result = polyglot::eval("javascript", "
const arr = [1, 2, 3, 4, 5];
arr.reduce((a, b) => a + b, 0)
    ");
    println("Sum: %d", js_result.as_int());

    // Execute Rust code
    let rs_result = polyglot::eval("rust", "
        let v: Vec<i64> = (1..=100).filter(|x| x % 2 == 0).sum();
        v
    ");
    println("Even sum: %d", rs_result.as_int());

    // Execute Java code
    let java_result = polyglot::eval("java", "
        java.time.Instant.now().toEpochMilli()
    ");
    println("Timestamp: %d", java_result.as_int());

    return 0;
}
```

### Unified Polyglot Dispatch

```fusion
use std::polyglot::{Dispatcher, Language};

// Unified dispatcher for calling functions across languages
struct PolyglotDispatcher {
    dispatchers: HashMap<string, Box<dyn Language>>,
}

impl PolyglotDispatcher {
    fn new() -> PolyglotDispatcher {
        return PolyglotDispatcher {
            dispatchers: HashMap::new(),
        };
    }

    fn register(&mut self, lang: string, engine: Box<dyn Language>) {
        self.dispatchers.insert(lang, engine);
    }

    fn call(&self, lang: &string, func: &string, args: Vec<PolyglotValue>) -> PolyglotValue {
        let engine = self.dispatchers.get(lang).unwrap();
        return engine.invoke(func, args);
    }
}

fn main() -> int {
    let mut dispatch = PolyglotDispatcher::new();
    dispatch.register("python", Box::new(PythonEngine::new()));
    dispatch.register("javascript", Box::new(JsEngine::new()));

    // Call the same logical function in different languages
    let py_result = dispatch.call(&"python".to_string(), &"math.sqrt".to_string(), vec![2.0.into()]);
    let js_result = dispatch.call(&"javascript".to_string(), &"Math.sqrt".to_string(), vec![2.0.into()]);

    println("Python sqrt(2): %f", py_result.as_float());
    println("JS sqrt(2): %f", js_result.as_float());

    return 0;
}
```

---

## Cross-Language Type Mapping

### Automatic Type Conversion Tables

Fusion automatically converts types at language boundaries. The tables below show the mappings.

### Primitive Type Mapping

**Fusion ↔ Python:**

| Fusion Type | Python Type | Direction | Notes |
|------------|-------------|-----------|-------|
| `int` | `int` | Bidirectional | Python has arbitrary precision |
| `float` | `float` | Bidirectional | Python float is f64 |
| `bool` | `bool` | Bidirectional | Direct mapping |
| `string` | `str` | Bidirectional | UTF-8 ↔ Python str |
| `bytes` | `bytes` | Bidirectional | Direct mapping |
| `()` | `None` | Bidirectional | Void ↔ None |

**Fusion ↔ JavaScript:**

| Fusion Type | JS Type | Direction | Notes |
|------------|---------|-----------|-------|
| `int` | `number` | Bidirectional | JS number is IEEE 754 f64 |
| `float` | `number` | Bidirectional | Direct |
| `bool` | `boolean` | Bidirectional | Direct |
| `string` | `string` | Bidirectional | UTF-8 ↔ JS string |
| `()` | `undefined` | Bidirectional | Void ↔ undefined |

**Fusion ↔ Java:**

| Fusion Type | Java Type | Direction | Notes |
|------------|-----------|-----------|-------|
| `int` | `long` | Bidirectional | Direct |
| `float` | `double` | Bidirectional | Direct |
| `bool` | `boolean` | Bidirectional | Direct |
| `string` | `String` | Bidirectional | UTF-8 ↔ Java String |

**Fusion ↔ Rust:**

| Fusion Type | Rust Type | Direction | Notes |
|------------|-----------|-----------|-------|
| `int` | `i64` | Bidirectional | Direct |
| `float` | `f64` | Bidirectional | Direct |
| `bool` | `bool` | Bidirectional | Direct |
| `string` | `String` | Bidirectional | UTF-8 owned |
| `&string` | `&str` | Bidirectional | Borrowed |

### Collection Type Mapping

| Fusion Type | Python | JavaScript | Java | Rust |
|------------|--------|------------|------|------|
| `Vec<T>` | `list` | `Array` | `ArrayList<T>` | `Vec<T>` |
| `HashMap<K,V>` | `dict` | `Object` | `HashMap<K,V>` | `HashMap<K,V>` |
| `Option<T>` | `T \| None` | `T \| undefined` | `T \| null` | `Option<T>` |
| `Result<T,E>` | `T \| Exception` | `T` (throws on Err) | `T` (throws on Err) | `Result<T,E>` |

### Conversion Functions

```fusion
use std::polyglot::convert;

fn main() -> int {
    // Explicit conversion functions
    let py_list = convert::to_python(&[1, 2, 3, 4, 5]);
    let fusion_vec: Vec<int> = convert::from_python(&py_list);

    let js_array = convert::to_javascript(&vec!["a", "b", "c"]);
    let fusion_strings: Vec<string> = convert::from_javascript(&js_array);

    // Automatic conversion at boundaries
    let np = polyglot::import_module("numpy");
    let arr = np.call_method("array", &[vec![1.0, 2.0, 3.0].into()]);
    // Fusion Vec<f64> automatically converts to Python list

    // Type coercion with explicit cast
    let py_int = polyglot::eval("python", "42");
    let fused: int = py_int.try_into().unwrap();  // explicit conversion

    return 0;
}
```

---

## Shared Memory & Reference Semantics

### Pass-by-Reference Across Languages

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

### Foreign Value Proxies

Foreign value proxies wrap non-Fusion objects so they can be used idiomatically in Fusion code:

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

### Shared Memory Regions

```fusion
use std::interop::{SharedRegion, SharedHandle};

fn main() -> int {
    // Create a named shared memory region
    let region = SharedRegion::create("model_weights", 1024 * 1024 * 100); // 100MB

    // Write data from Fusion
    let weights: Vec<f32> = train_model();
    region.write_slice(0, &weights);

    // Share with Python — zero copy via memory mapping
    let py_handle = region.share_to("python");
    polyglot::eval("python", &format("
import numpy as np
weights = np.frombuffer(shared_region, dtype=npfloat32)
normalized = weights / np.max(np.abs(weights))
print('Normalized shape:', normalized.shape)
    "));

    // Share with Rust — direct pointer access
    let rs_handle = region.share_to("rust");
    polyglot::eval("rust", "
        let weights: &[f32] = unsafe { std::slice::from_raw_parts(ptr, len) };
        let sum: f32 = weights.iter().sum();
    ");

    // Both languages see the same memory
    println("Weights shared across %d languages", region.attached_count());

    return 0;
}
```

### Mutation Visibility Across Languages

```fusion
use std::interop::{SharedState, MutationObserver};

struct SharedCounter {
    state: SharedState<int>,
}

impl SharedCounter {
    fn new() -> SharedCounter {
        return SharedCounter {
            state: SharedState::new(0),
        };
    }

    fn increment(&self) {
        self.state.modify(|v| *v = *v + 1);
    }

    fn get(&self) -> int {
        return self.state.read();
    }
}

fn main() -> int {
    let counter = SharedCounter::new();

    // Fusion increments
    counter.increment();
    counter.increment();

    // Python sees the update
    let py_val = polyglot::eval("python", &format("
counter_value = {}  # shared state is visible
counter_value
    ", counter.get()));
    println("Python sees: %d", py_val.as_int());

    // Python can modify shared state
    polyglot::eval("python", "
# Mutation is visible to Fusion
shared_counter_state += 5
    ");

    // Fusion sees Python's mutation
    println("After Python: %d", counter.get());

    return 0;
}
```

---

## Unified Toolchain

### Single Launcher (fusion CLI)

The `fuc` command is the unified entry point for all Fusion operations:

```bash
# Create a new project
fuc new my-project

# Build the project
fuc build

# Build in release mode
fuc build --release

# Run the project
fuc run

# Run tests
fuc test

# Run benchmarks
fuc bench

# Check code without building
fuc check

# Format code
fuc fmt

# Lint code
fuc lint

# Start REPL
fuc repl

# Run with polyglot support
fuc run --polyglot python,js

# Build for a specific target
fuc build --target wasm32-unknown-unknown
```

### Cross-Language Build System (Forge)

Forge handles building, linking, and dependency resolution for polyglot projects:

```bash
# Initialize a polyglot project
forge init --polyglot

# Build all languages
forge build

# Build specific language targets
forge build --lang fusion,rust

# Run cross-language tests
forge test

# Clean build artifacts
forge clean

# Show dependency tree
forge tree

# Update all dependencies
forge update
```

```toml
# forge.toml — Cross-language build configuration
[build]
languages = ["fusion", "rust", "python"]
targets = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]

[dependencies]
rust_native = { rust = { path = "./rust/native" } }
python_ext = { python = { package = "my-package", version = "1.0" } }

[test]
languages = ["fusion", "rust"]
parallel = true
```

### Cross-Language Debugging

```bash
# Start debugger with polyglot support
fuc debug --polyglot

# Attach to running process
fuc debug --attach <pid>

# Remote debugging
fuc debug --remote localhost:9229
```

```fusion
// Debug annotations for cross-language boundaries
#[debug(breakpoint)]
fn process_in_python(data: Vec<f64>) -> f64 {
    // Debugger will break here and show Fusion variables
    // Can step into Python code
    let result = polyglot::eval("python", "
import numpy as np
np.mean(data)
    ");
    return result.as_float();
}

#[debug(skip)]  // Skip this function during debugging
fn internal_helper() -> string {
    return "internal".to_string();
}
```

### Package Manager for Polyglot Projects

```bash
# Add a Python dependency
fuc dep add python:numpy>=1.20

# Add a Rust dependency
fuc dep add rust:serde=1.0

# Add a JavaScript dependency
fuc dep add npm:lodash=^4.17

# Add a C library dependency
fuc dep add c:libxml2

# List all dependencies
fuc dep list

# Update all dependencies
fuc dep update

# Remove a dependency
fuc dep remove numpy
```

---

## Guest/Host Boundaries

### Host vs Guest Language Definitions

| Aspect | Host (Fusion) | Guest (Python/JS/Java) |
|--------|---------------|----------------------|
| Execution | Native code | Interpreted / JIT |
| Memory | Manual / ownership | GC-managed |
| Types | Static, checked | Dynamic |
| Concurrency | Async/Await, threads | GIL, event loop |
| Error handling | Result/Option | Exceptions |
| Performance | Near C | Variable |

```fusion
// Host language (Fusion) manages lifecycle
fn main() -> int {
    // Initialize guest runtimes
    let py = polyglot::init_python();
    let js = polyglot::init_javascript();

    // Guest code runs within host-managed boundaries
    let result = py.eval("
import numpy as np
np.array([1, 2, 3]).mean()
    ");

    // Host enforces memory safety across boundaries
    let data = result.as_float();

    // Cleanup guest resources
    py.shutdown();
    js.shutdown();

    println("Result: %f", data);
    return 0;
}
```

### Cross-Language Error Handling

```fusion
use std::polyglot::Error;

// Guest exceptions become Fusion Result types
fn safe_python_eval(code: &string) -> Result<PolyglotValue, string> {
    return polyglot::eval_result("python", code);
}

fn main() -> int {
    // Handle Python exceptions as Fusion Results
    match safe_python_eval("1 / 0") {
        Ok(val) => println("Result: %v", val),
        Err(msg) => println("Python error: %s", msg),
    }

    // Propagate errors across boundaries
    let result: Result<int, string> = polyglot::eval("python", "
try:
    import nonexistent_module
    42
except ImportError as e:
    raise Exception(f'Module error: {e}')
    ".into());

    match result {
        Ok(val) => println("Value: %d", val),
        Err(e) => println("Caught: %s", e),
    }

    return 0;
}
```

### Cross-Language Concurrency Model

```fusion
use std::polyglot::{ThreadPool, JsRuntime, JvmHandle};

fn main() -> int {
    // Python thread pool
    let py_pool = ThreadPool::new("python", 4);
    let mut py_handles = Vec::new();

    for i in 0..10 {
        let h = py_pool.submit(move || {
            polyglot::eval("python", &format!("
import time
time.sleep(0.1)
{}
            ", i * 10))
        });
        py_handles.push(h);
    }

    // JavaScript event loop
    let js = JsRuntime::new();
    let future = js.eval_async("
        new Promise((resolve) => {
            setTimeout(() => resolve(42), 100);
        })
    ");

    // Collect Python results
    for h in py_handles {
        let result = h.await();
        println("Python: %s", result.as_string());
    }

    // Await JavaScript promise
    let js_result = future.await();
    println("JS: %d", js_result.as_int());

    return 0;
}
```

### Capability Restrictions

```fusion
// Restrict what guest languages can access
#[polyglot::capability(
    file_access = "read-only",
    network_access = false,
    env_access = false,
    process_access = false,
)]
extern "python" {
    fn safe_function(input: string) -> string;
}

// Sandbox execution
fn main() -> int {
    let sandbox = polyglot::Sandbox::new()
        .allow_file_read("/data")
        .allow_network(false)
        .allow_env(false)
        .allow_process(false)
        .build();

    let result = sandbox.eval("python", "
import os
os.listdir('/data')  # allowed
os.system('rm -rf /')  # blocked by capability check
    ");

    match result {
        Ok(val) => println("Result: %v", val),
        Err(e) => println("Blocked: %s", e),
    }

    return 0;
}
```

---

## Code Examples

### Calling Python from Fusion

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

### Calling Rust from Fusion

```fusion
use std::polyglot;

// Import Rust crates
#[polyglot::import(lang = "rust")]
mod serde_json;

#[polyglot::import(lang = "rust")]
mod rayon;

// Export Fusion data to Rust for processing
#[polyglot::export(lang = "rust")]
pub fn parallel_process(data: Vec<f64>) -> Vec<f64> {
    return data.par_iter().map(|x| x.sqrt() * 2.0).collect();
}

fn main() -> int {
    // Use Rust's serde for JSON
    let config = Config::new();
    let json = serde_json::to_string_pretty(&config)?;
    println("Config: %s", json);

    // Use Rust's rayon for parallel processing
    let data: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();
    let processed = parallel_process(data);
    println("Processed %d items", processed.len());

    return 0;
}
```

### Calling JavaScript from Fusion

```fusion
use std::polyglot;

fn main() -> int {
    // Use JavaScript for web-related tasks
    let fetch = polyglot::eval("javascript", "
async function fetchData(url) {
    const response = await fetch(url);
    return await response.json();
}
fetchData('https://api.example.com/data')
    ");

    let data = fetch.await();
    println("Data: %s", data.as_string());

    // Use JavaScript for string manipulation
    let result = polyglot::eval("javascript", "
'Hello, World!'.split('').reverse().join('')
    ");
    println("Reversed: %s", result.as_string());

    // Use JavaScript Date API
    let timestamp = polyglot::eval("javascript", "
new Date().toISOString()
    ");
    println("Time: %s", timestamp.as_string());

    return 0;
}
```

### Calling Java from Fusion

```fusion
use std::polyglot;

fn main() -> int {
    // Get or create JVM instance
    let jvm = polyglot::get_jvm();

    // Use Java's built-in classes
    let uuid = polyglot::eval("java", "
java.util.UUID.randomUUID().toString()
    ");
    println("UUID: %s", uuid.as_string());

    // Use Java collections
    let result = polyglot::eval("java", "
var list = new java.util.ArrayList();
list.add(1);
list.add(2);
list.add(3);
list.stream().mapToInt(i -> i * 2).sum()
    ");
    println("Java sum: %d", result.as_int());

    // Use Java's date/time API
    let now = polyglot::eval("java", "
java.time.LocalDateTime.now().format(
    java.time.format.DateTimeFormatter.ofPattern('yyyy-MM-dd HH:mm:ss')
)
    ");
    println("Java time: %s", now.as_string());

    // Run Java code in separate thread
    let handle = jvm.spawn_thread(|| {
        polyglot::eval("java", "
Thread.currentThread().getName()
        ")
    });
    let thread_name = handle.await();
    println("Java thread: %s", thread_name);

    return 0;
}
```

### Shared Memory Between Languages

```fusion
use std::interop::{SharedBuffer, SharedRegion};

fn main() -> int {
    // Create shared memory for model weights
    let region = SharedRegion::create("ml_weights", 1024 * 1024 * 10);

    // Fusion writes initial weights
    let weights: Vec<f32> = (0..1000).map(|i| i as f32 * 0.001).collect();
    region.write_slice(0, &weights);

    // Python reads and processes — zero copy
    polyglot::eval("python", &format("
import numpy as np
weights = np.frombuffer(shared_region, dtype=npfloat32)
normalized = weights / np.linalg.norm(weights)
print(f'Python processed {{len(normalized)}} weights')
    "));

    // Rust processes the same memory — zero copy
    polyglot::eval("rust", "
        let weights: &[f32] = unsafe { std::slice::from_raw_parts(ptr, len) };
        let sum: f32 = weights.iter().sum();
        println!(\"Rust sum: {}\", sum);
    ");

    // JavaScript accesses via ArrayBuffer
    polyglot::eval("javascript", "
const buffer = new Float32Array(shared_region);
const sum = buffer.reduce((a, b) => a + b, 0);
console.log('JS sum:', sum);
    ");

    // All languages see the same data without copying
    println("Shared memory used by %d languages", region.attached_count());

    return 0;
}
```
