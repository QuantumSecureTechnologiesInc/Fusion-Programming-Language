# Chapter 22: Pillar 4 — Modularity, Code Organization & Metaprogramming

> Module system, package management, project organization, generics, macros, and reflection

---

## Module System

Modules in Fusion provide namespacing, visibility control, and code organization. Every crate implicitly has a root module, and you can create a tree of nested modules.

### Module Declarations

Use the `mod` keyword to declare a module inline or map it to a file:

```fusion
// Inline module declaration
mod network {
    pub mod tcp {
        pub fn connect(host: string, port: int) -> Connection {
            return Connection::new(host, port);
        }
    }

    pub mod udp {
        pub fn send(data: bytes, target: string) -> int {
            return 0;
        }
    }
}

// File-based module declaration (maps to network/dns.fu)
mod network::dns {
    pub fn resolve(domain: string) -> string {
        return "127.0.0.1";
    }
}
```

Module declarations can be nested. Each `mod` creates a new namespace scope:

```fusion
mod app {
    mod controllers {
        pub mod auth {
            pub fn login(user: string, pass: string) -> Result<Session, string> {
                return Session::create(user);
            }
        }
        pub mod users {
            pub fn list() -> Vec<User> {
                return Vec::new();
            }
        }
    }

    mod models {
        pub struct User {
            pub id: int,
            pub name: string,
            pub email: string,
        }

        pub struct Session {
            token: string,
            user_id: int,
        }
    }
}
```

### Import / Export (use, pub)

The `pub` keyword makes items visible outside their module. The `use` keyword brings items into the current scope:

```fusion
// Items are private by default
mod internal {
    fn helper() -> string { return "hidden"; }

    pub fn public_api() -> string {
        return helper();  // can access private items in same module
    }
}

// Importing specific items
use app::models::User;
use app::controllers::auth::login;

// Importing with alias
use app::models::User as UserModel;

// Importing all public items from a module
use app::controllers::auth::*;

// Re-exporting
mod api {
    pub use crate::app::models::User;
    pub use crate::app::controllers::auth::login;

    // Re-export an entire module
    pub use crate::app::models;
}
```

### Visibility Controls

Fusion provides granular visibility modifiers:

```fusion
pub struct Config {
    pub name: string,           // visible to everyone
    pub(crate) internal: int,   // visible within this crate only
    secret: string,             // private — only this module
}

impl Config {
    pub fn new(name: string) -> Config {
        return Config {
            name,
            internal: 0,
            secret: "hidden".to_string(),
        };
    }

    pub(crate) fn reload(&mut self) {
        // callable from anywhere in this crate
        self.internal = self.internal + 1;
    }

    fn validate(&self) -> bool {
        // private — only callable within this module
        return self.name.len() > 0;
    }
}
```

| Visibility | Keyword | Scope |
|-----------|---------|-------|
| Fully public | `pub` | Anywhere |
| Crate-scoped | `pub(crate)` | Current crate only |
| Module-private | *(none)* | Current module only |

### Module Hierarchy and Namespace Management

Modules form a tree rooted at the crate root. Use `crate::` for absolute paths and `super::` for parent paths:

```fusion
mod outer {
    pub fn greet() -> string { return "hello from outer"; }

    mod inner {
        pub fn greet() -> string { return "hello from inner"; }

        pub fn call_outer() -> string {
            // Absolute path from crate root
            return crate::outer::greet();
        }

        pub fn call_sibling() -> string {
            // Relative path to parent's sibling
            return super::greet();
        }
    }
}
```

### Circular Dependency Handling

Fusion prevents circular module dependencies at compile time:

```fusion
// This produces a compile error:
mod a {
    use crate::b::foo;  // a depends on b
}

mod b {
    use crate::a::bar;  // b depends on a — CIRCULAR!
}

// Solution: extract shared items into a third module
mod common {
    pub struct SharedData {
        pub value: int,
    }
}

mod a {
    use crate::common::SharedData;  // no cycle
}

mod b {
    use crate::common::SharedData;  // no cycle
}
```

### Module Resolution Algorithm

The compiler resolves module paths using this algorithm:

