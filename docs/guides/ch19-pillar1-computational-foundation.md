# Chapter 19: Pillar 1 — The Computational Foundation (The Soul)

> Why a language must *compute* before it can do anything else — and how Fusion v2.0 Vortex earns the right to call itself a programming language.

---

## Introduction

A language without **Turing completeness** is a configuration file with ambitions. YAML, JSON, HTML, and TOML can describe data, declare structure, and wire dependencies — but they cannot *decide*, *loop*, or *discover* at runtime. The moment your system needs to make a choice that depends on input, or repeat an operation until a condition is met, you need a real language.

Fusion v2.0 Vortex is built on **Pillar 1: The Computational Foundation** — the guarantee that it is a fully Turing-complete, general-purpose programming language capable of expressing any computable function. This is not a footnote. It is the bedrock on which every other pillar (execution model, safety, quantum, ML, polyglot interop) rests.

### What Turing Completeness Means in Practice

| Capability | Markup/Config | Fusion v2.0 Vortex |
|---|---|---|
| Sequential execution | Static declaration order | Arbitrary control flow |
| Conditional branching | Limited (ternary in some) | `if`/`else`, `match`, guards |
| Iteration / loops | None | `while`, `for`, `loop`, recursion |
| Function abstraction | None | Named functions, closures, methods |
| Data manipulation | Read-only / declarative | Full arithmetic, string ops, bitwise |
| Type system | Schema-level only | Static + inference + refinement |

If a language cannot do all five things on the right side, it is a DSL or a config format. Fusion is none of those — it is a systems programming language with the full power of a Turing machine.

---

## Turing Completeness

### Sequential Execution

Fusion executes statements in order, top to bottom, left to right. Every statement produces a value or triggers a side effect, and the next statement begins only after the previous one completes.

```fusion
fn main() {
    let x: Int = 10;
    let y: Int = x + 5;        // 15
    let z: Int = y * 2;         // 30
    print("Result: {z}");       // "Result: 30"
}
```

Statements are separated by newlines or semicolons. Block expressions (`{ ... }`) produce the value of their last expression:

```fusion
fn compute() -> Int {
    let result = {
        let a = 3;
        let b = 4;
        a * a + b * b   // 25 — this is the block's value
    };
    result
}
```

### Selection

#### If/Else

```fusion
fn classify(n: Int) -> String {
    if n < 0 {
        "negative"
    } else if n == 0 {
        "zero"
    } else {
        "positive"
    }
}
```

`if` is an expression — it returns a value. No parentheses required around the condition.

#### Match (Pattern Matching)

```fusion
enum TrafficLight { Red, Yellow, Green }

fn can_go(light: TrafficLight) -> Bool {
    match light {
        TrafficLight::Green => true,
        TrafficLight::Yellow => false,
        TrafficLight::Red => false,
    }
}
```

Match supports guards, destructuring, and exhaustive checking:

```fusion
fn describe(value: Option<Int>) -> String {
    match value {
        Some(n) if n > 100 => "large",
        Some(n) if n > 0   => "small positive",
        Some(0)             => "exactly zero",
        Some(n)             => "negative: {n}",
        None                => "absent",
    }
}
```

The compiler guarantees exhaustiveness — if you omit a case, it errors.

### Iteration

#### While Loop

```fusion
fn sum_to(target: Int) -> Int {
    let mut sum = 0;
    let mut i = 1;
    while i <= target {
        sum += i;
        i += 1;
    }
    sum
}
```

#### For Loop (Iterator-Based)

```fusion
fn product_of_evens(numbers: &[Int]) -> Int {
    let mut product = 1;
    for n in numbers {
        if n % 2 == 0 {
            product *= n;
        }
    }
    product
}
```

Fusion `for` loops work with any iterable — arrays, ranges, iterators, hashmaps, channels.

#### Loop (Infinite + Break)

```fusion
fn read_until_quit() -> Vec<String> {
    let mut lines = Vec::new();
    loop {
        let input = read_line();
        if input == "quit" {
            break;
        }
        lines.push(input);
    }
    lines
}
```

`loop` is an expression — `break` returns a value:

```fusion
fn find_first_positive(numbers: &[Int]) -> Option<Int> {
    let mut i = 0;
    loop {
        if i >= numbers.len() {
            break None;
        }
        if numbers[i] > 0 {
            break Some(numbers[i]);
        }
        i += 1;
    }
}
```

