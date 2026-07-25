# Chapter 2: Syntax

> Variables, types, operators, control flow, functions, and comments in Fusion

---

## Variables and Bindings

Fusion uses `let` bindings to introduce variables. Variables are **immutable by default** — once assigned, their value cannot change unless you explicitly opt into mutability.

### Immutable Bindings

```fusion
fn main() -> int {
    let x: int = 42;
    let name: string = "Fusion";
    let pi_approx: float = 3.14;
    let active: bool = true;

    // This would cause a compile error:
    // x = 100;  // ERROR: cannot reassign immutable variable

    println("x = %d, name = %s", x, name);
    return 0;
}
```

### Mutable Bindings

Use `mut` to declare a variable that can be reassigned:

```fusion
fn main() -> int {
    let mut counter: int = 0;
    counter = 10;
    counter = counter + 5;
    println("counter = %d", counter);  // counter = 15

    let mut message: string = "hello";
    message = "world";
    println("message = %s", message);  // message = world

    return 0;
}
```

### Type Annotations

Fusion supports explicit type annotations, though the compiler can infer types in many cases:

```fusion
fn main() -> int {
    // Explicit type annotation
    let x: int = 42;

    // Type inference (compiler deduces 'int')
    let y = 42;

    // Explicit annotation for clarity
    let name: string = "Fusion";
    let active: bool = true;
    let ratio: float = 0.5;

    return 0;
}
```

### Constants

Use `const` for values that are known at compile time:

```fusion
const MAX_BUFFER: int = 4096;
const PI: float = 3.14159265;
const APP_NAME: string = "MyApp";

fn main() -> int {
    println("App: %s, Buffer: %d", APP_NAME, MAX_BUFFER);
    return 0;
}
```

### Static Declarations

Use `static` for global mutable state (use sparingly):

```fusion
static mut GLOBAL_COUNTER: int = 0;

fn increment() {
    GLOBAL_COUNTER = GLOBAL_COUNTER + 1;
}
```

---

## Data Types

Fusion provides the following built-in types:

### Primitive Types

| Type | Description | Size |
|------|-------------|------|
| `int` | Signed 64-bit integer | 8 bytes |
| `bool` | Boolean (true/false) | 1 byte |
| `string` | UTF-8 string | Variable |
| `float` | 64-bit floating point | 8 bytes |
| `void` | Unit type (no value) | 0 bytes |

### Integer Types

```fusion
fn main() -> int {
    let a: int = 42;          // Signed 64-bit integer
    let b: i64 = 100;         // Explicit signed 64-bit
    let c: u8 = 255;          // Unsigned 8-bit
    let d: u64 = 18446744073709551615;  // Unsigned 64-bit

    // Character literals are integer values
    let ch: int = 'A';  // 65
    let nl: int = '\n'; // 10

    println("a=%d, c=%d, ch=%d", a, c, ch);
    return 0;
}
```

### Boolean Type

```fusion
fn main() -> int {
    let active: bool = true;
    let disabled: bool = false;

    if active {
        println("System is active");
    }

    if !disabled {
        println("System is not disabled");
    }

    return 0;
}
```

### String Type

```fusion
fn main() -> int {
    let greeting: string = "Hello, Fusion!";
    let empty: string = "";
    let with_escape: string = "Line one\nLine two";
    let with_tab: string = "Column1\tColumn2";

    // Raw strings (no escape processing)
    let raw: string = r#"This is a "raw" string"#;

    println(greeting);
    println(with_escape);

    return 0;
}
```

### Float Type

```fusion
fn main() -> int {
    let pi: float = 3.14159;
    let e: float = 2.71828;
    let result: float = pi * 2.0;

    println("pi * 2 = %f", result);
    return 0;
}
```

### Array Type

```fusion
fn main() -> int {
    // Array literal
    let numbers: [int; 5] = [1, 2, 3, 4, 5];

    // Array repeat syntax (all zeros)
    let zeros: [int; 10] = [0; 10];

    // Array indexing
    let first: int = numbers[0];
    let third: int = numbers[2];
    println("first=%d, third=%d", first, third);

    return 0;
}
```

### Pointer Type

```fusion
fn main() -> int {
    let x: int = 42;
    let ptr: *int = &x;      // Address-of operator
    let val: int = *ptr;     // Dereference operator

    println("x=%d, val=%d", x, val);
    return 0;
}
```

### Slice Type

Slices are views into arrays:

```fusion
fn process(data: [int]) {
    // Slice of the full array
    let sub: [int] = data[1..4];  // Elements 1, 2, 3
    println("slice length: %d", sub.len());
}

fn main() -> int {
    let arr: [int; 5] = [10, 20, 30, 40, 50];
    process(arr);
    return 0;
}
```

---

## Operators

### Arithmetic Operators

```fusion
fn main() -> int {
    let a: int = 10;
    let b: int = 3;

    println("a + b = %d", a + b);   // 13
    println("a - b = %d", a - b);   // 7
    println("a * b = %d", a * b);   // 30
    println("a / b = %d", a / b);   // 3 (integer division)
    println("a %% b = %d", a %% b); // 1 (modulo)

    // Float arithmetic
    let x: float = 10.0;
    let y: float = 3.0;
    println("x / y = %f", x / y);  // 3.333...

    return 0;
}
```

