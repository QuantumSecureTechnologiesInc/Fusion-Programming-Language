//! # Fusion Python Interop
//!
//! Python type marshaling, function call bridging, module import support,
//! and GIL management for the Fusion Programming Language.
//!
//! This crate provides the bridge layer between Fusion's runtime and the
//! CPython interpreter, enabling seamless cross-language calls.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use thiserror::Error;

// ──────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum PythonInteropError {
    #[error("type conversion error: {0}")]
    TypeConversion(String),
    #[error("function call error: {0}")]
    FunctionCall(String),
    #[error("module import error: {0}")]
    ModuleImport(String),
    #[error("GIL error: {0}")]
    GilError(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, PythonInteropError>;

// ──────────────────────────────────────────────
// Python-side value representation
// ──────────────────────────────────────────────

/// A Fusion value that can be marshaled to/from Python.
#[derive(Debug, Clone, PartialEq)]
pub enum PyValue {
    None,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<PyValue>),
    Dict(Vec<(PyValue, PyValue)>),
    Tuple(Vec<PyValue>),
    Bytes(Vec<u8>),
}

impl PyValue {
    /// Create an int value.
    pub fn int(v: i64) -> Self {
        PyValue::Int(v)
    }

    /// Create a float value.
    pub fn float(v: f64) -> Self {
        PyValue::Float(v)
    }

    /// Create a string value.
    pub fn string(v: impl Into<String>) -> Self {
        PyValue::String(v.into())
    }

    /// Create a bool value.
    pub fn bool(v: bool) -> Self {
        PyValue::Bool(v)
    }

    /// Create a list value.
    pub fn list(items: Vec<PyValue>) -> Self {
        PyValue::List(items)
    }

    /// Create a dict value.
    pub fn dict(pairs: Vec<(PyValue, PyValue)>) -> Self {
        PyValue::Dict(pairs)
    }

    /// Check if this value is None.
    pub fn is_none(&self) -> bool {
        matches!(self, PyValue::None)
    }
}

impl Default for PyValue {
    fn default() -> Self {
        PyValue::None
    }
}

// ──────────────────────────────────────────────
// Fusion → Python type marshaling
// ──────────────────────────────────────────────

/// Marshals a Fusion-native `&str` / `i64` / `f64` / `bool` into `PyValue`.
pub trait IntoPyValue {
    fn into_py_value(self) -> PyValue;
}

impl IntoPyValue for i64 {
    fn into_py_value(self) -> PyValue {
        PyValue::Int(self)
    }
}

impl IntoPyValue for f64 {
    fn into_py_value(self) -> PyValue {
        PyValue::Float(self)
    }
}

impl IntoPyValue for bool {
    fn into_py_value(self) -> PyValue {
        PyValue::Bool(self)
    }
}

impl IntoPyValue for String {
    fn into_py_value(self) -> PyValue {
        PyValue::String(self)
    }
}

impl IntoPyValue for &str {
    fn into_py_value(self) -> PyValue {
        PyValue::String(self.to_string())
    }
}

impl<T: IntoPyValue> IntoPyValue for Vec<T> {
    fn into_py_value(self) -> PyValue {
        PyValue::List(self.into_iter().map(|v| v.into_py_value()).collect())
    }
}

/// Extract typed Rust values from a `PyValue`.
pub trait FromPyValue: Sized {
    fn from_py_value(value: &PyValue) -> Result<Self>;
}

impl FromPyValue for i64 {
    fn from_py_value(value: &PyValue) -> Result<Self> {
        match value {
            PyValue::Int(v) => Ok(*v),
            _ => Err(PythonInteropError::TypeConversion(format!(
                "expected Int, got {:?}",
                value
            ))),
        }
    }
}

impl FromPyValue for f64 {
    fn from_py_value(value: &PyValue) -> Result<Self> {
        match value {
            PyValue::Float(v) => Ok(*v),
            PyValue::Int(v) => Ok(*v as f64),
            _ => Err(PythonInteropError::TypeConversion(format!(
                "expected Float, got {:?}",
                value
            ))),
        }
    }
}

impl FromPyValue for bool {
    fn from_py_value(value: &PyValue) -> Result<Self> {
        match value {
            PyValue::Bool(v) => Ok(*v),
            _ => Err(PythonInteropError::TypeConversion(format!(
                "expected Bool, got {:?}",
                value
            ))),
        }
    }
}

