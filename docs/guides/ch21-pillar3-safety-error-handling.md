# Chapter 21: Pillar 3 — Safety, Reliability & Error Handling (The Airbags)

> Why a language that crashes silently is worse than one that refuses to compile — and how Fusion v2.0 Vortex makes bugs structurally impossible.

---

## Introduction

A car without airbags still drives. A language without safety still runs programs. But both are playing Russian roulette with every trip. Pillar 3 is Fusion v2.0 Vortex's answer to the question: *what happens when things go wrong?*

The answer: **the compiler catches them before they go wrong**, and when runtime errors are unavoidable, Fusion handles them with structured recovery instead of undefined behavior.

This pillar covers the type system, null/nil safety, memory safety, integer behavior, error handling, and security guarantees that make Fusion a language you can trust with critical infrastructure.

---

## Type System

### Static Typing with Inference

Fusion is statically typed — every expression has a known type at compile time. But you rarely need to write annotations because the compiler infers them:

```fusion
let x = 42;              // Int (inferred)
let y = 3.14;            // Float (inferred)
let name = "Fusion";     // String (inferred)
let items = vec![1, 2];  // Vec<Int> (inferred)

// Explicit annotation when needed
let parsed: Option<Int> = "42".parse();
let coords: (Float, Float) = (1.0, 2.0);
```

**Type inference rules:**
- Literals are typed by their form: `42` → `Int`, `3.14` → `Float`, `"hello"` → `String`
- Function return types are inferred from body
- Generic types are inferred from usage
- Closure parameter types can be inferred from first use

### Type Soundness Guarantees

Fusion's type system is **sound** — if the program compiles, it satisfies the type contracts:

```fusion
// These are compile errors, not runtime crashes
let x: Int = "hello";           // Error: expected Int, found String
let v: Vec<Int> = vec![1, "2"]; // Error: mixed types in vec!
let f: fn(Int) -> Int = |x| x.to_string(); // Error: wrong return type
```

**What type soundness means:**
- A function declared as `fn(Int) -> Int` will always return an `Int`
- A `Vec<Int>` will always contain `Int` values
- A reference `&T` will always point to a valid `T`
- Pattern matching is exhaustive — no missed cases

### Gradual Typing Support

When integrating with dynamic languages or scripting, Fusion supports gradual typing:

```fusion
// Dynamic type for polyglot interop
let dynamic_value: Dynamic = get_from_python();

// Type-checked at runtime
let result: Int = dynamic_value.try_into()?;

// Or force with runtime check
let result: Int = dynamic_value.downcast::<Int>()
    .ok_or("Expected Int")?;
```

**Gradual typing rules:**
- `Dynamic` bypasses static checks — use sparingly
- Runtime type checks occur at boundaries
- Gradual typing is for interop, not internal logic
- Statically typed code remains sound

### Refinement Types

Refinement types add constraints to base types — types that depend on values:

```fusion
// Positive integers only
type PositiveInt = {x: Int | x > 0};

// Bounded values
type Percentage = {x: Float | x >= 0.0 && x <= 100.0};

// Non-empty strings
type NonEmptyString = {s: String | !s.is_empty()};

// Valid email (regex refinement)
type Email = {s: String | s.matches(r"^[^@]+@[^@]+\.[^@]+$")};

fn divide(a: Int, b: {x: Int | x != 0}) -> Float {
    a as Float / b as Float
}

fn main() {
    let result = divide(10, 5);        // OK
    // let result = divide(10, 0);     // Compile error: 0 violates {x | x != 0}

    let pct: Percentage = 75.0;        // OK
    // let pct: Percentage = 150.0;    // Compile error: 150.0 > 100.0
}
```

### Dependent Types

Types that depend on runtime values — the most powerful type-level programming:

