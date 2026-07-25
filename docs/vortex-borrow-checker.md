# Vortex Borrow Checker Implementation

## Overview

The Vortex borrow checker is Fusion's safety engine that prevents data races, use-after-free, and other memory safety violations through **entropic flow analysis** and **affine type tracking**. It implements a thermodynamic safety model where variables are treated as permission streams that can collide, dissipate, or remain intact.

### Core Concepts

- **Entropic Flow Model**: Variables have permission states that flow through the program
- **Affine Type System**: Values can be moved but not copied (except for Copy types)
- **Permission States**: Intact, Exclusive Borrowed, Shared Borrowed, Dissipated
- **Collision Detection**: Identifies conflicting access patterns at compile time
- **Chaos Vacuum Diagnostics**: Rich, conversational error messages explaining the "why" of safety violations

### Architecture

```
crates/fuc/src/
├── vortex.fu                    # Self-hosted Vortex Engine (Fusion source)
├── borrowck.rs                  # Rust borrow checker implementation
├── diagnostics/
│   └── chaos_vacuum.fu          # Rich diagnostic reporting system
└── sema.rs                      # Semantic analysis with borrow states
```

---

## 1. Vortex Engine (vortex.fu)

The self-hosted Fusion implementation of the Vortex borrow checker using the Entropic Flow safety model.

```fusion
// Fusion Self-Hosted Vortex Engine
// Implements the Entropic Flow safety model to prevent data races and use-after-free.

import std.mem;
import std.io;

struct Loan {
    target: string;
    is_exclusive: bool;
    origin_line: int;
}

struct ChaosVacuum {
    reports: **char;
    count: int;
}

/// Analyses the entropic state of a variable borrow.
fn verify_borrow(state: *LoanStream, target: string, exclusive: bool, line: int) -> bool {
    let i: int = 0;
    while (i < state.loan_count) {
        let existing: *Loan = &state.loans[i];
        if (existing.target == target) {
            // Collision Logic: Stream B (Mutable) repels everything.
            // Stream A (Immutable) only repels Stream B.
            if (existing.is_exclusive || exclusive) {
                io.print("Entropic Collision: Variable '");
                io.print(target);
                io.print("' has conflicting streams at line ");
                io.print_int(line);
                io.print("\n");
                return false;
            }
        }
        i = i + 1;
    }
    return true;
}
```

### Key Principles

1. **Stream A (Immutable/Shared)**: Multiple readers allowed
2. **Stream B (Mutable/Exclusive)**: Single writer, repels all other access
3. **Collision Detection**: Any conflict between streams triggers entropic collision
4. **Loan Tracking**: Each borrow is tracked with target, exclusivity, and origin

---

## 2. Borrow Checker (borrowck.rs)

The Rust implementation providing affine type tracking and move semantics validation.