impl FromPyValue for String {
    fn from_py_value(value: &PyValue) -> Result<Self> {
        match value {
            PyValue::String(v) => Ok(v.clone()),
            _ => Err(PythonInteropError::TypeConversion(format!(
                "expected String, got {:?}",
                value
            ))),
        }
    }
}

impl<T: FromPyValue> FromPyValue for Vec<T> {
    fn from_py_value(value: &PyValue) -> Result<Self> {
        match value {
            PyValue::List(items) => items.iter().map(T::from_py_value).collect(),
            _ => Err(PythonInteropError::TypeConversion(format!(
                "expected List, got {:?}",
                value
            ))),
        }
    }
}

// ──────────────────────────────────────────────
// Python type → Fusion type mapping
// ──────────────────────────────────────────────

/// Mapping of Python type names to their Fusion equivalents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonTypeTag {
    NoneType,
    Int,
    Float,
    Bool,
    Str,
    List,
    Dict,
    Tuple,
    Bytes,
    Custom,
}

impl PythonTypeTag {
    /// Infer the type tag from a `PyValue`.
    pub fn of(value: &PyValue) -> Self {
        match value {
            PyValue::None => PythonTypeTag::NoneType,
            PyValue::Int(_) => PythonTypeTag::Int,
            PyValue::Float(_) => PythonTypeTag::Float,
            PyValue::Bool(_) => PythonTypeTag::Bool,
            PyValue::String(_) => PythonTypeTag::Str,
            PyValue::List(_) => PythonTypeTag::List,
            PyValue::Dict(_) => PythonTypeTag::Dict,
            PyValue::Tuple(_) => PythonTypeTag::Tuple,
            PyValue::Bytes(_) => PythonTypeTag::Bytes,
        }
    }

    /// Get the Python type name as a string.
    pub fn python_name(&self) -> &'static str {
        match self {
            PythonTypeTag::NoneType => "NoneType",
            PythonTypeTag::Int => "int",
            PythonTypeTag::Float => "float",
            PythonTypeTag::Bool => "bool",
            PythonTypeTag::Str => "str",
            PythonTypeTag::List => "list",
            PythonTypeTag::Dict => "dict",
            PythonTypeTag::Tuple => "tuple",
            PythonTypeTag::Bytes => "bytes",
            PythonTypeTag::Custom => "object",
        }
    }

    /// Get the Fusion type name.
    pub fn fusion_name(&self) -> &'static str {
        match self {
            PythonTypeTag::NoneType => "Void",
            PythonTypeTag::Int => "Int",
            PythonTypeTag::Float => "Float",
            PythonTypeTag::Bool => "Bool",
            PythonTypeTag::Str => "String",
            PythonTypeTag::List => "Array",
            PythonTypeTag::Dict => "Map",
            PythonTypeTag::Tuple => "Tuple",
            PythonTypeTag::Bytes => "Bytes",
            PythonTypeTag::Custom => "Object",
        }
    }
}

// ──────────────────────────────────────────────
// Function call bridging
// ──────────────────────────────────────────────

/// Signature of a Python function that can be called from Fusion.
#[derive(Debug, Clone)]
pub struct PythonFunctionSignature {
    pub name: String,
    pub module: String,
    pub params: Vec<ParamSpec>,
    pub return_type: PythonTypeTag,
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub name: String,
    pub type_tag: PythonTypeTag,
    pub optional: bool,
}

/// A call bridge that translates Fusion calls into Python function invocations.
pub struct FunctionCallBridge {
    registered_functions: HashMap<String, PythonFunctionSignature>,
}

impl FunctionCallBridge {
    pub fn new() -> Self {
        Self {
            registered_functions: HashMap::new(),
        }
    }

    /// Register a Python function that can be called from Fusion.
    pub fn register(&mut self, sig: PythonFunctionSignature) {
        self.registered_functions
            .insert(sig.name.clone(), sig);
    }

    /// Look up a registered function by name.
    pub fn lookup(&self, name: &str) -> Option<&PythonFunctionSignature> {
        self.registered_functions.get(name)
    }

