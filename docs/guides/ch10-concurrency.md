# Chapter 10: Concurrency

> Fibers, message passing, shared state, cluster computing, and process supervision

---

## Fibers and Cooperative Scheduling

Fibers are lightweight, cooperative threads that run within a single OS thread. They are scheduled cooperatively — a fiber must explicitly yield control.

### Basic Fibers

```fusion
use std::async;

fn main() -> int {
    // Spawn a fiber
    let fiber: async::Fiber = spawn {
        println("Fiber 1 started");
        async::yield();
        println("Fiber 1 resumed");
    };

    println("Main fiber");

    // Wait for fiber to complete
    fiber.join();

    return 0;
}
```

### Multiple Fibers

```fusion
use std::async;

fn worker(id: int) {
    for i in 0..5 {
        println("Worker %d: step %d", id, i);
        async::yield();
    }
    println("Worker %d: done", id);
}

fn main() -> int {
    // Spawn multiple fibers
    let fibers: [async::Fiber] = [
        spawn worker(1),
        spawn worker(2),
        spawn worker(3),
    ];

    // Wait for all fibers
    for fiber in fibers {
        fiber.join();
    }

    println("All workers completed");
    return 0;
}
```

### Fiber Communication

```fusion
use std::async;

fn producer(chan: async::Sender<int>) {
    for i in 0..10 {
        chan.send(i);
        println("Produced: %d", i);
    }
}

fn consumer(chan: async::Receiver<int>) {
    loop {
        let value: Option<int> = chan.recv();
        match value {
            Some(v) => println("Consumed: %d", v),
            None => break,
        }
    }
}

fn main() -> int {
    let (sender, receiver): (async::Sender<int>, async::Receiver<int>) = async::channel();

    let prod: async::Fiber = spawn producer(sender);
    let cons: async::Fiber = spawn consumer(receiver);

    prod.join();
    cons.join();

    return 0;
}
```

---

## Message Passing

### Channel-Based Communication

```fusion
use std::async;

fn sender(chan: async::Sender<string>) {
    let messages: [string; 3] = ["Hello", "World", "Fusion"];
    for msg in messages {
        chan.send(msg);
        println("Sent: %s", msg);
    }
}

fn receiver(chan: async::Receiver<string>) {
    loop {
        let msg: Option<string> = chan.recv();
        match msg {
            Some(m) => println("Received: %s", m),
            None => break,
        }
    }
}

fn main() -> int {
    let (tx, rx): (async::Sender<string>, async::Receiver<string>) = async::channel();

    let s: async::Fiber = spawn sender(tx);
    let r: async::Fiber = spawn receiver(rx);

    s.join();
    r.join();

    return 0;
}
```

### Broadcast Channel

```fusion
use std::async;

fn publisher(broadcaster: async::Broadcaster<string>) {
    for i in 0..5 {
        let msg: string = "Message " + i.to_string();
        broadcaster.broadcast(msg);
    }
}

fn subscriber(id: int, receiver: async::BroadcastReceiver<string>) {
    loop {
        let msg: Option<string> = receiver.recv();
        match msg {
            Some(m) => println("Subscriber %d: %s", id, m),
            None => break,
        }
    }
}

fn main() -> int {
    let (broadcaster, receivers): (async::Broadcaster<string>, Vec<async::BroadcastReceiver<string>>) = async::broadcast_channel();

    let pub_fiber: async::Fiber = spawn publisher(broadcaster);

    let mut subs: [async::Fiber] = [];
    for i in 0..3 {
        let rx: async::BroadcastReceiver<string> = receivers[i];
        subs.push(spawn subscriber(i, rx));
    }

    pub_fiber.join();
    for sub in subs {
        sub.join();
    }

    return 0;
}
```

---

## Shared State (Arc, Mutex)

### Mutex for Mutual Exclusion

```fusion
use std::sync;

fn main() -> int {
    // Shared counter with mutex
    let counter: sync::Mutex<int> = sync::Mutex::new(0);

    let fibers: [async::Fiber] = [];
    for i in 0..10 {
        let c: sync::Mutex<int> = counter.clone();
        fibers.push(spawn {
            for _ in 0..100 {
                let mut val: sync::MutexGuard<int> = c.lock();
                *val = *val + 1;
            }
        });
    }

    for fiber in fibers {
        fiber.join();
    }

    let final_count: int = counter.lock();
    println("Final count: %d", final_count);  // 1000

    return 0;
}
```