```fusion
// Vector with compile-time-known length
struct Vec<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> Vec<T, N> {
    fn new() -> Self { /* ... */ }

    fn push(self, value: T) -> Vec<T, {N + 1}> {
        // Return type changes!
        /* ... */
    }

    fn len(&self) -> usize {
        N  // Length is part of the type
    }
}

// Matrix multiplication with verified dimensions
fn matmul<A, B, C>(
    a: &Matrix<A, const M: usize, const N: usize>,
    b: &Matrix<B, const N: usize, const P: usize>,
) -> Matrix<Output, M, P> {
    // Dimensions are verified at compile time:
    // A is M×N, B is N×P, result is M×P
    // If dimensions don't match, it won't compile
}

fn main() {
    let v3: Vec<Int, 3> = Vec::new();
    let v4 = v3.push(4);  // Vec<Int, 4> — length changed in type

    let a = Matrix::<Float, 3, 4>::new();  // 3×4 matrix
    let b = Matrix::<Float, 4, 2>::new();  // 4×2 matrix
    let c = matmul(&a, &b);                // 3×2 matrix — dimensions verified

    // let bad = matmul(&a, &a);  // Compile error: 4 ≠ 3
}
```

**Dependent type capabilities:**
- Array lengths as types
- Matrix dimensions as types
- State machines as types (compile-time state tracking)
- Protocol verification (type-level proof of correct usage)

---

## Null/Nil Safety

### `Option<T>` for Absent Values

Fusion has no `null`, `nil`, `undefined`, or `None` as a language-level concept. Instead, it uses `Option<T>`:

```fusion
fn find_user(id: Int) -> Option<User> {
    // Returns Some(user) if found, None if not
}

fn main() {
    let user = find_user(42);

    // Must handle both cases
    match user {
        Some(u) => print("Found: {u.name}"),
        None => print("User not found"),
    }
}
```

**You cannot accidentally use an absent value:**

```fusion
let user = find_user(42);

// These are all compile errors:
// print(user.name);           // Error: Option has no field 'name'
// let name = user.unwrap();   // Error: unwrap is unsafe, explicit opt-in required

// You must handle it:
let name = match user {
    Some(u) => u.name,
    None => "Unknown".to_string(),
};
```

### `Result<T, E>` for Errors

Recoverable errors use `Result<T, E>`:

```fusion
fn read_config(path: &str) -> Result<Config, IoError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn main() {
    match read_config("app.toml") {
        Ok(config) => print("Config loaded: {config.name}"),
        Err(e) => print("Failed to load config: {e}"),
    }
}
```

### No Null Pointers

Fusion eliminates null pointer dereferences entirely:

```fusion
// These concepts don't exist in Fusion:
// - null pointer
// - null reference
// - nil
// - undefined
// - uninitialized memory

// Instead, every potentially-absent value is wrapped:
let ptr: Option<&Int> = Some(&42);  // Possibly absent
let ptr: Option<&Int> = None;       // Definitely absent

// You must check before using:
if let Some(value) = ptr {
    print("Value: {value}");
}

// Or use the ? operator to propagate:
fn process(input: &str) -> Result<Int, Error> {
    let parsed: Option<Int> = input.parse().ok()?;
    Ok(parsed * 2)
}
```

### Optional Chaining

```fusion
struct Company {
    ceo: Option<Person>,
}

struct Person {
    address: Option<Address>,
}

struct Address {
    city: String,
}

fn get_ceo_city(company: &Company) -> Option<String> {
    company.ceo?.address?.city.clone()
}

// Equivalent to:
fn get_ceo_city_verbose(company: &Company) -> Option<String> {
    match &company.ceo {
        Some(person) => match &person.address {
            Some(addr) => Some(addr.city.clone()),
            None => None,
        },
        None => None,
    }
}
```

### Default Values

```fusion
// Default trait
impl Default for Config {
    fn default() -> Self {
        Self {
            name: "app".to_string(),
            port: 8080,
            debug: false,
        }
    }
}

// Or operator
let config = user_config.unwrap_or_default();

// Or with custom default
let name = user_name.unwrap_or_else(|| "Anonymous".to_string());
```

