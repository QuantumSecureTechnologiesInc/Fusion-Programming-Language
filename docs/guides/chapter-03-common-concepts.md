> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 3: Common Programming Concepts

This chapter covers the fundamental concepts that appear in almost every programming language and how they work in Fusion. While many of these concepts will be familiar if you've programmed before, Fusion's implementation often adds safety and expressiveness guarantees that are worth understanding deeply.

We will learn about:
- **Variables and Mutability**: How to store and change data.
- **Data Types**: Integers, floats, booleans, characters, tuples, and arrays.
- **Functions**: Parameters, return values, and code organization.
- **Comments**: Documenting your code.
- **Control Flow**: `if` expressions and loops (`loop`, `while`, `for`).

---

## 3.1 Variables and Mutability

As mentioned in Chapter 2, variables in Fusion are **immutable by default**. This is a deliberate design choice to encourage writing code that is safe and easy to reason about. When a variable is immutable, you can be certain that its value won't change unexpectedly.

### 3.1.1 Immutable Variables

Consider this program:

```fusion
fn main() {
    let x = 5
    println("The value of x is: {}", x)
    // x = 6  // This would cause a compile-time error
}
```text

If you try to assign `6` to `x`, the compiler will produce an error: `cannot assign twice to immutable variable x`. This prevents bugs where one part of your code assumes a value is constant, but another part changes it.

### 3.1.2 Mutable Variables

If you need a variable to change, you must explicitly declare it as **mutable** using the `mut` keyword.

```fusion
fn main() {
    let mut x = 5
    println("The value of x is: {}", x)
    x = 6
    println("The value of x is: {}", x)
}
```text

Using `mut` conveys intent to future readers of the code (including yourself) that "this value will change".

### 3.1.3 Constants

Like immutable variables, **constants** are values that are bound to a name and are not allowed to change. However, there are differences:
- You declare constants using the `const` keyword instead of `let`.
- You **must** annotate the type of the value.
- Constants can be declared in any scope, including the global scope.
- They must be set to a *constant expression* (something that can be computed at compile time), not the result of a function call.

```fusion
const MAX_POINTS: u32 = 100_000
const PI: f64 = 3.14159
```text

Naming convention for constants is ALL_UPPERCASE with underscores.

### 3.1.4 Shadowing

You can declare a new variable with the same name as a previous variable. This is called **shadowing**.

```fusion
fn main() {
    let x = 5
    let x = x + 1    // New variable 'x' hides the previous one

    {
        let x = x * 2
        println("Inner scope x: {}", x) // Prints 12
    }

    println("Outer scope x: {}", x) // Prints 6
}
```text

Shadowing is different from mutation:
1. We use the `let` keyword again, creating a fresh variable.
2. We can change the **type** of the value while reusing the name.

```fusion
let spaces = "   "
let spaces = spaces.len() // First 'spaces' is string, second is int
```text

If we used `mut`, this type change would not be allowed.

---

## 3.2 Data Types

Every value in Fusion has a certain **data type**, which tells the compiler what kind of data is being specified so it knows how to work with that data. Fusion is a **statically typed** language, meaning that it must know the types of all variables at compile time.

### 3.2.1 Scalar Types

A **scalar** type represents a single value. Fusion has four primary scalar types: integers, floating-point numbers, booleans, and characters.

#### Integer Types

An integer is a number without a fractional component.

| Length  | Signed  | Unsigned |
| :------ | :------ | :------- |
| 8-bit   | `i8`    | `u8`     |
| 16-bit  | `i16`   | `u16`    |
| 32-bit  | `i32`   | `u32`    |
| 64-bit  | `i64`   | `u64`    |
| 128-bit | `i128`  | `u128`   |
| Arch    | `isize` | `usize`  |

- **Signed** (`i`): Can be positive, negative, or zero.
- **Unsigned** (`u`): Only positive numbers (and zero).
- **Arch**: `isize` and `usize` depend on the architecture of the computer your program is running on (64-bit on 64-bit systems).

