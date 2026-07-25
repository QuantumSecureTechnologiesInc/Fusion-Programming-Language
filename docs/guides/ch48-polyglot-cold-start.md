# Chapter 48: Cold Start & Resource Optimization

Cold start latency is the silent killer of polyglot serverless systems. A Rust function starts in 1ms; a Python function takes 200ms; a Java function takes 2s. When your request fans out to five languages, the cold start of the slowest one becomes your bottleneck. This chapter covers cold start anatomy, comparison, and mitigation strategies.

## Understanding Cold Start

A cold start happens when a serverless function is invoked for the first time (or after being idle). The runtime must:

1. Download the code package
2. Start the runtime process
3. Initialize the language runtime (interpreter, JIT, GC)
4. Load dependencies
5. Execute initialization code
6. Handle the first request

Each language pays different costs at each stage.

## Cold Start Comparison by Language

```
Language    Runtime        Package Size   Init Time   First Request   Steady State
──────────────────────────────────────────────────────────────────────────────────
Rust        Native binary  ~2 MB          ~0ms        ~1ms            ~0.5ms
Go          Native binary  ~8 MB          ~0ms        ~2ms            ~0.3ms
Node.js     V8             ~30 MB         ~30ms       ~40ms           ~5ms
Python      CPython        ~50 MB         ~80ms       ~150ms          ~10ms
Java        JVM + GraalVM  ~80 MB         ~800ms      ~1200ms         ~2ms
C#          .NET NativeAOT ~40 MB         ~200ms      ~300ms          ~5ms
Fusion      Hybrid         ~15 MB         ~20ms       ~25ms           ~3ms
```

### Measuring Cold Start

```python
# cold_start_benchmark.py
import time
import json
import statistics

def measure_cold_start(runtime_command, iterations=20):
    """Measure cold start time for a given runtime."""
    times = []

    for i in range(iterations):
        start = time.perf_counter_ns()

        # Simulate cold start: spawn a new process
        import subprocess
        result = subprocess.run(
            runtime_command,
            capture_output=True,
            timeout=30,
        )

        end = time.perf_counter_ns()
        elapsed_ms = (end - start) / 1_000_000
        times.append(elapsed_ms)

    return {
        "min": min(times),
        "max": max(times),
        "median": statistics.median(times),
        "p95": sorted(times)[int(len(times) * 0.95)],
        "p99": sorted(times)[int(len(times) * 0.99)],
    }

# Benchmark results
benchmarks = {
    "rust": measure_cold_start(["target/release/fusion-server"]),
    "go": measure_cold_start(["go", "run", "cmd/server/main.go"]),
    "python": measure_cold_start(["python", "-c", "import fusion; fusion.serve()"]),
    "node": measure_cold_start(["node", "dist/server.js"]),
}

for lang, stats in benchmarks.items():
    print(f"{lang}: median={stats['median']:.1f}ms, p99={stats['p99']:.1f}ms")
```

## JIT Warm-Up Costs

Just-In-Time compilers (Java's HotSpot, .NET's RyuJIT, V8) must observe code before optimizing it. This warm-up cost is separate from cold start and affects sustained performance.

### Java/JVM Warm-Up

```java
// Warm-up phases in Java
public class WarmUpDemo {
    // Phase 1: Interpreter (slow)
    // - Method is interpreted by the JVM
    // - No optimization, no inlining
    // - 10-100x slower than optimized

    // Phase 2: C1 Compiler (moderate)
    // - JVM collects profiling data
    // - Methods compiled to native code with basic optimizations
    // - 2-5x slower than fully optimized

    // Phase 3: C2 Compiler (fast)
    // - Hot methods recompiled with aggressive optimizations
    // - Inlining, loop unrolling, escape analysis
    // - Full native performance

    public static void main(String[] args) {
        // Measure warm-up cost
        long[] times = new long[100];
        for (int i = 0; i < 100; i++) {
            long start = System.nanoTime();
            processRequest();  // Same request 100 times
            long end = System.nanoTime();
            times[i] = (end - start) / 1_000_000;
        }

        // Print warm-up curve
        for (int i = 0; i < 100; i += 10) {
            System.out.printf("Iteration %3d: %d ms%n", i, times[i]);
        }
    }

    // Typical output:
    // Iteration   0: 450 ms   (interpreter)
    // Iteration  10: 120 ms   (C1 compiled)
    // Iteration  20: 45 ms    (C2 compiled)
    // Iteration  30: 8 ms     (fully optimized)
    // Iteration  40: 7 ms     (stable)
}
```

