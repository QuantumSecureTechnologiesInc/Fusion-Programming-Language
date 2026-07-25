# Chapter 36: Performance Profiling Across Language Boundaries

Polyglot systems amplify every performance problem. A 2x slowdown in a pure-Rust service is manageable; the same slowdown buried in an FFI call from Python to Rust, triggered by a serialization layer, and measured with Python's profiler that can't see past the boundary is a nightmare.

This chapter teaches you to see across language walls.

## End-to-End Tracing

Traditional profiling works within a single process. Polyglot systems need instrumentation that follows requests across language boundaries, through serialization layers, and into foreign runtimes.

### OpenTelemetry for Polyglot Systems

OpenTelemetry provides the standard for distributed tracing across language boundaries. Every language has an OTel SDK that speaks the same protocol.

```python
# Python service - producer side
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter

provider = TracerProvider()
processor = BatchSpanProcessor(OTLPSpanExporter())
provider.add_span_processor(processor)
trace.set_tracer_provider(provider)

tracer = trace.get_tracer("data-pipeline")

def process_batch(records):
    with tracer.start_as_current_span("python.process_batch") as span:
        span.set_attribute("batch.size", len(records))
        
        # Context propagates automatically through W3C Trace Context headers
        result = ffi_call_rust_preprocess(records)
        
        return result
```

```rust
// Rust service - consumer side
use opentelemetry::{trace::{Tracer, Span}, Context};

fn preprocess(records: Vec<Record>, parent_ctx: Context) -> Vec<ProcessedRecord> {
    let tracer = global::tracer("data-pipeline");
    let mut span = tracer
        .span_builder("rust.preprocess")
        .with_parent_context(parent_ctx)
        .start(&tracer);
    
    span.set_attribute("input.count", records.len() as i64);
    
    let result = expensive_computation(records);
    
    span.set_attribute("output.count", result.len() as i64);
    span.end();
    
    result
}
```

### Trace Context Propagation Across Languages

The W3C Trace Context standard ensures trace IDs survive language boundaries:

```
# HTTP headers automatically propagated:
traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
tracestate: vendor1=value1,vendor2=value2
```

Fusion runtime automatically injects and extracts these headers at FFI boundaries:

```fusion
@ffi_boundary(trace_propagation: w3c)
def call_external_service(payload: bytes) -> bytes:
    # Fusion runtime automatically:
    # 1. Extracts traceparent from current context
    # 2. Injects it into outgoing HTTP/FFI call
    # 3. Creates child span for this boundary crossing
    return http_post("http://rust-service/preprocess", payload)
```

### Span Creation at FFI Boundaries

Every language transition is a span:

```
[Python: request_handler] ──FFI──> [Rust: preprocess] ──FFI──> [C: crypto] ──FFI──> [Rust: serialize]
         │                              │                           │
         └── 15ms                      └── 3ms                     └── 1ms
```

The key insight: **spans at boundaries capture overhead that no single-language profiler can see**.

### Correlation IDs Through the Stack

For non-HTTP flows (shared memory, direct FFI), use correlation IDs:

```python
import uuid

def correlated_ffi_call(data: dict) -> dict:
    correlation_id = str(uuid.uuid4())
    
    # Log correlation ID in Python
    logger.info(f"FFI call starting", correlation_id=correlation_id)
    
    # Pass through FFI boundary
    result = rust_library.process(
        json.dumps(data),
        correlation_id=correlation_id
    )
    
    # Verify correlation in logs
    logger.info(f"FFI call complete", correlation_id=correlation_id)
    
    return json.loads(result)
```

## Context-Switching Overhead

The dirty secret of polyglot systems: most overhead isn't computation, it's context-switching.

### Measuring FFI Call Overhead

```python
import time
import statistics

def measure_ffi_overhead(calls=10000):
    """Measure the actual cost of crossing the FFI boundary."""
    overheads = []
    
    for _ in range(calls):
        start = time.perf_counter_ns()
        rust_library.identity(b"test")  # Minimal work on other side
        end = time.perf_counter_ns()
        overheads.append(end - start)
    
    return {
        "median_ns": statistics.median(overheads),
        "p99_ns": sorted(overheads)[int(len(overheads) * 0.99)],
        "mean_ns": statistics.mean(overheads),
    }

# Typical results:
# PyO3: 200-500ns median
# CFFI: 100-300ns median
# ctypes: 50-150ns median (but unsafe)
# JNI: 50-100ns median
# Wasm FFI: 5000-50000ns median
```