1. **Absolute paths** (`crate::` or `super::`): resolved from the crate root or parent module
2. **Relative paths**: resolved from the current module
3. **File-based modules**: a `mod foo;` declaration looks for `foo.fu` or `foo/mod.fu`
4. **External crates**: resolved via the package registry or workspace paths

```fusion
// Path resolution examples
mod a {
    pub fn hello() -> string { return "a::hello"; }
}

mod b {
    pub fn hello() -> string { return "b::hello"; }

    pub fn call_a() -> string {
        return crate::a::hello();   // absolute
    }

    pub fn call_a2() -> string {
        return super::a::hello();   // relative (b's parent is crate root)
    }
}
```

---

## Package Management

### Fusion.toml Manifest Format

Every Fusion project is defined by a `fusion.toml` manifest file:

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2024"
authors = ["Jane Doe <jane@example.com>"]
description = "A Fusion application"
license = "MIT"

[dependencies]
# Version from registry
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }

# Path dependency
my-lib = { path = "../my-lib" }

# Git dependency
utils = { git = "https://github.com/user/utils", branch = "main" }

# Workspace dependency
common = { workspace = true }

[dev-dependencies]
assert_cmd = "2.0"

[build-dependencies]
cc = "1.0"
```

### Dependency Declaration

Dependencies can be declared with different sources:

```fusion
// Direct version dependency
use serde::json;

// Path dependency
use my_lib::helpers;

// Git dependency with specific revision
// (declared in Fusion.toml, imported normally)
use utils::parse;
```

### Versioning Rules (SemVer)

Fusion follows Semantic Versioning:

| Version | Meaning |
|---------|---------|
| `MAJOR.MINOR.PATCH` | Standard SemVer |
| `>=1.0, <2.0` | Compatible with 1.x |
| `~1.2` | Compatible with 1.2.x |
| `^1.2` | Compatible with 1.x.x |

### Package Registry

The Fusion package registry (`registry.fusion-lang.org`) hosts public packages:

```bash
# Search for packages
fuc search json

# Install a package
fuc add serde

# Update dependencies
fuc update

# Publish your package
fuc publish
```

### Forge Build System

Forge is Fusion's build system, handling compilation, linking, and cross-language builds:

```bash
# Build the project
forge build

# Build in release mode
forge build --release

# Run tests
forge test

# Build with specific target
forge build --target wasm32-unknown-unknown
```

### Cross-Language Dependencies

Fusion can depend on libraries written in other languages:

```toml
[dependencies]
# C library
libxml2 = { c_lib = { path = "/usr/lib/libxml2.so" } }

# Python package (used via polyglot)
numpy = { polyglot = { lang = "python", package = "numpy", version = ">=1.20" } }

# Rust crate (used via FFI)
serde_json = { rust_crate = { version = "1.0" } }

# JavaScript npm package
lodash = { npm = { package = "lodash", version = "^4.17" } }
```

---

## Code Organization

### Project Structure Conventions

A standard Fusion project follows this layout:

```
my-project/
├── fusion.toml          # Package manifest
├── src/
│   ├── main.fu          # Binary entry point (crate root)
│   ├── lib.fu           # Library crate root (alternative)
│   ├── config.fu        # Module file
│   ├── models/
│   │   ├── mod.fu       # Module declarations for models
│   │   ├── user.fu      # User model
│   │   └── session.fu   # Session model
│   ├── handlers/
│   │   ├── mod.fu
│   │   ├── auth.fu
│   │   └── users.fu
│   └── utils.fu
├── tests/
│   ├── integration.fu
│   └── common/
│       └── mod.fu
├── benches/
│   └── performance.fu
└── docs/
    └── architecture.fu
```

### File Naming Conventions

- Use `snake_case` for all file and module names
- Module files match the `mod` declaration name
- Test files are prefixed with `test_` or live in `tests/`
- Use `.fu` as the file extension

```fusion
// src/models/mod.fu — declares submodules
pub mod user;
pub mod session;
pub mod post;

// src/models/user.fu — User struct and impl
pub struct User {
    pub id: int,
    pub name: string,
    pub email: string,
}
```

### Module Organization Patterns

**Facade Pattern** — expose a simplified public API:

```fusion
// src/api/mod.fu
pub mod auth;
pub mod data;

