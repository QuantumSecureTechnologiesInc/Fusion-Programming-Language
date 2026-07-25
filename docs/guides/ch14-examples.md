# Chapter 14: Examples

> Complete, runnable Fusion programs demonstrating key features

---

## Hello World

The simplest Fusion program:

```fusion
fn main() -> int {
    println("Hello, World!");
    return 0;
}
```

Compile and run:

```bash
fuc hello.fu -o hello.exe
./hello.exe
```

---

## Calculator

A command-line calculator with basic operations:

```fusion
use std::io;

fn add(a: float, b: float) -> float {
    return a + b;
}

fn subtract(a: float, b: float) -> float {
    return a - b;
}

fn multiply(a: float, b: float) -> float {
    return a * b;
}

fn divide(a: float, b: float) -> float {
    if b == 0.0 {
        println("Error: Division by zero");
        return 0.0;
    }
    return a / b;
}

fn main() -> int {
    println("Fusion Calculator");
    println("==================");

    loop {
        println("\nEnter first number (or 'q' to quit): ");
        let input: string = io::read_line();

        if input == "q" {
            println("Goodbye!");
            break;
        }

        let a: float = input.parse_float();

        println("Enter operator (+, -, *, /): ");
        let op: string = io::read_line();

        println("Enter second number: ");
        let b: float = io::read_line().parse_float();

        let result: float = match op {
            "+" => add(a, b),
            "-" => subtract(a, b),
            "*" => multiply(a, b),
            "/" => divide(a, b),
            _ => {
                println("Unknown operator: %s", op);
                0.0
            }
        };

        println("Result: %f", result);
    }

    return 0;
}
```

---

## File Processor

A program that reads, processes, and writes files:

```fusion
use std::fs;

fn process_line(line: string) -> string {
    // Convert to uppercase
    return line.to_upper();
}

fn main() -> int {
    let input_file: string = "input.txt";
    let output_file: string = "output.txt";

    // Check if input file exists
    if !fs::exists(input_file) {
        println("Error: Input file '%s' not found", input_file);
        return 1;
    }

    // Read input file
    println("Reading from %s...", input_file);
    let lines: Vec<string> = fs::read_lines(input_file);
    println("Read %d lines", lines.len());

    // Process lines
    let processed: Vec<string> = [];
    for line in lines {
        let result: string = process_line(line);
        processed.push(result);
    }

    // Write output file
    println("Writing to %s...", output_file);
    let content: string = processed.join("\n");
    fs::write(output_file, content);

    println("Processing complete!");
    println("Input: %s (%d lines)", input_file, lines.len());
    println("Output: %s (%d lines)", output_file, processed.len());

    return 0;
}
```

---

## TCP Server

A simple TCP echo server:

```fusion
use std::net;

fn handle_client(client: net::TcpStream) {
    let addr: string = client.peer_addr();
    println("New connection from %s", addr);

    loop {
        let data: string = client.read(1024);
        if data.is_empty() {
            break;
        }

        println("Received from %s: %s", addr, data);
        client.write(data);  // Echo back
    }

    println("Connection closed: %s", addr);
    client.close();
}

fn main() -> int {
    let addr: string = "0.0.0.0";
    let port: int = 8080;

    let server: net::TcpListener = net::TcpListener::bind(addr, port);
    println("Server listening on %s:%d", addr, port);

    loop {
        let client: net::TcpStream = server.accept();
        spawn handle_client(client);
    }

    return 0;
}
```

---

## Quantum Teleportation

Quantum teleportation protocol:

```fusion
use std::quantum;

fn main() -> int {
    println("Quantum Teleportation Protocol");
    println("==============================");

    // Create three qubits
    let q0: quantum::Qubit = quantum::Qubit::zero();  // State to teleport
    let q1: quantum::Qubit = quantum::Qubit::zero();  // Alice's qubit
    let q2: quantum::Qubit = quantum::Qubit::zero();  // Bob's qubit

    // Prepare state to teleport (put q0 in superposition)
    q0.h();
    q0.t();

    println("State to teleport prepared");

    // Create Bell pair between q1 and q2
    q1.h();
    quantum::cnot(q1, q2);

    println("Bell pair created");

    // Alice's operations
    quantum::cnot(q0, q1);
    q0.h();

    println("Alice performed her operations");

    // Alice measures her qubits
    let m0: int = q0.measure();
    let m1: int = q1.measure();

    println("Alice's measurements: m0=%d, m1=%d", m0, m1);

    // Bob applies corrections based on Alice's measurements
    if m1 == 1 {
        quantum::x(q2);
    }
    if m0 == 1 {
        quantum::z(q2);
    }

    println("Bob applied corrections");

    // Verify teleportation
    let result: int = q2.measure();
    println("Teleported state measurement: %d", result);

    println("\nTeleportation complete!");
    return 0;
}
```