---

## Memory Safety

### Ownership and Borrowing (Compile-Time Prevention)

The Vortex borrow checker prevents memory bugs at compile time:

```fusion
// Use-after-free: IMPOSSIBLE
fn use_after_free() {
    let x;
    {
        let y = 42;
        x = &y;  // Compile error: `y` does not live long enough
    }
    // print(x);  // Would be use-after-free — blocked by compiler
}

// Double-free: IMPOSSIBLE
fn double_free() {
    let s = String::from("hello");
    let s2 = s;  // Ownership moved
    // drop(s);  // Compile error: use of moved value
    drop(s2);    // OK
}

// Dangling pointer: IMPOSSIBLE
fn dangling_pointer() -> &str {
    let s = String::from("hello");
    &s  // Compile error: returns reference to local variable
    // s is dropped when function returns, reference would dangle
}
```

### Use-After-Free Prevention

```fusion
struct Buffer {
    data: Vec<u8>,
    position: usize,
}

impl Buffer {
    fn read(&mut self) -> Option<u8> {
        if self.position < self.data.len() {
            let byte = self.data[self.position];
            self.position += 1;
            Some(byte)
        } else {
            None
        }
    }
}

fn process() {
    let mut buf = Buffer { data: vec![1, 2, 3], position: 0 };
    let first = buf.read();  // Mutable borrow of buf

    // Cannot use buf while borrowed:
    // buf.read();  // Compile error: cannot borrow `buf` as mutable
    //              // because it is also borrowed as immutable

    print(first);  // First borrow used
    buf.read();    // OK — first borrow ended
}
```

### Double-Free Prevention

```fusion
struct Resource {
    handle: RawHandle,
}

impl Drop for Resource {
    fn drop(&mut self) {
        unsafe { close_handle(self.handle); }
        print("Resource dropped");
    }
}

fn main() {
    let r = Resource { handle: 42 };
    let r2 = r;  // Ownership moved

    // r is no longer valid — only r2 will be dropped
    // No double-free possible

    drop(r2);    // Explicitly drop
    // r2 is no longer valid

    // r2.drop();  // Compile error: use of moved value
}
```

### Buffer Overflow Prevention

```fusion
fn safe_array_access() {
    let arr = [1, 2, 3, 4, 5];

    // Bounds checking at compile time (when index is constant)
    let value = arr[2];  // OK — index 2 is valid
    // let value = arr[10];  // Compile error: index out of bounds

    // Runtime bounds checking (when index is dynamic)
    let index: usize = get_index();
    match arr.get(index) {
        Some(value) => print("Value: {value}"),
        None => print("Index out of bounds"),
    }

    // Safe slicing
    let slice = &arr[1..4];  // Compile error if range invalid
    // Or runtime-checked:
    let slice = arr.get(1..4).unwrap_or(&[]);
}
```

### Dangling Pointer Prevention

```fusion
struct Node {
    value: Int,
    next: Option<Box<Node>>,
}

impl Node {
    fn new(value: Int) -> Self {
        Self { value, next: None }
    }

    fn append(&mut self, value: Int) {
        match &mut self.next {
            Some(next) => next.append(value),
            None => {
                self.next = Some(Box::new(Node::new(value)));
            }
        }
    }
}

fn main() {
    let mut head = Node::new(1);
    head.append(2);
    head.append(3);

    // All nodes are Box<Node> — heap allocated, ownership tracked
    // When head goes out of scope, all nodes are dropped in order
    // No dangling pointers possible
}
```

### Linear Types for Resource Protocols