Defaults: If you don't specify a type, Fusion defaults to `i32`.

**Integer Literals**:
- Decimal: `98_222` (underscores can be used for readability)
- Hex: `0xff`
- Octal: `0o77`
- Binary: `0b1111_0000`
- Byte (u8 only): `b'A'`

#### Floating-Point Types

Fusion has two primitive types for floating-point numbers (numbers with decimal points):
- `f32`: 32-bit (single precision)
- `f64`: 64-bit (double precision) - **Default**

```fusion
let x = 2.0      // f64
let y: f32 = 3.0 // f32
```text

#### The Boolean Type

A boolean type has two possible values: `true` and `false`. They are one byte in size.

```fusion
let t = true
let f: bool = false
```text

#### The Character Type

The `char` type is the language's most primitive alphabetic type.

```fusion
let c = 'z'
let z = 'ℤ'
let heart_eyed_cat = '😻'
```text

Fusion `char` literals are specified with single quotes. A `char` represents a **Unicode Scalar Value**, meaning it can represent a lot more than just ASCII: accented letters, Chinese/Japanese/Korean ideographs, emoji, and zero-width spaces are all valid `char` values.

### 3.2.2 Compound Types

Compound types can group multiple values into one type. Fusion has two primitive compound types: tuples and arrays.

#### The Tuple Type

A tuple is a general way of grouping together a number of values with a variety of types into one compound type. Tuples have a fixed length: once declared, they cannot grow or shrink.

```fusion
let tup: (i32, f64, u8) = (500, 6.4, 1)

// Destructuring to access values
let (x, y, z) = tup
println("The value of y is: {}", y)

// Direct access with dot notation
let five_hundred = tup.0
let six_point_four = tup.1
```text

#### The Array Type

An array is a collection of multiple values of the **same type**. Arrays in Fusion have a **fixed length**.

```fusion
let a = [1, 2, 3, 4, 5]
let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
```text

You write an array's type using square brackets with the type of each element, a semicolon, and then the number of elements in the array:

```fusion
let a: [i32; 5] = [1, 2, 3, 4, 5]
```text

**Accessing Array Elements**:

```fusion
let first = a[0]
let second = a[1]
```text

**Out of Bounds Access**:
If you try to access an index that is past the end of the array (e.g., `a[10]`), Fusion will check this at runtime and **panic** (crash safely) rather than allowing buffer overflow bugs that lead to security vulnerabilities.

---

## 3.3 Functions

Functions are prevalent in Fusion code. We define functions using the `fn` keyword.

### 3.3.1 Parameters

Functions can have parameters, which are special variables that are part of a function's signature. When a function has parameters, you can provide it with concrete values (arguments) for those parameters.

```fusion
fn main() {
    print_labeled_measurement(5, 'h')
}

fn print_labeled_measurement(value: i32, unit_label: char) {
    println("The measurement is: {}{}", value, unit_label)
}
```text

In function signatures, you **must** declare the type of each parameter.

### 3.3.2 Statements and Expressions

Function bodies are made up of a series of statements optionally ending in an expression. It is important to distinguish between the two:
- **Statements** are instructions that perform some action and do not return a value.
- **Expressions** evaluate to a result value.

```fusion
// Statement (let definition)
let y = 6

// Expression logic
let y = {
    let x = 3
    x + 1  // Expression! Note the lack of semicolon
}
```text

If you add a semicolon to the end of an expression, you turn it into a statement, and it will then not return a value.

### 3.3.3 Functions with Return Values

Functions can return values to the code that calls them. We don't name return values, but we must declare their type after an arrow (`->`).

```fusion
fn five() -> i32 {
    5  // Implicit return (expression, no semicolon)
}

fn plus_one(x: i32) -> i32 {
    x + 1
}

fn main() {
    let x = plus_one(5)
    println("The value of x is: {}", x)
}
```text

---

## 3.4 Comments