#### Recursion

```fusion
fn fibonacci(n: Int) -> Int {
    if n <= 1 { return n; }
    fibonacci(n - 1) + fibonacci(n - 2)
}
```

Fusion guarantees **tail-call optimization** (TCO) when a function is in tail position, so deep recursion does not overflow the stack.

### Why This Is the Minimum

A language that lacks any one of these — sequential execution, selection, or iteration — cannot express all computable functions. Without sequential execution, there is no ordering of operations. Without selection, every program does the same thing regardless of input. Without iteration, you cannot process unbounded data. Fusion provides all three, plus higher-order functions and closures that make it strictly more powerful than a bare Turing machine in ergonomics.

---

## Data Representation

### Primitive Types

| Type | Description | Size | Example |
|---|---|---|---|
| `Int` | Signed 64-bit integer | 8 bytes | `42`, `-7`, `0` |
| `Float` | 64-bit IEEE 754 double | 8 bytes | `3.14`, `-0.5`, `1e10` |
| `Bool` | Boolean | 1 byte | `true`, `false` |
| `String` | UTF-8 heap-allocated string | Heap | `"hello"` |
| `Char` | Unicode scalar value | 4 bytes | `'A'`, `'Ω'`, `'你好'` |
| `Byte` | Unsigned 8-bit integer | 1 byte | `0xFF`, `128` |
| `Int32` | Signed 32-bit integer | 4 bytes | `42i32` |
| `Int16` | Signed 16-bit integer | 2 bytes | `42i16` |
| `Int8` | Signed 8-bit integer | 1 byte | `42i8` |
| `Float32` | 32-bit IEEE 754 single | 4 bytes | `3.14f32` |

```fusion
let age: Int = 30;
let pi: Float = 3.14159;
let active: Bool = true;
let name: String = "Fusion";
let symbol: Char = 'λ';
let byte_val: Byte = 0xAB;
```

### Composite Types

#### Arrays (Fixed-Size)

```fusion
let matrix: [Int; 3] = [1, 2, 3];
let grid: [[Float; 3]; 3] = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
print(matrix[1]); // 2
```

#### Vectors (Dynamic)

```fusion
let mut scores = Vec::new();
scores.push(95);
scores.push(87);
scores.push(92);

let avg = scores.iter().sum::<Float>() / scores.len() as Float;
```

#### Structs

```fusion
struct Point {
    x: Float,
    y: Float,
}

impl Point {
    fn new(x: Float, y: Float) -> Self {
        Self { x, y }
    }

    fn distance_to(&self, other: &Point) -> Float {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

let p1 = Point::new(0.0, 0.0);
let p2 = Point::new(3.0, 4.0);
print(p1.distance_to(&p2)); // 5.0
```

#### Enums

```fusion
enum Color {
    Red,
    Green,
    Blue,
    Custom(u8, u8, u8),
}

fn hex_string(c: &Color) -> String {
    match c {
        Color::Red => "#FF0000".to_string(),
        Color::Green => "#00FF00".to_string(),
        Color::Blue => "#0000FF".to_string(),
        Color::Custom(r, g, b) => "#{r:02X}{g:02X}{b:02X}".to_string(),
    }
}
```

#### Hashmaps

```fusion
let mut scores: HashMap<String, Int> = HashMap::new();
scores.insert("Alice".to_string(), 95);
scores.insert("Bob".to_string(), 87);

match scores.get("Alice") {
    Some(score) => print("Alice scored {score}"),
    None => print("No score found"),
}
```

### Variables

#### `let` (Immutable)

```fusion
let x = 42;
// x = 43;  // Compile error!
```

#### `let mut` (Mutable)

```fusion
let mut counter = 0;
counter += 1;
counter += 1;
print(counter); // 2
```

#### `const` (Compile-Time Constant)

```fusion
const MAX_SIZE: Int = 1024;
const PI: Float = 3.14159265358979;
const VERSION: &str = "2.0.0";
```

`const` values are inlined at compile time — no allocation, no runtime cost.

### Scoping Rules

Variables are scoped to their containing block. Inner scopes shadow outer variables:

```fusion
fn main() {
    let x = 1;
    {
        let x = 2;      // shadows outer x
        print(x);        // 2
    }
    print(x);            // 1 — original x still valid
}
```

