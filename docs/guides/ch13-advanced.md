# Chapter 13: Advanced

> Macros, async/await, FFI, memory management, and compiler internals

---

## Macros

Fusion supports procedural and declarative macros for metaprogramming.

### Declarative Macros

```fusion
// Define a macro
macro_rules! vec {
    ($($x:expr),*) => {
        {
            let mut temp: Vec<_> = Vec::new();
            $(
                temp.push($x);
            )*
            temp
        }
    };
}

// Use the macro
fn main() -> int {
    let numbers: Vec<int> = vec![1, 2, 3, 4, 5];
    println("Numbers: %s", numbers.to_string());

    let strings: Vec<string> = vec!["hello", "world", "fusion"];
    println("Strings: %s", strings.to_string());

    return 0;
}
```

### Procedural Macros

```fusion
// Derive macro example
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: float,
    y: float,
}

// Attribute macro example
#[test]
fn test_point() {
    let p: Point = Point { x: 1.0, y: 2.0 };
    assert_eq(p, Point { x: 1.0, y: 2.0 });
}

// Function-like macro
macro_rules! hashmap {
    ($($key:expr => $value:expr),*) => {
        {
            let mut map: HashMap<_, _> = HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

fn main() -> int {
    let scores: HashMap<string, int> = hashmap! {
        "Alice" => 95,
        "Bob" => 87,
        "Charlie" => 92
    };

    for (name, score) in scores {
        println("%s: %d", name, score);
    }

    return 0;
}
```

### Macro Hygiene

```fusion
// Hygienic macros don't conflict with user code
macro_rules! safe_let {
    ($name:ident, $value:expr) => {
        // This variable name won't conflict
        let $name = $value;
    };
}

fn main() -> int {
    safe_let!(x, 42);
    safe_let!(y, 100);

    println("x=%d, y=%d", x, y);
    return 0;
}
```

---

## Async/Await

Fusion supports asynchronous programming with async/await syntax.

### Basic Async Functions

```fusion
async fn fetch_data(url: string) -> string {
    // Async HTTP request
    let response: string = await http::get(url);
    return response;
}

async fn process_data() {
    let data: string = await fetch_data("https://api.example.com/data");
    println("Data: %s", data);
}

fn main() -> int {
    // Run async function
    async::run(process_data());
    return 0;
}
```

### Async with Fibers

```fusion
use std::async;

async fn worker(id: int) {
    for i in 0..5 {
        println("Worker %d: step %d", id, i);
        await async::sleep(100);  // Yield control
    }
    println("Worker %d: done", id);
}

fn main() -> int {
    // Spawn multiple async fibers
    let fibers: [async::Fiber] = [
        spawn worker(1),
        spawn worker(2),
        spawn worker(3),
    ];

    // Wait for all fibers
    for fiber in fibers {
        fiber.join();
    }

    println("All workers completed");
    return 0;
}
```

### Async Channels

```fusion
use std::async;

async fn producer(chan: async::Sender<int>) {
    for i in 0..10 {
        await chan.send(i);
        println("Produced: %d", i);
    }
}

async fn consumer(chan: async::Receiver<int>) {
    loop {
        let value: Option<int> = await chan.recv();
        match value {
            Some(v) => println("Consumed: %d", v),
            None => break,
        }
    }
}

fn main() -> int {
    let (tx, rx): (async::Sender<int>, async::Receiver<int>) = async::channel();

    let prod: async::Fiber = spawn producer(tx);
    let cons: async::Fiber = spawn consumer(rx);

    prod.join();
    cons.join();

    return 0;
}
```

### Async Error Handling

```fusion
use std::async;

async fn risky_operation() -> Result<int, string> {
    // Simulate async operation that might fail
    await async::sleep(100);

    if random() < 0.5 {
        return Err("Operation failed".to_string());
    }

    return Ok(42);
}

async fn main_async() {
    match await risky_operation() {
        Ok(value) => println("Success: %d", value),
        Err(error) => println("Error: %s", error),
    }
}

fn main() -> int {
    async::run(main_async());
    return 0;
}
```

---

## FFI (Foreign Function Interface)

Fusion can interoperate with C and other languages via FFI.

### Calling C Functions

```fusion
// Declare external C function
extern fn printf(fmt: string, ...) -> int;
extern fn malloc(size: int) -> *byte;
extern fn free(ptr: *byte);

// Use C functions
fn main() -> int {
    // Call C printf
    printf("Hello from Fusion via C printf!\n");

    // Allocate memory with C malloc
    let ptr: *byte = malloc(100);
    // ... use memory ...
    free(ptr);

    return 0;
}
```

### Calling Fusion from C

```fusion
// Export function for C
pub fn add(a: int, b: int) -> int {
    return a + b;
}

pub fn process_string(input: string) -> string {
    return "Processed: " + input;
}
```

```c
// C code calling Fusion
#include <stdio.h>

// Declare Fusion functions
extern long add(long a, long b);
extern char* process_string(char* input);

int main() {
    long result = add(2, 3);
    printf("Result: %ld\n", result);

    char* processed = process_string("hello");
    printf("Processed: %s\n", processed);

    return 0;
}
```

### Struct FFI

```fusion
// Define a struct for FFI
#[repr(C)]
struct Point {
    x: float,
    y: float,
}

// Export struct constructor
pub fn point_new(x: float, y: float) -> Point {
    return Point { x, y };
}

// Export struct method
pub fn point_distance(p: Point) -> float {
    return (p.x * p.x + p.y * p.y);
}
```

### Calling C Libraries

```fusion
// Link to external C library
#[link(name = "curl")]
extern fn curl_easy_init() -> *byte;
extern fn curl_easy_perform(curl: *byte) -> int;
extern fn curl_easy_cleanup(curl: *byte);

fn fetch_url(url: string) -> string {
    let curl: *byte = curl_easy_init();
    // Set up request...
    let result: int = curl_easy_perform(curl);
    curl_easy_cleanup(curl);
    return "response";  // Simplified
}
```

---

## Memory Management

Fusion uses ownership and borrowing for memory management, but also supports manual management for performance-critical code.

### Automatic Memory Management

```fusion
fn main() -> int {
    // Ownership-based memory management
    let s: string = "hello";  // Allocated on heap
    let t: string = s;        // s is moved, original freed

    // RAII (Resource Acquisition Is Initialization)
    let file: File = File::open("data.txt");  // Opened
    // file is automatically closed when it goes out of scope

    println("Automatic memory management");
    return 0;
}
```

### Manual Memory Management

```fusion
// For performance-critical code
@unsafe
@manual_memory
fn manual_allocation() {
    let ptr: *byte = std::alloc::malloc(1024);
    // ... use memory ...
    std::alloc::free(ptr);
}

// Arena allocator for batch allocation
fn arena_example() {
    let arena: std::alloc::Arena = std::alloc::Arena::new(1024 * 1024);  // 1MB

    // Allocate from arena (fast, no individual frees)
    let a: *int = arena.alloc::<int>();
    let b: *int = arena.alloc::<int>();

    // Arena is freed all at once
    arena.free();
}
```

### Memory Pools

```fusion
// Object pool for frequently allocated/deallocated objects
struct ObjectPool<T> {
    pool: Vec<T>,
    max_size: int,
}

impl<T> ObjectPool<T> {
    fn new(max_size: int) -> ObjectPool<T> {
        return ObjectPool {
            pool: Vec::new(),
            max_size,
        };
    }

    fn acquire(mut self) -> T {
        if self.pool.len() > 0 {
            return self.pool.pop();
        }
        return T::new();
    }

    fn release(mut self, obj: T) {
        if self.pool.len() < self.max_size {
            self.pool.push(obj);
        }
    }
}

fn main() -> int {
    let pool: ObjectPool<Buffer> = ObjectPool::new(100);

    // Acquire from pool
    let buf: Buffer = pool.acquire();
    // ... use buffer ...
    pool.release(buf);

    return 0;
}
```

---

## Compiler Internals

### Compilation Pipeline

```
Source Code (.fu)
    ↓
Lexer (lexer.fu) → Tokens
    ↓
Parser (parser.fu) → AST
    ↓
Semantic Analysis (sema.fu) → Typed AST
    ↓
[Vortex Borrow Checker] → Safety Validation
    ↓
IR Lowering (ir_lower.rs) → IrModule
    ↓
Optimizer (optimizer.fu) → Optimized IR
    ↓
Code Generation → LLVM IR / WASM / Native
```

### Writing Compiler Passes

```fusion
// Example: Constant folding optimizer pass
fn constant_fold(ir: &mut IrModule) {
    for func in &mut ir.functions {
        for block in &mut func.blocks {
            let mut i: int = 0;
            while i < block.instrs.len() {
                match &block.instrs[i] {
                    Instruction::BinaryOperation { dest, op, op1, op2 } => {
                        // Check if both operands are constants
                        if let (Value::IntConst(a), Value::IntConst(b)) = (&op1.val, &op2.val) {
                            let result: int = match op {
                                BinaryOp::Add => a + b,
                                BinaryOp::Sub => a - b,
                                BinaryOp::Mul => a * b,
                                BinaryOp::Div => a / b,
                                _ => continue,
                            };
                            // Replace with constant
                            block.instrs[i] = Instruction::Copy {
                                dest: dest.clone(),
                                src: TypedValue { val: Value::IntConst(result), ty: Type::Int },
                            };
                        }
                    }
                    _ => {}
                }
                i = i + 1;
            }
        }
    }
}
```

### Type Checking