```fusion
// Session types — type-safe communication protocols
enum Session {
    Start,
    Authenticated { token: Token },
    Closed,
}

struct Connection {
    state: Session,
}

impl Connection {
    fn authenticate(self, credentials: &Credentials) -> Result<Connection, AuthError> {
        match self.state {
            Session::Start => {
                let token = verify credentials?;
                Ok(Connection { state: Session::Authenticated { token } })
            }
            _ => Err(AuthError::InvalidState),
        }
    }

    fn send_data(self, data: &[u8]) -> Result<Connection, IoError> {
        match self.state {
            Session::Authenticated { token } => {
                // Send data with token
                Ok(self)
            }
            _ => Err(IoError::NotAuthenticated),
        }
    }

    fn close(self) {
        // Connection is consumed — cannot be used after close
        // This is guaranteed by the type system
    }
}

fn main() {
    let conn = Connection { state: Session::Start };

    // Must authenticate before sending
    // let conn = conn.send_data(b"hello");  // Compile error: wrong state

    let conn = conn.authenticate(&credentials)?;  // Now authenticated
    let conn = conn.send_data(b"hello")?;         // OK
    conn.close();                                   // Consumed
    // conn.send_data(b"more");  // Compile error: use after close
}
```

### Unsafe Blocks with Provenance Proofs

When you need to break the rules, Fusion requires explicit unsafe blocks with provenance:

```fusion
fn unsafe_example() {
    let mut x = 42;

    // Unsafe block — compiler trusts your claims
    let ptr = unsafe {
        // Must provide provenance proof
        std::ptr::addr_of_mut!(x)
    };

    // The compiler tracks that `ptr` was derived from `x`
    // and enforces rules accordingly

    unsafe {
        ptr.write(100);
    }

    print(x);  // 100

    // Unsafe does NOT disable borrow checking
    // It only allows dereferencing raw pointers
}
```

**Unsafe blocks allow:**
- Dereferencing raw pointers
- Calling unsafe functions
- Accessing mutable statics
- Implementing unsafe traits
- Accessing union fields

**Unsafe blocks do NOT allow:**
- Ignoring borrow rules
- Bypassing lifetime checks
- Undefined behavior (still a compile error)
- Data races (still detected)

---

## Integer Behavior

### Checked Arithmetic (Overflow Panics)

By default, integer overflow causes a panic:

```fusion
fn checked_addition() {
    let x: Int = Int::MAX;
    // let y = x + 1;  // Panics at runtime: integer overflow

    // Use checked methods for explicit handling
    let result = x.checked_add(1);
    match result {
        Some(y) => print("Result: {y}"),
        None => print("Overflow detected!"),
    }
}
```

### Wrapping Arithmetic

```fusion
fn wrapping_example() {
    let x: u8 = 255;
    let y = x.wrapping_add(1);
    print(y);  // 0 — wraps around

    let z = x.wrapping_mul(2);
    print(z);  // 254 — wraps
}
```

### Saturating Arithmetic

```fusion
fn saturating_example() {
    let x: u8 = 200;
    let y = x.saturating_add(100);
    print(y);  // 255 — saturates at max

    let z = x.saturating_sub(250);
    print(z);  // 0 — saturates at min
}
```

### Configurable Per-Operation

```fusion
// At the function level
#[arithmetic_mode = "checked"]
fn safe_math(a: Int, b: Int) -> Int {
    a + b  // Panics on overflow
}

#[arithmetic_mode = "wrapping"]
fn network_protocol(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)  // Wraps on overflow
}

#[arithmetic_mode = "saturating"]
fn image_processing(a: u8, b: u8) -> u8 {
    a.saturating_add(b)  // Saturates at max
}

// Or per-operation inline
fn mixed() {
    let a = 100_i32;
    let b = 200_i32;

    // Checked (panics on overflow)
    let sum = a.checked_add(b).unwrap();

    // Wrapping (wraps on overflow)
    let product = a.wrapping_mul(b);

    // Saturating (clamps on overflow)
    let clamped = a.saturating_add(b);
}
```

---

## Error Handling

### Recoverable: `Result<T, E>` with Pattern Matching

