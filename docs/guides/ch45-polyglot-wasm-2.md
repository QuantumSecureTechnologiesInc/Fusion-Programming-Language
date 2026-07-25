# Chapter 45: The Bleeding Edge (WebAssembly & Polyglot 2.0)

WebAssembly is the most significant development in polyglot computing since FFI. It doesn't just let you call between languages — it gives every language the same execution model, the same memory semantics, and the same security guarantees. This chapter covers where Wasm fits in the polyglot landscape, what it solves, and what's still unsolved.

## Why Wasm Changes Everything

Traditional polyglot approaches have a fundamental problem: every language has its own runtime, its own memory model, and its own garbage collector. When you call from Rust into Python, you're bridging two completely different execution environments. That bridge is where most bugs live.

WebAssembly flips this model. Instead of bridging runtimes, you compile every language to the same target — the Wasm instruction set. Now Rust, Python, and Go all run inside the same Wasm runtime (Wasmtime, Wasmer, or WasmEdge), sharing the same memory model and security guarantees.

```
Traditional:    Rust  ──FFI──▶  Python  ──FFI──▶  Go
                (two different runtimes, two memory models)

Wasm:           Rust  ──compile──▶ Wasm ──┐
                Python ──compile──▶ Wasm ──┤──▶ Wasmtime
                Go     ──compile──▶ Wasm ──┘    (one runtime)
```

## Compiling Languages to Wasm

### Rust to Wasm

Rust has first-class Wasm support. This is the easiest and most mature path.

```toml
# Cargo.toml
[package]
name = "fusion-core"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Required for Wasm

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Strip debug symbols
```

```rust
// src/lib.rs
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UserRecord {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
}

#[wasm_bindgen]
pub fn process_user(json: &str) -> Result<String, JsError> {
    let mut user: UserRecord = serde_json::from_str(json)
        .map_err(|e| JsError::new(&format!("Invalid JSON: {e}")))?;

    // Business logic
    user.name = user.name.trim().to_string();
    user.roles.sort();
    user.roles.dedup();

    serde_json::to_string(&user)
        .map_err(|e| JsError::new(&format!("Serialization error: {e}")))
}

#[wasm_bindgen]
pub fn validate_user(json: &str) -> Result<bool, JsError> {
    let user: UserRecord = serde_json::from_str(json)
        .map_err(|e| JsError::new(&format!("Invalid JSON: {e}")))?;

    Ok(!user.name.is_empty()
        && user.email.contains('@')
        && !user.roles.is_empty())
}
```

```bash
# Build for Wasm
cargo build --target wasm32-unknown-unknown --release

# Or use wasm-pack for browser/Node.js integration
wasm-pack build --target web --release
```

### Python to Wasm (Pyodide)

Pyodide compiles CPython to Wasm, letting you run Python in the browser or in a Wasm runtime. It's slower than native Python but eliminates the FFI bridge entirely.

```python
# fusion_pyodide.py — Python compiled to Wasm via Pyodide
import json
from typing import Dict, Any, List

def process_user(json_str: str) -> str:
    """Process a user record. Runs inside Wasm."""
    user = json.loads(json_str)

    # Business logic
    user["name"] = user["name"].strip()
    user["roles"] = sorted(set(user["roles"]))

    return json.dumps(user)

def validate_user(json_str: str) -> bool:
    """Validate a user record. Runs inside Wasm."""
    user = json.loads(json_str)
    return (
        bool(user.get("name"))
        and "@" in user.get("email", "")
        and len(user.get("roles", [])) > 0
    )
```

```javascript
// Loading Pyodide and calling Python functions
async function callPythonInWasm() {
    const pyodide = await loadPyodide();
    await pyodide.loadPackage("micropip");

    // Load your Python module
    const pythonCode = await fetch('/fusion_pyodide.py').then(r => r.text());
    pyodide.runPython(pythonCode);

    // Call Python functions from JavaScript
    const result = pyodide.runPython(`
        process_user('{"id":1,"name":"  Alice  ","email":"a@b.com","roles":["user","admin"]}')
    `);

    console.log(JSON.parse(result));
    // { id: 1, name: "Alice", email: "a@b.com", roles: ["admin", "user"] }
}
```

### Go to Wasm

Go's Wasm support targets `wasm/wasi` (WebAssembly System Interface). It's heavier than Rust's output but functional.

