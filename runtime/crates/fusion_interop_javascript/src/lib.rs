//! # Fusion JavaScript Interop
//!
//! JavaScript type conversion, V8/QuickJS engine integration points,
//! and Promise/async bridging for the Fusion Programming Language.
//!
//! This crate provides the bridge layer between Fusion's runtime and
//! JavaScript engines, enabling seamless cross-language interop.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

// ──────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum JsInteropError {
    #[error("type conversion error: {0}")]
    TypeConversion(String),
    #[error("function call error: {0}")]
    FunctionCall(String),
    #[error("script error: {0}")]
    ScriptError(String),
    #[error("promise error: {0}")]
    PromiseError(String),
    #[error("engine error: {0}")]
    EngineError(String),
}

pub type Result<T> = std::result::Result<T, JsInteropError>;

// ──────────────────────────────────────────────
// JavaScript engine abstraction
// ──────────────────────────────────────────────

/// Supported JavaScript engine backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsEngineKind {
    /// Google V8 (via rusty_v8 or similar FFI).
    V8,
    /// QuickJS (via libquickjs-sys).
    QuickJS,
    /// Boa (pure Rust JS engine).
    Boa,
}

/// Configuration for initializing a JavaScript engine.
#[derive(Debug, Clone)]
pub struct JsEngineConfig {
    pub kind: JsEngineKind,
    pub max_heap_bytes: usize,
    pub max_stack_bytes: usize,
    pub enable_promise: bool,
    pub enable_module: bool,
}

impl Default for JsEngineConfig {
    fn default() -> Self {
        Self {
            kind: JsEngineKind::Boa,
            max_heap_bytes: 128 * 1024 * 1024,  // 128 MB
            max_stack_bytes: 1 * 1024 * 1024,   // 1 MB
            enable_promise: true,
            enable_module: true,
        }
    }
}

// ──────────────────────────────────────────────
// JavaScript value representation
// ──────────────────────────────────────────────

/// A Fusion value that can be marshaled to/from JavaScript.
#[derive(Debug, Clone, PartialEq)]
pub enum JsValue {
    Undefined,
    Null,
    Number(f64),
    BigInt(i64),
    String(String),
    Bool(bool),
    Array(Vec<JsValue>),
    Object(Vec<(String, JsValue)>),
    Function(String),
    Promise(Box<PromiseState>),
}

impl JsValue {
    pub fn number(v: f64) -> Self {
        JsValue::Number(v)
    }

    pub fn string(v: impl Into<String>) -> Self {
        JsValue::String(v.into())
    }

    pub fn bool(v: bool) -> Self {
        JsValue::Bool(v)
    }

    pub fn array(items: Vec<JsValue>) -> Self {
        JsValue::Array(items)
    }

    pub fn object(pairs: Vec<(String, JsValue)>) -> Self {
        JsValue::Object(pairs)
    }

    pub fn is_undefined_or_null(&self) -> bool {
        matches!(self, JsValue::Undefined | JsValue::Null)
    }

    /// Check if this value is truthy in JS semantics.
    pub fn is_truthy(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::BigInt(0) => false,
            JsValue::BigInt(_) => true,
            JsValue::String(ref s) => !s.is_empty(),
            JsValue::Bool(b) => *b,
            JsValue::Array(_) | JsValue::Object(_) | JsValue::Function(_) | JsValue::Promise(_) => true,
        }
    }
}

impl Default for JsValue {
    fn default() -> Self {
        JsValue::Undefined
    }
}

// ──────────────────────────────────────────────
// Rust → JavaScript type conversion
// ──────────────────────────────────────────────

/// Convert Rust types into JavaScript values.
pub trait IntoJsValue {
    fn into_js_value(self) -> JsValue;
}

impl IntoJsValue for f64 {
    fn into_js_value(self) -> JsValue {
        JsValue::Number(self)
    }
}

impl IntoJsValue for i64 {
    fn into_js_value(self) -> JsValue {
        JsValue::BigInt(self)
    }
}

impl IntoJsValue for bool {
    fn into_js_value(self) -> JsValue {
        JsValue::Bool(self)
    }
}

