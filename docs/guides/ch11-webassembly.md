# Chapter 11: WebAssembly

> Compiling to WASM, limitations, running modules, and JavaScript interop

---

## Compiling to WASM

Fusion can compile directly to WebAssembly (WASM) for running in browsers and WASM runtimes.

### Basic WASM Compilation

```fusion
// hello.fu
fn main() -> int {
    println("Hello from WASM!");
    return 0;
}
```

```bash
# Compile to WASM
fuc hello.fu --target wasm32-unknown-unknown -o hello.wasm

# Compile with WASI support
fuc hello.fu --target wasm32-wasi -o hello.wasm
```

### Exported Functions

```fusion
// lib.fu - Library compiled to WASM

// Export function for JavaScript
pub fn add(a: int, b: int) -> int {
    return a + b;
}

pub fn multiply(a: int, b: int) -> int {
    return a * b;
}

pub fn greet(name: string) -> string {
    return "Hello, " + name + "!";
}
```

```bash
# Compile as library
fuc lib.fu --lib --target wasm32-unknown-unknown -o lib.wasm
```

### WASM with Memory

```fusion
// math.fu
pub fn fibonacci(n: int) -> int {
    if n <= 0 { return 0; }
    if n == 1 { return 1; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

pub fn factorial(n: int) -> int {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}

pub fn gcd(a: int, b: int) -> int {
    while b != 0 {
        let temp: int = b;
        b = a %% b;
        a = temp;
    }
    return a;
}
```

---

## WASM-Specific Limitations

### Memory Model

WASM uses a linear memory model. Fusion's memory management adapts to this:

```fusion
// WASM has linear memory - no pointer arithmetic outside bounds
fn main() -> int {
    // This is fine - stack allocation
    let x: int = 42;

    // This is fine - arrays use linear memory
    let arr: [int; 10] = [0; 10];

    // Pointers work within WASM memory
    let ptr: *int = &x;
    let val: int = *ptr;

    println("WASM memory works!");
    return 0;
}
```

### No OS Access

WASM modules run in a sandboxed environment:

```fusion
// These operations are limited or unavailable in WASM:
// - Direct file system access (use WASI)
// - Network access (use WebSocket API from JS)
// - Thread creation (use Web Workers from JS)

// Available in WASM:
fn main() -> int {
    // Math operations
    let result: float = 3.14 * 2.0;

    // String operations
    let greeting: string = "Hello, WASM!";

    // Array operations
    let arr: [int; 5] = [1, 2, 3, 4, 5];

    println("WASM computation: %f", result);
    return 0;
}
```

### WASI Support

For system access in WASM, use WASI (WebAssembly System Interface):

```fusion
use std::wasi;

fn main() -> int {
    // WASI allows file system access
    let content: string = wasi::fs::read_to_string("data.txt");
    println("File content: %s", content);

    // WASI allows environment variables
    let home: Option<string> = wasi::env::get("HOME");
    match home {
        Some(path) => println("HOME: %s", path),
        None => println("HOME not set"),
    }

    return 0;
}
```

---

## Running WASM Modules

### Using WASM Runtime

```javascript
// Load and run WASM module in JavaScript
async function runWasm() {
    const wasmBytes = await fetch('hello.wasm');
    const wasmModule = await WebAssembly.instantiateStreaming(wasmBytes);

    const { add, multiply, greet } = wasmModule.instance.exports;

    console.log(add(2, 3));        // 5
    console.log(multiply(4, 5));   // 20
    console.log(greet("World"));   // "Hello, World!"
}
```

### Node.js WASM

```javascript
const fs = require('fs');
const wasmBuffer = fs.readFileSync('hello.wasm');

WebAssembly.instantiate(wasmBuffer).then(wasmModule => {
    const { add, multiply } = wasmModule.instance.exports;

    console.log(add(10, 20));      // 30
    console.log(multiply(6, 7));   // 42
});
```

### WASM in Fusion

```fusion
use std::wasm;

fn main() -> int {
    // Load a WASM module
    let module: wasm::Module = wasm::Module::load("math.wasm");

    // Get exported functions
    let add_fn: wasm::Function = module.get_function("add");
    let multiply_fn: wasm::Function = module.get_function("multiply");

    // Call exported functions
    let result1: int = add_fn.call([1, 2]);
    let result2: int = multiply_fn.call([3, 4]);

    println("add(1, 2) = %d", result1);
    println("multiply(3, 4) = %d", result2);

    return 0;
}
```