```fusion
// Type checker implementation
fn check_expression(expr: &Expression, env: &TypeEnv) -> Result<Type, TypeError> {
    match expr {
        Expression::IntLiteral(_) => Ok(Type::Int),
        Expression::BoolLiteral(_) => Ok(Type::Bool),
        Expression::StringLiteral(_) => Ok(Type::String),
        Expression::Variable(name) => {
            env.get(name).ok_or(TypeError::UnboundVariable(name.clone()))
        }
        Expression::BinaryOperation { left, op, right } => {
            let left_type: Type = check_expression(left, env)?;
            let right_type: Type = check_expression(right, env)?;

            match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    if left_type == Type::Int && right_type == Type::Int {
                        Ok(Type::Int)
                    } else {
                        Err(TypeError::TypeMismatch("int".to_string(), left_type))
                    }
                }
                BinaryOp::Eq | BinaryOp::Neq => {
                    if left_type == right_type {
                        Ok(Type::Bool)
                    } else {
                        Err(TypeError::TypeMismatch(left_type, right_type))
                    }
                }
                _ => Err(TypeError::InvalidOperation),
            }
        }
        _ => Err(TypeError::UnsupportedExpression),
    }
}
```

---

# Advanced PLT Features

This section covers advanced programming language theory features implemented in Fusion v2.0 Vortex, providing powerful abstractions for complex software development.

---

## Algebraic Effects and Handlers

Algebraic effects separate effect description from effect implementation, enabling composable side effects without monadic overhead.

### What Are Algebraic Effects?

Algebraic effects describe *what* operations are performed (e.g., IO, state, async) without specifying *how* they're implemented. Handlers provide the implementation, allowing the same effectful code to run in different contexts.

### Defining Effects

```fusion
// Define an effect interface
effect IO {
    fn read() -> string;
    fn write(msg: string) -> unit;
}

effect State<S> {
    fn get() -> S;
    fn set(state: S) -> unit;
}

effect Error<E> {
    fn raise(error: E) -> nothing;
}
```

### Performing Effects

```fusion
// Effectful function - describes operations without implementation
fn process_data() -> string with IO, State<int> {
    let name: string = perform IO::read();
    let count: int = perform State::get();
    perform State::set(count + 1);
    perform IO::write("Processing: %s".format(name));
    return "Result: %s".format(name);
}
```

### Handling Effects with Handlers

```fusion
// Handler for IO effect that logs to console
fn console_handler<Body>(body: Body) -> string {
    handle body {
        IO::read() => {
            // Read from stdin
            let input: string = std::io::read_line();
            resume(input);
        }
        IO::write(msg) => {
            // Write to stdout
            std::io::print(msg);
            resume(());
        }
    }
}

// Handler for State effect using mutable reference
fn state_handler<S, Body>(initial: S, body: Body) -> (S, string) {
    let mut state: S = initial;
    handle body {
        State::get() => {
            resume(state);
        }
        State::set(new_state) => {
            state = new_state;
            resume(());
        }
    }
}
```

### Code Example: IO Effect with Handler

```fusion
effect IO {
    fn read() -> string;
    fn write(msg: string) -> unit;
}

fn greet() -> unit with IO {
    let name: string = perform IO::read();
    perform IO::write("Hello, %s!".format(name));
}

fn main() -> int {
    // Run with console handler
    console_handler(greet());
    return 0;
}
```

### Code Example: State Effect with Handler

```fusion
effect State<S> {
    fn get() -> S;
    fn set(state: S) -> unit;
}

fn counter() -> int with State<int> {
    let count: int = perform State::get();
    perform State::set(count + 1);
    let count2: int = perform State::get();
    return count2;
}

fn main() -> int {
    let final_state: int;
    (final_state, _) = state_handler(0, counter());
    println("Final count: %d", final_state);  // Output: 1
    return 0;
}
```

### Code Example: Error Effect with Recovery

```fusion
effect Error<E> {
    fn raise(error: E) -> nothing;
}

fn safe_divide(a: int, b: int) -> int with Error<string> {
    if b == 0 {
        perform Error::raise("Division by zero");
    }
    return a / b;
}

fn recover_handler<Body>(body: Body) -> Option<int> {
    handle body {
        Error::raise(error) => {
            println("Error caught: %s", error);
            resume(None);
        }
    }
}

fn main() -> int {
    let result: Option<int> = recover_handler(safe_divide(10, 0));
    match result {
        Some(v) => println("Result: %d", v),
        None => println("Recovery applied"),
    }
    return 0;
}
```

### Built-in Effects

Fusion provides standard effects:

```fusion
// Async effect for non-blocking operations
effect Async {
    fn spawn(task: () -> unit) -> unit;
    fn yield() -> unit;
    fn sleep(ms: int) -> unit;
}

// Logging effect
effect Log {
    fn debug(msg: string) -> unit;
    fn info(msg: string) -> unit;
    fn warn(msg: string) -> unit;
    fn error(msg: string) -> unit;
}

// Network effect
effect Network {
    fn connect(addr: string) -> Socket;
    fn send(socket: Socket, data: bytes) -> unit;
    fn recv(socket: Socket) -> bytes;
}
```

### Use Cases

- **Modular error handling** without exception overhead
- **Testable I/O** by swapping handlers for testing
- **Composable side effects** that stack without monad transformers
- **Domain-specific languages** with custom effect interpretations

### Common Patterns

1. **Effect stacking**: Combine multiple effects in one function signature
2. **Handler composition**: Nest handlers for layered effect interpretation
3. **Effect polymorphism**: Write functions generic over which effects they use

---

## Linear / Affine Types

Linear types guarantee resources are used exactly once, preventing leaks, double-frees, and use-after-free bugs.

### What Are Linear Types?

Linear types enforce that a value must be used exactly once in all possible execution paths. Affine types allow *at most* once usage (can be discarded but not duplicated).

### Resource Protocols

```fusion
// Linear type representing a file handle
linear File {
    fd: int,
    
    fn open(path: string) -> File {
        let fd: int = sys::open(path, O_RDONLY);
        return File { fd };
    }
    
    fn read(self: File) -> (string, File) {
        let data: string = sys::read(self.fd);
        return (data, self);
    }
    
    fn close(self: File) -> unit {
        sys::close(self.fd);
    }
}
```

### Exact-Once Usage Guarantees

```fusion
fn process_file(path: string) -> string {
    let file: File = File::open(path);
    // 'file' must be used exactly once
    
    let (data, file) = file.read();
    // 'file' is still linear - must be used again
    
    file.close();
    // 'file' is consumed - no longer available
    
    return data;
    
    // Error: if we forget file.close(), compiler error
    // Error: if we call file.read() after close(), compiler error
    // Error: if we use 'file' twice in parallel, compiler error
}
```

### Code Example: File Handle Protocol

```fusion
linear File {
    fd: int,
    
    fn open(path: string) -> File {
        let fd: int = sys::open(path, O_RDWR | O_CREAT);
        return File { fd };
    }
    
    fn write(self: File, data: string) -> File {
        sys::write(self.fd, data);
        return self;
    }
    
    fn close(self: File) -> unit {
        sys::close(self.fd);
    }
}

fn write_config(path: string, config: string) -> unit {
    let file: File = File::open(path);
    let file: File = file.write(config);
    file.close();
}
```

### Code Example: Network Socket Protocol

```fusion
linear Socket {
    fd: int,
    
    fn connect(addr: string, port: int) -> Socket {
        let fd: int = sys::socket(AF_INET, SOCK_STREAM);
        sys::connect(fd, addr, port);
        return Socket { fd };
    }
    
    fn send(self: Socket, data: bytes) -> Socket {
        sys::send(self.fd, data);
        return self;
    }
    
    fn recv(self: Socket) -> (bytes, Socket) {
        let data: bytes = sys::recv(self.fd);
        return (data, self);
    }
    
    fn close(self: Socket) -> unit {
        sys::close(self.fd);
    }
}

fn fetch_page(url: string) -> bytes {
    let sock: Socket = Socket::connect(url, 80);
    let sock: Socket = sock.send("GET / HTTP/1.1\r\nHost: %s\r\n\r\n".format(url).as_bytes());
    let (data, sock) = sock.recv();
    sock.close();
    return data;
}
```

### Code Example: Database Connection Protocol

```fusion
linear Connection {
    handle: int,
    
    fn connect(db_url: string) -> Connection {
        let handle: int = db::connect(db_url);
        return Connection { handle };
    }
    
    fn query(self: Connection, sql: string) -> (Result<Rows, Error>, Connection) {
        let result: Result<Rows, Error> = db::query(self.handle, sql);
        return (result, self);
    }
    
    fn transaction<F>(self: Connection, f: F) -> (Result<unit, Error>, Connection) {
        db::begin(self.handle);
        match f(self) {
            Ok(_) => {
                db::commit(self.handle);
                return (Ok(()), self);
            }
            Err(e) => {
                db::rollback(self.handle);
                return (Err(e), self);
            }
        }
    }
    
    fn close(self: Connection) -> unit {
        db::disconnect(self.handle);
    }
}

fn migrate_db(db_url: string) -> unit {
    let conn: Connection = Connection::connect(db_url);
    let (result, conn) = conn.query("CREATE TABLE users (id INT, name TEXT)");
    match result {
        Ok(_) => println("Table created"),
        Err(e) => println("Error: %s", e),
    }
    conn.close();
}
```

### Use Cases

- **Resource management**: File handles, network sockets, database connections
- **Memory safety**: Preventing use-after-free, double-free
- **Concurrency safety**: Ensuring resources aren't shared unsafely
- **Protocol enforcement**: Implementing state machines at the type level

### Common Patterns

1. **Builder pattern with linear return**: Each method returns self for chaining
2. **Linear destructuring**: Pattern match to consume and produce linear values
3. **Linear containers**: Collections that hold exactly one resource

---

## Dependent Types

Dependent types allow types to depend on values, enabling precise specifications verified at compile time.

### Types That Depend on Values

```fusion
// Type 'Vec(n, T)' depends on the value 'n'
type Vec(n: int, T) = {
    data: Array<T, n>,
    len: int where len == n,
}

// Type 'Fin(n)' depends on value 'n'
type Fin(n: int) = int where self >= 0 and self < n;
```

### Indexed Types

```fusion
// Indexed type family
type Matrix(rows: int, cols: int, T) = {
    data: Array<T, rows * cols>,
    rows: int where rows == rows,
    cols: int where cols == cols,
}

// Type-level computation
type Add(a: int, b: int) = a + b;
type Mul(a: int, b: int) = a * b;
```

### Code Example: Vector of Exact Length

