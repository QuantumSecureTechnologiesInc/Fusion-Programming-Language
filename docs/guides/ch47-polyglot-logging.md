# Chapter 47: Logging Correlation & Distributed Tracing

In a polyglot system, a single user request might touch Python, Rust, Go, and JavaScript services. When something goes wrong, you need to trace that request across all of them. This requires standardized logging, propagated trace IDs, and unified observability. Without it, debugging is a scavenger hunt across five log systems.

## The Logging Problem in Polyglot Systems

Each language has its own logging conventions:

```
Python:  logging.info("User created: %s", user_id)
Rust:    tracing::info!(user_id = %id, "User created");
Go:      log.Printf("User created: %d", userID)
JS:      console.log(`User created: ${userId}`);
```

Five languages, five log formats, five log levels, five timestamp formats. When a request fails, you grep for the request ID across five different log systems and find... nothing, because the IDs don't correlate.

The solution: standardize on a single log schema and propagate a trace ID through every boundary.

## Standardizing Log Schemas

Every log line, in every language, should emit the same JSON structure:

```json
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "level": "info",
  "service": "user-api",
  "trace_id": "abc123def456",
  "span_id": "789012",
  "message": "User created successfully",
  "user_id": 12345,
  "operation": "create_user",
  "duration_ms": 42
}
```

### The Schema Definition

```json
// docs/schemas/log-entry.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Unified Log Entry",
  "type": "object",
  "required": ["timestamp", "level", "service", "message"],
  "properties": {
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 with milliseconds, UTC"
    },
    "level": {
      "type": "string",
      "enum": ["trace", "debug", "info", "warn", "error", "fatal"]
    },
    "service": {
      "type": "string",
      "description": "Service name (e.g., user-api, auth-service)"
    },
    "trace_id": {
      "type": "string",
      "description": "128-bit trace ID, hex-encoded (32 chars)"
    },
    "span_id": {
      "type": "string",
      "description": "64-bit span ID, hex-encoded (16 chars)"
    },
    "parent_span_id": {
      "type": "string",
      "description": "Parent span ID for nested operations"
    },
    "message": {
      "type": "string"
    },
    "error": {
      "type": "object",
      "properties": {
        "type": { "type": "string" },
        "message": { "type": "string" },
        "stack": { "type": "string" }
      }
    },
    "attributes": {
      "type": "object",
      "description": "Service-specific structured data"
    }
  }
}
```

### Implementation in Each Language

```python
# logging_config.py — Unified logging for Python
import logging
import json
import uuid
import time
from datetime import datetime, timezone
from contextvars import ContextVar

# Context variables for trace correlation
trace_id_var: ContextVar[str] = ContextVar('trace_id', default='')
span_id_var: ContextVar[str] = ContextVar('span_id', default='')

class UnifiedFormatter(logging.Formatter):
    """Formats log entries as unified JSON."""

    def format(self, record):
        entry = {
            "timestamp": datetime.now(timezone.utc).isoformat(timespec='milliseconds'),
            "level": record.levelname.lower(),
            "service": getattr(record, 'service', 'unknown'),
            "message": record.getMessage(),
        }

        # Add trace context if available
        tid = trace_id_var.get('')
        sid = span_id_var.get('')
        if tid:
            entry["trace_id"] = tid
        if sid:
            entry["span_id"] = sid

        # Add exception info
        if record.exc_info and record.exc_info[1]:
            entry["error"] = {
                "type": type(record.exc_info[1]).__name__,
                "message": str(record.exc_info[1]),
                "stack": self.formatException(record.exc_info),
            }

        # Add structured attributes
        if hasattr(record, 'attributes'):
            entry["attributes"] = record.attributes

        return json.dumps(entry, default=str)

def setup_logging(service_name: str):
    handler = logging.StreamHandler()
    handler.setFormatter(UndefinedFormatter())
    handler.setFormatter(UnifiedFormatter())

    root = logging.getLogger()
    root.handlers.clear()
    root.addHandler(handler)
    root.setLevel(logging.DEBUG)

    # Store service name
    root.service = service_name

# Middleware to extract/propagate trace context
def trace_middleware(handler):
    """Extract trace_id from headers and set context variables."""
    def wrapper(request):
        # Extract from W3C Trace Context header
        traceparent = request.headers.get('traceparent', '')
        if traceparent:
            parts = traceparent.split('-')
            if len(parts) >= 2:
                trace_id_var.set(parts[1])
                span_id_var.set(parts[2])

        # Generate new trace if not present
        if not trace_id_var.get(''):
            trace_id_var.set(uuid.uuid4().hex)

        return handler(request)
    return wrapper
```

