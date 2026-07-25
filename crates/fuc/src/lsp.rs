//! Language Server Protocol (LSP) Implementation for Fusion.
//! Allows editors like VS Code to display real-time errors, types, and completion.

use crate::ast::{self, Declaration, Type};
use crate::ir;
use crate::parser;
use crate::sema;
use crate::types::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

pub struct LanguageServer {
    documents: FMap<FString, FString>,
    #[allow(dead_code)]
    next_id: u64,
}

impl LanguageServer {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            next_id: 1,
        }
    }

    /// Main event loop reading JSON-RPC from stdin
    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut stdout = io::stdout();

        loop {
            match read_message(&mut reader) {
                Ok(Some(msg)) => {
                    let response = self.handle_message(&msg);
                    if let Some(resp) = response {
                        let json_str = serde_json::to_string(&resp).unwrap_or_default();
                        write_message(&mut stdout, &json_str);
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    eprintln!("[LSP] Error reading message: {}", e);
                    break;
                }
            }
        }
    }
}

/// Read a single LSP message from the reader (Content-Length framing).
fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim();

        if line.is_empty() {
            break;
        }

        if let Some(val) = line.strip_prefix("Content-Length: ") {
            content_length = val.parse().ok();
        }
    }

    let len = match content_length {
        Some(l) => l,
        None => return Ok(None),
    };

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;

    let msg: Value = serde_json::from_slice(&buf).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("JSON parse error: {}", e))
    })?;

    Ok(Some(msg))
}

/// Write an LSP message to the writer (Content-Length framing).
fn write_message(writer: &mut impl Write, json: &str) {
    let msg = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
    let _ = writer.write_all(msg.as_bytes());
    let _ = writer.flush();
}

