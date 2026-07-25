# Chapter 5: Generics

> Writing flexible, reusable code with generic functions, structs, and trait bounds

---

## Generic Functions

Generics allow you to write functions that work with multiple types.

### Basic Generic Function

```fusion
fn largest<T>(list: [T]) -> T {
    let mut largest: T = list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    return largest;
}

fn main() -> int {
    let numbers: [int; 5] = [34, 50, 25, 100, 65];
    println("Largest number: %d", largest(numbers));

    let chars: [char; 3] = ['y', 'm', 'a'];
    println("Largest char: %c", largest(chars));

    return 0;
}
```

### Multiple Generic Parameters

```fusion
fn pair<A, B>(first: A, second: B) -> (A, B) {
    return (first, second);
}

fn swap<A, B>(pair: (A, B)) -> (B, A) {
    return (pair.1, pair.0);
}

fn main() -> int {
    let p: (int, string) = pair(42, "hello");
    println("p = (%d, %s)", p.0, p.1);

    let swapped: (string, int) = swap(p);
    println("swapped = (%s, %d)", swapped.0, swapped.1);

    return 0;
}
```

### Generic Functions with Trait Bounds

```fusion
trait Summable {
    fn zero() -> Self;
    fn add(self, other: Self) -> Self;
}

fn sum_all<T: Summable>(items: [T]) -> T {
    let mut total: T = T::zero();
    for item in items {
        total = total.add(item);
    }
    return total;
}
```

---

## Generic Structs

Structs can be parameterized over types:

### Basic Generic Struct

```fusion
struct Point<T> {
    x: T,
    y: T,
}

fn main() -> int {
    let int_point: Point<int> = Point { x: 1, y: 2 };
    let float_point: Point<float> = Point { x: 1.0, y: 2.0 };

    println("Int point: (%d, %d)", int_point.x, int_point.y);
    println("Float point: (%f, %f)", float_point.x, float_point.y);

    return 0;
}
```

### Multiple Generic Parameters

```fusion
struct Pair<A, B> {
    first: A,
    second: B,
}

impl<A, B> Pair<A, B> {
    fn new(first: A, second: B) -> Pair<A, B> {
        return Pair { first, second };
    }

    fn first(self) -> A {
        return self.first;
    }

    fn second(self) -> B {
        return self.second;
    }
}

fn main() -> int {
    let p: Pair<int, string> = Pair::new(42, "hello");
    println("first=%d, second=%s", p.first(), p.second());
    return 0;
}
```

### Generic Struct with Default Type

```fusion
struct Vec<T> {
    data: *T,
    length: int,
    capacity: int,
}

impl<T> Vec<T> {
    fn new() -> Vec<T> {
        return Vec {
            data: 0 as *T,
            length: 0,
            capacity: 0,
        };
    }

    fn push(mut self, item: T) {
        self.length = self.length + 1;
    }

    fn len(self) -> int {
        return self.length;
    }
}
```

---

## Trait Bounds

Trait bounds constrain which types can be used with generics.

### Basic Trait Bounds

```fusion
trait Printable {
    fn to_string(self) -> string;
}

fn print_item<T: Printable>(item: T) {
    println("%s", item.to_string());
}

// Multiple trait bounds
trait Drawable {
    fn draw(self);
}

trait Sized {
    fn size(self) -> int;
}

fn render<T: Printable + Drawable>(item: T) {
    println("Rendering: %s", item.to_string());
    item.draw();
}
```

### Where Clauses

For complex bounds, use `where` clauses:

```fusion
fn process<T, U>(item: T, config: U) -> string
where
    T: Printable + Clone,
    U: Sized,
{
    return item.to_string();
}
```

### Trait Objects (Dynamic Dispatch)

```fusion
trait Animal {
    fn speak(self) -> string;
}

struct Dog {
    name: string,
}

struct Cat {
    name: string,
}

impl Animal for Dog {
    fn speak(self) -> string {
        return "Woof!";
    }
}

impl Animal for Cat {
    fn speak(self) -> string {
        return "Meow!";
    }
}

fn make_speak(animal: &dyn Animal) {
    println("%s", animal.speak());
}

fn main() -> int {
    let dog: Dog = Dog { name: "Rex" };
    let cat: Cat = Cat { name: "Whiskers" };

    make_speak(&dog);
    make_speak(&cat);

    return 0;
}
```

---

## Monomorphization

Fusion uses **monomorphization** — the compiler generates specialized code for each concrete type used with generics.

### How It Works