```fusion
// Vector with compile-time known length
type Vec(n: int, T) = {
    data: Array<T, n>,
}

impl Vec(0, T) {
    fn new() -> Vec(0, T) {
        return Vec { data: [] };
    }
}

impl Vec(n, T) where n > 0 {
    fn push(self: Vec(n, T), value: T) -> Vec(n + 1, T) {
        return Vec { data: self.data + [value] };
    }
    
    fn head(self: Vec(n, T)) -> (T, Vec(n - 1, T)) {
        let (first, rest) = self.data.split();
        return (first, Vec { data: rest });
    }
}

// Type-safe operations
fn append_one(n: int, v: Vec(n, int)) -> Vec(n + 1, int) {
    return v.push(42);
}

fn concat<a: int, b: int>(a: Vec(a, int), b: Vec(b, int)) -> Vec(a + b, int) {
    return Vec { data: a.data + b.data };
}

fn main() -> int {
    let v0: Vec(0, int) = Vec::new();
    let v1: Vec(1, int) = v0.push(1);
    let v2: Vec(2, int) = v1.push(2);
    let v3: Vec(3, int) = v2.push(3);
    
    println("Length: %d", v3.data.len());  // Compile-time known: 3
    
    // Error: type mismatch if we try wrong length
    // let wrong: Vec(2, int) = v3;  // ERROR: expected Vec(2, int), got Vec(3, int)
    
    return 0;
}
```

### Code Example: Bounded Integer

```fusion
// Bounded integer type
type Bounded(min: int, max: int) = int where self >= min and self <= max;

// Safe division with non-zero divisor
fn safe_divide<a: int, b: int>(x: Bounded<0, 100>, y: Bounded<1, 10>) -> Bounded<0, 100> {
    // y is guaranteed non-zero by type
    return x / y;
}

// Array indexing with bounds check at compile time
fn get_at<n: int, i: int>(arr: Array<int, n>, idx: Bounded<0, n - 1>) -> int {
    return arr[idx];
}

fn main() -> int {
    let x: Bounded<0, 100> = 50;
    let y: Bounded<1, 10> = 5;
    let result: Bounded<0, 100> = safe_divide(x, y);
    
    let arr: Array<int, 5> = [10, 20, 30, 40, 50];
    let idx: Bounded<0, 4> = 2;
    let val: int = get_at(arr, idx);
    
    println("Result: %d, Value: %d", result, val);
    return 0;
}
```

### Code Example: String Concatenation Preserving Length

```fusion
// String with known length
type String(n: int) = {
    data: Array<char, n>,
    len: int where len == n,
}

impl String(0) {
    fn new() -> String(0) {
        return String { data: [], len: 0 };
    }
}

impl String(n) {
    fn append<m: int>(self: String(n), other: String(m)) -> String(n + m) {
        return String {
            data: self.data + other.data,
            len: self.len + other.len,
        };
    }
    
    fn chars(self: String(n)) -> Array<char, n> {
        return self.data;
    }
}

fn main() -> int {
    let s1: String(5) = String { data: ['H', 'e', 'l', 'l', 'o'], len: 5 };
    let s2: String(6) = String { data: [' ', 'W', 'o', 'r', 'l', 'd'], len: 6 };
    
    let combined: String(11) = s1.append(s2);
    println("Length: %d", combined.len);  // Compile-time known: 11
    
    return 0;
}
```

### Use Cases

- **Safe array indexing** without runtime bounds checks
- **Protocol verification** ensuring message sequences are correct
- **State machine enforcement** at the type level
- **Performance optimization** with compile-time size knowledge

### Common Patterns

1. **Indexed families**: Types parameterized by natural numbers
2. **Type-level computation**: Arithmetic operations on type indices
3. **Dependent pairs**: Pairs where second component's type depends on first

---

## Refinement Types

Refinement types add logical predicates to types, enabling compile-time verification of invariants.

### Types with Logical Predicates

```fusion
// Refinement type with predicate
type PosInt = int where self > 0;
type NonNegInt = int where self >= 0;
type EvenInt = int where self % 2 == 0;
```

### Code Example: Positive Integer

```fusion
// Positive integer type
type PosInt = int where self > 0;

// Safe square root for positive numbers
fn sqrt(x: PosInt) -> float {
    // x is guaranteed positive
    return (x as float).sqrt();
}

// Refinement with dependent predicate
type Divisor(n: int) = int where self != 0 and n % self == 0;

fn largest_divisor(n: int) -> Divisor<n> {
    let mut i: int = n / 2;
    while i > 0 {
        if n % i == 0 {
            return i;
        }
        i = i - 1;
    }
    return 1;
}

fn main() -> int {
    let x: PosInt = 42;
    let result: float = sqrt(x);
    println("sqrt(42) = %f", result);
    
    let d: Divisor<12> = largest_divisor(12);
    println("Largest divisor of 12: %d", d);  // Output: 6
    
    return 0;
}
```

### Code Example: Bounded Range

```fusion
// Bounded range type
type Range<min: int, max: int> = int where self >= min and self <= max;

// Percentage type
type Percentage = Range<0, 100>;

// Angle in degrees
type Angle = Range<0, 360>;

// Safe increment that stays in bounds
fn increment<min: int, max: int>(x: Range<min, max>) -> Range<min, max + 1> {
    return x + 1;
}

// Clamped addition
fn clamp_add<min: int, max: int>(a: Range<min, max>, b: Range<0, max - max>) -> Range<min, max> {
    let result: int = a + b;
    if result > max {
        return max;
    }
    return result;
}

fn main() -> int {
    let pct: Percentage = 75;
    let angle: Angle = 180;
    
    let new_pct: Percentage = clamp_add(pct, 25);
    println("New percentage: %d", new_pct);
    
    let new_angle: Angle = angle + 90;
    println("New angle: %d", new_angle);
    
    return 0;
}
```

### Code Example: Non-Empty String

```fusion
// Non-empty string type
type NonEmptyString = string where self.len() > 0;

// String with maximum length
type MaxString<max: int> = string where self.len() <= max;

// Trimmed string (no leading/trailing whitespace)
type TrimmedString = string where self == self.trim();

fn process_name(name: NonEmptyString) -> NonEmptyString {
    // name is guaranteed non-empty
    return name.trim().to_uppercase();
}

fn validate_input(input: MaxString<100>) -> bool {
    // input is guaranteed to be at most 100 characters
    return input.len() > 0;
}

fn main() -> int {
    let name: NonEmptyString = "  Alice  ";
    let processed: NonEmptyString = process_name(name);
    println("Processed: '%s'", processed);
    
    let short: MaxString<10> = "hello";
    let valid: bool = validate_input(short);
    println("Valid: %b", valid);
    
    return 0;
}
```

### Use Cases

- **Input validation** without runtime checks
- **Invariant enforcement** in data structures
- **API contract verification** at compile time
- **Security properties** like "buffer size must be less than N"

### Common Patterns

1. **Bounded integers**: Preventing overflow and underflow
2. **Non-empty collections**: Avoiding empty list errors
3. **Sorted sequences**: Maintaining ordering invariants
4. **Valid identifiers**: Ensuring strings match patterns

---

## Gradual Typing

Gradual typing allows mixing static and dynamic typing, providing flexibility while maintaining safety.

### Dynamic + Static Typing Hybrid

```fusion
// Static typing (default)
let x: int = 42;
let name: string = "Alice";

// Dynamic typing with 'dynamic' keyword
let dynamic_value: dynamic = 42;
let dynamic_string: dynamic = "hello";

// Type annotations optional for dynamic values
let result: dynamic = dynamic_value + dynamic_string;
```

### Code Example: Dynamic Values with Type Annotations

```fusion
// Function that works with any type
fn process(value: dynamic) -> dynamic {
    // Type checked at runtime
    if value is int {
        return value * 2;
    } else if value is string {
        return value.to_uppercase();
    } else {
        return value;
    }
}

// Generic function with dynamic dispatch
fn serialize(value: dynamic) -> string {
    match value {
        int: value.to_string(),
        float: value.to_string(),
        string: "\"%s\"".format(value),
        bool: value ? "true" : "false",
        list: "[%s]".format(value.map(serialize).join(", ")),
        _ => "unknown",
    }
}

fn main() -> int {
    let x: dynamic = 42;
    let y: dynamic = process(x);
    println("Result: %d", y as int);
    
    let s: dynamic = "hello";
    let t: dynamic = process(s);
    println("Result: %s", t as string);
    
    let data: dynamic = [1, "two", 3.0, true];
    println("Serialized: %s", serialize(data));
    
    return 0;
}
```

### Code Example: Gradual Type Checking

```fusion
// Type-safe dynamic dispatch
fn add<a: type, b: type>(x: a, y: b) -> dynamic {
    // Type checked at compile time when possible
    if a == int and b == int {
        return x + y;
    } else if a == string and b == string {
        return x + y;
    } else if a == int and b == string {
        return x.to_string() + y;
    } else {
        return "cannot add %s and %s".format(type_name<a>(), type_name<b>());
    }
}

// Opt-in static checking
@static_check
fn strict_add(x: int, y: int) -> int {
    return x + y;
}

// Opt-out to dynamic
@dynamic_check
fn flexible_add(x: dynamic, y: dynamic) -> dynamic {
    return x + y;
}

fn main() -> int {
    let result1: dynamic = add(1, 2);
    let result2: dynamic = add("hello", " world");
    let result3: dynamic = add(1, " apple");
    
    println("1 + 2 = %s", result1);
    println("hello + world = %s", result2);
    println("1 + apple = %s", result3);
    
    // Strict function only accepts ints
    let strict_result: int = strict_add(10, 20);
    println("Strict: %d", strict_result);
    
    // Flexible function accepts anything
    let flex_result: dynamic = flexible_add("anything", 42);
    println("Flexible: %s", flex_result);
    
    return 0;
}
```

### Use Cases

- **Gradual migration** from dynamic to static typing
- **Interoperability** with dynamically typed libraries
- **Metaprogramming** where types aren't known at compile time
- **Scripting** where flexibility is valued over safety

### Common Patterns