// Re-export only what consumers need
pub use auth::{login, logout, Session};
pub use data::{query, insert, Database};
```

**Plugin Pattern** — allow runtime extension:

```fusion
pub trait Plugin {
    fn name(&self) -> string;
    fn execute(&self, input: &dyn Any) -> Result<Box<dyn Any>, string>;
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> PluginRegistry {
        return PluginRegistry { plugins: Vec::new() };
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn execute_all(&self, input: &dyn Any) -> Vec<Result<Box<dyn Any>, string>> {
        return self.plugins.iter().map(|p| p.execute(input)).collect();
    }
}
```

### Separation of Concerns

```fusion
// src/models/user.fu — Domain model only
pub struct User {
    pub id: int,
    pub name: string,
    pub email: string,
}

// src/repositories/user_repo.fu — Data access
pub struct UserRepo {
    db: Database,
}

impl UserRepo {
    pub fn find_by_id(&self, id: int) -> Result<User, string> { ... }
    pub fn save(&self, user: &User) -> Result<(), string> { ... }
}

// src/services/auth.fu — Business logic
pub struct AuthService {
    user_repo: UserRepo,
    config: AuthConfig,
}

impl AuthService {
    pub fn login(&self, email: string, password: string) -> Result<Session, string> { ... }
}
```

---

## Metaprogramming

### Generics / Parametric Polymorphism

#### Generic Functions

```fusion
fn identity<T>(value: T) -> T {
    return value;
}

fn max_of_two<T: Comparable>(a: T, b: T) -> T {
    if a > b { return a; } else { return b; }
}

fn map_array<T, U>(arr: [T], f: fn(T) -> U) -> [U] {
    let mut result: [U] = [];
    for item in arr {
        result.push(f(item));
    }
    return result;
}

fn main() -> int {
    let x = identity(42);
    let y = identity("hello");
    let m = max_of_two(3, 7);

    let doubled = map_array([1, 2, 3], |x| x * 2);
    println("Doubled: %s", doubled.to_string());

    return 0;
}
```

#### Generic Structs

```fusion
struct Stack<T> {
    elements: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Stack<T> {
        return Stack { elements: Vec::new() };
    }

    fn push(mut self, item: T) {
        self.elements.push(item);
    }

    fn pop(mut self) -> Option<T> {
        return self.elements.pop();
    }

    fn peek(&self) -> Option<&T> {
        return self.elements.last();
    }

    fn is_empty(&self) -> bool {
        return self.elements.len() == 0;
    }
}

// Generic struct with multiple type parameters
struct Pair<A, B> {
    first: A,
    second: B,
}

impl<A, B> Pair<A, B> {
    fn new(first: A, second: B) -> Pair<A, B> {
        return Pair { first, second };
    }

