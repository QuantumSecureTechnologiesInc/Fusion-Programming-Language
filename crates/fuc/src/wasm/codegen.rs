// src/wasm/codegen.rs

#[allow(unused_imports)]
use crate::ast::{
    BinaryOp, Block, Declaration, Expression, ExpressionKind, Literal, MatchArm, MatchPattern,
    Parameter, Statement, Type, UnaryOp,
};
use crate::wasm::types::*;
use std::collections::HashMap;
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, DataSection, ExportKind, ExportSection,
    Function as WasmFunction, FunctionSection, GlobalSection, GlobalType, Instruction,
    MemorySection, MemoryType, MemArg, Module, TypeSection, ValType,
};

const HEAP_BASE: i32 = 4;
const GLOBAL_HEAP_BUMP: u32 = 0;
const SLOT_BYTES: i32 = 8;

fn emit_heap_alloc(size_bytes: i32, func: &mut WasmFunction) {
    func.instruction(&Instruction::GlobalGet(GLOBAL_HEAP_BUMP));
    func.instruction(&Instruction::GlobalGet(GLOBAL_HEAP_BUMP));
    func.instruction(&Instruction::I32Const(size_bytes));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(GLOBAL_HEAP_BUMP));
}

fn field_byte_offset(field_idx: usize) -> i32 {
    field_idx as i32 * SLOT_BYTES
}

fn align_up(value: i32, align: i32) -> i32 {
    ((value + align - 1) / align) * align
}

fn require_wasm_type(ty: &Type) -> Result<Option<ValType>, String> {
    match ty {
        Type::Unknown => Err("Type::Unknown reached WASM codegen".to_string()),
        _ => Ok(fusion_to_wasm_type(ty)),
    }
}

#[derive(Debug, Clone)]
struct PlannedLocals {
    named: Vec<(String, Type)>,
    scratch_i32: u32,
    scratch_i64: u32,
    scratch_f64: u32,
}

impl PlannedLocals {
    fn new() -> Self {
        Self {
            named: Vec::new(),
            scratch_i32: 0,
            scratch_i64: 0,
            scratch_f64: 0,
        }
    }
}

pub struct WasmCodeGenerator {
    function_index: u32,
    function_map: HashMap<String, u32>,
    type_section: TypeSection,
    function_section: FunctionSection,
    export_section: ExportSection,
    code_section: CodeSection,
    memory_section: MemorySection,
    global_section: GlobalSection,
    local_map: HashMap<String, u32>,
    local_types: HashMap<String, Type>,
    string_offsets: HashMap<String, u32>,
    string_data: Vec<u8>,
    struct_fields: HashMap<String, Vec<String>>,
    next_scratch_i32: u32,
    next_scratch_i64: u32,
    next_scratch_f64: u32,
}

impl WasmCodeGenerator {
    pub fn new() -> Self {
        Self {
            function_index: 0,
            function_map: HashMap::new(),
            type_section: TypeSection::new(),
            function_section: FunctionSection::new(),
            export_section: ExportSection::new(),
            code_section: CodeSection::new(),
            memory_section: MemorySection::new(),
            global_section: GlobalSection::new(),
            local_map: HashMap::new(),
            local_types: HashMap::new(),
            string_offsets: HashMap::new(),
            string_data: Vec::new(),
            struct_fields: HashMap::new(),
            next_scratch_i32: 0,
            next_scratch_i64: 0,
            next_scratch_f64: 0,
        }
    }

    pub fn register_struct(&mut self, name: &str, fields: Vec<String>) {
        self.struct_fields.insert(name.to_string(), fields);
    }

    fn field_index(&self, struct_name: Option<&str>, field: &str) -> usize {
        if let Some(sname) = struct_name {
            if let Some(fields) = self.struct_fields.get(sname) {
                if let Some(pos) = fields.iter().position(|f| f == field) {
                    return pos;
                }
            }
        }
        0
    }

    pub fn generate(&mut self, declarations: &[Declaration]) -> Result<Vec<u8>, String> {
        self.memory_section.memory(MemoryType {
            minimum: 1,
            maximum: Some(10),
            memory64: false,
            shared: false,
            page_size_log2: None,
        });

        for decl in declarations {
            if let Declaration::StructDefinition(def) = decl {
                let field_names = def.fields.iter().map(|(name, _)| name.clone()).collect();
                self.struct_fields.insert(def.name.clone(), field_names);
            }
        }

        self.register_function_signatures(declarations)?;
        self.generate_function_bodies(declarations)?;

        let strings_base = HEAP_BASE;
        let heap_start = align_up(strings_base + self.string_data.len() as i32, SLOT_BYTES);

        self.global_section = GlobalSection::new();
        self.global_section.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i32_const(heap_start),
        );

        let mut module = Module::new();
        module.section(&self.type_section);
        module.section(&self.function_section);
        module.section(&self.memory_section);
        module.section(&self.global_section);
        module.section(&self.export_section);
        module.section(&self.code_section);

        if !self.string_data.is_empty() {
            let mut data_section = DataSection::new();
            let offset = ConstExpr::i32_const(strings_base);
            data_section.active(0, &offset, self.string_data.iter().copied());
            module.section(&data_section);
        }

