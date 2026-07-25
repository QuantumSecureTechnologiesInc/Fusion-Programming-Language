# Fusion Bootstrap Compiler Codebase

## Overview

The Fusion bootstrap compiler (`fuc`) is a Rust-based compiler that compiles Fusion source code (`.fu` files) into native executables, object files, LLVM IR, or WebAssembly modules. This document contains the complete source code for the bootstrap compiler.

### Architecture

The compiler follows a traditional multi-phase architecture:

1. **Lexer** - Tokenizes source code into tokens with spans
2. **Parser** - Builds Abstract Syntax Tree (AST) from tokens
3. **Semantic Analysis** - Type checking and validation
4. **IR Lowering** - Converts typed AST to Intermediate Representation
5. **Optimization** - Applies optimization passes to IR
6. **Code Generation** - Emits LLVM IR, native code, or WASM

### Module Structure

```
crates/fuc/src/
├── main.fu          # Entry point
├── lib.fu           # Library root (module declarations)
├── cli.fu           # CLI argument parsing
├── lexer.fu         # Lexical analysis
├── ast.fu           # AST definitions
├── parser.fu        # Parsing (Chumsky parser combinators)
├── sema.fu          # Semantic analysis & type checking
├── ir.fu            # Intermediate Representation
├── optimizer.fu     # IR optimization passes
├── codegen.fu       # Codegen backend trait
├── codegen/
│   └── llvm.fu      # LLVM backend implementation
└── wasm/            # WebAssembly backend
    ├── mod.fu
    ├── backend.fu
    ├── codegen.fu
    ├── types.fu
    └── wasm_encoder.fu
```

---

## 1. Entry Point (main.fu)

```fusion
//! 7 mods + sema_shape_check

mod stage1_parser_api;
mod stage1_sema_api;
mod ast;
mod lexer;
mod parser;
mod sema;
mod dummy;

extern fn printf(fmt: string, ...) -> int;

pub fn main() -> int {
    printf("[fuc-native] before sema_shape_check\n");
    let result = stage1_parser_api::sema_shape_check("test.fu");
    printf("[fuc-native] sema_shape_check: %d\n", result);
    return result;
}
```

---

## 2. Library Root (lib.fu)

```fusion
//! Fusion compiler library.
/// AST definitions.
mod ast;
/// CLI helpers.
mod cli;
/// Code generation backends.
mod codegen;
/// IR lowering.
mod ir;
/// Lexer.
mod lexer;
/// Optimizer passes.
mod optimizer;
/// Parser.
mod parser;
/// Semantic analysis.
mod sema;
/// WASM code generation backend.
mod wasm;
/// Re-export backend trait.
pub use crate::codegen::Backend;
```

---

## 3. CLI Arguments (cli.fu)

```fusion
/// CLI arguments for the Fusion compiler.
#[derive(Clone, Debug)]
pub struct CompilerArgs {
    /// Input .fu file.
    pub input: FString,
    /// Output path.
    pub output: FString,
    /// Optimization level (0-3).
    pub opt_level: int,
    /// Target triple override.
    pub target: Option<FString>,
    /// Emit textual LLVM IR.
    pub emit_llvm: bool,
    /// Parse only (no semantic analysis or codegen).
    pub parse_only: bool,
    /// Run semantic analysis only (skip codegen).
    pub sema_only: bool,
    /// Emit a linked executable instead of an object file.
    pub emit_bin: bool,
    /// Compile as library (suppress entry point checks).
    pub lib: bool,
    /// Emit DWARF debug info.
    pub debug_info: bool,
    /// External libraries to link against (e.g., ["hypercycle_pqc"])
    pub link_libs: FVec<FString>,
    /// Library search paths (e.g., ["v2.0 Vortex/build/Release"])
    pub lib_paths: FVec<FString>,
}

/// Parse CLI arguments from command line.
pub fn parse_args() -> CompilerArgs {
    // Simple positional and flag-based argument parsing.
    // Usage: fuc [flags] <input.fu> [-o <output>]
    let args = std::env::args();
    let mut input = FString::from("");
    let mut output = FString::from("a.out");
    let mut opt_level: int = 0;
    let mut target: Option<FString> = None;
    let mut emit_llvm = false;
    let mut parse_only = false;
    let mut sema_only = false;
    let mut emit_bin = false;
    let mut lib = false;
    let mut debug_info = true;
    let mut link_libs: FVec<FString> = FVec::new();
    let mut lib_paths: FVec<FString> = FVec::new();
    let mut i: FSize = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-o" {
            i = i + 1;
            if i < args.len() { output = args[i].clone(); }
        } else if arg == "--opt-level" {
            i = i + 1;
            if i < args.len() { opt_level = std::str::parse::<u8>(&args[i]); }
        } else if arg == "--target" {
            i = i + 1;
            if i < args.len() { target = Some(args[i].clone()); }
        } else if arg == "--emit-llvm" {
            emit_llvm = true;
        } else if arg == "--parse-only" {
            parse_only = true;
        } else if arg == "--sema-only" {
            sema_only = true;
        } else if arg == "--emit-bin" {
            emit_bin = true;
        } else if arg == "--lib" {
            lib = true;
        } else if arg == "--no-debug" {
            debug_info = false;
        } else if arg == "--link-lib" {
            i = i + 1;
            if i < args.len() { link_libs.push(args[i].clone()); }
        } else if arg == "--lib-path" {
            i = i + 1;
            if i < args.len() { lib_paths.push(args[i].clone()); }
        } else if !arg.starts_with("-") && input.is_empty() {
            input = arg.clone();
        }
        i = i + 1;
    }
    CompilerArgs { input, output, opt_level, target, emit_llvm, parse_only, sema_only, emit_bin, lib, debug_info, link_libs, lib_paths }
}
```