    fn swap(self) -> Pair<B, A> {
        return Pair { first: self.second, second: self.first };
    }
}

fn main() -> int {
    let mut stack: Stack<int> = Stack::new();
    stack.push(10);
    stack.push(20);

    let top = stack.pop();
    match top {
        Some(val) => println("Top: %d", val),
        None => println("Stack is empty"),
    }

    let p: Pair<int, string> = Pair::new(42, "hello");
    let swapped = p.swap();
    println("Swapped: %s, %d", swapped.first, swapped.second);

    return 0;
}
```

#### Trait Bounds

```fusion
trait Printable {
    fn to_string(&self) -> string;
}

trait Serializable {
    fn serialize(&self) -> bytes;
}

// Single trait bound
fn print_item<T: Printable>(item: &T) {
    println("%s", item.to_string());
}

// Multiple trait bounds
fn process<T: Printable + Serializable>(item: &T) {
    println("Display: %s", item.to_string());
    let data = item.serialize();
    println("Bytes: %s", data.to_hex());
}

// Where clause for complex bounds
fn complex_fn<T, U>(t: T, u: U) -> string
where
    T: Printable + Clone,
    U: Serializable + Debug,
{
    return t.to_string();
}

// Associated type bounds
fn get_first<T: Iterator>(iter: T) -> Option<T::Item>
where
    T::Item: Printable,
{
    return iter.into_iter().next();
}
```

#### Monomorphization

Fusion uses monomorphization to generate specialized code for each concrete type:

```fusion
fn add<T: Addable>(a: T, b: T) -> T {
    return a + b;
}

// Usage
let x = add(1, 2);          // generates add_int
let y = add(1.0, 2.0);      // generates add_float
let z = add("a", "b");      // generates add_string

// The compiler produces specialized versions:
// fn add_int(a: int, b: int) -> int { return a + b; }
// fn add_float(a: float, b: float) -> float { return a + b; }
// fn add_string(a: string, b: string) -> string { return a + b; }
```

### Macros / Compile-time Execution

#### Declarative Macros

Declarative macros use pattern matching to generate code:

```fusion
// Define a macro with pattern matching
macro_rules! vec_of {
    // Match: vec![expr, expr, ...]
    ($($item:expr),* $(,)?) => {{
        let mut v = Vec::new();
        $(v.push($item);)*
        v
    }};
}

// Define a macro for struct construction
macro_rules! new_point {
    ($x:expr, $y:expr) => {{
        Point { x: $x, y: $y }
    }};
}

// Define a macro for repeated code
macro_rules! implement_debug {
    ($type:ty) => {
        impl Debug for $type {
            fn fmt(&self, f: &mut Formatter) -> string {
                return format!("%s({:?}", std::any::type_name::<Self>(), self);
            }
        }
    };
}

fn main() -> int {
    let numbers = vec_of![1, 2, 3, 4, 5];
    let point = new_point!(10, 20);

    println("Numbers: %s", numbers.to_string());
    println("Point: (%d, %d)", point.x, point.y);

    return 0;
}
```

#### Procedural Macros

Procedural macros operate on the token stream and generate code:

```fusion
// Derive macro — auto-implements traits
#[derive(Debug, Clone, PartialEq)]
struct User {
    name: string,
    age: int,
}

// Attribute macro — modifies the annotated item
#[log_calls]
fn important_function(x: int) -> int {
    return x * 2;
}

// Function-like macro — acts like a function at compile time
let sql = sql!(SELECT * FROM users WHERE id = {user_id});

// Custom derive macro definition
macro derive Serialize {
    fn expand(struct_def: TokenStream) -> TokenStream {
        // Parse the struct definition
        let name = struct_def.name;
        let fields = struct_def.fields;

        // Generate serialize implementation
        return quote! {
            impl Serialize for #name {
                fn serialize(&self) -> JsonValue {
                    let mut map = JsonObject::new();
                    #(
                        map.insert(stringify!(#fields), self.#fields.serialize());
                    )*
                    return JsonValue::Object(map);
                }
            }
        };
    }
}
```

#### Macro Expansion Examples

```fusion
// Before macro expansion:
macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut m = HashMap::new();
        $(m.insert($key, $value);)*
        m
    }};
}

// Usage:
let scores = hashmap! {
    "Alice" => 95,
    "Bob" => 87,
    "Charlie" => 92,
};

// After macro expansion:
// {
//     let mut m = HashMap::new();
//     m.insert("Alice", 95);
//     m.insert("Bob", 87);
//     m.insert("Charlie", 92);
//     m
// }
```

### Reflection / Introspection

#### Runtime Type Inspection

```fusion
use std::any::{type_name, TypeId};

fn inspect<T: 'static>(value: &T) {
    println("Type: %s", type_name::<T>());
    println("TypeId: %v", TypeId::of::<T>());
}

struct User {
    name: string,
    age: int,
}

fn main() -> int {
    let u = User { name: "Alice".to_string(), age: 30 };
    inspect(&u);
    // Output: Type: User
    //         TypeId: TypeId(0x1a2b3c...)

    inspect(&42);
    // Output: Type: int

    inspect(&"hello");
    // Output: Type: &str

    return 0;
}
```

#### Dynamic Dispatch

```fusion
trait Drawable {
    fn draw(&self);
    fn bounding_box(&self) -> Rect;
}

struct Circle {
    center: Point,
    radius: float,
}

struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