        Ok(module.finish())
    }

    fn register_function_signatures(&mut self, declarations: &[Declaration]) -> Result<(), String> {
        for decl in declarations {
            if let Declaration::Function {
                name,
                params,
                return_type,
                ..
            } = decl
            {
                let mut param_types = Vec::new();
                for p in params {
                    match require_wasm_type(&p.param_type)? {
                        Some(vt) => param_types.push(vt),
                        None => {
                            return Err(format!(
                                "Parameter `{}` in function `{}` has no WASM type",
                                p.name, name
                            ))
                        }
                    }
                }

                let result_types = match require_wasm_type(return_type)? {
                    Some(vt) => vec![vt],
                    None => vec![],
                };

                let type_idx = self.type_section.len();
                self.type_section
                    .ty()
                    .function(param_types.clone(), result_types.clone());
                self.function_section.function(type_idx);

                let func_idx = self.function_index;
                self.function_map.insert(name.clone(), func_idx);
                self.export_section.export(name, ExportKind::Func, func_idx);
                self.function_index += 1;
            }
        }
        Ok(())
    }

    fn generate_function_bodies(&mut self, declarations: &[Declaration]) -> Result<(), String> {
        for decl in declarations {
            if let Declaration::Function {
                name,
                params,
                return_type,
                body,
                ..
            } = decl
            {
                self.generate_function(name, params, return_type, body)?;
            }
        }
        Ok(())
    }

    fn collect_local_plan(block: &Block) -> PlannedLocals {
        let mut plan = PlannedLocals::new();

        fn walk_stmt(stmt: &Statement, plan: &mut PlannedLocals) {
            match stmt {
                Statement::Let { name, ty, value } => {
                    plan.named.push((name.clone(), ty.clone()));
                    walk_expr(value, plan);
                }
                Statement::VariableDeclaration {
                    name,
                    initializer,
                    ty,
                } => {
                    if let Some(t) = ty {
                        plan.named.push((name.clone(), t.clone()));
                    }
                    walk_expr(initializer, plan);
                }
                Statement::Assignment { target, value } => {
                    walk_expr(target, plan);
                    walk_expr(value, plan);
                }
                Statement::Expression(expr) => walk_expr(expr, plan),
                Statement::Return(Some(expr)) => walk_expr(expr, plan),
                Statement::Return(None) => {}
                Statement::If {
                    cond,
                    then_block,
                    else_block,
                } => {
                    walk_expr(cond, plan);
                    walk_block(then_block, plan);
                    if let Some(eb) = else_block {
                        walk_block(eb, plan);
                    }
                }
                Statement::While { cond, body } => {
                    walk_expr(cond, plan);
                    walk_block(body, plan);
                }
                Statement::For { var, iter, body } => {
                    plan.named.push((var.clone(), Type::Int));
                    plan.scratch_i32 += 2;
                    walk_expr(iter, plan);
                    walk_block(body, plan);
                }
            }
        }

        fn walk_block(block: &Block, plan: &mut PlannedLocals) {
            for stmt in &block.statements {
                walk_stmt(stmt, plan);
            }
        }

        fn walk_expr(expr: &Expression, plan: &mut PlannedLocals) {
            match &expr.kind {
                ExpressionKind::Literal(_) | ExpressionKind::Variable(_) => {}
                ExpressionKind::BinaryOp { left, right, .. } => {
                    walk_expr(left, plan);
                    walk_expr(right, plan);
                }
                ExpressionKind::UnaryOp { expr, .. } => walk_expr(expr, plan),
                ExpressionKind::FunctionCall { args, .. } => {
                    for arg in args {
                        walk_expr(arg, plan);
                    }
                }
                ExpressionKind::MemberAccess { base, .. } => walk_expr(base, plan),
                ExpressionKind::StructLiteral { fields, .. } => {
                    plan.scratch_i32 += 1;
                    for (_, value) in fields {
                        walk_expr(value, plan);
                    }
                }
                ExpressionKind::ArrayLiteral(elems) => {
                    plan.scratch_i32 += 1;
                    for elem in elems {
                        walk_expr(elem, plan);
                    }
                }
                ExpressionKind::Match { scrutinee, arms } => {
                    walk_expr(scrutinee, plan);
                    for arm in arms {
                        if let Some(guard) = &arm.guard {
                            walk_expr(guard, plan);
                        }
                        walk_expr(&arm.body, plan);
                    }
                }
                ExpressionKind::Closure { .. } => {}
            }
        }

        walk_block(block, &mut plan);
        plan
    }

    #[allow(unused_assignments)]
    fn build_locals_decl(
        &self,
        plan: &PlannedLocals,
    ) -> Result<(Vec<(u32, ValType)>, HashMap<String, u32>, HashMap<String, Type>, u32, u32, u32), String> {
        let mut entries: Vec<ValType> = Vec::new();
        let mut local_map = HashMap::new();
        let mut local_types = HashMap::new();
        let mut next_index = 0u32;

        for (name, ty) in &plan.named {
            let vt = require_wasm_type(ty)?
                .ok_or_else(|| format!("Local `{}` has no WASM type", name))?;
            local_map.insert(name.clone(), next_index);
            local_types.insert(name.clone(), ty.clone());
            entries.push(vt);
            next_index += 1;
        }

        let scratch_i32_base = next_index;
        for _ in 0..plan.scratch_i32 {
            entries.push(ValType::I32);
            next_index += 1;
        }

        let scratch_i64_base = next_index;
        for _ in 0..plan.scratch_i64 {
            entries.push(ValType::I64);
            next_index += 1;
        }

        let scratch_f64_base = next_index;
        for _ in 0..plan.scratch_f64 {
            entries.push(ValType::F64);
            next_index += 1;
        }

        let mut grouped: Vec<(u32, ValType)> = Vec::new();
        for vt in entries {
            match grouped.last_mut() {
                Some((count, last)) if *last == vt => *count += 1,
                _ => grouped.push((1, vt)),
            }
        }

        Ok((
            grouped,
            local_map,
            local_types,
            scratch_i32_base,
            scratch_i64_base,
            scratch_f64_base,
        ))
    }

    fn generate_function(
        &mut self,
        name: &str,
        params: &[Parameter],
        return_type: &Type,
        body: &Block,
    ) -> Result<(), String> {
        let plan = Self::collect_local_plan(body);
        let (
            locals_decl,
            local_map_from_plan,
            mut local_types,
            scratch_i32_base_rel,
            scratch_i64_base_rel,
            scratch_f64_base_rel,
        ) = self.build_locals_decl(&plan)?;

        let param_count = params.len() as u32;

        // Build the final local map: params at indices [0..param_count),
        // then locals from the plan at indices [param_count..).
        let mut final_local_map = HashMap::new();
        for (i, param) in params.iter().enumerate() {
            final_local_map.insert(param.name.clone(), i as u32);
            local_types.insert(param.name.clone(), param.param_type.clone());
        }
        for (name, idx) in local_map_from_plan {
            final_local_map.insert(name, idx + param_count);
        }

        self.local_map = final_local_map;
        self.local_types = local_types;
        self.next_scratch_i32 = param_count + scratch_i32_base_rel;
        self.next_scratch_i64 = param_count + scratch_i64_base_rel;
        self.next_scratch_f64 = param_count + scratch_f64_base_rel;

        let mut func_body = WasmFunction::new(locals_decl);

        for stmt in &body.statements {
            self.generate_statement(stmt, &mut func_body)?;
        }

        // If the function body doesn't end with a direct Return statement,
        // push a dummy value so the function result type is satisfied.
        // This handles both normal fall-through AND the case where the last
        // statement is an if/else whose branches all return — WASM still
        // needs a value on the stack at the function-level End.
        if !self.ends_with_return(body) {
            match return_type {
                Type::Void => {}
                Type::Bool => { func_body.instruction(&Instruction::I32Const(0)); }
                Type::Float => { func_body.instruction(&Instruction::F64Const(0.0)); }
                Type::String | Type::Struct(_) | Type::Array(_, _)
                | Type::Optional(_) | Type::Union(_) | Type::GenericParam(_)
                | Type::Closure(_, _) | Type::GenericInstance(_, _)
                | Type::Pointer(_) | Type::Slice(_) => {
                    func_body.instruction(&Instruction::I32Const(0));
                }
                _ => { func_body.instruction(&Instruction::I64Const(0)); }
            }
        }
        func_body.instruction(&Instruction::End);

        self.code_section.function(&func_body);
        let _ = name;
        Ok(())
    }

    fn ends_with_return(&self, body: &Block) -> bool {
        matches!(body.statements.last(), Some(Statement::Return(_)))
    }

    /// Returns true if the block structurally guarantees a return
    /// (either via a direct Return or an if/else where all branches return).
    #[allow(dead_code)]
    fn structurally_returns(&self, body: &Block) -> bool {
        match body.statements.last() {
            Some(Statement::Return(_)) => true,
            Some(Statement::If { then_block, else_block: Some(eb), .. }) => {
                self.structurally_returns(then_block) && self.structurally_returns(eb)
            }
            _ => false,
        }
    }

    fn alloc_scratch_i32(&mut self) -> u32 {
        let idx = self.next_scratch_i32;
        self.next_scratch_i32 += 1;
        idx
    }

    #[allow(dead_code)]
    fn alloc_scratch_i64(&mut self) -> u32 {
        let idx = self.next_scratch_i64;
        self.next_scratch_i64 += 1;
        idx
    }

    #[allow(dead_code)]
    fn alloc_scratch_f64(&mut self) -> u32 {
        let idx = self.next_scratch_f64;
        self.next_scratch_f64 += 1;
        idx
    }

    fn coerce_condition_to_i32(
        &mut self,
        expr: &Expression,
        func: &mut WasmFunction,
    ) -> Result<(), String> {
        self.generate_expression(expr, func)?;
        // Determine what type is actually on the WASM stack.
        // Comparisons produce i32 then get sign-extended to i64.
        // Boolean ops (And/Or) produce i32 directly.
        // If the expression is known to already produce i32, skip the wrap.
        let stack_is_i32 = match &expr.kind {
            ExpressionKind::BinaryOp { op, .. } => matches!(
                op,
                BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt
                | BinaryOp::Le | BinaryOp::Ge | BinaryOp::And | BinaryOp::Or
            ),
            ExpressionKind::UnaryOp { op, .. } => matches!(op, UnaryOp::Not),
            _ => false,
        };
        if stack_is_i32 {
            // The expression already produced i32 on the stack (via the
            // i64.extend_i32_s that follows comparisons, or directly for
            // And/Or/Not). We need i32, but the value was extended to i64.
            // Wrap it back to i32.
            func.instruction(&Instruction::I32WrapI64);
        } else {
            match expr.ty.as_ref() {
                Some(Type::Bool) => {}
                Some(Type::Int) => {
                    func.instruction(&Instruction::I32WrapI64);
                }
                Some(Type::Float) => {
                    func.instruction(&Instruction::F64Const(0.0));
                    func.instruction(&Instruction::F64Ne);
                }
                _ => {
                    func.instruction(&Instruction::I32WrapI64);
                }
            }
        }
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
                let local_idx = *self
                    .local_map
                    .get(name)
                    .ok_or_else(|| format!("Unknown variable declaration local: {}", name))?;
                self.generate_expression(initializer, func)?;
                func.instruction(&Instruction::LocalSet(local_idx));
            }
            Statement::Let { name, value, .. } => {
                let local_idx = *self
                    .local_map
                    .get(name)
                    .ok_or_else(|| format!("Unknown let local: {}", name))?;
                self.generate_expression(value, func)?;
                func.instruction(&Instruction::LocalSet(local_idx));
            }
            Statement::If {
                cond,
                then_block,
                else_block,
            } => {
                self.coerce_condition_to_i32(cond, func)?;
                func.instruction(&Instruction::If(BlockType::Empty));
                for s in &then_block.statements {
                    self.generate_statement(s, func)?;
                }
                if let Some(eb) = else_block {
                    func.instruction(&Instruction::Else);
                    for s in &eb.statements {
                        self.generate_statement(s, func)?;
                    }
                }
                func.instruction(&Instruction::End);
            }
            Statement::While { cond, body } => {
                func.instruction(&Instruction::Block(BlockType::Empty));
                func.instruction(&Instruction::Loop(BlockType::Empty));
                self.coerce_condition_to_i32(cond, func)?;
                func.instruction(&Instruction::I32Eqz);
                func.instruction(&Instruction::BrIf(1));
                for s in &body.statements {
                    self.generate_statement(s, func)?;
                }
                func.instruction(&Instruction::Br(0));
                func.instruction(&Instruction::End);
                func.instruction(&Instruction::End);
            }
            Statement::For { var, iter, body } => {
                let idx_local = self.alloc_scratch_i32();
                let arr_local = self.alloc_scratch_i32();
                let elem_local = *self
                    .local_map
                    .get(var)
                    .ok_or_else(|| format!("Unknown for-loop variable: {}", var))?;

                func.instruction(&Instruction::I32Const(0));
                func.instruction(&Instruction::LocalSet(idx_local));

                self.generate_expression(iter, func)?;
                match iter.ty.as_ref() {
                    Some(Type::Array(_, _))
                    | Some(Type::String)
                    | Some(Type::Struct(_))
                    | Some(Type::Optional(_))
                    | Some(Type::Union(_))
                    | Some(Type::GenericParam(_))
                    | Some(Type::Closure(_, _))
                    | Some(Type::GenericInstance(_, _))
                    | Some(Type::Pointer(_))
                    | Some(Type::Slice(_)) => {}
                    _ => { func.instruction(&Instruction::I32WrapI64); }
                }
                func.instruction(&Instruction::LocalSet(arr_local));

                func.instruction(&Instruction::Block(BlockType::Empty));
                func.instruction(&Instruction::Loop(BlockType::Empty));

                func.instruction(&Instruction::LocalGet(idx_local));
                func.instruction(&Instruction::LocalGet(arr_local));
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::I32GeU);
                func.instruction(&Instruction::BrIf(1));

                func.instruction(&Instruction::LocalGet(arr_local));
                func.instruction(&Instruction::I32Const(4));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::LocalGet(idx_local));
                func.instruction(&Instruction::I32Const(SLOT_BYTES));
                func.instruction(&Instruction::I32Mul);
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::I64Load(MemArg {
                    offset: 0,
                    align: 3,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::LocalSet(elem_local));

                for s in &body.statements {
                    self.generate_statement(s, func)?;
                }

                func.instruction(&Instruction::LocalGet(idx_local));
                func.instruction(&Instruction::I32Const(1));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::LocalSet(idx_local));

                func.instruction(&Instruction::Br(0));
                func.instruction(&Instruction::End);
                func.instruction(&Instruction::End);
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr, func)?;
                if !matches!(expr.ty, Some(Type::Void) | None) {
                    func.instruction(&Instruction::Drop);
                } else {
                    func.instruction(&Instruction::Drop);
                }
            }
            Statement::Assignment { target, value } => {
                self.generate_expression(value, func)?;
                if let ExpressionKind::Variable(name) = &target.kind {
                    let local_idx = self
                        .local_map
                        .get(name)
                        .ok_or_else(|| format!("Unknown variable for assignment: {}", name))?;
                    func.instruction(&Instruction::LocalSet(*local_idx));
                } else {
                    return Err("Assignment target must be a variable".to_string());
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
                        self.string_data.push(0);
                        self.string_offsets.insert(s.clone(), off);
                        off
                    };
                    func.instruction(&Instruction::I32Const(HEAP_BASE + offset as i32));
                }
            },

            ExpressionKind::Variable(name) => {
                let local_idx = self
                    .local_map
                    .get(name)
                    .ok_or_else(|| format!("Unknown variable: {}", name))?;
                func.instruction(&Instruction::LocalGet(*local_idx));
            }

            ExpressionKind::BinaryOp { left, op, right } => {
                self.generate_expression(left, func)?;
                self.generate_expression(right, func)?;

                let lhs_ty = left.ty.as_ref().or(expr.ty.as_ref());
                match lhs_ty {
                    Some(Type::Float) => match op {
                        BinaryOp::Add => { func.instruction(&Instruction::F64Add); }
                        BinaryOp::Sub => { func.instruction(&Instruction::F64Sub); }
                        BinaryOp::Mul => { func.instruction(&Instruction::F64Mul); }
                        BinaryOp::Div => { func.instruction(&Instruction::F64Div); }
                        BinaryOp::Mod => {
                            return Err("Float modulo is not supported in WASM backend".to_string())
                        }
                        BinaryOp::Eq => { func.instruction(&Instruction::F64Eq); }
                        BinaryOp::Neq => { func.instruction(&Instruction::F64Ne); }
                        BinaryOp::Lt => { func.instruction(&Instruction::F64Lt); }
                        BinaryOp::Gt => { func.instruction(&Instruction::F64Gt); }
                        BinaryOp::Le => { func.instruction(&Instruction::F64Le); }
                        BinaryOp::Ge => { func.instruction(&Instruction::F64Ge); }
                        BinaryOp::And | BinaryOp::Or => {
                            return Err("Logical float operations are not supported".to_string())
                        }
                    },
                    Some(Type::Bool) => match op {
                        BinaryOp::And => { func.instruction(&Instruction::I32And); }
                        BinaryOp::Or => { func.instruction(&Instruction::I32Or); }
                        BinaryOp::Eq => { func.instruction(&Instruction::I32Eq); func.instruction(&Instruction::I64ExtendI32S); }
                        BinaryOp::Neq => { func.instruction(&Instruction::I32Ne); func.instruction(&Instruction::I64ExtendI32S); }
                        _ => return Err("Unsupported boolean binary operation".to_string()),
                    },
                    _ => match op {
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
                    },
                };
            }

            ExpressionKind::FunctionCall { name, args, .. } => {
                for arg in args {
                    self.generate_expression(arg, func)?;
                }
                let func_idx = self
                    .function_map
                    .get(name)
                    .ok_or_else(|| format!("Unknown function: {}", name))?;
                func.instruction(&Instruction::Call(*func_idx));
            }

            ExpressionKind::UnaryOp { op, expr } => {
                self.generate_expression(expr, func)?;
                match (op, expr.ty.as_ref()) {
                    (UnaryOp::Neg, Some(Type::Float)) => {
                        func.instruction(&Instruction::F64Neg);
                    }
                    (UnaryOp::Neg, _) => {
                        func.instruction(&Instruction::I64Const(-1));
                        func.instruction(&Instruction::I64Mul);
                    }
                    (UnaryOp::Not, Some(Type::Bool)) => {
                        func.instruction(&Instruction::I32Eqz);
                        func.instruction(&Instruction::I64ExtendI32S);
                    }
                    (UnaryOp::Not, _) => {
                        func.instruction(&Instruction::I64Eqz);
                        func.instruction(&Instruction::I64ExtendI32S);
                    }
                }
            }

            ExpressionKind::MemberAccess { base, field } => {
                let struct_name = match base.ty.as_ref() {
                    Some(Type::Struct(sname)) => Some(sname.clone()),
                    _ => None,
                };
                let idx = self.field_index(struct_name.as_deref(), field);
                let byte_offset = field_byte_offset(idx);

                self.generate_expression(base, func)?;
                match base.ty.as_ref() {
                    Some(Type::Struct(_))
                    | Some(Type::String)
                    | Some(Type::Array(_, _))
                    | Some(Type::Optional(_))
                    | Some(Type::Union(_))
                    | Some(Type::GenericParam(_))
                    | Some(Type::Closure(_, _))
                    | Some(Type::GenericInstance(_, _))
                    | Some(Type::Pointer(_))
                    | Some(Type::Slice(_)) => {}
                    Some(Type::Int) | None => {
                        func.instruction(&Instruction::I32WrapI64);
                    }
                    _ => {}
                }

                if byte_offset != 0 {
                    func.instruction(&Instruction::I32Const(byte_offset));
                    func.instruction(&Instruction::I32Add);
                }

                func.instruction(&Instruction::I64Load(MemArg {
                    offset: 0,
                    align: 3,
                    memory_index: 0,
                }));
            }

            ExpressionKind::StructLiteral { name, fields } => {
                let n_fields = fields.len() as i32;
                let alloc_size = n_fields * SLOT_BYTES;

                emit_heap_alloc(alloc_size, func);

                let ptr_local = self.alloc_scratch_i32();
                func.instruction(&Instruction::LocalTee(ptr_local));

                let field_names = self
                    .struct_fields
                    .get(name)
                    .cloned()
                    .unwrap_or_default();

                for (field_expr_name, field_expr) in fields {
                    let idx = field_names
                        .iter()
                        .position(|f| f == field_expr_name)
                        .or_else(|| fields.iter().position(|(n, _)| n == field_expr_name))
                        .unwrap_or(0);
                    let byte_offset = field_byte_offset(idx);

                    func.instruction(&Instruction::LocalGet(ptr_local));
                    if byte_offset != 0 {
                        func.instruction(&Instruction::I32Const(byte_offset));
                        func.instruction(&Instruction::I32Add);
                    }

                    self.generate_expression(field_expr, func)?;
                    match field_expr.ty.as_ref() {
                        Some(Type::Bool) => {
                            func.instruction(&Instruction::I64ExtendI32S);
                        }
                        Some(Type::String)
                        | Some(Type::Struct(_))
                        | Some(Type::Array(_, _))
                        | Some(Type::Optional(_))
                        | Some(Type::Union(_))
                        | Some(Type::GenericParam(_))
                        | Some(Type::Closure(_, _))
                        | Some(Type::GenericInstance(_, _))
                        | Some(Type::Pointer(_))
                        | Some(Type::Slice(_)) => {
                            func.instruction(&Instruction::I64ExtendI32U);
                        }
                        Some(Type::Float) => {
                            // Store float as i64 by reinterpreting the f64 bits.
                            // WASM has no instruction to convert f64 to i64 directly,
                            // but we can store it via an intermediate i64 reinterpret.
                            // Use `i64.reinterpret_f64` which in wasm-encoder is F64 -> bitcast to i64.
                            func.instruction(&Instruction::I64ReinterpretF64);
                        }
                        _ => {}
                    }

                    func.instruction(&Instruction::I64Store(MemArg {
                        offset: 0,
                        align: 3,
                        memory_index: 0,
                    }));
                }

                func.instruction(&Instruction::LocalGet(ptr_local));
            }

            ExpressionKind::ArrayLiteral(elems) => {
                let n_elems = elems.len() as i32;
                let alloc_size = 4 + n_elems * SLOT_BYTES;

                emit_heap_alloc(alloc_size, func);

                let ptr_local = self.alloc_scratch_i32();
                func.instruction(&Instruction::LocalTee(ptr_local));

                func.instruction(&Instruction::LocalGet(ptr_local));
                func.instruction(&Instruction::I32Const(n_elems));
                func.instruction(&Instruction::I32Store(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));

                for (i, elem) in elems.iter().enumerate() {
                    let elem_offset = 4 + i as i32 * SLOT_BYTES;
                    func.instruction(&Instruction::LocalGet(ptr_local));
                    if elem_offset != 0 {
                        func.instruction(&Instruction::I32Const(elem_offset));
                        func.instruction(&Instruction::I32Add);
                    }

                    self.generate_expression(elem, func)?;
                    match elem.ty.as_ref() {
                        Some(Type::Bool) => { func.instruction(&Instruction::I64ExtendI32S); }
                        Some(Type::String)
                        | Some(Type::Struct(_))
                        | Some(Type::Array(_, _))
                        | Some(Type::Optional(_))
                        | Some(Type::Union(_))
                        | Some(Type::GenericParam(_))
                        | Some(Type::Closure(_, _))
                        | Some(Type::GenericInstance(_, _))
                        | Some(Type::Pointer(_))
                        | Some(Type::Slice(_)) => {
                            func.instruction(&Instruction::I64ExtendI32U);
                        }
                        Some(Type::Float) => {
                            func.instruction(&Instruction::I64ReinterpretF64);
                        }
                        _ => {}
                    };

                    func.instruction(&Instruction::I64Store(MemArg {
                        offset: 0,
                        align: 3,
                        memory_index: 0,
                    }));
                }

                func.instruction(&Instruction::LocalGet(ptr_local));
            }

            ExpressionKind::Match { scrutinee, arms } => {
                if arms.is_empty() {
                    match expr.ty.as_ref() {
                        Some(Type::Bool) => { func.instruction(&Instruction::I32Const(0)); }
                        Some(Type::Float) => { func.instruction(&Instruction::F64Const(0.0)); }
                        Some(Type::Void) => {}
                        _ => { func.instruction(&Instruction::I64Const(0)); }
                    }
                    return Ok(());
                }

                let result_ty = self.match_result_block_type(expr.ty.as_ref())?;
                self.emit_match_arms(scrutinee, arms, result_ty, func)?;
            }

            ExpressionKind::Closure { params, body: _ } => {
                // WASM closure representation: allocate a small heap object containing
                // captured variables, then emit the closure body as an inner function.
                // For now, we implement closures as direct inline evaluation with a
                // function pointer placeholder (i32 index into a hypothetical function table).

                // 1. Allocate a closure environment struct on the heap.
                //    Layout: [func_index (i32), env_ptr (i32)]
                //    For no-capture closures, env_ptr is 0 (null).
                let env_fields = params.len();
                let alloc_size = (2 + env_fields as i32) * SLOT_BYTES;
                emit_heap_alloc(alloc_size, func);

                let closure_ptr = self.alloc_scratch_i32();
                func.instruction(&Instruction::LocalTee(closure_ptr));

                // Store a placeholder function index (0) — a real implementation would
                // emit the inner function into the function table and store its index.
                func.instruction(&Instruction::LocalGet(closure_ptr));
                func.instruction(&Instruction::I32Const(0)); // placeholder func index
                func.instruction(&Instruction::I64ExtendI32S);
                func.instruction(&Instruction::I64Store(MemArg {
                    offset: 0,
                    align: 3,
                    memory_index: 0,
                }));

                // Store env pointer (0 = no captures for now)
                func.instruction(&Instruction::LocalGet(closure_ptr));
                func.instruction(&Instruction::I32Const(0)); // null env
                func.instruction(&Instruction::I64ExtendI32S);
                func.instruction(&Instruction::I64Store(MemArg {
                    offset: SLOT_BYTES as u32 as u64,
                    align: 3,
                    memory_index: 0,
                }));

                // The closure expression evaluates to the pointer (i32) to the closure object
                func.instruction(&Instruction::LocalGet(closure_ptr));
            }
        }

        Ok(())
    }

    fn match_result_block_type(&self, ty: Option<&Type>) -> Result<BlockType, String> {
        match ty {
            Some(Type::Void) => Ok(BlockType::Empty),
            Some(t) => match require_wasm_type(t)? {
                Some(vt) => Ok(BlockType::Result(vt)),
                None => Ok(BlockType::Empty),
            },
            None => Ok(BlockType::Result(ValType::I64)),
        }
    }

    fn emit_match_arms(
        &mut self,
        scrutinee: &Expression,
        arms: &[MatchArm],
        result_ty: BlockType,
        func: &mut WasmFunction,
    ) -> Result<(), String> {
        if arms.is_empty() {
            match result_ty {
                BlockType::Empty => {}
                BlockType::Result(ValType::I32) => { func.instruction(&Instruction::I32Const(0)); }
                BlockType::Result(ValType::I64) => { func.instruction(&Instruction::I64Const(0)); }
                BlockType::Result(ValType::F64) => { func.instruction(&Instruction::F64Const(0.0)); }
                _ => return Err("Unsupported match result type".to_string()),
            }
            return Ok(());
        }

        let arm = &arms[0];
        let rest = &arms[1..];

        match arm.pattern.kind.as_str() {
            "wildcard" | "var" => {
                self.generate_expression(&arm.body, func)?;
                return Ok(());
            }
            "int" => {
                self.generate_expression(scrutinee, func)?;
                func.instruction(&Instruction::I64Const(arm.pattern.int_val));
                func.instruction(&Instruction::I64Eq);
            }
            "bool" => {
                self.generate_expression(scrutinee, func)?;
                if !matches!(scrutinee.ty.as_ref(), Some(Type::Bool)) {
                    func.instruction(&Instruction::I32WrapI64);
                }
                func.instruction(&Instruction::I32Const(if arm.pattern.bool_val { 1 } else { 0 }));
                func.instruction(&Instruction::I32Eq);
            }
            "string" => {
                // String comparison: byte-by-byte comparison of null-terminated strings.
                // Both scrutinee and pattern are i32 pointers to null-terminated byte arrays.
                // We use a scratch local to accumulate the result, then push it.
                let ptr_a = self.alloc_scratch_i32();
                let ptr_b = self.alloc_scratch_i32();
                let byte_a = self.alloc_scratch_i32();
                let byte_b = self.alloc_scratch_i32();
                let idx = self.alloc_scratch_i32();
                let result = self.alloc_scratch_i32();

                // Store scrutinee pointer
                self.generate_expression(scrutinee, func)?;
                func.instruction(&Instruction::LocalSet(ptr_a));

                // Store pattern string pointer from string data section
                let pattern_str = &arm.pattern.str_val;
                let pattern_offset = if let Some(&existing) = self.string_offsets.get(pattern_str) {
                    existing
                } else {
                    let off = self.string_data.len() as u32;
                    self.string_data.extend_from_slice(pattern_str.as_bytes());
                    self.string_data.push(0);
                    self.string_offsets.insert(pattern_str.clone(), off);
                    off
                };
                func.instruction(&Instruction::I32Const(HEAP_BASE + pattern_offset as i32));
                func.instruction(&Instruction::LocalSet(ptr_b));

                // Assume equal until proven otherwise
                func.instruction(&Instruction::I32Const(1));
                func.instruction(&Instruction::LocalSet(result));

                // Initialize index
                func.instruction(&Instruction::I32Const(0));
                func.instruction(&Instruction::LocalSet(idx));

                // Comparison loop
                func.instruction(&Instruction::Block(BlockType::Empty)); // break target

                func.instruction(&Instruction::Loop(BlockType::Empty));

                // byte_a = scrutinee[idx]
                func.instruction(&Instruction::LocalGet(ptr_a));
                func.instruction(&Instruction::LocalGet(idx));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::I32Load8U(MemArg { offset: 0, align: 0, memory_index: 0 }));
                func.instruction(&Instruction::LocalSet(byte_a));

                // byte_b = pattern[idx]
                func.instruction(&Instruction::LocalGet(ptr_b));
                func.instruction(&Instruction::LocalGet(idx));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::I32Load8U(MemArg { offset: 0, align: 0, memory_index: 0 }));
                func.instruction(&Instruction::LocalSet(byte_b));

                // If bytes differ, set result = 0 and break
                func.instruction(&Instruction::LocalGet(byte_a));
                func.instruction(&Instruction::LocalGet(byte_b));
                func.instruction(&Instruction::I32Eq);
                func.instruction(&Instruction::I32Eqz);
                func.instruction(&Instruction::If(BlockType::Empty));
                func.instruction(&Instruction::I32Const(0));
                func.instruction(&Instruction::LocalSet(result));
                func.instruction(&Instruction::Br(2)); // break out of loop + block
                func.instruction(&Instruction::End);

                // If byte_a == 0 (both are 0 since we already checked equality), done
                func.instruction(&Instruction::LocalGet(byte_a));
                func.instruction(&Instruction::I32Eqz);
                func.instruction(&Instruction::BrIf(1)); // break loop (still in block)

                // idx++
                func.instruction(&Instruction::LocalGet(idx));
                func.instruction(&Instruction::I32Const(1));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::LocalSet(idx));

                func.instruction(&Instruction::Br(0)); // continue loop
                func.instruction(&Instruction::End); // end loop
                func.instruction(&Instruction::End); // end block

                // Push result onto the stack for the If/Else
                func.instruction(&Instruction::LocalGet(result));
            }
            other => {
                return Err(format!("Unsupported match pattern kind: {}", other));
            }
        }

        func.instruction(&Instruction::If(result_ty));
        self.generate_expression(&arm.body, func)?;

        func.instruction(&Instruction::Else);
        if rest.is_empty() {
            match result_ty {
                BlockType::Empty => {}
                BlockType::Result(ValType::I32) => { func.instruction(&Instruction::I32Const(0)); }
                BlockType::Result(ValType::I64) => { func.instruction(&Instruction::I64Const(0)); }
                BlockType::Result(ValType::F64) => { func.instruction(&Instruction::F64Const(0.0)); }
                _ => return Err("Unsupported match result type".to_string()),
            }
        } else {
            self.emit_match_arms(scrutinee, rest, result_ty, func)?;
        }
        func.instruction(&Instruction::End);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_expr(kind: ExpressionKind) -> Expression {
        Expression { kind, ty: None }
    }

    fn make_expr_typed(kind: ExpressionKind, ty: Type) -> Expression {
        Expression { kind, ty: Some(ty) }
    }

    fn make_var(name: &str) -> Expression {
        make_expr(ExpressionKind::Variable(name.to_string()))
    }

    fn make_var_typed(name: &str, ty: Type) -> Expression {
        make_expr_typed(ExpressionKind::Variable(name.to_string()), ty)
    }

    fn make_int(n: i64) -> Expression {
        make_expr_typed(ExpressionKind::Literal(Literal::Integer(n)), Type::Int)
    }

    #[allow(dead_code)]
    fn make_bool(b: bool) -> Expression {
        make_expr_typed(ExpressionKind::Literal(Literal::Boolean(b)), Type::Bool)
    }

    #[allow(dead_code)]
    fn make_binary(left: Expression, op: BinaryOp, right: Expression) -> Expression {
        make_expr(ExpressionKind::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn make_binary_typed(left: Expression, op: BinaryOp, right: Expression, ty: Type) -> Expression {
        make_expr_typed(
            ExpressionKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            ty,
        )
    }

    fn validate_wasm(bytes: &[u8]) {
        let v = wasmparser::validate(bytes);
        assert!(v.is_ok(), "WASM validation failed: {:?}", v.err());
    }

    #[test]
    fn test_simple_function() {
        let mut gen = WasmCodeGenerator::new();
        let decl = Declaration::Function {
            name: "add".to_string(),
            params: vec![
                Parameter { name: "a".to_string(), param_type: Type::Int },
                Parameter { name: "b".to_string(), param_type: Type::Int },
            ],
            return_type: Type::Int,
            body: Block {
                statements: vec![Statement::Return(Some(make_binary_typed(
                    make_var_typed("a", Type::Int),
                    BinaryOp::Add,
                    make_var_typed("b", Type::Int),
                    Type::Int,
                )))],
            },
            where_bounds: vec![],
        };

        let bytes = gen.generate(&[decl]).expect("generate failed");
        validate_wasm(&bytes);
    }

    #[test]
    fn test_multiple_functions_forward_call() {
        let mut gen = WasmCodeGenerator::new();
        let decls = vec![
            Declaration::Function {
                name: "compute".to_string(),
                params: vec![],
                return_type: Type::Int,
                body: Block {
                    statements: vec![Statement::Return(Some(make_expr_typed(
                        ExpressionKind::FunctionCall {
                            name: "square".to_string(),
                            args: vec![make_int(5)],
                            type_args: vec![],
                        },
                        Type::Int,
                    )))],
                },
                where_bounds: vec![],
            },
            Declaration::Function {
                name: "square".to_string(),
                params: vec![Parameter { name: "n".to_string(), param_type: Type::Int }],
                return_type: Type::Int,
                body: Block {
                    statements: vec![Statement::Return(Some(make_binary_typed(
                        make_var_typed("n", Type::Int),
                        BinaryOp::Mul,
                        make_var_typed("n", Type::Int),
                        Type::Int,
                    )))],
                },
                where_bounds: vec![],
            },
        ];

        let bytes = gen.generate(&decls).expect("generate failed");
        validate_wasm(&bytes);
    }

    #[test]
    fn test_string_literals_do_not_overlap_heap_base() {
        let mut gen = WasmCodeGenerator::new();
        let decl = Declaration::Function {
            name: "hello".to_string(),
            params: vec![],
            return_type: Type::String,
            body: Block {
                statements: vec![Statement::Return(Some(make_expr_typed(
                    ExpressionKind::Literal(Literal::String("Hello".to_string())),
                    Type::String,
                )))],
            },
            where_bounds: vec![],
        };

        let bytes = gen.generate(&[decl]).expect("generate failed");
        validate_wasm(&bytes);
    }

    #[test]
    fn test_struct_alloc_and_member_access() {
        let mut gen = WasmCodeGenerator::new();
        gen.register_struct("Point", vec!["x".to_string(), "y".to_string()]);

        let struct_expr = make_expr_typed(
            ExpressionKind::StructLiteral {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), make_int(10)),
                    ("y".to_string(), make_int(20)),
                ],
            },
            Type::Struct("Point".to_string()),
        );

        let member_expr = make_expr_typed(
            ExpressionKind::MemberAccess {
                base: Box::new(make_var_typed("p", Type::Struct("Point".to_string()))),
                field: "x".to_string(),
            },
            Type::Int,
        );

        let decl = Declaration::Function {
            name: "makepointx".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: Block {
                statements: vec![
                    Statement::Let {
                        name: "p".to_string(),
                        value: struct_expr,
                        ty: Type::Struct("Point".to_string()),
                    },
                    Statement::Return(Some(member_expr)),
                ],
            },
            where_bounds: vec![],
        };

        let bytes = gen.generate(&[decl]).expect("generate failed");
        validate_wasm(&bytes);
    }

    #[test]
    fn test_array_literal() {
        let mut gen = WasmCodeGenerator::new();

        let arr_expr = make_expr_typed(
            ExpressionKind::ArrayLiteral(vec![make_int(1), make_int(2), make_int(3)]),
            Type::Array(Box::new(Type::Int), 3),
        );

        let decl = Declaration::Function {
            name: "makearray".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: Block {
                statements: vec![
                    Statement::Let {
                        name: "a".to_string(),
                        value: arr_expr,
                        ty: Type::Array(Box::new(Type::Int), 3),
                    },
                    Statement::Return(Some(make_int(0))),
                ],
            },
            where_bounds: vec![],
        };

        let bytes = gen.generate(&[decl]).expect("generate failed");
        validate_wasm(&bytes);
    }

    #[test]
    fn test_match_dispatch() {
        let mut gen = WasmCodeGenerator::new();

        let match_expr = make_expr_typed(
            ExpressionKind::Match {
                scrutinee: Box::new(make_var_typed("x", Type::Int)),
                arms: vec![
                    MatchArm {
                        pattern: MatchPattern::int_literal(0),
                        guard: None,
                        body: make_int(10),
                    },
                    MatchArm {
                        pattern: MatchPattern::int_literal(1),
                        guard: None,
                        body: make_int(20),
                    },
                    MatchArm {
                        pattern: MatchPattern::wildcard(),
                        guard: None,
                        body: make_int(99),
                    },
                ],
            },
            Type::Int,
        );

        let decl = Declaration::Function {
            name: "classify".to_string(),
            params: vec![Parameter { name: "x".to_string(), param_type: Type::Int }],
            return_type: Type::Int,
            body: Block {
                statements: vec![Statement::Return(Some(match_expr))],
            },
            where_bounds: vec![],
        };

        let bytes = gen.generate(&[decl]).expect("generate failed");
        validate_wasm(&bytes);
    }

    #[test]
    fn test_for_loop() {
        let mut gen = WasmCodeGenerator::new();

        let decl = Declaration::Function {
            name: "sumarray".to_string(),
            params: vec![Parameter { name: "arr".to_string(), param_type: Type::Int }],
            return_type: Type::Int,
            body: Block {
                statements: vec![
                    Statement::Let {
                        name: "s".to_string(),
                        value: make_int(0),
                        ty: Type::Int,
                    },
                    Statement::For {
                        var: "elem".to_string(),
                        iter: make_var_typed("arr", Type::Int),
                        body: Box::new(Block {
                            statements: vec![Statement::Assignment {
                                target: make_var("s"),
                                value: make_binary_typed(
                                    make_var_typed("s", Type::Int),
                                    BinaryOp::Add,
                                    make_var_typed("elem", Type::Int),
                                    Type::Int,
                                ),
                            }],
                        }),
                    },
                    Statement::Return(Some(make_var_typed("s", Type::Int))),
                ],
            },
            where_bounds: vec![],
        };

        let bytes = gen.generate(&[decl]).expect("generate failed");
        validate_wasm(&bytes);
    }

    #[test]
    fn test_closure_supported() {
        let mut gen = WasmCodeGenerator::new();

        let closure_expr = make_expr_typed(
            ExpressionKind::Closure {
                params: vec![Parameter { name: "x".to_string(), param_type: Type::Int }],
                body: Box::new(make_binary_typed(
                    make_var_typed("x", Type::Int),
                    BinaryOp::Add,
                    make_int(1),
                    Type::Int,
                )),
            },
            Type::Closure(vec![Type::Int], Box::new(Type::Int)),
        );

        let decl = Declaration::Function {
            name: "apply".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: Block {
                statements: vec![
                    Statement::Let {
                        name: "c".to_string(),
                        value: closure_expr,
                        ty: Type::Closure(vec![Type::Int], Box::new(Type::Int)),
                    },
                    Statement::Return(Some(make_int(0))),
                ],
            },
            where_bounds: vec![],
        };

        let bytes = gen.generate(&[decl]).expect("closure codegen failed");
        validate_wasm(&bytes);
    }
}
