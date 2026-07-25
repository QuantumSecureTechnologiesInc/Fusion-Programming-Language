# Chapter 6: Standard Library

> I/O, strings, collections, filesystem, math, and process management

---

## I/O

### Basic Output

```fusion
fn main() -> int {
    // println with format string
    println("Hello, World!");
    println("Number: %d", 42);
    println("Float: %f", 3.14);
    println("String: %s", "Fusion");
    println("Char: %c", 'A');

    // Multiple values
    let name: string = "Fusion";
    let version: int = 2;
    println("%s v%d", name, version);

    return 0;
}
```

### Format Specifiers

| Specifier | Type | Example |
|-----------|------|---------|
| `%d` | int | `println("%d", 42)` |
| `%f` | float | `println("%f", 3.14)` |
| `%s` | string | `println("%s", "hello")` |
| `%c` | char | `println("%c", 'A')` |
| `%x` | hex | `println("%x", 255)` → `ff` |
| `%o` | octal | `println("%o", 8)` → `10` |
| `%b` | binary | `println("%b", 5)` → `101` |

### Reading Input

```fusion
fn main() -> int {
    print("Enter your name: ");
    let name: string = read_line();
    println("Hello, %s!", name);

    print("Enter your age: ");
    let age_input: string = read_line();
    // Convert string to int (simplified)
    let age: int = parse_int(age_input);
    println("You are %d years old", age);

    return 0;
}
```

### Standard I/O Module

```fusion
use std::io;

fn main() -> int {
    // Print to stdout
    io::print("Hello");
    io::println(" World");

    // Print to stderr
    io::eprint("Error: ");
    io::eprintln("something went wrong");

    // Read a line from stdin
    let line: string = io::read_line();

    return 0;
}
```

---

## Strings and String Utilities

### String Operations

```fusion
fn main() -> int {
    let s: string = "Hello, Fusion!";

    // Length
    let len: int = s.len();
    println("Length: %d", len);

    // Concatenation
    let greeting: string = "Hello" + ", " + "World!";
    println(greeting);

    // Repetition
    let repeated: string = "ha" * 3;  // "hahaha"
    println(repeated);

    // Substring
    let sub: string = s[0..5];  // "Hello"
    println(sub);

    // Contains
    let has_fusion: bool = s.contains("Fusion");
    println("Contains Fusion: %d", has_fusion);

    // StartsWith / EndsWith
    let starts: bool = s.starts_with("Hello");
    let ends: bool = s.ends_with("!");
    println("Starts with Hello: %d, Ends with !: %d", starts, ends);

    return 0;
}
```

### String Conversion

```fusion
fn main() -> int {
    // Int to string
    let num: int = 42;
    let num_str: string = num.to_string();
    println("Number as string: %s", num_str);

    // String to int
    let parsed: int = "123".parse_int();
    println("Parsed: %d", parsed);

    // Float to string
    let pi: float = 3.14159;
    let pi_str: string = pi.to_string();
    println("Pi: %s", pi_str);

    // Lowercase / Uppercase
    let upper: string = "hello".to_upper();
    let lower: string = "HELLO".to_lower();
    println("Upper: %s, Lower: %s", upper, lower);

    return 0;
}
```

### String Manipulation

```fusion
fn main() -> int {
    let s: string = "  Hello, World!  ";

    // Trim whitespace
    let trimmed: string = s.trim();
    println("Trimmed: '%s'", trimmed);

    // Split
    let csv: string = "apple,banana,cherry";
    let parts: [string] = csv.split(",");
    for part in parts {
        println("Part: %s", part);
    }

    // Join
    let words: [string; 3] = ["Hello", "World", "Fusion"];
    let joined: string = words.join(" ");
    println("Joined: %s", joined);

    // Replace
    let text: string = "Hello, World!";
    let replaced: string = text.replace("World", "Fusion");
    println("Replaced: %s", replaced);

    return 0;
}
```

---

## Collections

### Vector (Dynamic Array)

```fusion
fn main() -> int {
    // Create vector
    let mut v: Vec<int> = Vec::new();

    // Push elements
    v.push(1);
    v.push(2);
    v.push(3);

    // Access elements
    println("First: %d", v[0]);
    println("Length: %d", v.len());

    // Iterate
    for item in v {
        println("Item: %d", item);
    }

    // Pop
    let last: int = v.pop();
    println("Popped: %d", last);

    // Vector from literal
    let nums: Vec<int> = vec![1, 2, 3, 4, 5];
    println("Vector length: %d", nums.len());

    return 0;
}
```

### HashMap