```rust
// logging_config.rs — Unified logging for Rust
use tracing::{info, error, span, Level, Id};
use tracing_subscriber::{fmt, EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

static SPAN_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn init_logging(service_name: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    Registry::default()
        .with(filter)
        .with(fmt::layer().json().with_target(false))
        .init();

    // Set service name for all spans
    tracing::subscriber::with_default(
        tracing_subscriber::fmt::Subscriber::default(),
        || {
            info!(
                service = service_name,
                message = "Logging initialized"
            );
        }
    );
}

/// Create a new span with trace correlation
pub fn create_span(operation: &str, trace_id: &str) -> tracing::span::Span {
    let span_id = SPAN_COUNTER.fetch_add(1, Ordering::SeqCst);
    span!(
        Level::INFO,
        operation,
        trace_id = %trace_id,
        span_id = %format!("{:016x}", span_id),
    )
}

/// Propagate trace context across FFI boundaries
pub fn inject_trace_context(trace_id: &str, span_id: &str) -> Vec<(String, String)> {
    vec![
        ("traceparent".to_string(), format!("00-{trace_id}-{span_id}-01")),
        ("trace_id".to_string(), trace_id.to_string()),
        ("span_id".to_string(), span_id.to_string()),
    ]
}
```

```go
// logging_config.go — Unified logging for Go
package fusionlog

import (
    "context"
    "encoding/json"
    "log"
    "os"
    "time"
)

type LogEntry struct {
    Timestamp   string                 `json:"timestamp"`
    Level       string                 `json:"level"`
    Service     string                 `json:"service"`
    TraceID     string                 `json:"trace_id,omitempty"`
    SpanID      string                 `json:"span_id,omitempty"`
    ParentSpanID string                `json:"parent_span_id,omitempty"`
    Message     string                 `json:"message"`
    Attributes  map[string]interface{} `json:"attributes,omitempty"`
}

type Logger struct {
    serviceName string
    encoder     *json.Encoder
}

func New(serviceName string) *Logger {
    return &Logger{
        serviceName: serviceName,
        encoder:     json.NewEncoder(os.Stdout),
    }
}

func (l *Logger) Info(ctx context.Context, msg string, attrs map[string]interface{}) {
    entry := LogEntry{
        Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
        Level:     "info",
        Service:   l.serviceName,
        Message:   msg,
        Attributes: attrs,
    }

    // Extract trace context from context.Context
    if traceID := ctx.Value("trace_id"); traceID != nil {
        entry.TraceID = traceID.(string)
    }
    if spanID := ctx.Value("span_id"); spanID != nil {
        entry.SpanID = spanID.(string)
    }

    l.encoder.Encode(entry)
}

// Inject trace context into HTTP headers
func InjectTraceHeaders(ctx context.Context) map[string]string {
    headers := make(map[string]string)
    if traceID, ok := ctx.Value("trace_id").(string); ok {
        headers["traceparent"] = fmt.Sprintf("00-%s-%s-01", traceID, generateSpanID())
        headers["trace_id"] = traceID
    }
    return headers
}

// Extract trace context from HTTP headers
func ExtractTraceContext(headers map[string]string) context.Context {
    ctx := context.Background()
    if traceID := headers["trace_id"]; traceID != "" {
        ctx = context.WithValue(ctx, "trace_id", traceID)
    }
    if spanID := headers["span_id"]; spanID != "" {
        ctx = context.WithValue(ctx, "span_id", spanID)
    }
    return ctx
}
```

```javascript
// logger.js — Unified logging for JavaScript
const { AsyncLocalStorage } = require('node:async_hooks');
const crypto = require('node:crypto');

const traceStorage = new AsyncLocalStorage();

class UnifiedLogger {
    constructor(serviceName) {
        this.serviceName = serviceName;
    }

    _createEntry(level, message, attributes = {}) {
        const store = traceStorage.getStore() || {};
        return {
            timestamp: new Date().toISOString(),
            level,
            service: this.serviceName,
            trace_id: store.trace_id || '',
            span_id: store.span_id || '',
            message,
            ...(Object.keys(attributes).length > 0 ? { attributes } : {}),
        };
    }

    info(message, attributes) {
        console.log(JSON.stringify(this._createEntry('info', message, attributes)));
    }

    error(message, error, attributes = {}) {
        const entry = this._createEntry('error', message, {
            ...attributes,
            error: {
                type: error?.constructor?.name || 'Error',
                message: error?.message || String(error),
                stack: error?.stack,
            },
        });
        console.error(JSON.stringify(entry));
    }
}

// Middleware to extract/propagate trace context
function traceMiddleware(handler) {
    return (req, res) => {
        const traceparent = req.headers['traceparent'] || '';
        const traceId = req.headers['trace_id'] || crypto.randomUUID().replace(/-/g, '');
        const spanId = req.headers['span_id'] || crypto.randomBytes(8).toString('hex');

        const store = { trace_id: traceId, span_id: spanId };

        traceStorage.run(store, () => {
            // Set response headers for downstream services
            res.setHeader('trace_id', traceId);
            res.setHeader('span_id', spanId);
            handler(req, res);
        });
    };
}

module.exports = { UnifiedLogger, traceMiddleware };
```