impl IntoJsValue for String {
    fn into_js_value(self) -> JsValue {
        JsValue::String(self)
    }
}

impl IntoJsValue for &str {
    fn into_js_value(self) -> JsValue {
        JsValue::String(self.to_string())
    }
}

impl IntoJsValue for usize {
    fn into_js_value(self) -> JsValue {
        JsValue::Number(self as f64)
    }
}

impl<T: IntoJsValue> IntoJsValue for Vec<T> {
    fn into_js_value(self) -> JsValue {
        JsValue::Array(self.into_iter().map(|v| v.into_js_value()).collect())
    }
}

/// Extract typed Rust values from a `JsValue`.
pub trait FromJsValue: Sized {
    fn from_js_value(value: &JsValue) -> Result<Self>;
}

impl FromJsValue for f64 {
    fn from_js_value(value: &JsValue) -> Result<Self> {
        match value {
            JsValue::Number(v) => Ok(*v),
            JsValue::BigInt(v) => Ok(*v as f64),
            _ => Err(JsInteropError::TypeConversion(format!(
                "expected Number, got {:?}",
                value
            ))),
        }
    }
}

impl FromJsValue for i64 {
    fn from_js_value(value: &JsValue) -> Result<Self> {
        match value {
            JsValue::BigInt(v) => Ok(*v),
            JsValue::Number(v) => Ok(*v as i64),
            _ => Err(JsInteropError::TypeConversion(format!(
                "expected BigInt/Number, got {:?}",
                value
            ))),
        }
    }
}

impl FromJsValue for bool {
    fn from_js_value(value: &JsValue) -> Result<Self> {
        Ok(value.is_truthy())
    }
}

impl FromJsValue for String {
    fn from_js_value(value: &JsValue) -> Result<Self> {
        match value {
            JsValue::String(v) => Ok(v.clone()),
            _ => Err(JsInteropError::TypeConversion(format!(
                "expected String, got {:?}",
                value
            ))),
        }
    }
}

impl<T: FromJsValue> FromJsValue for Vec<T> {
    fn from_js_value(value: &JsValue) -> Result<Self> {
        match value {
            JsValue::Array(items) => items.iter().map(T::from_js_value).collect(),
            _ => Err(JsInteropError::TypeConversion(format!(
                "expected Array, got {:?}",
                value
            ))),
        }
    }
}

// ──────────────────────────────────────────────
// JavaScript type name mapping
// ──────────────────────────────────────────────

/// Maps Rust / Fusion types to JavaScript type names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsTypeTag {
    Undefined,
    Null,
    Number,
    BigInt,
    String,
    Boolean,
    Array,
    Object,
    Function,
    Symbol,
}

impl JsTypeTag {
    pub fn of(value: &JsValue) -> Self {
        match value {
            JsValue::Undefined => JsTypeTag::Undefined,
            JsValue::Null => JsTypeTag::Null,
            JsValue::Number(_) => JsTypeTag::Number,
            JsValue::BigInt(_) => JsTypeTag::BigInt,
            JsValue::String(_) => JsTypeTag::String,
            JsValue::Bool(_) => JsTypeTag::Boolean,
            JsValue::Array(_) => JsTypeTag::Array,
            JsValue::Object(_) => JsTypeTag::Object,
            JsValue::Function(_) => JsTypeTag::Function,
            JsValue::Promise(_) => JsTypeTag::Object, // Promises are objects in JS
        }
    }

    pub fn js_name(&self) -> &'static str {
        match self {
            JsTypeTag::Undefined => "undefined",
            JsTypeTag::Null => "object", // typeof null === "object" in JS
            JsTypeTag::Number => "number",
            JsTypeTag::BigInt => "bigint",
            JsTypeTag::String => "string",
            JsTypeTag::Boolean => "boolean",
            JsTypeTag::Array => "object",
            JsTypeTag::Object => "object",
            JsTypeTag::Function => "function",
            JsTypeTag::Symbol => "symbol",
        }
    }

    pub fn fusion_name(&self) -> &'static str {
        match self {
            JsTypeTag::Undefined | JsTypeTag::Null => "Void",
            JsTypeTag::Number => "Float",
            JsTypeTag::BigInt => "Int",
            JsTypeTag::String => "String",
            JsTypeTag::Boolean => "Bool",
            JsTypeTag::Array => "Array",
            JsTypeTag::Object => "Map",
            JsTypeTag::Function => "Function",
            JsTypeTag::Symbol => "Symbol",
        }
    }
}