```go
// main.go — Go compiled to Wasm
package main

import (
    "encoding/json"
    "fmt"
    "syscall/js"
)

type UserRecord struct {
    ID    int64    `json:"id"`
    Name  string   `json:"name"`
    Email string   `json:"email"`
    Roles []string `json:"roles"`
}

func processUser(this js.Value, args []js.Value) interface{} {
    input := args[0].String()

    var user UserRecord
    if err := json.Unmarshal([]byte(input), &user); err != nil {
        return js.ValueOf(map[string]interface{}{
            "error": err.Error(),
        })
    }

    // Business logic
    user.Name = trimSpace(user.Name)
    user.Roles = uniqueSorted(user.Roles)

    output, _ := json.Marshal(user)
    return js.ValueOf(string(output))
}

func main() {
    c := make(chan struct{})
    js.Global().Set("processUser", js.FuncOf(processUser))
    <-c // Block forever
}
```

```bash
# Build Go to Wasm
GOOS=js GOARCH=wasm go build -o fusion.wasm main.go

# Copy the Wasm support file
cp "$(go env GOROOT)/lib/wasm/wasm_exec.js" ./public/
```

### C# to Wasm (via .NET)

.NET 8+ has experimental Wasm support via NativeAOT-LLVM.

```csharp
// UserProcessor.cs — C# compiled to Wasm
using System.Text.Json;

namespace Fusion.Wasm;

public static class UserProcessor
{
    public static string ProcessUser(string json)
    {
        var user = JsonSerializer.Deserialize<UserRecord>(json)
            ?? throw new ArgumentException("Invalid JSON");

        // Business logic
        user.Name = user.Name.Trim();
        user.Roles = user.Roles.Distinct().Order().ToList();

        return JsonSerializer.Serialize(user);
    }

    public static bool ValidateUser(string json)
    {
        var user = JsonSerializer.Deserialize<UserRecord>(json);
        return user is not null
            && !string.IsNullOrEmpty(user.Name)
            && user.Email.Contains('@')
            && user.Roles.Count > 0;
    }
}

public record UserRecord(
    long Id,
    string Name,
    string Email,
    List<string> Roles
);
```

## Running Inside Wasmtime/Wasmer

Both Wasmtime and Wasmer are production-quality Wasm runtimes. They compile Wasm bytecode to native machine code at load time (AOT compilation) and execute it with near-native performance.

### Wasmtime Example

```rust
// host.rs — Rust host running Wasm modules
use wasmtime::*;

fn main() -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, "fusion_core.wasm")?;

    let mut store = Store::new(&engine, ());

    // Define host functions the Wasm module can call
    let log_func = Func::wrap(&mut store, |caller: Caller<'_, ()>, ptr: i32, len: i32| {
        // Read string from Wasm memory
        let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
        let data = &mut [0u8; 1024];
        memory.read(&caller, ptr as usize, data).unwrap();
        let msg = String::from_utf8_lossy(&data[..len as usize]);
        println!("[Wasm log]: {msg}");
    });

    // Instantiate the module
    let instance = Instance::new(&mut store, &module, &[log_func.into()])?;

    // Call exported function
    let process_user = instance.get_typed_func::<(i32, i32), i32>(&mut store, "process_user")?;

    let input = r#"{"id":1,"name":"Alice","email":"a@b.com","roles":["admin"]}"#;
    let input_bytes = input.as_bytes();

    // Allocate memory in Wasm and write input
    let alloc = instance.get_typed_func::<i32, i32>(&mut store, "alloc")?;
    let ptr = alloc.call(&mut store, input_bytes.len() as i32)?;

    let memory = instance.get_export(&mut store, "memory").unwrap().into_memory().unwrap();
    memory.write(&mut store, ptr as usize, input_bytes)?;

    // Call process_user
    let result_ptr = process_user.call(&mut store, (ptr, input_bytes.len() as i32))?;

    // Read result from Wasm memory
    let mut result_buf = [0u8; 4096];
    memory.read(&mut store, result_ptr as usize, &mut result_buf)?;
    let result = String::from_utf8_lossy(&result_buf[..result_buf.iter().position(|&b| b == 0).unwrap_or(4096)]);

    println!("Result: {result}");
    Ok(())
}
```

### Wasmer Example (Python Host)

```python
# wasmer_host.py — Python host running Wasm modules
from wasmer import Store, Module, Instance, Memory
from wasmer_compiler_cranelift import Compiler

# Load compiled Wasm module
store = Store(Compiler)
module = Module(store, open("fusion_core.wasm", "rb").read())
instance = Instance(module)

# Get exported memory
memory: Memory = instance.exports.memory

# Get exported functions
alloc = instance.exports.alloc
process_user = instance.exports.process_user

# Write input to Wasm memory
input_json = b'{"id":1,"name":"Alice","email":"a@b.com","roles":["admin"]}'
ptr = alloc(len(input_json))
memory.write(ptr, input_json)

# Call process_user
result_ptr = process_user(ptr, len(input_json))

# Read result from Wasm memory
result_bytes = memory.read(result_ptr, 4096)
result_str = result_bytes.split(b'\x00')[0].decode('utf-8')

print(f"Result: {result_str}")
```

