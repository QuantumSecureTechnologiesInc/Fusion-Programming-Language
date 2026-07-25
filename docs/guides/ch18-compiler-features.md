# Chapter 18: Compiler-Level Feature Enforcement

> Feature Toggle Engine, Interaction Witness, Transform Pipeline, Conflict Matrix, Compiler Verification, Transform Injection

---

## Feature Toggle Engine

The Feature Toggle Engine is Fusion's compiler-level mechanism for declaring, resolving, and enforcing advanced programming language features. Features are declared at the module level using the `uses:` directive and are resolved at compile time via dependency analysis and conflict detection.

### What Is a Feature Toggle Engine?

The Feature Toggle Engine integrates directly into the Fusion compiler pipeline. It:

1. Parses `uses:` declarations from each module
2. Builds a dependency graph across all modules in the program
3. Detects conflicts between incompatible features
4. Injects the appropriate compiler transforms in priority order
5. Enforces feature constraints at every stage of compilation

Features are not runtime flags — they alter the language semantics, type system behavior, and code generation strategy. Once a feature is declared, the compiler enforces its rules globally within that module.

### How Features Are Declared

Features are declared in the module header using `uses:` with a bracketed list:

```fusion
// mod.fu — Module declaration with feature toggles
module my_math_library::linear_algebra;

uses: [Effects, LinearTypes, DependentTypes];

pub fn matrix_multiply(
    a: Matrix<n, m>,           // DependentTypes: type depends on value
    b: Matrix<m, k>,           // LinearTypes: each matrix used exactly once
) -> Matrix<n, k>
    effect [AllocError]        // Effects: function may allocate
{
    let mut result = Matrix::zero(n, k);    // LinearTypes: result is linear
    for i in 0..n {
        for j in 0..k {
            let mut sum = 0;
            for l in 0..m {
                sum += a[i, l] * b[l, j];
            }
            result[i, j] = sum;
        }
    }
    return result;             // LinearTypes: ownership transfers to caller
}
```

### Feature Dependencies

Features may require other features. The compiler resolves these automatically:

```fusion
// The FormalVerification feature requires DependentTypes
uses: [FormalVerification];
// Compiler automatically enables DependentTypes:
// Resolved: [FormalVerification, DependentTypes]
```

Dependency chains are resolved transitively. If Feature A requires Feature B which requires Feature C, all three are enabled.

### Priority-Ordered Transform Injection

When multiple features are active, transforms are injected in a fixed priority order:

| Priority | Transform | Features |
|----------|-----------|----------|
| 1 | CPS Transform | Continuations |
| 2 | TCO Loop Transform | TCO |
| 3 | Linear Resource Tracking | LinearTypes |
| 4 | Capability Check Injection | CapabilitySecurity |
| 5 | Dependent Type Refinement | DependentTypes |
| 6 | Refinement Type Narrowing | RefinementTypes |
| 7 | Effect Handler Setup | Effects |
| 8 | Gradual Type Annotation | GradualTyping |
| 9 | Coroutine Frame Setup | Coroutines |
| 10 | Actor Mailbox Setup | Actors |
| 11 | Formal Proof Outline | FormalVerification |
| 12 | Type Provider Hook | TypeProviders |
| 13 | Effect Region Setup | EffectRegions |
| 14 | Provenance Tagging | UnsafeProvenance |
| 15 | Capability Gate Setup | CapabilitySecurity |
| 16 | Taint Flow Tracking | TaintTracking |

### Code Example: Declaring Features in mod.fu