```fusion
use std::io;
use std::fmt;

enum AppError {
    Io(io::Error),
    Parse(String),
    Validation { field: String, reason: String },
    NotFound { resource: String, id: Int },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Parse(msg) => write!(f, "Parse error: {msg}"),
            Self::Validation { field, reason } => {
                write!(f, "Validation failed for '{field}': {reason}")
            }
            Self::NotFound { resource, id } => {
                write!(f, "{resource} with id={id} not found")
            }
        }
    }
}

fn load_user(id: Int) -> Result<User, AppError> {
    let path = format!("users/{id}.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Io(e))?;

    let user: User = serde_json::from_str(&content)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    if user.name.is_empty() {
        return Err(AppError::Validation {
            field: "name".to_string(),
            reason: "Name cannot be empty".to_string(),
        });
    }

    Ok(user)
}

fn main() {
    match load_user(42) {
        Ok(user) => print("Loaded: {user.name}"),
        Err(AppError::NotFound { resource, id }) => {
            print("Could not find {resource} #{id}");
        }
        Err(AppError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {
            print("File not found");
        }
        Err(e) => {
            print("Unexpected error: {e}");
        }
    }
}
```

### The `?` Operator

```fusion
fn process_file(path: &str) -> Result<Config, AppError> {
    let content = std::fs::read_to_string(path)?;   // ? propagates Io error
    let config: Config = toml::from_str(&content)?;  // ? propagates Parse error
    validate_config(&config)?;                        // ? propagates Validation error
    Ok(config)
}

// ? works with any type that implements Try
// It converts the error type and returns early
```

### Unrecoverable: `panic` and `abort`

```fusion
fn critical_failure() {
    // Panic — unwinds the stack, calls destructors
    panic!("This should never happen: {details}");

    // Abort — immediately terminates, no cleanup
    std::process::abort();
}

// Set panic behavior globally
#[panic = "abort"]  // Panics become aborts
fn main() {
    // ...
}

// Or per-thread
fn supervised_thread() {
    let result = std::panic::catch_unwind(|| {
        risky_operation()
    });

    match result {
        Ok(value) => print("Success: {value}"),
        Err(_) => print("Thread panicked, but we recovered"),
    }
}
```

### Resource Cleanup: RAII, `defer`, `with_resource`

#### RAII (Resource Acquisition Is Initialization)

```fusion
struct FileGuard {
    handle: RawFd,
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        unsafe { close(self.handle); }
        print("File closed: fd={}", self.handle);
    }
}

fn process_file() {
    let file = FileGuard { handle: unsafe { open("data.txt") } };
    // Read/write operations
    // file is automatically dropped when function returns
    // No explicit close needed
}
```

#### `defer` for Cleanup Actions

```fusion
fn with_defer() {
    let mut resource = acquire_resource();

    defer! {
        release_resource(&mut resource);
        print("Resource released");
    }

    // Do work with resource
    // ...

    // resource is released automatically, even on error/panic
}
```

#### `with_resource` for Scoped Resources

```fusion
fn with_resource_example() {
    with_resource(
        || acquire_database_connection(),
        |conn| {
            // Use connection
            conn.execute("SELECT * FROM users")?;
            // Connection automatically returned/closed after this block
            Ok(())
        },
    )
}
```

### try/catch Pattern

```fusion
fn risky_operation() -> Result<Int, Error> {
    // ...

    // Catch panics and convert to Result
    let result = std::panic::catch_unwind(|| {
        operation_that_may_panic()
    });

    match result {
        Ok(value) => Ok(value),
        Err(panic) => {
            let msg = panic.downcast_ref::<String>()
                .unwrap_or(&"Unknown panic".to_string());
            Err(Error::from(format!("Operation panicked: {msg}")))
        }
    }
}
```

### Stack Traces

```fusion
// Automatic stack traces on panic
fn main() {
    // When a panic occurs, Fusion prints:
    //
    // thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5'
    //   at src/main.rs:42:10
    //
    // Stack backtrace:
    //   0: main::process_data
    //      at src/main.rs:42:10
    //   1: main::main
    //      at src/main.rs:10:5
    //   2: std::rt::lang_start
    //      at library/std/src/rt.rs:166:17

    // Or capture programmatically
    let backtrace = std::backtrace::Backtrace::capture();
    print!("{backtrace}");
}
```