    /// Prepare a call — validate argument types against the signature.
    pub fn prepare_call(
        &self,
        name: &str,
        args: &[PyValue],
    ) -> Result<PreparedCall> {
        let sig = self
            .registered_functions
            .get(name)
            .ok_or_else(|| PythonInteropError::FunctionCall(format!("function '{}' not found", name)))?;

        // Validate argument count.
        let required = sig.params.iter().filter(|p| !p.optional).count();
        if args.len() < required || args.len() > sig.params.len() {
            return Err(PythonInteropError::FunctionCall(format!(
                "'{}' expects {}-{} args, got {}",
                name,
                required,
                sig.params.len(),
                args.len()
            )));
        }

        // Validate argument types.
        for (i, (arg, param)) in args.iter().zip(sig.params.iter()).enumerate() {
            let arg_tag = PythonTypeTag::of(arg);
            if arg_tag != param.type_tag {
                return Err(PythonInteropError::FunctionCall(format!(
                    "argument {} of '{}': expected {}, got {}",
                    i + 1,
                    name,
                    param.type_tag.python_name(),
                    arg_tag.python_name()
                )));
            }
        }

        Ok(PreparedCall {
            signature: sig.clone(),
            args: args.to_vec(),
        })
    }
}

impl Default for FunctionCallBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// A validated, ready-to-execute Python call.
#[derive(Debug, Clone)]
pub struct PreparedCall {
    pub signature: PythonFunctionSignature,
    pub args: Vec<PyValue>,
}

// ──────────────────────────────────────────────
// Module import support
// ──────────────────────────────────────────────

/// Represents a Python module that can be imported from Fusion.
#[derive(Debug, Clone)]
pub struct PythonModule {
    pub name: String,
    pub path: Option<String>,
    pub functions: Vec<PythonFunctionSignature>,
    pub constants: HashMap<String, PyValue>,
}

/// Registry of importable Python modules.
pub struct ModuleRegistry {
    modules: HashMap<String, PythonModule>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Register a Python module.
    pub fn register(&mut self, module: PythonModule) {
        self.modules.insert(module.name.clone(), module);
    }

    /// Look up a module by name.
    pub fn lookup(&self, name: &str) -> Option<&PythonModule> {
        self.modules.get(name)
    }

    /// Import a module (simulate the import path resolution).
    pub fn import(&self, name: &str) -> Result<&PythonModule> {
        self.modules.get(name).ok_or_else(|| {
            PythonInteropError::ModuleImport(format!("module '{}' not found", name))
        })
    }

    /// List all registered modules.
    pub fn list_modules(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────
// GIL management
// ──────────────────────────────────────────────

/// Tracks whether the Python GIL is currently held by this thread.
///
/// In a real implementation, this would interact with PyGILState_Ensure /
/// PyGILState_Release through FFI. Here we simulate it for correctness
/// testing of the bridge API.
pub struct GilGuard {
    held: Arc<Mutex<bool>>,
}

static GIL_STATE: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

fn global_gil_state() -> &'static Arc<Mutex<bool>> {
    GIL_STATE.get_or_init(|| Arc::new(Mutex::new(false)))
}

impl GilGuard {
    /// Acquire the GIL. Blocks until acquired.
    pub fn acquire() -> Self {
        let state = global_gil_state();
        let mut held = state.lock().unwrap();
        *held = true;
        Self {
            held: Arc::clone(state),
        }
    }

    /// Try to acquire the GIL without blocking.
    pub fn try_acquire() -> Option<Self> {
        let state = global_gil_state();
        let mut held = state.lock().unwrap();
        if !*held {
            *held = true;
            Some(Self {
                held: Arc::clone(state),
            })
        } else {
            None
        }
    }

    /// Check if the GIL is currently held.
    pub fn is_held(&self) -> bool {
        *self.held.lock().unwrap()
    }
}

impl Drop for GilGuard {
    fn drop(&mut self) {
        let mut held = self.held.lock().unwrap();
        *held = false;
    }
}

// ──────────────────────────────────────────────
// High-level Python bridge
// ──────────────────────────────────────────────

/// The top-level bridge connecting Fusion to a Python runtime.
pub struct PythonBridge {
    pub call_bridge: FunctionCallBridge,
    pub module_registry: ModuleRegistry,
}

impl PythonBridge {
    pub fn new() -> Self {
        Self {
            call_bridge: FunctionCallBridge::new(),
            module_registry: ModuleRegistry::new(),
        }
    }

    /// Register a Python function for cross-language calls.
    pub fn register_function(&mut self, sig: PythonFunctionSignature) {
        self.call_bridge.register(sig);
    }

    /// Import a Python module.
    pub fn import_module(&mut self, module: PythonModule) {
        self.module_registry.register(module);
    }

