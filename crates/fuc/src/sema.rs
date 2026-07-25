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
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    Variable(String),
    FunctionCall { name: String, args: Vec<TypedExpression> },
    BinaryOperation { left: Box<TypedExpression>, right: Box<TypedExpression>, op: ir::BinaryOp },
    UnaryOperation { op: UnaryOp, expr: Box<TypedExpression> },
    ArrayLiteral(Vec<TypedExpression>),
    StructLiteral { name: String, fields: Vec<(String, ir::Type, TypedExpression)> },
    MemberAccess { base: Box<TypedExpression>, field: String, field_index: usize },
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
                Declaration::ExternFunction { name, params, return_type, .. } => {
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

        // Register struct definitions from the dedicated structs list
        for s in &prog.structs {
            let fields: Vec<(String, ir::Type)> = s.fields.iter().map(|(n, t)| (n.clone(), convert_type(t))).collect();
            self.structs.insert(s.name.clone(), fields);
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
                Some(TypedStatement::Let {
                    name: name.clone(),
                    value: inferred,
                    ty: actual_ty,
                })
            }
            Statement::VariableDeclaration { name, initializer, ty } => {
                let inferred = self.type_check_expr(initializer, locals, errors);
                let declared_ty = ty.as_ref().map(convert_type).unwrap_or(ir::Type::Unknown);
                let actual_ty = if declared_ty != ir::Type::Unknown { declared_ty } else { inferred.ty.clone() };
                locals.insert(name.clone(), actual_ty.clone());
                Some(TypedStatement::Let {
                    name: name.clone(),
                    value: inferred,
                    ty: actual_ty,
                })
            }
            Statement::Assignment { target, value } => {
                let typed_val = self.type_check_expr(value, locals, errors);
                let typed_target = self.type_check_expr(target, locals, errors);
                Some(TypedStatement::Assignment {
                    target: typed_target,
                    value: typed_val,
                })
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
                Some(TypedStatement::If {
                    cond: typed_cond,
                    then_block: typed_then,
                    else_block: typed_else,
                })
            }
            Statement::While { cond, body } => {
                let typed_cond = self.type_check_expr(cond, locals, errors);
                let typed_body = self.type_check_block(body, locals, expected_return, errors);
                Some(TypedStatement::While {
                    cond: typed_cond,
                    body: typed_body,
                })
            }
            Statement::For { var, iter, body } => {
                let typed_iter = self.type_check_expr(iter, locals, errors);
                // Infer loop variable type from iterator
                let iter_ty = match &typed_iter.ty {
                    ir::Type::Array(elem, _) | ir::Type::Slice(elem) => (**elem).clone(),
                    _ => ir::Type::Int,
                };
                locals.insert(var.clone(), iter_ty);
                let typed_body = self.type_check_block(body, locals, expected_return, errors);
                Some(TypedStatement::For {
                    var: var.clone(),
                    iter: typed_iter,
                    body: typed_body,
                })
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
                (TypedExpressionKind::FloatLiteral(*n), ir::Type::Float)
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
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge | BinaryOp::And | BinaryOp::Or => ir::Type::Bool,
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
                (TypedExpressionKind::UnaryOperation {
                    op: *op,
                    expr: Box::new(typed_inner),
                }, result_ty)
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
                let (field_ty, field_index) = match &typed_base.ty {
                    ir::Type::Struct(name) => {
                        match self.structs.get(name) {
                            Some(fields) => {
                                match fields.iter().enumerate().find(|(_, (n, _))| n == field) {
                                    Some((idx, (_, ty))) => (ty.clone(), idx),
                                    None => {
                                        errors.push(format!(
                                            "unknown field '{}' on struct '{}'",
                                            field, name
                                        ));
                                        (ir::Type::Unknown, 0)
                                    }
                                }
                            }
                            None => {
                                errors.push(format!("unknown struct '{}'", name));
                                (ir::Type::Unknown, 0)
                            }
                        }
                    }
                    other => {
                        errors.push(format!("member access on non-struct type: {:?}", other));
                        (ir::Type::Unknown, 0)
                    }
                };
                (TypedExpressionKind::MemberAccess {
                    base: Box::new(typed_base),
                    field: field.clone(),
                    field_index,
                }, field_ty)
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
                    let guard = arm.guard.as_ref().map(|g| {
                        let tg = self.type_check_expr(g, locals, errors);
                        tg
                    });
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
                (TypedExpressionKind::Closure {
                    params: typed_params,
                    body: Box::new(typed_body),
                }, ir::Type::Closure(param_types, Box::new(ret_ty)))
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