---

## 4. Lexer (lexer.fu)

```fusion
//! Lexer for the Fusion compiler.
use std::ops::Range;
/// Token definitions for the language.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    KwFn,
    KwLet,
    KwReturn,
    KwIf,
    KwElse,
    KwWhile,
    KwFor,
    KwIn,
    KwMatch,
    KwImpl,
    KwTrait,
    KwWhere,
    KwConst,
    KwStatic,
    KwUse,
    KwMod,
    KwPub,
    KwAsync,
    KwAwait,
    KwStruct,
    KwEnum,
    KwType,
    KwExtern,
    Identifier(FString),
    IntLiteral(FI64),
    StringLiteral(FString),
    Colon,
    ColonColon,
    Semicolon,
    Assign,
    Equals,
    NotEquals,
    Bang,
    Or,
    Pipe,
    And,
    FatArrow,
    Arrow,
    Range,
    Less,
    Greater,
    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Ellipsis,
    Dot,
    Question,
    Hash,
    TypeInt,
    TypeBool,
    TypeString,
    TypeVoid,
    True,
    False,
    Error,
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}
fn unescape_string(input: &str) -> FString {
    let mut out = String::new();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    return out;
}
/// Token paired with a span.
type LexerSpan = Range<FSize>;
type SpannedToken = (Token, LexerSpan);
fn is_ident_start(ch: int) -> FBool {
    return (ch >= b'a' && ch <= b'z') || (ch >= b'A' && ch <= b'Z') || ch == b'_';
}
fn is_ident_continue(ch: int) -> FBool {
    return is_ident_start(ch) || (ch >= b'0' && ch <= b'9');
}
fn keyword_or_ident(text: FString) -> Token {
    if text == "fn" {
        return Token::KwFn;
    }
    if text == "let" {
        return Token::KwLet;
    }
    if text == "return" {
        return Token::KwReturn;
    }
    if text == "if" {
        return Token::KwIf;
    }
    if text == "else" {
        return Token::KwElse;
    }
    if text == "while" {
        return Token::KwWhile;
    }
    if text == "for" {
        return Token::KwFor;
    }
    if text == "in" {
        return Token::KwIn;
    }
    if text == "match" {
        return Token::KwMatch;
    }
    if text == "impl" {
        return Token::KwImpl;
    }
    if text == "trait" {
        return Token::KwTrait;
    }
    if text == "where" {
        return Token::KwWhere;
    }
    if text == "const" {
        return Token::KwConst;
    }
    if text == "static" {
        return Token::KwStatic;
    }
    if text == "use" {
        return Token::KwUse;
    }
    if text == "mod" {
        return Token::KwMod;
    }
    if text == "pub" {
        return Token::KwPub;
    }
    if text == "async" {
        return Token::KwAsync;
    }
    if text == "await" {
        return Token::KwAwait;
    }
    if text == "struct" {
        return Token::KwStruct;
    }
    if text == "enum" {
        return Token::KwEnum;
    }
    if text == "type" {
        return Token::KwType;
    }
    if text == "extern" {
        return Token::KwExtern;
    }
    if text == "int" {
        return Token::TypeInt;
    }
    if text == "bool" {
        return Token::TypeBool;
    }
    if text == "string" {
        return Token::TypeString;
    }
    if text == "void" {
        return Token::TypeVoid;
    }
    if text == "true" {
        return Token::True;
    }
    if text == "false" {
        return Token::False;
    }
    return Token::Identifier(text);
}
/// Lexes the given input into tokens with spans.
pub fn lex(input: &str) -> FVec<SpannedToken> {
    let bytes = input.as_bytes();
    let mut tokens: FVec<SpannedToken> = Vec::new();
    let mut i = 0usize;
    let len = bytes.len();
    if len >= 2 && bytes[0] == b'#' && bytes[1] == b'!' {
        while i < len && bytes[i] != b'\n' {
            i = i + 1;
        }
    }
    while i < len {
        let ch = bytes[i];
        if ch == b' ' || ch == b'\t' || ch == b'\r' || ch == b'\n' || ch == 0x0c {
            i = i + 1;
            continue;
        }
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            i = i + 2;
            while i < len && bytes[i] != b'\n' {
                i = i + 1;
            }
            continue;
        }
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            i = i + 2;
            while i + 1 < len {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i = i + 2;
                    break;
                }
                i = i + 1;
            }
            continue;
        }
        let start = i;
        if ch == b'\'' {
            if i + 1 < len && is_ident_start(bytes[i + 1]) {
                let mut j = i + 2;
                while j < len && is_ident_continue(bytes[j]) {
                    j = j + 1;
                }
                if j >= len || bytes[j] != b'\'' {
                    let name = format!("'{}", input[i + 1..j].to_string());
                    tokens.push((Token::Identifier(name), start..j));
                    i = j;
                    continue;
                }
            }
            i = i + 1;
            let mut value: FI64 = 0;
            let mut valid = false;
            if i < len {
                if bytes[i] == b'\\' {
                    if i + 1 < len {
                        let esc = bytes[i + 1];
                        value = if esc == b'n' {
                            10
                        } else if esc == b't' {
                            9
                        } else if esc == b'r' {
                            13
                        } else if esc == b'\\' {
                            92
                        } else if esc == b'\'' {
                            39
                        } else {
                            esc as FI64
                        };
                        i = i + 2;
                        valid = true;
                    }
                } else {
                    value = bytes[i] as FI64;
                    i = i + 1;
                    valid = true;
                }
            }
            if valid && i < len && bytes[i] == b'\'' {
                i = i + 1;
                tokens.push((Token::IntLiteral(value), start..i));
            } else {
                tokens.push((Token::Error, start..i));
            }
            continue;
        }
        if is_ident_start(ch) {
            i = i + 1;
            while i < len && is_ident_continue(bytes[i]) {
                i = i + 1;
            }
            let text = input[start..i].to_string();
            tokens.push((keyword_or_ident(text), start..i));
            continue;
        }
        if ch >= b'0' && ch <= b'9' {
            i = i + 1;
            while i < len && bytes[i] >= b'0' && bytes[i] <= b'9' {
                i = i + 1;
            }
            let text = input[start..i].to_string();
            let parsed = text.parse::<i64>();
            if parsed.is_ok() {
                tokens.push((Token::IntLiteral(parsed.unwrap()), start..i));
            } else {
                tokens.push((Token::Error, start..i));
            }
            continue;
        }
        if ch == 34 {
            i = i + 1;
            while i < len {
                if bytes[i] == b'\\' {
                    if i + 1 < len {
                        i = i + 2;
                    } else {
                        i = i + 1;
                    }
                    continue;
                }
                if bytes[i] == 34 {
                    i = i + 1;
                    break;
                }
                i = i + 1;
            }
            if i >= start + 2 {
                let literal = &input[start + 1..i - 1];
                tokens.push((Token::StringLiteral(unescape_string(literal)), start..i));
            } else {
                tokens.push((Token::Error, start..i));
            }
            continue;
        }
        if ch == b'r' && i + 2 < len && bytes[i + 1] == b'#' && bytes[i + 2] == 34 {
            i = i + 3;
            while i + 1 < len {
                if bytes[i] == 34 && bytes[i + 1] == b'#' {
                    let end = i + 2;
                    let literal = &input[start + 3..i];
                    tokens.push((Token::StringLiteral(unescape_string(literal)), start..end));
                    i = end;
                    break;
                }
                i = i + 1;
            }
            continue;
        }
        if i + 2 < len && bytes[i] == b'.' && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' {
            tokens.push((Token::Ellipsis, i..i + 3));
            i = i + 3;
            continue;
        }
        if i + 1 < len && bytes[i] == b'.' && bytes[i + 1] == b'.' {
            tokens.push((Token::Range, i..i + 2));
            i = i + 2;
            continue;
        }
        if i + 1 < len && bytes[i] == b':' && bytes[i + 1] == b':' {
            tokens.push((Token::ColonColon, i..i + 2));
            i = i + 2;
            continue;
        }
        if i + 1 < len && bytes[i] == b'-' && bytes[i + 1] == b'>' {
            tokens.push((Token::Arrow, i..i + 2));
            i = i + 2;
            continue;
        }
        if i + 1 < len && bytes[i] == b'=' && bytes[i + 1] == b'=' {
            tokens.push((Token::Equals, i..i + 2));
            i = i + 2;
            continue;
        }
        if i + 1 < len && bytes[i] == b'=' && bytes[i + 1] == b'>' {
            tokens.push((Token::FatArrow, i..i + 2));
            i = i + 2;
            continue;
        }
        if i + 1 < len && bytes[i] == b'!' && bytes[i + 1] == b'=' {
            tokens.push((Token::NotEquals, i..i + 2));
            i = i + 2;
            continue;
        }
        if i + 1 < len && bytes[i] == b'|' && bytes[i + 1] == b'|' {
            tokens.push((Token::Or, i..i + 2));
            i = i + 2;
            continue;
        }
        if i + 1 < len && bytes[i] == b'&' && bytes[i + 1] == b'&' {
            tokens.push((Token::And, i..i + 2));
            i = i + 2;
            continue;
        }
        if ch == b'|' {
            tokens.push((Token::Pipe, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b':' {
            tokens.push((Token::Colon, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b';' {
            tokens.push((Token::Semicolon, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'=' {
            tokens.push((Token::Assign, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'!' {
            tokens.push((Token::Bang, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'<' {
            tokens.push((Token::Less, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'>' {
            tokens.push((Token::Greater, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'+' {
            tokens.push((Token::Plus, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'-' {
            tokens.push((Token::Minus, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'*' {
            tokens.push((Token::Star, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'/' {
            tokens.push((Token::Slash, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'&' {
            tokens.push((Token::Ampersand, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'(' {
            tokens.push((Token::LParen, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b')' {
            tokens.push((Token::RParen, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'{' {
            tokens.push((Token::LBrace, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'}' {
            tokens.push((Token::RBrace, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'[' {
            tokens.push((Token::LBracket, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b']' {
            tokens.push((Token::RBracket, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b',' {
            tokens.push((Token::Comma, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'.' {
            tokens.push((Token::Dot, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'?' {
            tokens.push((Token::Question, i..i + 1));
            i = i + 1;
            continue;
        }
        if ch == b'#' {
            tokens.push((Token::Hash, i..i + 1));
            i = i + 1;
            continue;
        }
        tokens.push((Token::Error, i..i + 1));
        i = i + 1;
    }
    return tokens;
}
```