## Capability-Based Security in Wasm

Wasm's security model is fundamentally different from traditional processes. Instead of "run with full permissions," Wasm uses capability-based security: a module can only do what its host explicitly allows.

### The Capability Model

```
Traditional process:
  ├── Can read any file the user can read
  ├── Can open any network connection
  ├── Can fork any child process
  └── Can access all environment variables

Wasm module:
  ├── Can only access memory explicitly given to it
  ├── Can only call functions the host explicitly imported
  ├── Can only use system resources the host explicitly provided
  └── Cannot escape its sandbox without host cooperation
```

### Implementing Capabilities

```rust
// capability_host.rs — Granting specific capabilities to a Wasm module
use wasmtime::*;

struct Capabilities {
    allowed_dirs: Vec<String>,
    allowed_network: bool,
    max_memory_pages: u32,
}

impl Capabilities {
    fn new() -> Self {
        Self {
            allowed_dirs: vec!["/tmp/sandbox".to_string()],
            allowed_network: false,
            max_memory_pages: 256, // 16 MB
        }
    }
}

fn main() -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, "untrusted_plugin.wasm")?;

    // Enforce memory limit
    let mut config = Config::default();
    config.max_memory_pages(256);
    let engine = Engine::new(&config)?;

    let mut store = Store::new(&engine, Capabilities::new());

    // Only expose read_file — no write, no network, no process spawning
    let read_file = Func::wrap(&mut store, |caller: Caller<'_, Capabilities>, ptr: i32, len: i32| -> i32 {
        let caps = caller.data();
        // Verify the path is in allowed directories
        // Return error if capability not granted
        -1 // placeholder
    });

    // Grant memory allocation
    let alloc = Func::wrap(&mut store, |size: i32| -> i32 {
        // Allocate within the memory limit
        0 // placeholder
    });

    let instance = Instance::new(&mut store, &module, &[
        Func::wrap(&mut store, read_file).into(),
        Func::wrap(&mut store, alloc).into(),
    ])?;

    // The Wasm module can ONLY call read_file and alloc.
    // It cannot open sockets, fork processes, or write files.
    Ok(())
}
```

## Memory Sandboxing Solves 90% of Memory-Handoff Problems

The biggest source of bugs in traditional polyglot systems is memory ownership. Who allocated that buffer? Who's responsible for freeing it? What happens if one language frees memory the other is still using?

Wasm eliminates this entire class of problems.

### The Memory Boundary Model

```
Traditional FFI:
  Rust allocates ──▶ Python reads ──▶ Who frees?
  (Segfault if Python frees Rust's memory, leak if nobody frees)

Wasm sandboxing:
  Module A memory:  [allocated by A, freed by A]
  Module B memory:  [allocated by B, freed by B]
  Host bridges:    [copy, don't share] — explicit data transfer
```

### Implementing Memory-Safe Data Transfer

```rust
// data_bridge.rs — Safe data transfer between Wasm modules
use wasmtime::*;

/// Transfer a string between two Wasm modules via the host.
/// Both modules have separate memories. Data is copied, not shared.
fn transfer_string(
    store_a: &mut Store<()>,
    instance_a: &Instance,
    store_b: &mut Store<()>,
    instance_b: &Instance,
) -> Result<()> {
    // Read from module A's memory
    let memory_a = instance_a.get_export(store_a, "memory").unwrap().into_memory().unwrap();
    let ptr_a = /* get source pointer from module A */;
    let len_a = /* get source length from module A */;
    let mut data = vec![0u8; len_a as usize];
    memory_a.read(store_a, ptr_a as usize, &mut data)?;

    // Allocate in module B's memory
    let alloc_b = instance_b.get_typed_func::<i32, i32>(store_b, "alloc")?;
    let ptr_b = alloc_b.call(store_b, len_a)?;

    // Write to module B's memory
    let memory_b = instance_b.get_export(store_b, "memory").unwrap().into_memory().unwrap();
    memory_b.write(store_b, ptr_b as usize, &data)?;

    // Module A can free its copy; Module B owns its copy
    // No shared ownership, no use-after-free, no double-free
    Ok(())
}
```