### GraalVM Native Image (No JIT)

```bash
# GraalVM native-image eliminates JIT warm-up
# by compiling to native binary at build time

# Build native image
native-image \
  --initialize-at-build-time \
  --no-fallback \
  --enable-url-protocols=http \
  -jar fusion-server.jar \
  fusion-server

# Cold start: ~200ms (JVM init) vs ~1200ms (JIT)
# First request: ~200ms vs ~1200ms
# Steady state: ~3ms (slightly slower than JIT-optimized)
# Tradeoff: faster startup, slightly slower peak throughput
```

### Python Warm-Up (Import Time)

```python
# warmup_benchmark.py — Measure Python import costs
import time
import importlib

def measure_import_time(module_name):
    """Measure how long it takes to import a module."""
    start = time.perf_counter_ns()
    importlib.import_module(module_name)
    end = time.perf_counter_ns()
    return (end - start) / 1_000_000

# Common Fusion dependencies
dependencies = [
    "fusion.core",
    "fusion.user",
    "fusion.auth",
    "fusion.api",
    "fastapi",
    "pydantic",
    "uvicorn",
]

total = 0
for dep in dependencies:
    try:
        elapsed = measure_import_time(dep)
        total += elapsed
        print(f"  {dep}: {elapsed:.1f}ms")
    except ImportError:
        print(f"  {dep}: not installed")

print(f"Total import time: {total:.1f}ms")
# Typical output:
#   fusion.core: 15.2ms
#   fusion.user: 8.3ms
#   fusion.auth: 12.1ms
#   fusion.api: 18.7ms
#   fastapi: 25.4ms
#   pydantic: 18.9ms
#   uvicorn: 12.3ms
# Total import time: 110.9ms
```

## Memory Footprint Comparison

```
Language    Idle Memory   Per-Request   Peak Memory   GC Pause
──────────────────────────────────────────────────────────────────
Rust        ~1 MB         ~0 KB*        ~2 MB         None
Go          ~5 MB         ~0 KB*        ~8 MB         ~0.5ms
Node.js     ~30 MB        ~10 KB        ~50 MB        ~5ms
Python      ~20 MB        ~20 KB        ~40 MB        ~10ms
Java        ~80 MB        ~5 KB         ~200 MB       ~20ms
C#          ~40 MB        ~8 KB         ~100 MB       ~15ms

* Rust and Go allocate on the stack or use arena allocation;
  per-request overhead is negligible.
```

### Measuring Memory in Kubernetes

```yaml
# memory-measurement.yaml
apiVersion: v1
kind: Pod
metadata:
  name: memory-measurement
spec:
  containers:
    - name: fusion-service
      image: fusion-service:latest
      resources:
        requests:
          memory: "64Mi"
          cpu: "100m"
        limits:
          memory: "256Mi"
          cpu: "500m"
      # Expose memory metrics for Prometheus
      env:
        - name: RUST_LOG
          value: "info"
        - name: MALLOC_CONF
          value: "background_thread:true,stats_print:true"
```

```python
# memory_monitor.py — Track memory per language process
import psutil
import time
import json

def monitor_process_memory(pid, duration_seconds=60):
    """Monitor memory usage of a process over time."""
    process = psutil.Process(pid)
    samples = []

    for _ in range(duration_seconds):
        mem = process.memory_info()
        samples.append({
            "timestamp": time.time(),
            "rss_mb": mem.rss / 1024 / 1024,
            "vms_mb": mem.vms / 1024 / 1024,
        })
        time.sleep(1)

    return {
        "pid": pid,
        "avg_rss_mb": sum(s["rss_mb"] for s in samples) / len(samples),
        "max_rss_mb": max(s["rss_mb"] for s in samples),
        "samples": samples,
    }
```

