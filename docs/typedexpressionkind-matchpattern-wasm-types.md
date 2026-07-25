# Fusion Compiler: TypedExpressionKind, MatchPattern & WASM Type Mappings

This document contains the actual source code definitions for:
1. **TypedExpressionKind** enum from `sema.rs`
2. **MatchPattern** struct from `ast.rs`
3. **WASM Type Mappings** from `wasm/types.rs`

---

## Table of Contents
1. [TypedExpressionKind Enum](#1-typedexpressionkind-enum)
2. [MatchPattern Struct](#2-matchpattern-struct)
3. [WASM Type Mappings](#3-wasm-type-mappings)

---

## 1. TypedExpressionKind Enum

**File:** `crates/fuc/src/sema.rs`  
**Lines:** 40-56  
**Purpose:** Represents all possible kinds of typed expressions after semantic analysis

```rust
pub enum TypedExpressionKind {
    IntLiteral(i64),
    BoolLiteral(bool),
    StringLiteral(String),
    Variable(String),
    FunctionCall { name: String, args: Vec<TypedExpression> },
    BinaryOperation { left: Box<TypedExpression>, right: Box<TypedExpression>, op: ir::BinaryOp },
    UnaryOperation { op: UnaryOp, expr: Box<TypedExpression> },
    ArrayLiteral(Vec<TypedExpression>),
    StructLiteral { name: String, fields: Vec<(String, ir::Type, TypedExpression)> },
    MemberAccess { base: Box<TypedExpression>, field: String },
    AddressOf(Box<TypedExpression>),
    Dereference(Box<TypedExpression>),
    Index { array: Box<TypedExpression>, index: Box<TypedExpression> },
    Match { scrutinee: Box<TypedExpression>, arms: Vec<TypedMatchArm> },
    Closure { params: Vec<(String, ir::Type)>, body: Box<TypedExpression> },
}
```

### Associated Types

**TypedMatchArm** (Lines 58-62):
```rust
pub struct TypedMatchArm {
    pub pattern: MatchPattern,
    pub guard: Option<TypedExpression>,
    pub body: TypedExpression,
}
```

**TypedStructDefinition** (Lines 64-67):
```rust
pub struct TypedStructDefinition {
    pub name: String,
    pub fields: Vec<(String, ir::Type)>,
}
```

---

## 2. MatchPattern Struct

**File:** `crates/fuc/src/ast.rs`  
**Lines:** 141-163  
**Purpose:** Represents pattern matching constructs in match expressions (bootstrap-compatible struct-based design)

```rust
#[derive(Debug, Clone)]
pub struct MatchPattern {
    pub kind: String, // "wildcard", "int", "bool", "string", "var"
    pub int_val: i64,
    pub bool_val: bool,
    pub str_val: String,
}

impl MatchPattern {
    pub fn wildcard() -> Self {
        MatchPattern { kind: "wildcard".to_string(), int_val: 0, bool_val: false, str_val: String::new() }
    }
    pub fn int_literal(val: i64) -> Self {
        MatchPattern { kind: "int".to_string(), int_val: val, bool_val: false, str_val: String::new() }
    }
    pub fn bool_literal(val: bool) -> Self {
        MatchPattern { kind: "bool".to_string(), int_val: 0, bool_val: val, str_val: String::new() }
    }
    pub fn string_literal(val: String) -> Self {
        MatchPattern { kind: "string".to_string(), int_val: 0, bool_val: false, str_val: val }
    }
    pub fn variable(name: String) -> Self {
        MatchPattern { kind: "var".to_string(), int_val: 0, bool_val: false, str_val: name }
    }
}
```

### Associated Type

**MatchArm** (Lines 167-172):
```rust
/// Match arm: pattern (with optional guard) => body.
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub guard: Option<Box<Expression>>,
    pub body: Expression,
}
```

---

## 3. WASM Type Mappings

**File:** `crates/fuc/src/wasm/types.rs`  
**Lines:** 1-59  
**Purpose:** Maps Fusion IR types to WebAssembly value types and provides type utilities

```rust
// src/wasm/types.rs - WASM Type Mappings

use crate::ir::Type;
use wasm_encoder::ValType;

/// Convert Fusion types to WASM types
pub fn fusion_to_wasm_type(fusion_type: &Type) -> Option<ValType> {
    match fusion_type {
        Type::Int => Some(ValType::I64),
        Type::Float => Some(ValType::F64),
        Type::Bool => Some(ValType::I32), // 0 = false, 1 = true
        Type::String => Some(ValType::I32),  // Pointer to memory
        Type::Void => None,                  // No return value
        Type::Struct(_) => Some(ValType::I32), // Heap pointer
        Type::GenericParam(_) => Some(ValType::I32), // Generic resolved to pointer
        Type::Array(_, _) => Some(ValType::I32), // Pointer to array
        Type::Optional(_) => Some(ValType::I32), // Pointer to option
        Type::Union(_) => Some(ValType::I32), // Tagged union pointer
        Type::Closure(_, _) => Some(ValType::I32), // Function table index
        Type::GenericInstance(_, _) => Some(ValType::I32), // Instance pointer
        Type::Unknown => None,               // Should not reach codegen
        Type::Pointer(_) | Type::Slice(_) => Some(ValType::I32),
    }
}

/// Check if a type needs memory allocation
#[allow(dead_code)]
pub fn needs_heap_allocation(fusion_type: &Type) -> bool {
    matches!(
        fusion_type,
        Type::String
            | Type::Struct(_)
            | Type::Array(_, _)
            | Type::Optional(_)
            | Type::Union(_)
            | Type::GenericInstance(_, _)
    )
}

/// Get the size in bytes for a type (for memory allocation)
#[allow(dead_code)]
pub fn type_size_bytes(fusion_type: &Type) -> u32 {
    match fusion_type {
        Type::Int => 8,                // i64
        Type::Float => 8,                  // f64
        Type::Bool => 4,                // i32
        Type::String => 4,                 // pointer
        Type::Struct(_) => 4,              // pointer
        Type::Array(_, _) => 4,               // pointer
        Type::Optional(_) => 4,            // pointer
        Type::Union(_) => 4,               // pointer
        Type::Closure(_, _) => 4,        // function index
        Type::GenericInstance(_, _) => 4, // pointer
        Type::Void => 0,
        Type::GenericParam(_) => 4, // pointer
        Type::Unknown => 0,
        Type::Pointer(_) | Type::Slice(_) => 4,
    }
}
```

---

## Summary

This document provides the exact source code for three critical type definitions in the Fusion compiler:

### TypedExpressionKind (17 variants)
- Represents all typed expression forms after semantic analysis
- Includes literals, variables, operations, function calls, control flow, and data structures
- Used throughout the type checker and IR lowering phases

### MatchPattern (struct-based design)
- Bootstrap-compatible pattern representation using tagged fields instead of enums
- Supports: wildcard, int literals, bool literals, string literals, and variable patterns
- Includes factory methods for each pattern type
- Used in match expressions for pattern matching

### WASM Type Mappings
- Maps Fusion IR types to WebAssembly `ValType` (I32, I64, F64)
- Type conversion: Int→I64, Float→F64, Bool→I32, References→I32
- Utility functions for heap allocation detection and type size calculation
- Critical for the WebAssembly code generation backend

**Note:** The Fusion codebase does not define custom WASM AST enums. Instead, it uses the `wasm_encoder` crate's types (`ValType`, `Instruction`, etc.) directly for WebAssembly generation, with type mapping functions to bridge Fusion IR types to WASM types.

---

*Document generated: 2026-07-02*  
*Source files from Fusion v2.0 Vortex compiler*