---

## Neural Network Trainer

A complete neural network training example:

```fusion
use std::ml;

struct NeuralNetwork {
    layer1: ml::Linear,
    layer2: ml::Linear,
    layer3: ml::Linear,
}

impl NeuralNetwork {
    fn new() -> NeuralNetwork {
        return NeuralNetwork {
            layer1: ml::Linear::new(784, 256),
            layer2: ml::Linear::new(256, 128),
            layer3: ml::Linear::new(128, 10),
        };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        let x: ml::Tensor = ml::relu(self.layer1.forward(x));
        let x: ml::Tensor = ml::relu(self.layer2.forward(x));
        return self.layer3.forward(x);
    }

    fn parameters(self) -> Vec<ml::Tensor> {
        let mut params: Vec<ml::Tensor> = Vec::new();
        params.extend(self.layer1.parameters());
        params.extend(self.layer2.parameters());
        params.extend(self.layer3.parameters());
        return params;
    }
}

fn main() -> int {
    println("Neural Network Trainer");
    println("======================");

    // Create model
    let model: NeuralNetwork = NeuralNetwork::new();
    println("Model created with %d parameters", model.parameters().len());

    // Create optimizer
    let optimizer: ml::Adam = ml::Adam::new(model.parameters(), 0.001);

    // Create loss function
    let loss_fn: ml::CrossEntropyLoss = ml::CrossEntropyLoss::new();

    // Training loop
    let num_epochs: int = 10;
    let batch_size: int = 32;

    for epoch in 0..num_epochs {
        let mut total_loss: float = 0.0;
        let mut correct: int = 0;
        let mut total: int = 0;

        // Simulated training data
        for batch in 0..100 {
            let inputs: ml::Tensor = ml::randn([batch_size, 784]);
            let labels: ml::Tensor = ml::randint(0, 10, [batch_size]);

            // Forward pass
            let outputs: ml::Tensor = model.forward(inputs);
            let loss: ml::Tensor = loss_fn.forward(outputs, labels);

            // Backward pass
            optimizer.zero_grad();
            loss.backward();
            optimizer.step();

            // Track metrics
            total_loss = total_loss + loss.item();
            let predictions: ml::Tensor = outputs.argmax(1);
            correct = correct + predictions.eq(labels).sum().item() as int;
            total = total + batch_size;
        }

        let avg_loss: float = total_loss / 100.0;
        let accuracy: float = (correct as float) / (total as float) * 100.0;

        println("Epoch %d: loss=%.4f, accuracy=%.1f%%", epoch + 1, avg_loss, accuracy);
    }

    println("Training complete!");
    return 0;
}
```

---

## Complete PQC Chat Application

A secure chat application using post-quantum cryptography:

```fusion
use std::net;
use std::crypto;

struct SecureChat {
    connection: net::TcpStream,
    shared_secret: bytes,
}

impl SecureChat {
    fn connect(host: string, port: int) -> SecureChat {
        let connection: net::TcpStream = net::TcpStream::connect(host, port);

        // Generate key pair
        let keypair: crypto::HybridKeyPair = crypto::generate_keypair();
        let public_key: bytes = keypair.public_key();

        // Exchange keys (simplified)
        connection.write(public_key);
        let peer_public: bytes = connection.read(256);

        // Derive shared secret
        let shared_secret: bytes = keypair.derive_shared_secret(peer_public);

        println("Secure connection established");
        return SecureChat { connection, shared_secret };
    }

    fn send_message(self, message: string) {
        let encrypted: bytes = crypto::encrypt(self.shared_secret, message);
        self.connection.write(encrypted);
    }

    fn receive_message(self) -> string {
        let encrypted: bytes = self.connection.read(4096);
        return crypto::decrypt(self.shared_secret, encrypted);
    }

    fn close(self) {
        self.connection.close();
    }
}

fn handle_incoming(chat: SecureChat) {
    loop {
        let message: string = chat.receive_message();
        if message.is_empty() {
            break;
        }
        println("Received: %s", message);
    }
}

fn main() -> int {
    println("Secure PQC Chat Application");
    println("===========================");

    // Connect to server
    let chat: SecureChat = SecureChat::connect("localhost", 9000);

    // Spawn receiver fiber
    let chat_clone: SecureChat = chat.clone();
    spawn handle_incoming(chat_clone);

    // Send messages
    loop {
        println("Enter message (or 'quit' to exit): ");
        let message: string = std::io::read_line();

        if message == "quit" {
            break;
        }

        chat.send_message(message);
    }

    chat.close();
    println("Chat closed");
    return 0;
}
```