### Assert Macros

```fusion
fn validated_process(data: &[Int]) {
    assert!(!data.is_empty(), "Data must not be empty");
    assert!(data.len() <= 1000, "Data too large: {}", data.len());

    let sum: Int = data.iter().sum();
    assert_eq!(sum, data.iter().sum::<Int>(), "Sum mismatch");

    // Debug-only assertions (stripped in release builds)
    debug_assert!(data.iter().all(|&x| x > 0), "All values must be positive");

    // Custom assertion with formatting
    assert!(
        data.len().is_power_of_two(),
        "Expected power of 2, got {}",
        data.len()
    );
}
```

---

## Security

### Capability-Based Security

Fusion uses capabilities to restrict what code can do:

```fusion
// Declare capabilities
#[capability = "filesystem:read"]
fn read_data(path: &str) -> Result<String, CapError> { /* ... */ }

#[capability = "filesystem:write"]
fn write_data(path: &str, data: &str) -> Result<(), CapError> { /* ... */ }

#[capability = "network:connect"]
fn fetch_url(url: &str) -> Result<Response, CapError> { /* ... */ }

// Capabilities are checked at compile time and runtime
fn main() {
    // Code without the capability gets a runtime error
    // read_data("secret.txt");  // Requires filesystem:read capability

    // Capabilities are granted by the runtime/sandbox
    let config = CapabilityConfig::new()
        .grant("filesystem:read", &["/data/*", "/config/*"])
        .grant("network:connect", &["api.example.com"]);

    let sandbox = Sandbox::new(config);
    sandbox.execute(|| {
        // Within the sandbox, capabilities are available
        let data = read_data("/config/app.toml")?;
        let response = fetch_url("https://api.example.com/data")?;
        Ok(())
    });
}
```

### Sandboxed Execution

```fusion
use std::sandbox::{Sandbox, Policy};

fn run_untrusted_code(code: &str) -> Result<String, SandboxError> {
    let policy = Policy::new()
        .memory_limit(64 * 1024 * 1024)   // 64 MB
        .cpu_time_limit(Duration::from_secs(5))
        .no_filesystem_access()
        .no_network_access()
        .no_process_creation();

    let sandbox = Sandbox::new(policy);

    let result = sandbox.evaluate(code)?;
    Ok(result.to_string())
}

// WASM sandboxing
fn run_in_wasm_sandbox(wasm_bytes: &[u8]) -> Result<String, SandboxError> {
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, wasm_bytes)?;

    // Minimal WASI capabilities
    let wasi = wasmtime_wasi::WasiCtxBuilder::new()
        .inherit_stdio()
        .build();

    let mut store = wasmtime::Store::new(&engine, wasi);
    let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

    // Execute with time/memory limits
    let result = instance
        .get_typed_func::<(), ()>(&mut store, "_start")?
        .call(&mut store, ())?;

    Ok("Execution completed".to_string())
}
```

### Memory Safety Guarantees

```fusion
// Fusion guarantees:
//
// ✓ No use-after-free (ownership + borrow checking)
// ✓ No double-free (linear types + drop check)
// ✓ No buffer overflow (bounds checking)
// ✓ No dangling pointers (lifetime analysis)
// ✓ No data races (thread-aware borrow checker)
// ✓ No uninitialized memory (definite initialization)
// ✓ No format string attacks (no runtime format strings)
// ✓ No integer overflow (checked arithmetic by default)
//
// All enforced at compile time with zero runtime cost.
```

### Type Safety Guarantees