### Comparison: FFI vs Wasm Memory Models

| Concern              | Traditional FFI                    | Wasm Sandbox                          |
|----------------------|-------------------------------------|---------------------------------------|
| Buffer ownership     | Manual tracking required           | Host mediates; each module owns its own |
| Use-after-free       | Common source of segfaults         | Impossible (separate memories)         |
| Double-free          | Requires careful reference counting | Impossible (no shared ownership)       |
| Memory leaks         | Possible if ownership unclear       | Module memory freed when instance drops |
| Thread safety        | Mutexes required across languages   | Each module is single-threaded (by default) |
| Buffer overflow      | Requires bounds checking           | Wasm enforces bounds automatically    |

## Where the Industry Is Heading

### WASI (WebAssembly System Interface)

WASI is the emerging standard for Wasm modules to interact with the outside world. It provides capability-based access to:
- File system (with explicit directory grants)
- Networking (with explicit socket grants)
- Clocks and random number generation
- Environment variables (with explicit key grants)

```
WASI Preview 2 (current):
  ├── wasi-filesystem    — File I/O with capability grants
  ├── wasi-sockets       — TCP/UDP with explicit grants
  ├── wasi-http          — HTTP client/server
  ├── wasi-clocks        — Wall clock and monotonic clock
  └── wasi-random        — Cryptographic randomness

WASI Future:
  ├── wasi-messaging     — Message queues (Kafka, NATS)
  ├── wasi-database      — Database connections
  ├── wasi-ai            — ML inference
  └── wasi-component     — Component model for composition
```

### The Component Model

The Wasm Component Model is the next evolution: Wasm modules that can be composed like LEGO blocks, with typed interfaces defined in WIT (Wasm Interface Type).

```wit
// user.wit — Wasm Interface Type definition
package fusion:user;

interface user {
    record user-record {
        id: s64,
        name: string,
        email: string,
        roles: list<string>,
    }

    record error {
        code: string,
        message: string,
        field: option<string>,
    }

    variant result<T, E> {
        ok(T),
        err(E),
    }

    process-user: func(json: string) -> result<user-record, error>;
    validate-user: func(json: string) -> bool;
}

world fusion-user {
    export user;
}
```

### Industry Adoption Trajectory

```
2024: Early adoption
  ├── Fermyon Spin (Wasm-first serverless)
  ├── Cosmonic (Wasm mesh networking)
  └── Fastly Compute (Wasm edge computing)

2025: Growing ecosystem
  ├── Wasmtime 2.0 (production-grade WASI)
  ├── Wasmer 5.0 (package registry for Wasm)
  └── Docker + Wasm (container alternative)

2026+: Mainstream
  ├── Cloud providers offering Wasm runtimes
  ├── Kubernetes operators for Wasm workloads
  └── Polyglot systems defaulting to Wasm boundaries
```

## Practical Integration: Wasm + Fusion

Here's how to integrate Wasm into a Fusion polyglot system today:

```toml
# fusion.toml
[modules]
core = { path = "./target/wasm32-unknown-unknown/release/fusion_core.wasm" }
python = { path = "./bindings/python/fusion_pyodide.wasm" }
go = { path = "./bindings/go/fusion.wasm" }

[runtime]
engine = "wasmtime"
version = "24.0"
max_memory_pages = 512

[security]
capabilities = ["filesystem:read:/data", "network:localhost:5432"]
sandbox = "strict"
```

```rust
// Using Wasm modules in Fusion
use fusion::wasm::{WasmRuntime, Module};

fn main() -> Result<()> {
    let runtime = WasmRuntime::new("wasmtime", "24.0")?;

    // Load modules with specific capabilities
    let core = runtime.load_module(
        "core",
        "target/wasm32-unknown-unknown/release/fusion_core.wasm",
        Capabilities::new()
            .with_filesystem_read("/data")
            .with_max_memory(512),
    )?;

    // Call functions across module boundaries
    let result: UserRecord = core.call("process_user", input_json)?;

    // The host mediates all data transfer between modules
    // Memory is sandboxed; no module can access another's memory
    Ok(())
}
```

## Summary

WebAssembly is not just another polyglot technique — it's a paradigm shift. It replaces fragile FFI bridges with a sandboxed, capability-adopted, memory-safe execution model. The tradeoffs today are performance overhead (10-30% vs native) and ecosystem maturity (not all languages compile cleanly to Wasm yet). But for security-critical polyglot systems, Wasm is already the right choice. For everyone else, it's the direction the industry is heading.
