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
    ExternFunction { name: String, params: Vec<Parameter>, return_type: Type, calling_convention: String },
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    ImplBlock(ImplBlock),
    TraitDefinition(TraitDefinition),
    ConstDecl(ConstDeclaration),
    StaticDecl(StaticDeclaration),
    TypeAlias(TypeAlias),
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

#[derive(Debug, Clone)]
pub enum EnumVariantData {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<(String, Type)>),
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub data: EnumVariantData,
}

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub trait_name: Option<String>,
    pub self_type: String,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub name: String,
    pub methods: Vec<TraitMethod>,
}

#[derive(Debug, Clone)]
pub struct ConstDeclaration {
    pub name: String,
    pub ty: Type,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct StaticDeclaration {
    pub name: String,
    pub ty: Type,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub ty: Type,
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