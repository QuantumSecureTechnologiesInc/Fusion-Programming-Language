# Chapter 3: Structs and Enums

> Defining custom types, enums, methods, traits, and pattern matching

---

## Struct Definitions and Instantiation

Structs are Fusion's primary way to group related data together.

### Basic Struct

```fusion
struct Point {
    x: float,
    y: float,
}

fn main() -> int {
    // Struct literal instantiation
    let p: Point = Point { x: 1.0, y: 2.0 };

    // Field shorthand (when variable name matches field name)
    let x: float = 3.0;
    let y: float = 4.0;
    let p2: Point = Point { x, y };

    println("p = (%f, %f)", p.x, p.y);
    return 0;
}
```

### Struct with Default Values

```fusion
struct Config {
    host: string,
    port: int,
    timeout: int,
    max_retries: int,
}

fn default_config() -> Config {
    return Config {
        host: "localhost",
        port: 8080,
        timeout: 30,
        max_retries: 3,
    };
}

fn main() -> int {
    let config: Config = default_config();
    println("Server: %s:%d", config.host, config.port);
    return 0;
}
```

### Nested Structs

```fusion
struct Address {
    street: string,
    city: string,
    zip: string,
}

struct Person {
    name: string,
    age: int,
    address: Address,
}

fn main() -> int {
    let person: Person = Person {
        name: "Alice",
        age: 30,
        address: Address {
            street: "123 Main St",
            city: "Springfield",
            zip: "12345",
        },
    };

    println("%s lives in %s", person.name, person.address.city);
    return 0;
}
```

---

## Field Access with Dot Notation

```fusion
struct Rectangle {
    width: float,
    height: float,
}

fn area(rect: Rectangle) -> float {
    return rect.width * rect.height;
}

fn perimeter(rect: Rectangle) -> float {
    return 2.0 * (rect.width + rect.height);
}

fn main() -> int {
    let rect: Rectangle = Rectangle { width: 10.0, height: 5.0 };

    // Read fields
    println("Width: %f", rect.width);
    println("Height: %f", rect.height);

    // Compute from fields
    println("Area: %f", area(rect));
    println("Perimeter: %f", perimeter(rect));

    // Mutable field access
    let mut r: Rectangle = rect;
    r.width = 20.0;
    println("New area: %f", area(r));

    return 0;
}
```

---

## Enums

Enums in Fusion can have three kinds of variants: unit, tuple, and struct variants.

### Unit Variants

```fusion
enum Direction {
    North,
    South,
    East,
    West,
}

fn main() -> int {
    let dir: Direction = Direction::North;

    let description: string = match dir {
        Direction::North => "Up",
        Direction::South => "Down",
        Direction::East => "Right",
        Direction::West => "Left",
    };
    println("Direction: %s", description);

    return 0;
}
```

### Tuple Variants

```fusion
enum Shape {
    Circle(float),                    // radius
    Rectangle(float, float),          // width, height
    Triangle(float, float, float),    // three sides
}

fn area(shape: Shape) -> float {
    return match shape {
        Shape::Circle(radius) => 3.14159 * radius * radius,
        Shape::Rectangle(w, h) => w * h,
        Shape::Triangle(a, b, c) => {
            // Heron's formula
            let s: float = (a + b + c) / 2.0;
            let val: float = s * (s - a) * (s - b) * (s - c);
            // Approximate sqrt for demonstration
            let result: float = val;
            result  // In production, use std::math::sqrt
        },
    };
}

fn main() -> int {
    let circle: Shape = Shape::Circle(5.0);
    let rect: Shape = Shape::Rectangle(4.0, 6.0);

    println("Circle area: %f", area(circle));
    println("Rectangle area: %f", area(rect));

    return 0;
}
```

### Struct Variants

```fusion
enum Message {
    Quit,
    Echo { text: string },
    Move { x: int, y: int },
    Color { r: u8, g: u8, b: u8 },
}

fn process(msg: Message) {
    match msg {
        Message::Quit => {
            println("Quitting");
        }
        Message::Echo { text } => {
            println("Echo: %s", text);
        }
        Message::Move { x, y } => {
            println("Moving to (%d, %d)", x, y);
        }
        Message::Color { r, g, b } => {
            println("Color: rgb(%d, %d, %d)", r, g, b);
        }
    }
}

fn main() -> int {
    process(Message::Quit);
    process(Message::Echo { text: "Hello!" });
    process(Message::Move { x: 10, y: 20 });
    process(Message::Color { r: 255, g: 128, b: 0 });
    return 0;
}
```

---

## Pattern Matching on Enums

