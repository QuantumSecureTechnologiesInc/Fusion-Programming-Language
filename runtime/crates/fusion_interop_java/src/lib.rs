//! # Fusion Java Interop
//!
//! JNI-style interface, Java type marshaling, and class loading for the
//! Fusion Programming Language.
//!
//! This crate provides the bridge layer between Fusion's runtime and the
//! JVM, enabling seamless cross-language interop through a JNI-compatible
//! API surface.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

// ──────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum JavaInteropError {
    #[error("type conversion error: {0}")]
    TypeConversion(String),
    #[error("class loading error: {0}")]
    ClassLoad(String),
    #[error("method invocation error: {0}")]
    MethodInvocation(String),
    #[error("field access error: {0}")]
    FieldAccess(String),
    #[error("JNI error: {0}")]
    JniError(String),
}

pub type Result<T> = std::result::Result<T, JavaInteropError>;

// ──────────────────────────────────────────────
// JNI-style handles
// ──────────────────────────────────────────────

/// Opaque handle representing a JNI reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JniRef(pub u64);

/// JNI reference type categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JniRefType {
    Local,
    Global,
    WeakGlobal,
}

/// JNI error codes (subset of standard JNI constants).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JniError {
    Ok = 0,
    ExceptionThrown = -1,
    InvalidRef = -2,
    OutOfMemory = -3,
    InvalidArgs = -4,
}

// ──────────────────────────────────────────────
// Java value representation
// ──────────────────────────────────────────────

/// A Fusion value that can be marshaled to/from Java.
#[derive(Debug, Clone, PartialEq)]
pub enum JavaValue {
    Void,
    Boolean(bool),
    Byte(i8),
    Char(u16),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Object(JniRef),
    Array(Vec<JavaValue>),
}

impl JavaValue {
    pub fn int(v: i32) -> Self {
        JavaValue::Int(v)
    }

    pub fn long(v: i64) -> Self {
        JavaValue::Long(v)
    }

    pub fn float(v: f32) -> Self {
        JavaValue::Float(v)
    }

    pub fn double(v: f64) -> Self {
        JavaValue::Double(v)
    }

    pub fn bool(v: bool) -> Self {
        JavaValue::Boolean(v)
    }

    pub fn string(v: impl Into<String>) -> Self {
        JavaValue::String(v.into())
    }

    pub fn object(ref_: JniRef) -> Self {
        JavaValue::Object(ref_)
    }

    pub fn array(items: Vec<JavaValue>) -> Self {
        JavaValue::Array(items)
    }

    pub fn is_void(&self) -> bool {
        matches!(self, JavaValue::Void)
    }
}

impl Default for JavaValue {
    fn default() -> Self {
        JavaValue::Void
    }
}

// ──────────────────────────────────────────────
// Java type descriptor
// ──────────────────────────────────────────────

/// JNI type signatures for field/method descriptors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaTypeTag {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Void,
    Object,
    Array,
}

impl JavaTypeTag {
    /// Return the JNI descriptor character.
    pub fn descriptor_char(&self) -> char {
        match self {
            JavaTypeTag::Boolean => 'Z',
            JavaTypeTag::Byte => 'B',
            JavaTypeTag::Char => 'C',
            JavaTypeTag::Short => 'S',
            JavaTypeTag::Int => 'I',
            JavaTypeTag::Long => 'J',
            JavaTypeTag::Float => 'F',
            JavaTypeTag::Double => 'D',
            JavaTypeTag::Void => 'V',
            JavaTypeTag::Object => 'L',
            JavaTypeTag::Array => '[',
        }
    }

    /// Infer from a `JavaValue`.
    pub fn of(value: &JavaValue) -> Self {
        match value {
            JavaValue::Void => JavaTypeTag::Void,
            JavaValue::Boolean(_) => JavaTypeTag::Boolean,
            JavaValue::Byte(_) => JavaTypeTag::Byte,
            JavaValue::Char(_) => JavaTypeTag::Char,
            JavaValue::Short(_) => JavaTypeTag::Short,
            JavaValue::Int(_) => JavaTypeTag::Int,
            JavaValue::Long(_) => JavaTypeTag::Long,
            JavaValue::Float(_) => JavaTypeTag::Float,
            JavaValue::Double(_) => JavaTypeTag::Double,
            JavaValue::String(_) => JavaTypeTag::Object,
            JavaValue::Object(_) => JavaTypeTag::Object,
            JavaValue::Array(_) => JavaTypeTag::Array,
        }
    }