1. **Type refinement**: Narrowing dynamic types with type checks
2. **Gradual boundaries**: Clear interfaces between static and dynamic code
3. **Type erasure**: Converting static types to dynamic for flexibility
4. **Type recovery**: Converting dynamic types back to static when possible

---

## Guaranteed TCO

Guaranteed tail-call optimization ensures recursive functions don't blow the stack, enabling safe recursion patterns.

### Tail-Call Optimization Guarantees

```fusion
// Compiler guarantees TCO for tail-recursive functions
@tailrec
fn factorial(n: int, acc: int = 1) -> int {
    if n <= 1 {
        return acc;
    }
    return factorial(n - 1, n * acc);  // Tail position
}

// Non-tail-recursive (not optimized)
fn bad_factorial(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * bad_factorial(n - 1);  // Not tail position
}
```

### Code Example: Tail-Recursive Factorial

```fusion
// Tail-recursive factorial with accumulator
@tailrec
fn factorial(n: int, acc: int = 1) -> int {
    if n <= 1 {
        return acc;
    }
    return factorial(n - 1, n * acc);
}

// Tail-recursive Fibonacci
@tailrec
fn fibonacci(n: int, a: int = 0, b: int = 1) -> int {
    if n == 0 {
        return a;
    }
    if n == 1 {
        return b;
    }
    return fibonacci(n - 1, b, a + b);
}

// Tail-recursive sum
@tailrec
fn sum(list: List<int>, acc: int = 0) -> int {
    match list {
        [] => acc,
        [head, ...tail] => sum(tail, acc + head),
    }
}

fn main() -> int {
    let fact: int = factorial(1000000);  // No stack overflow!
    println("Factorial: %d", fact);
    
    let fib: int = fibonacci(1000);
    println("Fibonacci: %d", fib);
    
    let numbers: List<int> = [1, 2, 3, 4, 5];
    let total: int = sum(numbers);
    println("Sum: %d", total);
    
    return 0;
}
```

### Code Example: Infinite Recursion Without Stack Overflow

```fusion
// Infinite sequence generator using TCO
@tailrec
fn generate(start: int, step: int) -> Generator<int> {
    yield start;
    return generate(start + step, step);  // Tail call
}

// State machine using TCO
@tailrec
fn state_machine(state: State, input: Input) -> State {
    let next_state: State = match (state, input) {
        (Idle, Start) => Running,
        (Running, Pause) => Paused,
        (Running, Stop) => Idle,
        (Paused, Resume) => Running,
        (Paused, Stop) => Idle,
        _ => state,  // Stay in current state
    };
    
    let next_input: Input = get_next_input();
    return state_machine(next_state, next_input);  // Tail call
}

// Tree traversal using TCO (continuation-passing style)
fn traverse_tree<T>(tree: Tree<T>, f: fn(T) -> unit) -> unit {
    traverse_helper(tree, f, []);
}

@tailrec
fn traverse_helper<T>(tree: Tree<T>, f: fn(T) -> unit, stack: List<Tree<T>>) -> unit {
    match tree {
        Leaf(value) => {
            f(value);
            match stack {
                [] => (),
                [next, ...rest] => traverse_helper(next, f, rest),
            }
        }
        Node(left, right) => {
            traverse_helper(left, f, [right, ...stack]);
        }
    }
}

fn main() -> int {
    // Infinite sequence without stack overflow
    let gen: Generator<int> = generate(0, 1);
    for i in 0..10 {
        println("Value: %d", gen.next());
    }
    
    // State machine runs forever
    // state_machine(Idle, get_input());
    
    return 0;
}
```

### Use Cases

- **Safe recursion** for large inputs
- **State machines** without explicit loops
- **Traversals** of deep data structures
- **Event loops** and interpreters

### Common Patterns

1. **Accumulator pattern**: Build result in tail position
2. **Continuation-passing**: Pass continuation as parameter
3. **Trampoline**: Return thunks instead of making tail calls
4. **Mutual recursion**: Ensure all recursive calls are tail calls

---

## Continuations

First-class continuations allow saving and restoring execution state, enabling advanced control flow.

### First-Class Control Flow

```fusion
// Call with current continuation
fn call_cc<F>(f: F) -> unit {
    let cont: Continuation = get_current_continuation();
    f(cont);
}

// Continuation represents "rest of computation"
fn example() -> int {
    call_cc(fn(k) {
        println("Before continuation");
        k(42);  // Jump back to call_cc
        println("After continuation (never reached)");
    });
    return 0;
}
```

### Code Example: Save and Restore Execution State

```fusion
// Save continuation for later use
fn save_state() -> Continuation {
    call_cc(fn(k) {
        return k;  // Return the continuation
    });
}

// Restore saved state
fn restore_state(cont: Continuation) -> unit {
    cont(());
}

// Example: checkpoint system
fn checkpoint_system() {
    let saved: Continuation = save_state();
    
    println("Processing...");
    let input: string = read_line();
    
    if input == "restart" {
        println("Restarting...");
        restore_state(saved);  // Jump back to checkpoint
    }
    
    println("Continuing...");
}

fn main() -> int {
    // Example: retry with continuation
    let result: int = call_cc(fn(k) {
        for i in 0..3 {
            let success: bool = attempt_operation();
            if success {
                k(42);  // Success: jump out with result
            }
            println("Attempt %d failed, retrying...", i);
        }
        return -1;  // All attempts failed
    });
    
    println("Result: %d", result);
    return 0;
}
```

### Code Example: Cooperative Multitasking

```fusion
// Simple coroutine using continuations
struct Coroutine {
    continuation: Continuation,
    running: bool,
}

fn create_coroutine<F>(f: F) -> Coroutine {
    let cont: Continuation = call_cc(fn(k) {
        f(k);
        return k;
    });
    return Coroutine { continuation: cont, running: true };
}

fn yield_to(cont: Continuation) -> unit {
    call_cc(fn(k) {
        cont(k);  // Switch to other continuation
        // We'll return here when someone switches back to us
    });
}

// Cooperative scheduler
fn scheduler(coroutines: List<Coroutine>) -> unit {
    let mut current: int = 0;
    loop {
        let coroutine: Coroutine = coroutines[current];
        if coroutine.running {
            coroutine.continuation(());
        }
        current = (current + 1) % coroutines.len();
        
        if all_done(coroutines) {
            break;
        }
    }
}

fn main() -> int {
    // Create coroutines
    let coro1: Coroutine = create_coroutine(fn(k) {
        for i in 0..5 {
            println("Coroutine 1: %d", i);
            yield_to(k);
        }
    });
    
    let coro2: Coroutine = create_coroutine(fn(k) {
        for i in 0..5 {
            println("Coroutine 2: %d", i);
            yield_to(k);
        }
    });
    
    // Run cooperatively
    scheduler([coro1, coro2]);
    
    return 0;
}
```

### Use Cases

- **Exception handling** with non-local returns
- **Coroutines** and generators
- **Backtracking** algorithms
- **Continuation-passing style** (CPS) transformations

### Common Patterns

1. **Delimited continuations**: Capture only part of the continuation
2. **Composable continuations**: Combine continuations safely
3. **One-shot continuations**: Can only be invoked once
4. **Multi-shot continuations**: Can be invoked multiple times

---

## Capability-Based Security

Capability-based security provides fine-grained access control through unforgeable tokens.

### Object-Capability Model

```fusion
// Capabilities are unforgeable tokens
struct FileCapability {
    path: string,
    permissions: Set<Permission>,
}

struct Permission {
    read: bool,
    write: bool,
    execute: bool,
}

// Grant capability to code
fn grant_file_access(path: string, perms: Set<Permission>) -> FileCapability {
    return FileCapability { path, permissions: perms };
}

// Use capability to access resource
fn read_file(cap: FileCapability) -> Option<string> {
    if cap.permissions.contains(Permission::Read) {
        return Some(fs::read(cap.path));
    }
    return None;
}
```

### Code Example: File System Capabilities

```fusion
// Fine-grained file capabilities
struct FileSystemCaps {
    read_caps: Map<string, FileCapability>,
    write_caps: Map<string, FileCapability>,
}

fn create_sandbox() -> FileSystemCaps {
    return FileSystemCaps {
        read_caps: Map::new(),
        write_caps: Map::new(),
    };
}

fn grant_read(sandbox: FileSystemCaps, path: string) -> FileSystemCaps {
    let cap: FileCapability = FileCapability {
        path: path.clone(),
        permissions: {Permission::Read},
    };
    let mut new_caps: FileSystemCaps = sandbox;
    new_caps.read_caps.insert(path, cap);
    return new_caps;
}

fn grant_write(sandbox: FileSystemCaps, path: string) -> FileSystemCaps {
    let cap: FileCapability = FileCapability {
        path: path.clone(),
        permissions: {Permission::Write},
    };
    let mut new_caps: FileSystemCaps = sandbox;
    new_caps.write_caps.insert(path, cap);
    return new_caps;
}

fn safe_read(sandbox: FileSystemCaps, path: string) -> Option<string> {
    let cap: Option<FileCapability> = sandbox.read_caps.get(path);
    match cap {
        Some(c) => read_file(c),
        None => None,  // No capability = no access
    }
}

fn safe_write(sandbox: FileSystemCaps, path: string, data: string) -> bool {
    let cap: Option<FileCapability> = sandbox.write_caps.get(path);
    match cap {
        Some(c) => write_file(c, data),
        None => false,  // No capability = no access
    }
}

fn main() -> int {
    let sandbox: FileSystemCaps = create_sandbox();
    let sandbox: FileSystemCaps = grant_read(sandbox, "/tmp/data.txt");
    let sandbox: FileSystemCaps = grant_write(sandbox, "/tmp/output.txt");
    
    // Can read
    let data: Option<string> = safe_read(sandbox, "/tmp/data.txt");
    match data {
        Some(d) => println("Read: %s", d),
        None => println("Cannot read"),
    }
    
    // Can write
    let success: bool = safe_write(sandbox, "/tmp/output.txt", "hello");
    println("Write: %b", success);
    
    // Cannot access /etc/passwd (no capability)
    let forbidden: Option<string> = safe_read(sandbox, "/etc/passwd");
    match forbidden {
        Some(_) => println("SHOULD NOT HAPPEN"),
        None => println("Correctly denied"),
    }
    
    return 0;
}
```