```fusion
fn main() -> int {
    // Create HashMap
    let mut scores: HashMap<string, int> = HashMap::new();

    // Insert
    scores.insert("Alice", 95);
    scores.insert("Bob", 87);
    scores.insert("Charlie", 92);

    // Get
    let alice_score: Option<int> = scores.get("Alice");
    match alice_score {
        Some(score) => println("Alice: %d", score),
        None => println("Alice not found"),
    }

    // Check existence
    let has_bob: bool = scores.contains("Bob");
    println("Bob exists: %d", has_bob);

    // Iterate
    for (name, score) in scores {
        println("%s: %d", name, score);
    }

    // Remove
    scores.remove("Charlie");

    return 0;
}
```

### HashSet

```fusion
fn main() -> int {
    // Create HashSet
    let mut set: HashSet<int> = HashSet::new();

    // Insert
    set.insert(1);
    set.insert(2);
    set.insert(3);
    set.insert(2);  // Duplicate, won't be added

    // Check membership
    let has_two: bool = set.contains(&2);
    println("Contains 2: %d", has_two);

    // Length
    println("Size: %d", set.len());  // 3

    // Iterate
    for item in set {
        println("Item: %d", item);
    }

    // Set operations
    let mut other: HashSet<int> = HashSet::new();
    other.insert(3);
    other.insert(4);
    other.insert(5);

    let intersection: HashSet<int> = set.intersection(&other);
    let union: HashSet<int> = set.union(&other);

    return 0;
}
```

---

## Filesystem Operations

### Reading Files

```fusion
use std::fs;

fn main() -> int {
    // Read entire file
    let content: string = fs::read_to_string("data.txt");
    println("File content:\n%s", content);

    // Read lines
    let lines: Vec<string> = fs::read_lines("data.txt");
    for line in lines {
        println("Line: %s", line);
    }

    // Check if file exists
    let exists: bool = fs::exists("data.txt");
    println("File exists: %d", exists);

    return 0;
}
```

### Writing Files

```fusion
use std::fs;

fn main() -> int {
    // Write to file (overwrite)
    fs::write("output.txt", "Hello, World!\n");

    // Append to file
    fs::append("output.txt", "Second line\n");

    // Write with format
    let data: string = "Name: Fusion\nVersion: 2.0\n";
    fs::write("config.txt", data);

    println("Files written successfully");

    return 0;
}
```

### Directory Operations

```fusion
use std::fs;

fn main() -> int {
    // Create directory
    fs::create_dir("new_folder");

    // Create directory recursively
    fs::create_dir_all("path/to/nested/folder");

    // List directory contents
    let entries: Vec<string> = fs::read_dir(".");
    for entry in entries {
        println("Entry: %s", entry);
    }

    // Remove file
    fs::remove_file("temp.txt");

    // Remove directory
    fs::remove_dir("empty_folder");

    return 0;
}
```

---

## Math Operations

### Basic Math

```fusion
use std::math;

fn main() -> int {
    // Basic operations
    let a: int = 10;
    let b: int = 3;

    println("a + b = %d", a + b);
    println("a - b = %d", a - b);
    println("a * b = %d", a * b);
    println("a / b = %d", a / b);
    println("a %% b = %d", a %% b);

    // Absolute value
    let abs_val: int = math::abs(-42);
    println("abs(-42) = %d", abs_val);

    // Min/Max
    let min_val: int = math::min(10, 20);
    let max_val: int = math::max(10, 20);
    println("min=%d, max=%d", min_val, max_val);

    // Power
    let pow_val: int = math::pow(2, 10);
    println("2^10 = %d", pow_val);

    return 0;
}
```

### Trigonometric Functions

```fusion
use std::math;

fn main() -> int {
    let angle: float = 3.14159 / 2.0;  // 90 degrees in radians

    println("sin(90) = %f", math::sin(angle));
    println("cos(90) = %f", math::cos(angle));
    println("tan(90) = %f", math::tan(angle));

    // Inverse functions
    let val: float = 1.0;
    println("asin(1) = %f", math::asin(val));
    println("acos(1) = %f", math::acos(val));
    println("atan(1) = %f", math::atan(val));

    return 0;
}
```

### Random Numbers

```fusion
use std::math::random;

fn main() -> int {
    // Random integer in range
    let rand_int: int = random::range(1, 100);
    println("Random int: %d", rand_int);

    // Random float between 0.0 and 1.0
    let rand_float: float = random::float();
    println("Random float: %f", rand_float);

    // Seed for reproducibility
    random::seed(42);
    let seeded: int = random::range(1, 100);
    println("Seeded random: %d", seeded);

    return 0;
}
```

---

## Process Management

### Running External Commands