### The Performance Cliff: Interpreted → Native Transitions

```
                        │
Throughput              │              ╭──── Native only
                        │            ╭─╯
                        │          ╭─╯
                        │        ╭─╯    ← Sweet spot:
                        │      ╭─╯       batch 100-1000 items
                        │    ╭─╯
                        │  ╭─╯
                        │╭─╯
                        ├╯ ← Per-call FFI overhead kills throughput
                        │
                        └─────────────────────────
                           Items per FFI call →
```

The cliff occurs because:
1. Each FFI call has fixed overhead (argument marshaling, stack setup)
2. The overhead dominates for small payloads
3. Throughput collapses when calls exceed ~100/second

### Batch Operations to Minimize Boundary Crossings

```python
# BAD: Per-item FFI calls
def process_items_bad(items: List[Data]) -> List[Result]:
    return [rust_library.process(item.serialize()) for item in items]

# GOOD: Batch FFI call
def process_items_good(items: List[Data]) -> List[Result]:
    batch = serialize_batch(items)
    results = rust_library.process_batch(batch)
    return deserialize_batch(results)
```

**Rule of thumb**: Batch when crossing more than 10 items. The serialization cost is usually less than 10 FFI call overheads.

### Buffer Pooling Strategies

Every FFI boundary that passes data creates a copy. Pooling reduces allocation pressure:

```rust
use crossbeam::queue::ArrayQueue;

pub struct BufferPool {
    pool: ArrayQueue<Vec<u8>>,
    buffer_size: usize,
}

impl BufferPool {
    pub fn new(capacity: usize, buffer_size: usize) -> Self {
        Self {
            pool: ArrayQueue::new(capacity),
            buffer_size,
        }
    }
    
    pub fn get(&self) -> Vec<u8> {
        self.pool.pop().unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }
    
    pub fn put(&self, mut buf: Vec<u8>) {
        buf.clear();
        if self.pool.push(buf).is_err() {
            // Pool full, let it drop
        }
    }
}

// Usage at FFI boundary
#[no_mangle]
pub extern "C" fn process_data(
    input: *const u8,
    len: usize,
    output: *mut *mut u8,
    output_len: *mut usize,
) -> i32 {
    let buffer = BUFFER_POOL.get();
    // ... process into buffer ...
    // Return pointer to pooled buffer
    // Caller must return it via return_buffer()
}
```

## Flame Graphs by Language

Flame graphs visualize where time is spent. In polyglot systems, you need to see across languages.

### Color-Coded Flame Graphs

Standard flame graphs use color by package. For polyglot systems, color by language:

```
Rust functions:    Red/Orange (#e03131, #f76707)
Python functions:  Blue/Indigo (#1c7ed6, #5c7cfa)
JavaScript:        Yellow (#fcc419, #fab005)
Go:                Cyan (#15aabf, #22b8cf)
Java:              Green (#2f9e44, #40c057)
C/C++:             Purple (#9c36b5, #be4bdb)
```

Tools like `flamegraph` (Rust) support custom color maps:

```bash
# Generate polyglot flame graph
perf script | \
  stackcollapse-perf.pl | \
  polyglot-colorize.py | \
  flamegraph.pl --title "Polyglot Profile" > profile.svg
```

### Tools for Polyglot Profiling

**Python: py-spy**
```bash
# Attach to running Python process
py-spy record -o python_profile.svg --duration 30 --pid $PID

# Shows native Rust code called via PyO3
py-spy top --pid $PID
```

**JavaScript: Chrome DevTools**
```bash
node --prof app.js
node --prof-process isolate-*.log > processed.txt

# For async profiling
node --inspect app.js
# Open chrome://inspect in Chrome
```

**Rust: cargo-flamegraph**
```bash
cargo flamegraph --bench my_benchmark
# Includes FFI calls from/to C and Python
```