## Pre-Warming Strategies

The most effective cold start mitigation is to not cold start at all. Pre-warming keeps functions hot and ready.

### Strategy 1: Periodic Ping

```python
# prewarmer.py — Keep functions warm with periodic invocations
import boto3
import time
from apscheduler.schedulers.background import BackgroundScheduler

lambda_client = boto3.client('lambda')

def prewarm_function(function_name):
    """Invoke a Lambda function to keep it warm."""
    try:
        lambda_client.invoke(
            FunctionName=function_name,
            InvocationType='RequestResponse',
            Payload=json.dumps({"prewarm": True}),
        )
        print(f"Pre-warmed {function_name}")
    except Exception as e:
        print(f"Failed to pre-warm {function_name}: {e}")

# Pre-warm every 5 minutes (Lambda keeps instances alive for ~15 min)
scheduler = BackgroundScheduler()
scheduler.add_job(prewarm_function, 'interval', minutes=5, args=['fusion-user-api'])
scheduler.add_job(prewarm_function, 'interval', minutes=5, args=['fusion-auth-api'])
scheduler.add_job(prewarm_function, 'interval', minutes=5, args=['fusion-core-api'])
scheduler.start()
```

### Strategy 2: SnapStart (AWS Lambda)

```yaml
# SAM template with SnapStart
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31

Resources:
  UserApiFunction:
    Type: AWS::Serverless::Function
    Properties:
      Runtime: java21
      Handler: com.fusion.user.ApiHandler
      SnapStart:
        ApplyOn: PublishedVersions
      MemorySize: 1024
      # SnapStart reduces cold start from ~1200ms to ~200ms
      # by snapshotting the initialized JVM state
```

### Strategy 3: Provisioned Concurrency

```yaml
# Provisioned concurrency keeps N instances always warm
UserApiFunction:
  Type: AWS::Serverless::Function
  Properties:
    Runtime: python3.12
    Handler: handler.main
    AutoPublishAlias: live
    ProvisionedConcurrencyConfig:
      ProvisionedConcurrentExecutions: 5
    # Cost: ~$15/month per provisioned instance
    # Benefit: 0ms cold start for up to 5 concurrent requests
```

### Strategy 4: Container Pre-Initialization

```dockerfile
# Dockerfile with pre-initialization
FROM python:3.12-slim

WORKDIR /app

# Install dependencies first (cached layer)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Pre-import heavy modules
RUN python -c "import fusion; import fastapi; import pydantic"

# Copy application code
COPY src/ ./src/

# Pre-warm the application
RUN python -c "from fusion.app import create_app; create_app()"

CMD ["uvicorn", "fusion.app:create_app", "--host", "0.0.0.0", "--port", "8000"]
```

## Trampoline Functions in Fast-Booting Languages

When a polyglot system has a mix of fast and slow languages, use trampoline functions: lightweight functions in fast-booting languages (Rust, Go) that route requests to slower services only when needed.

### The Trampoline Pattern