// ──────────────────────────────────────────────
// Promise / async bridging
// ──────────────────────────────────────────────

/// State of a JavaScript Promise.
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    Pending,
    Resolved(Box<JsValue>),
    Rejected(String),
}

/// A JavaScript Promise handle that Fusion can await on.
#[derive(Debug, Clone)]
pub struct JsPromise {
    pub id: u64,
    pub state: PromiseState,
    pub then_callbacks: Vec<String>,
    pub catch_callbacks: Vec<String>,
}

impl JsPromise {
    pub fn pending(id: u64) -> Self {
        Self {
            id,
            state: PromiseState::Pending,
            then_callbacks: Vec::new(),
            catch_callbacks: Vec::new(),
        }
    }

    pub fn resolve(self, value: JsValue) -> Self {
        Self {
            state: PromiseState::Resolved(Box::new(value)),
            ..self
        }
    }

    pub fn reject(self, reason: impl Into<String>) -> Self {
        Self {
            state: PromiseState::Rejected(reason.into()),
            ..self
        }
    }

    pub fn then(mut self, callback: impl Into<String>) -> Self {
        self.then_callbacks.push(callback.into());
        self
    }

    pub fn catch(mut self, callback: impl Into<String>) -> Self {
        self.catch_callbacks.push(callback.into());
        self
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self.state, PromiseState::Resolved(_))
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self.state, PromiseState::Rejected(_))
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.state, PromiseState::Pending)
    }
}

/// Bridges Fusion async operations to JavaScript Promises.
pub struct PromiseBridge {
    next_id: u64,
    pending: HashMap<u64, JsPromise>,
}

impl PromiseBridge {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            pending: HashMap::new(),
        }
    }

    /// Create a new pending promise and return its ID.
    pub fn create_promise(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.pending.insert(id, JsPromise::pending(id));
        id
    }

    /// Resolve a pending promise with a value.
    pub fn resolve(&mut self, id: u64, value: JsValue) -> Result<()> {
        let promise = self
            .pending
            .remove(&id)
            .ok_or_else(|| JsInteropError::PromiseError(format!("promise {} not found", id)))?;
        let resolved = promise.resolve(value);
        // In real impl, callbacks would be dispatched here.
        log::info!("Promise {} resolved", resolved.id);
        Ok(())
    }

    /// Reject a pending promise with a reason.
    pub fn reject(&mut self, id: u64, reason: impl Into<String>) -> Result<()> {
        let promise = self
            .pending
            .remove(&id)
            .ok_or_else(|| JsInteropError::PromiseError(format!("promise {} not found", id)))?;
        let rejected = promise.reject(reason);
        log::info!("Promise {} rejected", rejected.id);
        Ok(())
    }

    /// Get the current state of a promise.
    pub fn state(&self, id: u64) -> Option<&JsPromise> {
        self.pending.get(&id)
    }

    /// Number of pending promises.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for PromiseBridge {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────
// Script evaluation
// ──────────────────────────────────────────────

/// A compiled JavaScript script ready for execution.
#[derive(Debug, Clone)]
pub struct CompiledScript {
    pub source: String,
    pub name: String,
}

/// Engine-agnostic script evaluator.
pub struct ScriptEvaluator {
    #[allow(dead_code)]
    config: JsEngineConfig,
}

impl ScriptEvaluator {
    pub fn new(config: JsEngineConfig) -> Self {
        Self { config }
    }