### Type Inference

Fusion infers types from context — you rarely need explicit annotations:

```fusion
let x = 42;              // Int
let y = 3.14;            // Float
let name = "Fusion";     // String
let items = vec![1, 2];  // Vec<Int>

// When inference needs help:
let parsed: Option<Int> = "42".parse();
```

---

## Operators

### Arithmetic

| Operator | Description | Example | Result |
|---|---|---|---|
| `+` | Addition | `5 + 3` | `8` |
| `-` | Subtraction | `5 - 3` | `2` |
| `*` | Multiplication | `5 * 3` | `15` |
| `/` | Division | `10 / 3` | `3` (integer) or `3.333...` (float) |
| `%` | Modulo | `10 % 3` | `1` |
| `++` | Increment (statement) | `x++` | `x + 1` |
| `--` | Decrement (statement) | `x--` | `x - 1` |
| `**` | Exponentiation | `2 ** 10` | `1024` |

Integer division truncates toward zero. Use `.into_float()` or cast for floating-point division:

```fusion
let a: Int = 7 / 2;       // 3
let b: Float = 7.0 / 2.0; // 3.5
let c = 7 as Float / 2.0; // 3.5
```

### Comparison

| Operator | Description | Example |
|---|---|---|
| `==` | Equal | `5 == 5` → `true` |
| `!=` | Not equal | `5 != 3` → `true` |
| `<` | Less than | `3 < 5` → `true` |
| `>` | Greater than | `5 > 3` → `true` |
| `<=` | Less or equal | `5 <= 5` → `true` |
| `>=` | Greater or equal | `5 >= 3` → `true` |

### Logical

| Operator | Description | Example |
|---|---|---|
| `&&` | Logical AND | `true && false` → `false` |
| `\|\|` | Logical OR | `true \|\| false` → `true` |
| `!` | Logical NOT | `!true` → `false` |

Short-circuit evaluation: `&&` does not evaluate the right side if the left is `false`; `||` does not evaluate the right if the left is `true`.

### Bitwise

| Operator | Description | Example |
|---|---|---|
| `&` | AND | `0b1100 & 0b1010` → `0b1000` |
| `\|` | OR | `0b1100 \| 0b1010` → `0b1110` |
| `^` | XOR | `0b1100 ^ 0b1010` → `0b0110` |
| `<<` | Left shift | `1 << 3` → `8` |
| `>>` | Right shift | `16 >> 2` → `4` |
| `!` | Bitwise NOT | `!0u8` → `255` |

### Operator Precedence (Highest to Lowest)

| Precedence | Operators | Associativity |
|---|---|---|
| 1 | `()` `[]` `.` `::` | Left-to-right |
| 2 | `!` `-` (unary) `*` `&` (reference) | Right-to-left |
| 3 | `**` | Right-to-left |
| 4 | `*` `/` `%` | Left-to-right |
| 5 | `+` `-` | Left-to-right |
| 6 | `<<` `>>` | Left-to-right |
| 7 | `&` | Left-to-right |
| 8 | `^` | Left-to-right |
| 9 | `\|` | Left-to-right |
| 10 | `==` `!=` `<` `>` `<=` `>=` | Left-to-right |
| 11 | `&&` | Left-to-right |
| 12 | `\|\|` | Left-to-right |
| 13 | `..` `..=` | Left-to-right |
| 14 | `=` `+=` `-=` `*=` `/=` `%=` `&=` `\|=` `^=` `<<=` `>>=` | Right-to-left |

### Associativity Rules

- **Left-to-right**: `a - b - c` means `(a - b) - c`
- **Right-to-left**: `a = b = c` means `a = (b = c)`; `x ** y ** z` means `x ** (y ** z)`
- **Non-associative**: comparisons cannot be chained (`a < b < c` is an error — use `a < b && b < c`)

---

## Abstraction

### Functions

```fusion
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn greet(name: &str) -> String {
    "Hello, {name}!".to_string()
}

// No return value — implicit unit return
fn log_message(msg: &str) {
    print("[LOG] {msg}");
}
```

Functions are first-class values:

```fusion
let operation: fn(Int, Int) -> Int = add;
let result = operation(3, 4); // 7
```

### Methods via `impl` Blocks