```fusion
// Your generic code:
fn add<T>(a: T, b: T) -> T {
    return a + b;
}

// When you call:
let x: int = add(1, 2);
let y: float = add(1.0, 2.0);

// The compiler generates:
fn add_int(a: int, b: int) -> int {
    return a + b;
}

fn add_float(a: float, b: float) -> float {
    return a + b;
}
```

### Zero-Cost Abstractions

```fusion
// This generic function...
fn max<T>(a: T, b: T) -> T {
    if a > b { return a; } else { return b; }
}

// ...compiles to the same code as manually writing:
fn max_int(a: int, b: int) -> int {
    if a > b { return a; } else { return b; }
}

// No runtime overhead for generics!
```

---

## Type Inference

Fusion can infer generic types from usage:

```fusion
fn identity<T>(x: T) -> T {
    return x;
}

fn main() -> int {
    // Type inferred as int
    let x: int = identity(42);

    // Type inferred as string
    let y: string = identity("hello");

    // Type inferred from context
    let numbers: [int; 3] = [1, 2, 3];
    let first: int = identity(numbers[0]);

    println("x=%d, y=%s, first=%d", x, y, first);
    return 0;
}
```

### Inference in Closures

```fusion
fn apply<T, U>(f: fn(T) -> U, value: T) -> U {
    return f(value);
}

fn main() -> int {
    // Type inferred from usage
    let double = |x: int| x * 2;
    let result: int = apply(double, 5);
    println("Result: %d", result);  // 10

    return 0;
}
```

---

## Common Patterns

### Generic Enum

```fusion
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn main() -> int {
    let some_value: Option<int> = Option::Some(42);
    let no_value: Option<int> = Option::None;

    match some_value {
        Option::Some(v) => println("Value: %d", v),
        Option::None => println("No value"),
    }

    return 0;
}
```

### Generic Trait Implementation

```fusion
trait Convertible<T> {
    fn convert(self) -> T;
}

impl Convertible<int> for float {
    fn convert(self) -> int {
        return self as int;
    }
}

impl Convertible<string> for int {
    fn convert(self) -> string {
        return self.to_string();
    }
}

fn main() -> int {
    let pi: float = 3.14;
    let as_int: int = pi.convert();
    let as_string: string = as_int.convert();

    println("float->int: %d", as_int);
    println("int->string: %s", as_string);

    return 0;
}
```

### Phantom Types

```fusion
// Use phantom types for compile-time state tracking
struct Locked;
struct Unlocked;

struct Door<State> {
    _state: std::marker::PhantomData<State>,
}

impl Door<Unlocked> {
    fn new() -> Door<Unlocked> {
        return Door { _state: std::marker::PhantomData };
    }

    fn lock(self) -> Door<Locked> {
        println("Locking door");
        return Door { _state: std::marker::PhantomData };
    }
}

impl Door<Locked> {
    fn unlock(self) -> Door<Unlocked> {
        println("Unlocking door");
        return Door { _state: std::marker::PhantomData };
    }
}
```

---

## Tips and Best Practices

1. **Don't over-generify**: Only use generics when you need multiple type support.
2. **Use trait bounds liberally**: Constrain generics to ensure required operations exist.
3. **Prefer static dispatch**: Use monomorphization (generics) over dynamic dispatch (trait objects) when possible.
4. **Let the compiler infer**: Don't always specify types — let inference work.
5. **Document complex bounds**: Use `where` clauses and comments for readability.

---

## Complete Example: Generic Data Structure

```fusion
struct Stack<T> {
    items: Vec<T>,
    capacity: int,
}

impl<T> Stack<T> {
    fn new(capacity: int) -> Stack<T> {
        return Stack {
            items: Vec::with_capacity(capacity),
            capacity,
        };
    }

    fn push(mut self: &mut Stack<T>, item: T) -> Result<(), string> {
        if self.items.len() >= self.capacity {
            return Err("Stack overflow".to_string());
        }
        self.items.push(item);
        return Ok(());
    }

    fn pop(mut self: &mut Stack<T>) -> Option<T> {
        return self.items.pop();
    }

    fn peek(self: &Stack<T>) -> Option<&T> {
        if self.items.len() == 0 {
            return None;
        }
        return Some(&self.items[self.items.len() - 1]);
    }

    fn is_empty(self: &Stack<T>) -> bool {
        return self.items.len() == 0;
    }

    fn len(self: &Stack<T>) -> int {
        return self.items.len();
    }

    fn clear(mut self: &mut Stack<T>) {
        self.items.clear();
    }
}

// Implement Display for any T that implements Display
impl<T: Printable> Stack<T> {
    fn display(self: &Stack<T>) {
        print!("Stack[");
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("%s", item.to_string());
        }
        println!("]");
    }
}

fn main() -> int {
    // Integer stack
    let mut int_stack: Stack<int> = Stack::new(10);
    int_stack.push(1);
    int_stack.push(2);
    int_stack.push(3);

    println("Int stack length: %d", int_stack.len());
    println("Top: %d", int_stack.peek().unwrap());

    while let Some(value) = int_stack.pop() {
        println("Popped: %d", value);
    }

    // String stack
    let mut str_stack: Stack<string> = Stack::new(5);
    str_stack.push("hello".to_string());
    str_stack.push("world".to_string());

    str_stack.display();

    return 0;
}
```

