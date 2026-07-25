# Fusion Compiler: Complete Backend & Semantic Analysis Reference

This document contains the **actual source code** from the Fusion v2.0 Vortex bootstrap compiler's Rust backend, IR system, AST definitions, and semantic analyzer.

---

## Table of Contents

1. [IR Definitions (ir.rs)](#1-ir-definitions-irrs) — 260 lines
2. [IR Lowering (ir_lower.rs)](#2-ir-lowering-ir_lowerrs) — 654 lines
3. [WASM Code Generator (wasm/codegen.rs)](#3-wasm-code-generator-wasmcodegenrs) — 897 lines
4. [WASM Type Mappings (wasm/types.rs)](#4-wasm-type-mappings-wasmtpesrs) — 59 lines
5. [AST Definitions (ast.rs)](#5-ast-definitions-astrs) — 172 lines
6. [Semantic Analyzer (sema.rs)](#6-semantic-analyzer-semars) — 427 lines

**Total: 2,469 lines of compiler source code**

---

## 1. IR Definitions (ir.rs)

**File:** `crates/fuc/src/ir.rs`
**Purpose:** Unified IR types for the optimizer, codegen, and borrow checker.

```rust
//! Fusion Intermediate Representation (IR)
//! Unified IR types for the optimizer, codegen, and borrow checker.


// ---- Loan analysis (borrow checker) ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoanKind {
    Immutable,
    Mutable,
}

// ---- Core IR types ----

pub type BlockId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    IntConst(i64),
    BoolConst(bool),
    StringConst(String),
    FloatConst(f64),
    Variable(String),
    Temp(usize),
}

impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::IntConst(v) => v.hash(state),
            Value::BoolConst(v) => v.hash(state),
            Value::StringConst(v) => v.hash(state),
            Value::FloatConst(v) => v.to_bits().hash(state),
            Value::Variable(v) => v.hash(state),
            Value::Temp(v) => v.hash(state),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypedValue {
    pub val: Value,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Address {
    Variable { name: String, ty: Type },
    Pointer { val: TypedValue, pointed_to_ty: Type },
    Element { base: Box<Address>, index: TypedValue, element_ty: Type },
    Field { base: Box<Address>, field_index: usize, field_ty: Type, struct_name: String },
}

#[derive(Debug, Clone)]
pub enum Instruction {
    BinaryOperation {
        dest: TypedValue,
        op: BinaryOp,
        op1: TypedValue,
        op2: TypedValue,
    },
    Call {
        dest: Option<TypedValue>,
        func_name: String,
        args: Vec<TypedValue>,
    },
    Load {
        dest: TypedValue,
        src: Address,
    },
    Store {
        dest: Address,
        val: TypedValue,
    },
    GetElementPtr {
        dest: TypedValue,
        base_ptr: TypedValue,
        index: TypedValue,
        element_ty: Type,
    },
    GetFieldPtr {
        dest: TypedValue,
        base_ptr: TypedValue,
        field_index: usize,
        field_ty: Type,
        struct_name: String,
    },
    // Loan-related instructions (from middle-end IR)
    GetAddress {
        dest: String,
        var_name: String,
        is_mutable: bool,
    },
    // Stack allocation
    Alloca {
        dest: TypedValue,
        ty: Type,
    },
    // Value copy (for move/copy semantics)
    Copy {
        dest: TypedValue,
        src: TypedValue,
    },
    // Unary logical not
    UnaryNot {
        dest: TypedValue,
        operand: TypedValue,
    },
    // Phi node (SSA merge point)
    Phi {
        dest: TypedValue,
        incoming: Vec<(TypedValue, usize)>,
    },
    // Closure creation
    MakeClosure {
        dest: TypedValue,
        func_name: String,
        captured: Vec<TypedValue>,
    },
    // Debug/comment (no-op)
    Comment(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, Le, Ge,
    And, Or,
}

// Terminator tag constants
pub const TERM_JUMP: u8 = 1;
pub const TERM_COND_JUMP: u8 = 2;
pub const TERM_RETURN: u8 = 3;
pub const TERM_UNREACHABLE: u8 = 4;

#[derive(Debug, Clone)]
pub enum Terminator {
    Jump(usize),
    ConditionalJump {
        cond: TypedValue,
        then_block: usize,
        else_block: usize,
    },
    Return(Option<TypedValue>),
    Unreachable,
}

impl Terminator {
    pub fn tag(&self) -> u8 {
        match self {
            Terminator::Jump(_) => TERM_JUMP,
            Terminator::ConditionalJump { .. } => TERM_COND_JUMP,
            Terminator::Return(_) => TERM_RETURN,
            Terminator::Unreachable => TERM_UNREACHABLE,
        }
    }
    pub fn jump(target: usize) -> Self {
        Terminator::Jump(target)
    }
    pub fn conditional(cond: TypedValue, then_block: usize, else_block: usize) -> Self {
        Terminator::ConditionalJump { cond, then_block, else_block }
    }
    pub fn return_(val: Option<TypedValue>) -> Self {
        Terminator::Return(val)
    }
    pub fn unreachable() -> Self {
        Terminator::Unreachable
    }
}

pub fn return_terminator(val: Option<TypedValue>) -> Terminator {
    Terminator::return_(val)
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub instrs: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub blocks: Vec<BasicBlock>,
    pub entry_block: usize,
}

// ---- Module-level definitions ----

#[derive(Debug, Clone)]
pub struct IrStructDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct IrEnumDef {
    pub name: String,
    pub variants: Vec<(String, Option<Type>)>,
}

#[derive(Debug, Clone)]
pub struct IrGlobalString {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct IrExtern {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
    pub externs: Vec<IrExtern>,
    pub structs: Vec<IrStructDef>,
    pub enums: Vec<IrEnumDef>,
    pub global_strings: Vec<IrGlobalString>,
}

impl IrModule {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            externs: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            global_strings: Vec::new(),
        }
    }
}

// ---- Type definitions (delegated to ast) ----

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    String,
    Void,
    Float,
    Struct(String),
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    Slice(Box<Type>),
    GenericParam(String),
    Closure(Vec<Type>, Box<Type>),
    Optional(Box<Type>),
    Union(Vec<Type>),
    GenericInstance(String, Vec<Type>),
    Unknown,
}
```

---

## 2. IR Lowering (ir_lower.rs)

**File:** `crates/fuc/src/ir_lower.rs`
**Purpose:** Converts the typed AST (TypedProgram) into the compiler's intermediate representation (IrModule).

```rust
//! IR Lowering: TypedProgram → IrModule
//! Converts the typed AST into the compiler's intermediate representation.

use crate::ir::*;
use crate::sema::*;
use std::collections::HashMap;

/// IR builder with register counter and block management.
pub struct IrBuilder {
    reg_counter: usize,
    blocks: Vec<BasicBlock>,
    current_block: usize,
    string_counter: usize,
    global_strings: Vec<IrGlobalString>,
}

impl IrBuilder {
    pub fn new() -> Self {
        let entry = BasicBlock {
            label: "entry".to_string(),
            instrs: Vec::new(),
            terminator: Terminator::return_(None),
        };
        Self {
            reg_counter: 0,
            blocks: vec![entry],
            current_block: 0,
            string_counter: 0,
            global_strings: Vec::new(),
        }
    }

    fn next_reg(&mut self) -> usize {
        let r = self.reg_counter;
        self.reg_counter += 1;
        r
    }

    fn next_string_name(&mut self) -> String {
        let n = self.string_counter;
        self.string_counter += 1;
        format!(".str.{}", n)
    }

    fn emit(&mut self, instr: Instruction) {
        self.blocks[self.current_block].instrs.push(instr);
    }

    fn set_terminator(&mut self, term: Terminator) {
        self.blocks[self.current_block].terminator = term;
    }

    fn new_block(&mut self, label: String) -> usize {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            label,
            instrs: Vec::new(),
            terminator: Terminator::return_(None),
        });
        id
    }

    fn switch_to_block(&mut self, id: usize) {
        self.current_block = id;
    }

    fn temp_val(&self, reg: usize, ty: Type) -> TypedValue {
        TypedValue { val: Value::Temp(reg), ty }
    }

    fn int_val(&self, n: i64) -> TypedValue {
        TypedValue { val: Value::IntConst(n), ty: Type::Int }
    }

    fn bool_val(&self, b: bool) -> TypedValue {
        TypedValue { val: Value::BoolConst(b), ty: Type::Bool }
    }

    fn string_val(&self, s: String) -> TypedValue {
        TypedValue { val: Value::StringConst(s), ty: Type::String }
    }

    fn var_val(&self, name: &str, ty: Type) -> TypedValue {
        TypedValue { val: Value::Variable(name.to_string()), ty }
    }

    /// Lower a typed program into an IR module.
    pub fn lower_program(&mut self, prog: &TypedProgram) -> IrModule {
        let mut functions: Vec<IrFunction> = Vec::new();
        let externs: Vec<IrExtern> = Vec::new();
        let mut structs: Vec<IrStructDef> = Vec::new();

        // Lower structs
        for sd in &prog.structs {
            structs.push(IrStructDef {
                name: sd.name.clone(),
                fields: sd.fields.clone(),
            });
        }

        // Lower functions
        for func in &prog.functions {
            let ir_func = self.lower_function(func);
            functions.push(ir_func);
        }

        IrModule {
            functions,
            externs,
            structs,
            enums: Vec::new(),
            global_strings: std::mem::take(&mut self.global_strings),
        }
    }

    fn lower_function(&mut self, func: &TypedFunction) -> IrFunction {
        // Reset builder state
        self.reg_counter = 0;
        self.blocks.clear();
        self.current_block = 0;

        let entry = BasicBlock {
            label: "entry".to_string(),
            instrs: Vec::new(),
            terminator: Terminator::return_(None),
        };
        self.blocks.push(entry);

        // Allocate locals for parameters
        let mut var_map: HashMap<String, TypedValue> = HashMap::new();
        for (name, ty) in &func.params {
            let reg = self.next_reg();
            let dest = self.temp_val(reg, ty.clone());
            self.emit(Instruction::Alloca {
                dest: dest.clone(),
                ty: ty.clone(),
            });
            var_map.insert(name.clone(), dest);
        }

        // Lower the function body
        self.lower_block(&func.body, &mut var_map, &func.return_type);

        // Ensure terminator
        if self.blocks[self.current_block].instrs.is_empty()
            && matches!(self.blocks[self.current_block].terminator, Terminator::Return(None))
        {
            // Already has a default return terminator
        }

        IrFunction {
            name: func.name.clone(),
            blocks: self.blocks.clone(),
            entry_block: 0,
        }
    }

    fn lower_block(
        &mut self,
        stmts: &[TypedStatement],
        var_map: &mut HashMap<String, TypedValue>,
        return_type: &Type,
    ) {
        for stmt in stmts {
            self.lower_statement(stmt, var_map, return_type);
        }
    }

    fn lower_statement(
        &mut self,
        stmt: &TypedStatement,
        var_map: &mut HashMap<String, TypedValue>,
        return_type: &Type,
    ) {
        match stmt {
            TypedStatement::Let { name, value, ty } => {
                let val = self.lower_expression(value, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, ty.clone());
                self.emit(Instruction::Alloca { dest: dest.clone(), ty: ty.clone() });
                self.emit(Instruction::Store {
                    dest: Address::Variable { name: format!("%{}", reg), ty: ty.clone() },
                    val,
                });
                var_map.insert(name.clone(), dest);
            }
            TypedStatement::Assignment { target, value } => {
                let val = self.lower_expression(value, var_map);
                let addr = self.lower_lvalue(target, var_map);
                self.emit(Instruction::Store { dest: addr, val });
            }
            TypedStatement::Expression(expr) => {
                self.lower_expression(expr, var_map);
            }
            TypedStatement::Return(Some(expr)) => {
                let val = self.lower_expression(expr, var_map);
                self.set_terminator(Terminator::return_(Some(val)));
            }
            TypedStatement::Return(None) => {
                self.set_terminator(Terminator::return_(None));
            }
            TypedStatement::If { cond, then_block, else_block } => {
                let cond_val = self.lower_expression(cond, var_map);

                let then_id = self.new_block("then".to_string());
                let else_id = self.new_block("else".to_string());
                let merge_id = self.new_block("merge".to_string());

                self.set_terminator(Terminator::ConditionalJump {
                    cond: cond_val,
                    then_block: then_id,
                    else_block: else_id,
                });

                // Then block
                self.switch_to_block(then_id);
                self.lower_block(then_block, var_map, return_type);
                self.set_terminator(Terminator::Jump(merge_id));

                // Else block
                self.switch_to_block(else_id);
                if let Some(else_b) = else_block {
                    self.lower_block(else_b, var_map, return_type);
                }
                self.set_terminator(Terminator::Jump(merge_id));

                // Merge
                self.switch_to_block(merge_id);
            }
            TypedStatement::While { cond, body } => {
                let header_id = self.new_block("while_header".to_string());
                let body_id = self.new_block("while_body".to_string());
                let exit_id = self.new_block("while_exit".to_string());

                self.set_terminator(Terminator::Jump(header_id));

                // Header
                self.switch_to_block(header_id);
                let cond_val = self.lower_expression(cond, var_map);
                self.set_terminator(Terminator::ConditionalJump {
                    cond: cond_val,
                    then_block: body_id,
                    else_block: exit_id,
                });

                // Body
                self.switch_to_block(body_id);
                self.lower_block(body, var_map, return_type);
                self.set_terminator(Terminator::Jump(header_id));

                // Exit
                self.switch_to_block(exit_id);
            }
            TypedStatement::For { var, iter, body } => {
                let iter_val = self.lower_expression(iter, var_map);

                let header_id = self.new_block("for_header".to_string());
                let body_id = self.new_block("for_body".to_string());
                let exit_id = self.new_block("for_exit".to_string());

                // Allocate loop variable
                let elem_ty = match &iter_val.ty {
                    Type::Array(inner, _) | Type::Slice(inner) => (**inner).clone(),
                    _ => Type::Int,
                };
                let reg = self.next_reg();
                let loop_var = self.temp_val(reg, elem_ty.clone());
                self.emit(Instruction::Alloca { dest: loop_var.clone(), ty: elem_ty.clone() });
                var_map.insert(var.clone(), loop_var);

                // Index variable
                let idx_reg = self.next_reg();
                let idx_val = self.temp_val(idx_reg, Type::Int);
                self.emit(Instruction::Alloca { dest: idx_val.clone(), ty: Type::Int });
                self.emit(Instruction::Store {
                    dest: Address::Variable { name: format!("%{}", idx_reg), ty: Type::Int },
                    val: self.int_val(0),
                });

                self.set_terminator(Terminator::Jump(header_id));

                // Header: load index and check bounds
                self.switch_to_block(header_id);
                let load_reg = self.next_reg();
                let idx_loaded = self.temp_val(load_reg, Type::Int);
                let elem_ty2 = match &iter_val.ty {
                    Type::Array(inner, _) | Type::Slice(inner) => (**inner).clone(),
                    _ => Type::Int,
                };
                self.emit(Instruction::Load {
                    dest: idx_loaded.clone(),
                    src: Address::Variable { name: format!("%{}", idx_reg), ty: Type::Int },
                });
                // Load element from iter at index
                let elem_ptr_reg = self.next_reg();
                let elem_ptr = self.temp_val(elem_ptr_reg, Type::Pointer(Box::new(elem_ty2.clone())));
                self.emit(Instruction::GetElementPtr {
                    dest: elem_ptr.clone(),
                    base_ptr: iter_val.clone(),
                    index: idx_loaded.clone(),
                    element_ty: elem_ty2.clone(),
                });
                let loop_var_reg = self.next_reg();
                let loop_var2 = self.temp_val(loop_var_reg, elem_ty2.clone());
                self.emit(Instruction::Load {
                    dest: loop_var2,
                    src: Address::Pointer { val: elem_ptr, pointed_to_ty: elem_ty2 },
                });
                // TODO: proper bounds check
                self.set_terminator(Terminator::Jump(body_id));

                // Body
                self.switch_to_block(body_id);
                self.lower_block(body, var_map, return_type);
                // Increment index
                let new_idx_reg = self.next_reg();
                let new_idx = self.temp_val(new_idx_reg, Type::Int);
                self.emit(Instruction::BinaryOperation {
                    dest: new_idx.clone(),
                    op: BinaryOp::Add,
                    op1: idx_loaded,
                    op2: self.int_val(1),
                });
                self.emit(Instruction::Store {
                    dest: Address::Variable { name: format!("%{}", idx_reg), ty: Type::Int },
                    val: new_idx,
                });
                self.set_terminator(Terminator::Jump(header_id));

                // Exit
                self.switch_to_block(exit_id);
            }
        }
    }

    fn lower_expression(
        &mut self,
        expr: &TypedExpression,
        var_map: &HashMap<String, TypedValue>,
    ) -> TypedValue {
        match &expr.node {
            TypedExpressionKind::IntLiteral(n) => self.int_val(*n),
            TypedExpressionKind::BoolLiteral(b) => self.bool_val(*b),
            TypedExpressionKind::StringLiteral(s) => {
                let name = self.next_string_name();
                self.global_strings.push(IrGlobalString { name: name.clone(), value: s.clone() });
                self.string_val(name)
            }
            TypedExpressionKind::Variable(name) => {
                if let Some(var) = var_map.get(name) {
                    let reg = self.next_reg();
                    let dest = self.temp_val(reg, var.ty.clone());
                    self.emit(Instruction::Load {
                        dest: dest.clone(),
                        src: Address::Variable { name: name.clone(), ty: var.ty.clone() },
                    });
                    dest
                } else {
                    self.var_val(name, expr.ty.clone())
                }
            }
            TypedExpressionKind::FunctionCall { name, args } => {
                let lowered_args: Vec<TypedValue> = args.iter()
                    .map(|a| self.lower_expression(a, var_map))
                    .collect();
                let dest = if expr.ty != Type::Void {
                    let reg = self.next_reg();
                    Some(self.temp_val(reg, expr.ty.clone()))
                } else {
                    None
                };
                self.emit(Instruction::Call {
                    dest: dest.clone(),
                    func_name: name.clone(),
                    args: lowered_args,
                });
                dest.unwrap_or_else(|| self.temp_val(0, Type::Void))
            }
            TypedExpressionKind::BinaryOperation { left, right, op } => {
                let l = self.lower_expression(left, var_map);
                let r = self.lower_expression(right, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                self.emit(Instruction::BinaryOperation {
                    dest: dest.clone(),
                    op: op.clone(),
                    op1: l,
                    op2: r,
                });
                dest
            }
            TypedExpressionKind::UnaryOperation { op, expr: inner } => {
                let operand = self.lower_expression(inner, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                match op {
                    crate::ast::UnaryOp::Not => {
                        self.emit(Instruction::UnaryNot { dest: dest.clone(), operand });
                    }
                    crate::ast::UnaryOp::Neg => {
                        self.emit(Instruction::BinaryOperation {
                            dest: dest.clone(),
                            op: BinaryOp::Sub,
                            op1: self.int_val(0),
                            op2: operand,
                        });
                    }
                }
                dest
            }
            TypedExpressionKind::ArrayLiteral(elems) => {
                let lowered: Vec<TypedValue> = elems.iter()
                    .map(|e| self.lower_expression(e, var_map))
                    .collect();
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                self.emit(Instruction::Alloca { dest: dest.clone(), ty: expr.ty.clone() });
                for (i, elem) in lowered.iter().enumerate() {
                    let idx = self.int_val(i as i64);
                    let elem_ptr_reg = self.next_reg();
                    let elem_ptr = self.temp_val(elem_ptr_reg, Type::Pointer(Box::new(elem.ty.clone())));
                    self.emit(Instruction::GetElementPtr {
                        dest: elem_ptr.clone(),
                        base_ptr: dest.clone(),
                        index: idx,
                        element_ty: elem.ty.clone(),
                    });
                    self.emit(Instruction::Store {
                        dest: Address::Pointer { val: elem_ptr, pointed_to_ty: elem.ty.clone() },
                        val: elem.clone(),
                    });
                }
                dest
            }
            TypedExpressionKind::StructLiteral { name, fields } => {
                let reg = self.next_reg();
                let dest = self.temp_val(reg, Type::Struct(name.clone()));
                self.emit(Instruction::Alloca { dest: dest.clone(), ty: Type::Struct(name.clone()) });
                for (i, (_, _, field_expr)) in fields.iter().enumerate() {
                    let val = self.lower_expression(field_expr, var_map);
                    let field_ptr_reg = self.next_reg();
                    let field_ptr = self.temp_val(field_ptr_reg, Type::Pointer(Box::new(val.ty.clone())));
                    self.emit(Instruction::GetFieldPtr {
                        dest: field_ptr.clone(),
                        base_ptr: dest.clone(),
                        field_index: i,
                        field_ty: val.ty.clone(),
                        struct_name: name.clone(),
                    });
                    self.emit(Instruction::Store {
                        dest: Address::Pointer { val: field_ptr, pointed_to_ty: val.ty.clone() },
                        val,
                    });
                }
                dest
            }
            TypedExpressionKind::MemberAccess { base, field: _field } => {
                let base_val = self.lower_expression(base, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                if let Type::Struct(name) = &base_val.ty {
                    let field_idx = 0usize; // simplified: always field 0
                    let field_ptr_reg = self.next_reg();
                    let field_ptr = self.temp_val(field_ptr_reg, Type::Pointer(Box::new(expr.ty.clone())));
                    let name_clone = name.clone();
                    self.emit(Instruction::GetFieldPtr {
                        dest: field_ptr.clone(),
                        base_ptr: base_val,
                        field_index: field_idx,
                        field_ty: expr.ty.clone(),
                        struct_name: name_clone,
                    });
                    self.emit(Instruction::Load {
                        dest: dest.clone(),
                        src: Address::Pointer { val: field_ptr, pointed_to_ty: expr.ty.clone() },
                    });
                }
                dest
            }
            TypedExpressionKind::AddressOf(inner) => {
                let inner_val = self.lower_expression(inner, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, Type::Pointer(Box::new(inner_val.ty.clone())));
                self.emit(Instruction::Comment(format!("address_of {}", reg)));
                dest
            }
            TypedExpressionKind::Dereference(inner) => {
                let ptr_val = self.lower_expression(inner, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                self.emit(Instruction::Load {
                    dest: dest.clone(),
                    src: Address::Pointer { val: ptr_val, pointed_to_ty: expr.ty.clone() },
                });
                dest
            }
            TypedExpressionKind::Index { array, index } => {
                let arr_val = self.lower_expression(array, var_map);
                let idx_val = self.lower_expression(index, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                let elem_ptr_reg = self.next_reg();
                let elem_ptr = self.temp_val(elem_ptr_reg, Type::Pointer(Box::new(expr.ty.clone())));
                self.emit(Instruction::GetElementPtr {
                    dest: elem_ptr.clone(),
                    base_ptr: arr_val,
                    index: idx_val,
                    element_ty: expr.ty.clone(),
                });
                self.emit(Instruction::Load {
                    dest: dest.clone(),
                    src: Address::Pointer { val: elem_ptr, pointed_to_ty: expr.ty.clone() },
                });
                dest
            }
            TypedExpressionKind::Match { scrutinee: _, arms } => {
                let mut arm_blocks: Vec<usize> = Vec::new();
                let merge_id = self.new_block("match_merge".to_string());

                // Create a block for each arm
                for i in 0..arms.len() {
                    arm_blocks.push(self.new_block(format!("match_arm_{}", i)));
                }
                let _default_id = self.new_block("match_default".to_string());

                // Jump to first arm (simplified: no pattern matching in IR yet)
                self.set_terminator(Terminator::Jump(arm_blocks[0]));

                // Lower each arm
                for (i, arm) in arms.iter().enumerate() {
                    self.switch_to_block(arm_blocks[i]);
                    let val = self.lower_expression(&arm.body, var_map);
                    let reg = self.next_reg();
                    let result = self.temp_val(reg, expr.ty.clone());
                    self.emit(Instruction::Copy { dest: result, src: val });
                    self.set_terminator(Terminator::Jump(merge_id));
                }

                // Default
                self.switch_to_block(_default_id);
                self.set_terminator(Terminator::Unreachable);

                // Merge
                self.switch_to_block(merge_id);
                let result_reg = self.next_reg();
                self.temp_val(result_reg, expr.ty.clone())
            }
            TypedExpressionKind::Closure { params: _, body } => {
                let body_val = self.lower_expression(body, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                self.emit(Instruction::MakeClosure {
                    dest: dest.clone(),
                    func_name: String::new(),
                    captured: vec![body_val],
                });
                dest
            }
        }
    }

    fn lower_lvalue(
        &mut self,
        expr: &TypedExpression,
        var_map: &HashMap<String, TypedValue>,
    ) -> Address {
        match &expr.node {
            TypedExpressionKind::Variable(name) => {
                if let Some(var) = var_map.get(name) {
                    Address::Variable { name: name.clone(), ty: var.ty.clone() }
                } else {
                    Address::Variable { name: name.clone(), ty: expr.ty.clone() }
                }
            }
            TypedExpressionKind::MemberAccess { base, field: _field } => {
                let base_addr = self.lower_lvalue(base, var_map);
                let base_val = self.lower_expression(base, var_map);
                if let Type::Struct(struct_name) = &base_val.ty {
                    let field_idx = 0usize; // simplified
                    Address::Field {
                        base: Box::new(base_addr),
                        field_index: field_idx,
                        field_ty: expr.ty.clone(),
                        struct_name: struct_name.clone(),
                    }
                } else {
                    Address::Variable { name: "unknown".to_string(), ty: expr.ty.clone() }
                }
            }
            TypedExpressionKind::Dereference(inner) => {
                let ptr_val = self.lower_expression(inner, var_map);
                Address::Pointer { val: ptr_val, pointed_to_ty: expr.ty.clone() }
            }
            TypedExpressionKind::Index { array, index } => {
                let arr_val = self.lower_expression(array, var_map);
                let idx_val = self.lower_expression(index, var_map);
                Address::Element {
                    base: Box::new(Address::Variable { name: "array".to_string(), ty: arr_val.ty.clone() }),
                    index: idx_val,
                    element_ty: expr.ty.clone(),
                }
            }
            _ => Address::Variable { name: "unknown".to_string(), ty: expr.ty.clone() },
        }
    }
}

/// Lower a typed program to an IR module.
pub fn lower_program(prog: &TypedProgram) -> IrModule {
    let mut builder = IrBuilder::new();
    builder.lower_program(prog)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    #[test]
    fn test_lower_empty_program() {
        let prog = TypedProgram {
            functions: Vec::new(),
            structs: Vec::new(),
        };
        let module = lower_program(&prog);
        assert!(module.functions.is_empty());
        assert!(module.structs.is_empty());
    }

    #[test]
    fn test_lower_function_with_return() {
        use crate::ir::Type;
        let func = TypedFunction {
            name: "main".to_string(),
            params: Vec::new(),
            return_type: Type::Int,
            body: vec![
                TypedStatement::Return(Some(TypedExpression {
                    node: TypedExpressionKind::IntLiteral(42),
                    ty: Type::Int,
                    span: Span::default(),
                })),
            ],
        };
        let prog = TypedProgram {
            functions: vec![func],
            structs: Vec::new(),
        };
        let module = lower_program(&prog);
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "main");
        assert!(!module.functions[0].blocks.is_empty());
    }
}
```

---

## 3. WASM Code Generator (wasm/codegen.rs)

**File:** `crates/fuc/src/wasm/codegen.rs`
**Purpose:** Generates WebAssembly binary modules directly from the untyped AST (Declaration nodes).

```rust
// src/wasm/codegen.rs - WebAssembly Code Generator

use crate::ast::{
    BinaryOp, Block, Declaration, Expression, ExpressionKind, Literal, Parameter, Statement,
    Type, UnaryOp,
};
use crate::wasm::types::*;
use std::collections::HashMap;
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, DataSection, ExportKind, ExportSection,
    Function as WasmFunction, FunctionSection, Instruction, MemorySection, MemoryType, Module,
    TypeSection, ValType,
};

pub struct WasmCodeGenerator {
    function_index: u32,
    function_map: HashMap<String, u32>, // Function name -> index
    type_section: TypeSection,
    function_section: FunctionSection,
    export_section: ExportSection,
    code_section: CodeSection,
    memory_section: MemorySection,
    next_local_index: u32,
    local_map: HashMap<String, u32>, // Variable name -> local index
    string_offsets: HashMap<String, u32>, // String literal -> memory offset
    string_data: Vec<u8>,                  // Accumulated string bytes
}

impl WasmCodeGenerator {
    pub fn new() -> Self {
        WasmCodeGenerator {
            function_index: 0,
            function_map: HashMap::new(),
            type_section: TypeSection::new(),
            function_section: FunctionSection::new(),
            export_section: ExportSection::new(),
            code_section: CodeSection::new(),
            memory_section: MemorySection::new(),
            next_local_index: 0,
            local_map: HashMap::new(),
            string_offsets: HashMap::new(),
            string_data: Vec::new(),
        }
    }

    /// Generate WASM binary from AST
    pub fn generate(&mut self, declarations: &[Declaration]) -> Result<Vec<u8>, String> {
        // Add memory section (1 page = 64KB)
        self.memory_section.memory(MemoryType {
            minimum: 1,
            maximum: Some(10),
            memory64: false,
            shared: false,
            page_size_log2: None,
        });

        // Process all declarations
        for decl in declarations {
            self.generate_declaration(decl)?;
        }

        // Build final module
        let mut module = Module::new();
        module.section(&self.type_section);
        module.section(&self.function_section);
        module.section(&self.memory_section);
        module.section(&self.export_section);
        module.section(&self.code_section);

        // Emit data section with string literals
        if !self.string_data.is_empty() {
            let mut data_section = DataSection::new();
            let offset = ConstExpr::i32_const(0);
            data_section.active(0, &offset, self.string_data.iter().copied());
            module.section(&data_section);
        }

        Ok(module.finish())
    }

    fn generate_declaration(&mut self, decl: &Declaration) -> Result<(), String> {
        match decl {
            Declaration::Function { name, params, return_type, body, .. } => {
                self.generate_function(name, params, return_type, body)?;
            }
            Declaration::ModuleDecl { .. } | Declaration::UseDecl { .. } | Declaration::ImportDecl { .. } => {
                // Skip module system declarations in WASM generation
            }
            _ => {
                // Skip other declarations for now
            }
        }
        Ok(())
    }

    /// Count additional local variables needed beyond function parameters.
    fn count_locals(block: &Block) -> u32 {
        let mut count = 0u32;
        for stmt in &block.statements {
            match stmt {
                Statement::Let { .. } | Statement::VariableDeclaration { .. } => {
                    count += 1;
                }
                Statement::If { then_block, else_block, .. } => {
                    count += Self::count_locals(then_block);
                    if let Some(else_blk) = else_block {
                        count += Self::count_locals(else_blk);
                    }
                }
                Statement::While { body, .. } => {
                    count += Self::count_locals(body);
                }
                _ => {}
            }
        }
        count
    }

    fn generate_function(
        &mut self,
        name: &str,
        params: &[Parameter],
        return_type: &Type,
        body: &Block,
    ) -> Result<(), String> {
        // Build function type
        let param_types: Vec<ValType> = params
            .iter()
            .filter_map(|p| fusion_to_wasm_type(&p.param_type))
            .collect();

        let result_types: Vec<ValType> = fusion_to_wasm_type(return_type).into_iter().collect();

        // Add to type section
        let type_idx = self.type_section.len();
        self.type_section
            .ty()
            .function(param_types.clone(), result_types.clone());

        // Add to function section
        self.function_section.function(type_idx);

        // Map function name to index
        let func_idx = self.function_index;
        self.function_map.insert(name.to_string(), func_idx);
        self.function_index += 1;

        // Export the function
        self.export_section.export(name, ExportKind::Func, func_idx);

        // Generate function body
        let extra_locals = Self::count_locals(body);
        let mut func_body = if extra_locals > 0 {
            WasmFunction::new(vec![(extra_locals, ValType::I64)])
        } else {
            WasmFunction::new(vec![])
        };

        // Reset local tracking for this function
        self.next_local_index = params.len() as u32;
        self.local_map.clear();

        // Map parameters to locals
        for (i, param) in params.iter().enumerate() {
            self.local_map.insert(param.name.clone(), i as u32);
        }

        // Generate statements
        for stmt in &body.statements {
            self.generate_statement(stmt, &mut func_body)?;
        }

        func_body.instruction(&Instruction::End);

        // Add function to code section
        self.code_section.function(&func_body);

        Ok(())
    }

    fn generate_statement(
        &mut self,
        stmt: &Statement,
        func: &mut WasmFunction,
    ) -> Result<(), String> {
        match stmt {
            Statement::Return(Some(expr)) => {
                self.generate_expression(expr, func)?;
                func.instruction(&Instruction::Return);
            }
            Statement::Return(None) => {
                func.instruction(&Instruction::Return);
            }
            Statement::VariableDeclaration { name, initializer, .. } => {
                let local_idx = self.next_local_index;
                self.local_map.insert(name.clone(), local_idx);
                self.next_local_index += 1;
                self.generate_expression(initializer, func)?;
                func.instruction(&Instruction::LocalSet(local_idx));
            }
            Statement::Let { name, value, .. } => {
                let local_idx = self.next_local_index;
                self.local_map.insert(name.clone(), local_idx);
                self.next_local_index += 1;
                self.generate_expression(value, func)?;
                func.instruction(&Instruction::LocalSet(local_idx));
            }
            Statement::If { cond, then_block, else_block } => {
                self.generate_expression(cond, func)?;
                func.instruction(&Instruction::I32WrapI64);
                func.instruction(&Instruction::If(BlockType::Empty));

                for s in &then_block.statements {
                    self.generate_statement(s, func)?;
                }

                if let Some(else_blk) = else_block {
                    func.instruction(&Instruction::Else);
                    for s in &else_blk.statements {
                        self.generate_statement(s, func)?;
                    }
                }

                func.instruction(&Instruction::End);
                func.instruction(&Instruction::Unreachable);
            }
            Statement::While { cond, body } => {
                // WASM while pattern: block (outer) + loop (inner)
                func.instruction(&Instruction::Block(BlockType::Empty));
                func.instruction(&Instruction::Loop(BlockType::Empty));

                self.generate_expression(cond, func)?;
                func.instruction(&Instruction::I32WrapI64);
                func.instruction(&Instruction::I32Eqz);
                func.instruction(&Instruction::BrIf(1));

                for s in &body.statements {
                    self.generate_statement(s, func)?;
                }

                func.instruction(&Instruction::Br(0));
                func.instruction(&Instruction::End); // loop
                func.instruction(&Instruction::End); // block
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr, func)?;
                func.instruction(&Instruction::Drop);
            }
            Statement::Assignment { target, value } => {
                self.generate_expression(value, func)?;
                if let ExpressionKind::Variable(name) = &target.kind {
                    let local_idx = self
                        .local_map
                        .get(name)
                        .ok_or(format!("Unknown variable for assignment: {}", name))?;
                    func.instruction(&Instruction::LocalSet(*local_idx));
                } else {
                    return Err("Assignment target must be a variable".to_string());
                }
            }
            Statement::For { var, iter, body } => {
                let local_idx = self.next_local_index;
                self.local_map.insert(var.clone(), local_idx);
                self.next_local_index += 1;
                self.generate_expression(iter, func)?;
                func.instruction(&Instruction::LocalSet(local_idx));
                for s in &body.statements {
                    self.generate_statement(s, func)?;
                }
            }
        }
        Ok(())
    }

    fn generate_expression(
        &mut self,
        expr: &Expression,
        func: &mut WasmFunction,
    ) -> Result<(), String> {
        match &expr.kind {
            ExpressionKind::Literal(lit) => match lit {
                Literal::Integer(n) => {
                    func.instruction(&Instruction::I64Const(*n));
                }
                Literal::Float(f) => {
                    func.instruction(&Instruction::F64Const(*f));
                }
                Literal::Boolean(b) => {
                    func.instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
                }
                Literal::String(s) => {
                    let offset = if let Some(&existing) = self.string_offsets.get(s) {
                        existing
                    } else {
                        let off = self.string_data.len() as u32;
                        self.string_data.extend_from_slice(s.as_bytes());
                        self.string_data.push(0); // null terminator
                        self.string_offsets.insert(s.clone(), off);
                        off
                    };
                    func.instruction(&Instruction::I32Const(offset as i32));
                }
            },
            ExpressionKind::Variable(name) => {
                let local_idx = self
                    .local_map
                    .get(name)
                    .ok_or(format!("Unknown variable: {}", name))?;
                func.instruction(&Instruction::LocalGet(*local_idx));
            }
            ExpressionKind::BinaryOp { left, op, right } => {
                self.generate_expression(left, func)?;
                self.generate_expression(right, func)?;

                match op {
                    BinaryOp::Add => { func.instruction(&Instruction::I64Add); }
                    BinaryOp::Sub => { func.instruction(&Instruction::I64Sub); }
                    BinaryOp::Mul => { func.instruction(&Instruction::I64Mul); }
                    BinaryOp::Div => { func.instruction(&Instruction::I64DivS); }
                    BinaryOp::Mod => { func.instruction(&Instruction::I64RemS); }
                    BinaryOp::Eq => { func.instruction(&Instruction::I64Eq); func.instruction(&Instruction::I64ExtendI32S); }
                    BinaryOp::Neq => { func.instruction(&Instruction::I64Ne); func.instruction(&Instruction::I64ExtendI32S); }
                    BinaryOp::Lt => { func.instruction(&Instruction::I64LtS); func.instruction(&Instruction::I64ExtendI32S); }
                    BinaryOp::Gt => { func.instruction(&Instruction::I64GtS); func.instruction(&Instruction::I64ExtendI32S); }
                    BinaryOp::Le => { func.instruction(&Instruction::I64LeS); func.instruction(&Instruction::I64ExtendI32S); }
                    BinaryOp::Ge => { func.instruction(&Instruction::I64GeS); func.instruction(&Instruction::I64ExtendI32S); }
                    BinaryOp::And => { func.instruction(&Instruction::I64And); }
                    BinaryOp::Or => { func.instruction(&Instruction::I64Or); }
                };
            }
            ExpressionKind::FunctionCall { name, args, .. } => {
                for arg in args {
                    self.generate_expression(arg, func)?;
                }
                let func_idx = self
                    .function_map
                    .get(name)
                    .ok_or(format!("Unknown function: {}", name))?;
                func.instruction(&Instruction::Call(*func_idx));
            }
            ExpressionKind::UnaryOp { op, expr } => {
                self.generate_expression(expr, func)?;
                match op {
                    UnaryOp::Neg => {
                        func.instruction(&Instruction::I64Const(-1));
                        func.instruction(&Instruction::I64Mul);
                    }
                    UnaryOp::Not => {
                        func.instruction(&Instruction::I64Eqz);
                        func.instruction(&Instruction::I64ExtendI32S);
                    }
                }
            }
            ExpressionKind::MemberAccess { base, field: _field } => {
                self.generate_expression(base, func)?;
            }
            ExpressionKind::StructLiteral { name: _, fields } => {
                for (_field_name, field_expr) in fields {
                    self.generate_expression(field_expr, func)?;
                }
            }
            ExpressionKind::ArrayLiteral(elems) => {
                for elem in elems {
                    self.generate_expression(elem, func)?;
                }
            }
            ExpressionKind::Match { scrutinee, arms } => {
                self.generate_expression(scrutinee, func)?;
                func.instruction(&Instruction::Drop);
                if let Some(first_arm) = arms.first() {
                    self.generate_expression(&first_arm.body, func)?;
                } else {
                    func.instruction(&Instruction::I64Const(0));
                }
            }
            ExpressionKind::Closure { params: _, body } => {
                self.generate_expression(body, func)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_expr(kind: ExpressionKind) -> Expression {
        Expression { kind, ty: None }
    }

    fn make_var(name: &str) -> Expression {
        make_expr(ExpressionKind::Variable(name.to_string()))
    }

    fn make_int(n: i64) -> Expression {
        make_expr(ExpressionKind::Literal(Literal::Integer(n)))
    }

    #[allow(dead_code)]
    fn make_bool(b: bool) -> Expression {
        make_expr(ExpressionKind::Literal(Literal::Boolean(b)))
    }

    fn make_binary(left: Expression, op: BinaryOp, right: Expression) -> Expression {
        make_expr(ExpressionKind::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    #[test]
    fn test_simple_function() {
        let mut generator = WasmCodeGenerator::new();
        let decl = Declaration::Function {
            name: "add".to_string(),
            params: vec![
                Parameter { name: "a".to_string(), param_type: Type::Int },
                Parameter { name: "b".to_string(), param_type: Type::Int },
            ],
            return_type: Type::Int,
            body: Block {
                statements: vec![Statement::Return(Some(make_binary(
                    make_var("a"),
                    BinaryOp::Add,
                    make_var("b"),
                )))],
            },
            where_bounds: vec![],
        };
        let result = generator.generate(&[decl]);
        let wasm_bytes = result.expect("WASM generation failed");
        assert!(!wasm_bytes.is_empty());
        let validation = wasmparser::validate(&wasm_bytes);
        assert!(validation.is_ok(), "WASM validation failed: {:?}", validation.err());
    }

    #[test]
    fn test_all_binary_ops() {
        let mut generator = WasmCodeGenerator::new();
        let ops = [
            (BinaryOp::Add, "add_op"), (BinaryOp::Sub, "sub_op"),
            (BinaryOp::Mul, "mul_op"), (BinaryOp::Div, "div_op"),
            (BinaryOp::Mod, "mod_op"), (BinaryOp::Eq, "eq_op"),
            (BinaryOp::Neq, "neq_op"), (BinaryOp::Lt, "lt_op"),
            (BinaryOp::Gt, "gt_op"), (BinaryOp::Le, "le_op"),
            (BinaryOp::Ge, "ge_op"), (BinaryOp::And, "and_op"),
            (BinaryOp::Or, "or_op"),
        ];
        let mut decls = Vec::new();
        for (op, name) in &ops {
            decls.push(Declaration::Function {
                name: name.to_string(),
                params: vec![
                    Parameter { name: "x".to_string(), param_type: Type::Int },
                    Parameter { name: "y".to_string(), param_type: Type::Int },
                ],
                return_type: Type::Int,
                body: Block {
                    statements: vec![Statement::Return(Some(make_binary(
                        make_var("x"), *op, make_var("y"),
                    )))],
                },
                where_bounds: vec![],
            });
        }
        let result = generator.generate(&decls);
        let wasm_bytes = result.expect("WASM generation failed");
        assert!(!wasm_bytes.is_empty());
        let validation = wasmparser::validate(&wasm_bytes);
        assert!(validation.is_ok(), "WASM validation failed: {:?}", validation.err());
    }

    #[test]
    fn test_if_else() { /* ... see source ... */ }

    #[test]
    fn test_multiple_functions() { /* ... see source ... */ }

    #[test]
    fn test_while_loop() { /* ... see source ... */ }

    #[test]
    fn test_unary_ops() { /* ... see source ... */ }

    #[test]
    fn test_struct_access() { /* ... see source ... */ }

    #[test]
    fn test_nested_control_flow() { /* ... see source ... */ }

    #[test]
    fn test_string_literals() { /* ... see source ... */ }

    #[test]
    fn test_multi_string_dedup() { /* ... see source ... */ }
}
```

*Note: Test bodies after `test_all_binary_ops` are abbreviated in this document for brevity. The full source contains 11 tests covering functions, binary ops, control flow, strings, and deduplication — all validating the generated WASM with `wasmparser::validate`.*

---

## 4. WASM Type Mappings (wasm/types.rs)

**File:** `crates/fuc/src/wasm/types.rs`
**Purpose:** Maps Fusion IR types to WebAssembly value types and provides type utilities.

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
        Type::Float => 8,              // f64
        Type::Bool => 4,               // i32
        Type::String => 4,             // pointer
        Type::Struct(_) => 4,          // pointer
        Type::Array(_, _) => 4,        // pointer
        Type::Optional(_) => 4,        // pointer
        Type::Union(_) => 4,           // pointer
        Type::Closure(_, _) => 4,      // function index
        Type::GenericInstance(_, _) => 4, // pointer
        Type::Void => 0,
        Type::GenericParam(_) => 4,    // pointer
        Type::Unknown => 0,
        Type::Pointer(_) | Type::Slice(_) => 4,
    }
}
```

---

## 5. AST Definitions (ast.rs)

**File:** `crates/fuc/src/ast.rs`
**Purpose:** The unified AST types used across the compiler — expressions, statements, declarations, patterns.

```rust
//! Fusion AST module - re-exports from ast_types and ir.
//! Provides the unified AST types used across the compiler.

pub use crate::ast_types::StructInfo;
pub use crate::ir::Type;
pub use crate::ir::BinaryOp;

/// Span type for source locations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Default for Span {
    fn default() -> Self {
        Self { start: 0, end: 0 }
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

// ---- AST node types for aspirational modules ----

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub structs: Vec<StructDefinition>,
    pub declarations: Vec<Declaration>,
}

impl Program {
    pub fn new() -> Self {
        Self { functions: Vec::new(), structs: Vec::new(), declarations: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Function {
        name: String,
        params: Vec<Parameter>,
        return_type: Type,
        body: Block,
        where_bounds: Vec<()>,
    },
    ModuleDecl { name: String, body: Vec<Declaration> },
    UseDecl { path: Vec<String> },
    ImportDecl { path: Vec<String> },
    ExternFunction { name: String, params: Vec<Parameter>, return_type: Type },
    StructDefinition(StructDefinition),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Block,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let { name: String, value: Expression, ty: Type },
    Assignment { target: Expression, value: Expression },
    Expression(Expression),
    Return(Option<Expression>),
    VariableDeclaration { name: String, initializer: Expression, ty: Option<Type> },
    If { cond: Expression, then_block: Box<Block>, else_block: Option<Box<Block>> },
    While { cond: Expression, body: Box<Block> },
    For { var: String, iter: Expression, body: Box<Block> },
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Literal(Literal),
    Variable(String),
    BinaryOp { left: Box<Expression>, op: BinaryOp, right: Box<Expression> },
    UnaryOp { op: UnaryOp, expr: Box<Expression> },
    FunctionCall { name: String, args: Vec<Expression>, type_args: Vec<Type> },
    MemberAccess { base: Box<Expression>, field: String },
    StructLiteral { name: String, fields: Vec<(String, Expression)> },
    ArrayLiteral(Vec<Expression>),
    Match { scrutinee: Box<Expression>, arms: Vec<MatchArm> },
    Closure { params: Vec<Parameter>, body: Box<Expression> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub generics: Vec<String>,
}

pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for String {
    fn span(&self) -> Span { Span::default() }
}

/// Match pattern for pattern matching expressions.
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

/// Match arm: pattern (with optional guard) => body.
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub guard: Option<Box<Expression>>,
    pub body: Expression,
}
```

---

## 6. Semantic Analyzer (sema.rs)

**File:** `crates/fuc/src/sema.rs`
**Purpose:** Type checking and typed AST construction. Converts untyped AST → TypedProgram for the IR lowering phase.

```rust
//! Fusion Semantic Analyzer
//! The full sema implementation lives in sema.fu (self-hosted Fusion source).
//! This Rust implementation provides type checking and typed AST construction.

use crate::types::*;
use crate::ast::{self, Span, Expression, ExpressionKind, Statement, Literal, Block, Declaration, Type, BinaryOp, UnaryOp, MatchPattern};
use crate::ir;
use std::collections::HashMap;

// ---- Type stubs for borrowck and other modules ----

pub struct TypedProgram {
    pub functions: FVec<TypedFunction>,
    pub structs: FVec<TypedStructDefinition>,
}

pub struct TypedFunction {
    pub name: FString,
    pub params: FVec<(FString, ir::Type)>,
    pub return_type: ir::Type,
    pub body: FVec<TypedStatement>,
}

pub enum TypedStatement {
    Let { name: FString, value: TypedExpression, ty: ir::Type },
    Assignment { target: TypedExpression, value: TypedExpression },
    Expression(TypedExpression),
    Return(Option<TypedExpression>),
    If { cond: TypedExpression, then_block: FVec<TypedStatement>, else_block: Option<FVec<TypedStatement>> },
    While { cond: TypedExpression, body: FVec<TypedStatement> },
    For { var: FString, iter: TypedExpression, body: FVec<TypedStatement> },
}

pub struct TypedExpression {
    pub node: TypedExpressionKind,
    pub ty: ir::Type,
    pub span: Span,
}

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

pub struct TypedMatchArm {
    pub pattern: MatchPattern,
    pub guard: Option<TypedExpression>,
    pub body: TypedExpression,
}

pub struct TypedStructDefinition {
    pub name: String,
    pub fields: Vec<(String, ir::Type)>,
}

// ---- Analyzer ----

pub struct Analyzer {
    functions: HashMap<String, (Vec<ir::Type>, ir::Type)>,
    structs: HashMap<String, Vec<(String, ir::Type)>>,
}

pub struct SemaOutput {
    pub errors: FVec<FString>,
    pub program: Option<TypedProgram>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    /// Analyzes a parsed program and returns semantic diagnostics.
    pub fn analyze_output(&mut self, prog: ast::Program) -> SemaOutput {
        let mut errors: Vec<String> = Vec::new();

        // First pass: collect all function and struct signatures
        for decl in &prog.declarations {
            match decl {
                Declaration::Function { name, params, return_type, .. } => {
                    let param_types: Vec<ir::Type> = params.iter().map(|p| convert_type(&p.param_type)).collect();
                    let ret = convert_type(return_type);
                    self.functions.insert(name.clone(), (param_types, ret));
                }
                Declaration::ExternFunction { name, params, return_type } => {
                    let param_types: Vec<ir::Type> = params.iter().map(|p| convert_type(&p.param_type)).collect();
                    let ret = convert_type(return_type);
                    self.functions.insert(name.clone(), (param_types, ret));
                }
                Declaration::StructDefinition(sd) => {
                    let fields: Vec<(String, ir::Type)> = sd.fields.iter().map(|(n, t)| (n.clone(), convert_type(t))).collect();
                    self.structs.insert(sd.name.clone(), fields);
                }
                _ => {}
            }
        }

        // Second pass: type-check each function
        let mut typed_functions: Vec<TypedFunction> = Vec::new();
        for decl in &prog.declarations {
            if let Declaration::Function { name, params, return_type, body, .. } = decl {
                let typed_params: Vec<(String, ir::Type)> = params.iter()
                    .map(|p| (p.name.clone(), convert_type(&p.param_type)))
                    .collect();
                let ret_ty = convert_type(return_type);

                let mut local_vars: HashMap<String, ir::Type> = HashMap::new();
                for (n, t) in &typed_params {
                    local_vars.insert(n.clone(), t.clone());
                }

                let typed_body = self.type_check_block(body, &mut local_vars, &ret_ty, &mut errors);

                typed_functions.push(TypedFunction {
                    name: name.clone(),
                    params: typed_params,
                    return_type: ret_ty,
                    body: typed_body,
                });
            }
        }

        // Collect typed structs
        let typed_structs: Vec<TypedStructDefinition> = prog.structs.iter().map(|s| {
            TypedStructDefinition {
                name: s.name.clone(),
                fields: s.fields.iter().map(|(n, t)| (n.clone(), convert_type(t))).collect(),
            }
        }).collect();

        SemaOutput {
            errors: errors.clone(),
            program: if errors.is_empty() {
                Some(TypedProgram { functions: typed_functions, structs: typed_structs })
            } else {
                None
            },
        }
    }

    fn type_check_block(
        &self,
        block: &Block,
        locals: &mut HashMap<String, ir::Type>,
        expected_return: &ir::Type,
        errors: &mut Vec<String>,
    ) -> Vec<TypedStatement> {
        let mut stmts: Vec<TypedStatement> = Vec::new();
        for stmt in &block.statements {
            match self.type_check_statement(stmt, locals, expected_return, errors) {
                Some(s) => stmts.push(s),
                None => {}
            }
        }
        stmts
    }

    fn type_check_statement(
        &self,
        stmt: &Statement,
        locals: &mut HashMap<String, ir::Type>,
        expected_return: &ir::Type,
        errors: &mut Vec<String>,
    ) -> Option<TypedStatement> {
        match stmt {
            Statement::Let { name, value, ty } => {
                let inferred = self.type_check_expr(value, locals, errors);
                let declared_ty = convert_type(ty);
                let actual_ty = if declared_ty != ir::Type::Unknown { declared_ty.clone() } else { inferred.ty.clone() };
                locals.insert(name.clone(), actual_ty.clone());
                Some(TypedStatement::Let { name: name.clone(), value: inferred, ty: actual_ty })
            }
            Statement::VariableDeclaration { name, initializer, ty } => {
                let inferred = self.type_check_expr(initializer, locals, errors);
                let declared_ty = ty.as_ref().map(convert_type).unwrap_or(ir::Type::Unknown);
                let actual_ty = if declared_ty != ir::Type::Unknown { declared_ty } else { inferred.ty.clone() };
                locals.insert(name.clone(), actual_ty.clone());
                Some(TypedStatement::Let { name: name.clone(), value: inferred, ty: actual_ty })
            }
            Statement::Assignment { target, value } => {
                let typed_val = self.type_check_expr(value, locals, errors);
                let typed_target = self.type_check_expr(target, locals, errors);
                Some(TypedStatement::Assignment { target: typed_target, value: typed_val })
            }
            Statement::Expression(expr) => {
                let typed = self.type_check_expr(expr, locals, errors);
                Some(TypedStatement::Expression(typed))
            }
            Statement::Return(Some(expr)) => {
                let typed = self.type_check_expr(expr, locals, errors);
                if typed.ty != *expected_return && *expected_return != ir::Type::Void && *expected_return != ir::Type::Unknown {
                    errors.push(format!("Return type mismatch: expected {:?}, found {:?}", expected_return, typed.ty));
                }
                Some(TypedStatement::Return(Some(typed)))
            }
            Statement::Return(None) => {
                Some(TypedStatement::Return(None))
            }
            Statement::If { cond, then_block, else_block } => {
                let typed_cond = self.type_check_expr(cond, locals, errors);
                let typed_then = self.type_check_block(then_block, locals, expected_return, errors);
                let typed_else = else_block.as_ref().map(|b| self.type_check_block(b, locals, expected_return, errors));
                Some(TypedStatement::If { cond: typed_cond, then_block: typed_then, else_block: typed_else })
            }
            Statement::While { cond, body } => {
                let typed_cond = self.type_check_expr(cond, locals, errors);
                let typed_body = self.type_check_block(body, locals, expected_return, errors);
                Some(TypedStatement::While { cond: typed_cond, body: typed_body })
            }
            Statement::For { var, iter, body } => {
                let typed_iter = self.type_check_expr(iter, locals, errors);
                let iter_ty = match &typed_iter.ty {
                    ir::Type::Array(elem, _) | ir::Type::Slice(elem) => (**elem).clone(),
                    _ => ir::Type::Int,
                };
                locals.insert(var.clone(), iter_ty);
                let typed_body = self.type_check_block(body, locals, expected_return, errors);
                Some(TypedStatement::For { var: var.clone(), iter: typed_iter, body: typed_body })
            }
        }
    }

    fn type_check_expr(
        &self,
        expr: &Expression,
        locals: &HashMap<String, ir::Type>,
        errors: &mut Vec<String>,
    ) -> TypedExpression {
        let span = Span::default();
        let (node, ty) = match &expr.kind {
            ExpressionKind::Literal(Literal::Integer(n)) => {
                (TypedExpressionKind::IntLiteral(*n), ir::Type::Int)
            }
            ExpressionKind::Literal(Literal::Float(n)) => {
                (TypedExpressionKind::IntLiteral(*n as i64), ir::Type::Float)
            }
            ExpressionKind::Literal(Literal::Boolean(b)) => {
                (TypedExpressionKind::BoolLiteral(*b), ir::Type::Bool)
            }
            ExpressionKind::Literal(Literal::String(s)) => {
                (TypedExpressionKind::StringLiteral(s.clone()), ir::Type::String)
            }
            ExpressionKind::Variable(name) => {
                let ty = locals.get(name).cloned().unwrap_or(ir::Type::Unknown);
                (TypedExpressionKind::Variable(name.clone()), ty)
            }
            ExpressionKind::BinaryOp { left, right, op } => {
                let typed_left = self.type_check_expr(left, locals, errors);
                let typed_right = self.type_check_expr(right, locals, errors);
                let result_ty = match op {
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt
                    | BinaryOp::Le | BinaryOp::Ge | BinaryOp::And | BinaryOp::Or => ir::Type::Bool,
                    _ => typed_left.ty.clone(),
                };
                (TypedExpressionKind::BinaryOperation {
                    left: Box::new(typed_left),
                    right: Box::new(typed_right),
                    op: convert_binary_op(*op),
                }, result_ty)
            }
            ExpressionKind::UnaryOp { op, expr: inner } => {
                let typed_inner = self.type_check_expr(inner, locals, errors);
                let result_ty = typed_inner.ty.clone();
                (TypedExpressionKind::UnaryOperation { op: *op, expr: Box::new(typed_inner) }, result_ty)
            }
            ExpressionKind::FunctionCall { name, args, .. } => {
                let typed_args: Vec<TypedExpression> = args.iter()
                    .map(|a| self.type_check_expr(a, locals, errors))
                    .collect();
                let ret_ty = self.functions.get(name)
                    .map(|(_, ret)| ret.clone())
                    .unwrap_or(ir::Type::Unknown);
                (TypedExpressionKind::FunctionCall { name: name.clone(), args: typed_args }, ret_ty)
            }
            ExpressionKind::MemberAccess { base, field } => {
                let typed_base = self.type_check_expr(base, locals, errors);
                let field_ty = match &typed_base.ty {
                    ir::Type::Struct(name) => {
                        self.structs.get(name)
                            .and_then(|fields| fields.iter().find(|(n, _)| n == field))
                            .map(|(_, t)| t.clone())
                            .unwrap_or(ir::Type::Unknown)
                    }
                    _ => ir::Type::Unknown,
                };
                (TypedExpressionKind::MemberAccess { base: Box::new(typed_base), field: field.clone() }, field_ty)
            }
            ExpressionKind::StructLiteral { name, fields } => {
                let typed_fields: Vec<(String, ir::Type, TypedExpression)> = fields.iter()
                    .map(|(n, e)| {
                        let te = self.type_check_expr(e, locals, errors);
                        let ft = te.ty.clone();
                        (n.clone(), ft, te)
                    })
                    .collect();
                let ty = if name.is_empty() { ir::Type::Unknown } else { ir::Type::Struct(name.clone()) };
                (TypedExpressionKind::StructLiteral { name: name.clone(), fields: typed_fields }, ty)
            }
            ExpressionKind::ArrayLiteral(elems) => {
                let typed_elems: Vec<TypedExpression> = elems.iter()
                    .map(|e| self.type_check_expr(e, locals, errors))
                    .collect();
                let elem_ty = typed_elems.first().map(|e| e.ty.clone()).unwrap_or(ir::Type::Unknown);
                (TypedExpressionKind::ArrayLiteral(typed_elems), ir::Type::Array(Box::new(elem_ty), elems.len()))
            }
            ExpressionKind::Match { scrutinee, arms } => {
                let typed_scrutinee = self.type_check_expr(scrutinee, locals, errors);
                let typed_arms: Vec<TypedMatchArm> = arms.iter().map(|arm| {
                    let guard = arm.guard.as_ref().map(|g| self.type_check_expr(g, locals, errors));
                    let body = self.type_check_expr(&arm.body, locals, errors);
                    TypedMatchArm { pattern: arm.pattern.clone(), guard, body }
                }).collect();
                let result_ty = typed_arms.first().map(|a| a.body.ty.clone()).unwrap_or(ir::Type::Unknown);
                (TypedExpressionKind::Match { scrutinee: Box::new(typed_scrutinee), arms: typed_arms }, result_ty)
            }
            ExpressionKind::Closure { params, body } => {
                let typed_params: Vec<(String, ir::Type)> = params.iter()
                    .map(|p| (p.name.clone(), convert_type(&p.param_type)))
                    .collect();
                let mut closure_locals = locals.clone();
                for (n, t) in &typed_params {
                    closure_locals.insert(n.clone(), t.clone());
                }
                let typed_body = self.type_check_expr(body, &closure_locals, errors);
                let ret_ty = typed_body.ty.clone();
                let param_types: Vec<ir::Type> = typed_params.iter().map(|(_, t)| t.clone()).collect();
                (TypedExpressionKind::Closure { params: typed_params, body: Box::new(typed_body) },
                 ir::Type::Closure(param_types, Box::new(ret_ty)))
            }
        };
        TypedExpression { node, ty, span }
    }
}

fn convert_type(ty: &Type) -> ir::Type {
    match ty {
        Type::Int => ir::Type::Int,
        Type::Bool => ir::Type::Bool,
        Type::String => ir::Type::String,
        Type::Void => ir::Type::Void,
        Type::Unknown => ir::Type::Unknown,
        Type::Pointer(inner) => ir::Type::Pointer(Box::new(convert_type(inner))),
        Type::Array(elem, len) => ir::Type::Array(Box::new(convert_type(elem)), *len),
        Type::Struct(name) => ir::Type::Struct(name.clone()),
        Type::GenericParam(name) => ir::Type::GenericParam(name.clone()),
        Type::Slice(inner) => ir::Type::Slice(Box::new(convert_type(inner))),
        Type::Closure(params, ret) => {
            ir::Type::Closure(params.iter().map(convert_type).collect(), Box::new(convert_type(ret)))
        }
        Type::Float => ir::Type::Float,
        Type::Optional(inner) => ir::Type::Optional(Box::new(convert_type(inner))),
        Type::Union(types) => ir::Type::Union(types.iter().map(convert_type).collect()),
        Type::GenericInstance(name, args) => ir::Type::GenericInstance(name.clone(), args.iter().map(convert_type).collect()),
    }
}

fn convert_binary_op(op: ast::BinaryOp) -> ir::BinaryOp {
    match op {
        ast::BinaryOp::Add => ir::BinaryOp::Add,
        ast::BinaryOp::Sub => ir::BinaryOp::Sub,
        ast::BinaryOp::Mul => ir::BinaryOp::Mul,
        ast::BinaryOp::Div => ir::BinaryOp::Div,
        ast::BinaryOp::Mod => ir::BinaryOp::Mod,
        ast::BinaryOp::Eq => ir::BinaryOp::Eq,
        ast::BinaryOp::Neq => ir::BinaryOp::Neq,
        ast::BinaryOp::Lt => ir::BinaryOp::Lt,
        ast::BinaryOp::Gt => ir::BinaryOp::Gt,
        ast::BinaryOp::Le => ir::BinaryOp::Le,
        ast::BinaryOp::Ge => ir::BinaryOp::Ge,
        ast::BinaryOp::And => ir::BinaryOp::And,
        ast::BinaryOp::Or => ir::BinaryOp::Or,
    }
}

// Re-export Vortex types for backward compatibility
pub mod entropy {
    pub use crate::vortex::{EventCollision, PermissionState};
}
```

---

## Summary: Compiler Pipeline

```
Fusion Source (.fu)
       │
       ▼
   ┌─────────┐
   │ Parser   │  →  AST (ast.rs: Program, Declaration, Expression, Statement)
   └─────────┘
       │
       ▼
   ┌─────────┐
   │ Sema     │  →  Typed AST (sema.rs: TypedProgram, TypedFunction, TypedExpression)
   └─────────┘
       │
       ▼
   ┌─────────┐
   │ IR Lower │  →  IR Module (ir.rs: IrModule, BasicBlock, Instruction, Terminator)
   └─────────┘
       │
       ▼
   ┌──────────┐
   │ WASM     │  →  WebAssembly Binary (wasm/codegen.rs + wasm/types.rs)
   │ Codegen  │
   └──────────┘
```

### Key Type Relationships

| AST Type (`ast::Type`) | IR Type (`ir::Type`) | WASM Type (`ValType`) |
|---|---|---|
| `Int` | `Int` | `I64` |
| `Float` | `Float` | `F64` |
| `Bool` | `Bool` | `I32` |
| `String` | `String` | `I32` (pointer) |
| `Void` | `Void` | _(none)_ |
| `Struct(name)` | `Struct(name)` | `I32` (pointer) |
| `Array(T, n)` | `Array(T, n)` | `I32` (pointer) |
| `Pointer(T)` | `Pointer(T)` | `I32` |

---

*Document generated: 2026-07-02*
*Source files from Fusion v2.0 Vortex bootstrap compiler (`crates/fuc/src/`)*
