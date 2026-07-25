# Chapter 15: Reference

> Complete keyword list, operator precedence, type reference, API reference, compiler flags, and interop functions

---

## Complete Keyword List

### Reserved Keywords

| Keyword | Description |
|---------|-------------|
| `fn` | Function declaration |
| `let` | Variable binding |
| `mut` | Mutable variable |
| `const` | Constant declaration |
| `static` | Static variable |
| `return` | Return from function |
| `if` | Conditional statement |
| `else` | Alternative branch |
| `while` | While loop |
| `for` | For loop |
| `in` | Range/iterator |
| `match` | Pattern matching |
| `impl` | Implementation block |
| `trait` | Trait declaration |
| `where` | Where clause |
| `use` | Import declaration |
| `mod` | Module declaration |
| `pub` | Public visibility |
| `async` | Async function |
| `await` | Await expression |
| `struct` | Struct declaration |
| `enum` | Enum declaration |
| `type` | Type alias |
| `extern` | External function |

### Type Keywords

| Keyword | Description |
|---------|-------------|
| `int` | Signed 64-bit integer |
| `bool` | Boolean type |
| `string` | UTF-8 string |
| `void` | Unit type |
| `float` | 64-bit floating point |

### Literal Keywords

| Keyword | Description |
|---------|-------------|
| `true` | Boolean true |
| `false` | Boolean false |

### Special Tokens

| Token | Description |
|-------|-------------|
| `self` | Current instance |
| `super` | Parent module |
| `crate` | Current crate |

---

## Operator Precedence

From highest to lowest precedence:

| Precedence | Operator | Description |
|------------|----------|-------------|
| 1 | `!` | Logical NOT |
| 1 | `&` | Address-of |
| 1 | `*` | Dereference |
| 2 | `*` | Multiplication |
| 2 | `/` | Division |
| 2 | `%` | Modulo |
| 3 | `+` | Addition |
| 3 | `-` | Subtraction |
| 4 | `<<` | Left shift |
| 4 | `>>` | Right shift |
| 5 | `&` | Bitwise AND |
| 6 | `^` | Bitwise XOR |
| 7 | `\|` | Bitwise OR |
| 8 | `==` | Equal |
| 8 | `!=` | Not equal |
| 8 | `<` | Less than |
| 8 | `>` | Greater than |
| 8 | `<=` | Less or equal |
| 8 | `>=` | Greater or equal |
| 9 | `&&` | Logical AND |
| 10 | `\|\|` | Logical OR |

### Associativity

| Operators | Associativity |
|-----------|---------------|
| `!`, `&`, `*` (unary) | Right |
| `*`, `/`, `%` | Left |
| `+`, `-` | Left |
| `<<`, `>>` | Left |
| `&` | Left |
| `^` | Left |
| `\|` | Left |
| `==`, `!=`, `<`, `>`, `<=`, `>=` | None |
| `&&` | Left |
| `\|\|` | Left |

---

## Type Reference

### Primitive Types

| Type | Size | Range | Default |
|------|------|-------|---------|
| `int` | 8 bytes | -2^63 to 2^63-1 | 0 |
| `bool` | 1 byte | true/false | false |
| `string` | Variable | UTF-8 text | "" |
| `float` | 8 bytes | ±1.8e308 | 0.0 |
| `void` | 0 bytes | () | () |

### Integer Types

| Type | Size | Range |
|------|------|-------|
| `u8` | 1 byte | 0 to 255 |
| `u16` | 2 bytes | 0 to 65535 |
| `u32` | 4 bytes | 0 to 4294967295 |
| `u64` | 8 bytes | 0 to 18446744073709551615 |
| `i8` | 1 byte | -128 to 127 |
| `i16` | 2 bytes | -32768 to 32767 |
| `i32` | 4 bytes | -2147483648 to 2147483647 |
| `i64` | 8 bytes | -2^63 to 2^63-1 |

### Compound Types

| Type | Description | Example |
|------|-------------|---------|
| `[T; N]` | Fixed-size array | `[int; 5]` |
| `[T]` | Slice (view) | `[int]` |
| `(T, U, ...)` | Tuple | `(int, string)` |
| `*T` | Raw pointer | `*int` |
| `&T` | Shared reference | `&int` |
| `&mut T` | Mutable reference | `&mut int` |
| `Option<T>` | Optional value | `Option<int>` |
| `Result<T, E>` | Result type | `Result<int, string>` |

### Function Types

| Type | Description | Example |
|------|-------------|---------|
| `fn(T) -> U` | Function pointer | `fn(int) -> int` |
| `\|T\| U` | Closure | `\|x: int\| x * 2` |
| `fn(T, ...) -> U` | Variadic function | `fn(int, ...) -> int` |

---

## Standard Library API Reference

### std::io

| Function | Signature | Description |
|----------|-----------|-------------|
| `println` | `(fmt: string, ...)` | Print with newline |
| `print` | `(fmt: string, ...)` | Print without newline |
| `eprintln` | `(fmt: string, ...)` | Print to stderr |
| `eprint` | `(fmt: string, ...)` | Print to stderr without newline |
| `read_line` | `() -> string` | Read line from stdin |
| `read_bytes` | `(n: int) -> bytes` | Read N bytes from stdin |
| `write` | `(data: string)` | Write to stdout |
| `write_bytes` | `(data: bytes)` | Write bytes to stdout |

### std::fs