### Comparison Operators

```fusion
fn main() -> int {
    let a: int = 10;
    let b: int = 20;

    println("a == b: %d", a == b);  // 0 (false)
    println("a != b: %d", a != b);  // 1 (true)
    println("a < b: %d", a < b);    // 1 (true)
    println("a > b: %d", a > b);    // 0 (false)
    println("a <= b: %d", a <= b);  // 1 (true)
    println("a >= b: %d", a >= b);  // 0 (false)

    return 0;
}
```

### Logical Operators

```fusion
fn main() -> int {
    let a: bool = true;
    let b: bool = false;

    println("a && b: %d", a && b);  // 0 (false)
    println("a || b: %d", a || b);  // 1 (true)
    println("!a: %d", !a);          // 0 (false)

    // Short-circuit evaluation
    let x: int = 0;
    let y: int = 10;
    if x != 0 && y / x > 2 {
        // This won't execute because x != 0 is false
        println("safe division");
    }

    return 0;
}
```

### Bitwise Operators

```fusion
fn main() -> int {
    let a: int = 0b1010;  // 10
    let b: int = 0b1100;  // 12

    println("a & b = %d", a & b);   // 8 (1000)
    println("a | b = %d", a | b);   // 14 (1110)
    println("a ^ b = %d", a ^ b);   // 6 (0110)
    println("~a = %d", !a);          // bitwise NOT

    // Shift operators
    let shifted: int = 1 << 4;  // 16
    let right: int = 16 >> 2;   // 4
    println("shifted=%d, right=%d", shifted, right);

    return 0;
}
```

### Operator Precedence

From highest to lowest:

1. `!` (unary NOT), `&` (address-of), `*` (dereference)
2. `*`, `/`, `%`
3. `+`, `-`
4. `<<`, `>>`
5. `&` (bitwise AND)
6. `^` (bitwise XOR)
7. `|` (bitwise OR)
8. `==`, `!=`, `<`, `>`, `<=`, `>=`
9. `&&`
10. `||`

---

## Control Flow

### If/Else

```fusion
fn main() -> int {
    let temperature: int = 75;

    if temperature > 90 {
        println("It's hot!");
    } else if temperature > 70 {
        println("It's warm.");
    } else if temperature > 50 {
        println("It's cool.");
    } else {
        println("It's cold!");
    }

    // Parentheses around condition are optional
    if (temperature > 80) {
        println("High temperature alert");
    }

    return 0;
}
```

### While Loops

```fusion
fn main() -> int {
    let mut i: int = 0;
    while i < 10 {
        println("i = %d", i);
        i = i + 1;
    }

    // While with complex condition
    let mut value: int = 100;
    while value > 0 && value % 2 == 0 {
        value = value / 2;
    }
    println("Final value: %d", value);

    return 0;
}
```

### For-In Loops

```fusion
fn main() -> int {
    // Iterate over a range
    for i in 0..10 {
        println("i = %d", i);
    }

    // Iterate over an array
    let fruits: [string; 3] = ["apple", "banana", "cherry"];
    for fruit in fruits {
        println("fruit: %s", fruit);
    }

    // Iterate with step
    for i in (0..20).step(2) {
        println("even: %d", i);
    }

    return 0;
}
```

### Match Expressions

Pattern matching is a powerful control flow mechanism:

```fusion
fn describe(x: int) -> string {
    return match x {
        0 => "zero",
        1 => "one",
        2 => "two",
        3..5 => "three to five",
        _ => "something else",
    };
}

fn main() -> int {
    println(describe(0));   // "zero"
    println(describe(3));   // "three to five"
    println(describe(42));  // "something else"

    // Match with guards
    let age: int = 25;
    let category: string = match age {
        n if n < 13 => "child",
        n if n < 18 => "teenager",
        n if n < 65 => "adult",
        _ => "senior",
    };
    println("Category: %s", category);

    return 0;
}
```

---

## Functions

### Basic Functions

```fusion
// Simple function with no parameters
fn greet() {
    println("Hello!");
}

// Function with parameters and return type
fn add(a: int, b: int) -> int {
    return a + b;
}

// Function with explicit return
fn multiply(a: int, b: int) -> int {
    a * b  // Implicit return (last expression)
}

fn main() -> int {
    greet();
    let sum: int = add(3, 4);
    let product: int = multiply(5, 6);
    println("sum=%d, product=%d", sum, product);
    return 0;
}
```

### Functions with Multiple Returns (Tuples)

```fusion
fn swap(a: int, b: int) -> (int, int) {
    return (b, a);
}

fn divmod(n: int, d: int) -> (int, int) {
    return (n / d, n %% d);
}

fn main() -> int {
    let (x, y) = swap(1, 2);
    println("x=%d, y=%d", x, y);  // x=2, y=1

    let (quotient, remainder) = divmod(17, 5);
    println("17 / 5 = %d remainder %d", quotient, remainder);

    return 0;
}
```

### Closures

