# Chapter 27: Null/Nil Handling in Fusion

Tony Hoare called null references his "billion-dollar mistake." Fusion eliminates this entire class of bugs through its type system. This chapter covers how Fusion handles absent values and errors without null.

## The Billion Dollar Mistake

### History of Null Pointer References

In 1965, Tony Hoare introduced null references while designing ALGOL W. He later called it his "billion-dollar mistake":

> "I couldn't resist the temptation to put in a null reference, simply because it was so easy to implement. This has led to innumerable errors, vulnerabilities, and system crashes, which have probably caused a billion dollars of pain and damage in the last forty years."

The problem is fundamental: null is a valid value that violates type assumptions. When you have a `String`, you expect it to contain characters. When you have an `Int`, you expect a number. But null can masquerade as any type while being fundamentally different.

### Why Null is Dangerous

```fusion
// This is what null does to your program:
// It silently propagates errors through your entire system

// In Java:
// String name = getUserName(userId);  // returns null
// int length = name.length();  // NullPointerException!

// In Python:
// def get_user(id): return None  # Sometimes
// name = get_user(123).name  # AttributeError!

// The insidious part: null can appear ANYWHERE
struct User {
    name: String,      // Could be null in Java
    email: String,     // Could be null in Java
    preferences: Map,  // Could be null in Java
}

// Every field access is a potential crash site
```

The costs are staggering:
- **Runtime crashes**: NullPointerException is the most common runtime exception in Java
- **Security vulnerabilities**: Null dereferences have led to remote code execution bugs
- **Defensive coding**: Programmers add null checks everywhere, bloating code
- **Lost information**: Null conflates "no value" with "error occurred"
- **Type system holes**: Null can inhabit any reference type, undermining type safety

## Fusion's Approach

### No Null/Nil in the Type System

Fusion simply doesn't have null. Every value of a type is a valid instance of that type. There's no special "null" value that can sneak in.

```fusion
// In Fusion, this is impossible:
let name: String = null;  // Compile error: null is not a value

// Every String is a real String
let name: String = "Alice";  // Always valid

// If you need to represent "no value", you use Option<T>
let maybe_name: Option<String> = None;  // Explicitly absent
```

### Option<T> for Absent Values

Option is a sum type that represents a value that might or might not exist:

```fusion
// Option<T> is defined as:
enum Option<T> {
    Some(T),
    None,
}

// Usage
let some_number: Option<i32> = Some(42);
let no_number: Option<i32> = None;

// The compiler forces you to handle both cases
match some_number {
    Some(n) => println!("Got: {}", n),
    None => println!("No value"),
}
```

### Result<T, E> for Errors

Result represents an operation that might fail:

```fusion
// Result<T, E> is defined as:
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Usage
let result: Result<i32, String> = Ok(42);
let error: Result<i32, String> = Err("Something went wrong".into());

// The compiler forces you to handle both cases
match result {
    Ok(value) => println!("Success: {}", value),
    Err(e) => println!("Error: {}", e),
}
```

### Default Values via unwrap_or

For cases where you know a default value is appropriate:

```fusion
let name: Option<String> = None;

// Provide a default if None
let display_name = name.unwrap_or("Anonymous".into());
println!("{}", display_name);  // "Anonymous"

// Compute default lazily
let value = name.unwrap_or_else(|| {
    println!("Computing default...");
    "Default".into()
});

// Panic if None (use sparingly!)
let value = name.expect("Name must be present");  // Panics if None
```

## Option<T> Deep Dive

### Some and None Variants

```fusion
// Creating Options
let present: Option<i32> = Some(42);
let absent: Option<i32> = None;

// From nullable operations (in FFI contexts)
let maybe_value: Option<&str> = ffi_function().as_ref();

// From collections
let first: Option<&i32> = vec![1, 2, 3].first();
let last: Option<&i32> = vec![1, 2, 3].last();

// From maps
let value: Option<&String> = map.get("key");

// From parsing
let number: Option<i32> = "42".parse().ok();
```

### Pattern Matching on Option

Pattern matching is the primary way to handle Option:

```fusion
fn process_value(value: Option<i32>) -> String {
    match value {
        Some(0) => "Zero".into(),
        Some(n) if n > 0 => format!("Positive: {}", n),
        Some(n) => format!("Negative: {}", n),
        None => "Missing".into(),
    }
}

// if let for when you only care about Some
if let Some(value) = maybe_value {
    println!("Got: {}", value);
}

// while let for loops
while let Some(item) = iterator.next() {
    process(item);
}
```