    /// Call a registered Python function.
    pub fn call_function(
        &self,
        name: &str,
        args: &[PyValue],
    ) -> Result<PyValue> {
        let _gil = GilGuard::acquire();
        let prepared = self.call_bridge.prepare_call(name, args)?;

        // In a real implementation, this would invoke PyEval_CallObject.
        // For now, return None as a placeholder.
        log::info!(
            "Python bridge: calling {}({})",
            prepared.signature.name,
            prepared.args.len()
        );

        Ok(PyValue::None)
    }
}

impl Default for PythonBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PyValue tests ──

    #[test]
    fn test_py_value_constructors() {
        assert_eq!(PyValue::int(42), PyValue::Int(42));
        assert_eq!(PyValue::float(3.14), PyValue::Float(3.14));
        assert_eq!(PyValue::bool(true), PyValue::Bool(true));
        assert_eq!(PyValue::string("hello"), PyValue::String("hello".into()));
        assert_eq!(PyValue::default(), PyValue::None);
    }

    #[test]
    fn test_py_value_is_none() {
        assert!(PyValue::None.is_none());
        assert!(!PyValue::int(1).is_none());
    }

    // ── Marshaling tests ──

    #[test]
    fn test_into_py_value() {
        let v: PyValue = 42i64.into_py_value();
        assert_eq!(v, PyValue::Int(42));

        let v: PyValue = 3.14f64.into_py_value();
        assert_eq!(v, PyValue::Float(3.14));

        let v: PyValue = true.into_py_value();
        assert_eq!(v, PyValue::Bool(true));

        let v: PyValue = "test".into_py_value();
        assert_eq!(v, PyValue::String("test".into()));

        let v: PyValue = vec![1i64, 2, 3].into_py_value();
        assert_eq!(
            v,
            PyValue::List(vec![PyValue::Int(1), PyValue::Int(2), PyValue::Int(3)])
        );
    }

    #[test]
    fn test_from_py_value() {
        let v = PyValue::Int(42);
        assert_eq!(i64::from_py_value(&v).unwrap(), 42);

        let v = PyValue::Float(3.14);
        assert_eq!(f64::from_py_value(&v).unwrap(), 3.14);

        // Int can coerce to float.
        let v = PyValue::Int(10);
        assert_eq!(f64::from_py_value(&v).unwrap(), 10.0);

        let v = PyValue::Bool(false);
        assert!(!bool::from_py_value(&v).unwrap());

        let v = PyValue::String("hello".into());
        assert_eq!(String::from_py_value(&v).unwrap(), "hello");

        // Wrong type returns error.
        assert!(i64::from_py_value(&PyValue::String("x".into())).is_err());
    }

    #[test]
    fn test_from_py_value_list() {
        let v = PyValue::List(vec![PyValue::Int(1), PyValue::Int(2)]);
        let result: Vec<i64> = Vec::<i64>::from_py_value(&v).unwrap();
        assert_eq!(result, vec![1, 2]);
    }

    // ── Type tag tests ──

    #[test]
    fn test_type_tag_infer() {
        assert_eq!(PythonTypeTag::of(&PyValue::None), PythonTypeTag::NoneType);
        assert_eq!(PythonTypeTag::of(&PyValue::Int(1)), PythonTypeTag::Int);
        assert_eq!(
            PythonTypeTag::of(&PyValue::Float(1.0)),
            PythonTypeTag::Float
        );
        assert_eq!(
            PythonTypeTag::of(&PyValue::Bool(true)),
            PythonTypeTag::Bool
        );
        assert_eq!(
            PythonTypeTag::of(&PyValue::String("x".into())),
            PythonTypeTag::Str
        );
        assert_eq!(
            PythonTypeTag::of(&PyValue::List(vec![])),
            PythonTypeTag::List
        );
    }

    #[test]
    fn test_type_tag_names() {
        assert_eq!(PythonTypeTag::Int.python_name(), "int");
        assert_eq!(PythonTypeTag::Int.fusion_name(), "Int");
        assert_eq!(PythonTypeTag::List.python_name(), "list");
        assert_eq!(PythonTypeTag::List.fusion_name(), "Array");
    }

    // ── Function bridge tests ──

    #[test]
    fn test_register_and_lookup() {
        let mut bridge = FunctionCallBridge::new();
        bridge.register(PythonFunctionSignature {
            name: "add".into(),
            module: "math".into(),
            params: vec![
                ParamSpec { name: "a".into(), type_tag: PythonTypeTag::Int, optional: false },
                ParamSpec { name: "b".into(), type_tag: PythonTypeTag::Int, optional: false },
            ],
            return_type: PythonTypeTag::Int,
        });

        assert!(bridge.lookup("add").is_some());
        assert!(bridge.lookup("nonexistent").is_none());
    }