```fusion
module secure_api::auth_handler;

uses: [CapabilitySecurity, LinearTypes, Effects];

pub struct Token {
    value: LinearBytes,            // LinearTypes: must be consumed or explicitly dropped
    issuer: string,
    expiry: Timestamp,
}

pub fn validate_token(token: Token) -> Result<Claims, AuthError>
    effect [ClockRead, KeyLookup]
{
    // LinearTypes: token.value is moved into hmac_verify — cannot reuse
    let claims = hmac_verify(token.value, token.issuer)?;
    if claims.expiry < now() {
        return Err(AuthError::Expired);
    }
    return Ok(claims);
}

pub fn mint_token(claims: Claims, key: Capability<Key>) -> Token
    effect [ClockRead, HmacSign]
{
    // CapabilitySecurity: must hold `Key` capability to call hmac_sign
    let value = hmac_sign(key, &claims);
    Token {
        value,
        issuer: claims.issuer,
        expiry: claims.expiry,
    }
}
```

### Code Example: Feature Dependency Resolution

```fusion
// mod.fu — Demonstrating automatic dependency resolution
module quantum_crypto::hybrid;

uses: [FormalVerification, DependentTypes, RefinementTypes];
// Resolved automatically to:
// [FormalVerification, DependentTypes, RefinementTypes]
// because FormalVerification → DependentTypes

pub fn encrypt_message(
    plaintext: &[u8],
    key: SymmetricKey,
) -> Ciphertext
    effect [AeadEncrypt]
{
    // RefinementTypes: output length depends on input
    // DependentTypes: nonce type encodes algorithm choice
    let nonce = RandomBytes::<12>::generate();
    let ct = aead_encrypt(key, nonce, plaintext);
    return ct;
}

// Formal verification: the compiler generates proof obligations
// that encrypt_message satisfies the contract:
//   decrypt(key, encrypt(key, m)) == m
// for all m in &[u8]
```

### Code Example: Conflict Detection and Resolution

```fusion
// mod.fu — This will fail to compile
module bad_example;

uses: [Continuations, TCO];
// ERROR: Conflict detected at compile time

// Compiler output:
// ┌─────────────────────────────────────────────────────┐
// │ COMPILER ERROR                                      │
// │                                                     │
// │ Feature conflict: Continuations ↔ TCO               │
// │                                                     │
// │ These features are fundamentally incompatible:      │
// │ • Continuations require capturing the call stack    │
// │ • TCO eliminates the call stack frame               │
// │                                                     │
// │ Suggested fix: Remove one of the conflicting        │
// │ features, or refactor to avoid the conflict.        │
// │                                                     │
// │ File: bad_example/mod.fu:3                          │
// └─────────────────────────────────────────────────────┘

// Resolution: Remove TCO, use explicit loop instead
module fixed_example;

uses: [Continuations];

pub fn process_all(items: Vec<Item>) -> Vec<Result> {
    let mut results = Vec::new();
    let mut remaining = items;

    while !remaining.is_empty() {
        let item = remaining.remove(0);
        // Continuations captured here — TCO not needed
        results.push(process_with_continuation(item));
    }
    return results;
}
```

---

## Interaction Witness

An Interaction Witness is a compile-time metadata structure that captures the compatibility signature of a module's feature usage. It enables the compiler to detect conflicts between modules before linking.

### What Is an Interaction Witness?

When a module declares its features, the compiler generates an Interaction Witness — a SHA-256 hash of the module's feature set, transform ordering, and semantic constraints. Before linking modules, the compiler verifies that all witnesses are compatible.

### Metadata Hash Generation (SHA-256)

The witness hash is computed from:

1. The sorted list of active features
2. The transform priority ordering
3. The semantic constraints each feature imposes (e.g., "no mutable globals" for LinearTypes)

```fusion
// Compiler internals — the witness for a module
struct InteractionWitness {
    module_name: string,
    features: Vec<Feature>,
    transforms: Vec<TransformId>,
    constraints: Vec<SemanticConstraint>,
    hash: Sha256,               // Computed from the above fields
}
```

### Conflict Detection Between Features

The compiler maintains a global conflict matrix (see Conflict Matrix section). When two modules are linked, their witnesses are checked against this matrix:

```fusion
// Module A
uses: [LinearTypes, DependentTypes];

// Module B
uses: [GradualTyping];

// Linking A + B → WITNESS CONFLICT
// GradualTyping conflicts with LinearTypes and DependentTypes
```