```fusion
fn main() -> int {
    // Single-expression closure
    let double = |x: int| x * 2;
    println("double(5) = %d", double(5));  // 10

    // Multi-statement closure
    let greet = |name: string| {
        let msg: string = "Hello, " + name + "!";
        println(msg);
    };
    greet("Fusion");

    // Closure with captured environment
    let factor: int = 10;
    let scale = |x: int| x * factor;
    println("scale(3) = %d", scale(3));  // 30

    return 0;
}
```

### Recursive Functions

```fusion
fn factorial(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn fibonacci(n: int) -> int {
    if n <= 0 { return 0; }
    if n == 1 { return 1; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() -> int {
    println("5! = %d", factorial(5));     // 120
    println("fib(10) = %d", fibonacci(10)); // 55
    return 0;
}
```

---

## Comments

### Line Comments

```fusion
// This is a line comment
let x: int = 42; // Comment after code

// Multi-line comment using multiple line comments:
// Line 1
// Line 2
// Line 3
```

### Block Comments

```fusion
/* This is a block comment */
let x: int = 42;

/*
 * This is a multi-line block comment.
 * It can span multiple lines.
 */
let y: int = 100;

/* Block comments can /* nest */ in Fusion */
```

### Documentation Comments

```fusion
/// Adds two integers together.
/// Returns the sum of a and b.
fn add(a: int, b: int) -> int {
    return a + b;
}

/// A struct representing a 2D point.
struct Point {
    x: float,
    y: float,
}
```

---

## Attributes

Attributes provide metadata to the compiler, control code generation, and enable conditional compilation.

### Function Attributes

```fusion
// Mark function as critical for security analysis
#[intent(Critical)]
fn encrypt_data(data: [u8]) -> [u8] {
    // Implementation
    return data;
}

// Function used only in test builds
#[cfg(test)]
fn test_encrypt() {
    let data: [u8] = [1, 2, 3];
    let encrypted: [u8] = encrypt_data(data);
    assert(encrypted.len() == 3);
}
```

### Struct and Enum Attributes

```fusion
// Derive common trait implementations
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: float,
    y: float,
}

// Derive for enums
#[derive(Debug, Clone, Copy)]
enum Color {
    Red,
    Green,
    Blue,
}

// Conditional compilation for platform-specific code
#[cfg(target = "windows")]
fn get_platform() -> string {
    return "Windows";
}

#[cfg(target = "linux")]
fn get_platform() -> string {
    return "Linux";
}
```

### Module-Level Attributes

```fusion
// Enable unsafe code warnings
#![warn(unsafe)]

// Set module-level lints
#![deny(unused_variables)]

// Enable experimental features
#![feature(quantum_native)]

mod quantum_circuits;
```

### Common Attributes Reference

| Attribute | Description | Example |
|-----------|-------------|---------|
| `#[derive(Trait)]` | Auto-implement a trait | `#[derive(Debug, Clone)]` |
| `#[cfg(condition)]` | Conditional compilation | `#[cfg(test)]` |
| `#[intent(Level)]` | Security intent annotation | `#[intent(Critical)]` |
| `#[inline]` | Suggest function inlining | `#[inline]` |
| `#[deprecated]` | Mark as deprecated | `#[deprecated]` |
| `#[allow(warning)]` | Suppress a warning | `#[allow(unused)]` |
| `#[test]` | Mark as test function | `#[test]` |

### Common Mistakes with Attributes

```fusion
// WRONG: Attribute on wrong line
fn main() -> int {
    #[test]  // ERROR: Can't use function attribute here
    let x = 5;
    return 0;
}

// CORRECT: Attribute before the item
#[test]
fn test_something() {
    let x = 5;
    assert(x == 5);
}
```

---

## Common Patterns and Anti-Patterns

### Good Patterns

```fusion
// Use meaningful variable names
let user_count: int = 42;
let max_retries: int = 3;

// Prefer immutability
let result: int = compute(input);

// Use descriptive function names
fn calculate_total_price(quantity: int, price: float) -> float {
    return (quantity as float) * price;
}
```

### Anti-Patterns

```fusion
// Don't use single-letter names for complex values
let x: ComplexStruct = get_complex_struct();  // Bad

// Don't overuse mutability
let mut temp: int = 0;
temp = compute_a();
temp = compute_b(temp);
temp = compute_c(temp);  // Consider breaking into separate bindings

// Don't ignore return values that indicate errors
let file = open("data.csv");  // Better to handle the result
```

---

## Tips and Best Practices

1. **Prefer immutability**: Use `let` by default. Only use `mut` when reassignment is truly needed.
2. **Name things clearly**: `user_count` is better than `uc` or `n`.
3. **Keep functions small**: Each function should do one thing well.
4. **Use early returns**: They make code easier to read by reducing nesting.
5. **Leverage type inference**: Don't always annotate types — let the compiler infer when obvious.

---

## Cross-References

- **Chapter 3**: Structs and Enums for custom types
- **Chapter 4**: Memory Safety for ownership and borrowing
- **Chapter 5**: Generics for parameterized types
- **Chapter 15**: Reference for complete operator precedence table