```rust
//! Borrow Checking and Ownership Analysis (Vortex Base)
//! 
//! This module implements affine type tracking (move semantics).
//! It ensures that variables are not used after they have been moved.
use crate::types::*;
use std::collections::HashMap;

use crate::sema::{TypedProgram, TypedFunction, TypedStatement, TypedExpressionKind};
use crate::ast::{Span, Type};

#[derive(Clone, Debug, PartialEq)]
enum ValState {
    Uninitialized,
    Active,
    Moved(Span), // Tracks where it was moved for diagnostics
}

pub struct BorrowDiagnostic {
    pub span: Span,
    pub message: FString,
}

pub struct BorrowChecker {
    errors: FVec<BorrowDiagnostic>,
    env: FMap<FString, ValState>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            env: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, prog: &TypedProgram) -> FVec<BorrowDiagnostic> {
        for func in &prog.functions {
            self.check_function(func);
        }
        std::mem::replace(&mut self.errors, Vec::new())
    }

    fn check_function(&mut self, func: &TypedFunction) {
        self.env.clear();
        
        // Initialize parameters as Active
        for (name, _) in &func.params {
            self.env.insert(name.clone(), ValState::Active);
        }

        self.check_block(&func.body);
    }

    fn check_block(&mut self, block: &FVec<TypedStatement>) {
        for stmt in block {
            self.check_statement(stmt);
        }
    }

    fn check_statement(&mut self, stmt: &TypedStatement) {
        match stmt {
            TypedStatement::Let { name, value, .. } => {
                // Mark variable as uninitialized before checking the initializer
                self.env.insert(name.clone(), ValState::Uninitialized);
                self.check_expression(value);
                self.env.insert(name.clone(), ValState::Active);
            }
            TypedStatement::Assignment { target, value } => {
                self.check_expression(value);
                // If target is a variable, mark it active again (re-initialization)
                if let TypedExpressionKind::Variable(name) = &target.node {
                    self.env.insert(name.clone(), ValState::Active);
                } else {
                    self.check_expression(target);
                }
            }
            TypedStatement::Expression(expr) | TypedStatement::Return(Some(expr)) => {
                self.check_expression(expr);
            }
            TypedStatement::Return(None) => {}
            TypedStatement::If { cond, then_block, else_block } => {
                self.check_expression(cond);
                
                // Clone environment for branch analysis
                let env_before = self.env.clone();
                
                self.check_block(then_block);
                let env_then = self.env.clone();
                
                self.env = env_before;
                if let Some(else_b) = else_block {
                    self.check_block(else_b);
                }
                
                // Intersect environments (if moved in either branch, it's moved overall)
                for (name, state) in &env_then {
                    if let ValState::Moved(span) = state {
                        self.env.insert(name.clone(), ValState::Moved(span.clone()));
                    }
                }
            }
            TypedStatement::While { cond, body } => {
                self.check_expression(cond);
                self.check_block(body);
            }
        }
    }

    fn check_expression(&mut self, expr: &crate::sema::TypedExpression) {
        match &expr.node {
            TypedExpressionKind::Variable(name) => {
                if let Some(state) = self.env.get(name) {
                    if let ValState::Moved(moved_span) = state {
                        self.errors.push(BorrowDiagnostic {
                            span: expr.span.clone(),
                            message: format!("Use of moved value '{}'. It was previously moved at span {:?}", name, moved_span),
                        });
                    } else if !self.is_copy_type(&expr.ty) {
                        // Mark as moved if it's not a primitive/copy type
                        self.env.insert(name.clone(), ValState::Moved(expr.span.clone()));
                    }
                }
            }
            TypedExpressionKind::FunctionCall { args, .. } => {
                for arg in args {
                    self.check_expression(arg);
                }
            }
            TypedExpressionKind::BinaryOperation { left, right, .. } => {
                self.check_expression(left);
                self.check_expression(right);
            }
            TypedExpressionKind::ArrayLiteral(elems) => {
                for e in elems {
                    self.check_expression(e);
                }
            }
            TypedExpressionKind::StructLiteral { fields, .. } => {
                for (_, _, field_expr) in fields {
                    self.check_expression(field_expr);
                }
            }
            TypedExpressionKind::MemberAccess { base, .. } |
            TypedExpressionKind::AddressOf(base) |
            TypedExpressionKind::Dereference(base) => {
                self.check_expression(base);
            }
            TypedExpressionKind::Index { array, index } => {
                self.check_expression(array);
                self.check_expression(index);
            }
            TypedExpressionKind::Match { scrutinee, arms } => {
                self.check_expression(scrutinee);
                for (_, guard, body) in arms {
                    if let Some(g) = guard {
                        self.check_expression(g);
                    }
                    self.check_expression(body);
                }
            }
            _ => {} // Primitives and constants don't trigger moves
        }
    }

    /// Determines if a type implements copy semantics vs move semantics
    fn is_copy_type(&self, ty: &Type) -> FBool {
        match ty {
            Type::Int | Type::Bool | Type::Pointer(_) | Type::GenericParam(_) => true,
            // Structs, Arrays, and Strings are moved by default in Fusion
            Type::Struct(_) | Type::Array(_, _) | Type::String | Type::Slice(_) | Type::Closure(_, _) => false,
            _ => true,
        }
    }
}
```