impl LanguageServer {
    fn handle_message(&mut self, msg: &Value) -> Option<Value> {
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = msg.get("id");

        match method {
            "initialize" => {
                let id_val = id.cloned().unwrap_or(json!(1));
                Some(self.handle_initialize(id_val))
            }
            "textDocument/didOpen" => {
                self.handle_did_open(msg);
                None
            }
            "textDocument/didChange" => {
                self.handle_did_change(msg);
                None
            }
            "textDocument/hover" => {
                let id_val = id.cloned().unwrap_or(json!(0));
                Some(self.handle_hover(msg, id_val))
            }
            "textDocument/definition" => {
                let id_val = id.cloned().unwrap_or(json!(0));
                Some(self.handle_definition(msg, id_val))
            }
            "shutdown" => {
                let id_val = id.cloned().unwrap_or(json!(0));
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id_val,
                    "result": null
                }))
            }
            "exit" => {
                std::process::exit(0);
            }
            _ => None,
        }
    }

    fn handle_initialize(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "capabilities": {
                    "textDocumentSync": 1,
                    "hoverProvider": true,
                    "definitionProvider": true,
                    "diagnosticProvider": {
                        "interFileDependencies": false,
                        "workspaceDiagnostics": false
                    }
                }
            }
        })
    }

    fn handle_did_open(&mut self, msg: &Value) {
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let text_document = params.get("textDocument").cloned().unwrap_or(json!({}));
        let uri = text_document.get("uri").and_then(|u| u.as_str()).unwrap_or("");
        let text = text_document.get("text").and_then(|t| t.as_str()).unwrap_or("");

        self.documents.insert(uri.to_string(), text.to_string());
        self.publish_diagnostics(uri.to_string(), text.to_string());
    }

    fn handle_did_change(&mut self, msg: &Value) {
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let text_document = params.get("textDocument").cloned().unwrap_or(json!({}));
        let uri = text_document.get("uri").and_then(|u| u.as_str()).unwrap_or("");

        // Get the full text from changes
        let text = if let Some(changes) = params.get("contentChanges").and_then(|c| c.as_array()) {
            if let Some(last) = changes.last() {
                last.get("text").and_then(|t| t.as_str()).unwrap_or("")
            } else {
                ""
            }
        } else {
            ""
        };

        self.documents.insert(uri.to_string(), text.to_string());
        self.publish_diagnostics(uri.to_string(), text.to_string());
    }

    fn publish_diagnostics(&self, uri: FString, source: FString) {
        let parse_out = parser::parse_output(&source);
        let mut diagnostics: Vec<Value> = Vec::new();

        // Convert source to lines for position mapping
        let lines: Vec<&str> = source.lines().collect();

        for err in &parse_out.errors {
            let (start_line, start_char) = find_error_position(&lines, err);
            diagnostics.push(json!({
                "range": {
                    "start": { "line": start_line, "character": start_char },
                    "end": { "line": start_line, "character": start_char + 1 }
                },
                "severity": 1,
                "message": err
            }));
        }

        if let Some(prog) = parse_out.program {
            let mut analyzer = sema::Analyzer::new();
            let sema_out = analyzer.analyze_output(prog);

            for err in &sema_out.errors {
                let (start_line, start_char) = find_error_position(&lines, err);
                diagnostics.push(json!({
                    "range": {
                        "start": { "line": start_line, "character": start_char },
                        "end": { "line": start_line, "character": start_char + 1 }
                    },
                    "severity": 1,
                    "message": err
                }));
            }
        }

        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": uri,
                "diagnostics": diagnostics
            }
        });

        // We can't easily return a notification from this method,
        // so we send it directly. In a production LSP, you'd queue it.
        let json_str = serde_json::to_string(&notification).unwrap_or_default();
        let mut stdout = io::stdout();
        write_message(&mut stdout, &json_str);
    }

    fn handle_hover(&self, msg: &Value, id: Value) -> Value {
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let text_document = params.get("textDocument").cloned().unwrap_or(json!({}));
        let uri = text_document.get("uri").and_then(|u| u.as_str()).unwrap_or("");
        let position = params.get("position").cloned().unwrap_or(json!({}));

        let line = position.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as usize;
        let character = position.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as usize;

        if let Some(source) = self.documents.get(uri) {
            let symbol = find_symbol_at_position(source, line, character);
            if let Some(sym_name) = symbol {
                let parse_out = parser::parse_output(source);

                // First, check AST declarations (works without sema)
                if let Some(ref prog) = parse_out.program {
                    let hover_text = find_type_in_ast(prog, &sym_name);
                    if let Some(text) = hover_text {
                        return json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "contents": {
                                    "kind": "markdown",
                                    "value": format!("```fusion\n{}\n```", text)
                                }
                            }
                        });
                    }
                }

                // Then, run sema for richer type info
                if let Some(prog) = parse_out.program {
                    let mut analyzer = sema::Analyzer::new();
                    let sema_out = analyzer.analyze_output(prog);

                    // Look for type info in the typed program
                    if let Some(typed_prog) = &sema_out.program {
                        // Check functions
                        for func in &typed_prog.functions {
                            if func.name == sym_name {
                                let params_str: Vec<String> = func.params.iter()
                                    .map(|(n, t)| format!("{}: {}", n, type_to_string(t)))
                                    .collect();
                                let hover_text = format!(
                                    "fn {}({}) -> {}",
                                    func.name,
                                    params_str.join(", "),
                                    type_to_string(&func.return_type)
                                );
                                return json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "contents": {
                                            "kind": "markdown",
                                            "value": format!("```fusion\n{}\n```", hover_text)
                                        }
                                    }
                                });
                            }
                        }

                        // Check struct fields
                        for s in &typed_prog.structs {
                            if s.name == sym_name {
                                let fields_str: Vec<String> = s.fields.iter()
                                    .map(|(n, t)| format!("{}: {}", n, type_to_string(t)))
                                    .collect();
                                let hover_text = format!(
                                    "struct {} {{\n  {}\n}}",
                                    s.name,
                                    fields_str.join("\n  ")
                                );
                                return json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "contents": {
                                            "kind": "markdown",
                                            "value": format!("```fusion\n{}\n```", hover_text)
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
            }
        }

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": null
        })
    }

    fn handle_definition(&self, msg: &Value, id: Value) -> Value {
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let text_document = params.get("textDocument").cloned().unwrap_or(json!({}));
        let uri = text_document.get("uri").and_then(|u| u.as_str()).unwrap_or("");
        let position = params.get("position").cloned().unwrap_or(json!({}));

        let line = position.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as usize;
        let character = position.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as usize;

        if let Some(source) = self.documents.get(uri) {
            let symbol = find_symbol_at_position(source, line, character);
            if let Some(sym_name) = symbol {
                let parse_out = parser::parse_output(source);
                if let Some(prog) = parse_out.program {
                    let lines: Vec<&str> = source.lines().collect();
                    if let Some(def_line) = find_declaration_line(&prog, &sym_name) {
                        let char_offset = find_declaration_char(&lines, def_line, &sym_name);
                        return json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "uri": uri,
                                "range": {
                                    "start": { "line": def_line, "character": char_offset },
                                    "end": { "line": def_line, "character": char_offset + sym_name.len() as u64 }
                                }
                            }
                        });
                    }
                }
            }
        }

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": null
        })
    }
}