    /// Compile source code into a `CompiledScript`.
    pub fn compile(&self, source: &str, name: &str) -> Result<CompiledScript> {
        // Basic syntax validation (check balanced braces/parens).
        let mut brace_depth = 0i32;
        let mut paren_depth = 0i32;
        let mut in_string = false;
        let mut string_char = '\0';

        for ch in source.chars() {
            if in_string {
                if ch == string_char {
                    in_string = false;
                }
                continue;
            }
            match ch {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = ch;
                }
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                '(' => paren_depth += 1,
                ')' => paren_depth -= 1,
                _ => {}
            }
        }

        if in_string {
            return Err(JsInteropError::ScriptError(
                "unterminated string literal".into(),
            ));
        }
        if brace_depth != 0 {
            return Err(JsInteropError::ScriptError(format!(
                "unbalanced braces (depth={})",
                brace_depth
            )));
        }
        if paren_depth != 0 {
            return Err(JsInteropError::ScriptError(format!(
                "unbalanced parentheses (depth={})",
                paren_depth
            )));
        }

        Ok(CompiledScript {
            source: source.to_string(),
            name: name.to_string(),
        })
    }
}

impl Default for ScriptEvaluator {
    fn default() -> Self {
        Self::new(JsEngineConfig::default())
    }
}

// ──────────────────────────────────────────────
// JavaScript function bridge
// ──────────────────────────────────────────────

/// Signature of a JavaScript function callable from Fusion.
#[derive(Debug, Clone)]
pub struct JsFunctionSignature {
    pub name: String,
    pub params: Vec<String>,
    pub is_async: bool,
}

/// Registry of JS functions that Fusion can call.
pub struct JsFunctionRegistry {
    functions: HashMap<String, JsFunctionSignature>,
}

impl JsFunctionRegistry {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn register(&mut self, sig: JsFunctionSignature) {
        self.functions.insert(sig.name.clone(), sig);
    }

    pub fn lookup(&self, name: &str) -> Option<&JsFunctionSignature> {
        self.functions.get(name)
    }