### Move Semantics Rules

1. **Copy Types** (Int, Bool, Pointer): Copied on use, never moved
2. **Move Types** (Struct, Array, String, Slice, Closure): Moved on use
3. **Environment Tracking**: Each variable has state: Uninitialized → Active → Moved
4. **Branch Analysis**: Environment intersection for if/else (moved in any branch = moved overall)
5. **Re-initialization**: Assignment reactivates a variable

---

## 3. Chaos Vacuum Diagnostics (chaos_vacuum.fu)

Rich, conversational error reporting system that explains the thermodynamic "why" of safety violations.

```fusion
// Chaos Vacuum Diagnostics Engine - Rich Compiler Error Reporting
// Converted from FU Parts: chaos_vacuum_diagnostics_engine.rs
// Rationale: Replaces raw backtraces with descriptive conversational terminal reports
// that explain the thermodynamic "Why" of safety errors.

extern fn printf(fmt: string, ...) -> int;

/// Permission state constants (replaces Rust enum)
const PERM_DISSIPATED: int = 0;
const PERM_EXCLUSIVE_BORROWED: int = 1;
const PERM_SHARED_BORROWED: int = 2;
const PERM_INTACT: int = 3;

/// Event collision tracked by Vortex borrow checker
struct EventCollision {
    var_name: string,
    existing_state: int,       // PermissionState constant
    shared_borrow_count: int,  // Only valid for PERM_SHARED_BORROWED
    collision_span_start: int,
    collision_span_end: int,
}

/// Chaos Vacuum reporter for rich diagnostic output
struct ChaosVacuumReporter {
    source_filename: string,
    source_code: string,
}

fn reporter_new(filename: string, code: string) -> ChaosVacuumReporter {
    ChaosVacuumReporter {
        source_filename: filename,
        source_code: code,
    }
}

/// Map permission state to human-readable description
fn state_description(state: int, count: int) -> string {
    if state == PERM_DISSIPATED {
        return "consumed or moved into a different execution scope";
    }
    if state == PERM_EXCLUSIVE_BORROWED {
        return "exclusively borrowed by a mutable writer";
    }
    if state == PERM_SHARED_BORROWED {
        return "borrowed immutably by active shared readers";
    }
    return "residing intact inside standard local scope";
}

/// Publish a colorful conversational terminal report showing exactly where
/// the permission flow collided and how to resolve the conflict safely.
fn publish_collision_report(reporter: ChaosVacuumReporter, event: &EventCollision) {
    printf("Entropic Collision Detected in %s\n", (*event).var_name);
    printf("============================================================\n");
    printf("Variable flow '%s' suffered a permission stream intersection.\n", (*event).var_name);

    let desc: string = state_description((*event).existing_state, (*event).shared_borrow_count);
    printf("\nAnalysis of Flow Collision:\n");
    printf("  * The resource was already: %s.\n", desc);
    printf("  * You attempted to access or borrow it at position %d-%d\n", (*event).collision_span_start, (*event).collision_span_end);

    printf("\nRemediation Advice:\n");
    if (*event).existing_state == PERM_DISSIPATED {
        printf("    To repair this flow, allocate a fresh resource or structure your code to complete\n");
        printf("    the work before passing ownership to subsequent execution scopes.\n");
    }
    if (*event).existing_state == PERM_EXCLUSIVE_BORROWED || (*event).existing_state == PERM_SHARED_BORROWED {
        printf("    Fusion's Vortex Engine strictly forbids conflicting read/write access.\n");
        printf("    Wrap the borrow blocks inside explicit scope boundaries using '{ ... }'\n");
        printf("    to allow exclusive permission frames to exit before access.\n");
    }
    printf("============================================================\n\n");
}

/// Simple collision test accessible from compiler integration
fn test_collision_report() -> int {
    printf("=== Chaos Vacuum Diagnostics Test ===\n");

    let collision: EventCollision = EventCollision {
        var_name: "my_resource",
        existing_state: PERM_EXCLUSIVE_BORROWED,
        shared_borrow_count: 0,
        collision_span_start: 42,
        collision_span_end: 60,
    };

    let reporter: ChaosVacuumReporter = reporter_new("test.fu", "let x = my_resource;");
    publish_collision_report(reporter, &collision);

    printf("=== Diagnostics Engine Operational ===\n");
    return 0;
}
```