/// Find the symbol name at the given cursor position.
fn find_symbol_at_position(source: &str, line: usize, character: usize) -> Option<FString> {
    let lines: Vec<&str> = source.lines().collect();
    if line >= lines.len() {
        return None;
    }

    let target_line = lines[line];
    if character > target_line.len() {
        return None;
    }

    // Find word boundaries
    let byte_pos = target_line.char_indices()
        .nth(character)
        .map(|(i, _)| i)
        .unwrap_or(target_line.len());

    // Scan backward to find start of identifier
    let before = &target_line[..byte_pos];
    let start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    // Scan forward to find end of identifier
    let after = &target_line[byte_pos..];
    let end_offset = after.find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(after.len());
    let end = byte_pos + end_offset;

    let candidate = &target_line[start..end];

    if candidate.is_empty() || !candidate.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') {
        return None;
    }

    Some(candidate.to_string())
}

/// Find error position by scanning source lines for context clues.
fn find_error_position(lines: &[&str], error: &str) -> (u64, u64) {
    // Simple heuristic: try to find which line the error relates to
    // by looking for keywords or patterns mentioned in the error message.
    for (i, line) in lines.iter().enumerate() {
        if error.contains("Expected") {
            // Look for incomplete constructs
            if line.contains("fn ") || line.contains("struct ") || line.contains("let ") {
                return (i as u64, 0);
            }
        }
    }
    (0, 0)
}

/// Convert ir::Type to a human-readable string.
fn type_to_string(ty: &ir::Type) -> String {
    match ty {
        ir::Type::Int => "int".to_string(),
        ir::Type::Bool => "bool".to_string(),
        ir::Type::String => "string".to_string(),
        ir::Type::Void => "void".to_string(),
        ir::Type::Float => "float".to_string(),
        ir::Type::Struct(name) => name.clone(),
        ir::Type::Pointer(inner) => format!("*{}", type_to_string(inner)),
        ir::Type::Array(elem, len) => format!("[{}; {}]", type_to_string(elem), len),
        ir::Type::Slice(inner) => format!("[{}]", type_to_string(inner)),
        ir::Type::GenericParam(name) => name.clone(),
        ir::Type::Closure(params, ret) => {
            let params_str: Vec<String> = params.iter().map(type_to_string).collect();
            format!("({}) -> {}", params_str.join(", "), type_to_string(ret))
        }
        ir::Type::Optional(inner) => format!("{}?", type_to_string(inner)),
        ir::Type::Union(types) => {
            let types_str: Vec<String> = types.iter().map(type_to_string).collect();
            types_str.join(" | ")
        }
        ir::Type::GenericInstance(name, args) => {
            let args_str: Vec<String> = args.iter().map(type_to_string).collect();
            format!("{}<{}>", name, args_str.join(", "))
        }
        ir::Type::Unknown => "unknown".to_string(),
    }
}