### Atomic Operations

```fusion
use std::sync;

fn main() -> int {
    // Atomic counter (no mutex needed)
    let counter: sync::AtomicInt = sync::AtomicInt::new(0);

    let fibers: [async::Fiber] = [];
    for i in 0..10 {
        let c: sync::AtomicInt = counter.clone();
        fibers.push(spawn {
            for _ in 0..100 {
                c.fetch_add(1);
            }
        });
    }

    for fiber in fibers {
        fiber.join();
    }

    println("Atomic count: %d", counter.load());  // 1000

    return 0;
}
```

### Read-Write Lock

```fusion
use std::sync;

fn main() -> int {
    let data: sync::RwLock<Vec<int>> = sync::RwLock::new(Vec::new());

    // Multiple readers
    let readers: [async::Fiber] = [];
    for i in 0..5 {
        let d: sync::RwLock<Vec<int>> = data.clone();
        readers.push(spawn {
            let guard: sync::RwLockReadGuard<Vec<int>> = d.read();
            println("Reader %d: length = %d", i, guard.len());
        });
    }

    // Single writer
    let writer: async::Fiber = spawn {
        let mut guard: sync::RwLockWriteGuard<Vec<int>> = data.write();
        guard.push(42);
        println("Writer: added element");
    };

    for r in readers {
        r.join();
    }
    writer.join();

    return 0;
}
```

---

## Cluster Computing

### Nexus (Distributed Task System)

```fusion
use std::cluster;

fn main() -> int {
    // Connect to cluster
    let nexus: cluster::Nexus = cluster::Nexus::connect("nexus://localhost:7890");

    // Submit tasks to the cluster
    let task1: cluster::TaskHandle = nexus.submit(|| {
        // This runs on a cluster node
        return compute_heavy(1);
    });

    let task2: cluster::TaskHandle = nexus.submit(|| {
        return compute_heavy(2);
    });

    // Wait for results
    let result1: int = task1.await();
    let result2: int = task2.await();

    println("Task 1 result: %d", result1);
    println("Task 2 result: %d", result2);

    nexus.disconnect();
    return 0;
}
```

### TensorWeave (Distributed ML Training)

```fusion
use std::cluster;

fn main() -> int {
    // Initialize distributed training
    let weave: cluster::TensorWeave = cluster::TensorWeave::init();

    println("Node %d of %d", weave.rank(), weave.world_size());

    // Distribute data across nodes
    let local_data: ml::Tensor = weave.scatter(global_data, 0);

    // Local training step
    let local_loss: float = train_step(local_data);

    // All-reduce gradients
    let global_loss: float = weave.all_reduce(local_loss, cluster::ReduceOp::Mean);

    println("Node %d: loss = %f", weave.rank(), global_loss);

    weave.finalize();
    return 0;
}
```

### Map-Reduce Pattern

```fusion
use std::cluster;

fn main() -> int {
    let nexus: cluster::Nexus = cluster::Nexus::connect("nexus://localhost:7890");

    // Map phase: distribute work
    let tasks: [cluster::TaskHandle] = [];
    for i in 0..100 {
        let task: cluster::TaskHandle = nexus.submit(move || {
            return process_chunk(i);
        });
        tasks.push(task);
    }

    // Collect results
    let mut results: Vec<int> = Vec::new();
    for task in tasks {
        results.push(task.await());
    }

    // Reduce phase
    let total: int = results.fold(0, |acc, x| acc + x);
    println("Total: %d", total);

    nexus.disconnect();
    return 0;
}
```

---

## Warden Process Supervision

Warden monitors and restarts failed fibers/processes.

### Basic Supervision