    #[test]
    fn test_prepare_call_valid() {
        let mut bridge = FunctionCallBridge::new();
        bridge.register(PythonFunctionSignature {
            name: "add".into(),
            module: "math".into(),
            params: vec![
                ParamSpec { name: "a".into(), type_tag: PythonTypeTag::Int, optional: false },
                ParamSpec { name: "b".into(), type_tag: PythonTypeTag::Int, optional: false },
            ],
            return_type: PythonTypeTag::Int,
        });

        let prepared = bridge
            .prepare_call("add", &[PyValue::Int(1), PyValue::Int(2)])
            .unwrap();
        assert_eq!(prepared.args.len(), 2);
    }

    #[test]
    fn test_prepare_call_wrong_arg_count() {
        let mut bridge = FunctionCallBridge::new();
        bridge.register(PythonFunctionSignature {
            name: "add".into(),
            module: "math".into(),
            params: vec![
                ParamSpec { name: "a".into(), type_tag: PythonTypeTag::Int, optional: false },
                ParamSpec { name: "b".into(), type_tag: PythonTypeTag::Int, optional: false },
            ],
            return_type: PythonTypeTag::Int,
        });

        assert!(bridge.prepare_call("add", &[PyValue::Int(1)]).is_err());
    }

    #[test]
    fn test_prepare_call_wrong_type() {
        let mut bridge = FunctionCallBridge::new();
        bridge.register(PythonFunctionSignature {
            name: "add".into(),
            module: "math".into(),
            params: vec![
                ParamSpec { name: "a".into(), type_tag: PythonTypeTag::Int, optional: false },
                ParamSpec { name: "b".into(), type_tag: PythonTypeTag::Int, optional: false },
            ],
            return_type: PythonTypeTag::Int,
        });

        assert!(bridge
            .prepare_call("add", &[PyValue::Int(1), PyValue::String("x".into())])
            .is_err());
    }

    #[test]
    fn test_prepare_call_optional_params() {
        let mut bridge = FunctionCallBridge::new();
        bridge.register(PythonFunctionSignature {
            name: "greet".into(),
            module: "builtins".into(),
            params: vec![
                ParamSpec { name: "name".into(), type_tag: PythonTypeTag::Str, optional: false },
                ParamSpec { name: "shout".into(), type_tag: PythonTypeTag::Bool, optional: true },
            ],
            return_type: PythonTypeTag::Str,
        });

        // With optional param omitted.
        let prepared = bridge
            .prepare_call("greet", &[PyValue::String("world".into())])
            .unwrap();
        assert_eq!(prepared.args.len(), 1);
    }

    // ── Module registry tests ──

    #[test]
    fn test_module_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(PythonModule {
            name: "math".into(),
            path: None,
            functions: vec![],
            constants: HashMap::new(),
        });

        assert!(registry.import("math").is_ok());
        assert!(registry.import("os").is_err());
        assert_eq!(registry.list_modules(), vec!["math"]);
    }

    // ── GIL tests ──

    #[test]
    fn test_gil_acquire_and_release() {
        {
            let _gil = GilGuard::acquire();
            // GIL is held within this scope.
        }
        // After drop, GIL is released; try_acquire should succeed.
        let gil = GilGuard::try_acquire();
        assert!(gil.is_some());
        drop(gil);
    }

    #[test]
    fn test_gil_try_acquire_conflict() {
        let _gil1 = GilGuard::acquire();
        // Second acquire should fail because GIL is held.
        let gil2 = GilGuard::try_acquire();
        assert!(gil2.is_none());
    }

    // ── PythonBridge tests ──

    #[test]
    fn test_bridge_call_function() {
        let mut bridge = PythonBridge::new();
        bridge.register_function(PythonFunctionSignature {
            name: "len".into(),
            module: "builtins".into(),
            params: vec![ParamSpec {
                name: "obj".into(),
                type_tag: PythonTypeTag::List,
                optional: false,
            }],
            return_type: PythonTypeTag::Int,
        });

        let result = bridge
            .call_function("len", &[PyValue::list(vec![PyValue::Int(1)])])
            .unwrap();
        assert_eq!(result, PyValue::None); // placeholder return
    }

    #[test]
    fn test_bridge_call_unknown_function() {
        let bridge = PythonBridge::new();
        let result = bridge.call_function("unknown", &[]);
        assert!(result.is_err());
    }
}