impl Drawable for Circle {
    fn draw(&self) {
        println("Drawing circle at (%f, %f) r=%f", self.center.x, self.center.y, self.radius);
    }

    fn bounding_box(&self) -> Rect {
        return Rect::from_center(self.center, self.radius * 2.0, self.radius * 2.0);
    }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println("Drawing rectangle from (%f,%f) to (%f,%f)",
            self.top_left.x, self.top_left.y,
            self.bottom_right.x, self.bottom_right.y);
    }

    fn bounding_box(&self) -> Rect {
        return Rect::new(self.top_left, self.bottom_right);
    }
}

// Store heterogeneous collections via trait objects
fn render_all(shapes: &[&dyn Drawable]) {
    for shape in shapes {
        let bb = shape.bounding_box();
        println("Bounding box: %v", bb);
        shape.draw();
    }
}

fn main() -> int {
    let c = Circle { center: Point::new(0.0, 0.0), radius: 5.0 };
    let r = Rectangle {
        top_left: Point::new(-2.0, -2.0),
        bottom_right: Point::new(2.0, 2.0),
    };

    let shapes: Vec<&dyn Drawable> = vec![&c, &r];
    render_all(&shapes);

    return 0;
}
```

#### JSON Serialization / Deserialization

```fusion
use std::reflect::{self, Reflect, FieldInfo};

#[derive(Reflect, Serialize, Deserialize)]
struct Config {
    host: string,
    port: int,
    debug: bool,
    tags: Vec<string>,
}

fn main() -> int {
    // Serialization — inspect fields at runtime
    let config = Config {
        host: "localhost".to_string(),
        port: 8080,
        debug: true,
        tags: vec_of!["web", "api"],
    };

    // Serialize to JSON via reflection
    let json = reflect::to_json(&config);
    println("JSON: %s", json);

    // Deserialize from JSON
    let parsed: Config = reflect::from_json(json);
    println("Host: %s, Port: %d", parsed.host, parsed.port);

    // Iterate fields dynamically
    for field in reflect::fields::<Config>() {
        println("Field: %s (type: %s)", field.name, field.type_name);
    }

    return 0;
}
```

#### Field and Method Reflection

```fusion
use std::reflect;

struct Player {
    pub name: string,
    pub health: int,
    pub score: int,
    private_key: string,
}

impl Player {
    pub fn new(name: string) -> Player {
        return Player {
            name,
            health: 100,
            score: 0,
            private_key: "secret".to_string(),
        };
    }

    pub fn take_damage(&mut self, amount: int) {
        self.health = self.health - amount;
    }

    pub fn add_score(&mut self, points: int) {
        self.score = self.score + points;
    }
}

fn inspect_type<T: 'static>() {
    let type_info = reflect::type_info::<T>();
    println("Type: %s", type_info.name);

    for field in type_info.fields {
        println("  Field: %s — %s (public: %b)", field.name, field.type_name, field.is_public);
    }

    for method in type_info.methods {
        println("  Method: %s", method.name);
    }
}

fn main() -> int {
    inspect_type::<Player>();
    // Output:
    //   Type: Player
    //     Field: name — string (public: true)
    //     Field: health — int (public: true)
    //     Field: score — int (public: true)
    //     Field: private_key — string (public: false)
    //     Method: new
    //     Method: take_damage
    //     Method: add_score

    // Dynamic field access
    let mut player = Player::new("Hero".to_string());
    let health_field = reflect::get_field(&player, "health");
    println("Health: %v", health_field);

    reflect::set_field(&mut player, "score", 100);
    println("Score: %d", player.score);

    return 0;
}
```

---

## Code Examples

### Module Organization Example

```
my_app/
├── fusion.toml
├── src/
│   ├── main.fu
│   ├── lib.fu
│   ├── models/
│   │   ├── mod.fu
│   │   ├── user.fu
│   │   └── post.fu
│   ├── services/
│   │   ├── mod.fu
│   │   ├── auth.fu
│   │   └── content.fu
│   └── utils/
│       ├── mod.fu
│       └── validation.fu
└── tests/
    ├── mod.fu
    └── integration.fu
```

```fusion
// src/lib.fu
pub mod models;
pub mod services;
pub mod utils;