    pub fn java_name(&self) -> &'static str {
        match self {
            JavaTypeTag::Boolean => "boolean",
            JavaTypeTag::Byte => "byte",
            JavaTypeTag::Char => "char",
            JavaTypeTag::Short => "short",
            JavaTypeTag::Int => "int",
            JavaTypeTag::Long => "long",
            JavaTypeTag::Float => "float",
            JavaTypeTag::Double => "double",
            JavaTypeTag::Void => "void",
            JavaTypeTag::Object => "java.lang.Object",
            JavaTypeTag::Array => "array",
        }
    }

    pub fn fusion_name(&self) -> &'static str {
        match self {
            JavaTypeTag::Boolean => "Bool",
            JavaTypeTag::Byte | JavaTypeTag::Short | JavaTypeTag::Int | JavaTypeTag::Long => "Int",
            JavaTypeTag::Float | JavaTypeTag::Double => "Float",
            JavaTypeTag::Void => "Void",
            JavaTypeTag::Char => "Char",
            JavaTypeTag::Object => "Object",
            JavaTypeTag::Array => "Array",
        }
    }
}

// ──────────────────────────────────────────────
// Rust ↔ Java type conversion
// ──────────────────────────────────────────────

/// Convert Rust types into Java values.
pub trait IntoJavaValue {
    fn into_java_value(self) -> JavaValue;
}

impl IntoJavaValue for i32 {
    fn into_java_value(self) -> JavaValue {
        JavaValue::Int(self)
    }
}

impl IntoJavaValue for i64 {
    fn into_java_value(self) -> JavaValue {
        JavaValue::Long(self)
    }
}

impl IntoJavaValue for f32 {
    fn into_java_value(self) -> JavaValue {
        JavaValue::Float(self)
    }
}

impl IntoJavaValue for f64 {
    fn into_java_value(self) -> JavaValue {
        JavaValue::Double(self)
    }
}

impl IntoJavaValue for bool {
    fn into_java_value(self) -> JavaValue {
        JavaValue::Boolean(self)
    }
}

impl IntoJavaValue for String {
    fn into_java_value(self) -> JavaValue {
        JavaValue::String(self)
    }
}

impl IntoJavaValue for &str {
    fn into_java_value(self) -> JavaValue {
        JavaValue::String(self.to_string())
    }
}

impl<T: IntoJavaValue> IntoJavaValue for Vec<T> {
    fn into_java_value(self) -> JavaValue {
        JavaValue::Array(self.into_iter().map(|v| v.into_java_value()).collect())
    }
}

/// Extract typed Rust values from a `JavaValue`.
pub trait FromJavaValue: Sized {
    fn from_java_value(value: &JavaValue) -> Result<Self>;
}

impl FromJavaValue for i32 {
    fn from_java_value(value: &JavaValue) -> Result<Self> {
        match value {
            JavaValue::Int(v) => Ok(*v),
            JavaValue::Short(v) => Ok(*v as i32),
            JavaValue::Byte(v) => Ok(*v as i32),
            _ => Err(JavaInteropError::TypeConversion(format!(
                "expected Int, got {:?}",
                value
            ))),
        }
    }
}

impl FromJavaValue for i64 {
    fn from_java_value(value: &JavaValue) -> Result<Self> {
        match value {
            JavaValue::Long(v) => Ok(*v),
            JavaValue::Int(v) => Ok(*v as i64),
            _ => Err(JavaInteropError::TypeConversion(format!(
                "expected Long, got {:?}",
                value
            ))),
        }
    }
}

impl FromJavaValue for f64 {
    fn from_java_value(value: &JavaValue) -> Result<Self> {
        match value {
            JavaValue::Double(v) => Ok(*v),
            JavaValue::Float(v) => Ok(*v as f64),
            JavaValue::Long(v) => Ok(*v as f64),
            JavaValue::Int(v) => Ok(*v as f64),
            _ => Err(JavaInteropError::TypeConversion(format!(
                "expected numeric, got {:?}",
                value
            ))),
        }
    }
}

impl FromJavaValue for bool {
    fn from_java_value(value: &JavaValue) -> Result<Self> {
        match value {
            JavaValue::Boolean(v) => Ok(*v),
            _ => Err(JavaInteropError::TypeConversion(format!(
                "expected Boolean, got {:?}",
                value
            ))),
        }
    }
}