Code tells the computer what to do; comments tell the humans *why* code does what it does.

```fusion
// This is a simple line comment.

/*
   This is a block comment.
   It spans multiple lines.
*/

// In Fusion, idiomatic comments are often single-line comments using //
```text

**Documentation Comments**:
For documenting functions and libraries for other users, use triple slashes `///`. These support Markdown formatting and are used by the `fusion doc` tool.

```fusion
/// Adds one to the number given.
///
/// # Examples
///
/// ```
/// let arg = 5;
/// let answer = fusion::add_one(arg);
/// assert_eq!(6, answer);
/// ```
fn add_one(x: i32) -> i32 {
    x + 1
}
```text

---

## 3.5 Control Flow

The ability to run some code depending on whether a condition is true (`if`) and to run some code repeatedly (`loop`) are essential building blocks.

### 3.5.1 `if` Expressions

An `if` expression allows you to branch your code depending on conditions.

```fusion
fn main() {
    let number = 3

    if number < 5 {
        println("condition was true")
    } else {
        println("condition was false")
    }
}
```text

The condition **must** remain a `bool`. Unlike some languages, Fusion will not automatically convert non-boolean types (like integers) to a boolean. You cannot write `if number { ... }`.

**Using `if` in a `let` Statement**:
Because `if` is an expression, we can use it on the right side of a `let` statement to assign a value based on a condition.

```fusion
let condition = true
let number = if condition { 5 } else { 6 }
```text

Both arms must return the same type.

### 3.5.2 Repetition with Loops

Fusion has three kinds of loops: `loop`, `while`, and `for`.

#### `loop`

The `loop` keyword tells Fusion to execute a block of code over and over again forever or unti you explicitly tell it to stop using `break`.

```fusion
loop {
    println("again!")
    break // Necessary to exit
}
```text

**Returning values from loops**:
You can pass the result of the operation found in the loop to the rest of your code by putting it after the `break` expression.

```fusion
let mut counter = 0
let result = loop {
    counter += 1
    if counter == 10 {
        break counter * 2
    }
} // result is 20
```text

#### `while`

A `while` loop runs as long as a condition is true.

```fusion
let mut number = 3

while number != 0 {
    println("{}!", number)
    number -= 1
}

println("LIFTOFF!!!")
```text

#### `for`

The `for` loop is the most commonly used loop construct in Fusion because of its safety and conciseness when iterating over collections.

**Iterating over an array**:

```fusion
let a = [10, 20, 30, 40, 50]

for element in a {
    println("the value is: {}", element)
}
```text

**Iterating over a range**:

```fusion
for number in (1..4).rev() {
    println("{}!", number)
}
println("LIFTOFF!!!")
```text

This prints: 3!, 2!, 1!, LIFTOFF!!!

---

## 3.6 Summary

You now have a solid foundation in the common concepts of Fusion:
- Variables are immutable by default (`let`), use `mut` to change them.
- Functions are declared with `fn`, params need type annotations, and return values are expressions.
- Primitive types include integers, floats, booleans, and chars.
- Compound types allow grouping data (tuples, arrays).
- Control flow uses `if`, `match` (from Chapter 2), and loops (`loop`, `while`, `for`).

In the next chapter, we will tackle Fusion's most unique and powerful feature: ownership. This concept sets Fusion apart from other languages by enabling memory safety without a garbage collector.

---

## 3.7 Exercises

1. **Fibonacci**: Write a function `fib(n: u32) -> u32` that returns the *n*-th Fibonacci number.
2. **Temperature Conversion**: Create a program that converts temperatures between Fahrenheit and Celsius, allowing the user to choose the direction of conversion.
3. **The Twelve Days of Christmas**: Write a program that prints the lyrics to the Christmas carol "The Twelve Days of Christmas," taking advantage of the repetition in the song (looping).

---

[Next: Chapter 4 - Understanding Memory Safety →](./chapter-04-memory-safety.md)