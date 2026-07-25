# Chapter 41: Signal Handling & Graceful Shutdowns

Your polyglot service handles 10,000 requests per second. Kubernetes sends SIGTERM. If you don't handle it correctly, 50 in-flight requests are dropped, database connections leak, and your customer data gets corrupted. This chapter ensures that never happens.

## The Problem with Polyglot Shutdown

Each language handles signals differently, and when multiple languages are running in one process, coordinating shutdown becomes a distributed systems problem.

```
The Shutdown Coordination Challenge:

Main Process (Rust)
  │
  ├── Signal Handler (Rust): receives SIGTERM
  │   └── Needs to tell Python to shutdown
  │       └── Python's atexit handlers fire
  │           └── But Python doesn't know about Rust's in-flight requests
  │               └── Rust is still processing while Python is shutting down
  │                   └── Data corruption risk
  │
  └── Go goroutines (via FFI)
      └── Go's signal handling is independent
          └── May not receive SIGTERM at all
              └── Goroutines leak
```

### Why Default Shutdown Fails

```
Without proper shutdown handling:

1. SIGTERM received
2. Main process immediately exits
3. In-flight requests: DROPPED (50 requests at 3am)
4. Database connections: LEAKED (connection pool exhaustion)
5. File handles: NOT FLUSHED (data corruption)
6. Background tasks: ABANDONED (partial writes)
7. Cache: NOT SYNCED (stale data on restart)
8. Metrics: LOST (monitoring gaps)
```

## SIGTERM Propagation

### The Right Way to Handle Signals

```python
# shutdown.py — Python signal handling
import signal
import sys
import asyncio
from contextlib import asynccontextmanager

class GracefulShutdown:
    def __init__(self):
        self._shutdown_event = asyncio.Event()
        self._handlers = []
        self._timeout = 30  # seconds

    def register_handler(self, handler):
        """Register a shutdown handler (coroutine or function)."""
        self._handlers.append(handler)

    def setup_signal_handlers(self, loop):
        """Register OS signal handlers with the event loop."""
        for sig in (signal.SIGTERM, signal.SIGINT):
            loop.add_signal_handler(
                sig,
                lambda s=sig: asyncio.create_task(self._shutdown(s))
            )

    async def _shutdown(self, sig):
        """Execute graceful shutdown."""
        print(f"Received {sig.name}, starting graceful shutdown...")

        # Set event so main loop knows to stop accepting new requests
        self._shutdown_event.set()

        # Execute shutdown handlers in reverse order (LIFO)
        for handler in reversed(self._handlers):
            try:
                if asyncio.iscoroutinefunction(handler):
                    await asyncio.wait_for(handler(), timeout=self._timeout)
                else:
                    handler()
            except Exception as e:
                print(f"Shutdown handler error: {e}")

        print("Graceful shutdown complete")

    @property
    def should_stop(self):
        return self._shutdown_event.is_set()

# Usage
shutdown = GracefulShutdown()

@asynccontextmanager
async def lifespan(app):
    """FastAPI lifespan with graceful shutdown."""
    loop = asyncio.get_event_loop()
    shutdown.setup_signal_handlers(loop)

    # Register shutdown handlers
    shutdown.register_handler(close_database_pool)
    shutdown.register_handler(close_redis_connection)
    shutdown.register_handler(flush_pending_writes)
    shutdown.register_handler(deregister_from_service_mesh)

    yield

    # Cleanup is handled by shutdown handlers
```

### Rust Signal Handling

```rust
// shutdown.rs — Rust signal handling with tokio
use tokio::signal;
use tokio::sync::broadcast;
use std::sync::Arc;

pub struct ShutdownManager {
    shutdown_tx: broadcast::Sender<()>,
    connections: Arc<tokio::sync::Mutex<Vec<ConnectionHandle>>>,
}

impl ShutdownManager {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            shutdown_tx,
            connections: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn wait_for_shutdown(&self) {
        // Wait for SIGTERM or SIGINT
        let ctrl_c = signal::ctrl_c();
        #[cfg(unix)]
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");

        tokio::select! {
            _ = ctrl_c => {
                println!("Received SIGINT, shutting down...");
            }
            #[cfg(unix)]
            _ = sigterm.recv() => {
                println!("Received SIGTERM, shutting down...");
            }
        }

        // Broadcast shutdown signal to all tasks
        let _ = self.shutdown_tx.send(());

        // Wait for in-flight requests to complete
        self.drain_connections().await;
    }

    async fn drain_connections(&self) {
        let timeout = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();

        loop {
            let connections = self.connections.lock().await;
            if connections.is_empty() || start.elapsed() > timeout {
                break;
            }
            drop(connections);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    pub fn shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
}
```

### Cross-Language Shutdown Coordination