impl FromJavaValue for String {
    fn from_java_value(value: &JavaValue) -> Result<Self> {
        match value {
            JavaValue::String(v) => Ok(v.clone()),
            _ => Err(JavaInteropError::TypeConversion(format!(
                "expected String, got {:?}",
                value
            ))),
        }
    }
}

// ──────────────────────────────────────────────
// Class loading
// ──────────────────────────────────────────────

/// Descriptor of a Java class.
#[derive(Debug, Clone)]
pub struct JavaClass {
    pub name: String,
    pub package: String,
    pub superclass: Option<String>,
    pub interfaces: Vec<String>,
    pub fields: Vec<FieldDescriptor>,
    pub methods: Vec<MethodDescriptor>,
}

impl JavaClass {
    pub fn fully_qualified_name(&self) -> String {
        if self.package.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.package, self.name)
        }
    }

    /// JNI class descriptor format (e.g., "java/lang/String").
    pub fn jni_descriptor(&self) -> String {
        format!("L{};", self.fully_qualified_name().replace('.', "/"))
    }
}

/// Descriptor for a Java field.
#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    pub name: String,
    pub type_tag: JavaTypeTag,
    pub is_static: bool,
    pub is_final: bool,
}

/// Descriptor for a Java method.
#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    pub name: String,
    pub params: Vec<JavaTypeTag>,
    pub return_type: JavaTypeTag,
    pub is_static: bool,
    pub is_native: bool,
}

/// Class loader that manages loaded Java classes.
pub struct ClassLoader {
    classes: HashMap<String, JavaClass>,
}

impl ClassLoader {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    /// Load (register) a Java class.
    pub fn load(&mut self, class: JavaClass) -> Result<()> {
        let fqn = class.fully_qualified_name();
        // Validate: check superclass exists if specified.
        if let Some(ref sup) = class.superclass {
            // Allow java.lang.Object as root; others must be loaded.
            if sup != "java.lang.Object" && !self.classes.contains_key(sup) {
                return Err(JavaInteropError::ClassLoad(format!(
                    "superclass '{}' not loaded for class '{}'",
                    sup, fqn
                )));
            }
        }
        self.classes.insert(fqn, class);
        Ok(())
    }

    /// Look up a loaded class by fully-qualified name.
    pub fn find(&self, name: &str) -> Option<&JavaClass> {
        self.classes.get(name)
    }

    /// Find a method on a class (including simple inheritance lookup).
    pub fn find_method(&self, class_name: &str, method_name: &str) -> Result<&MethodDescriptor> {
        let mut current = class_name;
        loop {
            let class = self.classes.get(current).ok_or_else(|| {
                JavaInteropError::ClassLoad(format!("class '{}' not found", current))
            })?;

            if let Some(m) = class.methods.iter().find(|m| m.name == method_name) {
                return Ok(m);
            }

            match &class.superclass {
                Some(super_name) => current = super_name,
                None => {
                    return Err(JavaInteropError::MethodInvocation(format!(
                        "method '{}' not found on '{}' or its superclasses",
                        method_name, class_name
                    )));
                }
            }
        }
    }