```fusion
// Fusion guarantees:
//
// ✓ No type confusion (nominal typing)
// ✓ No invalid enum variants (exhaustive matching)
// ✓ No missing function returns (return type enforcement)
// ✓ No null dereference (Option<T>)
// ✓ No unhandled errors (Result<T, E> with ?)
// ✓ No invalid casts (checked conversions)
// ✓ No ABI mismatches (extern "C" verification)
//
// Type errors are caught at compile time.
```

---

## Code Examples

### Error Handling with Result

```fusion
use std::fmt;

// Custom error type with From conversions
#[derive(Debug)]
enum ParseError {
    InvalidNumber(String),
    EmptyInput,
    Overflow,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidNumber(s) => write!(f, "Invalid number: '{s}'"),
            Self::EmptyInput => write!(f, "Input was empty"),
            Self::Overflow => write!(f, "Number too large"),
        }
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::InvalidNumber(e.to_string())
    }
}

fn parse_config_value(input: &str) -> Result<Int, ParseError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let value: Int = trimmed.parse()?;

    if value < 0 {
        return Err(ParseError::Overflow);
    }

    Ok(value)
}

fn main() {
    let inputs = vec!["  42  ", "", "abc", "-5", "99999999999999999999"];

    for input in inputs {
        match parse_config_value(input) {
            Ok(value) => print!("'{input}' → {value}"),
            Err(e) => print!("'{input}' → Error: {e}"),
        }
    }
}
```

### Panic Recovery

```fusion
use std::panic;

fn safe_divide(a: Float, b: Float) -> Result<Float, String> {
    if b == 0.0 {
        return Err("Division by zero".to_string());
    }
    Ok(a / b)
}

fn risky_operation() -> Result<Int, Box<dyn std::error::Error>> {
    // This might panic
    let numbers: Vec<Int> = vec![1, 2, 3];

    // Catch the panic and convert to Result
    let result = panic::catch_unwind(|| {
        // This panics if index is out of bounds
        numbers[10]
    });

    match result {
        Ok(value) => Ok(value),
        Err(_) => Err("Index out of bounds".into()),
    }
}

fn process_with_recovery() {
    // try/catch pattern for panics
    let result = panic::catch_unwind(|| {
        // Code that might panic
        panic!("Something went wrong!");
    });

    match result {
        Ok(_) => print!("Operation succeeded"),
        Err(panic_info) => {
            let msg = panic_info
                .downcast_ref::<&str>()
                .unwrap_or(&"Unknown panic");
            print("Caught panic: {msg}");
            // Continue execution
        }
    }
}
```

### Resource Cleanup with RAII

```fusion
struct DatabaseConnection {
    url: String,
    connection_id: usize,
}

impl DatabaseConnection {
    fn connect(url: &str) -> Result<Self, DbError> {
        print("Connecting to {url}...");
        Ok(Self {
            url: url.to_string(),
            connection_id: next_id(),
        })
    }

    fn query(&self, sql: &str) -> Result<Vec<Row>, DbError> {
        print("Executing on connection {}: {sql}", self.connection_id);
        // ...
    }
}

impl Drop for DatabaseConnection {
    fn drop(&mut self) {
        print("Closing connection {} to {}", self.connection_id, self.url);
    }
}

struct Transaction<'a> {
    conn: &'a DatabaseConnection,
    active: bool,
}

impl<'a> Transaction<'a> {
    fn begin(conn: &'a DatabaseConnection) -> Self {
        print("Beginning transaction");
        Self { conn, active: true }
    }

    fn execute(&self, sql: &str) -> Result<(), DbError> {
        if !self.active {
            return Err(DbError::TransactionClosed);
        }
        self.conn.query(sql)?;
        Ok(())
    }

    fn commit(mut self) -> Result<(), DbError> {
        self.active = false;
        print("Committing transaction");
        Ok(())
    }

    fn rollback(mut self) -> Result<(), DbError> {
        self.active = false;
        print("Rolling back transaction");
        Ok(())
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if self.active {
            print("Auto-rolling back uncommitted transaction");
        }
    }
}

fn transfer_funds(conn: &DatabaseConnection, from: Int, to: Int, amount: Float) -> Result<(), DbError> {
    let tx = Transaction::begin(conn);

    tx.execute(&format!("UPDATE accounts SET balance = balance - {amount} WHERE id = {from}"))?;
    tx.execute(&format!("UPDATE accounts SET balance = balance + {amount} WHERE id = {to}"))?;

    tx.commit()?;

    // Transaction is committed — Drop does nothing
    Ok(())
}

fn main() {
    let conn = DatabaseConnection::connect("postgres://localhost/mydb")?;

    // Connection is automatically dropped when `conn` goes out of scope
    // Even if an error occurs

    let result = transfer_funds(&conn, 1, 2, 100.0);
    match result {
        Ok(()) => print("Transfer complete"),
        Err(e) => print("Transfer failed: {e}"),
    }

    // conn is dropped here — connection closed automatically
}
```