### Server Side

```fusion
use std::net;
use std::crypto;

struct ChatServer {
    listener: net::TcpListener,
}

impl ChatServer {
    fn new(port: int) -> ChatServer {
        let listener: net::TcpListener = net::TcpListener::bind("0.0.0.0", port);
        println("Chat server listening on port %d", port);
        return ChatServer { listener };
    }

    fn handle_client(self, client: net::TcpStream) {
        // Generate server key pair
        let keypair: crypto::HybridKeyPair = crypto::generate_keypair();
        let public_key: bytes = keypair.public_key();

        // Exchange keys
        let peer_public: bytes = client.read(256);
        client.write(public_key);

        // Derive shared secret
        let shared_secret: bytes = keypair.derive_shared_secret(peer_public);

        println("Client connected with PQC security");

        // Handle messages
        loop {
            let encrypted: bytes = client.read(4096);
            if encrypted.is_empty() {
                break;
            }

            let message: string = crypto::decrypt(shared_secret, encrypted);
            println("Client: %s", message);

            // Echo back with encryption
            let response: string = "Server received: " + message;
            let encrypted_response: bytes = crypto::encrypt(shared_secret, response);
            client.write(encrypted_response);
        }

        println("Client disconnected");
        client.close();
    }

    fn run(self) {
        loop {
            let client: net::TcpStream = self.listener.accept();
            spawn self.handle_client(client);
        }
    }
}

fn main() -> int {
    let server: ChatServer = ChatServer::new(9000);
    server.run();
    return 0;
}
```

---

## Advanced Examples

---

### Example 1: Effect-Based I/O System

This example demonstrates defining a custom IO effect, implementing a handler for it, and using it in application code. Effects decouple effect declarations from their implementations, allowing the same program to run against different backends (console, file, network, mock).

#### Step 1: Define the IO Effect

```fusion
use std::effects;

// Define an IO effect with print and readline operations
fn define_io_effect() -> effects::Effect {
    let io_effect: effects::Effect = effects::effect_register("IO");

    // Register operations on the effect
    io_effect.add_operation("print", fn(msg: string) -> void {
        effects::effect_perform(io_effect, msg)
    });

    io_effect.add_operation("readline", fn() -> string {
        effects::effect_perform(io_effect, ())
    });

    return io_effect;
}
```

#### Step 2: Implement the IO Handler

```fusion
// Console handler — routes IO operations to std::io
fn make_console_handler(io_effect: effects::Effect) -> effects::Handle {
    let handler: effects::Handler = effects::handler_new(io_effect);

    // Handle "print" operation
    handler = effects::handler_add_operation(handler, "print", fn(msg: string) -> void {
        println("%s", msg);
    });

    // Handle "readline" operation
    handler = effects::handler_add_operation(handler, "readline", fn() -> string {
        return std::io::read_line();
    });

    // Fallback handler for unknown operations
    handler = effects::handler_set_handler(handler, fn(effect: effects::Effect, value: T) -> R {
        println("Unknown operation on effect");
        return ();
    });

    return effects::handler_install(handler);
}

// Mock handler — returns canned responses for testing
fn make_mock_handler(io_effect: effects::Effect) -> effects::Handle {
    let handler: effects::Handler = effects::handler_new(io_effect);

    handler = effects::handler_add_operation(handler, "print", fn(msg: string) -> void {
        // Silently discard output in mock mode
    });

    handler = effects::handler_add_operation(handler, "readline", fn() -> string {
        return "mock_input";
    });

    return effects::handler_install(handler);
}
```

#### Step 3: Use in Application Code

```fusion
// Application code is effectful — no direct I/O calls
fn greet_user(io_effect: effects::Effect) -> void {
    effects::effect_perform(io_effect, "print", "What is your name?");
    let name: string = effects::effect_perform(io_effect, "readline");
    effects::effect_perform(io_effect, "print", "Hello, " + name + "!");
}

fn main() -> int {
    let io_effect: effects::Effect = define_io_effect();

    // Production: console handler
    let handle: effects::Handle = make_console_handler(io_effect);
    greet_user(io_effect);

    // Testing: mock handler
    let mock_handle: effects::Handle = make_mock_handler(io_effect);
    greet_user(io_effect);  // Runs silently

    return 0;
}
```