---

## 5. AST Definitions (ast.fu)

```fusion
//! AST definitions for the Fusion compiler.
/// Byte span for source ranges.
pub type Span = Range<FSize>;
/// AST node with source span.
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    /// Node payload.
    pub node: T,
    /// Byte span in the source.
    pub span: Span,
}
/// Language types.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    /// Signed 32-bit integer.
    Int,
    /// Boolean.
    Bool,
    /// UTF-8 string.
    String,
    /// Unit type.
    Void,
    /// Unknown type (used for forward/error-tolerant compilation).
    Unknown,
    /// Pointer to another type.
    Pointer(Box<Type>),
    /// Fixed-size array of an element type.
    Array(Box<Type>, FSize),
    /// Struct type, identified by name.
    Struct(FString),
    /// Generic type parameter (e.g., T in fn foo<T>(x: T)).
    GenericParam(FString),
    /// Slice type (view into array).
    Slice(Box<Type>),
    /// Closure type with captured environment.
    Closure(FVec<Type>, Box<Type>),
}
/// Item visibility.
#[derive(Clone, Debug, PartialEq)]
pub enum Visibility {
    /// Private (default).
    Private,
    /// Public (accessible from other modules).
    Public,
}
/// Top-level program container.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    /// Defined functions.
    pub functions: FVec<Function>,
    /// External declarations.
    pub externs: FVec<ExternFunction>,
    /// Struct type definitions.
    pub structs: FVec<StructDefinition>,
    /// Enum type definitions.
    pub enums: FVec<EnumDefinition>,
    /// Type aliases.
    pub type_aliases: FVec<TypeAliasDefinition>,
    /// Constant declarations.
    pub consts: FVec<ConstDefinition>,
    /// Static declarations.
    pub statics: FVec<StaticDefinition>,
    /// Use declarations.
    pub uses: FVec<UseDefinition>,
    /// Module declarations.
    pub mods: FVec<ModDefinition>,
    /// Trait declarations.
    pub traits: FVec<TraitDefinition>,
    /// Impl declarations.
    pub impls: FVec<ImplDefinition>,
}
/// Struct definition.
#[derive(Clone, Debug, PartialEq)]
pub struct StructDefinition {
    /// Struct name.
    pub name: FString,
    /// Generic type parameters.
    pub generics: FVec<FString>,
    /// Struct fields with types.
    pub fields: FVec<(FString, Type)>,
}
/// Enum variant definition.
#[derive(Clone, Debug, PartialEq)]
struct EnumVariant {
    /// Variant name.
    pub name: FString,
    /// Tuple-style payload fields.
    pub tuple_fields: FVec<Type>,
    /// Struct-style payload fields.
    pub named_fields: FVec<(FString, Type)>,
}
/// Enum definition.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumDefinition {
    /// Enum name.
    pub name: FString,
    /// Variants.
    pub variants: FVec<EnumVariant>,
}
/// Type alias definition.
#[derive(Clone, Debug, PartialEq)]
struct TypeAliasDefinition {
    /// Alias name.
    pub name: FString,
    /// Target type.
    pub target: Type,
}
/// Const definition.
#[derive(Clone, Debug, PartialEq)]
struct ConstDefinition {
    /// Const name.
    pub name: FString,
    /// Declared type if known.
    pub ty: Type,
    /// Initializer expression if present.
    pub value: Option<Spanned<Expression>>,
}
/// Static definition.
#[derive(Clone, Debug, PartialEq)]
struct StaticDefinition {
    /// Static name.
    pub name: FString,
    /// Declared type if known.
    pub ty: Type,
    /// Initializer expression if present.
    pub value: Option<Spanned<Expression>>,
}
/// Use declaration.
#[derive(Clone, Debug, PartialEq)]
struct UseDefinition {
    /// Imported path.
    pub path: FString,
}
/// Module declaration.
#[derive(Clone, Debug, PartialEq)]
struct ModDefinition {
    /// Module name.
    pub name: FString,
    /// Whether this module has an inline body.
    pub has_body: FBool,
}
/// Trait method signature.
#[derive(Clone, Debug, PartialEq)]
pub struct TraitMethodSig {
    /// Method name.
    pub name: FString,
    /// Parameter names and types.
    pub params: FVec<(FString, Type)>,
    /// Return type.
    pub return_type: Type,
}
/// Trait declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct TraitDefinition {
    /// Trait name.
    pub name: FString,
    /// Generic type parameters.
    pub generics: FVec<FString>,
    /// Required method signatures.
    pub methods: FVec<TraitMethodSig>,
}
/// Impl declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplDefinition {
    /// Optional trait being implemented.
    pub trait_name: Option<FString>,
    /// Target type being implemented for.
    pub target: FString,
    /// Generic type parameters.
    pub generics: FVec<FString>,
}
/// Resolved struct definition info for later compiler phases.
#[derive(Clone, Debug, PartialEq)]
struct StructInfo {
    /// Map field name to its type and index.
    pub fields: FMap<FString, (Type, FSize)>,
    /// Ordered list of fields for initialization and layout.
    pub ordered_fields: FVec<(FString, Type)>,
}
/// External function declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct ExternFunction {
    /// Function name.
    pub name: FString,
    /// Parameters and types.
    pub params: FVec<(FString, Type)>,
    /// Return type.
    pub return_type: Type,
    /// Whether the function is variadic (C-style).
    pub is_variadic: FBool,
}
/// Function definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// Function name.
    pub name: FString,
    /// Generic type parameters.
    pub generics: FVec<FString>,
    /// Parameters and types.
    pub params: FVec<(FString, Type)>,
    /// Return type.
    pub return_type: Type,
    /// Function body.
    pub body: Block,
}
/// Block of statements.
type Block = FVec<Spanned<Statement>>;
/// Statements.
#[derive(Clone, Debug, PartialEq)]
enum Statement {
    /// Let binding.
    Let { name: FString, ty: Type, value: Spanned<Expression> },
    /// Assignment to an L-value.
    Assignment { target: Spanned<Expression>, value: Spanned<Expression> },
    /// Return statement.
    Return(Option<Spanned<Expression>>),
    /// If/else statement.
    If { cond: Spanned<Expression>, then_block: Block, else_block: Option<Block> },
    /// While loop.
    While { cond: Spanned<Expression>, body: Block },
    /// Expression statement.
    Expression(Spanned<Expression>),
}
/// Match arm pattern (struct-based for bootstrap compatibility).
#[derive(Clone, Debug, PartialEq)]
pub struct MatchPattern {
    /// Pattern kind: "wildcard", "int", "bool", "string", "var".
    pub kind: FString,
    /// Integer value (for int patterns).
    pub int_val: FI64,
    /// Boolean value (for bool patterns).
    pub bool_val: FBool,
    /// String value (for string/var patterns).
    pub str_val: FString,
}
impl MatchPattern {
    pub fn wildcard() -> Self {
        MatchPattern { kind: "wildcard".to_string(), int_val: 0, bool_val: false, str_val: "".to_string() }
    }
    pub fn int_literal(val: FI64) -> Self {
        MatchPattern { kind: "int".to_string(), int_val: val, bool_val: false, str_val: "".to_string() }
    }
    pub fn bool_literal(val: FBool) -> Self {
        MatchPattern { kind: "bool".to_string(), int_val: 0, bool_val: val, str_val: "".to_string() }
    }
    pub fn string_literal(val: FString) -> Self {
        MatchPattern { kind: "string".to_string(), int_val: 0, bool_val: false, str_val: val }
    }
    pub fn variable(name: FString) -> Self {
        MatchPattern { kind: "var".to_string(), int_val: 0, bool_val: false, str_val: name }
    }
}
/// Match arm: pattern => expression.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchArm {
    /// Pattern to match against.
    pub pattern: MatchPattern,
    /// Optional guard condition.
    pub guard: Option<Box<Spanned<Expression>>>,
    /// Arm body.
    pub body: Spanned<Expression>,
}
/// Expressions.
#[derive(Clone, Debug, PartialEq)]
enum Expression {
    /// Integer literal.
    IntLiteral(FI64),
    /// Boolean literal.
    BoolLiteral(FBool),
    /// String literal.
    StringLiteral(FString),
    /// Variable reference.
    Variable(FString),
    /// Binary operation.
    BinaryOperation {
        op: BinaryOp,
        left: Box<Spanned<Expression>>,
        right: Box<Spanned<Expression>>,
    },
    /// Function call.
    FunctionCall { name: FString, args: FVec<Spanned<Expression>> },
    /// Method call: base.method(args)
    MethodCall { base: Box<Spanned<Expression>>, method: FString, args: FVec<Spanned<Expression>> },
    /// Array literal.
    ArrayLiteral(FVec<Spanned<Expression>>),
    /// Array repeat syntax: [value; N].
    ArrayRepeat { value: Box<Spanned<Expression>>, size: FSize },
    /// Struct literal: Name { field: expr, ... }.
    StructLiteral { name: FString, fields: FVec<(FString, Spanned<Expression>)> },
    /// Member access: base.field.
    MemberAccess { base: Box<Spanned<Expression>>, field: FString },
    /// Array indexing.
    Index { array: Box<Spanned<Expression>>, index: Box<Spanned<Expression>> },
    /// Address-of expression.
    AddressOf(Box<Spanned<Expression>>),
    /// Dereference expression.
    Dereference(Box<Spanned<Expression>>),
    /// Logical NOT expression.
    UnaryNot(Box<Spanned<Expression>>),
    /// Match expression: match scrutinee { arms }.
    Match { scrutinee: Box<Spanned<Expression>>, arms: FVec<MatchArm> },
    /// Lambda/closure expression: |params| body.
    Lambda { params: FVec<(FString, Type)>, body: Box<Spanned<Expression>>, captures: FVec<FString> },
    /// Array slice expression: expr[start..end].
    Slice { array: Box<Spanned<Expression>>, start: Option<Box<Spanned<Expression>>>, end: Option<Box<Spanned<Expression>>> },
}
/// Binary operators.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division.
    Div,
    /// Equality.
    Eq,
    /// Inequality.
    Neq,
    /// Less-than.
    Lt,
    /// Greater-than.
    Gt,
    /// Logical or.
    Or,
    /// Logical and.
    And,
}
```