```rust
// ffi_shutdown.rs — Coordinate shutdown across Rust and Python
use pyo3::prelude::*;

#[pyfunction]
fn register_python_shutdown_handler(py: Python, callback: PyObject) -> PyResult<()> {
    // Store callback in global state
    SHUTDOWN_CALLBACKS.lock().unwrap().push(callback);

    // Listen for Rust shutdown signal
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut shutdown_rx = global_shutdown_manager().shutdown_receiver();
        let _ = shutdown_rx.recv().await;

        // Execute Python shutdown handlers
        py.allow_threads(|| {
            for callback in SHUTDOWN_CALLBACKS.lock().unwrap().iter() {
                Python::with_gil(|py| {
                    let _ = callback.call0(py);
                });
            }
        });
    });

    Ok(())
}

// Python side
#[pyfunction]
fn on_shutdown(callback: PyObject) {
    register_python_shutdown_handler(callback);
}
```

## Shutdown Orchestration

### The Shutdown Order

Not all shutdown handlers can run in parallel. Some depend on others:

```
Shutdown Order (dependencies):

1. Stop accepting new requests (no dependencies)
   │
   ├── 2a. Flush pending writes (depends on 1)
   │       │
   │       └── 3a. Close file handles (depends on 2a)
   │
   ├── 2b. Deregister from service mesh (depends on 1)
   │
   └── 2c. Stop background tasks (depends on 1)
           │
           └── 3b. Close database connections (depends on 2c)
                   │
                   └── 3c. Close Redis connections (depends on 2c)
                           │
                           └── 4. Final cleanup (depends on all 3s)
                                   │
                                   └── 5. Exit process
```

### Shutdown Manager Implementation

```python
# shutdown_manager.py — Orchestrated shutdown
import asyncio
from enum import Enum
from dataclasses import dataclass
from typing import List, Callable, Awaitable

class ShutdownPhase(Enum):
    STOP_ACCEPTING = 100
    FLUSH_WRITES = 200
    CLOSE_CONNECTIONS = 300
    FINAL_CLEANUP = 400

@dataclass
class ShutdownHandler:
    name: str
    phase: ShutdownPhase
    handler: Callable[[], Awaitable[None]]
    timeout: float = 10.0

class ShutdownOrchestrator:
    def __init__(self):
        self._handlers: List[ShutdownHandler] = []
        self._shutdown_event = asyncio.Event()

    def register(self, name: str, phase: ShutdownPhase, timeout: float = 10.0):
        """Decorator to register a shutdown handler."""
        def decorator(func):
            self._handlers.append(ShutdownHandler(
                name=name, phase=phase, handler=func, timeout=timeout
            ))
            return func
        return decorator

    async def shutdown(self):
        """Execute all shutdown handlers in phase order."""
        self._shutdown_event.set()

        # Sort handlers by phase
        sorted_handlers = sorted(self._handlers, key=lambda h: h.phase.value)

        # Group handlers by phase (handlers in same phase run in parallel)
        phases = {}
        for handler in sorted_handlers:
            phases.setdefault(handler.phase, []).append(handler)

        for phase, handlers in sorted(phases.items()):
            print(f"Executing shutdown phase: {phase.name}")

            # Run handlers in this phase in parallel
            tasks = []
            for handler in handlers:
                tasks.append(self._run_handler(handler))

            # Wait for all handlers in this phase
            results = await asyncio.gather(*tasks, return_exceptions=True)

            # Check for errors
            for handler, result in zip(handlers, results):
                if isinstance(result, Exception):
                    print(f"ERROR in {handler.name}: {result}")

        print("Shutdown complete")

    async def _run_handler(self, handler: ShutdownHandler):
        """Run a single handler with timeout."""
        try:
            await asyncio.wait_for(handler.handler(), timeout=handler.timeout)
        except asyncio.TimeoutError:
            print(f"WARNING: {handler.name} timed out after {handler.timeout}s")

# Usage
orchestrator = ShutdownOrchestrator()

@orchestrator.register("stop_server", ShutdownPhase.STOP_ACCEPTING, timeout=5.0)
async def stop_server():
    server.close()
    await server.wait_closed()

@orchestrator.register("flush_writes", ShutdownPhase.FLUSH_WRITES, timeout=15.0)
async def flush_writes():
    await write_buffer.flush()

@orchestrator.register("close_db", ShutdownPhase.CLOSE_CONNECTIONS, timeout=10.0)
async def close_db():
    await database_pool.close()

@orchestrator.register("close_redis", ShutdownPhase.CLOSE_CONNECTIONS, timeout=5.0)
async def close_redis():
    await redis_client.close()
```

## Python atexit vs JVM Hooks vs Rust Handlers

### Python atexit