### Human-Readable Error Messages

The compiler generates detailed error messages when conflicts are detected:

```
┌──────────────────────────────────────────────────────────────┐
│ INTERACTION WITNESS CONFLICT                                 │
│                                                              │
│ Module: auth_handler  →  LinearTypes, CapabilitySecurity     │
│ Module: data_processor → GradualTyping                       │
│                                                              │
│ Conflicts found:                                             │
│   1. LinearTypes ↔ GradualTyping                             │
│      Linear types require static ownership tracking;         │
│      Gradual typing allows runtime type checks.              │
│      These semantics are mutually exclusive.                 │
│                                                              │
│ 2. CapabilitySecurity ↔ GradualTyping (transitive)           │
│    CapabilitySecurity requires static proof of authority;    │
│    Gradual typing defers checks to runtime.                  │
│                                                              │
│ Suggested fixes:                                             │
│   • Remove GradualTyping from data_processor                 │
│   • Use RefinementTypes instead of GradualTyping             │
│   • Split data_processor into typed and untyped submodules   │
└──────────────────────────────────────────────────────────────┘
```

### Code Example: Generating Witnesses

```fusion
// The compiler generates witnesses automatically.
// You can inspect them with:
//   fuc witness --module my_module

// mod.fu
module payment::processor;

uses: [Effects, LinearTypes, CapabilitySecurity];

// Compiler generates:
//   InteractionWitness {
//     module_name: "payment::processor",
//     features: [Effects, LinearTypes, CapabilitySecurity],
//     transforms: [
//       LinearResourceTracking,    // priority 3
//       CapabilityCheckInjection,  // priority 4
//       EffectHandlerSetup,        // priority 7
//       CapabilityGateSetup,       // priority 15
//     ],
//     constraints: [
//       NoAliasing(LinearBytes),
//       StaticCapabilityProof,
//       EffectDeclarationRequired,
//     ],
//     hash: "a3f8c1...9d2e"       // SHA-256 truncated
//   }
```

### Code Example: Verifying Compatibility

```fusion
// Check compatibility between two modules without linking
use std::compiler::witness;

fn check_compatibility() -> bool {
    let w1 = witness::load("payment::processor");
    let w2 = witness::load("payment::logger");

    match w1.verify_compatibility(&w2) {
        Ok(()) => {
            println("Modules are compatible");
            return true;
        }
        Err(conflicts) => {
            for c in conflicts {
                println("Conflict: %s ↔ %s", c.feature_a, c.feature_b);
                println("  Reason: %s", c.explanation);
            }
            return false;
        }
    }
}
```

### Code Example: Module-Level and Program-Level Witnesses

```fusion
// Program-level witness: the union of all module witnesses
// Generated automatically when compiling a full program

// main.fu
module my_program;

uses: [Effects, RefinementTypes];

// When compiled, the compiler produces a program-level witness
// that is the consistent merge of all module witnesses.
//
// If any module declares a conflicting feature, compilation fails.

use my_program::auth;        // uses: [CapabilitySecurity, LinearTypes]
use my_program::payments;    // uses: [Effects, LinearTypes]
use my_program::reports;     // uses: [RefinementTypes]

fn main() -> int {
    // Program-level witness is valid because:
    // • Effects, RefinementTypes, CapabilitySecurity, LinearTypes
    //   have no conflicts in the conflict matrix
    // • All dependencies are satisfied
    // • Transform priorities are consistent

    let session = auth::login()?;
    let receipt = payments::charge(&session, 49.99)?;
    let report = reports::generate(&session, &receipt);
    println("Report: %s", report);
    return 0;
}
```

---

## Transform Pipeline

The Transform Pipeline is the compiler's internal system for applying feature-specific code transformations. Each feature maps to one or more transforms that modify the intermediate representation (IR).

### The 16 Transform Implementations

