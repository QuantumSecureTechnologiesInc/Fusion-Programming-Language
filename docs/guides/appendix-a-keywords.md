> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Appendix A: Keywords and Operators

This appendix provides a comprehensive reference for Fusion's keywords, operators, and syntax elements.

---

## Keywords

Fusion reserves the following keywords. They cannot be used as identifiers.

### Currently Used Keywords

| Keyword    | Description                              |
| :--------- | :--------------------------------------- |
| `as`       | Type casting or renaming imports         |
| `async`    | Declare asynchronous function            |
| `await`    | Wait for async operation                 |
| `break`    | Exit loop early                          |
| `class`    | Define a struct/class type               |
| `const`    | Define a compile-time constant           |
| `continue` | Skip to next loop iteration              |
| `crate`    | Refer to current crate root              |
| `dyn`      | Dynamic dispatch trait object            |
| `else`     | Alternative branch                       |
| `enum`     | Define an enumeration                    |
| `extern`   | Link external code                       |
| `false`    | Boolean false literal                    |
| `fn`       | Define a function                        |
| `for`      | Iterate over a collection                |
| `if`       | Conditional branch                       |
| `impl`     | Implement functionality                  |
| `in`       | Part of `for` loop syntax                |
| `let`      | Bind a variable                          |
| `loop`     | Infinite loop                            |
| `match`    | Pattern match                            |
| `mod`      | Define a module                          |
| `move`     | Force closure to take ownership          |
| `mut`      | Mark as mutable                          |
| `pub`      | Make item public                         |
| `ref`      | Bind by reference in pattern             |
| `return`   | Return from function                     |
| `self`     | Current instance                         |
| `Self`     | Type alias for impl type                 |
| `static`   | Static lifetime or global                |
| `struct`   | Define a structure (alias for `class`)   |
| `super`    | Parent module                            |
| `trait`    | Define a trait                           |
| `true`     | Boolean true literal                     |
| `type`     | Type alias                               |
| `unsafe`   | Mark unsafe code block                   |
| `use`      | Import items into scope                  |
| `where`    | Specify trait bounds                     |
| `while`    | Conditional loop                         |
| `with`     | Context manager (e.g., `with no_grad()`) |

### Reserved Keywords (Future Use)

| Keyword    | Potential Use            |
| :--------- | :----------------------- |
| `abstract` | Abstract types           |
| `become`   | Tail recursion           |
| `box`      | Heap allocation          |
| `do`       | Do-while loops           |
| `final`    | Prevent overriding       |
| `macro`    | Macro definition         |
| `override` | Explicit method override |
| `priv`     | Private visibility       |
| `qubit`    | Native qubit type        |
| `tensor`   | Native tensor type       |
| `typeof`   | Type inspection          |
| `unsized`  | Unsized types            |
| `virtual`  | Virtual methods          |
| `yield`    | Generator yield          |

---

## Operators

### Arithmetic Operators

| Operator | Name               | Example |
| :------- | :----------------- | :------ |
| `+`      | Addition           | `a + b` |
| `-`      | Subtraction        | `a - b` |
| `*`      | Multiplication     | `a * b` |
| `/`      | Division           | `a / b` |
| `%`      | Remainder (modulo) | `a % b` |

### Comparison Operators

| Operator | Name             | Example  |
| :------- | :--------------- | :------- |
| `==`     | Equal            | `a == b` |
| `!=`     | Not equal        | `a != b` |
| `<`      | Less than        | `a < b`  |
| `>`      | Greater than     | `a > b`  |
| `<=`     | Less or equal    | `a <= b` |
| `>=`     | Greater or equal | `a >= b` |

### Logical Operators

| Operator | Name        | Example    |
| :------- | :---------- | :--------- |
| `&&`     | Logical AND | `a && b`   |
| `\|\|`   | Logical OR  | `a \|\| b` |
| `!`      | Logical NOT | `!a`       |

### Bitwise Operators

| Operator | Name        | Example  |
| :------- | :---------- | :------- |
| `&`      | Bitwise AND | `a & b`  |
| `\|`     | Bitwise OR  | `a \| b` |
| `^`      | Bitwise XOR | `a ^ b`  |
| `!`      | Bitwise NOT | `!a`     |
| `<<`     | Left shift  | `a << 2` |
| `>>`     | Right shift | `a >> 2` |

### Assignment Operators