### Capability-Secured Function

```fusion
use std::capability::{Capability, CapSet};

#[derive(Capability)]
struct FileAccess {
    read: bool,
    write: bool,
    paths: Vec<String>,
}

#[derive(Capability)]
struct NetworkAccess {
    connect: bool,
    domains: Vec<String>,
}

fn secure_function(
    file_cap: &FileAccess,
    net_cap: &NetworkAccess,
) -> Result<String, CapError> {
    // Check file capability
    if !file_cap.read {
        return Err(CapError::Denied("filesystem:read".into()));
    }

    if !file_cap.paths.contains(&"/data/config.toml".to_string()) {
        return Err(CapError::PathNotAllowed("/data/config.toml".into()));
    }

    // Check network capability
    if !net_cap.connect {
        return Err(CapError::Denied("network:connect".into()));
    }

    if !net_cap.domains.contains(&"api.example.com".to_string()) {
        return Err(CapError::DomainNotAllowed("api.example.com".into()));
    }

    // Both capabilities verified — proceed
    let config = std::fs::read_to_string("/data/config.toml")?;
    let response = reqwest::blocking::get("https://api.example.com/validate")?;

    Ok(format!("Config loaded, API responded: {}", response.status()))
}

fn main() {
    // Build capability set
    let file_cap = FileAccess {
        read: true,
        write: false,
        paths: vec!["/data/config.toml".to_string(), "/data/logs/*".to_string()],
    };

    let net_cap = NetworkAccess {
        connect: true,
        domains: vec!["api.example.com".to_string()],
    };

    match secure_function(&file_cap, &net_cap) {
        Ok(result) => print("Success: {result}"),
        Err(CapError::Denied(cap)) => print("Capability denied: {cap}"),
        Err(CapError::PathNotAllowed(path)) => print("Path not allowed: {path}"),
        Err(CapError::DomainNotAllowed(domain)) => print("Domain not allowed: {domain}"),
        Err(e) => print("Other error: {e}"),
    }
}
```

---

## Summary

Pillar 3 is what makes Fusion v2.0 Vortex a language you can **trust**:

- **Type system**: Static typing with inference, refinement types, dependent types — bugs caught at compile time
- **Null safety**: `Option<T>` eliminates null pointer exceptions; `Result<T, E>` eliminates unhandled errors
- **Memory safety**: Ownership + borrowing = no use-after-free, double-free, buffer overflow, or dangling pointers
- **Integer safety**: Checked arithmetic by default, with wrapping/saturating as opt-in alternatives
- **Error handling**: Structured `Result<T, E>` with `?` propagation, RAII cleanup, panic recovery
- **Security**: Capability-based access control, sandboxed execution, memory and type safety guarantees

These aren't features you use sometimes — they're the default. Safety is not optional in Fusion. It's the airbag that's always inflated, the seatbelt that's always fastened. You don't think about it until you need it, and when you need it, it's already there.

---

> **Next**: [Chapter 22 — Pillar 4: Quantum Computing (The Quantum Leap)](ch22-pillar4-quantum.md)