---

## 6. Codegen Backend Trait (codegen.fu)

```fusion
//! Code generation backend abstractions.
///
/// Defines the `Backend` trait, `CodegenError` error type, and
/// `CodegenConfig` shared across all code generation backends.

use crate::ir;

/// Errors produced during code generation.
pub enum CodegenError {
    /// LLVM internal error (builder failures, module print errors).
    LlvmError(String),
    /// Compiler internal error (missing variables, unresolved symbols).
    InternalError(String),
    /// Target-specific error (llc invocation failure, bad triple).
    TargetError(String),
    /// WASM backend error.
    WasmError(String),
}

/// Configuration options for code generation.
pub struct CodegenConfig {
    /// Optimization level (0-3).
    pub opt_level: int,
    /// Target triple override (e.g., "x86_64-pc-windows-msvc").
    pub target_triple: Option<String>,
    /// Whether to emit textual LLVM IR (.ll).
    pub emit_llvm: bool,
    /// Source file path (for debug info and diagnostics).
    pub source_file_path: string,
    /// Compile as library (skip entry point checks).
    pub is_lib: bool,
    /// Emit DWARF debug information.
    pub debug_info: bool,
    /// Link to produce a final executable (not just an object file).
    pub emit_bin: bool,
    /// External libraries to link against (e.g., ["hypercycle_pqc"])
    pub link_libs: FVec<String>,
    /// Library search paths (e.g., ["v2.0 Vortex/build/Release"])
    pub lib_paths: FVec<String>,
}

/// Trait for code generation backends.
///
/// Each backend takes a lowered IR module and produces native output
/// (object file, executable, or LLVM IR text).
pub trait Backend {
    /// Returns the backend name (e.g., "LLVM").
    fn name(&self) -> &str;

    /// Generate code from the given IR module.
    ///
    /// Writes output to `output_path` (object file or LLVM IR depending
    /// on the `emit_llvm` flag in the backend configuration).
    ///
    /// Returns `Ok(())` on success, or a `CodegenError` describing the
    /// failure.
    fn generate(&mut self, ir: &ir::Module, output_path: &str) -> Result<(), CodegenError>;
}

// ---------------------------------------------------------------------------
// Target detection helpers
// ---------------------------------------------------------------------------

/// Check if a target triple indicates WASM.
/// Recognizes "wasm32-unknown-unknown", "wasm32-wasi", and similar.
pub fn is_wasm_target(target: &Option<String>) -> bool {
    match target {
        Some(t) => {
            let t_str: string = t.as_str();
            // Check for "wasm" prefix
            let len = t_str.len();
            if len < 4 {
                return false;
            }
            let c0: int = t_str[0] as int;
            let c1: int = t_str[1] as int;
            let c2: int = t_str[2] as int;
            let c3: int = t_str[3] as int;
            return (c0 == 119) & (c1 == 97) & (c2 == 115) & (c3 == 109); // 'w','a','s','m'
        }
        None => false,
    }
}

/// Select the appropriate codegen backend based on target triple.
/// Returns true if WASM backend should be used.
pub fn select_wasm_backend(config: &CodegenConfig) -> bool {
    return is_wasm_target(&((*config).target_triple));
}

// LLVM backend module (`mod llvm;` above).
// Use `crate::llvm::LlvmBackend` to access it.
// WASM backend is declared as `crate::wasm::backend::WasmBackend` (top-level `mod wasm;` in lib.fu).
```