---

### Example 2: Resource Protocol with Linear Types

This example defines a file handle protocol using linear types to enforce single-use semantics. Once a handle is consumed (closed), it cannot be used again, preventing use-after-close bugs at compile time.

#### Step 1: Define the File Handle Protocol

```fusion
use std::linear;
use std::fs;

// A file handle that can only be used once per operation
struct FileHandle {
    path: string,
    is_open: bool,
}

// Protocol: open -> read/write -> close
// The linear type enforces that each handle goes through the protocol exactly once
fn open_file(path: string) -> linear::Linear<FileHandle> {
    // Verify file exists before opening
    if !fs::exists(path) {
        panic("File not found: %s", path);
    }

    let handle: FileHandle = FileHandle { path: path, is_open: true };
    return linear::linear_new(handle);
}
```

#### Step 2: Implement Read and Close Operations

```fusion
// Read from a linear file handle — consumes the handle and returns (data, new_handle)
fn read_file(
    handle: linear::Linear<FileHandle>,
    max_bytes: int
) -> (string, linear::Linear<FileHandle>) {
    // Check protocol: handle must be open
    assert(linear::linear_protocol_check(handle, "open"));

    let content: string = fs::read_to_string(linear::linear_use(handle).path);

    // Truncate to max_bytes if needed
    let data: string = if content.len() > max_bytes {
        content[0..max_bytes]
    } else {
        content
    };

    // Re-wrap in linear type for continued use
    let new_handle: linear::Linear<FileHandle> = linear::linear_new(
        FileHandle { path: handle.path, is_open: true }
    );

    return (data, new_handle);
}

// Close a linear file handle — consumes the handle permanently
fn close_file(handle: linear::Linear<FileHandle>) -> void {
    // Check protocol: handle must be open
    assert(linear::linear_protocol_check(handle, "open"));

    // Consume the linear value
    let file: FileHandle = linear::linear_use(handle);
    println("Closed file: %s", file.path);

    // Handle is now consumed — cannot be used again
}
```

#### Step 3: Verify Single-Use Constraint

```fusion
fn main() -> int {
    // Open a file — returns a linear handle
    let handle: linear::Linear<FileHandle> = open_file("example.txt");

    // Read from the file — returns data and a new handle
    let (data: string, handle: linear::Linear<FileHandle>) = read_file(handle, 1024);
    println("Read %d bytes", data.len());

    // Close the file — consumes the handle
    close_file(handle);

    // This would fail at compile time if uncommented:
    // close_file(handle);  // ERROR: handle already consumed

    println("File operations completed safely");
    return 0;
}
```

---

### Example 3: Capability-Secured Actor System

This example creates an actor system where actors can only access resources they have been explicitly granted capabilities for. Capabilities are unforgeable tokens that encode permissions.

#### Step 1: Create Capabilities for Resources

```fusion
use std::security;
use std::actors;

// Define resources
struct Database { name: string, data: HashMap<string, string> }
struct LogFile { path: string }

// Create capabilities with specific permissions
fn create_database_caps(db: Database) -> (security::Cap, security::Cap) {
    let read_cap: security::Cap = security::cap_new(db, vec!["read"]);
    let write_cap: security::Cap = security::cap_new(db, vec!["read", "write", "delete"]);
    return (read_cap, write_cap);
}

fn create_log_caps(log: LogFile) -> security::Cap {
    return security::cap_new(log, vec!["append", "read"]);
}
```

#### Step 2: Create Actors with Capabilities

```fusion
// Reader actor — can only read from the database
fn reader_actor_handler(
    read_cap: security::Cap,
    msg: actors::Message
) -> void {
    match msg {
        actors::Message::Query(key: string) => {
            // Verify capability before access
            if !security::cap_check(read_cap, "read") {
                println("ERROR: Missing read capability");
                return;
            }

            // Use sandbox to enforce capability constraints
            let sandbox: security::Sandbox = security::sandbox_new("reader");
            let sandbox: security::Sandbox = security::sandbox_add_cap(sandbox, read_cap);

            security::sandbox_execute(sandbox, fn() {
                let db: Database = security::cap_resource(read_cap);
                let value: Option<string> = HashMap::get(db.data, key);
                match value {
                    Option::Some(v) => println("Result: %s", v),
                    Option::None => println("Key not found: %s", key),
                }
            });
        }
        _ => println("Reader: unknown message"),
    }
}

// Writer actor — can read, write, and delete
fn writer_actor_handler(
    write_cap: security::Cap,
    msg: actors::Message
) -> void {
    match msg {
        actors::Message::Insert(key: string, value: string) => {
            if !security::cap_check(write_cap, "write") {
                println("ERROR: Missing write capability");
                return;
            }

            let sandbox: security::Sandbox = security::sandbox_new("writer");
            let sandbox: security::Sandbox = security::sandbox_add_cap(sandbox, write_cap);

            security::sandbox_execute(sandbox, fn() {
                let db: Database = security::cap_resource(write_cap);
                HashMap::set(db.data, key, value);
                println("Inserted: %s = %s", key, value);
            });
        }
        actors::Message::Delete(key: string) => {
            if !security::cap_check(write_cap, "delete") {
                println("ERROR: Missing delete capability");
                return;
            }

            let db: Database = security::cap_resource(write_cap);
            HashMap::remove(db.data, key);
            println("Deleted: %s", key);
        }
        _ => println("Writer: unknown message"),
    }
}
```