### Code Example: Network Capabilities

```fusion
// Network capabilities for restricted access
struct NetworkCaps {
    allowed_hosts: Set<string>,
    allowed_ports: Set<int>,
    max_connections: int,
}

fn create_network_sandbox(max_conn: int) -> NetworkCaps {
    return NetworkCaps {
        allowed_hosts: Set::new(),
        allowed_ports: Set::new(),
        max_connections: max_conn,
    };
}

fn allow_host(sandbox: NetworkCaps, host: string) -> NetworkCaps {
    let mut new_caps: NetworkCaps = sandbox;
    new_caps.allowed_hosts.insert(host);
    return new_caps;
}

fn allow_port(sandbox: NetworkCaps, port: int) -> NetworkCaps {
    let mut new_caps: NetworkCaps = sandbox;
    new_caps.allowed_ports.insert(port);
    return new_caps;
}

fn safe_connect(sandbox: NetworkCaps, host: string, port: int) -> Option<Socket> {
    if sandbox.allowed_hosts.contains(host) and sandbox.allowed_ports.contains(port) {
        return Some(Socket::connect(host, port));
    }
    return None;
}

fn main() -> int {
    let sandbox: NetworkCaps = create_network_sandbox(10);
    let sandbox: NetworkCaps = allow_host(sandbox, "api.example.com");
    let sandbox: NetworkCaps = allow_port(sandbox, 443);
    
    // Allowed connection
    let socket: Option<Socket> = safe_connect(sandbox, "api.example.com", 443);
    match socket {
        Some(s) => println("Connected!"),
        None => println("Connection denied"),
    }
    
    // Blocked connection
    let blocked: Option<Socket> = safe_connect(sandbox, "evil.com", 80);
    match blocked {
        Some(_) => println("SHOULD NOT HAPPEN"),
        None => println("Correctly blocked"),
    }
    
    return 0;
}
```

### Code Example: Sandboxed Execution

```fusion
// Sandboxed code execution with capabilities
struct Sandbox {
    file_caps: FileSystemCaps,
    network_caps: NetworkCaps,
    memory_limit: int,
    time_limit: int,
}

fn create_sandbox() -> Sandbox {
    return Sandbox {
        file_caps: create_sandbox(),
        network_caps: create_network_sandbox(5),
        memory_limit: 1024 * 1024 * 100,  // 100MB
        time_limit: 10000,  // 10 seconds
    };
}

fn run_in_sandbox<T>(sandbox: Sandbox, code: fn(Sandbox) -> T) -> Result<T, SandboxError> {
    // Set resource limits
    set_memory_limit(sandbox.memory_limit);
    set_time_limit(sandbox.time_limit);
    
    // Execute with capabilities
    let result: T = code(sandbox);
    
    // Reset limits
    reset_limits();
    
    return Ok(result);
}

// Example: safely run untrusted code
fn process_user_code(user_code: string) -> Result<string, SandboxError> {
    let sandbox: Sandbox = create_sandbox();
    let sandbox: Sandbox = grant_read(sandbox, "/tmp/user_data");
    let sandbox: Sandbox = grant_write(sandbox, "/tmp/output");
    let sandbox: NetworkCaps = allow_host(sandbox.network_caps, "api.trusted.com");
    
    return run_in_sandbox(sandbox, fn(s) {
        // user_code runs here with limited capabilities
        let result: string = execute(user_code, s);
        return result;
    });
}

fn main() -> int {
    match process_user_code("read('/tmp/user_data')") {
        Ok(result) => println("Result: %s", result),
        Err(e) => println("Error: %s", e),
    }
    
    return 0;
}
```

### Use Cases

- **Sandboxing** untrusted code
- **Least privilege** enforcement
- **Microservice isolation**
- **Plugin systems** with restricted access

### Common Patterns

1. **Capability leaking**: Only share capabilities intentionally
2. **Capability attenuation**: Reduce permissions when delegating
3. **Capability revocation**: Invalidate capabilities when no longer needed
4. **Capability composition**: Combine multiple capabilities

---

## Multiple Dispatch

Multiple dispatch selects methods based on the runtime types of all arguments, enabling flexible operator overloading.

### Method Resolution by All Argument Types

```fusion
// Multiple dispatch on all argument types
fn add(x: int, y: int) -> int {
    return x + y;
}

fn add(x: float, y: float) -> float {
    return x + y;
}

fn add(x: string, y: string) -> string {
    return x + y;
}

fn add(x: int, y: float) -> float {
    return (x as float) + y;
}

fn add(x: float, y: int) -> float {
    return x + (y as float);
}
```

### Code Example: Operator Overloading

```fusion
// Rich operator overloading with multiple dispatch
struct Vec2 {
    x: float,
    y: float,
}

struct Vec3 {
    x: float,
    y: float,
    z: float,
}

// Vector addition
fn add(a: Vec2, b: Vec2) -> Vec2 {
    return Vec2 { x: a.x + b.x, y: a.y + b.y };
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    return Vec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z };
}

// Scalar multiplication
fn mul(v: Vec2, s: float) -> Vec2 {
    return Vec2 { x: v.x * s, y: v.y * s };
}

fn mul(v: Vec3, s: float) -> Vec3 {
    return Vec3 { x: v.x * s, y: v.y * s, z: v.z * s };
}

fn mul(s: float, v: Vec2) -> Vec2 {
    return mul(v, s);
}

fn mul(s: float, v: Vec3) -> Vec3 {
    return mul(v, s);
}

// Dot product
fn dot(a: Vec2, b: Vec2) -> float {
    return a.x * b.x + a.y * b.y;
}

fn dot(a: Vec3, b: Vec3) -> float {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

fn main() -> int {
    let v1: Vec2 = Vec2 { x: 1.0, y: 2.0 };
    let v2: Vec2 = Vec2 { x: 3.0, y: 4.0 };
    
    let sum: Vec2 = v1 + v2;  // Uses add(Vec2, Vec2)
    let scaled: Vec2 = v1 * 2.0;  // Uses mul(Vec2, float)
    let product: float = dot(v1, v2);  // Uses dot(Vec2, Vec2)
    
    println("Sum: (%f, %f)", sum.x, sum.y);
    println("Scaled: (%f, %f)", scaled.x, scaled.y);
    println("Dot product: %f", product);
    
    return 0;
}
```

### Code Example: Matrix Algebra

```fusion
// Matrix types with multiple dispatch
struct Mat2 {
    data: [[float; 2]; 2],
}

struct Mat3 {
    data: [[float; 3]; 3],
}

struct Mat4 {
    data: [[float; 4]; 4],
}

// Matrix multiplication
fn mul(a: Mat2, b: Mat2) -> Mat2 {
    let mut result: Mat2 = Mat2 { data: [[0.0; 2]; 2] };
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                result.data[i][j] += a.data[i][k] * b.data[k][j];
            }
        }
    }
    return result;
}

fn mul(a: Mat3, b: Mat3) -> Mat3 {
    let mut result: Mat3 = Mat3 { data: [[0.0; 3]; 3] };
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                result.data[i][j] += a.data[i][k] * b.data[k][j];
            }
        }
    }
    return result;
}

// Matrix-vector multiplication
fn mul(a: Mat2, v: Vec2) -> Vec2 {
    return Vec2 {
        x: a.data[0][0] * v.x + a.data[0][1] * v.y,
        y: a.data[1][0] * v.x + a.data[1][1] * v.y,
    };
}

fn mul(a: Mat3, v: Vec3) -> Vec3 {
    return Vec3 {
        x: a.data[0][0] * v.x + a.data[0][1] * v.y + a.data[0][2] * v.z,
        y: a.data[1][0] * v.x + a.data[1][1] * v.y + a.data[1][2] * v.z,
        z: a.data[2][0] * v.x + a.data[2][1] * v.y + a.data[2][2] * v.z,
    };
}

// Matrix transpose
fn transpose(a: Mat2) -> Mat2 {
    return Mat2 {
        data: [
            [a.data[0][0], a.data[1][0]],
            [a.data[0][1], a.data[1][1]],
        ],
    };
}

fn transpose(a: Mat3) -> Mat3 {
    return Mat3 {
        data: [
            [a.data[0][0], a.data[1][0], a.data[2][0]],
            [a.data[0][1], a.data[1][1], a.data[2][1]],
            [a.data[0][2], a.data[1][2], a.data[2][2]],
        ],
    };
}

fn main() -> int {
    let m1: Mat2 = Mat2 { data: [[1.0, 2.0], [3.0, 4.0]] };
    let m2: Mat2 = Mat2 { data: [[5.0, 6.0], [7.0, 8.0]] };
    
    let product: Mat2 = m1 * m2;  // Uses mul(Mat2, Mat2)
    let v: Vec2 = Vec2 { x: 1.0, y: 2.0 };
    let mv: Vec2 = m1 * v;  // Uses mul(Mat2, Vec2)
    let t: Mat2 = transpose(m1);  // Uses transpose(Mat2)
    
    println("Product: [[%f, %f], [%f, %f]]",
            product.data[0][0], product.data[0][1],
            product.data[1][0], product.data[1][1]);
    
    return 0;
}
```

### Use Cases

- **Operator overloading** with type-specific behavior
- **Generic algorithms** that work across type hierarchies
- **Visitor patterns** without double dispatch
- **Domain-specific languages** with custom operators

### Common Patterns

1. **Method tables**: Runtime dispatch based on argument types
2. **Symmetric operations**: Define addition for (int, float) and (float, int)
3. **Fallback methods**: Generic methods for unmatched type combinations
4. **Specialized implementations**: Optimized versions for common cases

---

## Effect Polymorphism

Effect polymorphism allows functions to be generic over which effects they perform, enabling reusable effectful code.

### Generic Functions with Effect Signatures