Pattern matching in Fusion is exhaustive — you must handle all variants or use a wildcard.

```fusion
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(string),  // State name on quarter
}

fn value_in_cents(coin: Coin) -> int {
    return match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => {
            println("Quarter from %s!", state);
            25
        },
    };
}

// Pattern matching with guards
fn classify_number(n: int) -> string {
    return match n {
        0 => "zero",
        x if x > 0 && x % 2 == 0 => "positive even",
        x if x > 0 => "positive odd",
        x if x % 2 == 0 => "negative even",
        _ => "negative odd",
    };
}

fn main() -> int {
    let coin: Coin = Coin::Quarter("Vermont");
    println("Value: %d cents", value_in_cents(coin));

    println("5 is %s", classify_number(5));
    println("-4 is %s", classify_number(-4));

    return 0;
}
```

### Nested Pattern Matching

```fusion
enum Color {
    Rgb(u8, u8, u8),
    Hsl(float, float, float),
}

enum Theme {
    Light(Color),
    Dark(Color),
    Auto(Color, Color),
}

fn describe_theme(theme: Theme) -> string {
    return match theme {
        Theme::Light(Color::Rgb(r, g, b)) => {
            "Light theme with RGB color"
        }
        Theme::Dark(Color::Hsl(h, s, l)) => {
            "Dark theme with HSL color"
        }
        Theme::Auto(light, dark) => {
            "Auto-switching theme"
        }
        _ => "Unknown theme",
    };
}
```

---

## Method Definitions (Impl Blocks)

Methods are defined inside `impl` blocks. The first parameter is typically `self` (the instance).

```fusion
struct Point {
    x: float,
    y: float,
}

impl Point {
    // Constructor method (convention: 'new')
    fn new(x: float, y: float) -> Point {
        return Point { x, y };
    }

    // Method that takes self by value
    fn distance_from_origin(self) -> float {
        return self.x * self.x + self.y * self.y;
    }

    // Method that takes self by reference (borrowed)
    fn distance_to(self, other: Point) -> float {
        let dx: float = self.x - other.x;
        let dy: float = self.y - other.y;
        return dx * dx + dy * dy;
    }

    // Mutable method
    fn translate(mut self, dx: float, dy: float) -> Point {
        self.x = self.x + dx;
        self.y = self.y + dy;
        return self;
    }
}

fn main() -> int {
    let p1: Point = Point::new(3.0, 4.0);
    let p2: Point = Point::new(6.0, 8.0);

    println("Distance from origin: %f", p1.distance_from_origin());
    println("Distance between points: %f", p1.distance_to(p2));

    let p3: Point = p1.translate(1.0, 1.0);
    println("Translated: (%f, %f)", p3.x, p3.y);

    return 0;
}
```

### Multiple Impl Blocks

You can have multiple `impl` blocks for the same type:

```fusion
struct Calculator {
    value: float,
}

impl Calculator {
    fn new() -> Calculator {
        return Calculator { value: 0.0 };
    }

    fn add(self, n: float) -> Calculator {
        return Calculator { value: self.value + n };
    }
}

impl Calculator {
    // Second impl block (useful for trait implementations later)
    fn subtract(self, n: float) -> Calculator {
        return Calculator { value: self.value - n };
    }

    fn get(self) -> float {
        return self.value;
    }
}
```

---

## Traits and Trait Implementations

Traits define shared behavior that types can implement.

### Defining a Trait

```fusion
trait Drawable {
    fn draw(self);
    fn area(self) -> float;
}

trait Printable {
    fn to_string(self) -> string;
}
```

### Implementing a Trait

```fusion
struct Circle {
    radius: float,
}

struct Square {
    side: float,
}

impl Drawable for Circle {
    fn draw(self) {
        println("Drawing circle with radius %f", self.radius);
    }

    fn area(self) -> float {
        return 3.14159 * self.radius * self.radius;
    }
}

impl Drawable for Square {
    fn draw(self) {
        println("Drawing square with side %f", self.side);
    }

    fn area(self) -> float {
        return self.side * self.side;
    }
}

fn main() -> int {
    let c: Circle = Circle { radius: 5.0 };
    let s: Square = Square { side: 4.0 };

    c.draw();
    s.draw();

    println("Circle area: %f", c.area());
    println("Square area: %f", s.area());

    return 0;
}
```

### Trait Bounds

Use trait bounds to constrain generic types:

