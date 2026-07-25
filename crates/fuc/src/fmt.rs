//! Fusion Code Formatter (fuc fmt)
//! Addresses: No formatter, Developer Experience gaps.
use crate::types::*;

use crate::ast::*;

pub struct Formatter {
    indent_level: usize,
    indent_string: FString,
    output: FString,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            indent_string: "    ".to_string(), // 4 spaces default
            output: String::new(),
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str(&self.indent_string);
        }
    }

    fn newline(&mut self) {
        self.output.push('\n');
    }

    pub fn format_program(&mut self, prog: &Program) -> FString {
        for s in &prog.structs {
            self.format_struct(s);
            self.newline();
        }
        for f in &prog.functions {
            self.format_function(f);
            self.newline();
        }
        std::mem::replace(&mut self.output, String::new())
    }

    fn format_type(&mut self, ty: &Type) {
        let ty_str = match ty {
            Type::Int => "int".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Struct(name) => name.clone(),
            Type::Pointer(_inner) => "*".to_string(), // Needs recursive formatting
            _ => "unknown".to_string(),
        };
        self.output.push_str(&ty_str);
    }

    fn format_struct(&mut self, def: &StructDefinition) {
        self.push_indent();
        self.output.push_str(&format!("struct {} {{\n", def.name));
        self.indent_level += 1;
        
        for (field_name, field_ty) in &def.fields {
            self.push_indent();
            self.output.push_str(&format!("{}: ", field_name));
            self.format_type(field_ty);
            self.output.push_str(",\n");
        }
        
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}\n");
    }

    fn format_function(&mut self, func: &Function) {
        self.push_indent();
        self.output.push_str(&format!("fn {}(", func.name));
        
        for (i, param) in func.params.iter().enumerate() {
            self.output.push_str(&format!("{}: ", param.name));
            self.format_type(&param.param_type);
            if i < func.params.len() - 1 {
                self.output.push_str(", ");
            }
        }
        
        self.output.push_str(") -> ");
        self.format_type(&func.return_type);
        self.output.push_str(" {\n");
        
        self.indent_level += 1;
        for stmt in &func.body.statements {
            self.format_statement(stmt);
        }
        self.indent_level -= 1;
        
        self.push_indent();
        self.output.push_str("}\n");
    }

    fn format_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, value, ty: _ } => {
                self.push_indent();
                self.output.push_str(&format!("let {} = ", name));
                self.format_expression(value);
                self.output.push_str(";\n");
            }
            Statement::Assignment { target, value } => {
                self.push_indent();
                self.format_expression(target);
                self.output.push_str(" = ");
                self.format_expression(value);
                self.output.push_str(";\n");
            }
            Statement::Expression(expr) => {
                self.push_indent();
                self.format_expression(expr);
                self.output.push_str(";\n");
            }
            Statement::Return(Some(expr)) => {
                self.push_indent();
                self.output.push_str("return ");
                self.format_expression(expr);
                self.output.push_str(";\n");
            }
            Statement::Return(None) => {
                self.push_indent();
                self.output.push_str("return;\n");
            }
            Statement::VariableDeclaration { name, initializer, ty: _ } => {
                self.push_indent();
                self.output.push_str(&format!("var {} = ", name));
                self.format_expression(initializer);
                self.output.push_str(";\n");
            }
            Statement::If { cond, then_block, else_block } => {
                self.push_indent();
                self.output.push_str("if ");
                self.format_expression(cond);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for s in &then_block.statements {
                    self.format_statement(s);
                }
                self.indent_level -= 1;
                if let Some(else_b) = else_block {
                    self.push_indent();
                    self.output.push_str("} else {\n");
                    self.indent_level += 1;
                    for s in &else_b.statements {
                        self.format_statement(s);
                    }
                    self.indent_level -= 1;
                }
                self.push_indent();
                self.output.push_str("}\n");
            }
            Statement::While { cond, body } => {
                self.push_indent();
                self.output.push_str("while ");
                self.format_expression(cond);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for s in &body.statements {
                    self.format_statement(s);
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            Statement::For { var, iter, body } => {
                self.push_indent();
                self.output.push_str(&format!("for {} in ", var));
                self.format_expression(iter);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for s in &body.statements {
                    self.format_statement(s);
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
        }
    }

    fn format_expression(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::Literal(lit) => match lit {
                Literal::Integer(v) => self.output.push_str(&v.to_string()),
                Literal::Float(v) => self.output.push_str(&v.to_string()),
                Literal::Boolean(v) => self.output.push_str(if *v { "true" } else { "false" }),
                Literal::String(v) => self.output.push_str(&format!("\"{}\"", v)),
            },
            ExpressionKind::Variable(name) => self.output.push_str(name),
            ExpressionKind::BinaryOp { left, op, right } => {
                self.output.push('(');
                self.format_expression(left);
                let op_str = match op {
                    BinaryOp::Add => " + ",
                    BinaryOp::Sub => " - ",
                    BinaryOp::Mul => " * ",
                    BinaryOp::Div => " / ",
                    BinaryOp::Mod => " % ",
                    BinaryOp::Eq => " == ",
                    BinaryOp::Neq => " != ",
                    BinaryOp::Lt => " < ",
                    BinaryOp::Gt => " > ",
                    BinaryOp::Le => " <= ",
                    BinaryOp::Ge => " >= ",
                    BinaryOp::And => " && ",
                    BinaryOp::Or => " || ",
                };
                self.output.push_str(op_str);
                self.format_expression(right);
                self.output.push(')');
            }
            ExpressionKind::UnaryOp { op, expr: inner } => {
                match op {
                    UnaryOp::Neg => self.output.push('-'),
                    UnaryOp::Not => self.output.push('!'),
                }
                self.format_expression(inner);
            }
            ExpressionKind::FunctionCall { name, args, .. } => {
                self.output.push_str(name);
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    self.format_expression(arg);
                    if i < args.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push(')');
            }
            ExpressionKind::MemberAccess { base, field } => {
                self.format_expression(base);
                self.output.push_str(&format!(".{}", field));
            }
            ExpressionKind::StructLiteral { name, fields } => {
                self.output.push_str(&format!("{} {{ ", name));
                for (i, (fname, fval)) in fields.iter().enumerate() {
                    self.output.push_str(&format!("{}: ", fname));
                    self.format_expression(fval);
                    if i < fields.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(" }");
            }
            ExpressionKind::ArrayLiteral(elems) => {
                self.output.push('[');
                for (i, e) in elems.iter().enumerate() {
                    self.format_expression(e);
                    if i < elems.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push(']');
            }
            ExpressionKind::Match { scrutinee, arms } => {
                self.output.push_str("match ");
                self.format_expression(scrutinee);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for arm in arms {
                    self.push_indent();
                    match arm.pattern.kind.as_str() {
                        "wildcard" => self.output.push('_'),
                        "int" => self.output.push_str(&arm.pattern.int_val.to_string()),
                        "bool" => self.output.push_str(if arm.pattern.bool_val { "true" } else { "false" }),
                        "string" => self.output.push_str(&format!("\"{}\"", arm.pattern.str_val)),
                        "var" => self.output.push_str(&arm.pattern.str_val),
                        _ => self.output.push('_'),
                    }
                    if let Some(guard) = &arm.guard {
                        self.output.push_str(" if ");
                        self.format_expression(guard);
                    }
                    self.output.push_str(" => ");
                    self.format_expression(&arm.body);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push('}');
            }
            ExpressionKind::Closure { params, body } => {
                self.output.push('|');
                for (i, p) in params.iter().enumerate() {
                    self.output.push_str(&p.name);
                    if i < params.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str("| ");
                self.format_expression(body);
            }
        }
    }
}