| # | Transform | Feature | Description |
|---|-----------|---------|-------------|
| 1 | CPS Transform | Continuations | Converts direct-style code to continuation-passing style |
| 2 | TCO Loop Transform | TCO | Converts tail-recursive calls into loops |
| 3 | Linear Resource Tracking | LinearTypes | Inserts borrow-checker-style tracking for linear values |
| 4 | Capability Check Injection | CapabilitySecurity | Inserts runtime checks for capability-held authority |
| 5 | Dependent Type Refinement | DependentTypes | Refines types based on value-level constraints |
| 6 | Refinement Type Narrowing | RefinementTypes | Narrows types using predicates at branch points |
| 7 | Effect Handler Setup | Effects | Sets up effect handlers and performs effect inference |
| 8 | Gradual Type Annotation | GradualTyping | Inserts runtime type checks at erasure boundaries |
| 9 | Coroutine Frame Setup | Coroutines | Creates coroutine state machines and frame allocation |
| 10 | Actor Mailbox Setup | Actors | Creates actor mailboxes and message dispatch loops |
| 11 | Formal Proof Outline | FormalVerification | Generates proof obligations and verification conditions |
| 12 | Type Provider Hook | TypeProviders | Invokes type provider functions at compile time |
| 13 | Effect Region Setup | EffectRegions | Partitions effect scopes into isolated regions |
| 14 | Provenance Tagging | UnsafeProvenance | Tags pointers with origin metadata for safety |
| 15 | Capability Gate Setup | CapabilitySecurity | Inserts capability gates at resource boundaries |
| 16 | Taint Flow Tracking | TaintTracking | Tracks data flow through taint propagation rules |

### Transform Ordering and Priority

Transforms execute in strict priority order. Lower-numbered transforms run first, ensuring that foundational transformations (like CPS) are applied before dependent ones (like capability checks within CPS-transformed code).

```fusion
// The compiler processes transforms in this order:
//
//   Source Code
//       ↓
//   [1] CPS Transform (Continuations)
//       ↓
//   [2] TCO Loop Transform (TCO)
//       ↓
//   [3] Linear Resource Tracking (LinearTypes)
//       ↓
//   [4] Capability Check Injection (CapabilitySecurity)
//       ↓
//   ... (remaining transforms)
//       ↓
//   [16] Taint Flow Tracking (TaintTracking)
//       ↓
//   Final IR → Code Generation
```

### Code Example: CPS Transform for Continuations

```fusion
// Source code (direct style)
fn fibonacci(n: int) -> int {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// After CPS Transform (continuation-passing style)
fn fibonacci_cps(n: int, k: Continuation<int>) {
    if n <= 1 {
        k(n);                                        // tail call to continuation
    } else {
        fibonacci_cps(n - 1, |v1| {
            fibonacci_cps(n - 2, |v2| {
                k(v1 + v2);                          // combine results
            });
        });
    }
}

// Usage with reset/shift
fn main() -> int {
    let result = reset {
        fibonacci_cps(10, shift |k| k(0));
    };
    println("fib(10) = %d", result);
    return 0;
}
```

### Code Example: TCO Loop Transform

```fusion
// Source code (tail-recursive)
fn sum_list(items: Vec<int>, acc: int) -> int {
    match items {
        Nil => acc,
        Cons(head, tail) => sum_list(tail, acc + head),   // tail position
    }
}

// After TCO Transform (loop)
fn sum_list(items: Vec<int>, acc: int) -> int {
    let mut current_items = items;
    let mut current_acc = acc;
    loop {
        match current_items {
            Nil => return current_acc,
            Cons(head, tail) => {
                current_items = tail;
                current_acc = current_acc + head;
                continue;                // loop back instead of recursive call
            }
        }
    }
}
```

### Code Example: Linear Resource Tracking