```fusion
struct Rectangle {
    width: Float,
    height: Float,
}

impl Rectangle {
    fn new(width: Float, height: Float) -> Self {
        Self { width, height }
    }

    fn area(&self) -> Float {
        self.width * self.height
    }

    fn perimeter(&self) -> Float {
        2.0 * (self.width + self.height)
    }

    fn is_square(&self) -> Bool {
        (self.width - self.height).abs() < 1e-10
    }

    fn scale(&mut self, factor: Float) {
        self.width *= factor;
        self.height *= factor;
    }
}
```

### Closures and Lambdas

```fusion
// Inline closures
let square = |x: Int| x * x;
print(square(5)); // 25

// Closures capturing environment
let threshold = 100;
let is_over = |value: Int| value > threshold;
print(is_over(150)); // true

// Multi-line closure
let transform = |x: Int| {
    let doubled = x * 2;
    let incremented = doubled + 1;
    incremented
};

// Closures as function arguments
let numbers = vec![1, 2, 3, 4, 5];
let evens: Vec<Int> = numbers.iter()
    .filter(|&&n| n % 2 == 0)
    .cloned()
    .collect();
// evens == [2, 4]
```

### Recursion

```fusion
fn sum_list(list: &[Int]) -> Int {
    match list {
        [] => 0,
        [head, tail @ ..] => head + sum_list(tail),
    }
}
```

### First-Class Functions

Functions can be passed as arguments, returned from other functions, and stored in data structures:

```fusion
fn apply_twice(f: fn(Int) -> Int, x: Int) -> Int {
    f(f(x))
}

fn double(x: Int) -> Int { x * 2 }
fn inc(x: Int) -> Int { x + 1 }

print(apply_twice(double, 3)); // 12
print(apply_twice(inc, 3));    // 5

// Function factory
fn make_multiplier(factor: Int) -> fn(Int) -> Int {
    |x: Int| x * factor
}

let triple = make_multiplier(3);
print(triple(10)); // 30
```

---

## Code Examples

### Complete Program Using All Features

```fusion
// A mini calculator supporting arithmetic, functions, and closures
use std::collections::HashMap;

type CalculatorFn = fn(Float, Float) -> Float;

struct Calculator {
    history: Vec<(String, Float)>,
    constants: HashMap<String, Float>,
}

impl Calculator {
    fn new() -> Self {
        let mut constants = HashMap::new();
        constants.insert("pi".to_string(), 3.14159265358979);
        constants.insert("e".to_string(), 2.71828182845905);

        Self {
            history: Vec::new(),
            constants,
        }
    }

    fn add(&self, a: Float, b: Float) -> Float { a + b }
    fn sub(&self, a: Float, b: Float) -> Float { a - b }
    fn mul(&self, a: Float, b: Float) -> Float { a * b }
    fn div(&self, a: Float, b: Float) -> Float {
        if b == 0.0 {
            panic!("Division by zero");
        }
        a / b
    }

    fn operate(&mut self, op: &str, a: Float, b: Float) -> Float {
        let result = match op {
            "+" => self.add(a, b),
            "-" => self.sub(a, b),
            "*" => self.mul(a, b),
            "/" => self.div(a, b),
            _ => panic!("Unknown operator: {op}"),
        };
        self.history.push((op.to_string(), result));
        result
    }

    fn get_constant(&self, name: &str) -> Option<Float> {
        self.constants.get(name).copied()
    }

    fn last_result(&self) -> Option<Float> {
        self.history.last().map(|(_, result)| *result)
    }

    fn summary(&self) -> String {
        let count = self.history.len();
        let total: Float = self.history.iter().map(|(_, r)| r).sum();
        "Operations: {count}, Total of results: {total}".to_string()
    }
}

fn main() {
    let mut calc = Calculator::new();

    // Using the calculator
    let result = calc.operate("+", 10.0, 5.0);
    print("10 + 5 = {result}");

    let result = calc.operate("*", 3.0, 4.0);
    print("3 * 4 = {result}");

    // Using constants
    if let Some(pi) = calc.get_constant("pi") {
        let area = calc.operate("*", pi, 25.0);  // π * r² for r=5
        print("Area of circle: {area}");
    }

    // Closure for custom operation
    let power = |base: Float, exp: Int| -> Float {
        let mut result = 1.0;
        let mut i = 0;
        while i < exp {
            result *= base;
            i += 1;
        }
        result
    };

    let val = power(2.0, 10);
    print("2^10 = {val}");

    // Pattern matching on history
    for (op, result) in &calc.history {
        match op.as_str() {
            "+" => print("Addition result: {result}"),
            "*" => print("Multiplication result: {result}"),
            _ => print("Other operation: {result}"),
        }
    }

    print(calc.summary());
}
```