/// Find type information from AST declarations as a fallback.
fn find_type_in_ast(prog: &ast::Program, sym_name: &str) -> Option<String> {
    for decl in &prog.declarations {
        match decl {
            Declaration::Function { name, params, return_type, .. } => {
                if name == sym_name {
                    let params_str: Vec<String> = params.iter()
                        .map(|p| format!("{}: {}", p.name, ast_type_to_string(&p.param_type)))
                        .collect();
                    return Some(format!(
                        "fn {}({}) -> {}",
                        name,
                        params_str.join(", "),
                        ast_type_to_string(return_type)
                    ));
                }
            }
            Declaration::StructDefinition(sd) => {
                if sd.name == sym_name {
                    let fields_str: Vec<String> = sd.fields.iter()
                        .map(|(n, t)| format!("{}: {}", n, ast_type_to_string(t)))
                        .collect();
                    return Some(format!(
                        "struct {} {{\n  {}\n}}",
                        sd.name,
                        fields_str.join("\n  ")
                    ));
                }
            }
            Declaration::ExternFunction { name, params, return_type, calling_convention } => {
                if name == sym_name {
                    let params_str: Vec<String> = params.iter()
                        .map(|p| format!("{}: {}", p.name, ast_type_to_string(&p.param_type)))
                        .collect();
                    return Some(format!(
                        "extern \"{}\" fn {}({}) -> {}",
                        calling_convention,
                        name,
                        params_str.join(", "),
                        ast_type_to_string(return_type)
                    ));
                }
            }
            _ => {}
        }
    }

    // Check standalone function list
    for func in &prog.functions {
        if func.name == sym_name {
            let params_str: Vec<String> = func.params.iter()
                .map(|p| format!("{}: {}", p.name, ast_type_to_string(&p.param_type)))
                .collect();
            return Some(format!(
                "fn {}({}) -> {}",
                func.name,
                params_str.join(", "),
                ast_type_to_string(&func.return_type)
            ));
        }
    }

    None
}

/// Convert ast::Type to a human-readable string.
fn ast_type_to_string(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::Bool => "bool".to_string(),
        Type::String => "string".to_string(),
        Type::Void => "void".to_string(),
        Type::Unknown => "unknown".to_string(),
        Type::Float => "float".to_string(),
        Type::Pointer(inner) => format!("*{}", ast_type_to_string(inner)),
        Type::Array(elem, len) => format!("[{}; {}]", ast_type_to_string(elem), len),
        Type::Struct(name) => name.clone(),
        Type::GenericParam(name) => name.clone(),
        Type::Slice(inner) => format!("[{}]", ast_type_to_string(inner)),
        Type::Closure(params, ret) => {
            let params_str: Vec<String> = params.iter().map(ast_type_to_string).collect();
            format!("({}) -> {}", params_str.join(", "), ast_type_to_string(ret))
        }
        Type::Optional(inner) => format!("{}?", ast_type_to_string(inner)),
        Type::Union(types) => {
            let types_str: Vec<String> = types.iter().map(ast_type_to_string).collect();
            types_str.join(" | ")
        }
        Type::GenericInstance(name, args) => {
            let args_str: Vec<String> = args.iter().map(ast_type_to_string).collect();
            format!("{}<{}>", name, args_str.join(", "))
        }
    }
}

/// Find the line number where a symbol is declared in the AST.
fn find_declaration_line(prog: &ast::Program, sym_name: &str) -> Option<usize> {
    for decl in &prog.declarations {
        match decl {
            Declaration::Function { name, .. } if name == sym_name => return Some(0),
            Declaration::ExternFunction { name, .. } if name == sym_name => return Some(0),
            Declaration::StructDefinition(sd) if sd.name == sym_name => return Some(0),
            _ => {}
        }
    }

    for func in &prog.functions {
        if func.name == sym_name {
            return Some(0);
        }
    }

    None
}

/// Find the character offset of a symbol name on a given line.
fn find_declaration_char(lines: &[&str], line: usize, sym_name: &str) -> u64 {
    if line < lines.len() {
        if let Some(pos) = lines[line].find(sym_name) {
            return pos as u64;
        }
    }
    0
}