---

## 7. IR Optimizer (optimizer.fu)

```fusion
//! IR optimization passes.
use crate::ast::{self, BinaryOp};
use crate::ir::{Address, BasicBlock, Instruction, IrFunction, Module, TypedValue, Value};
/// Applies optimization passes to the module.
pub fn optimize(module: Module) -> Module {
    let mut module = module;
    for func in &mut module.functions {
        optimize_function(func);
    }
    module
}
fn resolve_address(addr: &Address, consts: &FMap<Value, TypedValue>) -> Address {
    let resolve = |v: &TypedValue, consts: &FMap<Value, TypedValue>| -> TypedValue {
        consts.get(&v.val).cloned().unwrap_or_else(|| v.clone())
    };
    match addr {
        Address::Variable { name, ty } => {
            Address::Variable {
                name: name.clone(),
                ty: ty.clone(),
            }
        }
        Address::Pointer { val, pointed_to_ty } => {
            Address::Pointer {
                val: resolve(val, consts),
                pointed_to_ty: pointed_to_ty.clone(),
            }
        }
        Address::Element { base, index, element_ty } => {
            Address::Element {
                base: Box::new(resolve_address(base, consts)),
                index: resolve(index, consts),
                element_ty: element_ty.clone(),
            }
        }
        Address::Field { base, field_index, field_ty, struct_name } => {
            Address::Field {
                base: Box::new(resolve_address(base, consts)),
                field_index: *field_index,
                field_ty: field_ty.clone(),
                struct_name: struct_name.clone(),
            }
        }
    }
}
fn optimize_function(func: &mut IrFunction) {
    let mut changed = true;
    while changed {
        changed = false;
        let mut block_indices: FVec<FSize> = Vec::new();
        let mut idx = 0;
        while idx < func.blocks.len() {
            block_indices.push(idx);
            idx = idx + 1;
        }
        for block_idx in block_indices {
            if let Some(block) = func.blocks.get_mut(block_idx) {
                if constant_fold_block(block) {
                    changed = true;
                }
            }
        }
    }
}
fn constant_fold_block(block: &mut BasicBlock) -> FBool {
    let mut changed = false;
    let mut const_values: FMap<Value, TypedValue> = HashMap::new();
    let resolve = |v: &TypedValue, consts: &FMap<Value, TypedValue>| -> TypedValue {
        consts.get(&v.val).cloned().unwrap_or_else(|| v.clone())
    };
    let mut new_instrs = Vec::new();
    for instr in block.instrs.drain(..) {
        match instr {
            Instruction::BinaryOperation { dest, op, op1, op2 } => {
                let val1 = resolve(&op1, &const_values);
                let val2 = resolve(&op2, &const_values);
                let folded = match (val1.val.clone(), val2.val.clone()) {
                    (Value::IntConst(i1), Value::IntConst(i2)) => {
                        match op {
                            BinaryOp::Add => Some(Value::IntConst(i1 + i2)),
                            BinaryOp::Sub => Some(Value::IntConst(i1 - i2)),
                            BinaryOp::Mul => Some(Value::IntConst(i1 * i2)),
                            BinaryOp::Div => {
                                if i2 != 0 { Some(Value::IntConst(i1 / i2)) } else { None }
                            }
                            BinaryOp::Eq => Some(Value::BoolConst(i1 == i2)),
                            BinaryOp::Neq => Some(Value::BoolConst(i1 != i2)),
                            BinaryOp::Lt => Some(Value::BoolConst(i1 < i2)),
                            BinaryOp::Gt => Some(Value::BoolConst(i1 > i2)),
                            _ => None,
                        }
                    }
                    (Value::BoolConst(b1), Value::BoolConst(b2)) => {
                        match op {
                            BinaryOp::Or => Some(Value::BoolConst(b1 || b2)),
                            BinaryOp::And => Some(Value::BoolConst(b1 && b2)),
                            BinaryOp::Eq => Some(Value::BoolConst(b1 == b2)),
                            BinaryOp::Neq => Some(Value::BoolConst(b1 != b2)),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Some(v) = folded {
                    const_values
                        .insert(
                            dest.val.clone(),
                            TypedValue {
                                val: v,
                                ty: dest.ty.clone(),
                            },
                        );
                    changed = true;
                    continue;
                }
                new_instrs
                    .push(Instruction::BinaryOperation {
                        dest,
                        op,
                        op1: val1,
                        op2: val2,
                    });
            }
            Instruction::Call { dest, func_name, args } => {
                let resolved_args = args
                    .into_iter()
                    .map(|a| resolve(&a, &const_values))
                    .collect();
                new_instrs
                    .push(Instruction::Call {
                        dest,
                        func_name,
                        args: resolved_args,
                    });
            }
            Instruction::Load { dest, src } => {
                let resolved_src = resolve_address(&src, &const_values);
                new_instrs
                    .push(Instruction::Load {
                        dest,
                        src: resolved_src,
                    });
            }
            Instruction::Store { dest, val } => {
                let resolved_dest = resolve_address(&dest, &const_values);
                let resolved_val = resolve(&val, &const_values);
                new_instrs
                    .push(Instruction::Store {
                        dest: resolved_dest,
                        val: resolved_val,
                    });
            }
            Instruction::GetElementPtr { dest, base_ptr, index, element_ty } => {
                let resolved_base = resolve(&base_ptr, &const_values);
                let resolved_index = resolve(&index, &const_values);
                new_instrs
                    .push(Instruction::GetElementPtr {
                        dest,
                        base_ptr: resolved_base,
                        index: resolved_index,
                        element_ty,
                    });
            }
            Instruction::GetFieldPtr {
                dest,
                base_ptr,
                field_index,
                field_ty,
                struct_name,
            } => {
                let resolved_base = resolve(&base_ptr, &const_values);
                new_instrs
                    .push(Instruction::GetFieldPtr {
                        dest,
                        base_ptr: resolved_base,
                        field_index,
                        field_ty,
                        struct_name,
                    });
            }
            other => new_instrs.push(other),
        }
    }
    block.instrs = new_instrs;
    match &mut block.terminator {
        crate::ir::Terminator::Return(Some(ref mut v)) => *v = resolve(v, &const_values),
        crate::ir::Terminator::ConditionalJump { ref mut cond, .. } => {
            *cond = resolve(cond, &const_values);
        }
        _ => {}
    }
    changed
}
```