| Operator | Name               | Example   |
| :------- | :----------------- | :-------- |
| `=`      | Assignment         | `a = b`   |
| `+=`     | Add assign         | `a += b`  |
| `-=`     | Subtract assign    | `a -= b`  |
| `*=`     | Multiply assign    | `a *= b`  |
| `/=`     | Divide assign      | `a /= b`  |
| `%=`     | Remainder assign   | `a %= b`  |
| `&=`     | AND assign         | `a &= b`  |
| `\|=`    | OR assign          | `a \|= b` |
| `^=`     | XOR assign         | `a ^= b`  |
| `<<=`    | Left shift assign  | `a <<= 2` |
| `>>=`    | Right shift assign | `a >>= 2` |

### Reference Operators

| Operator | Name               | Example  |
| :------- | :----------------- | :------- |
| `&`      | Borrow (reference) | `&x`     |
| `&mut`   | Mutable borrow     | `&mut x` |
| `*`      | Dereference        | `*ptr`   |

### Range Operators

| Operator | Name            | Example  |
| :------- | :-------------- | :------- |
| `..`     | Exclusive range | `0..10`  |
| `..=`    | Inclusive range | `0..=10` |
| `..`     | From start      | `..5`    |
| `..`     | To end          | `5..`    |

### Other Operators

| Operator | Name                | Example           |
| :------- | :------------------ | :---------------- |
| `?`      | Error propagation   | `file.read()?`    |
| `.`      | Field/method access | `obj.field`       |
| `::`     | Path separator      | `std::io`         |
| `->`     | Return type         | `fn foo() -> int` |
| `=>`     | Match arm           | `x => y`          |
| `@`      | Pattern binding     | `x @ 1..=5`       |
| `_`      | Wildcard pattern    | `let _ = x`       |

---

## Operator Precedence

From highest to lowest precedence:

1. Method calls, field access (`.`)
2. Function calls, array indexing
3. `?` (error propagation)
4. Unary operators (`-`, `!`, `*`, `&`)
5. `as` (type casting)
6. `*`, `/`, `%`
7. `+`, `-`
8. `<<`, `>>`
9. `&`
10. `^`
11. `|`
12. `==`, `!=`, `<`, `>`, `<=`, `>=`
13. `&&`
14. `||`
15. `..`, `..=`
16. `=`, `+=`, `-=`, etc.
17. `return`, `break`, closures

---

## Symbols and Punctuation

| Symbol | Usage                            |
| :----- | :------------------------------- |
| `{}`   | Blocks, struct literals          |
| `[]`   | Arrays, indexing, attributes     |
| `()`   | Tuples, function calls, grouping |
| `;`    | Statement terminator (optional)  |
| `,`    | Separator                        |
| `:`    | Type annotation                  |
| `'`    | Lifetimes, characters            |
| `"`    | Strings                          |
| `#`    | Attributes                       |
| `$`    | Macro variables                  |
| `\|`   | Closure parameters               |

---

## Attributes

| Attribute           | Purpose                 |
| :------------------ | :---------------------- |
| `#[derive(...)]`    | Auto-implement traits   |
| `#[test]`           | Mark test function      |
| `#[cfg(condition)]` | Conditional compilation |
| `#[allow(lint)]`    | Suppress warning        |
| `#[deny(lint)]`     | Treat warning as error  |
| `#[inline]`         | Suggest inlining        |
| `#[must_use]`       | Warn if result unused   |
| `#[deprecated]`     | Mark as deprecated      |
| `#[doc = "..."]`    | Documentation           |

---

## Type Syntax

```fusion
// Primitives
i8, i16, i32, i64, i128, isize
u8, u16, u32, u64, u128, usize
f32, f64
bool
char
str

// Aliases
int    // i64
float  // f64

// Compound
[T; N]        // Array of N elements
(T, U, V)     // Tuple
&T            // Immutable reference
&mut T        // Mutable reference
*const T      // Raw pointer (immutable)
*mut T        // Raw pointer (mutable)
fn(T) -> U    // Function pointer
Box<T>        // Owned heap pointer
Rc<T>         // Reference counted
Arc<T>        // Atomic reference counted
Vec<T>        // Dynamic array
Option<T>     // Optional value
Result<T, E>  // Success or error
HashMap<K, V> // Hash map

// Generics
T                    // Type parameter
T: Trait             // Bounded type
T: Trait + Trait2    // Multiple bounds
T: 'a                // Lifetime bound
'a                   // Lifetime
'static              // Static lifetime

// Trait objects
dyn Trait            // Dynamic dispatch
Box<dyn Trait>       // Boxed trait object
```text

---

This appendix serves as a quick reference for Fusion's syntax elements. For detailed explanations, refer to the relevant chapters.

[Back to Table of Contents](./README.md)