### Permission States

| State | Constant | Description |
|-------|----------|-------------|
| **Dissipated** | `PERM_DISSIPATED` | Value consumed/moved to different scope |
| **Exclusive** | `PERM_EXCLUSIVE_BORROWED` | Mutable borrow (single writer) |
| **Shared** | `PERM_SHARED_BORROWED` | Immutable borrows (multiple readers) |
| **Intact** | `PERM_INTACT` | Value in standard local scope |

### Diagnostic Output Example

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

## 4. Safety Annotations

The Vortex system recognizes special annotations in source code:

### @unsafe
Marks code that bypasses safety checks. Requires explicit manual control.

```fusion
@unsafe
fn dangerous_operation(ptr: *int) -> int {
    return *ptr; // Manual memory access
}
```

**Requirement**: Must be paired with `@manual_memory` or `@borrowed`

### @manual_memory
Declares explicit manual memory lifecycle control.

```fusion
@unsafe
@manual_memory
fn alloc_buffer(size: int) -> *byte {
    // Manual allocation
}
```

### @borrowed
Indicates borrowed reference with lifetime management.

```fusion
@borrowed
fn process_data(data: &Vec<int>) -> int {
    // Borrowed access
}
```

---

## 5. Borrow Checking Algorithm

### Phase 1: Environment Initialization
```
1. Clear environment for each function
2. Mark all parameters as Active
3. Prepare for statement traversal
```

### Phase 2: Statement Analysis
```
Let Binding:
  1. Mark variable as Uninitialized
  2. Check initializer expression
  3. Mark variable as Active

Assignment:
  1. Check value expression
  2. If target is variable → mark Active (re-initialization)
  3. Otherwise → check target expression

If/Else:
  1. Clone environment (env_before)
  2. Check then_block → env_then
  3. Restore env_before
  4. Check else_block
  5. Intersect: moved in either branch = moved overall

While:
  1. Check condition
  2. Check body
```

### Phase 3: Expression Analysis
```
Variable Reference:
  - If Moved → Error: "Use of moved value"
  - If Copy type → No state change
  - If Move type → Mark as Moved

Function Call:
  - Check all arguments recursively
  - Arguments may be moved

Binary Operation:
  - Check left operand
  - Check right operand

Struct Literal:
  - Check all field expressions
  - Fields may be moved into struct
```

### Phase 4: Move vs Copy Classification
```
Copy Types (never moved):
  - Int, Bool
  - Pointers (*)
  - Generic parameters

Move Types (moved on use):
  - Structs
  - Arrays
  - Strings
  - Slices
  - Closures
```

---

## 6. Integration with Compiler Pipeline

The Vortex borrow checker runs after semantic analysis and before IR lowering:

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
Optimization → Optimized IR
    ↓
Code Generation → LLVM/WASM/Native
```

### Compiler Flags

```bash
# Enable Vortex borrow checking
fuc --vortex input.fu -o output.exe

# With safety annotations
fuc --vortex --unsafe-allow input.fu -o output.exe
```

---

## 7. Error Categories

### Use After Move
```fusion
let s: string = "hello";
let t: string = s;      // s is moved
printf(s);              // ERROR: Use of moved value 's'
```

### Conflicting Borrows
```fusion
let mut data: Vec<int> = vec![1, 2, 3];
let ref1: &Vec<int> = &data;    // Shared borrow
let ref2: &mut Vec<int> = &mut data; // ERROR: Conflicting exclusive borrow
printf(ref1[0]);
```

### Double Mutable Borrow
```fusion
let mut x: int = 42;
let r1: &mut int = &mut x;
let r2: &mut int = &mut x;  // ERROR: Already exclusively borrowed
```

### Dissipated Resource
```fusion
fn consume(s: string) {
    // s is consumed/moved
}