---

## Compilation Pipeline Summary

The bootstrap compiler processes Fusion source code through these phases:

1. **Lexer** → Token stream with spans
2. **Parser** → AST (Abstract Syntax Tree)
3. **Semantic Analysis** → Typed AST with validation
4. **IR Lowering** → Intermediate Representation
5. **Optimization** → Optimized IR (constant folding, etc.)
6. **Code Generation** → LLVM IR / Native Object / WASM

### Key Features

- **Type System**: int, bool, string, void, pointers, arrays, structs, generics, slices, closures
- **Language Constructs**: functions, structs, enums, traits, impls, match expressions, lambdas
- **Error Handling**: Forward compilation with Unknown type for error tolerance
- **Bootstrap Compatibility**: Tagged-union structs for IR values (no Rust enums in bootstrap)
- **ABI Limit**: 16-byte struct limit in bootstrap compiler
- **Backends**: LLVM (native) and WASM (WebAssembly)

### Usage

```bash
# Parse only
fuc --parse-only input.fu

# Semantic analysis only
fuc --sema-only input.fu

# Emit LLVM IR
fuc --emit-llvm input.fu -o output.ll

# Compile to object file
fuc input.fu -o output.o

# Compile and link to executable
fuc --emit-bin input.fu -o output.exe

# With optimization
fuc --opt-level 2 input.fu -o output.o

# With library linking
fuc --emit-bin --link-lib hypercycle_pqc --lib-path build/Release input.fu -o output.exe
```

---

*Document generated: 2026-07-02*
*Bootstrap Compiler Version: fuc (Fusion v2.0 Vortex)*