```fusion
// Source code
use std::io::{File, Write};

fn write_config(path: string, data: Config) -> Result<(), IOError> {
    let file = File::create(path)?;
    file.write_all(data.to_json().as_bytes())?;
    // file is dropped here — LinearTypes ensures it was used exactly once
    return Ok(());
}

// After Linear Resource Tracking transform
fn write_config(path: string, data: Config) -> Result<(), IOError> {
    let file = File::create(path)?;                        // file: Linear<File>
    file.write_all(data.to_json().as_bytes())?;            // file moved into write_all
    // LinearTypes verify: file was used exactly once
    // No double-close, no use-after-close possible
    return Ok(());
}

// The following would fail compilation:
fn bad_write(path: string) -> Result<(), IOError> {
    let file = File::create(path)?;
    file.write_all(b"first")?;
    file.write_all(b"second")?;  // ERROR: file already consumed
    return Ok(());
    // Compiler output:
    // Linear error: `file` was consumed by write_all on line 3
    // but used again on line 4.
    // Consider cloning the file handle or restructuring.
}
```

### Code Example: Capability Check Injection

```fusion
// Source code
fn read_secret_file(path: string) -> Result<string, SecurityError> {
    let data = std::fs::read_to_string(path)?;
    return Ok(data);
}

// After Capability Check Injection transform
fn read_secret_file(path: string, cap: Capability<FileRead>) -> Result<string, SecurityError> {
    // Runtime check: caller must hold FileRead capability
    if !cap.grants(Authority::FileRead) {
        return Err(SecurityError::MissingCapability {
            required: "FileRead",
            file: path,
        });
    }
    let data = std::fs::read_to_string(path)?;
    return Ok(data);
}

// Calling code must provide the capability:
fn main() -> int {
    // CapabilitySecurity: must acquire FileRead capability first
    let cap = acquire_capability(Authority::FileRead)?;

    match read_secret_file("/etc/secret.txt".to_string(), cap) {
        Ok(data) => println("Secret: %s", data),
        Err(e) => println("Error: %s", e.to_string()),
    }
    return 0;
}
```

---

## Conflict Matrix

The Conflict Matrix defines all hard incompatibilities between features. These are enforced at compile time — the compiler will refuse to compile code that declares conflicting features.

### All 5 Hard Incompatibilities

#### 1. Continuations + TCO

| Feature A | Feature B | Reason |
|-----------|-----------|--------|
| Continuations | TCO | Continuations require capturing the call stack. TCO eliminates the stack frame, making continuation capture impossible. |

```
Error: Continuations ↔ TCO
Continuations capture the current continuation (the rest of the computation),
which requires a live stack frame. TCO (Tail Call Optimization) reuses the
current frame, destroying the continuation context. These semantics are
fundamentally incompatible.
```

**Resolution**: Use explicit loops with CPS manually, or choose one feature.

#### 2. CapabilitySecurity + UnsafeProvenance

| Feature A | Feature B | Reason |
|-----------|-----------|--------|
| CapabilitySecurity | UnsafeProvenance | Capability checks are statically enforced. UnsafeProvenance allows circumventing safety checks via raw pointer operations. |

```
Error: CapabilitySecurity ↔ UnsafeProvenance
CapabilitySecurity enforces that all resource access goes through
statically-proven capability holders. UnsafeProvenance allows creating
raw pointers that bypass safety checks, including capability gates.
Together they create a contradiction: capabilities are meaningless
if raw pointers can bypass them.
```

**Resolution**: Use CapabilitySecurity without UnsafeProvenance, or use standard safety features instead of capabilities.

#### 3. GradualTyping + LinearTypes

| Feature A | Feature B | Reason |
|-----------|-----------|--------|
| GradualTyping | LinearTypes | Gradual typing allows runtime type checks. Linear types require static ownership analysis. Runtime checks cannot enforce linear ownership. |

```
Error: GradualTyping ↔ LinearTypes
Gradual typing inserts runtime type checks at erasure boundaries,
deferring type safety to execution time. Linear types require static
proof that each value is used exactly once. A runtime check cannot
verify linear ownership because ownership is a compile-time concept.
```