| Function | Signature | Description |
|----------|-----------|-------------|
| `read_to_string` | `(path: string) -> string` | Read entire file |
| `read_bytes` | `(path: string) -> bytes` | Read file as bytes |
| `read_lines` | `(path: string) -> Vec<string>` | Read file as lines |
| `write` | `(path: string, content: string)` | Write to file |
| `write_bytes` | `(path: string, content: bytes)` | Write bytes to file |
| `append` | `(path: string, content: string)` | Append to file |
| `exists` | `(path: string) -> bool` | Check if file exists |
| `is_dir` | `(path: string) -> bool` | Check if path is directory |
| `metadata` | `(path: string) -> Metadata` | Get file metadata |
| `create_dir` | `(path: string)` | Create directory |
| `create_dir_all` | `(path: string)` | Create directory recursively |
| `read_dir` | `(path: string) -> Vec<string>` | List directory |
| `remove_file` | `(path: string)` | Remove file |
| `remove_dir` | `(path: string)` | Remove empty directory |
| `remove_dir_all` | `(path: string)` | Remove directory recursively |
| `copy` | `(src: string, dst: string)` | Copy file |
| `rename` | `(src: string, dst: string)` | Rename/move file |

### std::math

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs` | `(x: int) -> int` | Absolute value |
| `absf` | `(x: float) -> float` | Absolute value (float) |
| `min` | `(a: int, b: int) -> int` | Minimum |
| `max` | `(a: int, b: int) -> int` | Maximum |
| `pow` | `(base: int, exp: int) -> int` | Power |
| `powf` | `(base: float, exp: float) -> float` | Power (float) |
| `sqrt` | `(x: float) -> float` | Square root |
| `cbrt` | `(x: float) -> float` | Cube root |
| `sin` | `(x: float) -> float` | Sine |
| `cos` | `(x: float) -> float` | Cosine |
| `tan` | `(x: float) -> float` | Tangent |
| `asin` | `(x: float) -> float` | Arc sine |
| `acos` | `(x: float) -> float` | Arc cosine |
| `atan` | `(x: float) -> float` | Arc tangent |
| `atan2` | `(y: float, x: float) -> float` | Arc tangent (2 args) |
| `log` | `(x: float) -> float` | Natural logarithm |
| `log2` | `(x: float) -> float` | Base-2 logarithm |
| `log10` | `(x: float) -> float` | Base-10 logarithm |
| `exp` | `(x: float) -> float` | Exponential (e^x) |
| `floor` | `(x: float) -> float` | Floor |
| `ceil` | `(x: float) -> float` | Ceiling |
| `round` | `(x: float) -> float` | Round |
| `pi` | `() -> float` | Pi constant |
| `e` | `() -> float` | Euler's number |
| `nan` | `() -> float` | Not a Number |
| `inf` | `() -> float` | Infinity |

### std::string

| Function | Signature | Description |
|----------|-----------|-------------|
| `len` | `(s: string) -> int` | String length |
| `push` | `(s: string, c: char) -> string` | Append character |
| `contains` | `(s: string, sub: string) -> bool` | Check substring |
| `starts_with` | `(s: string, prefix: string) -> bool` | Check prefix |
| `ends_with` | `(s: string, suffix: string) -> bool` | Check suffix |
| `find` | `(s: string, sub: string) -> Option<int>` | Find substring |
| `split` | `(s: string, delim: string) -> Vec<string>` | Split string |
| `join` | `(parts: Vec<string>, sep: string) -> string` | Join strings |
| `replace` | `(s: string, from: string, to: string) -> string` | Replace |
| `trim` | `(s: string) -> string` | Trim whitespace |
| `to_upper` | `(s: string) -> string` | Uppercase |
| `to_lower` | `(s: string) -> string` | Lowercase |
| `parse_int` | `(s: string) -> Option<int>` | Parse as int |
| `parse_float` | `(s: string) -> Option<float>` | Parse as float |
| `format` | `(fmt: string, ...) -> string` | Format string |

### std::collections

| Function | Signature | Description |
|----------|-----------|-------------|
| `Vec::new` | `() -> Vec<T>` | Create empty vector |
| `Vec::with_capacity` | `(cap: int) -> Vec<T>` | Create with capacity |
| `Vec::push` | `(v: Vec<T>, item: T)` | Append element |
| `Vec::pop` | `(v: Vec<T>) -> Option<T>` | Remove last element |
| `Vec::len` | `(v: Vec<T>) -> int` | Length |
| `Vec::is_empty` | `(v: Vec<T>) -> bool` | Check empty |
| `Vec::get` | `(v: Vec<T>, idx: int) -> Option<T>` | Get element |
| `Vec::set` | `(v: Vec<T>, idx: int, val: T)` | Set element |
| `Vec::contains` | `(v: Vec<T>, val: T) -> bool` | Check contains |
| `Vec::iter` | `(v: Vec<T>) -> Iter<T>` | Get iterator |
| `Vec::map` | `(v: Vec<T>, f: fn(T) -> U) -> Vec<U>` | Map elements |
| `Vec::filter` | `(v: Vec<T>, f: fn(T) -> bool) -> Vec<T>` | Filter elements |
| `Vec::fold` | `(v: Vec<T>, init: U, f: fn(U, T) -> U) -> U` | Fold elements |
| `Vec::sort` | `(v: Vec<T>)` | Sort in place |
| `Vec::reverse` | `(v: Vec<T>)` | Reverse in place |
| `Vec::clone` | `(v: Vec<T>) -> Vec<T>` | Clone vector |
| `HashMap::new` | `() -> HashMap<K, V>` | Create empty map |
| `HashMap::get` | `(m: HashMap<K, V>, key: K) -> Option<V>` | Get value |
| `HashMap::set` | `(m: HashMap<K, V>, key: K, val: V)` | Set value |
| `HashMap::contains` | `(m: HashMap<K, V>, key: K) -> bool` | Check key exists |
| `HashMap::keys` | `(m: HashMap<K, V>) -> Vec<K>` | Get all keys |
| `HashMap::values` | `(m: HashMap<K, V>) -> Vec<V>` | Get all values |
| `HashMap::len` | `(m: HashMap<K, V>) -> int` | Length |

### std::crypto

| Function | Signature | Description |
|----------|-----------|-------------|
| `generate_keypair` | `() -> HybridKeyPair` | Generate hybrid key pair |
| `generate_signing_key` | `() -> HybridSigningKey` | Generate signing key |
| `encrypt` | `(key: bytes, data: string) -> bytes` | Encrypt data |
| `decrypt` | `(key: bytes, data: bytes) -> string` | Decrypt data |
| `sign` | `(key: HybridSigningKey, data: string) -> bytes` | Sign data |
| `verify` | `(key: HybridVerifyKey, data: string, sig: bytes) -> bool` | Verify signature |
| `hash` | `(data: string) -> bytes` | SHA-256 hash |
| `hash_bcrypt` | `(data: string, cost: int) -> bytes` | Bcrypt hash |
| `verify_bcrypt` | `(data: string, hash: bytes) -> bool` | Verify bcrypt |
| `generate_random` | `(len: int) -> bytes` | Generate random bytes |

### std::net

| Function | Signature | Description |
|----------|-----------|-------------|
| `TcpListener::bind` | `(addr: string, port: int) -> TcpListener` | Bind TCP listener |
| `TcpStream::connect` | `(host: string, port: int) -> TcpStream` | Connect to TCP server |
| `listener.accept` | `() -> TcpStream` | Accept connection |
| `listener.local_addr` | `() -> string` | Get local address |
| `stream.read` | `(buf_size: int) -> string` | Read from stream |
| `stream.write` | `(data: string)` | Write to stream |
| `stream.flush` | `()` | Flush stream |
| `stream.close` | `()` | Close stream |
| `HttpServer::new` | `(addr: string) -> HttpServer` | Create HTTP server |
| `server.route` | `(method: string, path: string, handler: fn(Request) -> Response)` | Add route |
| `server.start` | `(port: int)` | Start server |
| `HttpClient::new` | `() -> HttpClient` | Create HTTP client |
| `client.get` | `(url: string) -> Response` | GET request |
| `client.post` | `(url: string, body: string) -> Response` | POST request |
| `client.put` | `(url: string, body: string) -> Response` | PUT request |
| `client.delete` | `(url: string) -> Response` | DELETE request |

### std::async

| Function | Signature | Description |
|----------|-----------|-------------|
| `channel` | `() -> (Sender<T>, Receiver<T>)` | Create channel |
| `spawn` | `(f: fn()) -> Fiber` | Spawn fiber |
| `spawn_blocking` | `(f: fn()) -> Fiber` | Spawn blocking fiber |
| `yield` | `()` | Yield control |
| `sleep` | `(ms: int)` | Sleep for milliseconds |
| `sleep_async` | `(ms: int) -> Future<void>` | Async sleep |
| `JoinSet::new` | `() -> JoinSet<T>` | Create join set |
| `set.spawn` | `(f: fn() -> T)` | Spawn into set |
| `set.join_next` | `() -> Option<T>` | Join next completed |
| `Mutex::new` | `(val: T) -> Mutex<T>` | Create mutex |
| `mutex.lock` | `() -> MutexGuard<T>` | Acquire lock |
| `RwLock::new` | `(val: T) -> RwLock<T>` | Create RW lock |
| `rwlock.read` | `() -> ReadGuard<T>` | Acquire read lock |
| `rwlock.write` | `() -> WriteGuard<T>` | Acquire write lock |
| `Arc::new` | `(val: T) -> Arc<T>` | Create atomic reference |
| `arc.clone` | `() -> Arc<T>` | Clone reference |

### std::profiler

| Function | Signature | Description |
|----------|-----------|-------------|
| `start` | `(name: string)` | Start profiling section |
| `stop` | `(name: string)` | Stop profiling section |
| `report` | `()` | Print profile report |
| `dump` | `(path: string)` | Dump profile to file |
| `reset` | `()` | Reset profiler |

### std::log

| Function | Signature | Description |
|----------|-----------|-------------|
| `set_level` | `(level: Level)` | Set log level |
| `trace` | `(fmt: string, ...)` | Log trace message |
| `debug` | `(fmt: string, ...)` | Log debug message |
| `info` | `(fmt: string, ...)` | Log info message |
| `warn` | `(fmt: string, ...)` | Log warning message |
| `error` | `(fmt: string, ...)` | Log error message |

---

## Effects Module API

### Effect Management

| Function | Signature | Description |
|----------|-----------|-------------|
| `effect_perform` | `(effect: Effect, value: T) -> R` | Perform an effect with a value, returning the handler's result |
| `effect_handle` | `(effect: Effect, handler: Handler) -> Handle` | Create a handler for an effect, returns a handle for later installation |
| `effect_register` | `(effect: Effect) -> EffectId` | Register a new effect in the global registry |
| `effect_list` | `() -> Vec<EffectId>` | List all registered effects |

### Handler Construction

| Function | Signature | Description |
|----------|-----------|-------------|
| `handler_new` | `(effect: Effect) -> Handler` | Create a new handler for the given effect |
| `handler_add_operation` | `(handler: Handler, name: string, f: fn(T) -> R) -> Handler` | Add an operation handler to a handler chain |
| `handler_set_handler` | `(handler: Handler, f: fn(Effect, T) -> R) -> Handler` | Set the fallback handler for unmatched operations |
| `handler_install` | `(handle: Handle) -> void` | Install a previously created handler handle into the current scope |

### Built-in Effects

| Effect | Description | Operations |
|--------|-------------|------------|
| `IO` | Input/output operations | `print`, `read`, `write` |
| `State` | Mutable state management | `get`, `set`, `modify` |
| `Async` | Asynchronous computation | `spawn`, `await`, `sleep` |
| `Error` | Error handling | `raise`, `catch`, `retry` |
| `Log` | Logging operations | `trace`, `debug`, `info`, `warn`, `error` |
| `Network` | Network I/O operations | `http_get`, `http_post`, `tcp_connect` |

---

## Types Module API

### Linear Types

| Function | Signature | Description |
|----------|-----------|-------------|
| `linear_new` | `(value: T) -> Linear<T>` | Create a new linear value (can only be used once) |
| `linear_use` | `(linear: Linear<T>) -> T` | Consume the linear value, returning its inner value |
| `linear_check` | `(linear: Linear<T>) -> bool` | Check if the linear value has been consumed |
| `linear_protocol_check` | `(linear: Linear<T>, protocol: Protocol) -> bool` | Verify that usage follows a protocol specification |

### Dependent Types

| Function | Signature | Description |
|----------|-----------|-------------|
| `dependent_new` | `(value: T, predicate: fn(T) -> bool) -> Dependent<T>` | Create a dependent-typed value with an invariant predicate |
| `dependent_check` | `(dep: Dependent<T>) -> bool` | Check if the current value satisfies the predicate |
| `dependent_refine` | `(dep: Dependent<T>, new_pred: fn(T) -> bool) -> Dependent<T>` | Refine the predicate to a more specific constraint |

### Refinement Types

| Function | Signature | Description |
|----------|-----------|-------------|
| `refinement_new` | `(value: T, predicate: fn(T) -> bool) -> Refinement<T>` | Create a refinement type wrapping a value |
| `refinement_check` | `(r: Refinement<T>) -> bool` | Check if the value satisfies the refinement |
| `refinement_meets_predicate` | `(r: Refinement<T>, pred: fn(T) -> bool) -> bool` | Verify against an additional predicate |

### Dynamic Types

| Function | Signature | Description |
|----------|-----------|-------------|
| `dynamic_new` | `(value: T, type_tag: string) -> Dynamic<T>` | Create a dynamically-typed value with runtime type tag |
| `dynamic_typeof` | `(d: Dynamic<T>) -> string` | Get the runtime type tag of the dynamic value |
| `dynamic_cast` | `(d: Dynamic<T>, target: string) -> Option<T>` | Attempt to cast to a target type at runtime |

---

## Control Module API

### Continuations

| Function | Signature | Description |
|----------|-----------|-------------|
| `call_cc` | `(f: fn(Cont<T>) -> T) -> T` | Call with current continuation; `f` receives a continuation that can be invoked later |
| `cont_invoke` | `(cont: Cont<T>, value: T) -> void` | Invoke a captured continuation with a value (non-local return) |
| `cont_capture` | `() -> Cont<T>` | Capture the current continuation and return it as a value |
| `cont_restore` | `(cont: Cont<T>) -> void` | Restore execution to a previously captured continuation |

### Coroutines

| Function | Signature | Description |
|----------|-----------|-------------|
| `coroutine_new` | `(f: fn(Cont<T>) -> void) -> Coroutine<T>` | Create a new coroutine from a function that yields values |
| `coroutine_resume` | `(co: Coroutine<T>, value: T) -> Option<T>` | Resume a coroutine, optionally passing a value; returns the yielded value or None if complete |
| `coroutine_yield` | `(co: Cont<T>, value: T) -> void` | Yield a value from inside a coroutine back to the resumer |

---

## Security Module API

### Capabilities

| Function | Signature | Description |
|----------|-----------|-------------|
| `cap_new` | `(resource: Resource, permissions: Vec<string>) -> Cap` | Create a new capability granting specified permissions on a resource |
| `cap_grant` | `(cap: Cap, new_perms: Vec<string>) -> Cap` | Grant additional permissions to a capability (returns new cap) |
| `cap_revoke` | `(cap: Cap, perms: Vec<string>) -> Cap` | Revoke specific permissions from a capability |
| `cap_check` | `(cap: Cap, perm: string) -> bool` | Check if a capability has a specific permission |
| `cap_verify` | `(cap: Cap) -> bool` | Verify a capability is still valid and unrevoked |

### Sandboxing

| Function | Signature | Description |
|----------|-----------|-------------|
| `sandbox_new` | `(name: string) -> Sandbox` | Create a new isolated sandbox environment |
| `sandbox_add_cap` | `(sandbox: Sandbox, cap: Cap) -> Sandbox` | Add a capability to a sandbox |
| `sandbox_execute` | `(sandbox: Sandbox, f: fn() -> T) -> T` | Execute a function within the sandbox's capability constraints |

### Unsafe and Proofs

| Function | Signature | Description |
|----------|-----------|-------------|
| `unsafe_new` | `(description: string) -> Unsafe` | Mark a block as requiring unsafe authorization |
| `unsafe_verify` | `(u: Unsafe, proof: Proof) -> bool` | Verify an unsafe block with a proof |
| `proof_new` | `(claim: string, evidence: fn() -> bool) -> Proof` | Create a proof for a safety claim |
| `proof_check` | `(proof: Proof) -> bool` | Check if a proof is still valid |

---

## Dispatch Module API

### Multimethods

| Function | Signature | Description |
|----------|-----------|-------------|
| `multimethod_new` | `(name: string, dispatch_fn: fn(T) -> string) -> Multimethod` | Create a new multimethod with a dispatch function that selects a method by name |
| `multimethod_add` | `(mm: Multimethod, name: string, impl: fn(T) -> R) -> Multimethod` | Register a method implementation for a given dispatch key |
| `multimethod_dispatch` | `(mm: Multimethod, value: T) -> R` | Dispatch on a value, calling the appropriate method implementation |

---

## Actors Module API

### Actor System

| Function | Signature | Description |
|----------|-----------|-------------|
| `actor_new` | `(name: string, handler: fn(Message) -> void) -> Actor` | Create a new actor with a message handler |
| `actor_send` | `(actor: Actor, message: Message) -> void` | Send a message to an actor (fire-and-forget) |
| `actor_ask` | `(actor: Actor, message: Message) -> T` | Send a message and wait for a response (blocking) |
| `actor_broadcast` | `(actors: Vec<Actor>, message: Message) -> void` | Broadcast a message to multiple actors |

### Supervisors

| Function | Signature | Description |
|----------|-----------|-------------|
| `supervisor_new` | `(name: string, strategy: RestartStrategy) -> Supervisor` | Create a new supervisor with a restart strategy |
| `supervisor_add_child` | `(sup: Supervisor, actor: Actor) -> Supervisor` | Add an actor as a supervised child |
| `supervisor_start` | `(sup: Supervisor) -> void` | Start the supervisor, enabling automatic restart on failure |

---

## TCO Module API

### Tail-Call Optimization

| Function | Signature | Description |
|----------|-----------|-------------|
| `tco_detect` | `(f: fn() -> T) -> TcoInfo` | Analyze a function and detect tail-call optimization opportunities |
| `tco_optimize` | `(f: fn() -> T) -> OptimizedFn` | Optimize a function by converting eligible recursive calls to tail calls |
| `tco_verify` | `(f: fn() -> T) -> bool` | Verify that a function has been correctly optimized for tail calls |

### Compilation Stages

| Function | Signature | Description |
|----------|-----------|-------------|
| `stage_compile` | `(source: string) -> AST` | Compile source code into an abstract syntax tree |
| `stage_load` | `(ast: AST) -> IR` | Load an AST into intermediate representation |
| `stage_runtime` | `(ir: IR) -> RuntimeModule` | Load an IR module into the runtime for execution |
| `stage_partial_eval` | `(ir: IR) -> IR` | Perform partial evaluation (constant folding) on IR |

---

## Compiler Module API

### Feature Toggles

| Function | Signature | Description |
|----------|-----------|-------------|
| `feature_toggle_register` | `(name: string, default: bool) -> FeatureToggle` | Register a new feature toggle with a default state |
| `feature_toggle_resolve` | `(toggle: FeatureToggle, config: FeatureConfig) -> bool` | Resolve the final state of a toggle given a configuration |
| `feature_toggle_check_conflicts` | `(toggles: Vec<FeatureToggle>) -> Vec<Conflict>` | Check a set of toggles for mutual exclusion conflicts |

### Witness System

| Function | Signature | Description |
|----------|-----------|-------------|
| `witness_generate` | `(feature: Feature, context: CompileContext) -> Witness` | Generate a witness for a feature's presence in a compile context |
| `witness_verify` | `(witness: Witness, expected: Feature) -> bool` | Verify that a witness corresponds to the expected feature |
| `witness_check_conflicts` | `(witnesses: Vec<Witness>) -> Vec<WitnessConflict>` | Check a set of witnesses for conflicts |

### Feature Compilation

| Function | Signature | Description |
|----------|-----------|-------------|
| `compile_with_features` | `(source: string, features: Vec<string>) -> Compilation` | Compile source with specific features enabled |
| `validate_features` | `(source: string, required: Vec<string>) -> ValidationResult` | Validate that source code uses only the specified features |

---

## Integration Module API

> All 30 cross-feature functions enabling composition of effects, linear types, capabilities, actors, continuations, and more.

| Function | Signature | Description |
|----------|-----------|-------------|
| `integrate_effect_linear` | `(effect: Effect, linear: Linear<T>) -> Linear<R>` | Apply an effect to a linear value |
| `integrate_effect_cap` | `(effect: Effect, cap: Cap) -> Cap` | Apply an effect to a capability |
| `integrate_effect_actor` | `(effect: Effect, actor: Actor) -> Actor` | Apply an effect to an actor |
| `integrate_linear_cap` | `(linear: Linear<T>, cap: Cap) -> Linear<T>` | Bind a linear value to a capability |
| `integrate_linear_actor` | `(linear: Linear<T>, actor: Actor) -> Actor` | Attach a linear value to an actor's context |
| `integrate_linear_cont` | `(linear: Linear<T>, cont: Cont<T>) -> Cont<T>` | Capture a linear value in a continuation |
| `integrate_cap_actor` | `(cap: Cap, actor: Actor) -> Actor` | Grant a capability to an actor |
| `integrate_cap_cont` | `(cap: Cap, cont: Cont<T>) -> Cont<T>` | Capture a capability in a continuation |
| `integrate_cap_sandbox` | `(cap: Cap, sandbox: Sandbox) -> Sandbox` | Add a capability to a sandbox |
| `integrate_actor_supervisor` | `(actor: Actor, sup: Supervisor) -> Supervisor` | Add an actor to a supervisor |
| `integrate_actor_multimethod` | `(actor: Actor, mm: Multimethod) -> Multimethod` | Dispatch actor messages through a multimethod |
| `integrate_cont_coroutine` | `(cont: Cont<T>, co: Coroutine<T>) -> Coroutine<T>` | Integrate a continuation with a coroutine |
| `integrate_coroutine_actor` | `(co: Coroutine<T>, actor: Actor) -> Actor` | Drive an actor with a coroutine |
| `integrate_effect_tco` | `(effect: Effect, tco_info: TcoInfo) -> TcoInfo` | Analyze effect usage for TCO opportunities |
| `integrate_linear_tco` | `(linear: Linear<T>, tco: OptimizedFn) -> OptimizedFn` | Optimize tail calls that consume linear values |
| `integrate_cap_tco` | `(cap: Cap, tco: OptimizedFn) -> OptimizedFn` | Optimize tail calls with capability checking |
| `integrate_feature_effect` | `(toggle: FeatureToggle, effect: Effect) -> Effect` | Conditionally enable an effect via feature toggle |
| `integrate_feature_linear` | `(toggle: FeatureToggle, linear: Linear<T>) -> Linear<T>` | Conditionally enable linear type checking via feature toggle |
| `integrate_feature_cap` | `(toggle: FeatureToggle, cap: Cap) -> Cap` | Conditionally enable capability enforcement via feature toggle |
| `integrate_feature_actor` | `(toggle: FeatureToggle, actor: Actor) -> Actor` | Conditionally enable actor supervision via feature toggle |
| `integrate_witness_effect` | `(witness: Witness, effect: Effect) -> bool` | Verify an effect's witness in a compilation context |
| `integrate_witness_linear` | `(witness: Witness, linear: Linear<T>) -> bool` | Verify linear type usage via witness |
| `integrate_witness_cap` | `(witness: Witness, cap: Cap) -> bool` | Verify capability usage via witness |
| `integrate_witness_actor` | `(witness: Witness, actor: Actor) -> bool` | Verify actor setup via witness |
| `integrate_sandbox_effect` | `(sandbox: Sandbox, effect: Effect) -> Sandbox` | Add an effect to a sandbox |
| `integrate_sandbox_actor` | `(sandbox: Sandbox, actor: Actor) -> Sandbox` | Add an actor to a sandbox |
| `integrate_multimethod_dispatch_mm` | `(mm: Multimethod, actor: Actor) -> void` | Dispatch messages from an actor through a multimethod |
| `integrate_proof_cap` | `(proof: Proof, cap: Cap) -> bool` | Verify a capability with a proof |
| `integrate_proof_linear` | `(proof: Proof, linear: Linear<T>) -> bool` | Verify a linear value with a proof |
| `integrate_proof_actor` | `(proof: Proof, actor: Actor) -> bool` | Verify an actor's safety with a proof |

---

## Quantum Functions

### std::quantum

| Function | Signature | Description |
|----------|-----------|-------------|
| `Qubit::new` | `() -> Qubit` | Create qubit |
| `Qubit::zero` | `() -> Qubit` | Create \|0⟩ qubit |
| `Qubit::one` | `() -> Qubit` | Create \|1⟩ qubit |
| `Qubit::plus` | `() -> Qubit` | Create \|+⟩ qubit |
| `Qubit::minus` | `() -> Qubit` | Create \|−⟩ qubit |
| `Qubit::hadamard` | `(q: Qubit)` | Apply Hadamard gate |
| `Qubit::pauli_x` | `(q: Qubit)` | Apply Pauli-X gate |
| `Qubit::pauli_y` | `(q: Qubit)` | Apply Pauli-Y gate |
| `Qubit::pauli_z` | `(q: Qubit)` | Apply Pauli-Z gate |
| `Qubit::phase` | `(q: Qubit, theta: float)` | Apply phase gate |
| `Qubit::t_gate` | `(q: Qubit)` | Apply T gate |
| `Qubit::measure` | `(q: Qubit) -> int` | Measure qubit |
| `cnot` | `(control: Qubit, target: Qubit)` | CNOT gate |
| `toffoli` | `(c1: Qubit, c2: Qubit, target: Qubit)` | Toffoli gate |
| `swap` | `(q1: Qubit, q2: Qubit)` | Swap gate |
| `Circuit::new` | `(qubits: int, classical: int) -> Circuit` | Create circuit |
| `circuit.h` | `(q: int)` | Apply Hadamard to qubit index |
| `circuit.x` | `(q: int)` | Apply Pauli-X to qubit index |
| `circuit.cx` | `(c: int, t: int)` | Apply CNOT to qubit indices |
| `circuit.measure` | `(q: int, c: int)` | Measure qubit to classical bit |
| `circuit.run` | `(shots: int) -> HashMap<int, int>` | Run circuit |
| `Circuit::bell_state` | `() -> Circuit` | Create Bell state circuit |
| `Circuit::grover` | `(oracle: fn(Qubit), n: int) -> Circuit` | Create Grover circuit |
| `Circuit::qft` | `(n: int) -> Circuit` | Create QFT circuit |

---

## AI/ML Functions

### std::ml

| Function | Signature | Description |
|----------|-----------|-------------|
| `tensor` | `(data: ...) -> Tensor` | Create tensor |
| `Tensor::from_data` | `(data: Vec<float>, shape: Vec<int>) -> Tensor` | Create from data |
| `zeros` | `(shape: [int]) -> Tensor` | Create zero tensor |
| `ones` | `(shape: [int]) -> Tensor` | Create ones tensor |
| `randn` | `(shape: [int]) -> Tensor` | Create random tensor |
| `Tensor::shape` | `(t: Tensor) -> [int]` | Get shape |
| `Tensor::len` | `(t: Tensor) -> int` | Get element count |
| `Tensor::reshape` | `(t: Tensor, shape: [int]) -> Tensor` | Reshape |
| `Tensor::transpose` | `(t: Tensor) -> Tensor` | Transpose |
| `Tensor::to_vec` | `(t: Tensor) -> Vec<float>` | Convert to vector |

### std::ml::activations

| Function | Signature | Description |
|----------|-----------|-------------|
| `relu` | `(x: Tensor) -> Tensor` | ReLU activation |
| `sigmoid` | `(x: Tensor) -> Tensor` | Sigmoid activation |
| `tanh` | `(x: Tensor) -> Tensor` | Tanh activation |
| `softmax` | `(x: Tensor) -> Tensor` | Softmax activation |
| `leaky_relu` | `(x: Tensor, alpha: float) -> Tensor` | Leaky ReLU |
| `elu` | `(x: Tensor, alpha: float) -> Tensor` | ELU activation |
| `gelu` | `(x: Tensor) -> Tensor` | GELU activation |
| `swish` | `(x: Tensor) -> Tensor` | Swish activation |

### std::ml::loss

| Function | Signature | Description |
|----------|-----------|-------------|
| `cross_entropy` | `(pred: Tensor, target: Tensor) -> Tensor` | Cross-entropy loss |
| `mse` | `(pred: Tensor, target: Tensor) -> Tensor` | MSE loss |
| `mae` | `(pred: Tensor, target: Tensor) -> Tensor` | MAE loss |
| `huber` | `(pred: Tensor, target: Tensor, delta: float) -> Tensor` | Huber loss |
| `binary_cross_entropy` | `(pred: Tensor, target: Tensor) -> Tensor` | Binary cross-entropy |

### std::ml::optim

| Function | Signature | Description |
|----------|-----------|-------------|
| `SGD::new` | `(lr: float) -> SGD` | Create SGD optimizer |
| `Adam::new` | `(lr: float, beta1: float, beta2: float) -> Adam` | Create Adam optimizer |
| `optimizer.step` | `()` | Perform optimization step |
| `optimizer.zero_grad` | `()` | Zero gradients |

### std::ml::nn

| Function | Signature | Description |
|----------|-----------|-------------|
| `Linear::new` | `(in_features: int, out_features: int) -> Linear` | Create linear layer |
| `linear.forward` | `(x: Tensor) -> Tensor` | Forward pass |
| `Conv2d::new` | `(in_ch: int, out_ch: int, kernel: int) -> Conv2d` | Create conv layer |
| `conv2d.forward` | `(x: Tensor) -> Tensor` | Forward pass |
| `LayerNorm::new` | `(normalized_shape: [int]) -> LayerNorm` | Create layer norm |
| `layernorm.forward` | `(x: Tensor) -> Tensor` | Forward pass |
| `Embedding::new` | `(num_embeddings: int, dim: int) -> Embedding` | Create embedding |
| `embedding.forward` | `(x: Tensor) -> Tensor` | Forward pass |

---

## Interop Functions

### std::polyglot

| Function | Signature | Description |
|----------|-----------|-------------|
| `eval` | `(lang: string, code: string) -> ForeignValue` | Evaluate foreign code |
| `import_module` | `(name: string) -> ForeignModule` | Import foreign module |
| `cast` | `(val: ForeignValue) -> T` | Cast foreign value |
| `is_type` | `(val: ForeignValue, type: string) -> bool` | Check foreign type |

### std::polyglot::JsRuntime

| Function | Signature | Description |
|----------|-----------|-------------|
| `JsRuntime::new` | `() -> JsRuntime` | Create JS runtime |
| `runtime.eval` | `(code: string) -> ForeignValue` | Evaluate JS |
| `runtime.call` | `(fn: string, args: Vec<ForeignValue>) -> ForeignValue` | Call JS function |
| `runtime.eval_async` | `(code: string) -> Future<ForeignValue>` | Evaluate async JS |
| `runtime.set_global` | `(name: string, val: ForeignValue)` | Set global variable |

### std::interop::SharedBuffer

| Function | Signature | Description |
|----------|-----------|-------------|
| `SharedBuffer::new` | `(size: int) -> SharedBuffer` | Create shared buffer |
| `buffer.read_bytes` | `(offset: int, len: int) -> bytes` | Read bytes |
| `buffer.write_bytes` | `(offset: int, data: bytes)` | Write bytes |
| `buffer.read_f64` | `(offset: int) -> float` | Read f64 |
| `buffer.write_f64` | `(offset: int, val: float)` | Write f64 |
| `buffer.as_slice` | `() -> &[u8]` | Get as slice |

### std::interop::ForeignHandle

| Function | Signature | Description |
|----------|-----------|-------------|
| `ForeignHandle::new` | `(lang: string, id: string) -> ForeignHandle` | Create handle |
| `handle.call_method` | `(name: string, args: Vec<ForeignValue>) -> ForeignValue` | Call method |
| `handle.to_i64` | `() -> int` | Convert to int |
| `handle.to_f64` | `() -> float` | Convert to float |
| `handle.to_string` | `() -> string` | Convert to string |
| `handle.release` | `()` | Release handle |

### std::interop::ThreadPool

| Function | Signature | Description |
|----------|-----------|-------------|
| `ThreadPool::new` | `(lang: string, workers: int) -> ThreadPool` | Create thread pool |
| `pool.submit` | `(f: fn() -> T) -> JoinHandle<T>` | Submit task |
| `pool.shutdown` | `()` | Shutdown pool |

---

## Runtime Functions

### std::runtime

| Function | Signature | Description |
|----------|-----------|-------------|
| `process::args` | `() -> Vec<string>` | Get command line args |
| `process::env` | `(name: string) -> Option<string>` | Get environment variable |
| `process::set_env` | `(name: string, val: string)` | Set environment variable |
| `process::exit` | `(code: int)` | Exit process |
| `time::now` | `() -> int` | Current time (ms) |
| `time::sleep` | `(ms: int)` | Sleep |
| `time::timestamp` | `() -> int` | Unix timestamp |
| `random::i64` | `() -> int` | Random integer |
| `random::f64` | `() -> float` | Random float |
| `random::bytes` | `(n: int) -> bytes` | Random bytes |
| `gc::collect` | `()` | Force GC |
| `gc::stats` | `() -> GcStats` | GC statistics |

---

## Cloud/Mobile Functions

### std::cloud

| Function | Signature | Description |
|----------|-----------|-------------|
| `S3::new` | `(region: string) -> S3` | Create S3 client |
| `s3.put_object` | `(bucket: string, key: string, data: bytes)` | Upload object |
| `s3.get_object` | `(bucket: string, key: string) -> bytes` | Download object |
| `s3.list_objects` | `(bucket: string) -> Vec<string>` | List objects |
| `s3.delete_object` | `(bucket: string, key: string)` | Delete object |
| `DynamoDB::new` | `(region: string) -> DynamoDB` | Create DynamoDB client |
| `dynamo.put_item` | `(table: string, item: HashMap<string, AttributeValue>)` | Put item |
| `dynamo.get_item` | `(table: string, key: HashMap<string, AttributeValue>) -> Option<Item>` | Get item |
| `dynamo.query` | `(table: string, expr: string) -> Vec<Item>` | Query items |
| `Lambda::new` | `(region: string) -> Lambda` | Create Lambda client |
| `lambda.invoke` | `(function: string, payload: string) -> string` | Invoke function |

### std::mobile

| Function | Signature | Description |
|----------|-----------|-------------|
| `MobileApp::new` | `() -> MobileApp` | Create mobile app |
| `app.add_screen` | `(name: string, builder: fn() -> Widget)` | Add screen |
| `app.navigate` | `(screen: string)` | Navigate to screen |
| `app.run` | `()` | Run app |
| `Button::new` | `(label: string, on_click: fn()) -> Button` | Create button |
| `Text::new` | `(content: string) -> Text` | Create text |
| `TextInput::new` | `(hint: string) -> TextInput` | Create text input |
| `Image::new` | `(src: string) -> Image` | Create image |
| `List::new` | `(items: Vec<T>) -> List` | Create list |
| `Camera::capture` | `() -> Image` | Capture photo |
| `GPS::location` | `() -> (float, float)` | Get GPS location |
| `Notification::send` | `(title: string, body: string)` | Send notification |

---

## Compiler Flags Reference

### General Flags

| Flag | Description |
|------|-------------|
| `-o <path>` | Set output file path |
| `--opt-level <0-3>` | Optimization level |
| `--target <triple>` | Target triple |
| `--emit-llvm` | Emit LLVM IR |
| `--emit-bin` | Emit linked executable |
| `--lib` | Compile as library |

### Compilation Stages

| Flag | Description |
|------|-------------|
| `--parse-only` | Parse only |
| `--sema-only` | Semantic analysis only |

### Debug Flags

| Flag | Description |
|------|-------------|
| `--debug` | Include debug info |
| `--no-debug` | Exclude debug info |
| `--vortex` | Enable Vortex borrow checking |

### Linking Flags

| Flag | Description |
|------|-------------|
| `--link-lib <name>` | Link external library |
| `--lib-path <path>` | Library search path |

### Target Triples

| Triple | Platform |
|--------|----------|
| `x86_64-pc-windows-msvc` | Windows x64 |
| `x86_64-unknown-linux-gnu` | Linux x64 |
| `x86_64-apple-darwin` | macOS x64 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 |
| `aarch64-apple-darwin` | macOS ARM64 |
| `wasm32-unknown-unknown` | WebAssembly |
| `wasm32-wasi` | WebAssembly with WASI |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `FUSION_HOME` | Fusion installation directory |
| `FUSION_TARGET` | Default target triple |
| `FUSION_OPT_LEVEL` | Default optimization level |
| `FUSION_DEBUG` | Enable debug mode |
| `FUSION_LOG` | Log level (error, warn, info, debug, trace) |

---

## Tips for Reference Use

1. **Bookmark this chapter**: You'll come back to it often.
2. **Use IDE integration**: The LSP provides autocomplete and docs.
3. **Check compiler errors**: They often reference relevant documentation.
4. **Read the source**: The standard library is well-documented.
5. **Ask the community**: The Fusion community is helpful and welcoming.
6. **Use `fuc doc`**: Generate project-specific API docs.
7. **Check chapter-specific docs**: Each chapter has detailed usage examples.

---

## Cross-References

- **Chapter 1**: Getting Started for installation
- **Chapter 2**: Syntax for language basics
- **Chapter 6**: Standard Library for detailed API usage
- **Chapter 8**: Quantum Computing for quantum API details
- **Chapter 9**: Machine Learning for ML API details
- **Chapter 12**: Tooling for compiler and tool documentation
- **Chapter 16**: Polyglot Interoperability for interop API details
- **Chapter 17**: Fusion.toml Configuration for project settings