---

## Complete Example: Generic Algorithm

```fusion
// Generic sorting algorithm
fn bubble_sort<T: Comparable>(arr: &mut Vec<T>) {
    let n: int = arr.len();
    for i in 0..n {
        for j in 0..n-i-1 {
            if arr[j] > arr[j+1] {
                arr.swap(j, j+1);
            }
        }
    }
}

// Generic binary search
fn binary_search<T: Comparable>(arr: &[T], target: &T) -> Option<int> {
    let mut low: int = 0;
    let mut high: int = arr.len() - 1;

    while low <= high {
        let mid: int = low + (high - low) / 2;
        match arr[mid].compare(target) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => low = mid + 1,
            Ordering::Greater => high = mid - 1,
        }
    }

    return None;
}

// Generic map function
fn map<T, U>(arr: &[T], f: fn(&T) -> U) -> Vec<U> {
    let mut result: Vec<U> = Vec::with_capacity(arr.len());
    for item in arr {
        result.push(f(item));
    }
    return result;
}

// Generic filter function
fn filter<T>(arr: &[T], predicate: fn(&T) -> bool) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    for item in arr {
        if predicate(item) {
            result.push(item.clone());
        }
    }
    return result;
}

// Generic reduce function
fn reduce<T, U>(arr: &[T], initial: U, f: fn(U, &T) -> U) -> U {
    let mut acc: U = initial;
    for item in arr {
        acc = f(acc, item);
    }
    return acc;
}

// Generic find function
fn find<T>(arr: &[T], predicate: fn(&T) -> bool) -> Option<&T> {
    for item in arr {
        if predicate(item) {
            return Some(item);
        }
    }
    return None;
}

// Generic partition function
fn partition<T>(arr: &[T], predicate: fn(&T) -> bool) -> (Vec<T>, Vec<T>) {
    let mut matching: Vec<T> = Vec::new();
    let mut non_matching: Vec<T> = Vec::new();

    for item in arr {
        if predicate(item) {
            matching.push(item.clone());
        } else {
            non_matching.push(item.clone());
        }
    }

    return (matching, non_matching);
}

fn main() -> int {
    // Sort integers
    let mut numbers: Vec<int> = vec![5, 2, 8, 1, 9, 3];
    bubble_sort(&mut numbers);
    println("Sorted: %s", numbers.to_string());

    // Sort strings
    let mut names: Vec<string> = vec!["Charlie".to_string(), "Alice".to_string(), "Bob".to_string()];
    bubble_sort(&mut names);
    println("Names: %s", names.to_string());

    // Binary search
    let sorted: Vec<int> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let index: Option<int> = binary_search(&sorted, &5);
    match index {
        Some(i) => println("Found 5 at index %d", i),
        None => println("5 not found"),
    }

    // Map: double all numbers
    let doubled: Vec<int> = map(&numbers, |x| x * 2);
    println("Doubled: %s", doubled.to_string());

    // Filter: keep only even numbers
    let evens: Vec<int> = filter(&numbers, |x| x %% 2 == 0);
    println("Evens: %s", evens.to_string());

    // Reduce: sum all numbers
    let sum: int = reduce(&numbers, 0, |acc, x| acc + x);
    println("Sum: %d", sum);

    // Find first even number
    let first_even: Option<&int> = find(&numbers, |x| x %% 2 == 0);
    match first_even {
        Some(n) => println("First even: %d", n),
        None => println("No even numbers"),
    }

    // Partition: separate evens and odds
    let (evens, odds): (Vec<int>, Vec<int>) = partition(&numbers, |x| x %% 2 == 0);
    println("Evens: %s", evens.to_string());
    println("Odds: %s", odds.to_string());

    return 0;
}
```

---

## Cross-References

- **Chapter 3**: Structs and Enums for defining types
- **Chapter 4**: Memory Safety for ownership of generic values
- **Chapter 6**: Standard Library for generic collections
- **Chapter 15**: Reference for complete type system rules