**Resolution**: Use LinearTypes for safety-critical paths, GradualTyping for prototyping. Do not mix them in the same module.

#### 4. DependentTypes + GradualTyping

| Feature A | Feature B | Reason |
|-----------|-----------|--------|
| DependentTypes | GradualTyping | Dependent types encode invariants in the type system at compile time. Gradual typing weakens type guarantees to runtime checks, defeating the purpose of dependent types. |

```
Error: DependentTypes ↔ GradualTyping
Dependent types allow types to depend on values (e.g., Vec<n>),
providing compile-time guarantees about program behavior. Gradual
typing allows values to have dynamic types, making value-dependent
type checking impossible at compile time. The two systems directly
contradict each other.
```

**Resolution**: Use DependentTypes for formally verified code, GradualTyping for rapid prototyping. Keep them in separate modules.

#### 5. FormalVerification + GradualTyping

| Feature A | Feature B | Reason |
|-----------|-----------|--------|
| FormalVerification | GradualTyping | Formal verification proves correctness at compile time. Gradual typing defers checks to runtime, making formal proofs unsound. |

```
Error: FormalVerification ↔ GradualTyping
Formal verification generates proof obligations that are checked at
compile time. Gradual typing allows values to bypass static type
checks, meaning a proof cannot assume the types of all values.
The soundness of formal proofs depends on all types being statically
known.
```

**Resolution**: Use FormalVerification for core algorithms, GradualTyping for UI or scripting layers. Keep them isolated.

### Code Example: Conflict Error Messages

```fusion
// This module will fail to compile
module problematic;

uses: [Continuations, TCO, GradualTyping, LinearTypes];

// The compiler reports all conflicts simultaneously:

// ┌──────────────────────────────────────────────────────────────┐
// │ COMPILER ERROR: 4 conflicts detected in module "problematic" │
// │                                                              │
// │ 1. Continuations ↔ TCO                                       │
// │    Continuations require stack frames; TCO eliminates them.  │
// │                                                              │
// │ 2. GradualTyping ↔ LinearTypes                               │
// │    Runtime checks cannot enforce linear ownership.           │
// │                                                              │
// │ 3. GradualTyping ↔ DependentTypes (if DependentTypes        │
// │    were active — currently only through Continuations)       │
// │                                                              │
// │ 4. Transitive: TCO ↔ Continuations (primary conflict)       │
// │                                                              │
// │ Compilation aborted. Fix conflicts before retrying.          │
// └──────────────────────────────────────────────────────────────┘
```

### Code Example: Resolving Conflicts

```fusion
// RESOLUTION STRATEGY 1: Remove conflicting features
module solution_one;

uses: [Continuations, LinearTypes];
// Removed: TCO (conflicted with Continuations)
// Removed: GradualTyping (conflicted with LinearTypes)

// RESOLUTION STRATEGY 2: Split into separate modules
// module core_logic;
// uses: [DependentTypes, FormalVerification];
//
// module ui_layer;
// uses: [GradualTyping, Effects];

// RESOLUTION STRATEGY 3: Use compatible feature combinations
module solution_three;

uses: [Effects, RefinementTypes, Coroutines, Actors];
// All compatible — no conflicts in the matrix

pub fn process_order(order: Order) -> OrderResult
    effect [DatabaseWrite, NotificationSend]
{
    let validated = validate_order(order)?;           // RefinementTypes
    let result = persist_order(validated)?;           // Effects
    spawn_actor(NotificationActor::new(result));       // Actors
    return result;
}
```

---

## Cross-References

- **Chapter 4**: Memory Safety for ownership and borrowing basics
- **Chapter 10**: Concurrency for actors and effects in detail
- **Chapter 13**: Advanced for type system internals
- **Chapter 16**: Polyglot Interop for cross-language feature usage
- **Chapter 17**: Fusion.toml Configuration for feature flag configuration