```python
# atexit_handlers.py
import atexit
import sys

def cleanup_database():
    """Close database connections."""
    db.close()

def cleanup_temp_files():
    """Remove temporary files."""
    import shutil
    shutil.rmtree("/tmp/fusion-temp", ignore_errors=True)

# atexit handlers run in LIFO order
atexit.register(cleanup_database)
atexit.register(cleanup_temp_files)

# PROBLEM: atexit doesn't handle SIGTERM!
# It only fires on normal process exit.
# You need signal handlers to bridge this gap.

import signal

def handle_sigterm(signum, frame):
    """Convert SIGTERM to normal exit (triggers atexit)."""
    sys.exit(0)

signal.signal(signal.SIGTERM, handle_sigterm)
```

### JVM Shutdown Hooks

```java
// ShutdownHook.java
public class GracefulShutdown {
    private static final ExecutorService executor = Executors.newFixedThreadPool(4);
    private static final List<Runnable> shutdownHooks = new CopyOnWriteArrayList<>();

    public static void registerShutdownHook(Runnable hook) {
        shutdownHooks.add(hook);
    }

    static {
        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            System.out.println("Shutdown hook triggered");

            // Execute all registered hooks
            List<Future<?>> futures = new ArrayList<>();
            for (Runnable hook : shutdownHooks) {
                futures.add(executor.submit(hook));
            }

            // Wait for all hooks to complete (with timeout)
            long deadline = System.currentTimeMillis() + 30_000;
            for (Future<?> future : futures) {
                long remaining = deadline - System.currentTimeMillis();
                if (remaining <= 0) {
                    System.err.println("Shutdown timed out");
                    break;
                }
                try {
                    future.get(remaining, TimeUnit.MILLISECONDS);
                } catch (Exception e) {
                    System.err.println("Shutdown hook failed: " + e.getMessage());
                }
            }

            executor.shutdownNow();
            System.out.println("Shutdown complete");
        }));
    }
}
```

### Rust Drop + Signal Handling

```rust
// rust_shutdown.rs
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct GracefulServer {
    shutdown_flag: Arc<AtomicBool>,
}

impl GracefulServer {
    pub fn new() -> Self {
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        // Clone for signal handler
        let flag = shutdown_flag.clone();

        // Register signal handler
        ctrlc::set_handler(move || {
            println!("Received shutdown signal");
            flag.store(true, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        Self { shutdown_flag }
    }

    pub fn should_shutdown(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }
}

// Drop implementation for cleanup
impl Drop for GracefulServer {
    fn drop(&mut self) {
        println!("Cleaning up resources...");
        // Cleanup happens here automatically
    }
}
```

## Health Checks During Shutdown

### Kubernetes Health Check Configuration

```yaml
# kubernetes-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fusion-service
spec:
  template:
    spec:
      terminationGracePeriodSeconds: 45
      containers:
        - name: fusion
          image: fusion:latest
          ports:
            - containerPort: 8080
          livenessProbe:
            httpGet:
              path: /health/live
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 5
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 2
          lifecycle:
            preStop:
              exec:
                command: ["/bin/sh", "-c", "sleep 5"]
```

### Health Check Implementation

```python
# health.py — Health check endpoints
from fastapi import FastAPI, Response
import asyncio

app = FastAPI()

# Shutdown state
is_shutting_down = False

@app.get("/health/live")
async def liveness():
    """Liveness: is the process running?"""
    return {"status": "alive"}

@app.get("/health/ready")
async def readiness():
    """Readiness: can the process accept traffic?"""
    if is_shutting_down:
        return Response(content='{"status":"not_ready"}', status_code=503)

    # Check all dependencies
    checks = {
        "database": await check_database(),
        "redis": await check_redis(),
        "disk_space": check_disk_space(),
    }

    if all(checks.values()):
        return {"status": "ready", "checks": checks}
    else:
        return Response(
            content=f'{{"status":"not_ready","checks":{checks}}}',
            status_code=503
        )

@app.get("/health/shutdown")
async def shutdown_status():
    """Status of ongoing shutdown."""
    return {
        "shutting_down": is_shutting_down,
        "in_flight_requests": get_in_flight_count(),
        "shutdown_progress": get_shutdown_progress(),
    }
```

## Best Practices

1. **Always set `terminationGracePeriodSeconds`** in Kubernetes (default 30s)
2. **Add `preStop` sleep** to let load balancers drain
3. **Return 503 from readiness probe** during shutdown
4. **Use phased shutdown** — stop accepting before closing connections
5. **Set timeouts on every shutdown handler** — don't wait forever
6. **Log shutdown progress** — debugging 3am outages requires visibility
7. **Test shutdown regularly** — don't wait for production incidents
8. **Monitor connection leaks** — the #1 symptom of broken shutdown

## Summary

Graceful shutdown in polyglot systems requires:

1. **Signal propagation** across language boundaries
2. **Phased shutdown** with dependency ordering
3. **Health checks** that reflect shutdown state
4. **Timeouts** to prevent hanging shutdowns
5. **Testing** the shutdown path as rigorously as the happy path

The 30 seconds between SIGTERM and process kill is the difference between a clean restart and a corrupted database.