```fusion
use std::warden;

fn main() -> int {
    // Create a supervisor
    let supervisor: warden::Supervisor = warden::Supervisor::new();

    // Register a supervised fiber
    supervisor.supervise(|| {
        loop {
            println("Worker running");
            // Simulate work
            async::sleep(1000);

            // Simulate failure
            if random() < 0.1 {
                panic!("Worker failed!");
            }
        }
    });

    // Configure restart strategy
    supervisor.set_strategy(warden::Strategy {
        max_restarts: 5,
        restart_window: 60,  // seconds
        backoff: 1.0,        // seconds
    });

    // Start supervision
    supervisor.start();

    // Supervisor will automatically restart failed fibers
    println("Supervisor running");

    // Run forever
    loop {
        async::sleep(10000);
    }

    return 0;
}
```

### Supervision Tree

```fusion
use std::warden;

fn main() -> int {
    // Create supervision tree
    let root: warden::Supervisor = warden::Supervisor::new();

    // Database supervisor
    let db_sup: warden::Supervisor = root.child_supervisor("database");
    db_sup.supervise(|| start_database());

    // API supervisor
    let api_sup: warden::Supervisor = root.child_supervisor("api");
    api_sup.supervise(|| start_api_server());

    // Worker supervisor
    let worker_sup: warden::Supervisor = root.child_supervisor("workers");
    for i in 0..4 {
        worker_sup.supervise(move || start_worker(i));
    }

    // Start the entire tree
    root.start();

    println("Application started with supervision");
    loop {
        async::sleep(10000);
    }

    return 0;
}
```

---

## Common Patterns

### Fan-Out/Fan-In

```fusion
use std::async;

fn process(item: int) -> int {
    return item * 2;
}

fn main() -> int {
    let input: [int; 100] = [0; 100];  // Simplified

    // Fan-out: distribute work
    let senders: Vec<async::Sender<int>> = [];
    let receivers: Vec<async::Receiver<int>> = [];

    for _ in 0..4 {
        let (tx, rx): (async::Sender<int>, async::Receiver<int>) = async::channel();
        senders.push(tx);
        receivers.push(rx);
    }

    // Spawn workers
    let workers: [async::Fiber] = [];
    for i in 0..4 {
        let rx: async::Receiver<int> = receivers[i].clone();
        workers.push(spawn {
            loop {
                let item: Option<int> = rx.recv();
                match item {
                    Some(v) => {
                        let result: int = process(v);
                        // Send result to aggregator
                    }
                    None => break,
                }
            }
        });
    }

    // Distribute input
    for (i, item) in input.iter().enumerate() {
        senders[i %% 4].send(*item);
    }

    // Close channels
    for tx in senders {
        tx.close();
    }

    // Wait for workers
    for worker in workers {
        worker.join();
    }

    println("Fan-out/fan-in complete");
    return 0;
}
```

### Rate Limiting

```fusion
use std::async;

fn main() -> int {
    let limiter: async::RateLimiter = async::RateLimiter::new(100);  // 100 requests/second

    let fibers: [async::Fiber] = [];
    for i in 0..10 {
        fibers.push(spawn {
            for j in 0..10 {
                limiter.acquire();  // Wait for rate limit
                println("Request %d-%d", i, j);
                // Make request
            }
        });
    }

    for fiber in fibers {
        fiber.join();
    }

    return 0;
}
```

---

## Async/Await

Async/await provides a convenient syntax for asynchronous operations.

### Basic Async Functions

```fusion
use std::async;

async fn fetch_data(url: string) -> string {
    // Simulate async HTTP request
    let response: string = async::http_get(url).await;
    return response;
}

async fn process_all() {
    // Sequential
    let data1: string = fetch_data("https://api.example.com/1").await;
    let data2: string = fetch_data("https://api.example.com/2").await;

    // Parallel with join
    let (d1, d2): (string, string) = async::join!(
        fetch_data("https://api.example.com/1"),
        fetch_data("https://api.example.com/2")
    );

    println("Got: %s, %s", d1, d2);
}

fn main() -> int {
    async::run(process_all());
    return 0;
}
```

### Async Patterns

```fusion
use std::async;

async fn retry<T>(f: fn() -> async T, max_retries: int) -> T {
    let mut attempts: int = 0;
    loop {
        match f().await {
            Ok(result) => return result,
            Err(e) => {
                attempts = attempts + 1;
                if attempts >= max_retries {
                    panic!("Max retries exceeded: %s", e);
                }
                async::sleep(1000 * attempts).await;
            }
        }
    }
}

async fn fetch_with_retry() -> string {
    return retry(|| fetch_data("https://api.example.com"), 3).await;
}
```