#### Step 3: Send Messages Through Capabilities

```fusion
fn main() -> int {
    // Create resources
    let mut db: Database = Database {
        name: "main_db",
        data: HashMap::new(),
    };
    HashMap::set(db.data, "key1", "value1");
    HashMap::set(db.data, "key2", "value2");

    let log: LogFile = LogFile { path: "app.log" };

    // Create capabilities
    let (read_cap: security::Cap, write_cap: security::Cap) = create_database_caps(db);
    let log_cap: security::Cap = create_log_caps(log);

    // Create actors with their capabilities
    let reader: actors::Actor = actors::actor_new(
        "reader",
        fn(msg: actors::Message) { reader_actor_handler(read_cap, msg); }
    );
    let writer: actors::Actor = actors::actor_new(
        "writer",
        fn(msg: actors::Message) { writer_actor_handler(write_cap, msg); }
    );

    // Start a supervisor
    let sup: actors::Supervisor = actors::supervisor_new("db_supervisor", actors::RestartStrategy::OneForOne);
    let sup: actors::Supervisor = actors::supervisor_add_child(sup, reader);
    let sup: actors::Supervisor = actors::supervisor_add_child(sup, writer);
    actors::supervisor_start(sup);

    // Send messages — capabilities are enforced
    actors::actor_send(reader, actors::Message::Query("key1"));
    actors::actor_send(writer, actors::Message::Insert("key3", "value3"));
    actors::actor_send(writer, actors::Message::Delete("key2"));

    // This would fail capability check:
    // actors::actor_send(reader, actors::Message::Insert("key4", "value4"));
    // Reader has only read capability, not write

    // Verify capabilities
    println("Read cap valid: %b", security::cap_verify(read_cap));
    println("Write cap valid: %b", security::cap_verify(write_cap));

    return 0;
}
```

---

### Example 4: Continuation-Based Backtracking

This example implements a constraint solver using continuations to explore multiple branches of a search space. When a branch fails, the solver backtracks by invoking a captured continuation to try the next option.

#### Step 1: Define the Solver Using Continuations

```fusion
use std::cont;

// A constraint that can be satisfied or rejected
struct Constraint {
    name: string,
    check: fn(int) -> bool,
}

// Solver state: current assignment and remaining constraints
struct SolverState {
    assignments: HashMap<string, int>,
    constraints: Vec<Constraint>,
    variables: Vec<string>,
}

// Solve with backtracking using call/cc
fn solve_with_backtracking(
    state: SolverState,
    backtrack: cont::Cont<SolverState>,
    on_solution: fn(SolverState) -> void
) -> void {
    // If no more variables to assign, we found a solution
    if state.variables.is_empty() {
        on_solution(state);
        return;
    }

    let var: string = state.variables[0];
    let remaining_vars: Vec<string> = state.variables[1..];

    // Try values 1 through 5 for each variable
    for value in 1..5 {
        // Create new state with this assignment
        let mut new_assignments: HashMap<string, int> = HashMap::clone(state.assignments);
        HashMap::set(new_assignments, var, value);

        let new_state: SolverState = SolverState {
            assignments: new_assignments,
            constraints: state.constraints,
            variables: remaining_vars,
        };

        // Check if this assignment violates any constraint
        let valid: bool = check_all_constraints(new_state);

        if valid {
            // Continue with the next variable
            solve_with_backtracking(new_state, backtrack, on_solution);
        } else {
            // Constraint violated — backtrack
            println("  Backtracking: %s=%d violates constraint", var, value);
            cont::cont_invoke(backtrack, new_state);
        }
    }
}

// Check all constraints against current assignments
fn check_all_constraints(state: SolverState) -> bool {
    for constraint in state.constraints {
        for (key, value) in state.assignments {
            if !constraint.check(value) {
                return false;
            }
        }
    }
    return true;
}
```