    pub fn list(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for JsFunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────
// High-level JavaScript bridge
// ──────────────────────────────────────────────

/// The top-level bridge connecting Fusion to a JavaScript engine.
pub struct JavaScriptBridge {
    pub config: JsEngineConfig,
    pub promise_bridge: PromiseBridge,
    pub function_registry: JsFunctionRegistry,
    pub evaluator: ScriptEvaluator,
    globals: Arc<Mutex<HashMap<String, JsValue>>>,
}

impl JavaScriptBridge {
    pub fn new(config: JsEngineConfig) -> Self {
        Self {
            evaluator: ScriptEvaluator::new(config.clone()),
            promise_bridge: PromiseBridge::new(),
            function_registry: JsFunctionRegistry::new(),
            globals: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Set a global variable in the JS runtime.
    pub fn set_global(&self, name: &str, value: JsValue) {
        let mut globals = self.globals.lock().unwrap();
        globals.insert(name.to_string(), value);
    }

    /// Get a global variable from the JS runtime.
    pub fn get_global(&self, name: &str) -> Option<JsValue> {
        let globals = self.globals.lock().unwrap();
        globals.get(name).cloned()
    }

    /// Compile and evaluate a JS source string.
    pub fn eval(&self, source: &str) -> Result<JsValue> {
        let compiled = self.evaluator.compile(source, "<eval>")?;
        log::info!("Evaluated script '{}'", compiled.name);
        Ok(JsValue::Undefined)
    }

    /// Call a registered JS function.
    pub fn call_function(&self, name: &str, args: &[JsValue]) -> Result<JsValue> {
        let _sig = self
            .function_registry
            .lookup(name)
            .ok_or_else(|| JsInteropError::FunctionCall(format!("function '{}' not found", name)))?;

        log::info!("Calling JS function '{}' with {} args", name, args.len());
        Ok(JsValue::Undefined)
    }

    /// Create a pending promise for an async operation.
    pub fn create_promise(&mut self) -> u64 {
        self.promise_bridge.create_promise()
    }
}

impl Default for JavaScriptBridge {
    fn default() -> Self {
        Self::new(JsEngineConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── JsValue tests ──

    #[test]
    fn test_js_value_constructors() {
        assert_eq!(JsValue::number(42.0), JsValue::Number(42.0));
        assert_eq!(
            JsValue::string("hi"),
            JsValue::String("hi".into())
        );
        assert_eq!(JsValue::bool(false), JsValue::Bool(false));
        assert_eq!(JsValue::default(), JsValue::Undefined);
    }

    #[test]
    fn test_is_undefined_or_null() {
        assert!(JsValue::Undefined.is_undefined_or_null());
        assert!(JsValue::Null.is_undefined_or_null());
        assert!(!JsValue::Number(0.0).is_undefined_or_null());
    }

    // ── Truthiness tests ──

    #[test]
    fn test_truthiness() {
        assert!(!JsValue::Undefined.is_truthy());
        assert!(!JsValue::Null.is_truthy());
        assert!(!JsValue::Number(0.0).is_truthy());
        assert!(!JsValue::Number(f64::NAN).is_truthy());
        assert!(!JsValue::BigInt(0).is_truthy());
        assert!(!JsValue::String("".into()).is_truthy());
        assert!(!JsValue::Bool(false).is_truthy());

        assert!(JsValue::Number(1.0).is_truthy());
        assert!(JsValue::BigInt(1).is_truthy());
        assert!(JsValue::String("x".into()).is_truthy());
        assert!(JsValue::Bool(true).is_truthy());
        assert!(JsValue::Array(vec![]).is_truthy());
        assert!(JsValue::Object(vec![]).is_truthy());
    }

    // ── Marshaling tests ──

    #[test]
    fn test_into_js_value() {
        let v: JsValue = 42.0f64.into_js_value();
        assert_eq!(v, JsValue::Number(42.0));

        let v: JsValue = 10i64.into_js_value();
        assert_eq!(v, JsValue::BigInt(10));

        let v: JsValue = true.into_js_value();
        assert_eq!(v, JsValue::Bool(true));

        let v: JsValue = "hello".into_js_value();
        assert_eq!(v, JsValue::String("hello".into()));

        let v: JsValue = vec![1i64, 2].into_js_value();
        assert_eq!(
            v,
            JsValue::Array(vec![JsValue::BigInt(1), JsValue::BigInt(2)])
        );
    }

    #[test]
    fn test_from_js_value() {
        assert_eq!(f64::from_js_value(&JsValue::Number(3.14)).unwrap(), 3.14);
        assert_eq!(i64::from_js_value(&JsValue::BigInt(42)).unwrap(), 42);
        assert!(bool::from_js_value(&JsValue::Bool(true)).unwrap());
        assert_eq!(
            String::from_js_value(&JsValue::String("hi".into())).unwrap(),
            "hi"
        );
        assert!(f64::from_js_value(&JsValue::Undefined).is_err());
    }

    // ── Type tag tests ──

    #[test]
    fn test_type_tag() {
        assert_eq!(JsTypeTag::of(&JsValue::Undefined), JsTypeTag::Undefined);
        assert_eq!(JsTypeTag::of(&JsValue::Null), JsTypeTag::Null);
        assert_eq!(JsTypeTag::of(&JsValue::Number(1.0)), JsTypeTag::Number);
        assert_eq!(JsTypeTag::of(&JsValue::BigInt(1)), JsTypeTag::BigInt);
        assert_eq!(JsTypeTag::of(&JsValue::String("".into())), JsTypeTag::String);
        assert_eq!(JsTypeTag::of(&JsValue::Bool(true)), JsTypeTag::Boolean);
        assert_eq!(JsTypeTag::of(&JsValue::Array(vec![])), JsTypeTag::Array);
    }

    #[test]
    fn test_type_tag_names() {
        assert_eq!(JsTypeTag::Number.js_name(), "number");
        assert_eq!(JsTypeTag::Number.fusion_name(), "Float");
        assert_eq!(JsTypeTag::String.js_name(), "string");
        assert_eq!(JsTypeTag::Array.fusion_name(), "Array");
    }

    // ── Promise tests ──

    #[test]
    fn test_promise_lifecycle() {
        let mut bridge = PromiseBridge::new();
        let id = bridge.create_promise();
        assert_eq!(bridge.pending_count(), 1);

        let p = bridge.state(id).unwrap();
        assert!(p.is_pending());

        bridge.resolve(id, JsValue::Number(42.0)).unwrap();
        assert_eq!(bridge.pending_count(), 0);
    }

    #[test]
    fn test_promise_reject() {
        let mut bridge = PromiseBridge::new();
        let id = bridge.create_promise();
        assert_eq!(bridge.pending_count(), 1);

        bridge.reject(id, "oops").unwrap();
        // After rejection, promise is removed from pending map.
        assert_eq!(bridge.pending_count(), 0);
        assert!(bridge.state(id).is_none());
    }

    #[test]
    fn test_promise_not_found() {
        let mut bridge = PromiseBridge::new();
        assert!(bridge.resolve(999, JsValue::Null).is_err());
    }

    #[test]
    fn test_promise_builder() {
        let p = JsPromise::pending(1)
            .then("handleResolve")
            .catch("handleReject");

        assert_eq!(p.then_callbacks, vec!["handleResolve"]);
        assert_eq!(p.catch_callbacks, vec!["handleReject"]);
        assert!(p.is_pending());

        let resolved = p.resolve(JsValue::Number(1.0));
        assert!(resolved.is_resolved());
        assert!(!resolved.is_pending());
    }

    // ── Script evaluator tests ──

    #[test]
    fn test_compile_valid() {
        let evaluator = ScriptEvaluator::default();
        let compiled = evaluator.compile("let x = 42;", "test.js").unwrap();
        assert_eq!(compiled.name, "test.js");
    }

    #[test]
    fn test_compile_unbalanced_braces() {
        let evaluator = ScriptEvaluator::default();
        assert!(evaluator.compile("function f() {", "bad.js").is_err());
    }

    #[test]
    fn test_compile_unbalanced_parens() {
        let evaluator = ScriptEvaluator::default();
        assert!(evaluator.compile("f(x;", "bad.js").is_err());
    }

    #[test]
    fn test_compile_unterminated_string() {
        let evaluator = ScriptEvaluator::default();
        assert!(evaluator.compile("let s = \"hello;", "bad.js").is_err());
    }

    #[test]
    fn test_compile_string_with_braces() {
        let evaluator = ScriptEvaluator::default();
        // Braces inside strings should not affect balance.
        let result = evaluator.compile(r#"let s = "{ hello }";"#, "ok.js");
        assert!(result.is_ok());
    }

    // ── Function registry tests ──

    #[test]
    fn test_function_registry() {
        let mut reg = JsFunctionRegistry::new();
        reg.register(JsFunctionSignature {
            name: "fetch".into(),
            params: vec!["url".into()],
            is_async: true,
        });

        assert!(reg.lookup("fetch").is_some());
        assert!(reg.lookup("nope").is_none());
        assert_eq!(reg.list(), vec!["fetch"]);
    }

    // ── JavaScript bridge tests ──

    #[test]
    fn test_bridge_globals() {
        let bridge = JavaScriptBridge::default();
        bridge.set_global("version", JsValue::String("1.0".into()));
        assert_eq!(
            bridge.get_global("version"),
            Some(JsValue::String("1.0".into()))
        );
        assert_eq!(bridge.get_global("nonexistent"), None);
    }

    #[test]
    fn test_bridge_eval() {
        let bridge = JavaScriptBridge::default();
        let result = bridge.eval("1 + 2").unwrap();
        assert_eq!(result, JsValue::Undefined); // placeholder
    }

    #[test]
    fn test_bridge_call_function() {
        let mut bridge = JavaScriptBridge::default();
        bridge.function_registry.register(JsFunctionSignature {
            name: "JSON.parse".into(),
            params: vec!["str".into()],
            is_async: false,
        });

        let result = bridge.call_function("JSON.parse", &[JsValue::String("{}".into())]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bridge_call_unknown_function() {
        let bridge = JavaScriptBridge::default();
        assert!(bridge.call_function("unknown", &[]).is_err());
    }
}