---

## Supernova Runtime

The Supernova runtime manages execution across CPU, GPU, and QPU (Quantum Processing Unit) resources.

### Runtime Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Supernova Runtime                      │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   CPU Pool  │  │   GPU Pool  │  │   QPU Pool  │     │
│  │  (Threads)  │  │ (CUDA/HIP)  │  │ (Quantum)   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
├─────────────────────────────────────────────────────────┤
│              Task Scheduler & Work Stealer               │
├─────────────────────────────────────────────────────────┤
│              Memory Manager & Zero-Copy                  │
└─────────────────────────────────────────────────────────┘
```

### CPU/GPU/QPU Dispatch

```fusion
use std::runtime;

fn main() -> int {
    // Initialize Supernova runtime
    let rt: runtime::Supernova = runtime::Supernova::init();

    // CPU-bound task
    let cpu_result: int = rt.spawn_cpu(|| {
        return heavy_computation(1000000);
    }).await();

    // GPU-accelerated task
    let gpu_result: Vec<float> = rt.spawn_gpu(|| {
        // Runs on GPU via CUDA/HIP
        let data: Vec<float> = vec![1.0, 2.0, 3.0, 4.0];
        return data.map(|x| x * x);
    }).await();

    // QPU task (quantum computing)
    let qpu_result: float = rt.spawn_qpu(|| {
        // Runs on quantum processor
        return quantum_simulation();
    }).await();

    println("CPU: %d, GPU: %f, QPU: %f", cpu_result, gpu_result[0], qpu_result);

    rt.shutdown();
    return 0;
}
```

### Automatic Hardware Selection

```fusion
use std::runtime;

// Supernova automatically selects the best hardware
#[runtime::auto_dispatch]
fn matrix_multiply(a: Matrix, b: Matrix) -> Matrix {
    // Runtime decides: CPU for small, GPU for large, QPU for quantum-ready
    return a * b;
}

fn main() -> int {
    let rt: runtime::Supernova = runtime::Supernova::init();

    // Small matrix - runs on CPU
    let small_a: Matrix = Matrix::new(10, 10);
    let small_b: Matrix = Matrix::new(10, 10);
    let result1: Matrix = matrix_multiply(small_a, small_b);

    // Large matrix - automatically dispatched to GPU
    let large_a: Matrix = Matrix::new(10000, 10000);
    let large_b: Matrix = Matrix::new(10000, 10000);
    let result2: Matrix = matrix_multiply(large_a, large_b);

    return 0;
}
```

---

## Cortex Scheduler

The Cortex scheduler uses AI-powered routing to optimize task execution.

### How Cortex Works

```
┌─────────────────────────────────────────────────────────┐
│                   Cortex Scheduler                       │
├─────────────────────────────────────────────────────────┤
│  1. Analyze task characteristics (CPU/GPU/QPU, memory)  │
│  2. Predict optimal placement using ML model            │
│  3. Balance load across available resources              │
│  4. Adapt scheduling based on runtime feedback           │
└─────────────────────────────────────────────────────────┘
```

### Intent-Driven Execution

```fusion
use std::cortex;

fn main() -> int {
    let scheduler: cortex::Scheduler = cortex::Scheduler::new();

    // Declare intent - Cortex optimizes execution
    let intent: cortex::Intent = cortex::Intent {
        priority: cortex::Priority::High,
        resource_hint: cortex::ResourceHint::Auto,
        latency_target: 100,  // ms
    };

    // Cortex routes to best resource
    let result: int = scheduler.execute(intent, || {
        return compute_heavy();
    }).await();

    // Intent with specific requirements
    let ml_intent: cortex::Intent = cortex::Intent {
        priority: cortex::Priority::Normal,
        resource_hint: cortex::ResourceHint::GPU,
        memory_hint: 4096,  // MB
        latency_target: 500,
    };

    let model_result: Model = scheduler.execute(ml_intent, || {
        return train_model();
    }).await();

    return 0;
}
```

### Dynamic Load Balancing

```fusion
use std::cortex;