```fusion
trait Summarizable {
    fn summary(self) -> string;
}

fn notify(item: Summarizable) {
    println("Breaking news: %s", item.summary());
}

// Multiple trait bounds
trait Drawable {
    fn draw(self);
}

trait Printable {
    fn to_string(self) -> string;
}

fn render(item: Drawable + Printable) {
    item.draw();
    println("%s", item.to_string());
}
```

### Default Implementations

```fusion
trait Greetable {
    fn name(self) -> string;

    // Default implementation
    fn greet(self) -> string {
        return "Hello, " + self.name() + "!";
    }
}

struct User {
    name: string,
    age: int,
}

impl Greetable for User {
    fn name(self) -> string {
        return self.name;
    }
    // Using default greet() implementation
}

fn main() -> int {
    let user: User = User { name: "Alice", age: 30 };
    println(user.greet());  // "Hello, Alice!"
    return 0;
}
```

---

## Generic Structs and Enums

Generics allow you to write code that works with multiple types while maintaining type safety.

### Generic Structs

```fusion
// A Point that works with any numeric type
struct Point<T> {
    x: T,
    y: T,
}

fn main() -> int {
    // Integer point
    let int_point: Point<int> = Point { x: 10, y: 20 };

    // Float point
    let float_point: Point<float> = Point { x: 1.5, y: 2.5 };

    println("Int point: (%d, %d)", int_point.x, int_point.y);
    println("Float point: (%f, %f)", float_point.x, float_point.y);

    return 0;
}
```

### Generic Structs with Multiple Type Parameters

```fusion
// A Pair with potentially different types
struct Pair<T, U> {
    first: T,
    second: U,
}

// A HashMap entry
struct Entry<K, V> {
    key: K,
    value: V,
}

fn main() -> int {
    // Pair with different types
    let pair: Pair<int, string> = Pair { first: 42, second: "hello" };

    // Entry for a map
    let entry: Entry<string, int> = Entry { key: "count".to_string(), value: 5 };

    println("Pair: (%d, %s)", pair.first, pair.second);
    println("Entry: %s = %d", entry.key, entry.value);

    return 0;
}
```

### Generic Enums

```fusion
// Result type for error handling
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Optional value
enum Option<T> {
    Some(T),
    None,
}

fn divide(a: float, b: float) -> Result<float, string> {
    if b == 0.0 {
        return Result::Err("Division by zero".to_string());
    }
    return Result::Ok(a / b);
}

fn find_first(arr: [int], target: int) -> Option<int> {
    for i in 0..arr.len() {
        if arr[i] == target {
            return Option::Some(i);
        }
    }
    return Option::None;
}

fn main() -> int {
    // Using Result
    let result: Result<float, string> = divide(10.0, 3.0);
    match result {
        Result::Ok(value) => println("Result: %f", value),
        Result::Err(msg) => println("Error: %s", msg),
    }

    // Using Option
    let numbers: [int; 5] = [10, 20, 30, 40, 50];
    let found: Option<int> = find_first(numbers, 30);
    match found {
        Option::Some(index) => println("Found at index: %d", index),
        Option::None => println("Not found"),
    }

    return 0;
}
```

### Generic Impl Blocks

```fusion
struct Stack<T> {
    items: [T; 100],
    top: int,
}

impl<T> Stack<T> {
    fn new() -> Stack<T> {
        // Note: In real code, you'd need a way to initialize the array
        return Stack { items: [/* default */; 100], top: 0 };
    }

    fn push(mut self, item: T) {
        self.items[self.top] = item;
        self.top = self.top + 1;
    }

    fn pop(mut self) -> Option<T> {
        if self.top == 0 {
            return Option::None;
        }
        self.top = self.top - 1;
        return Option::Some(self.items[self.top]);
    }

    fn peek(self) -> Option<T> {
        if self.top == 0 {
            return Option::None;
        }
        return Option::Some(self.items[self.top - 1]);
    }

    fn is_empty(self) -> bool {
        return self.top == 0;
    }
}

fn main() -> int {
    let mut int_stack: Stack<int> = Stack::new();
    int_stack.push(10);
    int_stack.push(20);
    int_stack.push(30);

    match int_stack.pop() {
        Option::Some(value) => println("Popped: %d", value),  // 30
        Option::None => println("Stack is empty"),
    }

    return 0;
}
```

### Trait Bounds with Generics

```fusion
// Require that T implements Addable trait
trait Addable {
    fn add(self, other: T) -> T;
}

// Generic function with trait bound
fn sum<T: Addable>(items: [T]) -> T {
    let mut total: T = items[0];
    for i in 1..items.len() {
        total = total.add(items[i]);
    }
    return total;
}

// Multiple trait bounds
fn process<T: Printable + Clone>(item: T) -> T {
    println("%s", item.to_string());
    return item.clone();
}

fn main() -> int {
    let numbers: [int; 4] = [1, 2, 3, 4];
    let total: int = sum(numbers);
    println("Sum: %d", total);  // 10

    return 0;
}
```