## Propagating trace_id Through Boundaries

The trace ID must survive every boundary crossing: HTTP headers, Kafka messages, C library calls, and FFI calls.

### HTTP Propagation (W3C Trace Context)

```python
# propagation.py — Trace context propagation via HTTP headers
import requests

def make_traced_request(method, url, trace_id=None, **kwargs):
    """Make an HTTP request with trace context propagation."""
    if trace_id is None:
        import uuid
        trace_id = uuid.uuid4().hex

    headers = kwargs.pop('headers', {})
    headers['traceparent'] = f'00-{trace_id}-{generate_span_id()}-01'
    headers['trace_id'] = trace_id

    return requests.request(method, url, headers=headers, **kwargs)

def extract_trace_from_request(request):
    """Extract trace context from an incoming request."""
    trace_id = request.headers.get('trace_id', '')
    span_id = request.headers.get('span_id', '')
    return trace_id, span_id
```

### Kafka Message Propagation

```python
# kafka_propagation.py — Trace context in Kafka headers
from confluent_kafka import Producer, Consumer, KafkaError
import json
import uuid

def produce_with_trace(producer, topic, message, trace_id=None):
    """Produce a Kafka message with trace context in headers."""
    if trace_id is None:
        trace_id = uuid.uuid4().hex

    headers = [
        ('trace_id', trace_id.encode()),
        ('content_type', b'application/json'),
    ]

    producer.produce(
        topic,
        value=json.dumps(message).encode(),
        headers=headers,
    )
    producer.flush()

def consume_with_trace(consumer):
    """Consume a Kafka message and extract trace context."""
    msg = consumer.poll(1.0)
    if msg is None:
        return None, None, None
    if msg.error():
        return None, None, None

    # Extract trace context from headers
    trace_id = ''
    for key, value in msg.headers():
        if key == 'trace_id':
            trace_id = value.decode()

    message = json.loads(msg.value().decode())
    return trace_id, msg.topic(), message
```

### C Library Propagation

```c
/* trace_propagation.h — Trace context in C libraries */
#ifndef TRACE_PROPAGATION_H
#define TRACE_PROPAGATION_H

#include <stdint.h>
#include <string.h>

typedef struct {
    char trace_id[33];  /* 128-bit hex + null terminator */
    char span_id[17];   /* 64-bit hex + null terminator */
} trace_context_t;

/* Initialize trace context from environment or generate new */
static inline void trace_context_init(trace_context_t *ctx) {
    /* Try to extract from environment (set by parent process) */
    const char *env_trace = getenv("FUSION_TRACE_ID");
    const char *env_span = getenv("FUSION_SPAN_ID");

    if (env_trace && strlen(env_trace) == 32) {
        strncpy(ctx->trace_id, env_trace, 32);
        ctx->trace_id[32] = '\0';
    } else {
        /* Generate a new trace ID */
        /* In production, use a proper UUID library */
        snprintf(ctx->trace_id, 33, "%08x%08x%08x%08x",
                 rand(), rand(), rand(), rand());
    }

    if (env_span && strlen(env_span) == 16) {
        strncpy(ctx->span_id, env_span, 16);
        ctx->span_id[16] = '\0';
    } else {
        snprintf(ctx->span_id, 17, "%08x%08x", rand(), rand());
    }
}

/* Propagate trace context to child process */
static inline void trace_context_propagate(const trace_context_t *ctx) {
    setenv("FUSION_TRACE_ID", ctx->trace_id, 1);
    setenv("FUSION_SPAN_ID", ctx->span_id, 1);
}

/* Inject trace context into a buffer (for HTTP headers, etc.) */
static inline int trace_context_inject_header(
    const trace_context_t *ctx,
    char *buf, size_t buf_size
) {
    return snprintf(buf, buf_size,
        "trace_id: %s\r\nspan_id: %s\r\ntraceparent: 00-%s-%s-01\r\n",
        ctx->trace_id, ctx->span_id, ctx->trace_id, ctx->span_id);
}

#endif /* TRACE_PROPAGATION_H */
```