### Option Combinators

Option provides methods for chaining operations:

```fusion
let input: Option<String> = Some("  42  ".into());

// map: Transform the inner value
let number: Option<i32> = input.map(|s| s.trim().parse().ok()).flatten();
// Equivalent to: input.and_then(|s| s.trim().parse().ok())

// and_then: Chain operations that return Option
let result: Option<i32> = input
    .and_then(|s| s.trim().parse().ok())
    .and_then(|n| if n > 0 { Some(n) } else { None });

// or_else: Provide alternative if None
let value: Option<i32> = None.or_else(|| Some(42));

// map_or: Transform or use default
let display: String = input.map_or("None".into(), |s| s.trim().to_string());

// filter: Keep only if predicate matches
let positive: Option<i32> = Some(-5).filter(|&n| n > 0);  // None
```

### Optional Chaining

```fusion
// Deep access without null checks
struct User {
    profile: Option<Profile>,
}

struct Profile {
    address: Option<Address>,
}

struct Address {
    city: Option<String>,
}

fn get_city(user: &User) -> Option<&str> {
    // With null, this would be:
    // user.profile?.address?.city  (still could crash)
    
    // In Fusion, this is:
    user.profile.as_ref()
        .and_then(|p| p.address.as_ref())
        .and_then(|a| a.city.as_ref())
        .map(|s| s.as_str())
}

// Equivalent with flat_map
fn get_city_flat(user: &User) -> Option<&str> {
    user.profile.as_ref()
        .flat_map(|p| p.address.as_ref())
        .flat_map(|a| a.city.as_ref())
        .map(|s| s.as_str())
}
```

## Result<T, E> Deep Dive

### Ok and Err Variants

```fusion
// Creating Results
let success: Result<i32, String> = Ok(42);
let failure: Result<i32, String> = Err("Failed".into());

// From operations that might fail
let parsed: Result<i32, _> = "42".parse();
let file: Result<File, _> = File::open("data.txt");

// From Option
let value: Option<i32> = Some(42);
let result: Result<i32, String> = value.ok_or("Missing value".into());

// Converting between Option and Result
let option: Option<i32> = Some(42);
let result: Result<i32, &str> = option.ok_or("Missing");
```

### Pattern Matching on Result

```fusion
fn process_result(result: Result<i32, Error>) -> String {
    match result {
        Ok(value) => format!("Got: {}", value),
        Err(Error::NotFound) => "Not found".into(),
        Err(Error::PermissionDenied) => "Access denied".into(),
        Err(e) => format!("Error: {}", e),
    }
}

// if let for success handling
if let Ok(value) = operation() {
    println!("Success: {}", value);
}

// Let-else for early returns
fn process() -> Result<(), Error> {
    let value = operation().map_err(|e| {
        println!("Failed: {}", e);
        e
    })?;
    
    // Continue with value
    Ok(())
}
```

### Error Propagation with ?

The `?` operator propagates errors up the call stack:

```fusion
fn read_config(path: &str) -> Result<Config, ConfigError> {
    // ? propagates io::Error as ConfigError::Io
    let content = fs::read_to_string(path)?;
    
    // ? propagates parse::Error as ConfigError::Parse
    let config: Config = content.parse()?;
    
    // Manual propagation
    if !config.is_valid() {
        return Err(ConfigError::Invalid);
    }
    
    Ok(config)
}

// ? works with any type implementing Into<Error>
fn complex_operation() -> Result<Data, AppError> {
    let config = read_config("app.toml")?;  // ConfigError -> AppError
    let db = Database::connect(&config.db_url)?;  // DbError -> AppError
    let data = db.query("SELECT * FROM users")?;  // QueryError -> AppError
    
    Ok(data)
}
```

### Result Combinators

```fusion
let result: Result<i32, String> = Ok(42);

// map: Transform the success value
let doubled: Result<i32, String> = result.map(|n| n * 2);

// map_err: Transform the error value
let formatted: Result<i32, String> = result.map_err(|e| format!("Error: {}", e));

// and_then: Chain operations that return Result
let processed: Result<String, String> = result
    .and_then(|n| {
        if n > 0 {
            Ok(n.to_string())
        } else {
            Err("Not positive".into())
        }
    });

// or_else: Try alternative on error
let fallback: Result<i32, String> = Err("Failed".into())
    .or_else(|_| Ok(42));

// unwrap_or_else: Compute default on error
let value: i32 = Err("Failed".into())
    .unwrap_or_else(|e| {
        println!("Using default due to: {}", e);
        0
    });
```