```fusion
use std::process;

fn main() -> int {
    // Run a command
    let output: string = process::exec("echo Hello");
    println("Output: %s", output);

    // Run with arguments
    let result: string = process::exec("ls -la");

    // Check exit code
    let status: int = process::exec_status("false");
    println("Exit code: %d", status);

    return 0;
}
```

### Environment Variables

```fusion
use std::env;

fn main() -> int {
    // Get environment variable
    let home: Option<string> = env::get("HOME");
    match home {
        Some(path) => println("HOME: %s", path),
        None => println("HOME not set"),
    }

    // Set environment variable
    env::set("MY_VAR", "hello");

    // Get current directory
    let cwd: string = env::current_dir();
    println("Current dir: %s", cwd);

    // Get all environment variables
    let vars: Vec<(string, string)> = env::vars();
    for (key, value) in vars {
        println("%s=%s", key, value);
    }

    return 0;
}
```

---

## Error Handling

### Result Type

Result is used for operations that can fail:

```fusion
use std::result;

fn divide(a: float, b: float) -> Result<float, string> {
    if b == 0.0 {
        return Err("Division by zero".to_string());
    }
    return Ok(a / b);
}

fn main() -> int {
    let result: Result<float, string> = divide(10.0, 3.0);

    match result {
        Ok(value) => println("Result: %f", value),
        Err(msg) => println("Error: %s", msg),
    }

    // Using unwrap with default
    let value: float = divide(10.0, 0.0).unwrap_or(0.0);
    println("Value: %f", value);

    // Using unwrap with panic message
    let value2: float = divide(10.0, 2.0).unwrap();
    println("Value: %f", value2);

    return 0;
}
```

### Option Type

Option is used for values that may not exist:

```fusion
fn find_user(id: int) -> Option<string> {
    if id == 1 {
        return Some("Alice".to_string());
    }
    return None;
}

fn main() -> int {
    let user: Option<string> = find_user(1);

    match user {
        Some(name) => println("Found user: %s", name),
        None => println("User not found"),
    }

    // Chaining with and_then
    let upper: Option<string> = find_user(1)
        .and_then(|name| Some(name.to_upper()));

    // Using unwrap_or_else
    let name: string = find_user(99)
        .unwrap_or_else(|| "Unknown".to_string());
    println("Name: %s", name);

    // Using map
    let length: Option<int> = find_user(1)
        .map(|name| name.len());
    println("Name length: %d", length.unwrap_or(0));

    return 0;
}
```

### Assert Macros

```fusion
fn main() -> int {
    // Basic assert
    let x: int = 5;
    assert!(x > 0);

    // Assert with message
    assert!(x > 0, "x must be positive, got %d", x);

    // Assert equality
    assert_eq!(x, 5);
    assert_eq!(x, 5, "x should be 5");

    // Assert not equal
    assert_ne!(x, 0);

    // Custom assertion
    fn validate_age(age: int) {
        assert!(age >= 0 && age <= 150, "Invalid age: %d", age);
    }

    validate_age(25);
    // validate_age(-1);  // Would panic

    println("All assertions passed");
    return 0;
}
```

### Panic and Unwrap

```fusion
fn main() -> int {
    // Panic with custom message
    // panic!("Something went wrong");

    // Unwrap with custom panic message
    let value: Option<int> = None;
    // let v: int = value.expect("Value must exist");  // Panics with message

    // Safe unwrap patterns
    let some_value: Option<int> = Some(42);
    let default_value: int = 0;

    // Pattern 1: match
    let v1: int = match some_value {
        Some(v) => v,
        None => default_value,
    };

    // Pattern 2: unwrap_or
    let v2: int = some_value.unwrap_or(default_value);

    // Pattern 3: unwrap_or_else
    let v3: int = some_value.unwrap_or_else(|| {
        println("Computing default");
        return default_value;
    });

    println("v1=%d, v2=%d, v3=%d", v1, v2, v3);

    return 0;
}
```

---

## Tips and Best Practices

1. **Use the standard library**: It's well-tested and optimized.
2. **Check for errors**: Many I/O operations can fail — handle errors gracefully.
3. **Buffer I/O**: Use buffered readers/writers for performance.
4. **Use iterators**: They're often more efficient than manual loops.
5. **Prefer `Vec` over arrays**: Unless you know the exact size at compile time.
6. **Use Result for fallible operations**: Don't panic in library code.
7. **Use Option for nullable values**: Never use null.
8. **Chain operations**: Use map, and_then, unwrap_or for clean error handling.

---

## Cross-References

- **Chapter 2**: Syntax for basic operations
- **Chapter 5**: Generics for parameterized collections
- **Chapter 10**: Concurrency for parallel I/O
- **Chapter 15**: Reference for complete API listing