## OpenTelemetry Integration

OpenTelemetry is the industry standard for distributed tracing. It provides a vendor-neutral API for collecting traces, metrics, and logs.

### Setup Across Languages

```yaml
# docker-compose.yml — OpenTelemetry Collector
version: '3.8'
services:
  otel-collector:
    image: otel/opentelemetry-collector:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
      - "16686:16686" # Jaeger UI

  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "14268:14268"
      - "16686:16686"
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

```python
# otel_config.py — OpenTelemetry setup for Python
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource

def init_telemetry(service_name: str):
    resource = Resource.create({
        "service.name": service_name,
        "service.version": "1.0.0",
    })

    provider = TracerProvider(resource=resource)
    exporter = OTLPSpanExporter(endpoint="http://otel-collector:4317")
    processor = BatchSpanProcessor(exporter)
    provider.add_span_processor(processor)

    trace.set_tracer_provider(provider)
    return trace.get_tracer(service_name)

# Usage
tracer = init_telemetry("user-api")

def handle_request(request):
    with tracer.start_as_current_span("handle_request") as span:
        span.set_attribute("http.method", request.method)
        span.set_attribute("http.url", request.url)

        with tracer.start_as_current_span("validate_user"):
            user = validate_user(request.body)

        with tracer.start_as_current_span("save_user"):
            result = save_user(user)

        span.set_attribute("user.id", result.id)
        return result
```

```rust
// otel_config.rs — OpenTelemetry setup for Rust
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_telemetry(service_name: &str) {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://otel-collector:4317");

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry::sdk::trace::Config::default()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                ]))
        )
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
        .expect("Failed to install OpenTelemetry pipeline");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(OpenTelemetryLayer::new(provider.tracer(service_name)))
        .init();
}
```

```go
// otel_config.go — OpenTelemetry setup for Go
package fusionotel

import (
    "context"
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
    "go.opentelemetry.io/otel/sdk/resource"
    sdktrace "go.opentelemetry.io/otel/sdk/trace"
    semconv "go.opentelemetry.io/otel/semconv/v1.24.0"
)

func InitTelemetry(ctx context.Context, serviceName string) (func(), error) {
    exporter, err := otlptracegrpc.New(ctx,
        otlptracegrpc.WithEndpoint("otel-collector:4317"),
        otlptracegrpc.WithInsecure(),
    )
    if err != nil {
        return nil, err
    }

    res := resource.NewWithAttributes(
        semconv.SchemaURL,
        semconv.ServiceName(serviceName),
    )

    provider := sdktrace.NewTracerProvider(
        sdktrace.WithBatcher(exporter),
        sdktrace.WithResource(res),
    )

    otel.SetTracerProvider(provider)

    return func() { provider.Shutdown(ctx) }, nil
}
```

## Unified Logging Across 4+ Languages

The goal: every log line from every service can be correlated by trace ID and searched in a single place.

### Architecture

```
Service A (Python)  ──┐
Service B (Rust)   ──┤
Service C (Go)     ──┼──▶ OpenTelemetry Collector ──▶ Jaeger (traces)
Service D (JS)     ──┤                                ──▶ Loki (logs)
C Library          ──┘                                ──▶ Grafana (dashboard)
```

### Grafana Dashboard Query

```sql
-- Find all logs for a failed request (by trace ID)
{job="user-api"} | json | trace_id = "abc123def456" | line_format "{{.timestamp}} [{{.level}}] {{.service}}: {{.message}}"

-- Find all errors across services in the last hour
{job=~".*"} | json | level = "error" | timestamp >= now() - 1h

-- Find slow requests (>500ms) across all services
{job=~".*"} | json | attributes.duration_ms > 500
```

## Summary

Logging correlation in a polyglot system requires:
1. **Standardized schema** — Every service emits the same JSON structure
2. **Propagated trace ID** — Passed through HTTP headers, Kafka messages, and C libraries
3. **OpenTelemetry** — Vendor-neutral tracing that works across all languages
4. **Unified backend** — All logs searchable by trace ID in one place

The investment in logging infrastructure pays for itself the first time you trace a request across four services and find the one that's slow.