### Unwrap with Default Values

```fusion
// Safe unwrapping patterns
let config_value: Option<String> = get_config("timeout");

// Method 1: unwrap_or with literal
let timeout: u64 = config_value
    .and_then(|s| s.parse().ok())
    .unwrap_or(30);

// Method 2: unwrap_or_else for expensive defaults
let timeout: u64 = config_value
    .and_then(|s| s.parse().ok())
    .unwrap_or_else(|| {
        println!("Using default timeout");
        30
    });

// Method 3: Default trait
let timeout: u64 = config_value
    .and_then(|s| s.parse().ok())
    .unwrap_or_default();  // 0 for u64

// Method 4: Pattern matching for complex defaults
let timeout: u64 = match config_value {
    Some(s) if s == "fast" => 10,
    Some(s) if s == "slow" => 60,
    Some(s) => s.parse().unwrap_or(30),
    None => 30,
};
```

## Comparison with Other Languages

### Java: null + NullPointerException

```java
// Java allows null in any reference type
String name = null;  // Legal
int length = name.length();  // NullPointerException at runtime

// Defensive coding required everywhere
if (name != null) {
    int length = name.length();
    // But what if name becomes null between check and use? (TOCTOU)
}

// Optional (Java 8+) helps but isn't enforced
Optional<String> optName = Optional.ofNullable(name);
String result = optName.orElse("default");
```

### Python: None + AttributeError

```python
# Python allows None anywhere
name = None  # Legal
length = len(name)  # TypeError: object of type 'NoneType' has no len()

# No static type checking by default
def get_name(id: int) -> str:  # Return type is a lie
    return None  # Actually returns None

# Type hints help but aren't enforced
from typing import Optional
def get_name(id: int) -> Optional[str]:
    return None  # Now honest
```

### Rust: Option + Result

```rust
// Rust has no null, uses Option and Result
let name: Option<String> = Some("Alice".into());
let length: usize = name.unwrap().len();  // Panics if None

// Must handle explicitly
match name {
    Some(n) => println!("Length: {}", n.len()),
    None => println!("No name"),
}

// ? operator for error propagation
fn read_file(path: &str) -> Result<String, io::Error> {
    let content = fs::read_to_string(path)?;  // Propagates error
    Ok(content)
}
```

### Fusion: Option + Result + Linear Types

```fusion
// Fusion combines Rust-style Option/Result with linear types
let name: Option<String> = Some("Alice".into());

// Linear types ensure resources are used exactly once
let file = File::open("data.txt")?;
let content = file.read()?;  // file is moved, can't use again
// file.read()?;  // Compile error: file already moved

// Pattern matching with linear types
match name {
    Some(n) => {
        // n is bound, name is consumed
        println!("Name: {}", n);
    }
    None => {
        // name is consumed
        println!("No name");
    }
}
// name is no longer accessible (moved in match)
```

## Code Examples

### Safe Null Handling Patterns

```fusion
// Pattern 1: Default values
fn get_display_name(user: &User) -> String {
    user.nickname
        .clone()
        .unwrap_or_else(|| user.name.clone())
        .unwrap_or("Anonymous".into())
}

// Pattern 2: Collection operations
fn find_active_users(users: &[User]) -> Vec<&User> {
    users.iter()
        .filter(|u| u.status == Some(Status::Active))
        .collect()
}

// Pattern 3: Optional chaining for deep access
fn get_user_city(user: &User) -> Option<String> {
    user.address.as_ref()
        .and_then(|a| a.city.as_ref())
        .cloned()
}

// Pattern 4: Builder pattern with optional fields
struct RequestBuilder {
    url: Option<String>,
    method: Option<Method>,
    headers: Option<HashMap<String, String>>,
}

impl RequestBuilder {
    fn new() -> Self {
        Self {
            url: None,
            method: None,
            headers: None,
        }
    }
    
    fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
    
    fn build(self) -> Result<Request, RequestError> {
        let url = self.url.ok_or(RequestError::MissingUrl)?;
        let method = self.method.unwrap_or(Method::GET);
        
        Ok(Request {
            url,
            method,
            headers: self.headers.unwrap_or_default(),
        })
    }
}
```

### Error Recovery Patterns