```fusion
// Effect-polymorphic function
fn map<A, B, E>(list: List<A>, f: fn(A) -> B with E) -> List<B> with E {
    match list {
        [] => [],
        [head, ...tail] => [f(head), ...map(tail, f)],
    }
}

// Works with any effect
fn process_strings(strings: List<string>) -> List<int> with IO {
    return map(strings, fn(s) {
        perform IO::write("Processing: %s".format(s));
        return s.len();
    });
}

fn process_numbers(numbers: List<int>) -> List<int> with Error<string> {
    return map(numbers, fn(n) {
        if n < 0 {
            perform Error::raise("Negative number");
        }
        return n * 2;
    });
}
```

### Code Example: Effect-Polymorphic Function

```fusion
// Generic logging function
fn with_logging<A, E>(name: string, f: fn() -> A with E) -> A with E with Log {
    perform Log::info("Starting %s".format(name));
    let start: int = get_time();
    let result: A = f();
    let elapsed: int = get_time() - start;
    perform Log::info("Finished %s in %dms".format(name, elapsed));
    return result;
}

// Generic error handling
fn with_retry<A, E>(max_attempts: int, f: fn() -> A with E) -> A with E with Error<string> {
    let mut attempt: int = 0;
    loop {
        match attempt {
            a if a >= max_attempts => {
                perform Error::raise("Max attempts exceeded");
            }
            _ => {
                match f() {
                    result => return result,
                    error => {
                        attempt = attempt + 1;
                        perform Log::warn("Attempt %d failed".format(attempt));
                    }
                }
            }
        }
    }
}

// Composable effectful operations
fn fetch_and_process<A, B>(
    url: string,
    fetch: fn(string) -> A with Network,
    process: fn(A) -> B with IO
) -> B with Network with IO {
    let data: A = fetch(url);
    let result: B = process(data);
    return result;
}

fn main() -> int {
    // Use with logging
    let result: int = with_logging("computation", fn() {
        return 42 * 42;
    });
    
    // Use with retry
    let value: string = with_retry(3, fn() {
        return risky_operation();
    });
    
    // Use with different effects
    let processed: string = fetch_and_process(
        "https://api.example.com/data",
        fn(url) { return http::get(url); },
        fn(data) { return data.to_uppercase(); }
    );
    
    return 0;
}
```

### Use Cases

- **Reusable effectful code** that works with any effect combination
- **Library functions** that don't prescribe specific effects
- **Testable code** where effects can be swapped for testing
- **Composable abstractions** that layer effects cleanly

### Common Patterns

1. **Effect abstraction**: Write code generic over effect types
2. **Effect stacking**: Combine multiple effects in one signature
3. **Effect subtyping**: More specific effects are subtypes of more general ones
4. **Effect erasure**: Remove effects at compile time for performance

---

## Formal Verification

Formal verification proves program correctness using mathematical logic, catching bugs that testing might miss.

### Preconditions, Postconditions, Invariants

```fusion
// Function contracts with pre/postconditions
fn abs(x: int) -> int
    requires x >= -2147483648  // Precondition
    ensures result >= 0         // Postcondition
{
    if x < 0 {
        return -x;
    }
    return x;
}

// Loop invariants
fn sum_array(arr: Array<int>) -> int
    requires arr.len() > 0
    ensures result >= arr[0]
{
    let mut sum: int = 0;
    let mut i: int = 0;
    invariant i <= arr.len()
    invariant sum >= 0 if arr.len() > 0
    while i < arr.len() {
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}
```

### Code Example: Verified Function Contract

```fusion
// Verified binary search
fn binary_search(arr: Array<int>, target: int) -> Option<int>
    requires is_sorted(arr)
    ensures match result {
        Some(idx) => arr[idx] == target,
        None => !arr.contains(target),
    }
{
    let mut low: int = 0;
    let mut high: int = arr.len();
    
    invariant 0 <= low <= high <= arr.len()
    invariant !arr[0..low].contains(target)
    invariant arr[high..arr.len()].contains(target) implies false
    while low < high {
        let mid: int = low + (high - low) / 2;
        if arr[mid] == target {
            return Some(mid);
        } else if arr[mid] < target {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    return None;
}

// Verified sorting function
fn insertionsort(arr: Array<int>) -> Array<int>
    ensures is_sorted(result)
    ensures result.len() == arr.len()
    ensures result.permutation_of(arr)
{
    let mut result: Array<int> = arr.clone();
    let mut i: int = 1;
    
    invariant is_sorted(result[0..i])
    invariant result.permutation_of(arr)
    while i < result.len() {
        let key: int = result[i];
        let mut j: int = i - 1;
        
        while j >= 0 and result[j] > key {
            result[j + 1] = result[j];
            j = j - 1;
        }
        result[j + 1] = key;
        i = i + 1;
    }
    return result;
}

fn main() -> int {
    let arr: Array<int> = [5, 3, 8, 1, 2];
    let sorted: Array<int> = insertionsort(arr);
    println("Sorted: %s", sorted);
    
    let idx: Option<int> = binary_search(sorted, 3);
    match idx {
        Some(i) => println("Found at index %d", i),
        None => println("Not found"),
    }
    
    return 0;
}
```

### Code Example: ZKP Proof

```fusion
// Zero-knowledge proof system
struct ZKProof {
    commitment: Hash,
    challenge: Hash,
    response: Hash,
}

// Prove knowledge of discrete log without revealing it
fn prove_knowledge(g: int, h: int, p: int, x: int) -> ZKProof
    requires pow(g, x, p) == h
    ensures verify_zkp(g, h, p, proof)
{
    // Commitment phase
    let r: int = random_mod(p);
    let commitment: int = pow(g, r, p);
    
    // Challenge phase (Fiat-Shamir)
    let challenge: Hash = hash(commitment);
    
    // Response phase
    let response: int = (r + x * challenge) % (p - 1);
    
    return ZKProof {
        commitment: hash(commitment),
        challenge,
        response: hash(response),
    };
}

fn verify_zkp(g: int, h: int, p: int, proof: ZKProof) -> bool {
    // Verify the proof without learning x
    let commitment: int = unhash(proof.commitment);
    let challenge: int = unhash(proof.challenge);
    let response: int = unhash(proof.response);
    
    let lhs: int = pow(g, response, p);
    let rhs: int = commitment * pow(h, challenge, p) % p;
    
    return lhs == rhs;
}

// Verified arithmetic circuit
struct Circuit {
    constraints: List<Constraint>,
}

struct Constraint {
    a: Wire,
    b: Wire,
    c: Wire,
    op: Operation,
}

fn verify_circuit(circuit: Circuit, inputs: Map<Wire, int>, outputs: Map<Wire, int>) -> bool
    ensures result == circuit.satisfies(inputs, outputs)
{
    let mut wires: Map<Wire, int> = inputs.clone();
    
    for constraint in circuit.constraints {
        let a_val: int = wires.get(constraint.a);
        let b_val: int = wires.get(constraint.b);
        let c_val: int = match constraint.op {
            Operation::Add => a_val + b_val,
            Operation::Mul => a_val * b_val,
        };
        wires.insert(constraint.c, c_val);
    }
    
    // Check outputs match
    for (wire, expected) in outputs {
        if wires.get(wire) != expected {
            return false;
        }
    }
    
    return true;
}

fn main() -> int {
    let g: int = 2;
    let h: int = 8;
    let p: int = 17;
    let x: int = 3;  // Secret: 2^3 mod 17 = 8
    
    let proof: ZKProof = prove_knowledge(g, h, p, x);
    let valid: bool = verify_zkp(g, h, p, proof);
    println("Proof valid: %b", valid);
    
    return 0;
}
```

### Use Cases

- **Critical systems** where bugs are unacceptable
- **Cryptographic protocols** requiring mathematical proofs
- **Compiler correctness** ensuring optimizations are safe
- **Protocol verification** for distributed systems

### Common Patterns

1. **Hoare logic**: Pre/postcondition verification
2. **Model checking**: Exhaustive state space exploration
3. **SMT solving**: Automated theorem proving
4. **Proof assistants**: Interactive theorem proving

---

## Partial Evaluation / Staging

Multi-stage programming allows code generation and specialization at compile time, improving runtime performance.

### Multi-Stage Programming

```fusion
// Stage annotations
@stage(1) fn compile_time computation() -> int {
    return 42;  // Evaluated at compile time
}

@stage(0) fn runtime computation() -> int {
    return dynamic_value;  // Evaluated at runtime
}

// Staged code generation
@stage(1) fn generate_adder(n: int) -> fn(int) -> int {
    return fn(x: int) -> int {
        return x + n;  // 'n' is known at stage 1
    };
}
```

### Code Example: Compile-Time Specialization

```fusion
// Compile-time specialization based on known values
@stage(1) fn specialized_parser(format: string) -> fn(string) -> Result<Dynamic, string> {
    // Parse format string at compile time
    let spec: FormatSpec = parse_format(format);
    
    return fn(input: string) -> Result<Dynamic, string> {
        // Generated parser specific to this format
        return spec.parse(input);
    };
}

// Staged matrix operations
@stage(1) fn generate_matrix_multiply(m: int, n: int, p: int) -> fn(Matrix<m, n>, Matrix<n, p>) -> Matrix<m, p> {
    // Generate optimized multiplication for specific dimensions
    return fn(a: Matrix<m, n>, b: Matrix<n, p>) -> Matrix<m, p> {
        let mut result: Matrix<m, p> = Matrix::zero();
        for i in 0..m {
            for j in 0..p {
                for k in 0..n {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        return result;
    };
}

// Compile-time regex compilation
@stage(1) fn compile_regex(pattern: string) -> fn(string) -> Option<Match> {
    let regex: Regex = Regex::compile(pattern);  // Compile at stage 1
    
    return fn(input: string) -> Option<Match> {
        return regex.match(input);  // Use compiled regex at stage 0
    };
}

fn main() -> int {
    // Specialized at compile time
    let parse_int: fn(string) -> Result<int, string> = specialized_parser("%d");
    let result: Result<int, string> = parse_int("42");
    
    // Matrix multiply specialized for 3x3
    let mul_3x3: fn(Matrix<3, 3>, Matrix<3, 3>) -> Matrix<3, 3> = generate_matrix_multiply(3, 3, 3);
    
    // Regex compiled once
    let email_re: fn(string) -> Option<Match> = compile_regex(r"^[a-zA-Z0-9+_.-]+@[a-zA-Z0-9.-]+$");
    let is_email: bool = email_re("user@example.com").is_some();
    
    return 0;
}
```