fn main() -> int {
    let scheduler: cortex::Scheduler = cortex::Scheduler::new();

    // Cortex monitors and rebalances automatically
    scheduler.set_strategy(cortex::Strategy::Adaptive);

    // Submit many tasks - Cortex distributes intelligently
    let tasks: [cortex::TaskHandle] = [];
    for i in 0..1000 {
        let handle: cortex::TaskHandle = scheduler.submit(|| {
            return process_item(i);
        });
        tasks.push(handle);
    }

    // Cortex may move tasks between CPU/GPU at runtime
    // based on load and task characteristics

    for task in tasks {
        task.await();
    }

    println("All tasks completed");
    return 0;
}
```

---

## Configuration in Fusion.toml

### Runtime Configuration

```toml
[runtime.supernova]
# Hardware pools
cpu_threads = 8
gpu_enabled = true
gpu_device = "auto"  # or specific device ID
qpu_enabled = false
qpu_backend = "simulator"

# Memory management
memory_pool_size = "4GB"
zero_copy_enabled = true

# Work stealing
work_stealing_enabled = true
steal_interval_ms = 10
```

### Scheduler Configuration

```toml
[runtime.cortex]
# AI-powered scheduling
enabled = true
model_path = "models/scheduler.onnx"
learning_rate = 0.001

# Intent defaults
default_priority = "normal"
default_latency_target_ms = 100

# Load balancing
balancing_strategy = "adaptive"  # or "round-robin", "least-loaded"
rebalance_interval_ms = 1000
```

### Fiber Configuration

```toml
[runtime.fibers]
# Fiber pool settings
min_fibers = 100
max_fibers = 10000
stack_size_kb = 256

# Scheduling
preemption_enabled = false  # cooperative only
time_slice_ms = 10

# Channel settings
channel_buffer_size = 256
```

### Warden Supervision Configuration

```toml
[runtime.warden]
# Default supervision strategy
max_restarts = 5
restart_window_seconds = 60
backoff_seconds = 1.0
max_backoff_seconds = 30.0

# Logging
log_restart_events = true
log_level = "info"
```

---

## Complete Example: Concurrent Task Processing

```fusion
use std::async;
use std::sync;

struct TaskQueue {
    tasks: sync::Mutex<Vec<Task>>,
    completed: sync::Mutex<Vec<Result>>,
    worker_count: int,
}

struct Task {
    id: int,
    data: string,
    priority: int,
}

struct Result {
    task_id: int,
    output: string,
    duration_ms: int,
}

impl TaskQueue {
    fn new(worker_count: int) -> TaskQueue {
        return TaskQueue {
            tasks: sync::Mutex::new(Vec::new()),
            completed: sync::Mutex::new(Vec::new()),
            worker_count,
        };
    }

    fn add_task(self: &TaskQueue, task: Task) {
        let mut tasks: sync::MutexGuard<Vec<Task>> = self.tasks.lock();
        tasks.push(task);
    }

    fn process_task(task: Task) -> Result {
        let start: int = time::now();
        // Simulate processing
        let output: string = "Processed: " + task.data;
        let duration: int = time::now() - start;

        return Result {
            task_id: task.id,
            output,
            duration_ms: duration,
        };
    }

    fn run(self: &TaskQueue) {
        let fibers: [async::Fiber] = [];

        for worker_id in 0..self.worker_count {
            let queue: &TaskQueue = self;
            fibers.push(spawn {
                loop {
                    let task: Option<Task> = {
                        let mut tasks: sync::MutexGuard<Vec<Task>> = queue.tasks.lock();
                        tasks.pop()
                    };

                    match task {
                        Some(t) => {
                            let result: Result = TaskQueue::process_task(t);
                            let mut completed: sync::MutexGuard<Vec<Result>> = queue.completed.lock();
                            completed.push(result);
                        }
                        None => {
                            async::sleep(10).await;
                        }
                    }
                }
            });
        }

        for fiber in fibers {
            fiber.join();
        }
    }

    fn get_results(self: &TaskQueue) -> Vec<Result> {
        let completed: sync::MutexGuard<Vec<Result>> = self.completed.lock();
        return completed.clone();
    }
}