```fusion
// Pattern 1: Retry with fallback
fn read_with_retry(path: &str, retries: u32) -> Result<String, io::Error> {
    let mut last_error = None;
    
    for _ in 0..retries {
        match fs::read_to_string(path) {
            Ok(content) => return Ok(content),
            Err(e) => {
                last_error = Some(e);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    
    Err(last_error.unwrap())
}

// Pattern 2: Partial success
fn process_batch(items: Vec<Item>) -> (Vec<Result<Output, Error>>, Vec<Item>) {
    let mut successes = vec![];
    let mut failures = vec![];
    
    for item in items {
        match process(item.clone()) {
            Ok(output) => successes.push(Ok(output)),
            Err(e) => {
                successes.push(Err(e));
                failures.push(item);
            }
        }
    }
    
    (successes, failures)
}

// Pattern 3: Error aggregation
fn validate_user(user: &User) -> Result<(), Vec<ValidationError>> {
    let mut errors = vec![];
    
    if user.name.is_empty() {
        errors.push(ValidationError::EmptyName);
    }
    
    if user.email.is_none() {
        errors.push(ValidationError::MissingEmail);
    }
    
    if let Some(age) = user.age {
        if age < 0 || age > 150 {
            errors.push(ValidationError::InvalidAge);
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### Resource Cleanup with Option/Result

```fusion
// Pattern 1: RAII with linear types
fn process_file(path: &str) -> Result<String, io::Error> {
    let file = File::open(path)?;  // File opened
    let content = file.read()?;     // File read, file handle dropped here
    Ok(content)
    // File is automatically closed when dropped
}

// Pattern 2: Cleanup on error
fn complex_operation() -> Result<Data, AppError> {
    let temp_dir = TempDir::new().map_err(AppError::Io)?;
    let temp_file = temp_dir.create_file("temp.txt").map_err(AppError::Io)?;
    
    // If we fail here, temp_dir is automatically cleaned up
    let data = process(&temp_file)?;
    
    // If we fail here, temp_dir is still cleaned up
    let result = transform(data)?;
    
    // temp_dir is cleaned up when we return
    Ok(result)
}

// Pattern 3: Option for optional resources
struct Connection {
    socket: Option<TcpStream>,
}

impl Connection {
    fn connect(&mut self, addr: SocketAddr) -> Result<(), io::Error> {
        let stream = TcpStream::connect(addr)?;
        self.socket = Some(stream);
        Ok(())
    }
    