**Java: async-profiler**
```bash
# Attach to running JVM
./profiler.sh -d 30 -o flamegraph.html $PID

# Includes JNI calls to native code
```

**Go: pprof**
```bash
go tool pprof http://localhost:6060/debug/pprof/profile
# Shows CGo calls
```

**Fusion: fusion profile**
```bash
# Profile entire polyglot pipeline
fusion profile --lang all --duration 30 --output polyglot_profile.html

# Profile specific boundary
fusion profile --boundary python-to-rust --output boundary_profile.html
```

### Identifying Hot Paths Across Languages

```
┌─────────────────────────────────────────────────────────────┐
│ Total Time: 1000ms                                          │
├─────────────────────────────────────────────────────────────┤
│ Python: 400ms (40%)                                         │
│   ├─ Data preparation: 150ms                                │
│   ├─ Serialization: 100ms                                   │
│   └─ Result handling: 150ms                                 │
├─────────────────────────────────────────────────────────────┤
│ FFI Boundary: 200ms (20%)                                   │
│   ├─ Argument marshaling: 80ms                              │
│   ├─ Memory allocation: 60ms                                │
│   └─ Context switching: 60ms                                │
├─────────────────────────────────────────────────────────────┤
│ Rust: 350ms (35%)                                           │
│   ├─ Core computation: 300ms                                │
│   └─ Memory management: 50ms                                │
├─────────────────────────────────────────────────────────────┤
│ GC/Other: 50ms (5%)                                         │
└─────────────────────────────────────────────────────────────┘
```

**Key insight**: FFI boundary (20%) is often the second-largest cost. Reducing crossings is usually cheaper than optimizing the computation.

## Profiling Tools Matrix

| Language | Sampling Profiler | Tracing Profiler | Flame Graphs | FFI Visibility |
|----------|------------------|------------------|--------------|----------------|
| **Python** | cProfile, py-spy, scalene | line_profiler, memory_profiler | py-spy, austin | PyO3 traces |
| **JavaScript** | Chrome DevTools, node --prof | clinic.js, 0x | clinic flame | N-API traces |
| **Rust** | perf, cargo-flamegraph | tracing, tokio-console | flamegraph crate | FFI spans |
| **Java** | JProfiler, async-profiler | YourKit, VisualVM | async-profiler | JNI spans |
| **Go** | pprof, go tool trace | Jaeger, Zipkin | pprof web | CGo spans |
| **C/C++** | perf, Instruments, VTune | strace, DTrace | flamegraph.pl | N/A (native) |
| **Fusion** | fusion profile | fusion trace | Built-in | All boundaries |

### Python: cProfile, py-spy, scalene

```python
# cProfile - function-level profiling
import cProfile
import pstats

cProfile.run('process_data()', 'profile_output')
stats = pstats.Stats('profile_output')
stats.sort_stats('cumulative')
stats.print_stats(20)

# py-spy - sampling profiler (attached to running process)
# Install: pip install py-spy
# Run: py-spy top --pid <PID>
# Run: py-spy record -o flame.svg --pid <PID>

# scalene - CPU, GPU, memory profiler
# Install: pip install scalene
# Run: scalene script.py
```

### JavaScript: Chrome DevTools, node --prof

```javascript
// node --prof
// Run: node --prof app.js
// Process: node --prof-process isolate-*.log > profile.txt

// Chrome DevTools profiling
// 1. Start with --inspect flag
// 2. Open chrome://inspect
// 3. Click "inspect" on your process
// 4. Go to "Performance" tab
// 5. Record and analyze

// clinic.js
// Install: npm install -g clinic
// Run: clinic doctor -- node app.js
// Run: clinic flame -- node app.js
```

### Rust: perf, cargo-flamegraph

```bash
# perf (Linux)
cargo build --release
perf record -g ./target/release/my_app
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg

# cargo-flamegraph
cargo install flamegraph
cargo flamegraph

# For benchmarks
cargo bench --bench my_benchmark
```

### Java: JProfiler, async-profiler

