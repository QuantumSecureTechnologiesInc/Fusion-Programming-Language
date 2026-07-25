//! AST Generic Monomorphization Pass
//! Addresses: Limited generic monomorphization.
//!
//! Scans the AST for generic functions and structs, tracking instantiation
//! types at call sites, and generates concrete monomorphized copies.
use crate::types::*;
use std::collections::HashMap;

use crate::ast::{Program, Function, Type, StructDefinition, Declaration, Block, Statement, Expression, ExpressionKind};

pub struct Instantiation {
    pub generic_name: FString,
    pub concrete_types: FVec<Type>,
    pub new_mangled_name: FString,
}

pub struct Monomorphizer {
    generic_funcs: FMap<FString, Function>,
    generic_structs: FMap<FString, StructDefinition>,
    instantiations: FVec<Instantiation>,
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            generic_funcs: HashMap::new(),
            generic_structs: HashMap::new(),
            instantiations: Vec::new(),
        }
    }

    /// Primary entry point. Transforms a program with generics into a 
    /// program with purely concrete types.
    pub fn monomorphize_program(&mut self, mut prog: Program) -> Program {
        // Step 1: Extract and index all generic definitions
        self.extract_generics(&mut prog);

        // Step 2: Traverse AST to find all concrete uses (Function Calls, Struct Literals)
        self.collect_instantiations(&prog);

        // Step 3: Generate concrete copies based on instantiations
        let concrete_funcs = self.generate_concrete_functions();
        prog.functions.extend(concrete_funcs);

        // Step 4: Remove original generic templates (they cannot be compiled directly)
        prog.functions.retain(|f| f.generics.is_empty());
        prog.structs.retain(|s| s.generics.is_empty());

        prog
    }

    fn extract_generics(&mut self, prog: &mut Program) {
        for func in &prog.functions {
            if !func.generics.is_empty() {
                self.generic_funcs.insert(func.name.clone(), func.clone());
            }
        }
        for s in &prog.structs {
            if !s.generics.is_empty() {
                self.generic_structs.insert(s.name.clone(), s.clone());
            }
        }
    }

    fn collect_instantiations(&mut self, prog: &Program) {
        // Walk all functions looking for calls with type arguments
        for func in &prog.functions {
            self.collect_from_block(&func.body);
        }
        // Also scan top-level declarations for nested functions
        for decl in &prog.declarations {
            if let Declaration::Function { body, .. } = decl {
                self.collect_from_block(body);
            }
        }
    }

    fn collect_from_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.collect_from_statement(stmt);
        }
    }

    fn collect_from_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { value, .. } => self.collect_from_expression(value),
            Statement::Assignment { target, value } => {
                self.collect_from_expression(target);
                self.collect_from_expression(value);
            }
            Statement::Expression(expr) => self.collect_from_expression(expr),
            Statement::Return(Some(expr)) => self.collect_from_expression(expr),
            Statement::VariableDeclaration { initializer, .. } => {
                self.collect_from_expression(initializer);
            }
            Statement::If { cond, then_block, else_block } => {
                self.collect_from_expression(cond);
                self.collect_from_block(then_block);
                if let Some(eb) = else_block {
                    self.collect_from_block(eb);
                }
            }
            Statement::While { cond, body } => {
                self.collect_from_expression(cond);
                self.collect_from_block(body);
            }
            Statement::For { iter, body, .. } => {
                self.collect_from_expression(iter);
                self.collect_from_block(body);
            }
            Statement::Return(None) => {}
        }
    }

    fn collect_from_expression(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::FunctionCall { name, args, type_args } => {
                if !type_args.is_empty() && self.generic_funcs.contains_key(name) {
                    // Check if this exact instantiation already exists
                    let already_seen = self.instantiations.iter().any(|i| {
                        i.generic_name == *name && i.concrete_types == *type_args
                    });
                    if !already_seen {
                        let mangled = self.mangle_name(name, type_args);
                        self.instantiations.push(Instantiation {
                            generic_name: name.clone(),
                            concrete_types: type_args.clone(),
                            new_mangled_name: mangled,
                        });
                    }
                }
                // Recurse into arguments to find nested generic calls
                for arg in args {
                    self.collect_from_expression(arg);
                }
            }
            ExpressionKind::BinaryOp { left, right, .. } => {
                self.collect_from_expression(left);
                self.collect_from_expression(right);
            }
            ExpressionKind::UnaryOp { expr, .. } => {
                self.collect_from_expression(expr);
            }
            ExpressionKind::MemberAccess { base, .. } => {
                self.collect_from_expression(base);
            }
            ExpressionKind::StructLiteral { fields, .. } => {
                for (_, field_expr) in fields {
                    self.collect_from_expression(field_expr);
                }
            }
            ExpressionKind::ArrayLiteral(elems) => {
                for elem in elems {
                    self.collect_from_expression(elem);
                }
            }
            ExpressionKind::Match { scrutinee, arms } => {
                self.collect_from_expression(scrutinee);
                for arm in arms {
                    self.collect_from_expression(&arm.body);
                    if let Some(guard) = &arm.guard {
                        self.collect_from_expression(guard);
                    }
                }
            }
            ExpressionKind::Closure { body, .. } => {
                self.collect_from_expression(body);
            }
            ExpressionKind::Literal(_) | ExpressionKind::Variable(_) => {}
        }
    }

    fn mangle_name(&self, base: &str, types: &FVec<Type>) -> FString {
        let mut name = FString::from(base);
        for ty in types {
            name.push_str("_");
            let ty_str = match ty {
                Type::Int => "int",
                Type::Bool => "bool",
                Type::String => "string",
                Type::Struct(s) => s.as_str(),
                _ => "unk",
            };
            name.push_str(ty_str);
        }
        name
    }

    fn generate_concrete_functions(&self) -> FVec<Function> {
        let mut new_funcs = Vec::new();

        for inst in &self.instantiations {
            if let Some(template) = self.generic_funcs.get(&inst.generic_name) {
                let mut concrete_func = template.clone();
                concrete_func.name = inst.new_mangled_name.clone();
                concrete_func.generics.clear(); // Now it's concrete

                // Create a substitution map: T -> Int, U -> Bool
                let mut type_map: FMap<FString, Type> = HashMap::new();
                for (i, gen_name) in template.generics.iter().enumerate() {
                    if let Some(concrete_ty) = inst.concrete_types.get(i) {
                        type_map.insert(gen_name.clone(), concrete_ty.clone());
                    }
                }

                // Substitute parameter types
                for param in &mut concrete_func.params {
                    self.substitute_type(&mut param.param_type, &type_map);
                }
                
                // Substitute return type
                self.substitute_type(&mut concrete_func.return_type, &type_map);
                
                new_funcs.push(concrete_func);
            }
        }
        new_funcs
    }

    fn substitute_type(&self, ty: &mut Type, map: &FMap<FString, Type>) {
        match ty {
            Type::GenericParam(name) => {
                if let Some(concrete) = map.get(name) {
                    *ty = concrete.clone();
                }
            }
            Type::Pointer(inner) => self.substitute_type(inner, map),
            Type::Array(inner, _) => self.substitute_type(inner, map),
            Type::Slice(inner) => self.substitute_type(inner, map),
            _ => {}
        }
    }
}