---

## Interop with JavaScript

### Calling JavaScript from Fusion

```fusion
use std::js;

fn main() -> int {
    // Call JavaScript console.log
    js::call("console.log", ["Hello from Fusion!"]);

    // Call JavaScript function
    let result: int = js::eval("2 + 3");
    println("JS result: %d", result);

    // Access JavaScript global objects
    let window_width: int = js::get("window.innerWidth");

    return 0;
}
```

### Calling Fusion from JavaScript

```javascript
// JavaScript code
const fusionModule = await WebAssembly.instantiateStreaming(fetch('app.wasm'));

// Call Fusion functions
const result = fusionModule.instance.exports.processData(42);
console.log('Result from Fusion:', result);

// Pass strings to Fusion
const greeting = fusionModule.instance.exports.greet('JavaScript');
console.log(greeting);
```

### Shared Memory

```fusion
use std::js;

fn main() -> int {
    // Create shared memory buffer
    let buffer: js::ArrayBuffer = js::ArrayBuffer::new(1024);

    // Write data
    let view: js::Int32Array = js::Int32Array::new(buffer);
    view.set(0, 42);
    view.set(1, 100);

    // Pass to JavaScript
    js::call("processBuffer", [buffer]);

    return 0;
}
```

### Event Handling

```fusion
use std::js;

fn on_click(event: js::Event) {
    println("Clicked at (%d, %d)", event.clientX, event.clientY);
}

fn main() -> int {
    // Register event handler
    let button: js::Element = js::document::get_element_by_id("myButton");
    button.add_event_listener("click", on_click);

    // Register multiple handlers
    let input: js::Element = js::document::get_element_by_id("myInput");
    input.add_event_listener("input", |event: js::Event| {
        let value: string = event.target.value;
        println("Input changed: %s", value);
    });

    println("Event handlers registered");
    return 0;
}
```

---

## WASM Performance Tips

### Optimize for WASM

```fusion
// Good: Use simple types
pub fn compute(x: int, y: int) -> int {
    return x * y + x - y;
}

// Good: Avoid heap allocation in hot paths
pub fn process_array(arr: [int]) -> int {
    let mut sum: int = 0;
    for item in arr {
        sum = sum + item;
    }
    return sum;
}

// Good: Use bulk memory operations when possible
pub fn fill_array(arr: [int], value: int) {
    for i in 0..arr.len() {
        arr[i] = value;
    }
}
```

### Memory Management

```fusion
// WASM memory grows automatically but watch for limits
fn main() -> int {
    // Allocate within WASM memory limits
    let big_array: [int; 1000000] = [0; 1000000];

    // Use memory efficiently
    let result: int = process_array(big_array);

    println("Processed %d elements", big_array.len());
    return 0;
}
```

---

## Common Patterns

### WASM Module Pattern

```fusion
// Define module interface
pub struct AppState {
    counter: int,
    name: string,
}

impl AppState {
    pub fn new() -> AppState {
        return AppState {
            counter: 0,
            name: "Fusion App".to_string(),
        };
    }

    pub fn increment(mut self) -> int {
        self.counter = self.counter + 1;
        return self.counter;
    }

    pub fn get_name(self) -> string {
        return self.name;
    }
}

// Export for JavaScript
pub fn create_app() -> AppState {
    return AppState::new();
}

pub fn update_app(app: AppState) -> int {
    return app.increment();
}
```

### WASM Service Worker

```fusion
use std::wasm;

fn main() -> int {
    // Register service worker
    wasm::service_worker::register("sw.js");

    // Handle messages
    wasm::service_worker::on_message(|event: wasm::MessageEvent| {
        let data: string = event.data;
        println("Received: %s", data);

        // Send response
        event.source.send("Processed: " + data);
    });

    println("Service worker started");
    return 0;
}
```

---

## Tips and Best Practices

1. **Minimize WASM size**: Use `--opt-level 3` and strip debug info.
2. **Use WASI for I/O**: For file system and network access.
3. **Pass data efficiently**: Use shared memory instead of copying.
4. **Handle errors gracefully**: WASM has limited error reporting.
5. **Test in multiple runtimes**: V8, SpiderMonkey, and WASI runtimes may differ.

---

## Cross-References

- **Chapter 1**: Getting Started for compilation flags
- **Chapter 12**: Tooling for WASM debugging tools
- **Chapter 14**: Examples for complete WASM applications
