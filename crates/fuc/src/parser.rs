//! Fusion Parser
//! Recursive-descent parser with Pratt expression parsing.
//! Uses the existing lexer and maps to the existing AST types.

use crate::ast::{self, BinaryOp, Block, ConstDeclaration, Declaration, EnumDefinition, EnumVariant, EnumVariantData, Expression, ExpressionKind, Function, ImplBlock, Literal, MatchArm, MatchPattern, Parameter, Span, Statement, StaticDeclaration, StructDefinition, TraitDefinition, TraitMethod, Type, TypeAlias, UnaryOp};
use crate::lexer::{self, Token};
use crate::types::*;

/// Parser output containing the parsed program and any errors.
pub struct ParserOutput {
    pub program: Option<ast::Program>,
    pub errors: FVec<FString>,
}

// ---- Parser struct ----

struct Parser {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    pos: usize,
    errors: Vec<String>,
}

impl Parser {
    fn new(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Self {
        Parser { tokens, pos: 0, errors: Vec::new() }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn _peek_span(&self) -> Span {
        self.tokens.get(self.pos).map(|(_, r)| Span::new(r.start, r.end)).unwrap_or_default()
    }

    fn advance(&mut self) -> Option<(Token, std::ops::Range<usize>)> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.peek() {
            Some(t) if std::mem::discriminant(t) == std::mem::discriminant(&expected) => {
                self.advance();
                Ok(())
            }
            Some(t) => {
                Err(format!("Expected {:?}, found {:?}", expected, t))
            }
            None => Err(format!("Expected {:?}, found end of file", expected)),
        }
    }

    fn skip_semicolons(&mut self) {
        while self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
    }

    // ---- Top-level parsing ----

    fn parse_program(&mut self) -> ast::Program {
        let mut declarations: Vec<Declaration> = Vec::new();
        let mut structs: Vec<StructDefinition> = Vec::new();

        self.skip_semicolons();

        while self.pos < self.tokens.len() {
            match self.peek() {
                Some(Token::KwFn) => {
                    match self.parse_fn_decl() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwExtern) => {
                    match self.parse_extern_decl() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwStruct) => {
                    match self.parse_struct_def() {
                        Ok(s) => structs.push(s),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwMod) => {
                    match self.parse_mod_decl() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwUse) => {
                    match self.parse_use_decl() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwImport) => {
                    match self.parse_import_decl() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwPub) => {
                    // Skip pub keyword, then try to parse the item
                    self.advance();
                    // After pub, try fn/struct/extern/enum/impl/trait/const/static/type
                    match self.peek() {
                        Some(Token::KwFn) => {
                            match self.parse_fn_decl() {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                            }
                        }
                        Some(Token::KwStruct) => {
                            match self.parse_struct_def() {
                                Ok(s) => structs.push(s),
                                Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                            }
                        }
                        Some(Token::KwEnum) => {
                            match self.parse_enum_def() {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                            }
                        }
                        Some(Token::KwImpl) => {
                            match self.parse_impl_block() {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                            }
                        }
                        Some(Token::KwTrait) => {
                            match self.parse_trait_def() {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                            }
                        }
                        Some(Token::KwConst) => {
                            match self.parse_const_decl() {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                            }
                        }
                        Some(Token::KwStatic) => {
                            match self.parse_static_decl() {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                            }
                        }
                        Some(Token::KwType) => {
                            match self.parse_type_alias() {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                            }
                        }
                        _ => {
                            self.errors.push("Expected fn, struct, extern, enum, impl, trait, const, static, or type after pub".to_string());
                            self.sync_to_next_top_level();
                        }
                    }
                }
                Some(Token::KwEnum) => {
                    match self.parse_enum_def() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwImpl) => {
                    match self.parse_impl_block() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwTrait) => {
                    match self.parse_trait_def() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwConst) => {
                    match self.parse_const_decl() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwStatic) => {
                    match self.parse_static_decl() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::KwType) => {
                    match self.parse_type_alias() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { self.errors.push(e); self.sync_to_next_top_level(); }
                    }
                }
                Some(Token::Hash) => {
                    // Skip attributes: #[...]
                    self.skip_attribute();
                }
                Some(Token::Error) => {
                    self.errors.push(format!("Unexpected token at position {}", self.pos));
                    self.advance();
                }
                None => break,
                _ => {
                    self.errors.push(format!("Unexpected token {:?} at top level", self.peek()));
                    self.advance();
                }
            }
            self.skip_semicolons();
        }

        ast::Program {
            functions: Vec::new(),
            structs,
            declarations,
        }
    }

    fn sync_to_next_top_level(&mut self) {
        // Skip tokens until we find a top-level keyword or end of file
        while self.pos < self.tokens.len() {
            match self.peek() {
                Some(Token::KwFn) | Some(Token::KwExtern) | Some(Token::KwStruct)
                | Some(Token::KwMod) | Some(Token::KwUse) | Some(Token::KwImport) | Some(Token::KwPub)
                | Some(Token::KwEnum) | Some(Token::KwImpl) | Some(Token::KwTrait)
                | Some(Token::KwConst) | Some(Token::KwStatic) | Some(Token::KwType) => break,
                _ => { self.advance(); }
            }
        }
    }

    #[allow(dead_code)]
    fn skip_aspirational_item(&mut self) {
        let mut brace_depth = 0i32;
        while self.pos < self.tokens.len() {
            match self.peek() {
                Some(Token::LBrace) => { brace_depth += 1; self.advance(); }
                Some(Token::RBrace) => {
                    if brace_depth == 0 { break; }
                    brace_depth -= 1;
                    self.advance();
                }
                Some(Token::Semicolon) if brace_depth == 0 => { self.advance(); break; }
                _ => { self.advance(); }
            }
        }
    }

    fn skip_attribute(&mut self) {
        // Skip #[ ... ]
        self.advance(); // skip Hash
        if self.peek() == Some(&Token::LBracket) {
            self.advance(); // skip LBracket
            let mut depth = 1i32;
            while self.pos < self.tokens.len() && depth > 0 {
                match self.peek() {
                    Some(Token::LBracket) => { depth += 1; }
                    Some(Token::RBracket) => { depth -= 1; }
                    _ => {}
                }
                self.advance();
            }
        }
    }

    // ---- Declarations ----

    fn parse_fn_decl(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwFn)?;
        let name = self.parse_identifier()?;
        let params = self.parse_param_list()?;
        let return_type = self.parse_return_type()?;
        let body = self.parse_block()?;
        Ok(Declaration::Function {
            name,
            params,
            return_type,
            body,
            where_bounds: vec![],
        })
    }