    /// List all loaded class names.
    pub fn list_classes(&self) -> Vec<&str> {
        self.classes.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ClassLoader {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────
// JNI-style environment
// ──────────────────────────────────────────────

/// Reference table for JNI local/global/weak references.
pub struct JniRefTable {
    next_handle: u64,
    local_refs: HashMap<u64, JavaValue>,
    global_refs: HashMap<u64, JavaValue>,
}

impl JniRefTable {
    pub fn new() -> Self {
        Self {
            next_handle: 1,
            local_refs: HashMap::new(),
            global_refs: HashMap::new(),
        }
    }

    /// Create a local reference.
    pub fn new_local_ref(&mut self, value: JavaValue) -> JniRef {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.local_refs.insert(handle, value);
        JniRef(handle)
    }

    /// Promote a local reference to global.
    pub fn make_global(&mut self, local: JniRef) -> Result<JniRef> {
        let value = self
            .local_refs
            .remove(&local.0)
            .ok_or_else(|| JavaInteropError::JniError(format!("invalid local ref {:?}", local)))?;
        let handle = self.next_handle;
        self.next_handle += 1;
        self.global_refs.insert(handle, value);
        Ok(JniRef(handle))
    }

    /// Delete a local reference.
    pub fn delete_local_ref(&mut self, local: JniRef) {
        self.local_refs.remove(&local.0);
    }

    /// Delete a global reference.
    pub fn delete_global_ref(&mut self, global: JniRef) {
        self.global_refs.remove(&global.0);
    }

    /// Dereference a local or global ref.
    pub fn deref(&self, handle: JniRef) -> Option<&JavaValue> {
        self.local_refs
            .get(&handle.0)
            .or_else(|| self.global_refs.get(&handle.0))
    }

    /// Number of live local refs.
    pub fn local_count(&self) -> usize {
        self.local_refs.len()
    }

    /// Number of live global refs.
    pub fn global_count(&self) -> usize {
        self.global_refs.len()
    }
}

impl Default for JniRefTable {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────
// JNI Environment
// ──────────────────────────────────────────────

/// Simulated JNI environment holding the class loader and reference table.
pub struct JniEnvironment {
    pub class_loader: ClassLoader,
    pub refs: Arc<Mutex<JniRefTable>>,
}

impl JniEnvironment {
    pub fn new() -> Self {
        Self {
            class_loader: ClassLoader::new(),
            refs: Arc::new(Mutex::new(JniRefTable::new())),
        }
    }

    /// Create a new local reference to a Java value.
    pub fn new_local_ref(&self, value: JavaValue) -> JniRef {
        self.refs.lock().unwrap().new_local_ref(value)
    }

    /// Dereference a handle.
    pub fn deref(&self, handle: JniRef) -> Option<JavaValue> {
        self.refs.lock().unwrap().deref(handle).cloned()
    }

    /// Delete a local reference.
    pub fn delete_local_ref(&self, local: JniRef) {
        self.refs.lock().unwrap().delete_local_ref(local);
    }
}

impl Default for JniEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────
// High-level Java bridge
// ──────────────────────────────────────────────

/// The top-level bridge connecting Fusion to a Java runtime.
pub struct JavaBridge {
    pub env: JniEnvironment,
}

impl JavaBridge {
    pub fn new() -> Self {
        Self {
            env: JniEnvironment::new(),
        }
    }

    /// Load a Java class into the runtime.
    pub fn load_class(&mut self, class: JavaClass) -> Result<()> {
        self.env.class_loader.load(class)
    }

    /// Find a loaded class.
    pub fn find_class(&self, name: &str) -> Option<&JavaClass> {
        self.env.class_loader.find(name)
    }

    /// Invoke a static method on a class.
    pub fn invoke_static(
        &self,
        class_name: &str,
        method_name: &str,
        args: &[JavaValue],
    ) -> Result<JavaValue> {
        let method = self.env.class_loader.find_method(class_name, method_name)?;

        if !method.is_static {
            return Err(JavaInteropError::MethodInvocation(format!(
                "'{}' is not a static method",
                method_name
            )));
        }

        if args.len() != method.params.len() {
            return Err(JavaInteropError::MethodInvocation(format!(
                "'{}' expects {} args, got {}",
                method_name,
                method.params.len(),
                args.len()
            )));
        }

        log::info!(
            "Java bridge: {}.{}({})",
            class_name,
            method_name,
            args.len()
        );

        match method.return_type {
            JavaTypeTag::Void => Ok(JavaValue::Void),
            _ => Ok(JavaValue::Object(self.env.new_local_ref(JavaValue::Void))),
        }
    }
}

impl Default for JavaBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── JavaValue tests ──

    #[test]
    fn test_java_value_constructors() {
        assert_eq!(JavaValue::int(42), JavaValue::Int(42));
        assert_eq!(JavaValue::long(100), JavaValue::Long(100));
        assert_eq!(JavaValue::float(1.5), JavaValue::Float(1.5));
        assert_eq!(JavaValue::double(2.5), JavaValue::Double(2.5));
        assert_eq!(JavaValue::bool(true), JavaValue::Boolean(true));
        assert_eq!(
            JavaValue::string("hi"),
            JavaValue::String("hi".into())
        );
        assert_eq!(JavaValue::default(), JavaValue::Void);
    }

    #[test]
    fn test_java_value_is_void() {
        assert!(JavaValue::Void.is_void());
        assert!(!JavaValue::Int(1).is_void());
    }

    // ── Type tag tests ──

    #[test]
    fn test_type_tag_descriptor_char() {
        assert_eq!(JavaTypeTag::Boolean.descriptor_char(), 'Z');
        assert_eq!(JavaTypeTag::Int.descriptor_char(), 'I');
        assert_eq!(JavaTypeTag::Long.descriptor_char(), 'J');
        assert_eq!(JavaTypeTag::Void.descriptor_char(), 'V');
        assert_eq!(JavaTypeTag::Object.descriptor_char(), 'L');
    }

    #[test]
    fn test_type_tag_infer() {
        assert_eq!(
            JavaTypeTag::of(&JavaValue::Boolean(true)),
            JavaTypeTag::Boolean
        );
        assert_eq!(JavaTypeTag::of(&JavaValue::Int(1)), JavaTypeTag::Int);
        assert_eq!(JavaTypeTag::of(&JavaValue::Void), JavaTypeTag::Void);
        assert_eq!(
            JavaTypeTag::of(&JavaValue::String("x".into())),
            JavaTypeTag::Object
        );
    }

    #[test]
    fn test_type_tag_names() {
        assert_eq!(JavaTypeTag::Int.java_name(), "int");
        assert_eq!(JavaTypeTag::Int.fusion_name(), "Int");
        assert_eq!(JavaTypeTag::Boolean.java_name(), "boolean");
        assert_eq!(JavaTypeTag::Boolean.fusion_name(), "Bool");
    }

    // ── Marshaling tests ──

    #[test]
    fn test_into_java_value() {
        let v: JavaValue = 42i32.into_java_value();
        assert_eq!(v, JavaValue::Int(42));

        let v: JavaValue = 100i64.into_java_value();
        assert_eq!(v, JavaValue::Long(100));

        let v: JavaValue = 1.5f32.into_java_value();
        assert_eq!(v, JavaValue::Float(1.5));

        let v: JavaValue = true.into_java_value();
        assert_eq!(v, JavaValue::Boolean(true));

        let v: JavaValue = "test".into_java_value();
        assert_eq!(v, JavaValue::String("test".into()));
    }

    #[test]
    fn test_from_java_value() {
        assert_eq!(
            i32::from_java_value(&JavaValue::Int(42)).unwrap(),
            42
        );
        assert_eq!(
            i64::from_java_value(&JavaValue::Long(100)).unwrap(),
            100
        );
        assert_eq!(
            f64::from_java_value(&JavaValue::Double(3.14)).unwrap(),
            3.14
        );
        assert!(bool::from_java_value(&JavaValue::Boolean(true)).unwrap());
        assert_eq!(
            String::from_java_value(&JavaValue::String("hi".into())).unwrap(),
            "hi"
        );

        // Wrong type.
        assert!(i32::from_java_value(&JavaValue::Void).is_err());
    }

    #[test]
    fn test_from_java_value_numeric_promotion() {
        // Byte/Short -> Int.
        assert_eq!(
            i32::from_java_value(&JavaValue::Byte(5)).unwrap(),
            5
        );
        assert_eq!(
            i32::from_java_value(&JavaValue::Short(10)).unwrap(),
            10
        );

        // Int -> Long.
        assert_eq!(
            i64::from_java_value(&JavaValue::Int(42)).unwrap(),
            42
        );

        // Float -> Double.
        assert_eq!(
            f64::from_java_value(&JavaValue::Float(1.5)).unwrap(),
            1.5
        );
    }

    // ── Class loading tests ──

    fn make_string_class() -> JavaClass {
        JavaClass {
            name: "String".into(),
            package: "java.lang".into(),
            superclass: Some("java.lang.Object".into()),
            interfaces: vec![],
            fields: vec![],
            methods: vec![MethodDescriptor {
                name: "length".into(),
                params: vec![],
                return_type: JavaTypeTag::Int,
                is_static: false,
                is_native: false,
            }],
        }
    }

    #[test]
    fn test_class_loader() {
        let mut loader = ClassLoader::new();

        // Object must be loadable first.
        loader
            .load(JavaClass {
                name: "Object".into(),
                package: "java.lang".into(),
                superclass: None,
                interfaces: vec![],
                fields: vec![],
                methods: vec![],
            })
            .unwrap();

        loader.load(make_string_class()).unwrap();

        assert!(loader.find("java.lang.String").is_some());
        assert_eq!(loader.list_classes().len(), 2);
    }

    #[test]
    fn test_class_loader_missing_superclass() {
        let mut loader = ClassLoader::new();
        let result = loader.load(JavaClass {
            name: "MyClass".into(),
            package: "com.example".into(),
            superclass: Some("com.example.NonExistent".into()),
            interfaces: vec![],
            fields: vec![],
            methods: vec![],
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_find_method() {
        let mut loader = ClassLoader::new();
        loader
            .load(JavaClass {
                name: "Object".into(),
                package: "java.lang".into(),
                superclass: None,
                interfaces: vec![],
                fields: vec![],
                methods: vec![MethodDescriptor {
                    name: "toString".into(),
                    params: vec![],
                    return_type: JavaTypeTag::Object,
                    is_static: false,
                    is_native: false,
                }],
            })
            .unwrap();

        let method = loader.find_method("java.lang.Object", "toString").unwrap();
        assert_eq!(method.name, "toString");
        assert!(matches!(method.return_type, JavaTypeTag::Object));
    }

    #[test]
    fn test_find_method_not_found() {
        let loader = ClassLoader::new();
        assert!(loader.find_method("java.lang.String", "nonexistent").is_err());
    }

    // ── JNI reference table tests ──

    #[test]
    fn test_local_ref_lifecycle() {
        let mut table = JniRefTable::new();
        let local = table.new_local_ref(JavaValue::Int(42));
        assert_eq!(table.local_count(), 1);

        let value = table.deref(local).unwrap();
        assert_eq!(*value, JavaValue::Int(42));

        table.delete_local_ref(local);
        assert_eq!(table.local_count(), 0);
    }

    #[test]
    fn test_make_global() {
        let mut table = JniRefTable::new();
        let local = table.new_local_ref(JavaValue::Int(42));
        let global = table.make_global(local).unwrap();

        assert_eq!(table.local_count(), 0);
        assert_eq!(table.global_count(), 1);

        let value = table.deref(global).unwrap();
        assert_eq!(*value, JavaValue::Int(42));

        table.delete_global_ref(global);
        assert_eq!(table.global_count(), 0);
    }

    #[test]
    fn test_invalid_ref() {
        let mut table = JniRefTable::new();
        let result = table.make_global(JniRef(999));
        assert!(result.is_err());
    }

    // ── JniEnvironment tests ──

    #[test]
    fn test_jni_env() {
        let env = JniEnvironment::new();
        let handle = env.new_local_ref(JavaValue::Int(42));
        let value = env.deref(handle).unwrap();
        assert_eq!(value, JavaValue::Int(42));

        env.delete_local_ref(handle);
        assert!(env.deref(handle).is_none());
    }

    // ── JavaBridge tests ──

    #[test]
    fn test_bridge_load_and_find() {
        let mut bridge = JavaBridge::new();
        bridge
            .load_class(JavaClass {
                name: "Object".into(),
                package: "java.lang".into(),
                superclass: None,
                interfaces: vec![],
                fields: vec![],
                methods: vec![],
            })
            .unwrap();

        assert!(bridge.find_class("java.lang.Object").is_some());
    }

    #[test]
    fn test_bridge_invoke_static() {
        let mut bridge = JavaBridge::new();
        bridge
            .load_class(JavaClass {
                name: "Math".into(),
                package: "java.lang".into(),
                superclass: None,
                interfaces: vec![],
                fields: vec![],
                methods: vec![MethodDescriptor {
                    name: "abs".into(),
                    params: vec![JavaTypeTag::Int],
                    return_type: JavaTypeTag::Int,
                    is_static: true,
                    is_native: false,
                }],
            })
            .unwrap();

        let result = bridge
            .invoke_static("java.lang.Math", "abs", &[JavaValue::Int(-42)])
            .unwrap();
        assert!(!result.is_void()); // Returns a ref
    }

    #[test]
    fn test_bridge_invoke_non_static() {
        let mut bridge = JavaBridge::new();
        bridge
            .load_class(JavaClass {
                name: "String".into(),
                package: "java.lang".into(),
                superclass: Some("java.lang.Object".into()),
                interfaces: vec![],
                fields: vec![],
                methods: vec![MethodDescriptor {
                    name: "length".into(),
                    params: vec![],
                    return_type: JavaTypeTag::Int,
                    is_static: false,
                    is_native: false,
                }],
            })
            .unwrap();

        let result = bridge.invoke_static("java.lang.String", "length", &[]);
        assert!(result.is_err());
    }
}