### Code Example: Load-Time Optimization

```fusion
// Load-time configuration optimization
@stage(1) fn optimize_config(config: Config) -> OptimizedConfig {
    // Optimize configuration at load time
    let mut optimized: OptimizedConfig = OptimizedConfig::new();
    
    // Pre-compute hash tables
    for (key, value) in config.settings {
        optimized.hash_table.insert(hash(key), value);
    }
    
    // Pre-compile paths
    for path in config.paths {
        optimized.compiled_paths.push(compile_path(path));
    }
    
    return optimized;
}

// JIT compilation of hot paths
@stage(1) fn jit_compile(expr: Expr) -> fn(Environment) -> Value {
    // Analyze expression at load time
    let analysis: Analysis = analyze(expr);
    
    // Generate specialized code
    if analysis.is_constant {
        let value: Value = eval(expr, empty_env);
        return fn(_) -> Value { return value; };
    }
    
    if analysis.is_linear {
        let slope: float = compute_slope(expr);
        let intercept: float = compute_intercept(expr);
        return fn(env) -> Value {
            return Value::Float(slope * env.get("x") + intercept);
        };
    }
    
    // Fallback to interpretation
    return fn(env) -> Value {
        return eval(expr, env);
    };
}

// Template specialization
@stage(1) fn specialize_template<T: type>(template: Template, type_info: TypeInfo<T>) -> CompiledTemplate<T> {
    // Specialize template for specific type
    let mut compiled: CompiledTemplate<T> = CompiledTemplate::new();
    
    // Generate type-specific code
    for instruction in template.instructions {
        match instruction {
            Instruction::Load(field) => {
                if type_info.is_inline(field) {
                    compiled.add(OptimizedInstruction::LoadInline(field));
                } else {
                    compiled.add(OptimizedInstruction::LoadIndirect(field));
                }
            }
            Instruction::Compute(op) => {
                if type_info.is_simd_compatible() {
                    compiled.add(OptimizedInstruction::SimdCompute(op));
                } else {
                    compiled.add(OptimizedInstruction::ScalarCompute(op));
                }
            }
        }
    }
    
    return compiled;
}

fn main() -> int {
    // Load-time optimization
    let config: Config = load_config();
    let optimized: OptimizedConfig = optimize_config(config);
    
    // JIT compilation
    let expr: Expr = parse("x * 2 + 1");
    let compiled: fn(Environment) -> Value = jit_compile(expr);
    let result: Value = compiled(Environment::from("x", 21));
    println("Result: %f", result.as_float());
    
    // Template specialization
    let template: Template = load_template("vector");
    let specialized: CompiledTemplate<Vec3> = specialize_template(template, TypeInfo::of::<Vec3>());
    
    return 0;
}
```

### Use Cases

- **Performance optimization** through specialization
- **Domain-specific language** compilation
- **Code generation** for repetitive patterns
- **Configuration-driven** optimization

### Common Patterns

1. **Staged metaprogramming**: Generate code at different stages
2. **Partial evaluation**: Specialize functions with known inputs
3. **JIT compilation**: Compile hot paths at runtime
4. **Template specialization**: Generate type-specific code

---

## Actor Model

The actor model provides lightweight concurrency through message-passing actors with supervision trees.

### Built-in Actors with Supervision Trees

```fusion
// Actor definition
actor Counter {
    state: int = 0;
    
    fn increment() -> unit {
        self.state = self.state + 1;
    }
    
    fn get() -> int {
        return self.state;
    }
    
    fn reset() -> unit {
        self.state = 0;
    }
}

// Supervisor definition
supervisor CounterSupervisor {
    strategy: one_for_one,
    children: [Counter],
    
    fn on_child_failure(child: ActorRef, reason: string) -> unit {
        println("Child %s failed: %s", child.name, reason);
        // Strategy handles restart
    }
}
```

### Code Example: Actor Creation and Message Passing

```fusion
// Actor types and messages
enum WorkerMessage {
    Process(Data),
    Shutdown,
}

enum WorkerResponse {
    Completed(Result),
    Error(string),
}

actor Worker {
    id: int,
    inbox: Mailbox<WorkerMessage>,
    
    fn init(id: int) -> unit {
        self.id = id;
        self.inbox = Mailbox::new();
    }
    
    fn handle(msg: WorkerMessage) -> Option<WorkerResponse> {
        match msg {
            WorkerMessage::Process(data) => {
                let result: Result = process_data(data);
                return Some(WorkerResponse::Completed(result));
            }
            WorkerMessage::Shutdown => {
                return None;  // Actor stops
            }
        }
    }
}

// Actor creation and communication
fn main() -> int {
    // Create workers
    let worker1: ActorRef<WorkerMessage> = Actor::spawn(Worker::init(1));
    let worker2: ActorRef<WorkerMessage> = Actor::spawn(Worker::init(2));
    
    // Send messages
    worker1.send(WorkerMessage::Process(data1));
    worker2.send(WorkerMessage::Process(data2));
    
    // Receive responses
    let response1: WorkerResponse = worker1.receive();
    let response2: WorkerResponse = worker2.receive();
    
    // Pattern match on responses
    match (response1, response2) {
        (WorkerResponse::Completed(r1), WorkerResponse::Completed(r2)) => {
            println("Both completed: %s, %s", r1, r2);
        }
        _ => println("At least one failed"),
    }
    
    // Shutdown workers
    worker1.send(WorkerMessage::Shutdown);
    worker2.send(WorkerMessage::Shutdown);
    
    return 0;
}
```

### Code Example: Supervision Tree with Restart Strategies

```fusion
// Supervisor strategies
enum RestartStrategy {
    one_for_one,      // Restart only failed child
    one_for_all,      // Restart all children
    rest_for_one,     // Restart failed and subsequent children
}

// Worker actor
actor Worker {
    id: int,
    workload: int,
    
    fn init(id: int, workload: int) -> unit {
        self.id = id;
        self.workload = workload;
        
        // Simulate work
        for i in 0..workload {
            if random() < 0.1 {
                panic("Worker %d failed".format(self.id));
            }
            work();
        }
    }
}

// Supervisor with restart strategy
supervisor PoolSupervisor {
    strategy: one_for_one,
    max_restarts: 3,
    max_duration: 60,  // seconds
    
    children: [
        ChildSpec::new("worker1", || Worker::init(1, 100)),
        ChildSpec::new("worker2", || Worker::init(2, 200)),
        ChildSpec::new("worker3", || Worker::init(3, 300)),
    ],
    
    fn on_child_started(child: ActorRef) -> unit {
        println("Child %s started", child.name);
    }
    
    fn on_child_restarted(child: ActorRef, restarts: int) -> unit {
        println("Child %s restarted (%d times)", child.name, restarts);
    }
    
    fn on_child_failed(child: ActorRef, reason: string) -> unit {
        println("Child %s permanently failed: %s", child.name, reason);
    }
}

// Application supervision tree
supervisor AppSupervisor {
    strategy: rest_for_one,
    
    children: [
        ChildSpec::new("database", || DatabaseActor::init()),
        ChildSpec::new("cache", || CacheActor::init()),
        ChildSpec::new("pool", || PoolSupervisor::init()),
        ChildSpec::new("web", || WebActor::init()),
    ],
}

fn main() -> int {
    // Start application with supervision tree
    let app: ActorRef = Actor::spawn(AppSupervisor::init());
    
    // Application runs until interrupted
    signal::interrupt_handler(fn() {
        app.send(SupervisorMessage::Shutdown);
    });
    
    app.wait();
    
    return 0;
}
```

### Use Cases

- **Concurrent systems** with many independent components
- **Fault-tolerant applications** with automatic recovery
- **Distributed systems** with message-passing communication
- **Event-driven architectures** with actor-based event handling

### Common Patterns

1. **Let it crash**: Let actors fail and be restarted by supervisors
2. **Message routing**: Direct messages to specific actors
3. **Actor pools**: Manage groups of similar actors
4. **Watchers**: Monitor actor lifecycle events

---

## Custom Allocators

Custom allocators allow specialized memory management for different use cases, improving performance and reducing fragmentation.

### Per-Type Memory Allocation

```fusion
// Custom allocator trait
trait Allocator {
    fn allocate(size: int) -> *byte;
    fn deallocate(ptr: *byte, size: int) -> unit;
    fn reallocate(ptr: *byte, old_size: int, new_size: int) -> *byte;
}

// Type-specific allocator
struct TypedAllocator<T: type> {
    allocator: Box<dyn Allocator>,
    _phantom: PhantomData<T>,
}

impl<T> TypedAllocator<T> {
    fn new(allocator: Box<dyn Allocator>) -> TypedAllocator<T> {
        return TypedAllocator {
            allocator,
            _phantom: PhantomData,
        };
    }
    
    fn allocate(&self) -> *T {
        let ptr: *byte = self.allocator.allocate(size_of::<T>());
        return ptr as *T;
    }
    
    fn deallocate(&self, ptr: *T) -> unit {
        self.allocator.deallocate(ptr as *byte, size_of::<T>());
    }
}
```

### Code Example: Arena Allocator