// Re-export the public API
pub use services::{auth, content};
pub use models::{User, Post};
```

```fusion
// src/models/mod.fu
pub mod user;
pub mod post;

pub use user::User;
pub use post::Post;
```

```fusion
// src/models/user.fu
pub struct User {
    pub id: int,
    pub name: string,
    pub email: string,
}

impl User {
    pub fn validate(&self) -> bool {
        return self.email.contains("@") && self.name.len() > 0;
    }
}
```

```fusion
// src/main.fu
use my_app::{User, Post};
use my_app::services::{auth, content};

fn main() -> int {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    if user.validate() {
        let session = auth::login(&user);
        println("Welcome, %s!", user.name);
    }

    return 0;
}
```

### Generic Data Structure

```fusion
use std::hash::Hash;

// Generic ordered map with O(log n) operations
struct OrderedMap<K: Comparable + Hash, V> {
    root: Option<Box<TreeNode<K, V>>>,
    size: int,
}

struct TreeNode<K, V> {
    key: K,
    value: V,
    left: Option<Box<TreeNode<K, V>>>,
    right: Option<Box<TreeNode<K, V>>>,
}

impl<K: Comparable + Hash, V> OrderedMap<K, V> {
    fn new() -> OrderedMap<K, V> {
        return OrderedMap { root: None, size: 0 };
    }

    fn insert(mut self, key: K, value: V) {
        self.root = Self::insert_node(self.root, key, value);
        self.size = self.size + 1;
    }

    fn get(&self, key: &K) -> Option<&V> {
        return Self::find_node(&self.root, key);
    }

    fn len(&self) -> int {
        return self.size;
    }

    fn keys(&self) -> Vec<&K> {
        let mut result = Vec::new();
        Self::collect_keys(&self.root, &mut result);
        return result;
    }

    fn insert_node(
        node: Option<Box<TreeNode<K, V>>>,
        key: K,
        value: V,
    ) -> Option<Box<TreeNode<K, V>>> {
        match node {
            None => Some(Box::new(TreeNode {
                key,
                value,
                left: None,
                right: None,
            })),
            Some(mut n) => {
                if key < n.key {
                    n.left = Self::insert_node(n.left, key, value);
                } else if key > n.key {
                    n.right = Self::insert_node(n.right, key, value);
                } else {
                    n.value = value;
                }
                Some(n)
            }
        }
    }

    fn find_node<'a>(
        node: &'a Option<Box<TreeNode<K, V>>>,
        key: &K,
    ) -> Option<&'a V> {
        match node {
            None => None,
            Some(n) => {
                if key < &n.key {
                    Self::find_node(&n.left, key)
                } else if key > &n.key {
                    Self::find_node(&n.right, key)
                } else {
                    Some(&n.value)
                }
            }
        }
    }

    fn collect_keys<'a>(node: &'a Option<Box<TreeNode<K, V>>>, result: &mut Vec<&'a K>) {
        if let Some(n) = node {
            Self::collect_keys(&n.left, result);
            result.push(&n.key);
            Self::collect_keys(&n.right, result);
        }
    }
}

fn main() -> int {
    let mut map: OrderedMap<string, int> = OrderedMap::new();
    map.insert("apple".to_string(), 5);
    map.insert("banana".to_string(), 3);
    map.insert("cherry".to_string(), 7);

    let keys = map.keys();
    for key in keys {
        println("%s: %d", key, map.get(key).unwrap());
    }

    return 0;
}
```

### Macro Definition and Usage

```fusion
// Utility macros for common patterns

// Benchmarking macro
macro_rules! benchmark {
    ($name:expr, $body:block) => {{
        let start = std::time::Instant::now();
        let result = $body;
        let elapsed = start.elapsed();
        println("[BENCH] %s: %v (%v ns)", $name, elapsed.as_secs_f64(), elapsed.as_nanos());
        result
    }};
}