```rust
// trampoline.rs — Rust trampoline that routes to slower languages
use std::time::Instant;

enum ServiceStatus {
    Ready,
    Warming,
    Cold,
}

struct TrampolineRouter {
    rust_handler: Box<dyn Handler>,
    python_process: Option<ChildProcess>,
    python_status: ServiceStatus,
}

impl TrampolineRouter {
    fn new() -> Self {
        Self {
            rust_handler: Box::new(RustUserHandler::new()),
            python_process: None,
            python_status: ServiceStatus::Cold,
        }
    }

    async fn handle_request(&mut self, request: Request) -> Response {
        let start = Instant::now();

        // Route to Rust if possible (fast path)
        if self.rust_handler.can_handle(&request) {
            let response = self.rust_handler.handle(request).await;
            tracing::info!(
                latency_us = start.elapsed().as_micros(),
                path = "rust_fast_path",
                "Request handled by Rust trampoline"
            );
            return response;
        }

        // Ensure Python is warm
        if matches!(self.python_status, ServiceStatus::Cold) {
            self.warm_python().await;
        }

        // Route to Python (slow path)
        let response = self.forward_to_python(request).await;
        tracing::info!(
            latency_us = start.elapsed().as_micros(),
            path = "python_slow_path",
            "Request forwarded to Python"
        );
        response
    }

    async fn warm_python(&mut self) {
        tracing::info!("Pre-warming Python process");
        self.python_status = ServiceStatus::Warming;

        // Start Python process in background
        let process = Command::new("python")
            .arg("-m")
            .arg("fusion.python_handler")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start Python");

        // Wait for Python to signal readiness
        // (e.g., read "ready" from stdout)
        self.python_process = Some(process);
        self.python_status = ServiceStatus::Ready;
    }
}
```

### Go Trampoline with Fallback

```go
// trampoline.go — Go trampoline with automatic fallback
package trampoline

import (
    "context"
    "time"
)

type Router struct {
    goHandler    Handler
    pythonClient *http.Client
    pythonReady  bool
}

func NewRouter() *Router {
    return &Router{
        goHandler:    &GoUserHandler{},
        pythonClient: &http.Client{Timeout: 5 * time.Second},
        pythonReady:  false,
    }
}

func (r *Router) Handle(ctx context.Context, req Request) Response {
    start := time.Now()

    // Fast path: Go can handle this
    if r.goHandler.CanHandle(req) {
        resp := r.goHandler.Handle(ctx, req)
        logRequest("go_fast", start)
        return resp
    }

    // Slow path: Forward to Python
    if !r.pythonReady {
        r.preheatPython(ctx)
    }

    resp, err := r.forwardToPython(ctx, req)
    if err != nil {
        // Fallback: try Go even if it's not ideal
        logRequest("python_failed_fallback_to_go", start)
        return r.goHandler.FallbackHandle(ctx, req)
    }

    logRequest("python_slow", start)
    return resp
}

func (r *Router) preheatPython(ctx context.Context) {
    // Send a lightweight request to warm up Python
    req, _ := http.NewRequestWithContext(ctx, "GET", "http://python:8000/healthz", nil)
    r.pythonClient.Do(req)
    r.pythonReady = true
}
```

### Trampoline Decision Matrix

```
Request Type           Trampoline Language   Why
─────────────────────────────────────────────────────────────
JSON validation        Rust                  No runtime needed
String processing      Rust                  Zero-copy, SIMD
Database queries       Go                    Connection pooling
ML inference           Python                Library ecosystem
Report generation      Python                pandas, matplotlib
File processing        Rust                  Memory safety
Authentication         Go                    Concurrency model
Business rules         Either                Depends on complexity
```

## Resource Optimization Summary

| Strategy                     | Cold Start Reduction | Cost     | Complexity |
|------------------------------|----------------------|----------|------------|
| Trampoline functions         | 80-99%               | Low      | Medium     |
| Pre-warming (periodic ping)  | 90-99%               | Low      | Low        |
| SnapStart (JVM)             | 80-90%               | None     | Low        |
| Provisioned concurrency      | 100%                 | High     | Low        |
| GraalVM native-image        | 80-90%               | None     | Medium     |
| Rust/Go for fast paths      | 95-99%               | None     | High       |
| Docker pre-initialization   | 60-80%               | None     | Low        |

## Summary

Cold start is a solvable problem. The key insight: you don't need every language to start fast — you need the fast languages to shield the slow ones. Trampoline functions in Rust/Go handle most requests without ever touching Python. Pre-warming keeps the slow languages hot. And when cold starts are unavoidable, SnapStart and provisioned concurrency eliminate them entirely.

The 80/20 rule applies: handle 80% of requests in Rust/Go (1-2ms cold start), and reserve Python for the 20% that truly need its ecosystem. Your p99 latency will thank you.