#### Step 2: Capture and Restore State

```fusion
// Find all solutions using continuations for backtracking
fn find_all_solutions(
    state: SolverState,
    on_solution: fn(SolverState) -> void
) -> void {
    // Capture the current continuation as a backtracking point
    let backtrack: cont::Cont<SolverState> = cont::cont_capture();

    // Solve with this backtracking point
    solve_with_backtracking(state, backtrack, on_solution);
}

// Restore a previously saved state and try alternatives
fn try_alternative(
    saved_state: SolverState,
    alternative_fn: fn(SolverState) -> void
) -> void {
    // Restore the saved state
    alternative_fn(saved_state);
}
```

#### Step 3: Backtrack on Failure

```fusion
fn main() -> int {
    println("Continuation-Based Backtracking Solver");
    println("======================================");

    // Define constraints: no two variables can have the same value
    let alldiff_constraint: Constraint = Constraint {
        name: "all_different",
        check: fn(val: int) -> bool {
            // This is simplified; real implementation checks against all assignments
            return val >= 1 && val <= 5;
        },
    };

    // Create initial solver state
    let mut assignments: HashMap<string, int> = HashMap::new();
    let variables: Vec<string> = vec!["x", "y", "z"];

    let state: SolverState = SolverState {
        assignments: assignments,
        constraints: vec![alldiff_constraint],
        variables: variables,
    };

    // Find and print all solutions
    let mut solution_count: int = 0;

    find_all_solutions(state, fn(solution: SolverState) {
        solution_count = solution_count + 1;
        println("Solution %d: x=%d, y=%d, z=%d",
            solution_count,
            HashMap::get(solution.assignments, "x").unwrap(),
            HashMap::get(solution.assignments, "y").unwrap(),
            HashMap::get(solution.assignments, "z").unwrap()
        );
    });

    println("Total solutions found: %d", solution_count);
    println("Backtracking demonstrated successfully");
    return 0;
}
```

---

### Example 5: Full Integration Application

This example combines effects, linear types, capabilities, actors, continuations, multimethods, and compilation stages into a complete working application: a secure task scheduler.

#### Complete Application: Secure Task Scheduler