    fn parse_extern_decl(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwExtern)?;
        // Parse optional calling convention: extern "C" fn, extern "stdcall" fn, etc.
        let calling_convention = match self.peek() {
            Some(Token::StringLiteral(_)) => {
                let s = if let Some((Token::StringLiteral(s), _)) = self.advance() {
                    s
                } else {
                    "system".to_string()
                };
                // Validate known calling conventions
                match s.as_str() {
                    "C" | "stdcall" | "fastcall" | "system" | "cdecl" | "vectorcall" => s,
                    other => {
                        return Err(format!(
                            "Unknown calling convention \"{}\". Expected one of: \"C\", \"stdcall\", \"fastcall\", \"system\", \"cdecl\", \"vectorcall\"",
                            other
                        ));
                    }
                }
            }
            _ => "system".to_string(), // default calling convention
        };
        self.expect(Token::KwFn)?;
        let name = self.parse_identifier()?;
        let params = self.parse_param_list()?;
        let return_type = self.parse_return_type()?;
        self.expect(Token::Semicolon)?;
        Ok(Declaration::ExternFunction {
            name,
            params,
            return_type,
            calling_convention,
        })
    }

    fn parse_struct_def(&mut self) -> Result<StructDefinition, String> {
        self.expect(Token::KwStruct)?;
        let name = self.parse_identifier()?;
        self.expect(Token::LBrace)?;
        let mut fields: Vec<(String, Type)> = Vec::new();
        loop {
            if self.peek() == Some(&Token::RBrace) {
                self.advance();
                break;
            }
            let field_name = self.parse_identifier()?;
            self.expect(Token::Colon)?;
            let field_type = self.parse_type()?;
            fields.push((field_name, field_type));
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            }
            if self.peek() == Some(&Token::RBrace) {
                continue;
            }
        }
        Ok(StructDefinition {
            name,
            fields,
            generics: vec![],
        })
    }

    fn parse_enum_def(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwEnum)?;
        let name = self.parse_identifier()?;
        self.expect(Token::LBrace)?;
        let mut variants: Vec<EnumVariant> = Vec::new();
        loop {
            if self.peek() == Some(&Token::RBrace) {
                self.advance();
                break;
            }
            let variant_name = self.parse_identifier()?;
            let data = if self.peek() == Some(&Token::LParen) {
                // Tuple variant: Variant(Type) or Variant(Type1, Type2, ...)
                self.advance(); // skip LParen
                let mut types = Vec::new();
                if self.peek() != Some(&Token::RParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Token::RParen)?;
                if types.len() == 1 {
                    EnumVariantData::Tuple(types)
                } else {
                    EnumVariantData::Tuple(types)
                }
            } else if self.peek() == Some(&Token::LBrace) {
                // Struct variant: Variant { field: Type, ... }
                self.advance(); // skip LBrace
                let mut fields = Vec::new();
                loop {
                    if self.peek() == Some(&Token::RBrace) {
                        self.advance();
                        break;
                    }
                    let field_name = self.parse_identifier()?;
                    self.expect(Token::Colon)?;
                    let field_type = self.parse_type()?;
                    fields.push((field_name, field_type));
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                }
                EnumVariantData::Struct(fields)
            } else {
                EnumVariantData::Unit
            };
            variants.push(EnumVariant { name: variant_name, data });
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            }
        }
        Ok(Declaration::EnumDefinition(EnumDefinition { name, variants }))
    }

    fn parse_impl_block(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwImpl)?;
        // Parse the first identifier (trait name or type name)
        let first_name = self.parse_identifier()?;
        // Check if this is `impl Trait for Type` or `impl Type`
        let (trait_name, self_type) = if self.peek() == Some(&Token::KwFor) {
            self.advance(); // skip 'for'
            let type_name = self.parse_identifier()?;
            (Some(first_name), type_name)
        } else {
            (None, first_name)
        };
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        loop {
            if self.peek() == Some(&Token::RBrace) {
                self.advance();
                break;
            }
            // Parse a method: fn name(params) -> Type { body } or fn name(params) -> Type;
            methods.push(self.parse_trait_method_body()?);
        }
        Ok(Declaration::ImplBlock(ImplBlock { trait_name, self_type, methods }))
    }

    fn parse_trait_def(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwTrait)?;
        let name = self.parse_identifier()?;
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        loop {
            if self.peek() == Some(&Token::RBrace) {
                self.advance();
                break;
            }
            // Parse a method: fn name(params) -> Type; or fn name(params) -> Type { body }
            let method = self.parse_trait_method_body()?;
            methods.push(TraitMethod {
                name: method.name,
                params: method.params,
                return_type: method.return_type,
                body: Some(method.body),
            });
        }
        Ok(Declaration::TraitDefinition(TraitDefinition { name, methods }))
    }

    fn parse_trait_method_body(&mut self) -> Result<Function, String> {
        self.expect(Token::KwFn)?;
        let name = self.parse_identifier()?;
        let params = self.parse_param_list()?;
        let return_type = self.parse_return_type()?;
        // Check if the method has a body or is just a signature (ends with ;)
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
            // No body, return a Function with empty block
            return Ok(Function {
                name,
                params,
                return_type,
                body: Block { statements: vec![] },
                generics: vec![],
            });
        }
        let body = self.parse_block()?;
        Ok(Function {
            name,
            params,
            return_type,
            body,
            generics: vec![],
        })
    }

    fn parse_const_decl(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwConst)?;
        let name = self.parse_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        Ok(Declaration::ConstDecl(ConstDeclaration { name, ty, value }))
    }

    fn parse_static_decl(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwStatic)?;
        let name = self.parse_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        Ok(Declaration::StaticDecl(StaticDeclaration { name, ty, value }))
    }

    fn parse_type_alias(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwType)?;
        let name = self.parse_identifier()?;
        self.expect(Token::Assign)?;
        let ty = self.parse_type()?;
        self.expect(Token::Semicolon)?;
        Ok(Declaration::TypeAlias(TypeAlias { name, ty }))
    }

    fn parse_mod_decl(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwMod)?;
        let name = self.parse_identifier()?;
        self.expect(Token::Semicolon)?;
        Ok(Declaration::ModuleDecl {
            name,
            body: vec![],
        })
    }

    fn parse_use_decl(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwUse)?;
        let mut path: Vec<String> = Vec::new();
        loop {
            path.push(self.parse_identifier()?);
            if self.peek() == Some(&Token::ColonColon) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Token::Semicolon)?;
        Ok(Declaration::UseDecl { path })
    }

    fn parse_import_decl(&mut self) -> Result<Declaration, String> {
        self.expect(Token::KwImport)?;
        let mut path: Vec<String> = Vec::new();
        loop {
            path.push(self.parse_identifier()?);
            if self.peek() == Some(&Token::Dot) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Token::Semicolon)?;
        Ok(Declaration::ImportDecl { path })
    }

    // ---- Types ----

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek() {
            Some(Token::TypeInt) => { self.advance(); Ok(Type::Int) }
            Some(Token::TypeBool) => { self.advance(); Ok(Type::Bool) }
            Some(Token::TypeString) => { self.advance(); Ok(Type::String) }
            Some(Token::TypeVoid) => { self.advance(); Ok(Type::Void) }
            Some(Token::Star) => {
                self.advance();
                let inner = self.parse_type()?;
                Ok(Type::Pointer(Box::new(inner)))
            }
            Some(Token::Ampersand) => {
                self.advance();
                let inner = self.parse_type()?;
                Ok(Type::Pointer(Box::new(inner)))
            }
            Some(Token::LBracket) => {
                self.advance();
                let elem = self.parse_type()?;
                self.expect(Token::Semicolon)?;
                let len = self.parse_int_literal()? as usize;
                self.expect(Token::RBracket)?;
                Ok(Type::Array(Box::new(elem), len))
            }
            Some(Token::Identifier(_)) => {
                let mut name = self.parse_identifier()?;
                while self.peek() == Some(&Token::ColonColon) {
                    self.advance();
                    name.push_str("::");
                    name.push_str(&self.parse_identifier()?);
                }
                Ok(Type::Struct(name))
            }
            Some(Token::LParen) => {
                // Function type: (T1, T2) -> T3
                self.advance();
                let mut params = Vec::new();
                loop {
                    if self.peek() == Some(&Token::RParen) {
                        self.advance();
                        break;
                    }
                    params.push(self.parse_type()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                }
                if self.peek() == Some(&Token::Arrow) {
                    self.advance();
                    let ret = self.parse_type()?;
                    Ok(Type::Closure(params, Box::new(ret)))
                } else {
                    // Just a parenthesized type, treat as the first param
                    Ok(params.into_iter().next().unwrap_or(Type::Void))
                }
            }
            _ => Err(format!("Expected type, found {:?}", self.peek())),
        }
    }

    fn parse_return_type(&mut self) -> Result<Type, String> {
        if self.peek() == Some(&Token::Arrow) {
            self.advance(); // skip Arrow
            return self.parse_type();
        }
        if self.peek() == Some(&Token::Colon) {
            self.advance();
            return self.parse_type();
        }
        // Default return type
        Ok(Type::Void)
    }

    // ---- Parameters ----

    fn parse_param_list(&mut self) -> Result<Vec<Parameter>, String> {
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        if self.peek() == Some(&Token::RParen) {
            self.advance();
            return Ok(params);
        }
        loop {
            // Check for variadic ...
            if self.peek() == Some(&Token::Ellipsis) {
                self.advance();
                break;
            }
            let name = self.parse_identifier()?;
            self.expect(Token::Colon)?;
            let param_type = self.parse_type()?;
            params.push(Parameter { name, param_type });
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Token::RParen)?;
        Ok(params)
    }

    // ---- Block and Statements ----

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        self.skip_semicolons();
        loop {
            if self.peek() == Some(&Token::RBrace) {
                self.advance();
                break;
            }
            if self.pos >= self.tokens.len() {
                return Err("Unterminated block".to_string());
            }
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.sync_to_semicolon_or_brace();
                }
            }
            self.skip_semicolons();
        }
        Ok(Block { statements })
    }

    fn sync_to_semicolon_or_brace(&mut self) {
        while self.pos < self.tokens.len() {
            match self.peek() {
                Some(Token::Semicolon) | Some(Token::RBrace) | Some(Token::LBrace) => break,
                _ => { self.advance(); }
            }
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek() {
            Some(Token::KwLet) => self.parse_let_stmt(),
            Some(Token::KwReturn) => self.parse_return_stmt(),
            Some(Token::KwIf) => self.parse_if_stmt(),
            Some(Token::KwWhile) => self.parse_while_stmt(),
            Some(Token::KwFor) => self.parse_for_stmt(),
            _ => {
                // Try assignment or expression statement
                let expr = self.parse_expression()?;
                if self.peek() == Some(&Token::Assign) {
                    self.advance();
                    let value = self.parse_expression()?;
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Assignment { target: expr, value })
                } else {
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Expression(expr))
                }
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Statement, String> {
        self.expect(Token::KwLet)?;
        let is_mut = self.peek().map(|t| {
            if let Token::Identifier(s) = t { s == "mut" } else { false }
        }).unwrap_or(false);
        if is_mut {
            self.advance();
        }
        let name = self.parse_identifier()?;
        let ty = if self.peek() == Some(&Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        if is_mut {
            Ok(Statement::VariableDeclaration {
                name,
                initializer: value,
                ty,
            })
        } else {
            Ok(Statement::Let {
                name,
                value,
                ty: ty.unwrap_or(Type::Unknown),
            })
        }
    }

    fn parse_return_stmt(&mut self) -> Result<Statement, String> {
        self.expect(Token::KwReturn)?;
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
            return Ok(Statement::Return(None));
        }
        let expr = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        Ok(Statement::Return(Some(expr)))
    }

    fn parse_if_stmt(&mut self) -> Result<Statement, String> {
        self.expect(Token::KwIf)?;
        let cond = self.parse_expression()?;
        let then_block = Box::new(self.parse_block()?);
        let else_block = if self.peek() == Some(&Token::KwElse) {
            self.advance();
            if self.peek() == Some(&Token::KwIf) {
                // else if — wrap in another If statement
                let inner_if = self.parse_if_stmt()?;
                Some(Box::new(Block { statements: vec![inner_if] }))
            } else {
                Some(Box::new(self.parse_block()?))
            }
        } else {
            None
        };
        Ok(Statement::If {
            cond,
            then_block,
            else_block,
        })
    }

    fn parse_for_stmt(&mut self) -> Result<Statement, String> {
        self.expect(Token::KwFor)?;
        let var = self.parse_identifier()?;
        self.expect(Token::KwIn)?;
        let iter = self.parse_expression()?;
        let body = Box::new(self.parse_block()?);
        Ok(Statement::For { var, iter, body })
    }

    fn parse_while_stmt(&mut self) -> Result<Statement, String> {
        self.expect(Token::KwWhile)?;
        let cond = self.parse_expression()?;
        let body = Box::new(self.parse_block()?);
        Ok(Statement::While { cond, body })
    }

    // ---- Expressions (Pratt parser) ----

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_binary_expr(0)
    }

    /// Pratt binding power for binary operators
    fn bp(token: &Token) -> Option<(u8, u8)> {
        match token {
            Token::Or | Token::Pipe => Some((1, 2)),
            Token::And | Token::Ampersand => Some((3, 4)),
            Token::Equals | Token::NotEquals => Some((5, 6)),
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => Some((5, 6)),
            Token::Plus | Token::Minus => Some((7, 8)),
            Token::Star | Token::Slash | Token::Percent => Some((9, 10)),
            _ => None,
        }
    }

    fn token_to_binary_op(token: &Token) -> Option<BinaryOp> {
        match token {
            Token::Plus => Some(BinaryOp::Add),
            Token::Minus => Some(BinaryOp::Sub),
            Token::Star => Some(BinaryOp::Mul),
            Token::Slash => Some(BinaryOp::Div),
            Token::Percent => Some(BinaryOp::Mod),
            Token::Equals => Some(BinaryOp::Eq),
            Token::NotEquals => Some(BinaryOp::Neq),
            Token::Less => Some(BinaryOp::Lt),
            Token::LessEqual => Some(BinaryOp::Le),
            Token::Greater => Some(BinaryOp::Gt),
            Token::GreaterEqual => Some(BinaryOp::Ge),
            Token::And | Token::Ampersand => Some(BinaryOp::And),
            Token::Or | Token::Pipe => Some(BinaryOp::Or),
            _ => None,
        }
    }

    fn parse_binary_expr(&mut self, min_bp: u8) -> Result<Expression, String> {
        let mut lhs = self.parse_unary()?;

        loop {
            let token = match self.peek() {
                Some(t) => t.clone(),
                None => break,
            };

            let (left_bp, right_bp) = match Self::bp(&token) {
                Some(bp) => bp,
                None => break,
            };

            if left_bp < min_bp {
                break;
            }

            let op = Self::token_to_binary_op(&token).unwrap();
            self.advance();
            let rhs = self.parse_binary_expr(right_bp)?;
            lhs = Expression {
                kind: ExpressionKind::BinaryOp {
                    left: Box::new(lhs),
                    op,
                    right: Box::new(rhs),
                },
                ty: None,
            };
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        match self.peek() {
            Some(Token::Bang) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression {
                    kind: ExpressionKind::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr) },
                    ty: None,
                })
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression {
                    kind: ExpressionKind::UnaryOp { op: UnaryOp::Neg, expr: Box::new(expr) },
                    ty: None,
                })
            }
            Some(Token::Ampersand) => {
                self.advance();
                let expr = self.parse_unary()?;
                // AddressOf: skip for now, treat as identity
                Ok(expr)
            }
            Some(Token::Star) => {
                self.advance();
                let expr = self.parse_unary()?;
                // Dereference: skip for now, treat as identity
                Ok(expr)
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                Some(Token::LParen) => {
                    // Function call
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(Token::RParen)?;

                    // Extract function name from the expression
                    let name = match &expr.kind {
                        ExpressionKind::Variable(n) => n.clone(),
                        _ => {
                            self.errors.push("Function call target must be an identifier".to_string());
                            "unknown".to_string()
                        }
                    };
                    expr = Expression {
                        kind: ExpressionKind::FunctionCall { name, args, type_args: vec![] },
                        ty: None,
                    };
                }
                Some(Token::Dot) => {
                    self.advance();
                    let field = self.parse_identifier()?;
                    expr = Expression {
                        kind: ExpressionKind::MemberAccess {
                            base: Box::new(expr),
                            field,
                        },
                        ty: None,
                    };
                }
                Some(Token::LBrace) => {
                    // Struct literal: TypeName { field: value, ... }
                    let name = match &expr.kind {
                        ExpressionKind::Variable(n) => n.clone(),
                        _ => {
                            self.errors.push("Struct literal must start with a type name".to_string());
                            "unknown".to_string()
                        }
                    };
                    self.advance(); // skip LBrace
                    let mut fields = Vec::new();
                    if self.peek() != Some(&Token::RBrace) {
                        loop {
                            let field_name = self.parse_identifier()?;
                            self.expect(Token::Colon)?;
                            let field_value = self.parse_expression()?;
                            fields.push((field_name, field_value));
                            if self.peek() == Some(&Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(Token::RBrace)?;
                    expr = Expression {
                        kind: ExpressionKind::StructLiteral { name, fields },
                        ty: None,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.peek() {
            Some(Token::IntLiteral(n)) => {
                let n = *n;
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::Literal(Literal::Integer(n)),
                    ty: None,
                })
            }
            Some(Token::StringLiteral(_)) => {
                let s = if let Some((Token::StringLiteral(s), _)) = self.advance() {
                    s
                } else {
                    "".to_string()
                };
                Ok(Expression {
                    kind: ExpressionKind::Literal(Literal::String(s)),
                    ty: None,
                })
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    ty: None,
                })
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(false)),
                    ty: None,
                })
            }
            Some(Token::Identifier(_)) => {
                let mut name = self.parse_identifier()?;
                while self.peek() == Some(&Token::ColonColon) {
                    self.advance();
                    name.push_str("::");
                    name.push_str(&self.parse_identifier()?);
                }
                Ok(Expression {
                    kind: ExpressionKind::Variable(name),
                    ty: None,
                })
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::KwMatch) => self.parse_match_expr(),
            Some(Token::LBrace) => self.parse_struct_literal(),
            Some(Token::LBracket) => self.parse_array_literal(),
            Some(Token::Pipe) => self.parse_closure_expr(),
            _ => Err(format!("Expected expression, found {:?}", self.peek())),
        }
    }

    // ---- Parsing methods for new expression types ----

    fn parse_match_expr(&mut self) -> Result<Expression, String> {
        self.expect(Token::KwMatch)?;
        let scrutinee = Box::new(self.parse_expression()?);
        self.expect(Token::LBrace)?;
        let mut arms: Vec<MatchArm> = Vec::new();
        while self.peek() != Some(&Token::RBrace) && self.pos < self.tokens.len() {
            let pattern = self.parse_match_pattern()?;
            let guard = if self.peek() == Some(&Token::KwIf) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            self.expect(Token::FatArrow)?;
            let body = self.parse_expression()?;
            arms.push(MatchArm { pattern, guard, body });
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Expression {
            kind: ExpressionKind::Match { scrutinee, arms },
            ty: None,
        })
    }

    fn parse_match_pattern(&mut self) -> Result<MatchPattern, String> {
        match self.peek() {
            Some(Token::IntLiteral(n)) => {
                let n = *n;
                self.advance();
                Ok(MatchPattern::int_literal(n))
            }
            Some(Token::True) => {
                self.advance();
                Ok(MatchPattern::bool_literal(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(MatchPattern::bool_literal(false))
            }
            Some(Token::StringLiteral(_)) => {
                let s = if let Some((Token::StringLiteral(s), _)) = self.advance() {
                    s
                } else {
                    "".to_string()
                };
                Ok(MatchPattern::string_literal(s))
            }
            Some(Token::Identifier(_)) => {
                let name = self.parse_identifier()?;
                if name == "_" {
                    Ok(MatchPattern::wildcard())
                } else {
                    Ok(MatchPattern::variable(name))
                }
            }
            _ => Err(format!("Expected match pattern, found {:?}", self.peek())),
        }
    }

    fn parse_struct_literal(&mut self) -> Result<Expression, String> {
        // Struct literal: Name { field: expr, ... }
        // But we need to check if this is a struct literal or just a block.
        // A struct literal starts with an identifier then LBrace.
        // Since we're in parse_primary already at LBrace, we need to look ahead.
        // Actually, struct literal in Fusion syntax is just { field: value, ... }
        // We need to figure out the struct name from context.
        // For now, parse as anonymous struct literal.
        self.expect(Token::LBrace)?;
        let mut fields: Vec<(String, Expression)> = Vec::new();
        if self.peek() == Some(&Token::RBrace) {
            self.advance();
            return Ok(Expression {
                kind: ExpressionKind::StructLiteral { name: String::new(), fields },
                ty: None,
            });
        }
        loop {
            let field_name = self.parse_identifier()?;
            self.expect(Token::Colon)?;
            let value = self.parse_expression()?;
            fields.push((field_name, value));
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Expression {
            kind: ExpressionKind::StructLiteral { name: String::new(), fields },
            ty: None,
        })
    }

    fn parse_array_literal(&mut self) -> Result<Expression, String> {
        self.expect(Token::LBracket)?;
        let mut elements: Vec<Expression> = Vec::new();
        if self.peek() == Some(&Token::RBracket) {
            self.advance();
            return Ok(Expression {
                kind: ExpressionKind::ArrayLiteral(elements),
                ty: None,
            });
        }
        loop {
            elements.push(self.parse_expression()?);
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Token::RBracket)?;
        Ok(Expression {
            kind: ExpressionKind::ArrayLiteral(elements),
            ty: None,
        })
    }

    fn parse_closure_expr(&mut self) -> Result<Expression, String> {
        // Closure: |param1: Type, param2: Type| body
        self.expect(Token::Pipe)?;
        let mut params: Vec<Parameter> = Vec::new();
        if self.peek() != Some(&Token::Pipe) {
            loop {
                let name = self.parse_identifier()?;
                let param_type = if self.peek() == Some(&Token::Colon) {
                    self.advance();
                    self.parse_type()?
                } else {
                    Type::Unknown
                };
                params.push(Parameter { name, param_type });
                if self.peek() == Some(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::Pipe)?;
        let body = Box::new(self.parse_expression()?);
        Ok(Expression {
            kind: ExpressionKind::Closure { params, body },
            ty: None,
        })
    }

    // ---- Helpers ----

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.advance() {
            Some((Token::Identifier(name), _)) => Ok(name),
            Some((t, _)) => Err(format!("Expected identifier, found {:?}", t)),
            None => Err("Expected identifier, found end of file".to_string()),
        }
    }

    fn parse_int_literal(&mut self) -> Result<i64, String> {
        match self.advance() {
            Some((Token::IntLiteral(n), _)) => Ok(n),
            Some((t, _)) => Err(format!("Expected integer, found {:?}", t)),
            None => Err("Expected integer, found end of file".to_string()),
        }
    }
}

// ---- Public API ----

/// Parses source code into a Program AST.
pub fn parse_output(source: &str) -> ParserOutput {
    let token_stream = lexer::lex(source);
    let mut parser = Parser::new(token_stream.tokens);
    let program = parser.parse_program();
    let errors = parser.errors.into_iter().collect();
    ParserOutput {
        program: Some(program),
        errors,
    }
}

/// Parses a source file into a Program AST.
pub fn parse_file(path: &str) -> ParserOutput {
    match std::fs::read_to_string(path) {
        Ok(source) => parse_output(&source),
        Err(e) => ParserOutput {
            program: None,
            errors: vec![format!("Failed to read file '{}': {}", path, e)],
        },
    }
}

/// Parses source and returns a status code (0 = success, non-zero = errors).
pub fn parse_status(source: &str) -> FI64 {
    let output = parse_output(source);
    if output.errors.is_empty() && output.program.is_some() {
        0
    } else {
        output.errors.len() as FI64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_fn() {
        let src = "fn add(x: int, y: int) -> int { return x + y; }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert!(!prog.declarations.is_empty());
        match &prog.declarations[0] {
            Declaration::Function { name, params, return_type, body, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "x");
                assert_eq!(params[1].name, "y");
                assert_eq!(*return_type, Type::Int);
                assert_eq!(body.statements.len(), 1);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_extern_fn() {
        let src = "extern fn printf(fmt: string, ...) -> int;";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.declarations.len(), 1);
    }

    #[test]
    fn test_parse_struct() {
        let src = "struct Point { x: int, y: int }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.structs.len(), 1);
        assert_eq!(prog.structs[0].name, "Point");
        assert_eq!(prog.structs[0].fields.len(), 2);
    }

    #[test]
    fn test_parse_if_else() {
        let src = "fn max(x: int, y: int) -> int { if x > y { return x; } else { return y; } }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.declarations.len(), 1);
    }

    #[test]
    fn test_parse_while_loop() {
        let src = "fn countdown(n: int) -> int { let mut x: int = n; while x > 0 { x = x - 1; } return 0; }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.declarations.len(), 1);
    }

    #[test]
    fn test_parse_let_binding() {
        let src = "fn main() -> int { let x: int = 42; return x; }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.declarations.len(), 1);
    }

    #[test]
    fn test_parse_unary_ops() {
        let src = "fn negate(x: int) -> int { return -x; } fn is_zero(x: int) -> int { return !x; }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.declarations.len(), 2);
    }

    #[test]
    fn test_parse_member_access() {
        let src = "fn get_x(p: Point) -> int { return p.x; }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.declarations.len(), 1);
    }

    #[test]
    fn test_parse_function_call() {
        let src = "fn main() -> int { let x: int = square(5); return x; }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.declarations.len(), 1);
    }

    #[test]
    fn test_parse_string_literal() {
        let src = "fn hello() -> string { return \"Hello, World!\"; }";
        let output = parse_output(src);
        assert!(output.errors.is_empty(), "Parse errors: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert_eq!(prog.declarations.len(), 1);
    }

    #[test]
    fn test_parse_real_fu_file() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.fu");
        let output = parse_file(path);
        assert!(output.program.is_some(), "Failed to parse: {:?}", output.errors);
        let prog = output.program.unwrap();
        assert!(!prog.declarations.is_empty(), "Should have declarations");
    }

    #[test]
    fn test_parse_status() {
        let status = parse_status("fn main() -> int { return 0; }");
        assert_eq!(status, 0);
    }
}