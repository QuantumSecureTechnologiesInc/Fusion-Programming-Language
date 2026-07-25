# Fusion Core Compiler Modules: Parser, Semantic Analyzer & Vortex Engine

This document contains the complete source code for three critical Fusion compiler components:
1. **Parser** (parser.fu) - Transforms tokens into AST
2. **Semantic Analyzer** (sema.fu) - Type checking and validation
3. **Vortex Engine** (vortex.fu) - Borrow checker and safety analysis

---

## Table of Contents
1. [Parser Module](#1-parser-module-parserfu)
2. [Semantic Analyzer Module](#2-semantic-analyzer-module-semafu)
3. [Vortex Borrow Checker](#3-vortex-borrow-checker-vortexfu)

---

## 1. Parser Module (parser.fu)

**File:** `crates/fuc/src/parser.fu`  
**Lines:** 1,488  
**Purpose:** Transforms token stream into Abstract Syntax Tree (AST)

```fusion
//! Parser for the Fusion compiler.
use crate::ast::{self, Spanned};
use crate::lexer::Token;
use chumsky::prelude::*;
/// Parser error type.
pub type ParserError = Simple<Token>;
fn type_parser() -> impl Parser<Token, ast::Type, Error = ParserError> + Clone {
    recursive(|ty| {
        let ident = select! {
            Token::Identifier(name) => name
        };
        let path = ident
            .clone()
            .then(
                just(Token::ColonColon)
                    .ignore_then(ident.clone())
                    .repeated(),
            )
            .map(|(head, tail)| {
                let mut name = head;
                for seg in tail {
                    name = format!("{}::{}", name, seg);
                }
                name
            });
        let generics = just(Token::Less)
            .ignore_then(
                any()
                    .try_map(|t, span| {
                        if t == Token::Greater {
                            Err(Simple::custom(span, "end of generics"))
                        } else {
                            Ok(t)
                        }
                    })
                    .repeated()
                    .then_ignore(just(Token::Greater)),
            )
            .or_not();
        let base = select! {
            Token::TypeInt => ast::Type::Int, Token::TypeBool => ast::Type::Bool,
            Token::TypeString => ast::Type::String, Token::TypeVoid => ast::Type::Void,
        }
        .or(
            path.then(generics)
                .map(|(name, _)| ast::Type::Struct(name)),
        )
        .then(just(Token::Greater).repeated())
        .map(|(ty, _)| ty);
        let array = just(Token::LBracket)
            .ignore_then(ty.clone())
            .then_ignore(just(Token::Semicolon))
            .then(
                select! {
                    Token::IntLiteral(i) => i as usize
                },
            )
            .then_ignore(just(Token::RBracket))
            .map(|(elem, len)| ast::Type::Array(Box::new(elem), len));
        let pointer = just(Token::Star)
            .or(just(Token::Ampersand))
            .repeated()
            .then(base.or(array))
            .foldr(|_, inner_ty| ast::Type::Pointer(Box::new(inner_ty)));
        pointer
    })
}
#[derive(Clone, Debug)]
enum PostfixOp {
    Index(Spanned<ast::Expression>),
    Member(FString),
    MethodCall { method: FString, args: FVec<Spanned<ast::Expression>> },
}
#[derive(Clone, Debug)]
enum UnaryOp {
    Deref,
    AddrOf,
    Await,
    Not,
}
fn expr_parser() -> impl Parser<
    Token,
    Spanned<ast::Expression>,
    Error = ParserError,
> + Clone {
    let ident = select! {
        Token::Identifier(id) => id
    };
    let path = ident
        .clone()
        .then(just(Token::ColonColon).ignore_then(ident.clone()).repeated())
        .map(|(head, tail)| {
            let mut name = head;
            for seg in tail {
                name = format!("{}::{}", name, seg);
            }
            name
        });
    recursive(|expr| {
        let atom = select! {
            Token::IntLiteral(i) => ast::Expression::IntLiteral(i), Token::True =>
            ast::Expression::BoolLiteral(true), Token::False =>
            ast::Expression::BoolLiteral(false), Token::StringLiteral(s) =>
            ast::Expression::StringLiteral(s),
        }
            .map_with_span(|node, span| Spanned { node, span });
        let unit = just(Token::LParen)
            .ignore_then(just(Token::RParen))
            .map(|_| ast::Expression::IntLiteral(0))
            .map_with_span(|node, span| Spanned { node, span });
        let array_list = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .map(ast::Expression::ArrayLiteral);
        let array_repeat = expr
            .clone()
            .then_ignore(just(Token::Semicolon))
            .then(
                select! {
                    Token::IntLiteral(i) => i as usize
                },
            )
            .map(|(value, size)| ast::Expression::ArrayRepeat {
                value: Box::new(value),
                size,
            });
        let array_literal = array_repeat
            .or(array_list)
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map_with_span(|node, span| Spanned { node, span });
        let field_longhand = ident
            .clone()
            .then_ignore(just(Token::Colon))
            .then(expr.clone());
        let field_shorthand = ident
            .clone()
            .map_with_span(|name, span| {
                let var_expr = Spanned {
                    node: ast::Expression::Variable(name.clone()),
                    span,
                };
                (name, var_expr)
            });
        let field_init = field_longhand.or(field_shorthand);
        let struct_literal = path
            .clone()
            .then(
                field_init
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|(name, fields)| ast::Expression::StructLiteral {
                name,
                fields,
            })
            .map_with_span(|node, span| Spanned { node, span });
        let variable = path
            .clone()
            .map_with_span(|id, span| Spanned {
                node: ast::Expression::Variable(id),
                span,
            });
        let parenthesized = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));
        // Lambda: |params| expr  or  |params| { block }
        let lambda_param = ident
            .clone()
            .then(just(Token::Colon).ignore_then(type_parser()).or_not())
            .map(|(name, ty)| (name, ty.unwrap_or(ast::Type::Unknown)));
        let lambda_body_stmts = stmt_parser()
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map(|stmts| {
                let mut out: FVec<Spanned<ast::Expression>> = Vec::new();
                for stmt in stmts {
                    match stmt.node {
                        ast::Statement::Expression(e) => out.push(e),
                        ast::Statement::Return(Some(e)) => out.push(e),
                        _ => {}
                    }
                }
                if out.is_empty() {
                    let unit = Spanned {
                        node: ast::Expression::IntLiteral(0),
                        span: 0..0,
                    };
                    out.push(unit);
                }
                out
            });
        let lambda_expr = just(Token::Pipe)
            .ignore_then(
                lambda_param
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .then_ignore(just(Token::Pipe)),
            )
            .then(lambda_body_stmts.or(expr.clone().map(|e| vec![e])))
            .map_with_span(|(params, bodies), span| {
                let body = if bodies.len() == 1 {
                    bodies[0].clone()
                } else {
                    Spanned {
                        node: bodies.last().map_or(
                            ast::Expression::IntLiteral(0),
                            |e| e.node.clone(),
                        ),
                        span: span.clone(),
                    }
                };
                Spanned {
                    node: ast::Expression::Lambda {
                        params: params
                            .into_iter()
                            .map(|(name, ty)| (name, ty))
                            .collect(),
                        body: Box::new(body),
                        captures: Vec::new(),
                    },
                    span,
                }
            });
        let match_expr = just(Token::KwMatch)
            .ignore_then(expr.clone())
            .then(
                just(Token::FatArrow)
                    .to(ast::MatchPattern::wildcard())
                    .or(
                        select! {
                            Token::IntLiteral(i) => ast::MatchPattern::int_literal(i),
                            Token::True => ast::MatchPattern::bool_literal(true),
                            Token::False => ast::MatchPattern::bool_literal(false),
                            Token::StringLiteral(s) => ast::MatchPattern::string_literal(s),
                        },
                    )
                    .or(ident.clone().map(ast::MatchPattern::variable))
                    .or(just(Token::Minus).ignore_then(
                        select! { Token::IntLiteral(i) => i }
                    ).map(|i: FI64| ast::MatchPattern::int_literal(-i)))
                    .then(just(Token::KwIf).ignore_then(expr.clone()).or_not())
                    .then_ignore(just(Token::FatArrow))
                    .then(expr.clone())
                    .map(|((pattern, guard), body)| ast::MatchArm {
                        pattern,
                        guard: guard.map(Box::new),
                        body,
                    })
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(scrutinee, arms), span| Spanned {
                node: ast::Expression::Match {
                    scrutinee: Box::new(scrutinee),
                    arms,
                },
                span,
            });
        let call = path
            .then(just(Token::Bang).or_not())
            .then(
                expr
                    .clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .map(|((name, _), args)| ast::Expression::FunctionCall {
                name,
                args,
            });
        let primary = atom
            .or(unit)
            .or(struct_literal)
            .or(array_literal)
            .or(call.map_with_span(|node, span| Spanned { node, span }))
            .or(variable)
            .or(lambda_expr)
            .or(match_expr)
            .or(parenthesized);
        // Slice syntax: expr[start..end] or expr[..end] or expr[start..]
        let slice_full = expr
            .clone()
            .then(
                just(Token::Dot)
                    .ignore_then(just(Token::Dot))
                    .ignore_then(expr.clone())
                    .then(
                        just(Token::Dot)
                            .ignore_then(just(Token::Dot))
                            .ignore_then(expr.clone().or_not()),
                    ),
            )
            .map(|(array, (start, end))| Spanned {
                node: ast::Expression::Slice {
                    array: Box::new(array),
                    start: Some(Box::new(start)),
                    end: end.map(Box::new),
                },
                span: 0..0,
            });
        let slice_expr = expr
            .clone()
            .then(
                just(Token::Dot)
                    .ignore_then(just(Token::Dot))
                    .ignore_then(just(Token::Assign).or_not())
                    .ignore_then(expr.clone().or_not())
                    .or_not(),
            )
            .map(|(array, range_opt)| {
                match range_opt {
                    Some(Some(end)) => Spanned {
                        node: ast::Expression::Slice {
                            array: Box::new(array),
                            start: None,
                            end: Some(Box::new(end)),
                        },
                        span: 0..0,
                    },
                    _ => array,
                }
            });
        let postfix = primary
            .clone()
            .then(
                expr
                    .clone()
                    .delimited_by(just(Token::LBracket), just(Token::RBracket))
                    .map(PostfixOp::Index)
                    .or(slice_full.or(slice_expr).map(|_| PostfixOp::Member("".to_string())))
                    .or(
                        just(Token::Dot)
                            .ignore_then(ident.clone())
                            .then(
                                expr
                                    .clone()
                                    .separated_by(just(Token::Comma))
                                    .allow_trailing()
                                    .delimited_by(just(Token::LParen), just(Token::RParen))
                                    .or_not(),
                            )
                            .map(|(method, args)| {
                                match args {
                                    Some(args) => PostfixOp::MethodCall {
                                        method,
                                        args: args.into_iter().collect(),
                                    },
                                    None => PostfixOp::Member(method),
                                }
                            }),
                    )
                    .map_with_span(|op, span: std::ops::Range<FSize>| (op, span))
                    .repeated(),
            )
            .foldl(|base, (op, span)| {
                let span = base.span.start..span.end;
                let node = match op {
                    PostfixOp::Index(index) => {
                        ast::Expression::Index {
                            array: Box::new(base),
                            index: Box::new(index),
                        }
                    }
                    PostfixOp::Member(field) => {
                        ast::Expression::MemberAccess {
                            base: Box::new(base),
                            field,
                        }
                    }
                    PostfixOp::MethodCall { method, args } => {
                        ast::Expression::MethodCall {
                            base: Box::new(base),
                            method,
                            args,
                        }
                    }
                };
                Spanned { node, span }
            });
        let unary = just(Token::Star)
            .to(UnaryOp::Deref)
            .or(just(Token::Ampersand).to(UnaryOp::AddrOf))
            .or(just(Token::Bang).to(UnaryOp::Not))
            .or(just(Token::KwAwait).to(UnaryOp::Await))
            .repeated()
            .then(postfix)
            .foldr(|op, rhs| {
                let span = rhs.span.start..rhs.span.end;
                Spanned {
                    node: match op {
                        UnaryOp::Deref => ast::Expression::Dereference(Box::new(rhs)),
                        UnaryOp::AddrOf => ast::Expression::AddressOf(Box::new(rhs)),
                        UnaryOp::Not => ast::Expression::UnaryNot(Box::new(rhs)),
                        UnaryOp::Await => rhs.node,
                    },
                    span,
                }
            });
        let op = |token, op| just(token).to(op);
        let product = unary
            .clone()
            .then(
                op(Token::Star, ast::BinaryOp::Mul)
                    .or(op(Token::Slash, ast::BinaryOp::Div))
                    .then(unary)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned {
                    node: ast::Expression::BinaryOperation {
                        op,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    },
                    span,
                }
            });
        let sum = product
            .clone()
            .then(
                op(Token::Plus, ast::BinaryOp::Add)
                    .or(op(Token::Minus, ast::BinaryOp::Sub))
                    .then(product)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned {
                    node: ast::Expression::BinaryOperation {
                        op,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    },
                    span,
                }
            });
        let compare = sum
            .clone()
            .then(
                op(Token::Equals, ast::BinaryOp::Eq)
                    .or(op(Token::NotEquals, ast::BinaryOp::Neq))
                    .or(op(Token::Less, ast::BinaryOp::Lt))
                    .or(op(Token::Greater, ast::BinaryOp::Gt))
                    .then(sum)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned {
                    node: ast::Expression::BinaryOperation {
                        op,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    },
                    span,
                }
            });
        let logical_and = compare
            .clone()
            .then(op(Token::And, ast::BinaryOp::And).then(compare).repeated())
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned {
                    node: ast::Expression::BinaryOperation {
                        op,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    },
                    span,
                }
            });
        logical_and
            .clone()
            .then(op(Token::Or, ast::BinaryOp::Or).then(logical_and).repeated())
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned {
                    node: ast::Expression::BinaryOperation {
                        op,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    },
                    span,
                }
            })
    })
}
fn stmt_parser() -> impl Parser<
    Token,
    Spanned<ast::Statement>,
    Error = ParserError,
> + Clone {
    let ident = select! {
        Token::Identifier(id) => id
    };
    let expr = expr_parser();
    recursive(|stmt| {
        let block = stmt
            .clone()
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));
        let fallback_condition = any()
            .try_map(|t, span| {
                if t == Token::LBrace {
                    Err(Simple::custom(span, "start of condition block"))
                } else {
                    Ok(t)
                }
            })
            .repeated()
            .map_with_span(|_, span| Spanned {
                node: ast::Expression::BoolLiteral(true),
                span,
            });
        let condition = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .or(expr.clone())
            .or(fallback_condition);
        let let_decl = just(Token::KwLet)
            .ignore_then(
                select! {
                    Token::Identifier(name) if name == "mut" => ()
                }
                .or_not()
                .ignore_then(ident.clone()),
            )
            .then(
                just(Token::Colon)
                    .ignore_then(type_parser())
                    .or_not(),
            )
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|((name, ty), value)| ast::Statement::Let {
                name,
                ty: ty.unwrap_or(ast::Type::Unknown),
                value,
            });
        let assignment = expr
            .clone()
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(target, value)| ast::Statement::Assignment {
                target,
                value,
            });
        let ret = just(Token::KwReturn)
            .ignore_then(expr.clone().or_not())
            .then_ignore(just(Token::Semicolon))
            .map(ast::Statement::Return);
        let if_stmt = just(Token::KwIf)
            .ignore_then(condition.clone())
            .then(block.clone())
            .then(just(Token::KwElse).ignore_then(block.clone()).or_not())
            .map(|((cond, then_block), else_block)| ast::Statement::If {
                cond,
                then_block,
                else_block,
            });
        let while_stmt = just(Token::KwWhile)
            .ignore_then(condition)
            .then(block)
            .map(|(cond, body)| ast::Statement::While {
                cond,
                body,
            });
        let for_stmt = just(Token::KwFor)
            .ignore_then(
                any()
                    .try_map(|t, span| {
                        if t == Token::KwIn {
                            Err(Simple::custom(span, "end of for pattern"))
                        } else {
                            Ok(t)
                        }
                    })
                    .repeated()
                    .then_ignore(just(Token::KwIn)),
            )
            .ignore_then(expr.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|(iter_expr, for_body)| {
                let cond_span = iter_expr.span.clone();
                let setup_stmt = Spanned {
                    node: ast::Statement::Expression(iter_expr),
                    span: cond_span.clone(),
                };
                let mut desugared_body: FVec<Spanned<ast::Statement>> = Vec::new();
                desugared_body.push(setup_stmt);
                for stmt in for_body {
                    desugared_body.push(stmt);
                }
                ast::Statement::While {
                    cond: Spanned {
                        node: ast::Expression::BoolLiteral(true),
                        span: cond_span,
                    },
                    body: desugared_body,
                }
            });
        let expr_stmt = expr
            .clone()
            .then_ignore(just(Token::Semicolon))
            .map(ast::Statement::Expression);
        let_decl
            .or(assignment)
            .or(ret)
            .or(if_stmt)
            .or(while_stmt)
            .or(for_stmt)
            .or(expr_stmt)
            .map_with_span(|node, span| Spanned { node, span })
    })
}
#[derive(Clone, Debug)]
enum TopLevel {
    Function(ast::Function),
    Extern(ast::ExternFunction),
    Struct(ast::StructDefinition),
    Enum(ast::EnumDefinition),
    TypeAlias(ast::TypeAliasDefinition),
    Const(ast::ConstDefinition),
    Static(ast::StaticDefinition),
    Use(ast::UseDefinition),
    Mod(ast::ModDefinition),
    Trait(ast::TraitDefinition),
    Impl { def: ast::ImplDefinition, methods: FVec<ast::Function> },
}
fn program_parser() -> impl Parser<Token, ast::Program, Error = ParserError> {
    let ident = select! {
        Token::Identifier(id) => id
    };
    let path = ident
        .clone()
        .then(just(Token::ColonColon).ignore_then(ident.clone()).repeated())
        .map(|(head, tail)| {
            let mut name = head;
            for seg in tail {
                name = format!("{}::{}", name, seg);
            }
            name
        });
    let brace_block = recursive(|block| {
        let non_brace = any().try_map(|t, span| {
            if t == Token::LBrace || t == Token::RBrace {
                Err(Simple::custom(span, "brace boundary"))
            } else {
                Ok(())
            }
        });
        let inner = block.clone().map(|_| ()).or(non_brace);
        just(Token::LBrace)
            .ignore_then(inner.repeated())
            .then_ignore(just(Token::RBrace))
            .map(|_| ())
    });
    let visibility = just(Token::KwPub)
        .ignore_then(
            just(Token::LParen)
                .ignore_then(any().repeated())
                .then_ignore(just(Token::RParen))
                .or_not(),
        )
        .or_not();
    let generic_params = just(Token::Less)
        .ignore_then(
            ident
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing(),
        )
        .then_ignore(just(Token::Greater))
        .or_not()
        .map(|opt| opt.unwrap_or_else(|| Vec::new()));
    let stmt = stmt_parser();
    let block = stmt.repeated().delimited_by(just(Token::LBrace), just(Token::RBrace));
    let function_block = block
        .clone()
        .or(brace_block.clone().map(|_| Vec::new()));
    let field_or_param = visibility
        .clone()
        .ignore_then(ident.clone())
        .then_ignore(just(Token::Colon))
        .then(type_parser());
    let fields_or_params_list = field_or_param
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .map(|fields| fields.into_iter().collect::<FVec<_>>());
    let struct_def = visibility.clone()
        .ignore_then(just(Token::KwStruct))
        .ignore_then(ident.clone())
        .then(generic_params.clone())
        .then(
            fields_or_params_list
                .clone()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map(|((name, generics), fields)| ast::StructDefinition { name, generics, fields });
    let struct_def_opaque = visibility.clone()
        .ignore_then(just(Token::KwStruct))
        .ignore_then(ident.clone())
        .then(
            any()
                .try_map(|t, span| {
                    if t == Token::LBrace {
                        Err(Simple::custom(span, "start of struct body"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .then_ignore(brace_block.clone()),
        )
        .map(|(name, _)| ast::StructDefinition {
            name,
            generics: Vec::new(),
            fields: Vec::new(),
        });
    let return_type_strict = just(Token::Colon)
        .or(just(Token::Arrow))
        .ignore_then(type_parser());
    let return_type_fallback = just(Token::Colon)
        .or(just(Token::Arrow))
        .ignore_then(
            any()
                .try_map(|t, span| {
                    if t == Token::LBrace || t == Token::KwWhere {
                        Err(Simple::custom(span, "start of function body"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated(),
        )
        .to(ast::Type::Unknown);
    let return_type = return_type_strict
        .or(return_type_fallback)
        .or_not()
        .map(|t| t.unwrap_or(ast::Type::Void));
    let params_list = field_or_param
        .clone()
        .separated_by(just(Token::Comma))
        .map(|fields| fields.into_iter().collect::<FVec<_>>());
    let params_list_trailing = field_or_param
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .map(|fields| fields.into_iter().collect::<FVec<_>>());
    let params_then_variadic = params_list
        .then_ignore(just(Token::Comma))
        .then_ignore(just(Token::Ellipsis))
        .map(|params| (params, true));
    let params_ellipsis_only = just(Token::Ellipsis).map(|_| (Vec::new(), true));
    let params_no_variadic = params_list_trailing.map(|params| (params, false));
    let params_with_variadic = params_then_variadic
        .or(params_ellipsis_only)
        .or(params_no_variadic)
        .or_not()
        .map(|opt| opt.unwrap_or_else(|| (Vec::new(), false)));
    let extern_func = visibility.clone()
        .ignore_then(just(Token::KwExtern))
        .ignore_then(just(Token::KwFn))
        .ignore_then(ident.clone())
        .then(
            params_with_variadic.delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(return_type.clone())
        .then_ignore(just(Token::Semicolon))
        .map(|((name, (params, is_variadic)), return_type)| ast::ExternFunction {
            name,
            params,
            return_type,
            is_variadic,
        });
    let function = visibility.clone()
        .ignore_then(just(Token::KwAsync).or_not())
        .ignore_then(just(Token::KwFn))
        .ignore_then(ident.clone())
        .then(generic_params.clone())
        .then(
            fields_or_params_list.delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(return_type.clone())
        .then(
            just(Token::KwWhere)
                .ignore_then(
                    any()
                        .try_map(|t, span| {
                            if t == Token::LBrace {
                                Err(Simple::custom(span, "where clause ended"))
                            } else {
                                Ok(t)
                            }
                        })
                        .repeated(),
                )
                .or_not(),
        )
        .then(function_block.clone())
        .map(|(((((name, generics), params), return_type), _), body)| ast::Function {
            name,
            generics,
            params,
            return_type,
            body,
        });
    let function_opaque = visibility.clone()
        .ignore_then(just(Token::KwAsync).or_not())
        .ignore_then(just(Token::KwFn))
        .ignore_then(ident.clone())
        .then(
            any()
                .try_map(|t, span| {
                    if t == Token::LBrace {
                        Err(Simple::custom(span, "start of function body"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .then_ignore(brace_block.clone()),
        )
        .map(|(name, _)| ast::Function {
            name,
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ast::Type::Unknown,
            body: Vec::new(),
        });
    let enum_tuple_payload = type_parser()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .or_not()
        .map(|opt| opt.unwrap_or_else(|| Vec::new()));
    let enum_named_field = ident
        .clone()
        .then_ignore(just(Token::Colon))
        .then(type_parser());
    let enum_named_payload = enum_named_field
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .or_not()
        .map(|opt| opt.unwrap_or_else(|| Vec::new()));
    let enum_variant = ident
        .clone()
        .then(enum_tuple_payload)
        .then(enum_named_payload)
        .map(|((name, tuple_fields), named_fields)| ast::EnumVariant {
            name,
            tuple_fields,
            named_fields,
        });
    let enum_decl = visibility.clone()
        .ignore_then(just(Token::KwEnum))
        .ignore_then(ident.clone())
        .then(generic_params.clone())
        .then(
            enum_variant
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map(|((name, _), variants)| TopLevel::Enum(ast::EnumDefinition { name, variants }));
    let enum_decl_opaque = visibility.clone()
        .ignore_then(just(Token::KwEnum))
        .ignore_then(ident.clone())
        .then_ignore(
            any()
                .try_map(|t, span| {
                    if t == Token::LBrace {
                        Err(Simple::custom(span, "start of enum body"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .then_ignore(brace_block.clone()),
        )
        .map(|name| {
            TopLevel::Enum(ast::EnumDefinition {
                name,
                variants: Vec::new(),
            })
        });
    let type_decl = visibility.clone()
        .ignore_then(just(Token::KwType))
        .ignore_then(ident.clone())
        .then(generic_params.clone())
        .then_ignore(just(Token::Assign))
        .then(type_parser())
        .then_ignore(just(Token::Semicolon))
        .map(|((name, _), target)| {
            TopLevel::TypeAlias(ast::TypeAliasDefinition {
                name,
                target,
            })
        });
    let type_decl_opaque = visibility.clone()
        .ignore_then(just(Token::KwType))
        .ignore_then(ident.clone())
        .then_ignore(
            any()
                .try_map(|t, span| {
                    if t == Token::Semicolon {
                        Err(Simple::custom(span, "end of type alias"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .then_ignore(just(Token::Semicolon)),
        )
        .map(|name| {
            TopLevel::TypeAlias(ast::TypeAliasDefinition {
                name,
                target: ast::Type::Unknown,
            })
        });
    let const_decl = visibility.clone()
        .ignore_then(just(Token::KwConst))
        .ignore_then(ident.clone())
        .then(just(Token::Colon).ignore_then(type_parser()).or_not())
        .then(just(Token::Assign).ignore_then(expr_parser()).or_not())
        .then_ignore(just(Token::Semicolon))
        .map(|((name, ty), value)| {
            TopLevel::Const(ast::ConstDefinition {
                name,
                ty: ty.unwrap_or(ast::Type::Unknown),
                value,
            })
        });
    let const_decl_opaque = visibility.clone()
        .ignore_then(just(Token::KwConst))
        .ignore_then(ident.clone())
        .then(just(Token::Colon).ignore_then(type_parser()).or_not())
        .then_ignore(
            any()
                .try_map(|t, span| {
                    if t == Token::Semicolon {
                        Err(Simple::custom(span, "end of const"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .then_ignore(just(Token::Semicolon)),
        )
        .map(|(name, ty)| {
            TopLevel::Const(ast::ConstDefinition {
                name,
                ty: ty.unwrap_or(ast::Type::Unknown),
                value: None,
            })
        });
    let static_decl = visibility.clone()
        .ignore_then(just(Token::KwStatic))
        .ignore_then(ident.clone())
        .then(just(Token::Colon).ignore_then(type_parser()).or_not())
        .then(just(Token::Assign).ignore_then(expr_parser()).or_not())
        .then_ignore(just(Token::Semicolon))
        .map(|((name, ty), value)| {
            TopLevel::Static(ast::StaticDefinition {
                name,
                ty: ty.unwrap_or(ast::Type::Unknown),
                value,
            })
        });
    let static_decl_opaque = visibility.clone()
        .ignore_then(just(Token::KwStatic))
        .ignore_then(ident.clone())
        .then(just(Token::Colon).ignore_then(type_parser()).or_not())
        .then_ignore(
            any()
                .try_map(|t, span| {
                    if t == Token::Semicolon {
                        Err(Simple::custom(span, "end of static"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .then_ignore(just(Token::Semicolon)),
        )
        .map(|(name, ty)| {
            TopLevel::Static(ast::StaticDefinition {
                name,
                ty: ty.unwrap_or(ast::Type::Unknown),
                value: None,
            })
        });
    let use_decl = visibility.clone()
        .ignore_then(just(Token::KwUse))
        .ignore_then(
            any()
                .try_map(|t, span| {
                    if t == Token::Semicolon {
                        Err(Simple::custom(span, "end of use"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .map(|tokens| {
                    let mut out = String::new();
                    for token in tokens {
                        let part = match token {
                            Token::Identifier(name) => name,
                            Token::ColonColon => "::".to_string(),
                            Token::Star => "*".to_string(),
                            Token::Comma => ",".to_string(),
                            Token::LBrace => "{".to_string(),
                            Token::RBrace => "}".to_string(),
                            _ => "".to_string(),
                        };
                        if part.is_empty() {
                            continue;
                        }
                        if !out.is_empty()
                            && part != "::"
                            && part != ","
                            && part != "}"
                            && !out.ends_with("::")
                            && !out.ends_with("{")
                            && !out.ends_with(",")
                        {
                            out.push(' ');
                        }
                        out.push_str(&part);
                    }
                    out
                })
                .then_ignore(just(Token::Semicolon)),
        )
        .map(|path| TopLevel::Use(ast::UseDefinition { path }));
    let mod_decl = visibility.clone()
        .ignore_then(just(Token::KwMod))
        .ignore_then(ident.clone())
        .then_ignore(just(Token::Semicolon))
        .map(|name| {
            TopLevel::Mod(ast::ModDefinition {
                name,
                has_body: false,
            })
        });
    let mod_block = visibility.clone()
        .ignore_then(just(Token::KwMod))
        .ignore_then(ident.clone())
        .then_ignore(brace_block.clone())
        .map(|name| {
            TopLevel::Mod(ast::ModDefinition {
                name,
                has_body: true,
            })
        });
    let impl_self_ref = just(Token::Ampersand)
        .ignore_then(
            select! {
                Token::Identifier(name) if name == "mut" => ()
            }
            .or_not(),
        )
        .ignore_then(
            select! {
                Token::Identifier(name) if name == "self" => name
            },
        )
        .map(|name| (name, ast::Type::Unknown));
    let impl_self_value = select! {
            Token::Identifier(name) if name == "self" => name
        }
        .map(|name| (name, ast::Type::Unknown));
    let impl_param = impl_self_ref
        .or(impl_self_value)
        .or(field_or_param.clone());
    let impl_params_list = impl_param
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .map(|params| params.into_iter().collect::<FVec<_>>());
    let impl_method_strict = visibility
        .clone()
        .ignore_then(just(Token::KwAsync).or_not())
        .ignore_then(just(Token::KwFn))
        .ignore_then(ident.clone())
        .then(generic_params.clone())
        .then(
            impl_params_list
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(return_type.clone())
        .then(
            just(Token::KwWhere)
                .ignore_then(
                    any()
                        .try_map(|t, span| {
                            if t == Token::LBrace {
                                Err(Simple::custom(span, "where clause ended"))
                            } else {
                                Ok(t)
                            }
                        })
                        .repeated(),
                )
                .or_not(),
        )
        .then(function_block.clone())
        .map(|(((((name, generics), params), return_type), _), body)| ast::Function {
            name,
            generics,
            params,
            return_type,
            body,
        });
    let impl_method_opaque = visibility
        .clone()
        .ignore_then(just(Token::KwAsync).or_not())
        .ignore_then(just(Token::KwFn))
        .ignore_then(ident.clone())
        .then(
            any()
                .try_map(|t, span| {
                    if t == Token::LBrace {
                        Err(Simple::custom(span, "start of impl method body"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated(),
        )
        .then(function_block.clone())
        .map(|((name, _), body)| ast::Function {
            name,
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ast::Type::Unknown,
            body,
        });
    let impl_method = impl_method_strict.or(impl_method_opaque);
    let impl_decl = visibility
        .clone()
        .ignore_then(just(Token::KwImpl))
        .ignore_then(
            any()
                .try_map(|t, span| {
                    if t == Token::LBrace {
                        Err(Simple::custom(span, "end of impl header"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated(),
        )
        .then(impl_method.repeated().delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map(|(header_tokens, methods)| {
            let mut trait_name: Option<FString> = None;
            let mut target = "impl".to_string();
            // Detect "impl TraitName for TypeName" pattern
            let mut seen_for = false;
            let mut after_for: Option<FString> = None;
            let mut pre_for_upper: Option<FString> = None;
            for token in &header_tokens {
                if let Token::Identifier(name) = token {
                    if name == "for" {
                        seen_for = true;
                    } else if seen_for && after_for.is_none() {
                        after_for = Some(name.clone());
                    } else if !seen_for && name.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                        if pre_for_upper.is_none() {
                            pre_for_upper = Some(name.clone());
                        }
                    }
                }
            }
            if let Some(for_type) = after_for {
                trait_name = pre_for_upper;
                target = for_type;
            } else {
                for token in &header_tokens {
                    if let Token::Identifier(name) = token {
                        if let Some(first) = name.chars().next() {
                            if first.is_ascii_uppercase() {
                                target = name.clone();
                                break;
                            }
                        }
                    }
                }
                if target == "impl" {
                    for token in header_tokens {
                        if let Token::Identifier(name) = token {
                            target = name;
                        }
                    }
                }
            }
            let mut lowered_methods: FVec<ast::Function> = Vec::new();
            for method in methods {
                let rewritten_return_type = match method.return_type.clone() {
                    ast::Type::Struct(name) if name == "Self" => {
                        ast::Type::Struct(target.clone())
                    }
                    ast::Type::Pointer(inner) => {
                        if let ast::Type::Struct(name) = *inner {
                            if name == "Self" {
                                ast::Type::Pointer(Box::new(ast::Type::Struct(target.clone())))
                            } else {
                                ast::Type::Pointer(Box::new(ast::Type::Struct(name)))
                            }
                        } else {
                            ast::Type::Pointer(inner)
                        }
                    }
                    other => other,
                };
                let mut rewritten_params: FVec<(FString, ast::Type)> = Vec::new();
                for (param_name, param_ty) in method.params {
                    let rewritten_ty = match param_ty {
                        ast::Type::Struct(name) if name == "Self" => {
                            ast::Type::Struct(target.clone())
                        }
                        ast::Type::Pointer(inner) => {
                            if let ast::Type::Struct(name) = *inner {
                                if name == "Self" {
                                    ast::Type::Pointer(Box::new(ast::Type::Struct(target.clone())))
                                } else {
                                    ast::Type::Pointer(Box::new(ast::Type::Struct(name)))
                                }
                            } else {
                                ast::Type::Pointer(inner)
                            }
                        }
                        other => other,
                    };
                    rewritten_params.push((param_name, rewritten_ty));
                }
                lowered_methods.push(ast::Function {
                    name: format!("{}::{}", target, method.name),
                    generics: method.generics,
                    params: rewritten_params,
                    return_type: rewritten_return_type,
                    body: method.body,
                });
            }
            TopLevel::Impl {
                def: ast::ImplDefinition {
                    trait_name,
                    target,
                    generics: Vec::new(),
                },
                methods: lowered_methods,
            }
        });
    let impl_decl_opaque = visibility
        .clone()
        .ignore_then(just(Token::KwImpl))
        .ignore_then(
            any()
                .try_map(|t, span| {
                    if t == Token::LBrace {
                        Err(Simple::custom(span, "end of impl header"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .then_ignore(brace_block.clone()),
        )
        .map(|header_tokens| {
            let mut trait_name: Option<FString> = None;
            let mut target = "impl".to_string();
            // Detect "impl TraitName for TypeName" pattern
            let mut seen_for = false;
            let mut after_for: Option<FString> = None;
            let mut pre_for_upper: Option<FString> = None;
            for token in &header_tokens {
                if let Token::Identifier(name) = token {
                    if name == "for" {
                        seen_for = true;
                    } else if seen_for && after_for.is_none() {
                        after_for = Some(name.clone());
                    } else if !seen_for && name.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                        if pre_for_upper.is_none() {
                            pre_for_upper = Some(name.clone());
                        }
                    }
                }
            }
            if let Some(for_type) = after_for {
                trait_name = pre_for_upper;
                target = for_type;
            } else {
                for token in &header_tokens {
                    if let Token::Identifier(name) = token {
                        if let Some(first) = name.chars().next() {
                            if first.is_ascii_uppercase() {
                                target = name.clone();
                                break;
                            }
                        }
                    }
                }
                if target == "impl" {
                    for token in header_tokens {
                        if let Token::Identifier(name) = token {
                            target = name;
                        }
                    }
                }
            }
            TopLevel::Impl {
                def: ast::ImplDefinition {
                    trait_name,
                    target,
                    generics: Vec::new(),
                },
                methods: Vec::new(),
            }
        });
    let trait_decl = visibility
        .clone()
        .ignore_then(just(Token::KwTrait))
        .ignore_then(ident.clone())
        .then(generic_params.clone())
        .then(
            // Trait method signatures: fn name(params) -> Type;
            field_or_param
                .clone()
                .then(return_type.clone())
                .map(|((name, ty), ret)| ast::TraitMethodSig {
                    name,
                    params: vec![("self".to_string(), ty)],
                    return_type: ret,
                })
                .separated_by(just(Token::Semicolon))
                .allow_trailing()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
                .or_not()
                .map(|opt| opt.unwrap_or_else(|| Vec::new())),
        )
        .map(|((name, generics), methods)| TopLevel::Trait(ast::TraitDefinition { name, generics, methods }));
    let attributes = just(Token::Hash)
        .ignore_then(just(Token::LBracket))
        .ignore_then(
            any()
                .try_map(|t, span| {
                    if t == Token::RBracket {
                        Err(Simple::custom(span, "end of attribute"))
                    } else {
                        Ok(t)
                    }
                })
                .repeated()
                .then_ignore(just(Token::RBracket)),
        )
        .repeated();
    let top_level_item = extern_func
        .map(TopLevel::Extern)
        .or(function.map(TopLevel::Function))
        .or(function_opaque.map(TopLevel::Function))
        .or(struct_def.map(TopLevel::Struct))
        .or(struct_def_opaque.map(TopLevel::Struct))
        .or(enum_decl)
        .or(enum_decl_opaque)
        .or(type_decl)
        .or(type_decl_opaque)
        .or(const_decl)
        .or(const_decl_opaque)
        .or(static_decl)
        .or(static_decl_opaque)
        .or(use_decl)
        .or(mod_decl)
        .or(mod_block)
        .or(impl_decl)
        .or(impl_decl_opaque)
        .or(trait_decl);
    let top_level_item = attributes.ignore_then(top_level_item);
    top_level_item
        .repeated()
        .map(|items| {
            let mut functions = Vec::new();
            let mut externs = Vec::new();
            let mut structs = Vec::new();
            let mut enums = Vec::new();
            let mut type_aliases = Vec::new();
            let mut consts = Vec::new();
            let mut statics = Vec::new();
            let mut uses = Vec::new();
            let mut mods = Vec::new();
            let mut traits = Vec::new();
            let mut impls = Vec::new();
            for item in items {
                match item {
                    TopLevel::Function(f) => functions.push(f),
                    TopLevel::Extern(e) => externs.push(e),
                    TopLevel::Struct(s) => structs.push(s),
                    TopLevel::Enum(e) => enums.push(e),
                    TopLevel::TypeAlias(a) => type_aliases.push(a),
                    TopLevel::Const(c) => consts.push(c),
                    TopLevel::Static(s) => statics.push(s),
                    TopLevel::Use(u) => uses.push(u),
                    TopLevel::Mod(m) => mods.push(m),
                    TopLevel::Trait(t) => traits.push(t),
                    TopLevel::Impl { def, methods } => {
                        impls.push(def);
                        for method in methods {
                            functions.push(method);
                        }
                    }
                }
            }
            ast::Program {
                functions,
                externs,
                structs,
                enums,
                type_aliases,
                consts,
                statics,
                uses,
                mods,
                traits,
                impls,
            }
        })
        .then_ignore(end())
}
/// Parses source text into an AST program.
pub fn parse_program(input: &str) -> (Option<ast::Program>, FVec<ParserError>) {
    let tokens = crate::lexer::lex(input);
    let token_stream = chumsky::Stream::from_iter(
        tokens.len()..tokens.len(),
        tokens.into_iter(),
    );
    program_parser().parse_recovery(token_stream)
}
/// Parse output container for native-friendly field access.
pub struct ParseOutput {
    /// Parsed program, if successful.
    pub program: Option<ast::Program>,
    /// Parser diagnostics.
    pub errors: FVec<ParserError>,
}
/// Real parser output used by host compiler entry paths.
pub fn parse_host_output(input: &str) -> ParseOutput {
    let (program, errors) = parse_program(input);
    ParseOutput { program, errors }
}

/// Parser output used by stage1 status helpers.
///
/// This now returns the real parser output path and no longer synthesises
/// `Some(empty_program)` sentinel values.
pub fn parse_output(input: &str) -> ParseOutput {
    return parse_host_output(input);
}

/// In-process parser status helper for stage1 API wiring.
///
/// Returns:
/// - 0: parse succeeded
/// - 3: parse produced diagnostics
/// - 4: parse produced no program
pub fn parse_status(input: FString) -> FInt {
    let output = parse_output(&input);
    if output.errors.len() > 0 {
        return 3;
    }
    if output.program.is_none() {
        return 4;
    }
    return 0;
}
```

---

## 2. Semantic Analyzer Module (sema.fu)

**File:** `crates/fuc/src/sema.fu`  
**Lines:** 1,576  
**Purpose:** Type checking, validation, and typed AST generation

```fusion
//! Semantic analysis and type checking for the Fusion compiler.
use crate::ast::*;
/// Native semantic diagnostic payload.
#[derive(Clone, Debug)]
pub struct SemanticDiagnostic {
    /// Source span associated with the diagnostic.
    pub span: Span,
    /// Human-readable diagnostic message.
    pub message: FString,
}
/// Typed program with extern declarations.
#[derive(Clone, Debug)]
pub struct TypedProgram {
    /// Defined functions.
    pub functions: FVec<TypedFunction>,
    /// External declarations.
    pub externs: FVec<TypedExternFunction>,
    /// Struct definitions.
    pub structs: FVec<TypedStructDefinition>,
}
/// Typed function definition.
#[derive(Clone, Debug)]
pub struct TypedFunction {
    /// Function name.
    pub name: FString,
    /// Parameter names and types.
    pub params: FVec<(FString, Type)>,
    /// Return type.
    pub return_type: Type,
    /// Typed body.
    pub body: TypedBlock,
}
/// Typed external function declaration.
#[derive(Clone, Debug)]
pub struct TypedExternFunction {
    /// Function name.
    pub name: FString,
    /// Parameter names and types.
    pub params: FVec<(FString, Type)>,
    /// Return type.
    pub return_type: Type,
    /// Whether the function is variadic.
    pub is_variadic: FBool,
}
/// Typed struct definition.
#[derive(Clone, Debug)]
pub struct TypedStructDefinition {
    /// Struct name.
    pub name: FString,
    /// Fields in order (name, type).
    pub fields: FVec<(FString, Type)>,
}
/// Typed block of statements.
type TypedBlock = FVec<TypedStatement>;
/// Typed statements.
#[derive(Clone, Debug)]
enum TypedStatement {
    /// Let binding.
    Let { name: FString, ty: Type, value: TypedExpression },
    /// Assignment statement.
    Assignment { target: TypedExpression, value: TypedExpression },
    /// Return statement.
    Return(Option<TypedExpression>),
    /// If/else statement.
    If { cond: TypedExpression, then_block: TypedBlock, else_block: Option<TypedBlock> },
    /// While loop.
    While { cond: TypedExpression, body: TypedBlock },
    /// Expression statement.
    Expression(TypedExpression),
}
/// Typed expression wrapper.
#[derive(Clone, Debug)]
struct TypedExpression {
    /// Expression node.
    pub node: TypedExpressionKind,
    /// Result type.
    pub ty: Type,
    /// Source span.
    pub span: Span,
}
/// Typed expression kinds.
#[derive(Clone, Debug)]
enum TypedExpressionKind {
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
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
    },
    /// Function call.
    FunctionCall { name: FString, args: FVec<TypedExpression> },
    /// Array literal.
    ArrayLiteral(FVec<TypedExpression>),
    /// Array repeat syntax.
    ArrayRepeat { value: Box<TypedExpression>, size: FSize },
    /// Array indexing.
    Index { array: Box<TypedExpression>, index: Box<TypedExpression> },
    /// Address-of expression.
    AddressOf(Box<TypedExpression>),
    /// Dereference expression.
    Dereference(Box<TypedExpression>),
    /// Struct member access.
    MemberAccess { base: Box<TypedExpression>, field_name: FString, field_index: FSize },
    /// Struct literal expression.
    StructLiteral { name: FString, fields: FVec<(FString, FSize, TypedExpression)> },
    /// Match expression (desugared to if-else in IR).
    Match { scrutinee: Box<TypedExpression>, arms: FVec<(MatchPattern, Option<TypedExpression>, TypedExpression)> },
    /// Lambda/closure expression.
    Lambda { params: FVec<(FString, Type)>, body: Box<TypedExpression>, captures: FVec<(FString, Type)> },
    /// Array slice expression.
    Slice { array: Box<TypedExpression>, start: Option<Box<TypedExpression>>, end: Option<Box<TypedExpression>> },
}
/// Semantic analysis output for native-friendly field access.
pub struct AnalyzeOutput {
    /// Typed program, if semantic analysis succeeded.
    pub program: Option<TypedProgram>,
    /// Semantic diagnostics.
    pub errors: FVec<SemanticDiagnostic>,
}
struct SymbolTable {
    scopes: FVec<FMap<FString, Type>>,
    functions: FMap<FString, (FVec<Type>, Type, FBool, FBool)>,
    structs: FMap<FString, StructInfo>,
    named_types: FSet<FString>,
}
impl SymbolTable {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            named_types: HashSet::new(),
        }
    }
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn insert_var(&mut self, name: FString, ty: Type) -> Result<(), FString> {
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.contains_key(&name) {
            Err(format!("Variable '{}' is already defined in this scope.", name))
        } else {
            current_scope.insert(name, ty);
            Ok(())
        }
    }
    fn lookup_var(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
    fn insert_func(
        &mut self,
        name: FString,
        params: FVec<Type>,
        ret: Type,
        is_variadic: FBool,
        is_extern: FBool,
    ) -> Result<(), FString> {
        if self.functions.contains_key(&name) {
            Err(format!("Function '{}' is already defined.", name))
        } else {
            self.functions
                .insert(name, (params, ret, is_variadic, is_extern));
            Ok(())
        }
    }
    fn lookup_func(&self, name: &str) -> Option<&(FVec<Type>, Type, FBool, FBool)> {
        self.functions.get(name)
    }
    fn type_layout(&self, ty: &Type) -> Option<(FSize, FSize)> {
        match ty {
            Type::Int => Some((4, 4)),
            Type::Bool => Some((1, 1)),
            Type::String => Some((8, 8)),
            Type::Void => Some((0, 1)),
            Type::Unknown => None,
            Type::GenericParam(_) => Some((8, 8)),
            Type::Pointer(_) => Some((8, 8)),
            Type::Slice(_) => Some((16, 8)),
            Type::Array(elem, len) => {
                let (elem_size, elem_align) = self.type_layout(elem)?;
                let size = elem_size.checked_mul(*len)?;
                Some((size, elem_align))
            }
            Type::Struct(name) => {
                let info = self.structs.get(name)?;
                let mut size = 0usize;
                let mut max_align = 1usize;
                for (_, field_ty) in &info.ordered_fields {
                    let (field_size, field_align) = self.type_layout(field_ty)?;
                    if field_align > max_align {
                        max_align = field_align;
                    }
                    let padding = (field_align - (size % field_align)) % field_align;
                    size = size.checked_add(padding)?;
                    size = size.checked_add(field_size)?;
                }
                let final_padding = (max_align - (size % max_align)) % max_align;
                size = size.checked_add(final_padding)?;
                Some((size, max_align))
            }
            Type::Closure(_, _) => Some((24, 8)),
        }
    }
    fn reserve_struct(&mut self, name: FString) -> Result<(), FString> {
        if self.structs.contains_key(&name) {
            return Err(format!("Struct '{}' is already defined.", name));
        }
        self.named_types.insert(name.clone());
        self.structs
            .insert(
                name,
                StructInfo {
                    fields: HashMap::new(),
                    ordered_fields: Vec::new(),
                },
            );
        Ok(())
    }
    fn register_named_type(&mut self, name: FString) {
        self.named_types.insert(name);
    }
    fn define_struct(
        &mut self,
        name: FString,
        fields: FVec<(FString, Type)>,
    ) -> Result<(), FString> {
        let mut field_map = HashMap::new();
        for (i, (field_name, field_ty)) in fields.iter().enumerate() {
            if field_map.contains_key(field_name) {
                return Err(
                    format!("Duplicate field '{}' in struct '{}'", field_name, name),
                );
            }
            field_map.insert(field_name.clone(), (field_ty.clone(), i));
        }
        self.structs
            .insert(
                name,
                StructInfo {
                    fields: field_map,
                    ordered_fields: fields,
                },
            );
        Ok(())
    }
    fn lookup_struct(&self, name: &str) -> Option<&StructInfo> {
        self.structs.get(name)
    }
    fn is_valid_type(&self, ty: &Type) -> FBool {
        match ty {
            Type::Int | Type::Bool | Type::String | Type::Void => {
                true
            }
            Type::Unknown => true,
            Type::GenericParam(_) => true,
            Type::Slice(inner) => self.is_valid_type(inner),
            Type::Closure(_, ret) => self.is_valid_type(ret),
            Type::Pointer(inner) => self.is_valid_type(inner),
            Type::Array(inner, _) => self.is_valid_type(inner),
            Type::Struct(name) => {
                self.structs.contains_key(name)
                    || self.named_types.contains(name)
                    || name == "Option"
                    || name == "Result"
                    || name == "FVec"
                    || name == "Vec"
                    || name == "FMap"
                    || name == "HashMap"
                    || name == "FSet"
                    || name == "HashSet"
                    || name == "FBTreeMap"
                    || name == "BTreeMap"
                    || name == "FBTreeSet"
                    || name == "BTreeSet"
                    || name == "FString"
                    || name == "FBool"
                    || name == "FChar"
                    || name == "FInt"
                    || name == "FI64"
                    || name == "FU32"
                    || name == "FU64"
                    || name == "FSize"
                    || name == "Span"
                    || name == "Range"
                    || name == "Type"
                    || name == "Block"
                    || name == "TypedBlock"
                    || name == "TypedExpressionKind"
                    || name == "Program"
                    || name == "Token"
                    || name == "ParseOutput"
                    || name == "AnalyzeOutput"
                    || name == "ParserError"
                    || name == "MatchPattern"
                    || name == "MatchArm"
                    || name == "TraitMethodSig"
                    || name == "TraitDefinition"
                    || name == "ImplDefinition"
                    || name == "UseDefinition"
                    || name == "ModDefinition"
                    || name == "EnumDefinition"
                    || name == "EnumVariant"
                    || name == "Visibility"
                    || name == "ConstDefinition"
                    || name == "StaticDefinition"
                    || name == "TypeAliasDefinition"
                    || name == "T"
                    || name == "U"
                    || name == "V"
                    || name == "K"
                    || name == "E"
                    || name.contains("::")
            }
        }
    }
}
/// Semantic analyzer.
pub struct Analyzer {
    symbols: SymbolTable,
    errors: FVec<SemanticDiagnostic>,
    current_return_type: Option<Type>,
}
impl Analyzer {
    /// Creates a new analyzer.
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            current_return_type: None,
        }
    }
    fn is_lvalue(&self, expr: &TypedExpression) -> FBool {
        matches!(
            expr.node,
            TypedExpressionKind::Variable(_)
                | TypedExpressionKind::Dereference(_)
                | TypedExpressionKind::Index { .. }
                | TypedExpressionKind::MemberAccess { .. }
        )
    }
    fn type_name_is(name: &str, simple: &str) -> FBool {
        if name == simple {
            return true;
        }
        let qualified = format!("::{}", simple);
        return name.ends_with(&qualified);
    }
    fn normalise_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Struct(name)
                if Self::type_name_is(name, "FInt")
                    || Self::type_name_is(name, "FI64")
                    || Self::type_name_is(name, "FU32")
                    || Self::type_name_is(name, "FU64")
                    || Self::type_name_is(name, "FSize")
                    || Self::type_name_is(name, "FChar")
                    || Self::type_name_is(name, "i32")
                    || Self::type_name_is(name, "i64")
                    || Self::type_name_is(name, "u32")
                    || Self::type_name_is(name, "u64")
                    || Self::type_name_is(name, "usize")
                    || Self::type_name_is(name, "isize")
                    || Self::type_name_is(name, "int")
                    || Self::type_name_is(name, "char") => Type::Int,
            Type::Struct(name)
                if Self::type_name_is(name, "FBool")
                    || Self::type_name_is(name, "bool") => Type::Bool,
            Type::Struct(name)
                if Self::type_name_is(name, "FString")
                    || Self::type_name_is(name, "String")
                    || Self::type_name_is(name, "str")
                    || Self::type_name_is(name, "string") => Type::String,
            Type::Struct(name)
                if Self::type_name_is(name, "Span")
                    || Self::type_name_is(name, "Range") => Type::Unknown,
            Type::Pointer(inner) => {
                Type::Pointer(Box::new(self.normalise_type(inner)))
            }
            Type::Array(inner, size) => {
                Type::Array(Box::new(self.normalise_type(inner)), *size)
            }
            Type::GenericParam(_) => ty.clone(),
            Type::Slice(inner) => {
                Type::Slice(Box::new(self.normalise_type(inner)))
            }
            Type::Closure(params, ret) => {
                let norm_params: FVec<Type> = params.iter().map(|p| self.normalise_type(p)).collect();
                Type::Closure(norm_params, Box::new(self.normalise_type(ret)))
            }
            _ => ty.clone(),
        }
    }
    fn types_compatible(&self, expected: &Type, found: &Type) -> FBool {
        let expected_norm = self.normalise_type(expected);
        let found_norm = self.normalise_type(found);
        if expected_norm == found_norm {
            return true;
        }
        if matches!(expected_norm, Type::Unknown) || matches!(found_norm, Type::Unknown) {
            return true;
        }
        // GenericParam is compatible with anything (monomorphisation happens later)
        if matches!(expected_norm, Type::GenericParam(_)) || matches!(found_norm, Type::GenericParam(_)) {
            return true;
        }
        false
    }
    /// Resolves callable names with a module-qualified fallback.
    ///
    /// If `module::func` is referenced but only `func` is known in the current
    /// symbol table, this returns `func`. Otherwise it returns `raw_name`.
    fn resolve_callable_name(&self, raw_name: &str) -> FString {
        if self.symbols.lookup_func(raw_name).is_some() {
            return raw_name.to_string();
        }

        if raw_name.contains("::") {
            let segments: FVec<&str> = raw_name.split("::").collect();
            if segments.len() > 1 {
                let mut idx = 1;
                while idx < segments.len() {
                    let mut suffix = String::new();
                    let mut inner = idx;
                    while inner < segments.len() {
                        if inner > idx {
                            suffix.push_str("::");
                        }
                        suffix.push_str(segments[inner]);
                        inner += 1;
                    }
                    if self.symbols.lookup_func(&suffix).is_some() {
                        return suffix;
                    }
                    idx += 1;
                }
            }
            if !segments.is_empty() {
                let leaf = segments[segments.len() - 1];
                if self.symbols.lookup_func(leaf).is_some() {
                    return leaf.to_string();
                }
            }
        }

        return raw_name.to_string();
    }
    fn report_error(&mut self, span: Span, msg: FString) {
        self.errors.push(SemanticDiagnostic {
            span,
            message: msg,
        });
    }
    /// Analyzes and type-checks a program.
    pub fn analyze(
        mut self,
        program: Program,
    ) -> (Option<TypedProgram>, FVec<SemanticDiagnostic>) {
        let Program {
            functions,
            externs,
            structs,
            enums,
            type_aliases,
            consts: _,
            statics: _,
            uses: _,
            mods: _,
            traits: _,
            impls: _,
        } = program;
        for enum_def in &enums {
            self.symbols.register_named_type(enum_def.name.clone());
        }
        for alias_def in &type_aliases {
            self.symbols.register_named_type(alias_def.name.clone());
        }
        for s in &structs {
            if let Err(msg) = self.symbols.reserve_struct(s.name.clone()) {
                self.report_error(0..0, msg);
            }
        }
        for s in &structs {
            self.validate_struct_definition(s);
        }
        for ext in &externs {
            if let Some(msg) = self.check_extern_abi(ext) {
                self.report_error(0..0, msg);
            }
            let param_types = ext
                .params
                .iter()
                .map(|p| self.rewrite_extern_type(&p.1))
                .collect();
            let ret_type = self.rewrite_extern_type(&ext.return_type);
            if let Err(msg) = self
                .symbols
                .insert_func(
                    ext.name.clone(),
                    param_types,
                    ret_type,
                    ext.is_variadic,
                    true,
                )
            {
                self.report_error(0..0, msg);
            }
        }
        for func in &functions {
            let param_types = func.params.iter().map(|p| p.1.clone()).collect();
            if let Err(msg) = self
                .symbols
                .insert_func(
                    func.name.clone(),
                    param_types,
                    func.return_type.clone(),
                    false,
                    false,
                )
            {
                self.report_error(0..0, msg);
            }
        }
        if !self.errors.is_empty() {
            return (None, self.errors);
        }
        let mut typed_functions = Vec::new();
        for func in functions {
            if let Some(msg) = self.check_function_abi(&func) {
                self.report_error(0..0, msg);
            }
            // Native self-host mode keeps entry-point checks permissive so the
            // compiler can analyze larger Rust-oriented sources incrementally.
            self.current_return_type = Some(func.return_type.clone());
            self.symbols.push_scope();
            for (name, ty) in &func.params {
                if let Err(msg) = self
                    .symbols
                    .insert_var(name.clone(), self.normalise_type(ty))
                {
                    self.report_error(0..0, msg);
                }
            }
            let body = self.analyze_block(func.body);
            self.symbols.pop_scope();
            typed_functions
                .push(TypedFunction {
                    name: func.name,
                    params: func.params,
                    return_type: func.return_type,
                    body,
                });
        }
        if self.errors.is_empty() {
            let typed_externs = externs
                .into_iter()
                .map(|ext| {
                    let params = ext
                        .params
                        .into_iter()
                        .map(|(name, ty)| (name, self.rewrite_extern_type(&ty)))
                        .collect();
                    let return_type = self.rewrite_extern_type(&ext.return_type);
                    TypedExternFunction {
                        name: ext.name,
                        params,
                        return_type,
                        is_variadic: ext.is_variadic,
                    }
                })
                .collect();
            let typed_structs = structs
                .iter()
                .filter_map(|s| {
                    let info = self.symbols.lookup_struct(&s.name)?;
                    Some(TypedStructDefinition {
                        name: s.name.clone(),
                        fields: info.ordered_fields.clone(),
                    })
                })
                .collect();
            (
                Some(TypedProgram {
                    functions: typed_functions,
                    externs: typed_externs,
                    structs: typed_structs,
                }),
                self.errors,
            )
        } else {
            (None, self.errors)
        }
    }
    /// Native-friendly wrapper around `analyze` with field-based access.
    pub fn analyze_output(self, program: Program) -> AnalyzeOutput {
        let (typed_program, errors) = self.analyze(program);
        AnalyzeOutput {
            program: typed_program,
            errors,
        }
    }
    fn check_extern_abi(&self, ext: &ExternFunction) -> Option<FString> {
        if ext.is_variadic {
            for (name, ty) in &ext.params {
                if self.is_aggregate_type(ty) {
                    return Some(
                        format!(
                            "Extern '{}' is variadic; aggregate parameter '{}: {:?}' is not allowed. Use pointer or split fields.",
                            ext.name, name, ty
                        ),
                    );
                }
            }
        }
        if self.is_aggregate_type(&ext.return_type) {
            return Some(
                format!(
                    "Extern '{}' returns aggregate type '{:?}'. Use a pointer return (e.g., *T) instead.",
                    ext.name, ext.return_type
                ),
            );
        }
        for (name, ty) in &ext.params {
            if self.is_aggregate_type(ty) {
                if let Some((size, _)) = self.symbols.type_layout(ty) {
                    if size > 32 {
                        return Some(
                            format!(
                                "Extern '{}' has large aggregate parameter '{}: {:?}' ({} bytes). Use pointer or split fields.",
                                ext.name, name, ty, size
                            ),
                        );
                    }
                }
            }
        }
        None
    }
    fn is_aggregate_type(&self, ty: &Type) -> FBool {
        matches!(ty, Type::Struct(_) | Type::Array(_, _))
    }
    fn rewrite_extern_type(&self, ty: &Type) -> Type {
        if self.is_aggregate_type(ty) {
            Type::Pointer(Box::new(ty.clone()))
        } else {
            ty.clone()
        }
    }
    fn check_function_abi(&self, func: &Function) -> Option<FString> {
        for (name, ty) in &func.params {
            if self.is_aggregate_type(ty) {
                if let Some((size, _)) = self.symbols.type_layout(ty) {
                    if size > 65536 {
                        return Some(
                            format!(
                                "Function '{}' has by-value aggregate parameter '{}: {:?}' larger than 64KB ({}). Use a pointer instead.",
                                func.name, name, ty, size
                            ),
                        );
                    }
                }
            }
        }
        if self.is_aggregate_type(&func.return_type) {
            if let Some((size, _)) = self.symbols.type_layout(&func.return_type) {
                if size > 65536 {
                    return Some(
                        format!(
                            "Function '{}' has by-value aggregate return type '{:?}' larger than 64KB ({}). Use a pointer return instead.",
                            func.name, func.return_type, size
                        ),
                    );
                }
            }
        }
        None
    }
    fn validate_struct_definition(&mut self, def: &StructDefinition) {
        let mut valid_fields = Vec::new();
        for (field_name, field_ty) in &def.fields {
            if !self.symbols.is_valid_type(field_ty) {
                self.report_error(
                    0..0,
                    format!(
                        "Field '{}' in struct '{}' has unknown type {:?}.", field_name,
                        def.name, field_ty
                    ),
                );
            } else {
                valid_fields.push((field_name.clone(), field_ty.clone()));
            }
        }
        if let Err(msg) = self.symbols.define_struct(def.name.clone(), valid_fields) {
            self.report_error(0..0, msg);
        }
        let mut visited = HashSet::new();
        visited.insert(def.name.clone());
        for (_, field_ty) in &def.fields {
            self.check_recursive_type(field_ty, &mut visited, &def.name);
        }
    }
    fn check_recursive_type(
        &mut self,
        ty: &Type,
        visited: &mut FSet<FString>,
        origin_struct: &str,
    ) {
        match ty {
            Type::Struct(name) => {
                if visited.contains(name) {
                    self.report_error(
                        0..0,
                        format!(
                            "Struct '{}' has infinite recursive definition via field of type '{}'.",
                            origin_struct, name
                        ),
                    );
                } else if let Some(info) = self.symbols.lookup_struct(name).cloned() {
                    visited.insert(name.clone());
                    for (_, field_ty) in &info.ordered_fields {
                        self.check_recursive_type(field_ty, visited, origin_struct);
                    }
                    visited.remove(name);
                }
            }
            Type::Pointer(_) => {}
            Type::Array(inner, _) => {
                self.check_recursive_type(inner, visited, origin_struct)
            }
            _ => {}
        }
    }
    fn analyze_block(&mut self, block: Block) -> TypedBlock {
        block.into_iter().map(|stmt| self.analyze_statement(stmt)).collect()
    }
    fn analyze_statement(&mut self, stmt: Spanned<Statement>) -> TypedStatement {
        match stmt.node {
            Statement::Let { name, ty, value } => {
                let typed_value = self.analyze_expression(value);
                if !self.types_compatible(&ty, &typed_value.ty) {
                    self.report_error(
                        typed_value.span.clone(),
                        format!(
                            "Type mismatch. Expected {:?}, found {:?}.", ty, typed_value
                            .ty
                        ),
                    );
                }
                if let Err(msg) = self
                    .symbols
                    .insert_var(name.clone(), self.normalise_type(&ty))
                {
                    self.report_error(stmt.span, msg);
                }
                TypedStatement::Let {
                    name,
                    ty,
                    value: typed_value,
                }
            }
            Statement::Assignment { target, value } => {
                let typed_target = self.analyze_expression(target);
                let typed_value = self.analyze_expression(value);
                let is_lvalue = matches!(
                    typed_target.node, TypedExpressionKind::Variable(_) |
                    TypedExpressionKind::Dereference(_) | TypedExpressionKind::Index { ..
                    } | TypedExpressionKind::MemberAccess { .. }
                );
                if !is_lvalue {
                    self.report_error(
                        typed_target.span.clone(),
                        "Left-hand side of assignment must be an L-value.".to_string(),
                    );
                }
                if !self.types_compatible(&typed_target.ty, &typed_value.ty) {
                    self.report_error(
                        typed_value.span.clone(),
                        format!(
                            "Type mismatch. Cannot assign {:?} to target of type {:?}.",
                            typed_value.ty, typed_target.ty
                        ),
                    );
                }
                TypedStatement::Assignment {
                    target: typed_target,
                    value: typed_value,
                }
            }
            Statement::Return(expr_opt) => {
                let expected = self.current_return_type.as_ref().unwrap().clone();
                match (expr_opt, expected) {
                    (Some(expr), expected_ty) => {
                        let typed_expr = self.analyze_expression(expr);
                        if !self.types_compatible(&expected_ty, &typed_expr.ty) {
                            self.report_error(
                                typed_expr.span.clone(),
                                format!(
                                    "Type mismatch. Function expected to return {:?}, but returned {:?}.",
                                    expected_ty, typed_expr.ty
                                ),
                            );
                        }
                        TypedStatement::Return(Some(typed_expr))
                    }
                    (None, Type::Void) => TypedStatement::Return(None),
                    (None, expected_ty) => {
                        self.report_error(
                            stmt.span,
                            format!(
                                "Function must return a value of type {:?}.", expected_ty
                            ),
                        );
                        TypedStatement::Return(None)
                    }
                }
            }
            Statement::If { cond, then_block, else_block } => {
                let typed_cond = self.analyze_expression(cond);
                if !self.types_compatible(&Type::Bool, &typed_cond.ty) {
                    self.report_error(
                        typed_cond.span.clone(),
                        format!(
                            "If condition must be of type Bool, found {:?}.", typed_cond
                            .ty
                        ),
                    );
                }
                self.symbols.push_scope();
                let typed_then = self.analyze_block(then_block);
                self.symbols.pop_scope();
                let typed_else = else_block
                    .map(|block| {
                        self.symbols.push_scope();
                        let b = self.analyze_block(block);
                        self.symbols.pop_scope();
                        b
                    });
                TypedStatement::If {
                    cond: typed_cond,
                    then_block: typed_then,
                    else_block: typed_else,
                }
            }
            Statement::While { cond, body } => {
                let typed_cond = self.analyze_expression(cond);
                if !self.types_compatible(&Type::Bool, &typed_cond.ty) {
                    self.report_error(
                        typed_cond.span.clone(),
                        format!(
                            "While condition must be of type Bool, found {:?}.",
                            typed_cond.ty
                        ),
                    );
                }
                self.symbols.push_scope();
                let typed_body = self.analyze_block(body);
                self.symbols.pop_scope();
                TypedStatement::While {
                    cond: typed_cond,
                    body: typed_body,
                }
            }
            Statement::Expression(expr) => {
                TypedStatement::Expression(self.analyze_expression(expr))
            }
        }
    }
    fn analyze_expression(&mut self, expr: Spanned<Expression>) -> TypedExpression {
        let (node, ty) = match expr.node {
            Expression::IntLiteral(i) => {
                (TypedExpressionKind::IntLiteral(i), Type::Int)
            }
            Expression::BoolLiteral(b) => {
                (TypedExpressionKind::BoolLiteral(b), Type::Bool)
            }
            Expression::StringLiteral(s) => {
                (TypedExpressionKind::StringLiteral(s), Type::String)
            }
            Expression::Variable(name) => {
                if let Some(ty) = self.symbols.lookup_var(&name) {
                    (TypedExpressionKind::Variable(name), ty.clone())
                } else {
                    // Native self-host mode: keep unknown symbols as Unknown to
                    // allow progressive compilation of partially-resolved sources.
                    (TypedExpressionKind::Variable(name), Type::Unknown)
                }
            }
            Expression::BinaryOperation { op, left, right } => {
                let typed_left = self.analyze_expression(*left);
                let typed_right = self.analyze_expression(*right);
                let ty = match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div => {
                        if !self.types_compatible(&Type::Int, &typed_left.ty)
                            || !self.types_compatible(&Type::Int, &typed_right.ty)
                        {
                            self.report_error(
                                expr.span.clone(),
                                format!("Operator {:?} requires Int operands.", op),
                            );
                        }
                        Type::Int
                    }
                    BinaryOp::Eq | BinaryOp::Neq => {
                        if !self.types_compatible(&typed_left.ty, &typed_right.ty) {
                            self.report_error(
                                expr.span.clone(),
                                format!(
                                    "Operands for {:?} must have same type. left={:?}, right={:?}.",
                                    op, typed_left.ty, typed_right.ty
                                ),
                            );
                        }
                        Type::Bool
                    }
                    BinaryOp::Lt | BinaryOp::Gt => {
                        if !self.types_compatible(&Type::Int, &typed_left.ty)
                            || !self.types_compatible(&Type::Int, &typed_right.ty)
                        {
                            self.report_error(
                                expr.span.clone(),
                                format!("Operator {:?} requires Int operands.", op),
                            );
                        }
                        Type::Bool
                    }
                    BinaryOp::Or | BinaryOp::And => {
                        if !self.types_compatible(&Type::Bool, &typed_left.ty)
                            || !self.types_compatible(&Type::Bool, &typed_right.ty)
                        {
                            self.report_error(
                                expr.span.clone(),
                                format!("Operator {:?} requires Bool operands.", op),
                            );
                        }
                        Type::Bool
                    }
                };
                (
                    TypedExpressionKind::BinaryOperation {
                        op,
                        left: Box::new(typed_left),
                        right: Box::new(typed_right),
                    },
                    ty,
                )
            }
            Expression::FunctionCall { name, args } => {
                let typed_args: FVec<_> = args
                    .into_iter()
                    .map(|arg| self.analyze_expression(arg))
                    .collect();
                let resolved_name = self.resolve_callable_name(&name);
                let arg_count = typed_args.len();
                if let Some((param_types, ret_type, is_variadic, is_extern)) = self
                    .symbols
                    .lookup_func(&resolved_name)
                    .cloned()
                {
                    let mut coerced_args: FVec<TypedExpression> = Vec::new();
                    for (idx, arg) in typed_args.into_iter().enumerate() {
                        let expected = param_types.get(idx);
                        if let Some(expected_ty) = expected {
                            if is_extern
                                && matches!(expected_ty, Type::Pointer(inner) if self.is_aggregate_type(inner))
                                && self.is_aggregate_type(&arg.ty)
                                && self.is_lvalue(&arg)
                            {
                                coerced_args.push(TypedExpression {
                                    node: TypedExpressionKind::AddressOf(Box::new(arg)),
                                    ty: expected_ty.clone(),
                                    span: expr.span.clone(),
                                });
                                continue;
                            }
                        }
                        coerced_args.push(arg);
                    }
                    let strict_signature = !(param_types.is_empty() && matches!(ret_type, Type::Unknown));
                    if strict_signature && !is_variadic && param_types.len() != arg_count {
                        self.report_error(
                            expr.span.clone(),
                            format!(
                                "Function '{}' expects {} arguments, got {}.", resolved_name,
                                param_types.len(), arg_count
                            ),
                        );
                    } else if strict_signature && is_variadic && arg_count < param_types.len() {
                        self.report_error(
                            expr.span.clone(),
                            format!(
                                "Function '{}' expects at least {} arguments, got {}.",
                                resolved_name, param_types.len(), arg_count
                            ),
                        );
                    }
                    for (i, (arg, expected_ty)) in coerced_args
                        .iter()
                        .zip(&param_types)
                        .enumerate()
                    {
                        let both_concrete = !matches!(expected_ty, Type::Unknown)
                            && !matches!(arg.ty, Type::Unknown);
                        let expected_primitive = matches!(
                            expected_ty,
                            Type::Int
                                | Type::Bool
                                | Type::String
                                | Type::Void
                        );
                        if both_concrete
                            && expected_primitive
                            && !self.types_compatible(expected_ty, &arg.ty)
                        {
                            self.report_error(
                                arg.span.clone(),
                                format!(
                                    "Argument {} of '{}' type mismatch.",
                                    i + 1,
                                    resolved_name
                                ),
                            );
                        }
                    }
                    (
                        TypedExpressionKind::FunctionCall {
                            name: resolved_name,
                            args: coerced_args,
                        },
                        ret_type,
                    )
                } else {
                    (
                        TypedExpressionKind::FunctionCall {
                            name,
                            args: typed_args,
                        },
                        Type::Unknown,
                    )
                }
            }
            Expression::MethodCall { base, method, args } => {
                let typed_base = self.analyze_expression(*base);
                let mut typed_args: FVec<_> = Vec::new();
                typed_args.push(typed_base);
                for arg in args {
                    typed_args.push(self.analyze_expression(arg));
                }
                let mut method_name = method.clone();
                if let Some(receiver) = typed_args.first() {
                    if let Type::Struct(struct_name) = &receiver.ty {
                        let qualified = format!("{}::{}", struct_name, method);
                        if self.symbols.lookup_func(&qualified).is_some() {
                            method_name = qualified;
                        }
                    }
                }
                let name = self.resolve_callable_name(&method_name);
                let arg_count = typed_args.len();
                if let Some((param_types, ret_type, is_variadic, is_extern)) = self
                    .symbols
                    .lookup_func(&name)
                    .cloned()
                {
                    let mut coerced_args: FVec<TypedExpression> = Vec::new();
                    for (idx, arg) in typed_args.into_iter().enumerate() {
                        let expected = param_types.get(idx);
                        if let Some(expected_ty) = expected {
                            if is_extern
                                && matches!(expected_ty, Type::Pointer(inner) if self.is_aggregate_type(inner))
                                && self.is_aggregate_type(&arg.ty)
                                && self.is_lvalue(&arg)
                            {
                                coerced_args.push(TypedExpression {
                                    node: TypedExpressionKind::AddressOf(Box::new(arg)),
                                    ty: expected_ty.clone(),
                                    span: expr.span.clone(),
                                });
                                continue;
                            }
                        }
                        coerced_args.push(arg);
                    }
                    let strict_signature = !(param_types.is_empty() && matches!(ret_type, Type::Unknown));
                    if strict_signature && !is_variadic && param_types.len() != arg_count {
                        self.report_error(
                            expr.span.clone(),
                            format!(
                                "Function '{}' expects {} arguments, got {}.", name,
                                param_types.len(), arg_count
                            ),
                        );
                    } else if strict_signature && is_variadic && arg_count < param_types.len() {
                        self.report_error(
                            expr.span.clone(),
                            format!(
                                "Function '{}' expects at least {} arguments, got {}.",
                                name, param_types.len(), arg_count
                            ),
                        );
                    }
                    for (i, (arg, expected_ty)) in coerced_args
                        .iter()
                        .zip(&param_types)
                        .enumerate()
                    {
                        let both_concrete = !matches!(expected_ty, Type::Unknown)
                            && !matches!(arg.ty, Type::Unknown);
                        let expected_primitive = matches!(
                            expected_ty,
                            Type::Int
                                | Type::Bool
                                | Type::String
                                | Type::Void
                        );
                        if both_concrete
                            && expected_primitive
                            && !self.types_compatible(expected_ty, &arg.ty)
                        {
                            self.report_error(
                                arg.span.clone(),
                                format!(
                                    "Argument {} of '{}' type mismatch.",
                                    i + 1,
                                    name
                                ),
                            );
                        }
                    }
                    (
                        TypedExpressionKind::FunctionCall {
                            name,
                            args: coerced_args,
                        },
                        ret_type,
                    )
                } else {
                    (
                        TypedExpressionKind::FunctionCall {
                            name,
                            args: typed_args,
                        },
                        Type::Unknown,
                    )
                }
            }
            Expression::ArrayLiteral(elements) => {
                let typed_elements: FVec<_> = elements
                    .into_iter()
                    .map(|e| self.analyze_expression(e))
                    .collect();
                if typed_elements.is_empty() {
                    self.report_error(
                        expr.span.clone(),
                        "Empty array literals are not yet supported (cannot infer type)."
                            .to_string(),
                    );
                    return TypedExpression {
                        node: TypedExpressionKind::ArrayLiteral(vec![]),
                        ty: Type::Void,
                        span: expr.span,
                    };
                }
                let first_ty = typed_elements[0].ty.clone();
                for (i, elem) in typed_elements.iter().enumerate().skip(1) {
                    if !self.types_compatible(&first_ty, &elem.ty) {
                        self.report_error(
                            elem.span.clone(),
                            format!(
                                "Element {} has type {:?}, but expected {:?} (to match first element).",
                                i, elem.ty, first_ty
                            ),
                        );
                    }
                }
                let array_ty = Type::Array(
                    Box::new(first_ty),
                    typed_elements.len(),
                );
                (TypedExpressionKind::ArrayLiteral(typed_elements), array_ty)
            }
            Expression::ArrayRepeat { value, size } => {
                let typed_value = self.analyze_expression(*value);
                if size == 0 {
                    self.report_error(
                        expr.span.clone(),
                        "Array size must be greater than zero.".to_string(),
                    );
                }
                let array_ty = Type::Array(Box::new(typed_value.ty.clone()), size);
                (
                    TypedExpressionKind::ArrayRepeat {
                        value: Box::new(typed_value),
                        size,
                    },
                    array_ty,
                )
            }
            Expression::Index { array, index } => {
                let typed_array = self.analyze_expression(*array);
                let typed_index = self.analyze_expression(*index);
                if !self.types_compatible(&Type::Int, &typed_index.ty) {
                    self.report_error(
                        typed_index.span.clone(),
                        format!(
                            "Array index must be of type Int, found {:?}.", typed_index
                            .ty
                        ),
                    );
                }
                if let Type::Array(elem_ty, _) = typed_array.ty.clone() {
                    (
                        TypedExpressionKind::Index {
                            array: Box::new(typed_array),
                            index: Box::new(typed_index),
                        },
                        *elem_ty,
                    )
                } else {
                    let allow_index = matches!(typed_array.ty, Type::Unknown)
                        || matches!(
                            &typed_array.ty,
                            Type::Struct(name)
                                if name == "Vec"
                                    || name == "FVec"
                                    || Self::type_name_is(name, "Vec")
                                    || Self::type_name_is(name, "FVec")
                        );
                    if !allow_index {
                        self.report_error(
                            typed_array.span.clone(),
                            format!("Cannot index non-array type {:?}.", typed_array.ty),
                        );
                    }
                    (
                        TypedExpressionKind::Index {
                            array: Box::new(typed_array),
                            index: Box::new(typed_index),
                        },
                        Type::Unknown,
                    )
                }
            }
            Expression::MemberAccess { base, field } => {
                let typed_base = self.analyze_expression(*base);
                if let Type::Struct(struct_name) = &typed_base.ty {
                    if let Some(info) = self.symbols.lookup_struct(struct_name).cloned()
                    {
                        if let Some((field_ty, field_idx)) = info.fields.get(&field) {
                            (
                                TypedExpressionKind::MemberAccess {
                                    base: Box::new(typed_base),
                                    field_name: field.clone(),
                                    field_index: *field_idx,
                                },
                                field_ty.clone(),
                            )
                        } else {
                            self.report_error(
                                expr.span.clone(),
                                format!(
                                    "Struct '{}' has no field named '{}'.", struct_name, field
                                ),
                            );
                            (
                                TypedExpressionKind::MemberAccess {
                                    base: Box::new(typed_base),
                                    field_name: field,
                                    field_index: 0,
                                },
                                Type::Void,
                            )
                        }
                    } else {
                        if !matches!(typed_base.ty, Type::Unknown) {
                            self.report_error(
                                typed_base.span.clone(),
                                format!("Unknown struct type '{}'.", struct_name),
                            );
                        }
                        (
                            TypedExpressionKind::MemberAccess {
                                base: Box::new(typed_base),
                                field_name: field,
                                field_index: 0,
                            },
                            Type::Unknown,
                        )
                    }
                } else {
                    if !matches!(typed_base.ty, Type::Unknown) {
                        self.report_error(
                            typed_base.span.clone(),
                            format!(
                                "Cannot access member '{}' of non-struct type {:?}.", field,
                                typed_base.ty
                            ),
                        );
                    }
                    (
                        TypedExpressionKind::MemberAccess {
                            base: Box::new(typed_base),
                            field_name: field,
                            field_index: 0,
                        },
                        Type::Unknown,
                    )
                }
            }
            Expression::StructLiteral { name, fields } => {
                if let Some(info) = self.symbols.lookup_struct(&name).cloned() {
                    let mut typed_fields = Vec::new();
                    let mut initialized = HashSet::new();
                    for (field_name, field_expr) in fields {
                        if let Some((expected_ty, field_idx)) = info
                            .fields
                            .get(&field_name)
                        {
                            let typed_expr = self.analyze_expression(field_expr);
                            if !self.types_compatible(expected_ty, &typed_expr.ty) {
                                self.report_error(
                                    typed_expr.span.clone(),
                                    format!(
                                        "Type mismatch for field '{}' in struct '{}'. Expected {:?}, found {:?}.",
                                        field_name, name, expected_ty, typed_expr.ty
                                    ),
                                );
                            }
                            typed_fields
                                .push((field_name.clone(), *field_idx, typed_expr));
                            initialized.insert(field_name);
                        } else {
                            self.report_error(
                                expr.span.clone(),
                                format!(
                                    "Struct '{}' has no field named '{}'.", name, field_name
                                ),
                            );
                        }
                    }
                    for (required_field, _) in &info.ordered_fields {
                        if !initialized.contains(required_field) {
                            self.report_error(
                                expr.span.clone(),
                                format!(
                                    "Missing field '{}' in literal for struct '{}'.",
                                    required_field, name
                                ),
                            );
                        }
                    }
                    (
                        TypedExpressionKind::StructLiteral {
                            name: name.clone(),
                            fields: typed_fields,
                        },
                        Type::Struct(name),
                    )
                } else {
                    (
                        TypedExpressionKind::StructLiteral {
                            name: name.clone(),
                            fields: Vec::new(),
                        },
                        Type::Unknown,
                    )
                }
            }
            Expression::UnaryNot(inner) => {
                let typed_inner = self.analyze_expression(*inner);
                let typed_zero = if self.types_compatible(&Type::Bool, &typed_inner.ty) {
                    TypedExpression {
                        node: TypedExpressionKind::BoolLiteral(false),
                        ty: Type::Bool,
                        span: expr.span.clone(),
                    }
                } else {
                    TypedExpression {
                        node: TypedExpressionKind::IntLiteral(0),
                        ty: Type::Int,
                        span: expr.span.clone(),
                    }
                };
                (
                    TypedExpressionKind::BinaryOperation {
                        op: BinaryOp::Eq,
                        left: Box::new(typed_inner),
                        right: Box::new(typed_zero),
                    },
                    Type::Bool,
                )
            }
            Expression::Match { scrutinee, arms } => {
                let typed_scrutinee = self.analyze_expression(*scrutinee);
                let mut typed_arms: FVec<(MatchPattern, Option<TypedExpression>, TypedExpression)> = Vec::new();
                let mut result_ty: Option<Type> = None;
                for arm in arms {
                    let guard = arm.guard.map(|g| self.analyze_expression(*g));
                    let typed_body = self.analyze_expression(arm.body);
                    if let Some(ref expected) = result_ty {
                        if !self.types_compatible(expected, &typed_body.ty) {
                            self.report_error(
                                typed_body.span.clone(),
                                format!("Match arm type mismatch: expected {:?}, found {:?}.", expected, typed_body.ty),
                            );
                        }
                    } else {
                        result_ty = Some(typed_body.ty.clone());
                    }
                    typed_arms.push((arm.pattern, guard, typed_body));
                }
                let final_ty = result_ty.unwrap_or(Type::Void);
                (
                    TypedExpressionKind::Match {
                        scrutinee: Box::new(typed_scrutinee),
                        arms: typed_arms,
                    },
                    final_ty,
                )
            }
            Expression::Lambda { params, body, captures } => {
                self.symbols.push_scope();
                let mut typed_params: FVec<(FString, Type)> = Vec::new();
                for (name, ty) in &params {
                    let norm_ty = self.normalise_type(ty);
                    if let Err(msg) = self.symbols.insert_var(name.clone(), norm_ty.clone()) {
                        self.report_error(expr.span.clone(), msg);
                    }
                    typed_params.push((name.clone(), norm_ty));
                }
                let typed_body = self.analyze_expression(*body);
                self.symbols.pop_scope();
                let mut typed_captures: FVec<(FString, Type)> = Vec::new();
                for cap_name in captures {
                    if let Some(cap_ty) = self.symbols.lookup_var(&cap_name) {
                        typed_captures.push((cap_name, cap_ty.clone()));
                    }
                }
                let closure_ty = Type::Closure(
                    typed_params.iter().map(|(_, t)| t.clone()).collect(),
                    Box::new(typed_body.ty.clone()),
                );
                (
                    TypedExpressionKind::Lambda {
                        params: typed_params,
                        body: Box::new(typed_body),
                        captures: typed_captures,
                    },
                    closure_ty,
                )
            }
            Expression::Slice { array, start, end } => {
                let typed_array = self.analyze_expression(*array);
                let mut typed_start: Option<Box<TypedExpression>> = None;
                let mut typed_end: Option<Box<TypedExpression>> = None;
                if let Some(s) = start {
                    let ts = self.analyze_expression(*s);
                    if !self.types_compatible(&Type::Int, &ts.ty) {
                        self.report_error(ts.span.clone(), "Slice start index must be Int.".to_string());
                    }
                    typed_start = Some(Box::new(ts));
                }
                if let Some(e) = end {
                    let te = self.analyze_expression(*e);
                    if !self.types_compatible(&Type::Int, &te.ty) {
                        self.report_error(te.span.clone(), "Slice end index must be Int.".to_string());
                    }
                    typed_end = Some(Box::new(te));
                }
                let elem_ty = match &typed_array.ty {
                    Type::Array(elem, _) => *elem.clone(),
                    _ => Type::Unknown,
                };
                let slice_ty = Type::Slice(Box::new(elem_ty));
                (
                    TypedExpressionKind::Slice {
                        array: Box::new(typed_array),
                        start: typed_start,
                        end: typed_end,
                    },
                    slice_ty,
                )
            }
            Expression::AddressOf(inner) => {
                let typed_inner = self.analyze_expression(*inner);
                let is_lvalue = matches!(
                    typed_inner.node, TypedExpressionKind::Variable(_) |
                    TypedExpressionKind::Dereference(_) | TypedExpressionKind::Index { ..
                    } | TypedExpressionKind::MemberAccess { .. }
                );
                if !is_lvalue {
                    self.report_error(
                        expr.span.clone(),
                        "Cannot take the address of a temporary value.".to_string(),
                    );
                }
                let ptr_ty = Type::Pointer(Box::new(typed_inner.ty.clone()));
                (TypedExpressionKind::AddressOf(Box::new(typed_inner)), ptr_ty)
            }
            Expression::Dereference(inner) => {
                let typed_inner = self.analyze_expression(*inner);
                match typed_inner.ty.clone() {
                    Type::Pointer(inner_ty) => {
                        (
                            TypedExpressionKind::Dereference(Box::new(typed_inner)),
                            *inner_ty,
                        )
                    }
                    other => {
                        self.report_error(
                            expr.span.clone(),
                            format!("Cannot dereference non-pointer type {:?}.", other),
                        );
                        (
                            TypedExpressionKind::Dereference(Box::new(typed_inner)),
                            Type::Void,
                        )
                    }
                }
            }
        };
        TypedExpression {
            node,
            ty,
            span: expr.span,
        }
    }
}

/// In-process parser+sema status helper for stage1 API wiring.
///
/// Returns:
/// - 0: parse+sema succeeded
/// - 3: parse produced diagnostics
/// - 4: parse produced no program
/// - 5: semantic analysis produced diagnostics
/// - 6: semantic analysis produced no typed program
pub fn analyze_source_status(input: FString) -> FInt {
    let parse_output = crate::parser::parse_output(&input);
    if parse_output.errors.len() > 0 {
        return 3;
    }
    if parse_output.program.is_none() {
        return 4;
    }

    let program = parse_output.program.unwrap();
    let analyzer = Analyzer::new();
    let sema_output = analyzer.analyze_output(program);
    if sema_output.errors.len() > 0 {
        return 5;
    }
    if sema_output.program.is_none() {
        return 6;
    }
    return 0;
}
```

---

## 3. Vortex Borrow Checker (vortex.fu)

**File:** `crates/fuc/src/vortex.fu`  
**Lines:** 38  
**Purpose:** Self-hosted borrow checker with entropic flow safety model

```fusion
// Fusion Self-Hosted Vortex Engine
// Implements the Entropic Flow safety model to prevent data races and use-after-free.

import std.mem;
import std.io;

struct Loan {
    target: string;
    is_exclusive: bool;
    origin_line: int;
}

struct ChaosVacuum {
    reports: **char;
    count: int;
}

/// Analyses the entropic state of a variable borrow.
fn verify_borrow(state: *LoanStream, target: string, exclusive: bool, line: int) -> bool {
    let i: int = 0;
    while (i < state.loan_count) {
        let existing: *Loan = &state.loans[i];
        if (existing.target == target) {
            // Collision Logic: Stream B (Mutable) repels everything.
            // Stream A (Immutable) only repels Stream B.
            if (existing.is_exclusive || exclusive) {
                io.print("Entropic Collision: Variable '");
                io.print(target);
                io.print("' has conflicting streams at line ");
                io.print_int(line);
                io.print("\n");
                return false;
            }
        }
        i = i + 1;
    }
    return true;
}
```

---

## Summary

This document provides complete source code for three essential Fusion compiler components:

### Parser (1,488 lines)
- Transforms token streams into AST using Chumsky parser combinators
- Supports: types, expressions, statements, functions, structs, enums, traits, impl blocks
- Features: generics, variadic functions, lambdas, match expressions, slices

### Semantic Analyzer (1,576 lines)
- Type checking and validation with symbol table management
- Type normalization (FInt → Int, FString → String, etc.)
- ABI validation for extern functions and aggregate types
- Recursive type detection and struct field validation
- Generates typed AST for IR lowering

### Vortex Engine (38 lines)
- Self-hosted borrow checker implementing entropic flow model
- Loan tracking with exclusive vs shared borrows
- Collision detection for conflicting access patterns
- Thermodynamic safety principles

**Total:** 3,102 lines of compiler source code

---

*Document generated: 2026-07-02*  
*Fusion v2.0 Vortex Bootstrap Compiler*