let msg: string = "test";
consume(msg);
printf(msg);  // ERROR: Resource dissipated
```

---

## 8. Thermodynamic Safety Model

The Vortex engine models program execution as a thermodynamic system:

### Entropic Flow Principles

1. **Conservation of Ownership**: Every resource has exactly one owner at any time
2. **Entropy Increase**: Moving a resource increases its entropy (dissipation)
3. **Stream Collision**: Conflicting access patterns create entropic collisions
4. **Permission Dissipation**: Resources lose permissions when moved or consumed

### Flow Types

- **Stream A (Shared/Immutable)**: Multiple readers, low entropy
- **Stream B (Exclusive/Mutable)**: Single writer, high entropy
- **Collision**: Stream A + Stream B = Entropic violation

### Remediation Strategies

1. **Scope Boundaries**: Use `{ ... }` to limit borrow lifetimes
2. **Early Completion**: Finish work before moving resources
3. **Fresh Allocation**: Create new resources when needed
4. **Explicit Lifetimes**: Use `@borrowed` annotations for clarity

---

## 9. Testing the Vortex Engine

### Test: Collision Report
```fusion
fn test_collision_report() -> int {
    let collision: EventCollision = EventCollision {
        var_name: "my_resource",
        existing_state: PERM_EXCLUSIVE_BORROWED,
        shared_borrow_count: 0,
        collision_span_start: 42,
        collision_span_end: 60,
    };
    
    let reporter: ChaosVacuumReporter = reporter_new("test.fu", "let x = my_resource;");
    publish_collision_report(reporter, &collision);
    
    return 0;
}
```

### Test: Borrow Checker
```rust
fn test_borrow_check() {
    let mut checker = BorrowChecker::new();
    let prog = parse_and_typecheck("test.fu");
    let errors = checker.check_program(&prog);
    
    for error in errors {
        println!("{} at {:?}", error.message, error.span);
    }
}
```

---

## 10. Performance Characteristics

### Time Complexity
- **Environment Operations**: O(1) average (HashMap)
- **Statement Analysis**: O(n) where n = statement count
- **Expression Analysis**: O(m) where m = expression tree depth
- **Total**: O(n + m) linear in program size

### Space Complexity
- **Environment Storage**: O(v) where v = variable count per scope
- **Branch Cloning**: O(v × b) where b = branch depth
- **Error Collection**: O(e) where e = error count

### Optimizations
1. **Environment Reuse**: Clone only at branch points
2. **Move Tracking**: Span-based for precise diagnostics
3. **Copy Type Fast-Path**: Skip move tracking for primitives
4. **Early Exit**: Stop on first collision in verification

---

## Summary

The Vortex borrow checker provides:

✅ **Memory Safety**: No use-after-free or data races  
✅ **Thermodynamic Model**: Entropic flow analysis for permission tracking  
✅ **Affine Types**: Move semantics with Copy type exceptions  
✅ **Rich Diagnostics**: Chaos Vacuum explains the "why" behind errors  
✅ **Self-Hosted**: Core engine written in Fusion itself  
✅ **Compiler Integration**: Seamless pipeline integration  
✅ **Performance**: Linear time/space complexity  

### Future Enhancements

- Lifetime regions and explicit lifetime annotations
- Parallel borrow checking for multi-threaded code
- Incremental checking for IDE integration
- Advanced flow-sensitive analysis
- Integration with LLVM sanitizer passes

---

*Document generated: 2026-07-02*  
*Vortex Engine Version: Fusion v2.0 Vortex*  
*Safety Model: Entropic Flow Analysis with Affine Types*