### Common Patterns and Anti-Patterns with Generics

```fusion
// GOOD: Constrained generics are more flexible
fn compare<T: PartialEq>(a: T, b: T) -> bool {
    return a == b;
}

// GOOD: Use generic enums for type-safe error handling
enum IoError {
    NotFound,
    PermissionDenied,
    Other(string),
}

fn read_file(path: string) -> Result<string, IoError> {
    // Implementation
    return Result::Ok("content".to_string());
}

// BAD: Over-using generics when concrete types suffice
// Don't make everything generic if you only ever use one type
struct OnlyIntWrapper<T> {  // Unnecessary generic
    value: T,
}

// BETTER: Use concrete type when generic isn't needed
struct Counter {
    value: int,
}
```

### Common Mistakes with Generics

```fusion
// WRONG: Forgetting to specify type when compiler can't infer
// let result = divide(10.0, 0.0);  // Error: ambiguous type

// CORRECT: Specify the type explicitly
let result: Result<float, string> = divide(10.0, 0.0);

// WRONG: Using generic when concrete type is needed
// fn process(x: T) { ... }  // Error: T not defined

// CORRECT: Define the generic parameter
fn process<T>(x: T) { ... }

// WRONG: Mixing incompatible types in generic context
// let p: Point<int> = Point { x: 1.5, y: 2 };  // Error: types don't match

// CORRECT: Use consistent types
let p: Point<int> = Point { x: 1, y: 2 };
let q: Point<float> = Point { x: 1.5, y: 2.5 };
```

---

## Common Patterns and Anti-Patterns

### Good Patterns

```fusion
// Use newtype pattern for type safety
struct UserId(int);
struct Email(string);

fn get_user(id: UserId) -> User {
    // Implementation
}

// Use enums for state machines
enum ConnectionState {
    Disconnected,
    Connecting { attempt: int },
    Connected { session_id: string },
    Error { message: string },
}

// Use traits for polymorphism
trait Serializable {
    fn serialize(self) -> string;
    fn deserialize(data: string) -> Self;
}
```

### Anti-Patterns

```fusion
// BAD: God struct with too many fields
struct MegaStruct {  // Don't do this
    field1: string,
    field2: int,
    field3: float,
    field4: bool,
    field5: [int; 100],
    field6: string,
    field7: int,
    field8: float,
}

// BETTER: Break into smaller focused structs
struct Metadata {
    name: string,
    version: int,
}

struct Config {
    enabled: bool,
    timeout: float,
}

// BAD: Using strings for everything (stringly-typed)
struct User {
    name: string,
    role: string,  // Should be an enum
    status: string,  // Should be an enum
}

// BETTER: Use enums for type-safe states
enum Role { Admin, User, Guest }
enum Status { Active, Inactive, Banned }
```

### Builder Pattern

```fusion
struct Request {
    method: string,
    url: string,
    headers: [string; 10],
    body: string,
}

impl Request {
    fn get(url: string) -> Request {
        return Request {
            method: "GET",
            url,
            headers: [""; 10],
            body: "",
        };
    }

    fn with_header(mut self, header: string) -> Request {
        // Add header logic
        return self;
    }

    fn with_body(mut self, body: string) -> Request {
        self.body = body;
        return self;
    }
}

fn main() -> int {
    let req: Request = Request::get("https://api.example.com")
        .with_header("Content-Type: application/json")
        .with_body("{\"key\": \"value\"}");

    println("%s %s", req.method, req.url);
    return 0;
}
```

### Type Aliases

```fusion
type UserID = int;
type Email = string;

fn send_notification(user: UserID, email: Email) {
    println("Sending to user %d at %s", user, email);
}
```

---

## Tips and Best Practices

1. **Use constructors**: Always provide a `new` method for structs.
2. **Keep structs small**: Large structs should be broken into smaller ones.
3. **Use enums for state**: Enums are perfect for modeling states or variants.
4. **Implement Display-like traits**: Always implement `to_string` or similar for debugging.
5. **Prefer composition over inheritance**: Use traits for shared behavior, not inheritance.

---

## Cross-References

- **Chapter 2**: Syntax for basic type system
- **Chapter 4**: Memory Safety for ownership of structs
- **Chapter 5**: Generics for parameterized structs and traits
- **Chapter 6**: Standard Library for common traits