### Fibonacci with Recursion

```fusion
// Naive recursive Fibonacci (exponential time)
fn fib_naive(n: Int) -> Int {
    if n <= 1 { return n; }
    fib_naive(n - 1) + fib_naive(n - 2)
}

// Memoized Fibonacci (linear time)
fn fib_memoized(n: Int, cache: &mut HashMap<Int, Int>) -> Int {
    if n <= 1 { return n; }
    match cache.get(&n) {
        Some(&cached) => cached,
        None => {
            let result = fib_memoized(n - 1, cache) + fib_memoized(n - 2, cache);
            cache.insert(n, result);
            result
        }
    }
}

// Tail-recursive Fibonacci (optimized to loop by compiler)
fn fib_tail(n: Int) -> Int {
    fn helper(n: Int, a: Int, b: Int) -> Int {
        if n == 0 { return a; }
        helper(n - 1, b, a + b)
    }
    helper(n, 0, 1)
}

fn main() {
    // Naive — fine for small n
    for i in 0..20 {
        print("fib({i}) = {fib_naive(i)}");
    }

    // Memoized — fast for large n
    let mut cache = HashMap::new();
    for i in 0..50 {
        let result = fib_memoized(i, &mut cache);
        print("fib({i}) = {result}");
    }

    // Tail-recursive — compiler optimizes to iteration
    for i in 0..50 {
        let result = fib_tail(i);
        print("fib({i}) = {result}");
    }
}
```

### Factorial with Tail-Call Optimization

```fusion
// Standard recursive factorial
fn factorial(n: Int) -> Int {
    if n <= 1 { return 1; }
    n * factorial(n - 1)
}

// Tail-recursive factorial — compiler converts to loop
fn factorial_tail(n: Int) -> Int {
    fn helper(remaining: Int, accumulator: Int) -> Int {
        if remaining <= 1 {
            return accumulator;
        }
        helper(remaining - 1, accumulator * remaining)
    }
    helper(n, 1)
}

// Iterative factorial (explicit loop)
fn factorial_iter(n: Int) -> Int {
    let mut result = 1;
    let mut i = 2;
    while i <= n {
        result *= i;
        i += 1;
    }
    result
}

// Generic over numeric types using traits
fn factorial_generic<T: Numeric>(n: T) -> T {
    fn helper<T: Numeric>(remaining: T, acc: T) -> T {
        if remaining <= T::one() {
            return acc;
        }
        helper(remaining - T::one(), acc * remaining)
    }
    helper(n, T::one())
}

fn main() {
    // All three produce the same result
    print("10! = {factorial(10)}");        // 3628800
    print("10! = {factorial_tail(10)}");    // 3628800
    print("10! = {factorial_iter(10)}");    // 3628800

    // Tail-recursive version handles large inputs without stack overflow
    print("20! = {factorial_tail(20)}");    // 2432902008176640000

    // Generic version works with different numeric types
    print("10! (Float) = {factorial_generic::<Float>(10.0)}");
}
```

---

## Summary

Pillar 1 establishes that Fusion v2.0 Vortex is a **real programming language** — not a DSL, not a config format, not a markup language. It provides:

- **Full Turing completeness** via sequential execution, selection (`if`/`match`), and iteration (`while`/`for`/`loop`/recursion)
- **Rich data representation** with primitives, structs, enums, vectors, hashmaps, and type inference
- **Complete operator set** covering arithmetic, comparison, logical, and bitwise operations with well-defined precedence and associativity
- **Powerful abstraction** through functions, methods, closures, and first-class function values

These capabilities are the minimum bar for a language to be considered general-purpose. Every subsequent pillar — the execution model, memory safety, quantum computing, ML, and polyglot interop — builds on this foundation. Without Turing completeness, nothing else matters.

---

> **Next**: [Chapter 20 — Pillar 2: The Execution Model & Memory (The Engine)](ch20-pillar2-execution-memory.md)
