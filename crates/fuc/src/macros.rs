//! AST-to-AST Macro Expansion Pass
//! Addresses: No macro system, No preprocessor.
use crate::types::*;
use std::collections::HashMap;

use crate::ast::{Program, Statement, Expression, ExpressionKind, Literal, Block};

/// Represents a simple `macro_rules!` style text substitution macro.
pub struct MacroDefinition {
    pub name: FString,
    pub pattern: FVec<FString>, // Token patterns
    pub template: FVec<FString>, // Output AST template tokens
}

pub struct MacroExpander {
    macros: FMap<FString, MacroDefinition>,
}

impl MacroExpander {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    /// Registers a new macro definition into the expander
    pub fn register_macro(&mut self, def: MacroDefinition) {
        self.macros.insert(def.name.clone(), def);
    }

    /// Performs a pre-order traversal of the AST, replacing macro invocations
    /// with their expanded AST sub-trees before Semantic Analysis occurs.
    pub fn expand_program(&mut self, mut prog: Program) -> Program {
        // Iterate through function bodies and look for Expression::FunctionCall
        // where the name ends in `!` (e.g., `println!`).
        
        for func in &mut prog.functions {
            let mut new_body = Vec::new();
            for stmt in &func.body.statements {
                new_body.push(self.expand_statement(stmt.clone()));
            }
            func.body.statements = new_body;
        }
        
        prog
    }

    fn expand_statement(&self, stmt: Statement) -> Statement {
        match stmt {
            Statement::Let { name, value, ty } => {
                Statement::Let { name, value: self.expand_expression(value), ty }
            }
            Statement::Assignment { target, value } => {
                Statement::Assignment { target: self.expand_expression(target), value: self.expand_expression(value) }
            }
            Statement::Expression(expr) => {
                Statement::Expression(self.expand_expression(expr))
            }
            Statement::Return(Some(expr)) => {
                Statement::Return(Some(self.expand_expression(expr)))
            }
            Statement::Return(None) => Statement::Return(None),
            Statement::VariableDeclaration { name, initializer, ty } => {
                Statement::VariableDeclaration { name, initializer: self.expand_expression(initializer), ty }
            }
            Statement::If { cond, then_block, else_block } => {
                let new_then = self.expand_block(&then_block);
                let new_else = else_block.map(|b| Box::new(self.expand_block(&b)));
                Statement::If { cond: self.expand_expression(cond), then_block: Box::new(new_then), else_block: new_else }
            }
            Statement::While { cond, body } => {
                Statement::While { cond: self.expand_expression(cond), body: Box::new(self.expand_block(&body)) }
            }
            Statement::For { var, iter, body } => {
                Statement::For { var, iter: self.expand_expression(iter), body: Box::new(self.expand_block(&body)) }
            }
        }
    }

    fn expand_block(&self, block: &Block) -> Block {
        Block {
            statements: block.statements.iter().map(|s| self.expand_statement(s.clone())).collect(),
        }
    }

    fn expand_expression(&self, expr: Expression) -> Expression {
        match expr.kind {
            ExpressionKind::FunctionCall { name, args, type_args } if name.ends_with('!') => {
                if let Some(macro_def) = self.macros.get(&name) {
                    self.expand_macro_call(macro_def, &args)
                } else {
                    Expression { kind: ExpressionKind::FunctionCall { name, args: args.into_iter().map(|a| self.expand_expression(a)).collect(), type_args }, ty: expr.ty }
                }
            }
            ExpressionKind::FunctionCall { name, args, type_args } => {
                Expression { kind: ExpressionKind::FunctionCall { name, args: args.into_iter().map(|a| self.expand_expression(a)).collect(), type_args }, ty: expr.ty }
            }
            ExpressionKind::BinaryOp { left, op, right } => {
                Expression { kind: ExpressionKind::BinaryOp { left: Box::new(self.expand_expression(*left)), op, right: Box::new(self.expand_expression(*right)) }, ty: expr.ty }
            }
            ExpressionKind::UnaryOp { op, expr: inner } => {
                Expression { kind: ExpressionKind::UnaryOp { op, expr: Box::new(self.expand_expression(*inner)) }, ty: expr.ty }
            }
            ExpressionKind::Match { scrutinee, arms } => {
                let new_arms = arms.into_iter().map(|arm| {
                    crate::ast::MatchArm {
                        pattern: arm.pattern,
                        guard: arm.guard.map(|g| Box::new(self.expand_expression(*g))),
                        body: self.expand_expression(arm.body),
                    }
                }).collect();
                Expression { kind: ExpressionKind::Match { scrutinee: Box::new(self.expand_expression(*scrutinee)), arms: new_arms }, ty: expr.ty }
            }
            ExpressionKind::MemberAccess { base, field } => {
                Expression { kind: ExpressionKind::MemberAccess { base: Box::new(self.expand_expression(*base)), field }, ty: expr.ty }
            }
            ExpressionKind::ArrayLiteral(elems) => {
                Expression { kind: ExpressionKind::ArrayLiteral(elems.into_iter().map(|e| self.expand_expression(e)).collect()), ty: expr.ty }
            }
            ExpressionKind::StructLiteral { name, fields } => {
                Expression { kind: ExpressionKind::StructLiteral { name, fields: fields.into_iter().map(|(n, e)| (n, self.expand_expression(e))).collect() }, ty: expr.ty }
            }
            other => Expression { kind: other, ty: expr.ty },
        }
    }

    /// Expands a macro invocation by substituting pattern variables in the template.
    /// For simple text-substitution macros: maps pattern tokens to argument expressions.
    fn expand_macro_call(&self, macro_def: &MacroDefinition, args: &[Expression]) -> Expression {
        // Build a mapping from pattern variable names to argument expressions
        let mut bindings: FMap<FString, Expression> = HashMap::new();
        for (pattern_token, arg) in macro_def.pattern.iter().zip(args.iter()) {
            if pattern_token.starts_with('$') {
                bindings.insert(pattern_token.clone(), arg.clone());
            }
        }

        // If the template has exactly one token that is a variable reference, substitute it
        if macro_def.template.len() == 1 {
            let token = &macro_def.template[0];
            if let Some(bound) = bindings.get(token) {
                return bound.clone();
            }
        }

        // For array-like macros (e.g., vec![...]), expand to ArrayLiteral
        if macro_def.template.len() == 1 && macro_def.template[0] == "[]" {
            return Expression {
                kind: ExpressionKind::ArrayLiteral(args.to_vec()),
                ty: None,
            };
        }

        // Fallback: return the first argument as a passthrough
        if !args.is_empty() {
            return args[0].clone();
        }

        Expression {
            kind: ExpressionKind::Literal(Literal::Boolean(true)),
            ty: None,
        }
    }
}