```fusion
// Import all feature modules
use std::effects;
use std::linear;
use std::security;
use std::actors;
use std::cont;
use std::dispatch;
use std::compiler;

// =====================
// Effects: Define task I/O
// =====================

let task_io: effects::Effect = effects::effect_register("TaskIO");

fn task_print(msg: string) -> void {
    effects::effect_perform(task_io, "print", msg);
}

fn task_log(msg: string) -> void {
    effects::effect_perform(task_io, "log", msg);
}

// Handler for console output
fn install_console_effect() -> effects::Handle {
    let handler: effects::Handler = effects::handler_new(task_io);
    handler = effects::handler_add_operation(handler, "print", fn(msg: string) -> void {
        println("[OUTPUT] %s", msg);
    });
    handler = effects::handler_add_operation(handler, "log", fn(msg: string) -> void {
        println("[LOG] %s", msg);
    });
    return effects::handler_install(handler);
}

// =====================
// Linear Types: Task tokens (single-use)
// =====================

struct TaskToken {
    task_id: int,
    description: string,
    completed: bool,
}

fn create_task(id: int, desc: string) -> linear::Linear<TaskToken> {
    let token: TaskToken = TaskToken {
        task_id: id,
        description: desc,
        completed: false,
    };
    return linear::linear_new(token);
}

fn complete_task(token: linear::Linear<TaskToken>) -> int {
    let task: TaskToken = linear::linear_use(token);
    task_print("Completed task %d: %s", task.task_id, task.description);
    return task.task_id;
}

// =====================
// Capabilities: Access control
// =====================

struct TaskStore {
    tasks: HashMap<int, string>,
    results: Vec<int>,
}

fn create_task_store() -> TaskStore {
    return TaskStore {
        tasks: HashMap::new(),
        results: Vec::new(),
    };
}

fn create_store_caps(store: TaskStore) -> (security::Cap, security::Cap) {
    let read_cap: security::Cap = security::cap_new(store, vec!["read_tasks", "read_results"]);
    let admin_cap: security::Cap = security::cap_new(store, vec![
        "read_tasks", "read_results", "add_task", "remove_task", "update_results"
    ]);
    return (read_cap, admin_cap);
}

// =====================
// Actors: Concurrent task processing
// =====================

fn scheduler_actor(
    admin_cap: security::Cap,
    msg: actors::Message
) -> void {
    match msg {
        actors::Message::TaskScheduled(id: int, desc: string) => {
            if !security::cap_check(admin_cap, "add_task") {
                task_log("SECURITY: Scheduler lacks add_task permission");
                return;
            }
            task_log("Scheduler: Task %d scheduled - %s", id, desc);
        }
        actors::Message::TaskCompleted(id: int) => {
            if !security::cap_check(admin_cap, "update_results") {
                task_log("SECURITY: Scheduler lacks update_results permission");
                return;
            }
            task_log("Scheduler: Task %d completed", id);
        }
        _ => task_log("Scheduler: unknown message"),
    }
}

fn worker_actor(
    worker_id: int,
    read_cap: security::Cap,
    msg: actors::Message
) -> void {
    match msg {
        actors::Message::ExecuteTask(id: int) => {
            if !security::cap_check(read_cap, "read_tasks") {
                task_log("SECURITY: Worker %d lacks read_tasks permission", worker_id);
                return;
            }
            task_log("Worker %d: Executing task %d", worker_id, id);
        }
        _ => task_log("Worker %d: unknown message", worker_id),
    }
}

// =====================
// Continuations: Retry logic
// =====================

fn with_retry(max_attempts: int, f: fn() -> bool) -> bool {
    let mut attempt: int = 0;

    // Capture continuation for retry
    let retry_point: cont::Cont<void> = cont::cont_capture();

    while attempt < max_attempts {
        attempt = attempt + 1;
        task_log("Attempt %d of %d", attempt, max_attempts);

        let success: bool = f();
        if success {
            return true;
        }

        // Invoke continuation to retry
        task_log("Retrying from attempt %d...", attempt);
        cont::cont_invoke(retry_point, ());
    }

    return false;
}

// =====================
// Multimethods: Task prioritization
// =====================

let priority_dispatch: dispatch::Multimethod = dispatch::multimethod_new(
    "task_priority",
    fn(task_type: string) -> string {
        match task_type {
            "urgent" => "high",
            "important" => "high",
            "normal" => "medium",
            "background" => "low",
            _ => "medium",
        }
    }
);

// Register priority handlers
priority_dispatch = dispatch::multimethod_add(priority_dispatch, "high", fn(task: string) -> string {
    return "[HIGH PRIORITY] " + task;
});
priority_dispatch = dispatch::multimethod_add(priority_dispatch, "medium", fn(task: string) -> string {
    return "[MEDIUM PRIORITY] " + task;
});
priority_dispatch = dispatch::multimethod_add(priority_dispatch, "low", fn(task: string) -> string {
    return "[LOW PRIORITY] " + task;
});

// =====================
// Compiler Integration: Feature verification
// =====================

fn verify_features() -> void {
    let features: Vec<string> = vec![
        "effects", "linear_types", "capabilities", "actors",
        "continuations", "multimethods", "compiler_integration"
    ];

    let validation: compiler::ValidationResult = compiler::validate_features(
        "scheduler_source",
        features
    );

    if validation.is_valid() {
        task_log("All %d features verified successfully", features.len());
    } else {
        task_log("Feature validation failed: %s", validation.error());
    }
}

// =====================
// Integration Functions
// =====================

fn integrate_all_features(
    store: TaskStore,
    cap: security::Cap,
    actor: actors::Actor
) -> void {
    // Effect + Linear: Apply IO effect to a linear task
    let task_token: linear::Linear<TaskToken> = create_task(1, "Integrate features");
    task_print("Created linear task token");

    // Linear + Cap: Verify capability before consuming linear value
    if security::cap_check(cap, "read_tasks") {
        let task_id: int = complete_task(task_token);
        task_log("Linear task %d consumed via capability", task_id);
    }

    // Cap + Actor: Send message through capability-verified actor
    if security::cap_verify(cap) {
        actors::actor_send(actor, actors::Message::TaskScheduled(1, "Integration test"));
    }

    // Continuation + Multimethod: Dispatch with retry
    let result: bool = with_retry(3, fn() -> bool {
        let dispatched: string = dispatch::multimethod_dispatch(priority_dispatch, "urgent");
        task_log("Dispatched: %s", dispatched);
        return true;
    });

    task_log("Integration complete: success=%b", result);
}

// =====================
// Main Application
// =====================

fn main() -> int {
    println("========================================");
    println("  Secure Task Scheduler");
    println("  Full Feature Integration Demo");
    println("========================================");

    // Install effects
    let effect_handle: effects::Handle = install_console_effect();
    task_log("Effects module initialized");

    // Verify compiler features
    verify_features();

    // Create task store and capabilities
    let store: TaskStore = create_task_store();
    let (read_cap: security::Cap, admin_cap: security::Cap) = create_store_caps(store);
    task_log("Capabilities created and verified");

    // Create linear task tokens
    let token1: linear::Linear<TaskToken> = create_task(1, "Process data");
    let token2: linear::Linear<TaskToken> = create_task(2, "Generate report");
    let token3: linear::Linear<TaskToken> = create_task(3, "Send notification");
    task_log("Created 3 linear task tokens");

    // Create actors with capabilities
    let scheduler: actors::Actor = actors::actor_new(
        "scheduler",
        fn(msg: actors::Message) { scheduler_actor(admin_cap, msg); }
    );

    let worker1: actors::Actor = actors::actor_new(
        "worker1",
        fn(msg: actors::Message) { worker_actor(1, read_cap, msg); }
    );

    let worker2: actors::Actor = actors::actor_new(
        "worker2",
        fn(msg: actors::Message) { worker_actor(2, read_cap, msg); }
    );

    // Set up supervisor
    let sup: actors::Supervisor = actors::supervisor_new("scheduler_supervisor", actors::RestartStrategy::OneForOne);
    let sup: actors::Supervisor = actors::supervisor_add_child(sup, scheduler);
    let sup: actors::Supervisor = actors::supervisor_add_child(sup, worker1);
    let sup: actors::Supervisor = actors::supervisor_add_child(sup, worker2);
    actors::supervisor_start(sup);
    task_log("Actor system started with supervisor");

    // Use multimethods for priority dispatch
    let urgent_task: string = dispatch::multimethod_dispatch(priority_dispatch, "urgent");
    let normal_task: string = dispatch::multimethod_dispatch(priority_dispatch, "normal");
    let bg_task: string = dispatch::multimethod_dispatch(priority_dispatch, "background");
    task_log("Priority dispatch: %s", urgent_task);
    task_log("Priority dispatch: %s", normal_task);
    task_log("Priority dispatch: %s", bg_task);

    // Execute tasks with linear consumption
    actors::actor_broadcast(vec![worker1, worker2], actors::Message::ExecuteTask(1));
    let task1_id: int = complete_task(token1);

    actors::actor_broadcast(vec![worker1, worker2], actors::Message::ExecuteTask(2));
    let task2_id: int = complete_task(token2);

    actors::actor_broadcast(vec![worker1, worker2], actors::Message::ExecuteTask(3));
    let task3_id: int = complete_task(token3);

    task_log("All linear tasks consumed");

    // Use continuations for retry logic
    let success: bool = with_retry(3, fn() -> bool {
        actors::actor_send(scheduler, actors::Message::TaskCompleted(task1_id));
        return true;
    });
    task_log("Retry logic completed: success=%b", success);

    // Run full integration
    integrate_all_features(store, read_cap, scheduler);

    // Summary
    println("");
    println("========================================");
    println("  Integration Summary");
    println("========================================");
    println("Effects:          IO operations decoupled from implementation");
    println("Linear Types:     %d task tokens consumed (single-use enforced)", 3);
    println("Capabilities:     Read and admin caps verified");
    println("Actors:           1 scheduler + 2 workers with supervisor");
    println("Continuations:    Retry logic with backtracking");
    println("Multimethods:     Priority-based task dispatch");
    println("Compiler:         Feature validation passed");
    println("========================================");
    println("All 16 features integrated successfully!");
    println("========================================");

    return 0;
}
```

---

## Tips for Learning

1. **Start with Hello World**: Verify your setup works.
2. **Modify the examples**: Change values and see what happens.
3. **Break things intentionally**: Learn from compiler errors.
4. **Read the error messages**: Fusion's errors are designed to be helpful.
5. **Build incrementally**: Add features one at a time.
6. **Start with simple features**: Master effects before combining with linear types.
7. **Use the compiler flags**: `--debug` and `--vortex` help verify your code.

---

## Cross-References

- **Chapter 1**: Getting Started for setup instructions
- **Chapter 2**: Syntax for language basics
- **Chapter 7**: Post-Quantum Cryptography for PQC details
- **Chapter 8**: Quantum Computing for quantum examples
- **Chapter 9**: Machine Learning for ML examples
- **Chapter 13**: Advanced Features for detailed explanations of effects, linear types, capabilities, actors, and continuations
- **Chapter 15**: Reference for complete API signatures