    fn send(&mut self, data: &[u8]) -> Result<(), io::Error> {
        match &mut self.socket {
            Some(stream) => {
                stream.write_all(data)?;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Not connected",
            )),
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Socket is automatically closed when dropped
        if let Some(socket) = self.socket.take() {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}
```

## Advanced Patterns

### The Builder Pattern with Required Fields

```fusion
// Compile-time enforcement of required fields
struct EmailBuilder<HasSubject = No, HasBody = No> {
    subject: Option<String>,
    body: Option<String>,
    to: Vec<String>,
    _phantom: PhantomData<(HasSubject, HasBody)>,
}

struct Yes;
struct No;

impl EmailBuilder {
    fn new() -> Self {
        Self {
            subject: None,
            body: None,
            to: vec![],
            _phantom: PhantomData,
        }
    }
}

impl<HasBody> EmailBuilder<No, HasBody> {
    fn subject(self, subject: impl Into<String>) -> EmailBuilder<Yes, HasBody> {
        EmailBuilder {
            subject: Some(subject.into()),
            body: self.body,
            to: self.to,
            _phantom: PhantomData,
        }
    }
}

impl<HasSubject> EmailBuilder<HasSubject, No> {
    fn body(self, body: impl Into<String>) -> EmailBuilder<HasSubject, Yes> {
        EmailBuilder {
            subject: self.subject,
            body: Some(body.into()),
            to: self.to,
            _phantom: PhantomData,
        }
    }
}

impl EmailBuilder<Yes, Yes> {
    fn send(self) -> Result<(), SendError> {
        // Required fields are guaranteed present
        let subject = self.subject.unwrap();
        let body = self.body.unwrap();
        
        // Send email...
        Ok(())
    }
}

// Usage
let email = EmailBuilder::new()
    .to("user@example.com")
    .subject("Hello")  // Now EmailBuilder<Yes, No>
    .body("World")      // Now EmailBuilder<Yes, Yes>
    .send()?;           // Compiles: all required fields present

// This won't compile:
// EmailBuilder::new().to("user@example.com").send()?;  // Error: missing subject and body
```

### Typestate Pattern

```fusion
// Compile-time state machine
struct Connection<State> {
    socket: Option<TcpStream>,
    _state: PhantomData<State>,
}

struct Disconnected;
struct Connected;
struct Authenticated;

impl Connection<Disconnected> {
    fn connect(addr: SocketAddr) -> Result<Connection<Connected>, io::Error> {
        let socket = TcpStream::connect(addr)?;
        Ok(Connection {
            socket: Some(socket),
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    fn authenticate(self, credentials: &Credentials) -> Result<Connection<Authenticated>, AuthError> {
        let mut socket = self.socket.ok_or(AuthError::NoSocket)?;
        
        // Authentication logic...
        
        Ok(Connection {
            socket: Some(socket),
            _state: PhantomData,
        })
    }
}

impl Connection<Authenticated> {
    fn send(&mut self, data: &[u8]) -> Result<(), io::Error> {
        let socket = self.socket.as_mut().ok_or(io::ErrorKind::NotConnected)?;
        socket.write_all(data)
    }
}

// Usage
let conn = Connection::connect(addr)?;           // Disconnected -> Connected
let conn = conn.authenticate(&creds)?;           // Connected -> Authenticated
conn.send(b"hello")?;                           // Only works in Authenticated state

// This won't compile:
// conn.send(b"hello")?;  // Error: Connection<Disconnected> doesn't have send
```

### The Null Object Pattern

```fusion
// Null object as a valid implementation
trait Logger {
    fn log(&self, message: &str);
}

struct ConsoleLogger;
struct NullLogger;

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("{}", message);
    }
}

impl Logger for NullLogger {
    fn log(&self, _message: &str) {
        // Do nothing
    }
}

// Use Option<Logger> or a trait object
fn process_with_logging(logger: &dyn Logger, data: &[u8]) {
    logger.log("Processing started");
    // ...
    logger.log("Processing completed");
}

// Usage
let logger: Box<dyn Logger> = if verbose {
    Box::new(ConsoleLogger)
} else {
    Box::new(NullLogger)
};

process_with_logging(&*logger, &data);
```

### The Visitor Pattern with Option

```fusion
// Tree traversal with optional results
enum Tree<T> {
    Leaf(T),
    Node(Vec<Tree<T>>),
}

impl<T> Tree<T> {
    fn find(&self, predicate: impl Fn(&T) -> bool) -> Option<&T> {
        match self {
            Tree::Leaf(value) => {
                if predicate(value) {
                    Some(value)
                } else {
                    None
                }
            }
            Tree::Node(children) => {
                for child in children {
                    if let Some(found) = child.find(&predicate) {
                        return Some(found);
                    }
                }
                None
            }
        }
    }
    
    fn find_map<U>(&self, f: impl Fn(&T) -> Option<U>) -> Option<U> {
        match self {
            Tree::Leaf(value) => f(value),
            Tree::Node(children) => {
                for child in children {
                    if let Some(result) = child.find_map(&f) {
                        return Some(result);
                    }
                }
                None
            }
        }
    }
}

// Usage
let tree = Tree::Node(vec![
    Tree::Leaf(1),
    Tree::Node(vec![Tree::Leaf(2), Tree::Leaf(3)]),
    Tree::Leaf(4),
]);

let found = tree.find(|&x| x > 2);  // Some(3)
let doubled = tree.find_map(|&x| if x > 2 { Some(x * 2) } else { None });  // Some(6)
```

## Best Practices

### When to Use Option

- Function return types when "no value" is a valid outcome
- Fields that might not be initialized
- Collection lookups (get returns Option)
- Parsing operations that might fail to produce a value

### When to Use Result

- Operations that can fail with an error
- I/O operations
- Parsing that produces error messages
- Network operations

### When to Use Default Values

- Configuration with sensible defaults
- UI displays where missing data shouldn't crash
- Optional parameters with common values

### When to Panic

- Programmer errors (index out of bounds, unwrap on None when it's logically impossible)
- Invariant violations that should never happen
- During development/prototyping (but replace before production)

## Summary

Fusion's approach to null handling provides:

1. **Type Safety**: No null means no null pointer exceptions
2. **Explicit Absence**: Option<T> makes "no value" explicit and type-safe
3. **Error Handling**: Result<T, E> separates success from failure
4. **Composability**: Combinators enable fluent handling of optional values
5. **Performance**: Zero-cost abstractions for Option and Result
6. **Linear Types**: Resources are used exactly once, preventing double-free and use-after-free

The key insight: null is a hack that conflates "no value" with "error" with "invalid". Fusion's type system makes each case explicit, enabling the compiler to catch bugs at compile time rather than runtime.

In the next chapter, we'll explore advanced type system features that build on these foundations.