fn main() -> int {
    let queue: TaskQueue = TaskQueue::new(4);

    // Add tasks
    for i in 0..100 {
        queue.add_task(Task {
            id: i,
            data: "item_" + i.to_string(),
            priority: i %% 3,
        });
    }

    // Process all tasks concurrently
    queue.run();

    // Get results
    let results: Vec<Result> = queue.get_results();
    println("Processed %d tasks", results.len());

    return 0;
}
```

---

## Complete Example: GPU-Accelerated Computation

```fusion
use std::runtime;
use std::gpu;

struct Matrix {
    data: Vec<float>,
    rows: int,
    cols: int,
}

impl Matrix {
    fn new(rows: int, cols: int) -> Matrix {
        let size: int = rows * cols;
        let mut data: Vec<float> = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(0.0);
        }
        return Matrix { data, rows, cols };
    }

    fn from_data(rows: int, cols: int, data: Vec<float>) -> Matrix {
        return Matrix { data, rows, cols };
    }

    fn get(self: &Matrix, i: int, j: int) -> float {
        return self.data[i * self.cols + j];
    }

    fn set(self: &mut Matrix, i: int, j: int, val: float) {
        self.data[i * self.cols + j] = val;
    }
}

// GPU kernel for matrix multiplication
#[gpu::kernel]
fn matrix_multiply_kernel(
    a: gpu::Buffer<float>,
    b: gpu::Buffer<float>,
    c: gpu::Buffer<float>,
    n: int,
) {
    let row: int = gpu::thread_idx_y();
    let col: int = gpu::thread_idx_x();

    if row < n && col < n {
        let sum: float = 0.0;
        for k in 0..n {
            sum = sum + a[row * n + k] * b[k * n + col];
        }
        c[row * n + col] = sum;
    }
}

fn gpu_matrix_multiply(a: &Matrix, b: &Matrix) -> Matrix {
    let rt: runtime::Supernova = runtime::Supernova::current();
    let ctx: gpu::Context = rt.gpu_context();

    // Transfer data to GPU
    let gpu_a: gpu::Buffer<float> = ctx.buffer_from_slice(&a.data);
    let gpu_b: gpu::Buffer<float> = ctx.buffer_from_slice(&b.data);
    let gpu_c: gpu::Buffer<float> = ctx.buffer::<float>(a.rows * b.cols);

    // Configure kernel launch
    let block_size: int = 16;
    let grid_x: int = (b.cols + block_size - 1) / block_size;
    let grid_y: int = (a.rows + block_size - 1) / block_size;

    // Launch kernel
    ctx.launch(
        matrix_multiply_kernel,
        gpu::Grid::new(grid_x, grid_y),
        gpu::Block::new(block_size, block_size),
        (gpu_a, gpu_b, gpu_c, a.rows),
    );

    // Transfer result back to CPU
    let result_data: Vec<float> = ctx.buffer_to_vec(&gpu_c);
    return Matrix::from_data(a.rows, b.cols, result_data);
}

fn main() -> int {
    let rt: runtime::Supernova = runtime::Supernova::init();

    // Create matrices
    let n: int = 1024;
    let mut a: Matrix = Matrix::new(n, n);
    let mut b: Matrix = Matrix::new(n, n);

    // Initialize with random data
    for i in 0..n {
        for j in 0..n {
            a.set(i, j, (i * n + j) as float);
            b.set(i, j, (i + j) as float);
        }
    }

    // GPU-accelerated multiplication
    let result: Matrix = gpu_matrix_multiply(&a, &b);

    println("Result[0][0] = %f", result.get(0, 0));
    println("Result[%d][%d] = %f", n-1, n-1, result.get(n-1, n-1));

    rt.shutdown();
    return 0;
}
```

---

## Tips and Best Practices

1. **Prefer message passing**: It's safer than shared state.
2. **Use atomic operations**: When you need simple counters.
3. **Limit shared state**: Keep critical sections small.
4. **Use supervision**: For production systems, always use Warden.
5. **Test concurrent code**: Use stress tests to find race conditions.
6. **Leverage Supernova**: Let the runtime choose the best hardware.
7. **Use async/await**: For cleaner asynchronous code.
8. **Configure via Fusion.toml**: Keep runtime settings in config, not code.

---

## Cross-References

- **Chapter 4**: Memory Safety for safe concurrency primitives
- **Chapter 6**: Standard Library for async I/O
- **Chapter 12**: Tooling for concurrent testing tools