```bash
# async-profiler (recommended - low overhead)
git clone https://github.com/async-profiler/async-profiler
cd async-profiler && make
./profiler.sh -d 30 -f flamegraph.html -o flamegraph $PID

# JProfiler (commercial)
# Attach to JVM via GUI
# Analyze CPU, memory, threads, locks
```

### Go: pprof

```go
import _ "net/http/pprof"

func main() {
    go func() {
        http.ListenAndServe("localhost:6060", nil)
    }()
    // ... your app ...
}

// Run: go tool pprof http://localhost:6060/debug/pprof/profile?seconds=30
// Web UI: go tool pprof -http=:8080 profile.pb.gz
```

### Fusion: fusion profile

```bash
# Profile entire pipeline
fusion profile --input data.json --output profile.html

# Profile with language breakdown
fusion profile --breakdown-by-lang --output lang_breakdown.html

# Profile specific function
fusion profile --function "process_batch" --output func_profile.html

# Compare profiles (before/after optimization)
fusion profile diff before.html after.html --output diff.html
```

## Optimization Strategies

### Moving Compute to the Right Language

Profile first, then move:

```
Decision Matrix:
┌────────────────────────┬──────────────┬──────────────┬──────────────┐
│ Task Type              │ Python       │ Rust         │ JavaScript   │
├────────────────────────┼──────────────┼──────────────┼──────────────┤
│ I/O-bound (API calls)  │ Good         │ Good         │ Good         │
│ CPU-bound (compute)    │ Poor         │ Excellent    │ Fair         │
│ Data processing        │ Good         │ Excellent    │ Fair         │
│ String manipulation    │ Good         │ Fair         │ Good         │
│ Regex processing       │ Fair         │ Excellent    │ Good         │
│ JSON/serialization     │ Good         │ Excellent    │ Good         │
│ Image/video processing │ Fair (PIL)   │ Excellent    │ Fair         │
└────────────────────────┴──────────────┴──────────────┴──────────────┘
```

**Rule**: Move the hot path, not the whole function.

### Reducing Serialization Overhead

```python
# BAD: Serialize/deserialize per item
for item in items:
    json_bytes = json.dumps(item).encode()
    result = rust_process(json_bytes)
    output.append(json.loads(result))

# GOOD: Batch with efficient format
import msgpack
batch = msgpack.packb(items)
result = rust_process_batch(batch)
output = msgpack.unpackb(result)

# BETTER: Zero-copy at boundary
import numpy as np
array = np.array(data, dtype=np.float32)
# Pass array buffer directly to Rust
result = rust_process_array(array.ctypes.data, len(array))
```

### Caching at Boundaries

```python
from functools import lru_cache
import hashlib

@lru_cache(maxsize=1024)
def cached_rust_call(input_hash: bytes) -> bytes:
    # Cache results of expensive FFI calls
    return rust_library.process(input_hash)

def process_with_cache(data: dict) -> dict:
    data_bytes = msgpack.packb(data)
    input_hash = hashlib.sha256(data_bytes).digest()
    
    result_hash = cached_rust_call(input_hash)
    return msgpack.unpackb(result_hash)
```

### Lazy Evaluation Across Languages

```fusion
// Fusion: Lazy pipeline that defers FFI crossings
def process_pipeline(data: Stream<Record>) -> Stream<Result> {
    data
    |> filter(lambda r: r的重要性 > threshold)  // Python filter (cheap)
    |> batch(1000)                               // Batch for FFI
    |> rust_process_batch()                      // Single FFI crossing
    |> unbatch()                                 // Unbatch results
    |> map(lambda r: format_result(r))           // Python formatting (cheap)
}
```

**Key principle**: Do cheap operations in the current language. Cross boundaries only when necessary, and batch when you do.

## Summary

- **Use OpenTelemetry** for end-to-end tracing across language boundaries
- **Measure FFI overhead** before optimizing computation
- **Batch operations** to minimize boundary crossings
- **Color-code flame graphs** by language for polyglot profiling
- **Move compute to the right language** based on task type
- **Cache at boundaries** to avoid repeated serialization
- **Lazy evaluation** defers expensive crossings until needed

Next chapter: [Chapter 37: Cognitive Load & Team Onboarding →](ch37-polyglot-onboarding.md)