// Retry macro with exponential backoff
macro_rules! retry {
    ($max_attempts:expr, $body:block) => {{
        let mut attempt = 0;
        let mut delay_ms = 100;
        loop {
            attempt = attempt + 1;
            match (|| -> Result<_, string> { $body })() {
                Ok(val) => break Ok(val),
                Err(e) => {
                    if attempt >= $max_attempts {
                        break Err(format!("Failed after %d attempts: %s", $max_attempts, e));
                    }
                    println("Attempt %d failed: %s, retrying in %dms...", attempt, e, delay_ms);
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    delay_ms = delay_ms * 2;
                }
            }
        }
    }};
}

// Builder pattern macro
macro_rules! builder {
    ($name:ident { $($field:ident : $type:ty),* $(,)? }) => {
        struct $name {
            $($field: $type,)*
        }

        struct ${concat($name, Builder)} {
            $($field: Option<$type>,)*
        }

        impl ${concat($name, Builder)} {
            fn new() -> Self {
                return Self { $($field: None,)* };
            }

            $(
                fn $field(mut self, value: $type) -> Self {
                    self.$field = Some(value);
                    return self;
                }
            )*

            fn build(self) -> Result<$name, string> {
                return Ok($name {
                    $($field: self.$field.ok_or(format!("{} is required", stringify!($field)))?,)*
                });
            }
        }
    };
}

// Usage
builder!(ServerConfig {
    host: string,
    port: int,
    max_connections: int,
});

fn main() -> int {
    // Benchmarking
    let result = benchmark!("fibonacci", {
        fn fib(n: int) -> int {
            if n <= 1 { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(30)
    });
    println("fib(30) = %d", result);

    // Retry logic
    let data = retry!(3, {
        std::fs::read_to_string("config.toml")
    });

    // Builder pattern
    let config = ServerConfigBuilder::new()
        .host("localhost".to_string())
        .port(8080)
        .max_connections(100)
        .build()
        .unwrap();

    println("Server: %s:%d", config.host, config.port);

    return 0;
}
```

### Reflection-based Serialization

```fusion
use std::reflect::{self, Reflect, FieldVisitor};

#[derive(Reflect)]
struct DatabaseConfig {
    host: string,
    port: int,
    username: string,
    password: string,
    max_pool_size: int,
    ssl_enabled: bool,
}

// Generic serialization via reflection
fn to_json<T: Reflect>(value: &T) -> string {
    let mut parts: Vec<string> = Vec::new();

    for field in reflect::fields::<T>() {
        let field_value = reflect::get_field(value, field.name);
        let json_value = reflect::value_to_json(field_value);
        parts.push(format!("\"%s\": %s", field.name, json_value));
    }

    return format!("{ %s }", parts.join(", "));
}

// Generic deserialization via reflection
fn from_json<T: Reflect>(json: string) -> Result<T, string> {
    let mut result = reflect::create_default::<T>();
    let map = reflect::parse_json_object(json)?;

    for field in reflect::fields::<T>() {
        if let Some(value) = map.get(field.name) {
            reflect::set_field(&mut result, field.name, value.clone());
        }
    }

    return Ok(result);
}

// Filter sensitive fields
fn to_safe_json<T: Reflect>(value: &T, exclude_fields: &[string]) -> string {
    let mut parts: Vec<string> = Vec::new();

    for field in reflect::fields::<T>() {
        if exclude_fields.contains(&field.name.to_string()) {
            continue;
        }
        let field_value = reflect::get_field(value, field.name);
        let json_value = reflect::value_to_json(field_value);
        parts.push(format!("\"%s\": %s", field.name, json_value));
    }

    return format!("{ %s }", parts.join(", "));
}

fn main() -> int {
    let config = DatabaseConfig {
        host: "db.example.com".to_string(),
        port: 5432,
        username: "admin".to_string(),
        password: "secret123".to_string(),
        max_pool_size: 20,
        ssl_enabled: true,
    };

    // Full serialization
    let full_json = to_json(&config);
    println("Full: %s", full_json);

    // Safe serialization (exclude password)
    let safe_json = to_safe_json(&config, &["password".to_string()]);
    println("Safe: %s", safe_json);

    // Deserialization
    let parsed = from_json::<DatabaseConfig>(full_json).unwrap();
    println("Host: %s, Port: %d", parsed.host, parsed.port);

    return 0;
}
```