```fusion
// Arena allocator for fast batch allocation
struct Arena {
    buffer: *byte,
    size: int,
    offset: int,
}

impl Arena {
    fn new(size: int) -> Arena {
        let buffer: *byte = sys::mmap(size);
        return Arena {
            buffer,
            size,
            offset: 0,
        };
    }
    
    fn alloc<T>(&mut self) -> *T {
        let align: int = align_of::<T>();
        let aligned_offset: int = (self.offset + align - 1) & !(align - 1);
        
        if aligned_offset + size_of::<T>() > self.size {
            panic("Arena overflow");
        }
        
        let ptr: *T = (self.buffer + aligned_offset) as *T;
        self.offset = aligned_offset + size_of::<T>();
        return ptr;
    }
    
    fn reset(&mut self) -> unit {
        self.offset = 0;
    }
    
    fn free(self) -> unit {
        sys::munmap(self.buffer, self.size);
    }
}

// Usage example
fn process_requests(requests: List<Request>) -> List<Response> {
    let mut arena: Arena = Arena::new(1024 * 1024);  // 1MB arena
    let mut responses: List<Response> = [];
    
    for request in requests {
        // Allocate from arena (fast, no individual frees)
        let temp: *TempData = arena.alloc::<TempData>();
        *temp = process_temp(request);
        
        let response: Response = generate_response(temp);
        responses.push(response);
    }
    
    // Arena is freed all at once at end of function
    arena.free();
    
    return responses;
}

fn main() -> int {
    let requests: List<Request> = load_requests();
    let responses: List<Response> = process_requests(requests);
    println("Processed %d requests", responses.len());
    return 0;
}
```

### Code Example: GPU Memory Allocator

```fusion
// GPU memory allocator for graphics/compute
struct GPUAllocator {
    device: Device,
    pool: MemoryPool,
}

impl GPUAllocator {
    fn new(device: Device) -> GPUAllocator {
        return GPUAllocator {
            device,
            pool: MemoryPool::new(device),
        };
    }
    
    fn alloc_buffer<T>(&self, size: int) -> GPUBuffer<T> {
        let ptr: *byte = self.pool.allocate(size);
        return GPUBuffer {
            ptr,
            size,
            device: self.device,
        };
    }
    
    fn alloc_texture(&self, width: int, height: int, format: TextureFormat) -> GPUTexture {
        let size: int = width * height * format.bytes_per_pixel();
        let ptr: *byte = self.pool.allocate(size);
        return GPUTexture {
            ptr,
            width,
            height,
            format,
            device: self.device,
        };
    }
    
    fn free_all(&self) -> unit {
        self.pool.free_all();
    }
}

// Example: Render pipeline with GPU allocator
fn render_frame(scene: Scene, allocator: GPUAllocator) -> Frame {
    // Allocate GPU resources
    let vertex_buffer: GPUBuffer<Vertex> = allocator.alloc_buffer(scene.vertices.len() * size_of::<Vertex>());
    let index_buffer: GPUBuffer<int> = allocator.alloc_buffer(scene.indices.len() * size_of::<int>());
    let color_texture: GPUTexture = allocator.alloc_texture(1920, 1080, TextureFormat::RGBA8);
    let depth_texture: GPUTexture = allocator.alloc_texture(1920, 1080, TextureFormat::Depth32);
    
    // Upload data to GPU
    vertex_buffer.upload(&scene.vertices);
    index_buffer.upload(&scene.indices);
    
    // Render
    let frame: Frame = render(scene, vertex_buffer, index_buffer, color_texture, depth_texture);
    
    // Free GPU resources
    allocator.free_all();
    
    return frame;
}

fn main() -> int {
    let device: Device = Device::init();
    let allocator: GPUAllocator = GPUAllocator::new(device);
    
    let scene: Scene = load_scene();
    let frame: Frame = render_frame(scene, allocator);
    display_frame(frame);
    
    return 0;
}
```

### Use Cases

- **Performance-critical code** where allocation overhead matters
- **Embedded systems** with limited memory
- **Game engines** with frame-based allocation
- **Real-time systems** needing deterministic allocation

### Common Patterns

1. **Arena allocation**: Batch allocate, batch free
2. **Pool allocation**: Pre-allocate fixed-size blocks
3. **Slab allocation**: Allocate from large memory slabs
4. **Bump allocation**: Simple pointer bumping

---

## Unsafe Provenance

Unsafe provenance tracks the origin of unsafe code, requiring formal proofs for unsafe operations.

### Formal Proof Requirements for Unsafe Code

```fusion
// Unsafe block with proof obligation
unsafe fn raw_pointer_deref(ptr: *int) -> int
    requires ptr.is_valid()
    requires ptr.is_aligned()
    ensures result == *ptr
{
    // Compiler verifies proof obligations
    return *ptr;
}

// Proof certificates
proof valid_pointer(ptr: *int) -> bool {
    return ptr != null and ptr.is_aligned() and ptr.is_in_bounds();
}

// Use unsafe with proof
fn safe_usage(ptr: *int) -> int {
    if proof::check(valid_pointer(ptr)) {
        return unsafe { raw_pointer_deref(ptr) };
    }
    panic("Invalid pointer");
}
```

### Code Example: Unsafe Block with Proof

```fusion
// Unsafe operations with proof requirements
unsafe fn vector_get_unchecked<T>(vec: Vector<T>, index: int) -> T
    requires index >= 0 and index < vec.len()
    ensures result == vec[index]
{
    // Proof obligation: index in bounds
    return vec.data[index];
}

// Proof generation
fn generate_bounds_proof(vec: Vector<int>, index: int) -> Proof {
    let mut proof: Proof = Proof::new();
    
    // Prove index >= 0
    proof.add_step(Step::Assert(index >= 0));
    
    // Prove index < vec.len()
    proof.add_step(Step::Assert(index < vec.len()));
    
    // Combine proofs
    proof.add_step(Step::Conclude("Index in bounds"));
    
    return proof;
}

// Verified usage
fn safe_vector_get(vec: Vector<int>, index: int) -> Option<int> {
    let proof: Proof = generate_bounds_proof(vec, index);
    
    if proof::verify(proof) {
        return Some(unsafe { vector_get_unchecked(vec, index) });
    }
    
    return None;
}

// Example with complex proof
fn matrix_multiply_unchecked(
    a: Matrix<M, K>,
    b: Matrix<K, N>,
    result: Matrix<M, N>,
    i: int,
    j: int
) -> unit
    requires i >= 0 and i < M
    requires j >= 0 and j < N
    requires result.rows == M and result.cols == N
{
    let mut sum: float = 0.0;
    for k in 0..K {
        sum += a[i][k] * b[k][j];
    }
    result[i][j] = sum;
}

fn main() -> int {
    let vec: Vector<int> = Vector::from([1, 2, 3, 4, 5]);
    let idx: int = 2;
    
    match safe_vector_get(vec, idx) {
        Some(val) => println("Value: %d", val),
        None => println("Index out of bounds"),
    }
    
    return 0;
}
```

### Code Example: Provenance Tracking

```fusion
// Provenance tracking for raw pointers
struct Provenance<T> {
    ptr: *T,
    origin: Origin,
    proof: Proof,
}

enum Origin {
    Stack,
    Heap,
    External,
    Allocator,
}

impl<T> Provenance<T> {
    fn from_stack(value: &T) -> Provenance<T> {
        return Provenance {
            ptr: value as *T,
            origin: Origin::Stack,
            proof: Proof::stack_allocation(value),
        };
    }
    
    fn from_heap(value: Box<T>) -> Provenance<T> {
        let ptr: *T = Box::into_raw(value);
        return Provenance {
            ptr,
            origin: Origin::Heap,
            proof::heap_allocation(ptr),
        };
    }
    
    fn deref(&self) -> &T
        requires self.proof.is_valid()
    {
        return unsafe { &*self.ptr };
    }
    
    fn into_box(self) -> Box<T>
        requires self.origin == Origin::Heap
    {
        return unsafe { Box::from_raw(self.ptr) };
    }
}

// Example: Safe pointer manipulation with provenance
fn process_data(data: *byte, length: int) -> string
    requires data.is_valid()
    requires length >= 0
    requires data + length <= data.end()
{
    let provenance: Provenance<[byte]> = Provenance {
        ptr: data as *[
byte],
        origin: Origin::External,
        proof: Proof::external_buffer(data, length),
    };
    
    let slice: &[byte] = provenance.deref();
    return String::from_utf8_lossy(slice);
}

// Provenance-aware memory management
struct ManagedMemory {
    ptr: *byte,
    size: int,
    provenance: Proof,
}

impl ManagedMemory {
    fn alloc(size: int) -> ManagedMemory {
        let ptr: *byte = unsafe { sys::malloc(size) };
        return ManagedMemory {
            ptr,
            size,
            provenance: Proof::heap_allocation(ptr, size),
        };
    }
    
    fn as_slice(&self) -> &[byte]
        requires self.proof.is_valid()
    {
        return unsafe { 
            core::slice::from_raw_parts(self.ptr, self.size)
        };
    }
    
    fn free(self) -> unit
        requires self.proof.is_valid()
    {
        unsafe { sys::free(self.ptr) };
    }
}

fn main() -> int {
    let data: *byte = get_external_data();
    let length: int = get_data_length();
    
    let result: string = process_data(data, length);
    println("Processed: %s", result);
    
    let mut mem: ManagedMemory = ManagedMemory::alloc(1024);
    let slice: &[byte] = mem.as_slice();
    println("Memory size: %d", slice.len());
    mem.free();
    
    return 0;
}
```

### Use Cases

- **Low-level systems programming** with safe abstractions
- **FFI boundaries** where proof of safety is required
- **Performance-critical code** that needs unsafe optimizations
- **Legacy code integration** with verified wrappers

### Common Patterns

1. **Proof certificates**: Generate and verify safety proofs
2. **Provenance tracking**: Track origin and validity of pointers
3. **Safe wrappers**: Provide safe interfaces to unsafe operations
4. **Formal verification**: Use theorem provers to verify safety

---

## Tips and Best Practices

1. **Use macros wisely**: Don't overuse them — they can hurt readability.
2. **Prefer async over threads**: For I/O-bound workloads.
3. **Use FFI sparingly**: Only when you need to call external code.
4. **Trust the compiler**: Let ownership and borrowing handle memory.
5. **Profile before optimizing**: Use the profiler to find real bottlenecks.

---

## Cross-References

- **Chapter 4**: Memory Safety for ownership details
- **Chapter 10**: Concurrency for async patterns
- **Chapter 12**: Tooling for debugging and profiling
