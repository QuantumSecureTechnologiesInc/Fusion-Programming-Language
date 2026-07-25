# Chapter 31: Part 4 — Architecture & Best Practices

> Design patterns for polyglot systems, project structure, common pitfalls, memory management across language boundaries, security isolation, and real-world migration strategies

---

## Design Patterns for Polyglot

The patterns in this chapter address recurring structural problems that appear when multiple languages share a single system. Each pattern targets a specific failure mode of polyglot architectures.

### Strangler Fig Pattern (Gradual Migration)

The strangler fig wraps a legacy system incrementally, replacing endpoints one at a time until the original system can be retired. In a polyglot context, each new endpoint is written in the language best suited for the task while the legacy system continues serving traffic.

```fusion
// route_registry.fusion — strangler fig route dispatcher
// Each route can be handled by a different language runtime

struct RouteEntry {
    path: str,
    handler_lang: str,        // "fusion", "python", "node", "go"
    handler_module: str,
    legacy_path: ?str,        // fallback to legacy system
    enabled: bool,
    migration_phase: int,     // 0 = legacy, 1 = shadow, 2 = canary, 3 = full
}

struct StranglerFig {
    routes: map<str, RouteEntry>,
    metrics: MigrationMetrics,
}

struct MigrationMetrics {
    legacy_latency_p99: f64,
    new_latency_p99: f64,
    legacy_error_rate: f64,
    new_error_rate: f64,
    traffic_pct_new: f64,
}

impl StranglerFig {
    fn dispatch(req: Request) -> Response {
        let route = self.routes.get(req.path);

        match route.migration_phase {
            0 => self.invoke_legacy(route, req),
            1 => {
                // Shadow mode: run both, return legacy result, log diff
                let legacy_result = self.invoke_legacy(route, req);
                let new_result = self.invoke_handler(route, req);
                self.metrics.log_shadow_diff(route.path, &legacy_result, &new_result);
                legacy_result
            }
            2 => {
                // Canary: small traffic fraction goes to new handler
                if self.metrics.canary_rollout(route.path) {
                    self.invoke_handler(route, req)
                } else {
                    self.invoke_legacy(route, req)
                }
            }
            3 => self.invoke_handler(route, req),
            _ => panic!("invalid migration phase: {}", route.migration_phase),
        }
    }

    fn invoke_handler(route: &RouteEntry, req: Request) -> Response {
        match route.handler_lang {
            "fusion" => fusion::invoke(&route.handler_module, req),
            "python" => py_runtime::invoke(&route.handler_module, req),
            "node"   => js_runtime::invoke(&route.handler_module, req),
            "go"     => go_runtime::invoke(&route.handler_module, req),
            lang     => panic!("unsupported handler language: {}", lang),
        }
    }

    fn invoke_legacy(route: &RouteEntry, req: Request) -> Response {
        let legacy_path = route.legacy_path.as_ref()
            .expect("legacy path required for phases 0-2");
        http::forward(legacy_path, req)
    }
}
```

**Phased rollout progression:**

| Phase | Traffic Split | Purpose |
|-------|--------------|---------|
| 0 — Legacy | 100% legacy | Baseline, no new code serving traffic |
| 1 — Shadow | 100% legacy, 100% new (shadow) | Validate correctness without user impact |
| 2 — Canary | 1-10% new, 90-99% legacy | Measure latency/error deltas under load |
| 3 — Full | 100% new | Legacy decommissioned for this route |

### Anti-Corruption Layer (ACL)

The ACL isolates legacy or foreign system data models from your domain model. It translates between the external API's representation and your internal representation, preventing legacy schemas from leaking into new code.

```fusion
// acl_legacy_payment.fusion
// Isolates the legacy SOAP payment API behind a clean domain interface

use fusion::net::http_client;

struct LegacyPaymentACL {
    endpoint: str,
    auth_token: str,
    timeout_ms: u64,
}

// Your domain model — clean, typed, intentional
struct PaymentRequest {
    amount: Money,
    currency: CurrencyCode,
    recipient: AccountId,
    idempotency_key: str,
}

struct PaymentResult {
    transaction_id: TransactionId,
    status: PaymentStatus,
    confirmed_at: ?DateTime,
}

// The legacy system's model — messy, stringly-typed, full of quirks
struct LegacySoapPayload {
    amt: str,              // string-encoded decimal "123.45"
    ccy: str,              // 3-letter ISO via different codes
    rcpt_acct: str,        // unpadded account number
    rcpt_bank: str,        // SWIFT/BIC code
    ref_id: str,           // idempotency key mapped to their ref
    sign: str,             // HMAC signature computed over concatenated fields
}

impl LegacyPaymentACL {
    // Translate domain model → legacy model
    fn to_legacy(req: &PaymentRequest) -> LegacySoapPayload {
        LegacySoapPayload {
            amt: req.amount.to_string(2),    // "123.45"
            ccy: Self::map_currency(req.currency),
            rcpt_acct: format!("{:0>12}", req.recipient.0),
            rcpt_bank: req.recipient.bank_code.clone(),
            ref_id: req.idempotency_key.clone(),
            sign: Self::compute_hmac(req),
        }
    }

    // Translate legacy model → domain model
    fn from_legacy(resp: LegacySoapResponse) -> PaymentResult {
        PaymentResult {
            transaction_id: TransactionId(resp.txn_ref.parse().unwrap_or_default()),
            status: match resp.status_code.as_str() {
                "00" => PaymentStatus::Confirmed,
                "01" => PaymentStatus::Pending,
                "02" => PaymentStatus::Failed(resp.error_msg.clone()),
                _    => PaymentStatus::Unknown(resp.status_code.clone()),
            },
            confirmed_at: resp.timestamp.parse().ok(),
        }
    }

    fn map_currency(c: CurrencyCode) -> str {
        // Legacy system uses non-standard codes
        match c {
            CurrencyCode::USD => "840",
            CurrencyCode::EUR => "978",
            CurrencyCode::GBP => "826",
            _ => panic!("unsupported currency for legacy system: {:?}", c),
        }
    }

    async fn execute(&self, req: PaymentRequest) -> Result<PaymentResult, PaymentError> {
        let legacy_req = Self::to_legacy(&req);
        let soap_body = Self::serialize_soap(&legacy_req);

        let resp = http_client::post(&self.endpoint)
            .header("Authorization", &format!("Bearer {}", self.auth_token))
            .header("Content-Type", "text/xml; charset=utf-8")
            .timeout(Duration::from_millis(self.timeout_ms))
            .body(soap_body)
            .send()
            .await
            .map_err(|e| PaymentError::Network(e.to_string()))?;

        let legacy_resp = Self::parse_soap_response(resp.body())
            .map_err(|e| PaymentError::Parse(e.to_string()))?;

        Ok(Self::from_legacy(legacy_resp))
    }
}
```

### API Gateway Pattern (Unified Entry Point)

A single entry point routes requests to the correct language runtime based on path, header, or content-type. Every external consumer talks to one endpoint; internal polyglot routing is invisible.

```fusion
// gateway.fusion

struct Gateway {
    routes: Vec<RouteBinding>,
    middleware: Vec<Middleware>,
    rate_limiter: RateLimiter,
    auth_provider: AuthProvider,
}

struct RouteBinding {
    prefix: str,
    runtime: RuntimeTarget,
    strip_prefix: bool,
    timeout_ms: u64,
}

enum RuntimeTarget {
    FusionModule(str),
    PythonWasm(str),
    NodeProcess(str),
    GoBinary(str),
    ExternalUrl(str),
}

impl Gateway {
    async fn handle(&self, req: Request) -> Response {
        // Apply middleware chain
        let mut ctx = RequestContext::from(req);
        for mw in &self.middleware {
            ctx = mw.process(ctx).await?;
        }

        // Match route
        let route = self.routes.iter()
            .find(|r| ctx.path.starts_with(&r.prefix))
            .ok_or_else(|| GatewayError::RouteNotFound(ctx.path.clone()))?;

        // Rate limit
        self.rate_limiter.check(&ctx.client_id)?;

        // Authenticate
        let identity = self.auth_provider.authenticate(&ctx).await?;

        // Delegate to runtime with timeout
        let result = tokio::time::timeout(
            Duration::from_millis(route.timeout_ms),
            self.invoke_runtime(&route.runtime, ctx, &identity),
        ).await;

        match result {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => Response::error(502, format!("upstream error: {}", e)),
            Err(_) => Response::error(504, "gateway timeout".into()),
        }
    }

    fn invoke_runtime(
        &self,
        target: &RuntimeTarget,
        ctx: RequestContext,
        identity: &Identity,
    ) -> impl Future<Output = Result<Response, GatewayError>> {
        async move {
            match target {
                RuntimeTarget::FusionModule(path) =>
                    fusion::invoke(path, ctx).await,
                RuntimeTarget::PythonWasm(path) =>
                    py_wasm::invoke(path, ctx).await,
                RuntimeTarget::NodeProcess(path) =>
                    node_ipc::invoke(path, ctx).await,
                RuntimeTarget::GoBinary(path) =>
                    go_exec::invoke(path, ctx).await,
                RuntimeTarget::ExternalUrl(url) =>
                    http::proxy(url, ctx).await,
            }
        }
    }
}
```

### Event-Driven Architecture (Loose Coupling)

Languages communicate through events rather than direct function calls. This decouples lifecycles — a Python service can be replaced without touching Fusion code, as long as both publish and consume the same event schema.

```fusion
// event_bus.fusion

struct EventEnvelope {
    id: EventId,
    topic: str,
    schema_version: u32,
    timestamp: DateTime,
    source_lang: str,
    payload: Vec<u8>,       // serialized with schema-aware encoder
    metadata: map<str, str>,
}

trait EventHandler: Send + Sync {
    fn handle(&self, event: &EventEnvelope) -> Result<(), EventError>;
}

struct EventBus {
    handlers: map<str, Vec<Box<dyn EventHandler>>>,
    serializers: SerializerRegistry,
}

impl EventBus {
    fn publish<T: Serializable>(&self, topic: &str, event: &T, source: &str) -> EventEnvelope {
        let payload = self.serializers.encode(topic, event);
        EventEnvelope {
            id: EventId::generate(),
            topic: topic.to_string(),
            schema_version: T::SCHEMA_VERSION,
            timestamp: DateTime::now(),
            source_lang: source.to_string(),
            payload,
            metadata: map::new(),
        }
    }

    fn subscribe(&mut self, topic: &str, handler: Box<dyn EventHandler>) {
        self.handlers.entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    async fn dispatch(&self, event: EventEnvelope) -> Result<(), EventError> {
        let handlers = self.handlers.get(&event.topic)
            .ok_or_else(|| EventError::NoSubscribers(event.topic.clone()))?;

        for handler in handlers {
            handler.handle(&event)?;
        }
        Ok(())
    }
}

// Schema registry prevents cross-language deserialization mismatches
struct SerializerRegistry {
    schemas: map<str, SchemaEntry>,
}

struct SchemaEntry {
    version: u32,
    fusion_type_id: TypeId,
    python_type_name: str,
    node_type_name: str,
    go_type_name: str,
    // Versioned serializers per language
    encoders: map<str, Box<dyn Encoder>>,
    decoders: map<str, Box<dyn Decoder>>,
}
```

### CQRS (Command Query Responsibility Segregation)

CQRS separates write models (commands) from read models (queries), which is especially useful in polyglot systems where different languages have different optimization profiles for reads vs writes.

```fusion
// cqrs_boundary.fusion

// Write side — Fusion (strongly typed, safe)
struct CommandHandler {
    store: EventStore,
    validator: CommandValidator,
}

impl CommandHandler {
    fn handle(cmd: &dyn Command) -> Result<Vec<Event>, CommandError> {
        self.validator.validate(cmd)?;

        let events = match cmd.type_name() {
            "CreateOrder" => self.handle_create_order(cmd.downcast_ref::<CreateOrder>().unwrap()),
            "AddLineItem" => self.handle_add_line_item(cmd.downcast_ref::<AddLineItem>().unwrap()),
            "SubmitPayment" => self.handle_submit_payment(cmd.downcast_ref::<SubmitPayment>().unwrap()),
            _ => return Err(CommandError::UnknownCommand(cmd.type_name().into())),
        };

        self.store.append(events.clone())?;
        Ok(events)
    }
}

// Read side — Python (optimized for analytics and search)
// This runs in a separate Python process, consuming events via IPC

// Python read model consumer (runs in a Python subprocess):
// class ReadModelConsumer:
//     def __init__(self, event_stream, search_index, analytics_db):
//         self.event_stream = event_stream
//         self.search_index = search_index
//         self.analytics_db = analytics_db
//
//     def process(self, event):
//         match event.topic:
//             case "order.created":
//                 self.search_index.index_order(event.payload)
//                 self.analytics_db.record_order(event.payload)
//             case "order.submitted":
//                 self.search_index.update_status(event.payload)
//                 self.analytics_db.record_payment(event.payload)

// Query side — Node.js (optimized for JSON serialization to frontend)
// Node process handles read queries via HTTP, queries the same read store
```

### Circuit Breaker (Fault Isolation)

When a foreign language runtime or external service fails, the circuit breaker stops sending requests to it, preventing cascading failures across the polyglot boundary.

```fusion
// circuit_breaker.fusion

enum CircuitState {
    Closed,        // Normal operation — requests pass through
    Open,          // Failure detected — requests blocked, fallback used
    HalfOpen,      // Trial period — one request allowed through to test recovery
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    open_duration: Duration,
    last_failure_at: ?DateTime,
    fallback: Box<dyn Fn(&dyn Any) -> Result<Box<dyn Any>, BreakerError>>,
}

impl CircuitBreaker {
    fn new(config: BreakerConfig) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold: config.failure_threshold,
            success_threshold: config.success_threshold,
            open_duration: config.open_duration,
            last_failure_at: None,
            fallback: config.fallback,
        }
    }

    fn call<F, T>(&mut self, action: F) -> Result<T, BreakerError>
    where
        F: FnOnce() -> Result<T, BreakerError>,
        T: 'static,
    {
        match self.state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitState::HalfOpen;
                    self.attempt_call(action)
                } else {
                    // Circuit is open — invoke fallback
                    Err(self.invoke_fallback())
                }
            }
            CircuitState::HalfOpen | CircuitState::Closed => self.attempt_call(action),
        }
    }

    fn attempt_call<F, T>(&mut self, action: F) -> Result<T, BreakerError>
    where
        F: FnOnce() -> Result<T, BreakerError>,
        T: 'static,
    {
        match action() {
            Ok(val) => self.on_success(val),
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }

    fn on_success<T>(&mut self, val: T) -> Result<T, BreakerError> {
        self.failure_count = 0;
        self.success_count += 1;

        if self.state == CircuitState::HalfOpen && self.success_count >= self.success_threshold {
            self.state = CircuitState::Closed;
            self.success_count = 0;
        }
        Ok(val)
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.success_count = 0;
        self.last_failure_at = Some(DateTime::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    fn should_attempt_reset(&self) -> bool {
        self.last_failure_at.map(|t| t.elapsed() >= self.open_duration)
            .unwrap_or(false)
    }

    fn invoke_fallback(&self) -> BreakerError {
        // Fallback is pre-configured: cached response, default value, error message
        BreakerError::CircuitOpen("service unavailable, fallback engaged".into())
    }
}

// Usage: wrapping a cross-language call
fn call_python_ml_model(input: &Tensor) -> Result<Prediction, BreakerError> {
    static mut BREAKER: Option<CircuitBreaker> = None;

    unsafe {
        let breaker = BREAKER.get_or_insert_with(|| {
            CircuitBreaker::new(BreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                open_duration: Duration::from_secs(30),
                fallback: Box::new(|_| Ok(Box::new(Prediction::default()))),
            })
        });

        breaker.call(|| {
            py_runtime::invoke("ml.predict", input.clone())
                .map_err(|e| BreakerError::Upstream(e.to_string()))
        })
    }
}
```

### Sidecar Pattern (Companion Services)

A sidecar runs alongside a service, providing cross-cutting concerns (logging, monitoring, TLS, secret rotation) without embedding those concerns in the service code. Each language runtime can have its own sidecar implementation.

```fusion
// sidecar_registry.fusion

struct SidecarSpec {
    name: str,
    language: str,         // Language of the companion process
    image: str,            // Container image or binary path
    ports: Vec<PortMapping>,
    shared_volume: str,    // IPC via shared memory or Unix socket
    health_check: HealthCheck,
}

struct SidecarRegistry {
    sidecars: map<str, SidecarInstance>,
}

struct SidecarInstance {
    spec: SidecarSpec,
    pid: Option<u32>,
    status: SidecarStatus,
    started_at: ?DateTime,
}

enum SidecarStatus {
    Stopped,
    Starting,
    Running,
    Failed(str),
}

impl SidecarRegistry {
    fn attach(service_name: &str, sidecar: SidecarSpec) -> Result<(), SidecarError> {
        let instance = SidecarInstance {
            spec: sidecar,
            pid: None,
            status: SidecarStatus::Stopped,
            started_at: None,
        };
        self.sidecars.insert(service_name.to_string(), instance);
        Ok(())
    }

    fn start_all(&mut self) -> Result<(), SidecarError> {
        for (name, instance) in &mut self.sidecars {
            Self::start_sidecar(name, instance)?;
        }
        Ok(())
    }

    fn start_sidecar(name: &str, instance: &mut SidecarInstance) -> Result<(), SidecarError> {
        match instance.spec.language {
            "rust" => {
                let child = std::process::Command::new(&instance.spec.image)
                    .arg("--sidecar-mode")
                    .arg("--ipc-path")
                    .arg(&instance.spec.shared_volume)
                    .spawn()
                    .map_err(|e| SidecarError::Spawn(e.to_string()))?;
                instance.pid = Some(child.id());
                instance.status = SidecarStatus::Running;
                instance.started_at = Some(DateTime::now());
            }
            "python" => {
                let child = std::process::Command::new("python3")
                    .arg(&instance.spec.image)
                    .arg("--sidecar")
                    .arg("--ipc-path")
                    .arg(&instance.spec.shared_volume)
                    .spawn()
                    .map_err(|e| SidecarError::Spawn(e.to_string()))?;
                instance.pid = Some(child.id());
                instance.status = SidecarStatus::Running;
                instance.started_at = Some(DateTime::now());
            }
            lang => return Err(SidecarError::UnsupportedLanguage(lang.into())),
        }

        log::info!("started sidecar '{}' for service '{}'", instance.spec.name, name);
        Ok(())
    }
}

// Common sidecar responsibilities (language-agnostic):
// - mTLS termination
// - Log collection and forwarding
// - Metrics collection (Prometheus, StatsD)
// - Secret rotation polling
// - Health check aggregation
// - Distributed tracing propagation
```

---

## Project Structure Best Practices

### Monorepo vs Polyrepo

The choice between monorepo and polyrepo determines how much coordination overhead you accept in exchange for atomic refactoring.

**Monorepo** — all languages live in one repository:

```
fusion-project/
├── BUILD                       # Root build configuration
├── fusion/
│   ├── src/
│   ├── tests/
│   └── BUILD
├── services/
│   ├── payment/
│   │   ├── fusion/             # Fusion payment logic
│   │   ├── python/             # ML fraud detection
│   │   │   ├── pyproject.toml
│   │   │   └── src/
│   │   ├── node/               # Webhook handlers
│   │   │   ├── package.json
│   │   │   └── src/
│   │   └── go/                 # High-perf settlement
│   │       ├── go.mod
│   │       └── cmd/
│   └── auth/
│       ├── fusion/
│       └── python/             # Token introspection
├── shared/
│   ├── schemas/                # Language-neutral schema definitions
│   │   ├── payment.event.schema.json
│   │   └── auth.token.schema.json
│   ├── proto/                  # Protobuf definitions
│   │   └── payment.proto
│   └── ffi-headers/            # Auto-generated FFI bindings
│       ├── payment.py.h
│       └── payment.go.h
├── infra/
│   ├── docker/
│   └── k8s/
└── tools/
    ├── schema-gen/             # Schema code generator
    └── ffi-gen/                # FFI binding generator
```

**Polyrepo** — each language/service owns its repository:

```
# Separate repos with shared schema registry
fusion-payment-fusion/     # Pure Fusion payment service
fusion-payment-python/     # ML fraud detection microservice
fusion-payment-node/       # Webhook handlers
fusion-payment-go/         # High-perf settlement
fusion-shared-schemas/     # Central schema definitions (consumed by all)
fusion-ffi-bindings/       # Auto-generated cross-language bindings
```

| Factor | Monorepo | Polyrepo |
|--------|----------|----------|
| Atomic cross-language refactors | Yes | No (requires coordination) |
| Build time isolation | Harder | Natural |
| Dependency versioning | Unified | Per-service, risk of drift |
| CI/CD complexity | Single pipeline, complex | Per-repo, simpler pipelines |
| Code ownership clarity | Needs CODEOWNERS | Natural per-repo |
| Schema evolution | Easy to coordinate | Requires schema registry |

### Module Organization by Domain

Group code by domain, not by language. Within a domain, use language subdirectories.

```
services/
├── billing/
│   ├── fusion/          # Domain logic, orchestration
│   │   ├── invoice.gen.fusion
│   │   ├── subscription.gen.fusion
│   │   └── BUILD
│   ├── python/          # Tax calculation (complex rate tables)
│   │   ├── tax_calc.py
│   │   ├── rates_2026.json
│   │   └── pyproject.toml
│   ├── go/              # Ledger writes (performance-critical)
│   │   ├── ledger.go
│   │   └── go.mod
│   └── tests/
│       ├── integration/
│       └── property/     # Cross-language property tests
├── notifications/
│   ├── fusion/          # Routing, channel selection
│   ├── python/          # NLP for message personalization
│   └── node/            # Real-time WebSocket delivery
```

### Shared vs Language-Specific Code

```fusion
// shared_types.fusion
// These types are the source of truth for cross-language data contracts

#[schema_version(3)]
struct Invoice {
    id: InvoiceId,
    account_id: AccountId,
    line_items: Vec<LineItem>,
    total: Money,
    currency: CurrencyCode,
    issued_at: DateTime,
    due_at: DateTime,
    status: InvoiceStatus,
}

#[schema_version(2)]
struct LineItem {
    description: str,
    quantity: u32,
    unit_price: Money,
    tax_rate: f64,
}
```

```python
# shared/schemas/python/invoice.py
# Auto-generated from schema definitions — do not edit manually
# Generator: fusion-schema-gen v2.4.0
# Source: shared_types.fusion

from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal

@dataclass(frozen=True)
class Invoice:
    id: str
    account_id: str
    line_items: tuple[LineItem, ...]  # frozen = immutable
    total: Decimal
    currency: str
    issued_at: datetime
    due_at: datetime
    status: str

@dataclass(frozen=True)
class LineItem:
    description: str
    quantity: int
    unit_price: Decimal
    tax_rate: float
```

### Configuration Management

```toml
# fusion.toml — language-agnostic config with language-specific sections

[project]
name = "fusion-payment"
version = "2.4.1"

[project.fusion]
target = "x86_64"
optimization = "release"
features = ["simd", "async-io"]

[project.python]
version = "3.11"
virtual_env = ".venv"
packages = ["numpy", "pandas", "scikit-learn"]

[project.node]
version = "20"
package_manager = "pnpm"

[project.go]
version = "1.22"
mod_path = "github.com/example/fusion-payment"

[service.payment]
port = 8080
log_level = "info"

[service.payment.fusion]
modules = ["invoice", "subscription", "routing"]

[service.payment.python]
module = "fraud_detection"
entry_point = "src/fraud/main.py"
workers = 4

[service.payment.go]
binary = "cmd/settlement/main.go"
workers = 8

[cross_language]
schema_dir = "shared/schemas"
ffi_header_dir = "shared/ffi-headers"
serialization = "msgpack"
schema_registry_url = "http://localhost:8800"
```

### Secret Management

```fusion
// secrets.fusion
// Secrets are never stored in code or config files.
// They are injected at runtime via environment variables, vault APIs, or sidecar injection.

struct SecretManager {
    backend: SecretBackend,
    cache: SecretCache,
}

enum SecretBackend {
    EnvVar,                    // Reads from environment variables
    VaultHttp(String),         // HashiCorp Vault HTTP API
    AwsSecretsManager(String), // AWS Secrets Manager
    Sidecar(String),           // Unix socket to sidecar process
}

impl SecretManager {
    async fn get(&self, key: &str) -> Result<Secret, SecretError> {
        // Check cache first (secrets are cached for a short TTL)
        if let Some(cached) = self.cache.get(key) {
            return Ok(cached);
        }

        let secret = match &self.backend {
            SecretBackend::EnvVar => {
                let val = std::env::var(key)
                    .map_err(|_| SecretError::NotFound(key.into()))?;
                Secret::from_string(val)
            }
            SecretBackend::VaultHttp(addr) => {
                let resp = http::get(&format!("{}/v1/secret/data/{}", addr, key))
                    .header("X-Vault-Token", &self.vault_token())
                    .send().await
                    .map_err(|e| SecretError::Backend(e.to_string()))?;
                Secret::from_vault_response(resp.body())
            }
            SecretBackend::AwsSecretsManager(arn) => {
                aws::secrets_manager::get_secret(arn, key).await
                    .map_err(|e| SecretError::Backend(e.to_string()))?
            }
            SecretBackend::Sidecar(path) => {
                unix_socket::request(path, key).await
                    .map_err(|e| SecretError::Backend(e.to_string()))?
            }
        };

        self.cache.insert(key, secret.clone(), Duration::from_secs(300));
        Ok(secret)
    }
}
```

---

## Common Pitfalls & Anti-patterns

### The "Everything in One Language" Trap

**Symptom:** Team forces all new code into a single language despite clear mismatches between language strengths and task requirements.

**Example:** Writing a data pipeline in Fusion when Python's NumPy/Pandas ecosystem would handle the matrix math in 10% of the code.

**Fix:** Evaluate each module against language-task fit. The cost of interop is real but often less than reimplementing ecosystem capabilities.

### Over-engineering Interoperability

**Symptom:** Building custom RPC frameworks, serialization layers, or service meshes before any cross-language call exists.

**Fix:** Start with direct FFI or IPC. Introduce abstraction only when you have three or more language boundaries to manage.

### Ignoring Build Times

**Symptom:** CI builds take 45 minutes because all language runtimes compile sequentially.

```fusion
// BAD: Sequential build
// build_all.sh:
//   cargo build --release
//   python -m build
//   go build ./...
//   tsc

// GOOD: Parallel build with dependency tracking
struct BuildGraph {
    nodes: Vec<BuildTarget>,
    edges: Vec<(BuildTarget, BuildTarget)>,  // dependency edges
}

impl BuildGraph {
    async fn build_parallel(&self) -> BuildResult {
        let topo = self.topological_sort();
        let levels = self.parallel_levels(&topo);

        for (i, level) in levels.iter().enumerate() {
            log::info!("build phase {}/{}: {:?} (parallel)", i + 1, levels.len(), level);

            let results: Vec<_> = level.iter()
                .map(|target| self.build_target(target))
                .collect();

            for result in results {
                result?;
            }
        }
        Ok(BuildResult::Success)
    }
}
```

### Neglecting Error Handling Across Boundaries

**Symptom:** A Python exception in an FFI call crashes the entire Fusion process because the error wasn't caught at the boundary.

```fusion
// BAD: No error handling at boundary
extern "python" {
    fn ml_predict(input: *const u8) -> *const u8;  // Python can raise, Fusion assumes it won't
}

// GOOD: Boundary-aware error handling
#[ffi(error_handling = "catch_all")]
extern "python" {
    fn ml_predict(input: *const u8) -> FfiResult<*const u8>;
}

// FfiResult captures both success and failure, including
// the foreign language's error type and stack trace
enum FfiResult<T> {
    Ok(T),
    ForeignError {
        language: str,
        error_type: str,
        message: str,
        stack_trace: ?str,
    },
}
```

### Version Lock-in

**Symptom:** Upgrading Python from 3.9 to 3.12 breaks FFI bindings because the CPython ABI changed.

**Fix:** Pin language versions per service. Test upgrades in shadow mode before switching production traffic.

### Debugging Nightmares

**Symptom:** Stack trace shows frames in Fusion, Python, and C simultaneously. No single debugger handles all three.

```fusion
// Mitigation: structured cross-language tracing
struct CrossLangTrace {
    trace_id: TraceId,
    spans: Vec<TraceSpan>,
}

struct TraceSpan {
    span_id: SpanId,
    parent_id: ?SpanId,
    language: str,
    module: str,
    function: str,
    start_time: DateTime,
    end_time: ?DateTime,
    status: SpanStatus,
    attributes: map<str, str>,
}

// Every FFI boundary crossing logs a span
fn ffi_boundary_cross(target_lang: &str, target_fn: &str) -> SpanId {
    let span = TraceSpan {
        span_id: SpanId::generate(),
        parent_id: current_span_id(),
        language: target_lang,
        module: String::new(),
        function: target_fn.to_string(),
        start_time: DateTime::now(),
        end_time: None,
        status: SpanStatus::Ok,
        attributes: map::new(),
    };
    trace_log::push(span);
    span.span_id
}
```

### Memory Leaks at FFI Boundaries

**Symptom:** Each FFI call allocates memory in the foreign runtime. Neither runtime frees it because each assumes the other is responsible.

```fusion
// BAD: Ownership ambiguity
extern "cdecl" {
    fn get_buffer() -> *mut u8;  // Who frees this?
}

// GOOD: Explicit ownership transfer
extern "cdecl" {
    fn alloc_buffer(size: u64) -> *mut u8;     // Caller owns
    fn free_buffer(ptr: *mut u8);               // Caller must call this
    fn get_buffer_len(ptr: *const u8) -> u64;   // Query size
}

// EVEN BETTER: RAII wrapper
struct ForeignBuffer {
    ptr: *mut u8,
    len: u64,
}

impl Drop for ForeignBuffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { free_buffer(self.ptr); }
        }
    }
}

fn get_data() -> Result<ForeignBuffer, FfiError> {
    let ptr = unsafe { get_buffer() };
    if ptr.is_null() {
        return Err(FfiError::NullPointer);
    }
    let len = unsafe { get_buffer_len(ptr) };
    Ok(ForeignBuffer { ptr, len })
    // ForeignBuffer is dropped here, calling free_buffer
}
```

### Performance Cliffs at Language Switches

**Symptom:** A tight loop that calls into Python 10,000 times per second is 100x slower than expected due to GIL contention and serialization overhead.

```fusion
// BAD: Per-element FFI call
for element in &data {
    let result = py_runtime::invoke("ml.process_element", element.clone())?;
    results.push(result);
}

// GOOD: Batch across the boundary
let batch = BatchView::new(&data, 1024);  // 1024 elements per batch
for chunk in batch {
    let batch_results = py_runtime::invoke("ml.process_batch", chunk.to_python_batch())?;
    results.extend(batch_results);
}
```

---

## Memory Management & Ownership Handoffs

This section addresses the most dangerous class of bugs in polyglot systems: memory ownership confusion at language boundaries.

### Who Owns the Memory? (GC vs Borrow Checker)

Each language runtime has a different memory management model. At boundaries, you must explicitly transfer ownership.

| Language | Model | Who Frees? |
|----------|-------|-----------|
| Fusion | Ownership + borrow checker | Deterministic (RAII) |
| C/C++ | Manual / RAII | Developer must call free/destructor |
| Python | Reference counting + tracing GC | GC, but only within Python runtime |
| Node.js | V8 GC (tracing) | GC, but only within V8 runtime |
| Go | Concurrent tracing GC | GC, but only within Go runtime |
| Rust (extern) | Ownership + borrow checker | Deterministic |

**Rule:** When passing memory across a language boundary, exactly one side must free it. Define this explicitly.

```fusion
// Ownership transfer protocol
//
// Convention: FFI memory is freed by the side that allocated it.
// For Fusion→C: Fusion calls C's free function.
// For C→Fusion: Fusion takes ownership and drops it.
// For Fusion→Python: Reference count is incremented, Python owns it.
// For Python→Fusion: Fusion takes a reference or copies the data.

// Fusion allocating for C
fn send_to_c(data: &[u8]) {
    let c_buf = unsafe { libc::malloc(data.len()) as *mut u8 };
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), c_buf, data.len());
    }
    // C function takes ownership — it must call libc::free(c_buf)
    unsafe { c_process(c_buf, data.len()); }
}

// Python allocating for Fusion — PyO3 style
// #[pyfunction]
// fn process_data<'py>(py: Python<'py>, input: &PyBytes) -> PyResult<&'py PyBytes> {
//     // Returns a reference — Python still owns it, Fusion borrows it
//     let result = fusion_process(input.as_bytes());
//     Ok(PyBytes::new(py, &result))
// }
```

### Memory Layout Mismatches (Struct Padding/Alignment)

Structures defined in different languages may have different memory layouts due to compiler-specific padding rules.

```fusion
// BAD: Assumed matching layout between Fusion and C
#[repr(C)]  // This is correct — forces C-compatible layout
struct PacketHeader {
    version: u8,     // offset 0
    // 3 bytes padding inserted by compiler
    length: u32,     // offset 4 (not 1!)
    flags: u16,      // offset 8
    // 2 bytes padding
    sequence: u32,   // offset 12
}
// Total: 16 bytes, not 11

// Verify layout at compile time
const _: () = assert!(std::mem::size_of::<PacketHeader>() == 16);
const _: () = assert!(std::mem::align_of::<PacketHeader>() == 4);
```

### repr(C) and @Struct Annotations

```fusion
// @Struct forces C-compatible layout with explicit packing control
@Struct(packing = 1)  // No padding between fields
struct PackedHeader {
    version: u8,      // offset 0
    length: u32,      // offset 1 (no padding!)
    flags: u16,       // offset 5
    sequence: u32,    // offset 7
}
// Total: 11 bytes

@Struct(alignment = 8)  // Force 8-byte alignment
struct AlignedBlock {
    data: [u8; 5],
}
// Total: 16 bytes (5 bytes + 3 padding to align to 8, then padded to next 8)

// Verify at compile time
const _: () = assert!(std::mem::size_of::<PackedHeader>() == 11);

// Cross-language layout verification
// This runs in CI to catch layout drift between Fusion and C/Python definitions
#[test]
fn verify_packet_layout() {
    let header = PacketHeader {
        version: 1,
        length: 1024,
        flags: 0x01,
        sequence: 42,
    };

    let bytes = unsafe {
        std::slice::from_raw_parts(
            &header as *const PacketHeader as *const u8,
            std::mem::size_of::<PacketHeader>(),
        )
    };

    // Verify field offsets match the C struct definition
    assert_eq!(bytes[0], 1);                    // version
    assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 1024);  // length
    assert_eq!(u16::from_le_bytes(bytes[8..10].try_into().unwrap()), 1);    // flags
    assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 42);  // sequence
}
```

### Reference Counting vs Tracing GC

When sharing objects between Fusion (ownership-based) and Python (reference counting + tracing GC), you need a bridge that satisfies both memory models.

```fusion
// py_bridge.fusion — reference-counted bridge for Python objects
// This mirrors PyO3's approach but is implemented in pure Fusion

struct PyRef {
    ptr: *mut PyObject,    // Python object pointer
    runtime: Arc<PyRuntime>,
}

impl PyRef {
    fn new(ptr: *mut PyObject, runtime: Arc<PyRuntime>) -> Self {
        unsafe { Py_IncRef(ptr); }  // Increment Python's refcount
        PyRef { ptr, runtime }
    }
}

impl Clone for PyRef {
    fn clone(&self) -> Self {
        unsafe { Py_IncRef(self.ptr); }  // Increment refcount on clone
        PyRef {
            ptr: self.ptr,
            runtime: self.runtime.clone(),
        }
    }
}

impl Drop for PyRef {
    fn drop(&mut self) {
        unsafe { Py_DecRef(self.ptr); }  // Decrement refcount — Python GC may collect
    }
}

// Safety: PyRef is !Send because Python objects are thread-local
// To send across threads, use PySendRef which acquires the GIL
struct PySendRef {
    inner: PyRef,
}

unsafe impl Send for PySendRef {}

impl PySendRef {
    fn send_to_thread(self) -> PyRef {
        // Must acquire GIL before using the object in another thread
        self.inner.runtime.with_gil(|py| {
            // Object is now safe to use in the new thread
            self.inner
        })
    }
}
```

### FFI Memory Safety Patterns

```fusion
// Pattern 1: Borrow with lifetime across FFI
// Safe when the foreign side guarantees the pointer remains valid

extern "cdecl" {
    fn get_string_table() -> *const StringTable;
    fn string_table_lookup(table: *const StringTable, id: u32) -> *const u8;
}

struct BorrowedStr<'a> {
    ptr: *const u8,
    _marker: PhantomData<&'a str>,
}

impl<'a> BorrowedStr<'a> {
    fn as_str(&self) -> Option<&'a str> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let cstr = std::ffi::CStr::from_ptr(self.ptr as *const i8);
            cstr.to_str().ok()
        }
    }
}

fn lookup_string(table: &StringTable, id: u32) -> Option<BorrowedStr<'_>> {
    let ptr = unsafe { string_table_lookup(table as *const StringTable, id) };
    Some(BorrowedStr {
        ptr,
        _marker: PhantomData,
    })
}

// Pattern 2: Copy across boundary (safe, but slower)
fn copy_string_from_c(ptr: *const u8) -> Result<String, FfiError> {
    if ptr.is_null() {
        return Err(FfiError::NullPointer);
    }
    unsafe {
        let cstr = std::ffi::CStr::from_ptr(ptr as *const i8);
        Ok(cstr.to_str()
            .map_err(|_| FfiError::InvalidUtf8)?
            .to_string())
    }
}

// Pattern 3: Arena allocator for batch operations
struct FfiArena {
    allocations: Vec<*mut u8>,
}

impl FfiArena {
    fn alloc(&mut self, size: usize) -> *mut u8 {
        let ptr = unsafe { libc::malloc(size) };
        self.allocations.push(ptr);
        ptr
    }

    fn reset(&mut self) {
        for ptr in self.allocations.drain(..) {
            unsafe { libc::free(ptr); }
        }
    }
}

impl Drop for FfiArena {
    fn drop(&mut self) {
        self.reset();
    }
}
```

### Preventing Memory Leaks at Boundaries

```fusion
// Pattern: Leak detector for FFI allocations
struct FfiLeakDetector {
    allocations: map<*mut u8, AllocationInfo>,
    enabled: bool,
}

struct AllocationInfo {
    size: usize,
    allocated_at: str,  // source location
    freed: bool,
}

impl FfiLeakDetector {
    fn track_alloc(&mut self, ptr: *mut u8, size: usize) {
        if !self.enabled { return; }
        self.allocations.insert(ptr, AllocationInfo {
            size,
            allocated_at: std::panic::Location::caller().to_string(),
            freed: false,
        });
    }

    fn track_free(&mut self, ptr: *mut u8) {
        if !self.enabled { return; }
        if let Some(info) = self.allocations.get_mut(&ptr) {
            info.freed = true;
        }
    }

    fn report_leaks(&self) -> Vec<&AllocationInfo> {
        self.allocations.values()
            .filter(|info| !info.freed)
            .collect()
    }
}

// Integrate with FFI calls
extern "cdecl" {
    fn alloc_buffer(size: u64) -> *mut u8;
    fn free_buffer(ptr: *mut u8);
}

fn safe_alloc(size: usize) -> *mut u8 {
    let ptr = unsafe { alloc_buffer(size as u64) };
    FFI_LEAK_DETECTOR.lock().track_alloc(ptr, size);
    ptr
}

fn safe_free(ptr: *mut u8) {
    FFI_LEAK_DETECTOR.lock().track_free(ptr);
    unsafe { free_buffer(ptr); }
}
```

---

## Security Boundaries Between Languages

### Input Sanitization at Boundaries

Every data crossing a language boundary must be validated. The receiving language should not trust the sending language's invariants.

```fusion
// boundary_validator.fusion

struct BoundaryValidator {
    rules: Vec<ValidationRule>,
}

enum ValidationRule {
    MaxLength(str, usize),
    Regex(str, str),
    TypeCheck(str, fn(&dyn Any) -> bool),
    Sanitize(str, Sanitizer),
}

struct Sanitizer {
    strip_html: bool,
    strip_control_chars: bool,
    max_utf8_length: usize,
    allowed_char_categories: Vec<CharCategory>,
}

impl BoundaryValidator {
    fn validate_cross_boundary<T: Serializable>(
        &self,
        data: &T,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<T, ValidationError> {
        let serialized = data.serialize();
        let mut cleaned = serialized.clone();

        for rule in &self.rules {
            match rule {
                ValidationRule::MaxLength(field, max) => {
                    if let Some(val) = cleaned.get(field) {
                        if val.as_str().map(|s| s.len()) > Some(*max) {
                            return Err(ValidationError::FieldTooLong(field.clone(), *max));
                        }
                    }
                }
                ValidationRule::Sanitize(field, sanitizer) => {
                    if let Some(val) = cleaned.get_mut(field) {
                        if let Some(s) = val.as_str() {
                            let clean = sanitizer.sanitize(s);
                            *val = serde_json::Value::String(clean);
                        }
                    }
                }
                ValidationRule::Regex(field, pattern) => {
                    if let Some(val) = cleaned.get(field) {
                        if let Some(s) = val.as_str() {
                            if !regex::Regex::new(pattern).unwrap().is_match(s) {
                                return Err(ValidationError::PatternMismatch(field.clone()));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        T::deserialize(&cleaned)
            .map_err(|e| ValidationError::DeserializationFailed(e.to_string()))
    }
}

impl Sanitizer {
    fn sanitize(&self, input: &str) -> String {
        let mut result = input.to_string();

        if self.strip_html {
            result = html_escape::strip_tags(&result);
        }

        if self.strip_control_chars {
            result = result.chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                .collect();
        }

        if result.len() > self.max_utf8_length {
            result.truncate(self.max_utf8_length);
        }

        result
    }
}
```

### Privilege Separation (Sandboxing Unsafe Code)

Foreign code runs in reduced-privilege sandboxes. A vulnerability in a Python dependency cannot compromise the Fusion core.

```fusion
// sandbox.fusion

struct SandboxConfig {
    max_memory_bytes: u64,
    max_cpu_time_ms: u64,
    max_file_descriptors: u32,
    allowed_syscalls: Vec<Syscall>,
    network_policy: NetworkPolicy,
    filesystem_policy: FilesystemPolicy,
}

struct Sandbox {
    config: SandboxConfig,
    pid: u32,
    resource_usage: ResourceUsage,
}

impl Sandbox {
    fn spawn_sandboxed(config: SandboxConfig, binary: &str, args: &[&str]) -> Result<Sandbox, SandboxError> {
        // On Linux, use seccomp + namespaces
        // On macOS, use sandbox-exec profiles
        // On Windows, use restricted token + job objects

        #[cfg(target_os = "linux")]
        {
            let child = unsafe {
                let pid = libc::fork();
                if pid == 0 {
                    // Child process — apply sandbox
                    Self::apply_seccomp(&config.allowed_syscalls);
                    Self::apply_resource_limits(&config);
                    libc::execvp(
                        binary.as_ptr() as *const libc::c_char,
                        Self::to_c_args(binary, args).as_ptr(),
                    );
                    libc::_exit(1);
                }
                pid
            };

            Ok(Sandbox {
                config,
                pid: child as u32,
                resource_usage: ResourceUsage::new(),
            })
        }
    }

    fn apply_seccomp(allowed: &[Syscall]) {
        // Build seccomp filter allowing only listed syscalls
        // All other syscalls trigger SIGSYS (kill the process)
        let filter = SeccompFilter::new(allowed);
        unsafe { seccomp::apply_filter(&filter); }
    }

    fn apply_resource_limits(config: &SandboxConfig) {
        unsafe {
            libc::setrlimit(libc::RLIMIT_AS, &libc::rlimit {
                rlim_cur: config.max_memory_bytes,
                rlim_max: config.max_memory_bytes,
            });
            libc::setrlimit(libc::RLIMIT_CPU, &libc::rlimit {
                rlim_cur: config.max_cpu_time_ms / 1000,
                rlim_max: config.max_cpu_time_ms / 1000,
            });
            libc::setrlimit(libc::RLIMIT_NOFILE, &libc::rlimit {
                rlim_cur: config.max_file_descriptors as u64,
                rlim_max: config.max_file_descriptors as u64,
            });
        }
    }
}
```

### Capability-Based Security for Interop

Instead of trusting all code equally, each language runtime receives only the capabilities it needs.

```fusion
// capabilities.fusion

struct Capability {
    resource: Resource,
    actions: Vec<Action>,
    constraints: Vec<Constraint>,
}

enum Resource {
    Network(String),     // Host/port
    Filesystem(String),  // Path prefix
    Memory(u64),         // Max bytes
    Cpu(Duration),       // Max compute time
    Database(String),    // Connection string
}

enum Action {
    Read, Write, Execute, Listen, Connect, Fork,
}

struct CapabilitySet {
    capabilities: Vec<Capability>,
}

impl CapabilitySet {
    fn check(&self, resource: &Resource, action: &Action) -> Result<(), CapabilityDenied> {
        let matching = self.capabilities.iter()
            .find(|cap| cap.resource == *resource && cap.actions.contains(action));

        match matching {
            Some(cap) => {
                // Check constraints
                for constraint in &cap.constraints {
                    constraint.evaluate()?;
                }
                Ok(())
            }
            None => Err(CapabilityDenied {
                resource: resource.clone(),
                action: action.clone(),
            }),
        }
    }
}

// Example: Python service gets network access only to the ML model API
let python_caps = CapabilitySet {
    capabilities: vec![
        Capability {
            resource: Resource::Network("ml-model.internal:8080".into()),
            actions: vec![Action::Connect],
            constraints: vec![],
        },
        Capability {
            resource: Resource::Filesystem("/tmp/python-workspace".into()),
            actions: vec![Action::Read, Action::Write],
            constraints: vec![],
        },
        Capability {
            resource: Resource::Memory(512 * 1024 * 1024),  // 512MB
            actions: vec![],
            constraints: vec![],
        },
    ],
};
```

### Memory Safety Guarantees Across Languages

```fusion
// memory_safety_contract.fusion
// Defines what each language guarantees about memory safety

struct MemorySafetyContract {
    source_lang: str,
    guarantees: Vec<MemoryGuarantee>,
    verified_by: str,     // "compile-time", "runtime", "manual-review"
}

enum MemoryGuarantee {
    NoBufferOverflow,
    NoUseAfterFree,
    NoDoubleFree,
    NoUninitializedRead,
    NoDataRaces,
    BoundedAllocation { max_bytes: u64 },
    NoAliasingMutability,  // No &mut aliasing (Rust/Fusion guarantee)
}

// At each FFI boundary, document which guarantees hold
fn ffi_boundary_contract(source: &str, target: &str) -> Vec<MemorySafetyContract> {
    match (source, target) {
        ("fusion", "c") => vec![
            MemorySafetyContract {
                source_lang: "fusion".into(),
                guarantees: vec![
                    MemoryGuarantee::NoBufferOverflow,
                    MemoryGuarantee::NoUseAfterFree,
                    MemoryGuarantee::NoUninitializedRead,
                ],
                verified_by: "compile-time".into(),
            },
        ],
        ("fusion", "python") => vec![
            MemorySafetyContract {
                source_lang: "fusion".into(),
                guarantees: vec![
                    MemoryGuarantee::NoBufferOverflow,
                    MemoryGuarantee::BoundedAllocation { max_bytes: 1024 * 1024 * 100 },
                ],
                verified_by: "compile-time".into(),
            },
            MemorySafetyContract {
                source_lang: "python".into(),
                guarantees: vec![
                    MemoryGuarantee::NoBufferOverflow,   // Python bounds-checks arrays
                    MemoryGuarantee::NoUseAfterFree,     // GC prevents this
                ],
                verified_by: "runtime".into(),
            },
        ],
        ("c", "fusion") => vec![
            MemorySafetyContract {
                source_lang: "c".into(),
                guarantees: vec![],  // No guarantees — C is unsafe
                verified_by: "manual-review".into(),
            },
        ],
        _ => vec![],
    }
}
```

---

## Code Examples

### Strangler Fig Migration Example

Complete working example of migrating a user service from a legacy Node.js API to Fusion.

```fusion
// strangler_user_service.fusion

use fusion::net::{Request, Response, Router};
use fusion::ipc;

struct UserServiceMigration {
    node_legacy: ipc::ChildProcess,   // Legacy Node.js process
    fusion_new: FusionUserService,    // New Fusion implementation
    config: MigrationConfig,
    metrics: MigrationMetrics,
}

struct MigrationConfig {
    node_binary: str,
    legacy_entry: str,
    shadow_mode: bool,
    canary_pct: f64,              // 0.0 - 1.0
    feature_flags: map<str, bool>,
}

impl UserServiceMigration {
    fn setup(config: MigrationConfig) -> Result<Self, MigrationError> {
        let node_legacy = ipc::ChildProcess::spawn(config.node_binary, &[
            "--entry", config.legacy_entry,
            "--port", "0",  // Random port, we'll discover it
        ])?;

        let fusion_new = FusionUserService::new();

        Ok(UserServiceMigration {
            node_legacy,
            fusion_new,
            config,
            metrics: MigrationMetrics::new(),
        })
    }

    async fn handle_request(&mut self, req: Request) -> Response {
        match req.path.as_str() {
            // Phase 1: Shadow mode — run both, compare
            p if p.starts_with("/api/users") && self.config.shadow_mode => {
                let legacy_result = self.call_node_legacy(&req).await;
                let fusion_result = self.fusion_new.handle(&req).await;

                // Compare results
                match self.compare_results(&legacy_result, &fusion_result) {
                    ComparisonResult::Match => {
                        self.metrics.record_match(&req.path);
                        legacy_result  // Return legacy result
                    }
                    ComparisonResult::Mismatch(diff) => {
                        self.metrics.record_mismatch(&req.path, &diff);
                        log::warn!("shadow mismatch on {}: {}", req.path, diff);
                        legacy_result  // Still return legacy, but alert
                    }
                }
            }

            // Phase 2: Canary — small % goes to new
            p if p.starts_with("/api/users") && self.config.canary_pct > 0.0 => {
                let hash = self.consistent_hash(&req);
                if (hash % 1000) < (self.config.canary_pct * 1000.0) as u64 {
                    self.metrics.record_canary_hit(&req.path);
                    match self.fusion_new.handle(&req).await {
                        Ok(resp) => resp,
                        Err(e) => {
                            log::error!("canary failure, falling back to legacy: {}", e);
                            self.call_node_legacy(&req).await
                        }
                    }
                } else {
                    self.call_node_legacy(&req).await
                }
            }

            // Phase 3: Full cutover
            _ => {
                self.metrics.record_full_cutover(&req.path);
                match self.fusion_new.handle(&req).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        log::error!("full cutover failure: {}", e);
                        // Last resort fallback to legacy
                        self.call_node_legacy(&req).await
                    }
                }
            }
        }
    }

    fn compare_results(&self, a: &Response, b: &Response) -> ComparisonResult {
        if a.status == b.status && a.body == b.body {
            ComparisonResult::Match
        } else {
            ComparisonResult::Mismatch(format!(
                "status: {} vs {}, body length: {} vs {}",
                a.status, b.status, a.body.len(), b.body.len()
            ))
        }
    }

    fn consistent_hash(&self, req: &Request) -> u64 {
        let key = format!("{}:{}", req.path, req.headers.get("x-user-id").unwrap_or(&""));
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&key, &mut hasher);
        std::hash::Hasher::finish(&hasher)
    }

    async fn call_node_legacy(&mut self, req: &Request) -> Response {
        let serialized = serde_json::to_vec(req).unwrap();
        let result = self.node_legacy.call("handleRequest", &serialized).await;
        serde_json::from_slice(&result).unwrap_or_else(|_| Response::error(500, "parse error"))
    }
}
```

### Anti-Corruption Layer Implementation

Complete ACL wrapping a legacy SOAP payment service.

```fusion
// acl_complete_example.fusion

use fusion::net::http_client;
use fusion::crypto::hmac;

// --- Domain Models (clean, Fusion-native) ---

struct PaymentDomain {
    id: PaymentId,
    amount: Decimal,
    currency: CurrencyCode,
    recipient: AccountId,
    sender: AccountId,
    memo: Option<String>,
    created_at: DateTime,
    status: PaymentStatus,
}

enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed(PaymentFailureReason),
    Reversed,
}

enum PaymentFailureReason {
    InsufficientFunds,
    InvalidAccount,
    NetworkTimeout,
    ComplianceHold,
    Unknown(String),
}

// --- Legacy Models (what the SOAP API actually uses) ---

struct LegacySoapRequest {
    // Field names reflect the actual SOAP schema — messy, abbreviated
    pymt_amt: String,         // "1234567" (cents, string-encoded)
    pymt_ccy: String,         // "840" (ISO numeric code, string)
    rcpt_acct_num: String,    // "000123456789" (zero-padded)
    rcpt_bank_bic: String,    // "BOFAUS3N" (BIC code)
    sndr_acct_num: String,
    sndr_bank_bic: String,
    pymt_ref: String,         // Idempotency key mapped to their reference
    pymt_desc: String,
    sign: String,             // HMAC-SHA256 over concatenated fields
    sign_ts: String,          // "20260724120000Z" (compact timestamp)
}

struct LegacySoapResponse {
    resp_code: String,     // "00" = success, "01" = pending, "02" = failed
    resp_msg: String,
    txn_ref: String,
    settle_date: String,   // "20260725" (compact date)
    fee_amt: String,       // Fee in cents, string-encoded
}

// --- ACL Implementation ---

struct PaymentACL {
    endpoint: String,
    signing_key: Vec<u8>,
    client: http_client::Client,
    timeout: Duration,
}

impl PaymentACL {
    fn new(config: ACLConfig) -> Self {
        PaymentACL {
            endpoint: config.legacy_endpoint,
            signing_key: config.signing_key,
            client: http_client::Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs))
                .build(),
            timeout: Duration::from_secs(config.timeout_secs),
        }
    }

    // Domain → Legacy translation
    fn to_legacy(&self, domain: &PaymentDomain) -> Result<LegacySoapRequest, ACLError> {
        Ok(LegacySoapRequest {
            pymt_amt: (domain.amount * 100.0).to_integral().to_string(),
            pymt_ccy: Self::currency_to_numeric(domain.currency)?,
            rcpt_acct_num: format!("{:0>12}", domain.recipient.0),
            rcpt_bank_bic: domain.recipient.bank_bic.clone(),
            sndr_acct_num: format!("{:0>12}", domain.sender.0),
            sndr_bank_bic: domain.sender.bank_bic.clone(),
            pymt_ref: domain.id.to_string(),
            pymt_desc: domain.memo.clone().unwrap_or_default(),
            sign: String::new(),  // Computed below
            sign_ts: domain.created_at.format("%Y%m%d%H%M%SZ"),
        })
    }

    // Compute HMAC signature for legacy API
    fn sign_request(&self, req: &mut LegacySoapRequest) {
        let payload = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            req.pymt_amt, req.pymt_ccy, req.rcpt_acct_num,
            req.rcpt_bank_bic, req.sndr_acct_num, req.pymt_ref,
            req.sign_ts,
        );
        req.sign = hmac::sign(&self.signing_key, payload.as_bytes());
    }

    // Legacy → Domain translation
    fn from_legacy(&self, resp: LegacySoapResponse) -> Result<PaymentDomain, ACLError> {
        let status = match resp.resp_code.as_str() {
            "00" => PaymentStatus::Completed,
            "01" => PaymentStatus::Processing,
            "02" => PaymentStatus::Failed(
                PaymentFailureReason::Unknown(resp.resp_msg.clone())
            ),
            code => PaymentStatus::Failed(
                PaymentFailureReason::Unknown(format!("unknown code: {}", code))
            ),
        };

        Ok(PaymentDomain {
            id: PaymentId::from_string(&resp.txn_ref)?,
            amount: Decimal::ZERO,  // Not in response, caller retains
            currency: CurrencyCode::USD,  // Caller retains
            recipient: AccountId::from_string("0")?,  // Caller retains
            sender: AccountId::from_string("0")?,
            memo: None,
            created_at: DateTime::now(),
            status,
        })
    }

    fn currency_to_numeric(c: CurrencyCode) -> Result<String, ACLError> {
        match c {
            CurrencyCode::USD => Ok("840".into()),
            CurrencyCode::EUR => Ok("978".into()),
            CurrencyCode::GBP => Ok("826".into()),
            CurrencyCode::JPY => Ok("392".into()),
            _ => Err(ACLError::UnsupportedCurrency(c)),
        }
    }

    // Execute the full translation and API call
    async fn execute(&self, domain: PaymentDomain) -> Result<PaymentDomain, ACLError> {
        let mut legacy_req = self.to_legacy(&domain)?;
        self.sign_request(&mut legacy_req);

        let soap_body = Self::build_soap_envelope(&legacy_req);
        let resp = self.client.post(&self.endpoint)
            .header("Content-Type", "text/xml; charset=utf-8")
            .header("SOAPAction", "\"ProcessPayment\"")
            .body(soap_body)
            .send()
            .await
            .map_err(|e| ACLError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ACLError::HttpError(resp.status()));
        }

        let legacy_resp = Self::parse_soap_response(resp.body())?;
        let mut result = self.from_legacy(legacy_resp)?;
        result.amount = domain.amount;
        result.currency = domain.currency;
        result.recipient = domain.recipient;
        result.sender = domain.sender;

        Ok(result)
    }
}
```

### Circuit Breaker Pattern (Full Implementation)

Complete circuit breaker with metrics, fallbacks, and monitoring.

```fusion
// circuit_breaker_complete.fusion

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// --- Configuration ---

struct BreakerConfig {
    failure_threshold: u32,      // Failures before opening circuit
    success_threshold: u32,      // Successes to close circuit from half-open
    open_duration: Duration,     // How long to stay open before half-open
    half_open_max_calls: u32,    // Max calls allowed in half-open state
    fallback_timeout: Duration,  // Timeout for fallback execution
}

impl Default for BreakerConfig {
    fn default() -> Self {
        BreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            open_duration: Duration::from_secs(30),
            half_open_max_calls: 1,
            fallback_timeout: Duration::from_secs(5),
        }
    }
}

// --- State ---

enum CircuitState {
    Closed {
        failure_count: u32,
        success_count: u32,
        last_failure: Option<Instant>,
    },
    Open {
        opened_at: Instant,
    },
    HalfOpen {
        success_count: u32,
        total_calls: u32,
    },
}

// --- Metrics ---

struct BreakerMetrics {
    total_calls: u64,
    total_successes: u64,
    total_failures: u64,
    total_rejected: u64,
    state_transitions: Vec<(CircuitStateLabel, Instant)>,
}

enum CircuitStateLabel {
    Closed, Open, HalfOpen,
}

// --- Main Breaker ---

struct CircuitBreaker {
    config: BreakerConfig,
    state: CircuitState,
    metrics: BreakerMetrics,
    name: String,
}

impl CircuitBreaker {
    fn new(name: &str, config: BreakerConfig) -> Self {
        CircuitBreaker {
            config,
            state: CircuitState::Closed {
                failure_count: 0,
                success_count: 0,
                last_failure: None,
            },
            metrics: BreakerMetrics {
                total_calls: 0,
                total_successes: 0,
                total_failures: 0,
                total_rejected: 0,
                state_transitions: vec![],
            },
            name: name.to_string(),
        }
    }

    fn call<F, R, E>(&mut self, action: F) -> Result<R, BreakerError<E>>
    where
        F: FnOnce() -> Result<R, E>,
    {
        self.metrics.total_calls += 1;

        // Check if we should allow the call
        match &self.state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.config.open_duration {
                    log::info!("[{}] transitioning from Open to HalfOpen", self.name);
                    self.transition_to(CircuitState::HalfOpen {
                        success_count: 0,
                        total_calls: 0,
                    });
                    // Allow through for trial
                } else {
                    self.metrics.total_rejected += 1;
                    return Err(BreakerError::CircuitOpen);
                }
            }
            CircuitState::HalfOpen { total_calls, .. } => {
                if *total_calls >= self.config.half_open_max_calls {
                    self.metrics.total_rejected += 1;
                    return Err(BreakerError::CircuitOpen);
                }
            }
            CircuitState::Closed { .. } => {}
        }

        // Execute the action
        match action() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(BreakerError::Upstream(e))
            }
        }
    }

    fn on_success(&mut self) {
        self.metrics.total_successes += 1;

        match &mut self.state {
            CircuitState::Closed { success_count, failure_count, .. } => {
                *success_count += 1;
                *failure_count = 0;  // Reset consecutive failures
            }
            CircuitState::HalfOpen { success_count, total_calls } => {
                *success_count += 1;
                *total_calls += 1;

                if *success_count >= self.config.success_threshold {
                    log::info!("[{}] HalfOpen → Closed after {} successes", self.name, success_count);
                    self.transition_to(CircuitState::Closed {
                        failure_count: 0,
                        success_count: 0,
                        last_failure: None,
                    });
                }
            }
            CircuitState::Open { .. } => unreachable!("should not receive success while open"),
        }
    }

    fn on_failure(&mut self) {
        self.metrics.total_failures += 1;

        match &mut self.state {
            CircuitState::Closed { failure_count, success_count, last_failure } => {
                *failure_count += 1;
                *success_count = 0;
                *last_failure = Some(Instant::now());

                if *failure_count >= self.config.failure_threshold {
                    log::warn!("[{}] Closed → Open after {} failures", self.name, failure_count);
                    self.transition_to(CircuitState::Open {
                        opened_at: Instant::now(),
                    });
                }
            }
            CircuitState::HalfOpen { total_calls, .. } => {
                *total_calls += 1;
                log::warn!("[{}] HalfOpen → Open (trial call failed)", self.name);
                self.transition_to(CircuitState::Open {
                    opened_at: Instant::now(),
                });
            }
            CircuitState::Open { .. } => unreachable!("should not receive failure while open"),
        }
    }

    fn transition_to(&mut self, new_state: CircuitState) {
        let label = match &new_state {
            CircuitState::Closed { .. } => CircuitStateLabel::Closed,
            CircuitState::Open { .. } => CircuitStateLabel::Open,
            CircuitState::HalfOpen { .. } => CircuitStateLabel::HalfOpen,
        };
        self.metrics.state_transitions.push((label, Instant::now()));
        self.state = new_state;
    }

    fn is_available(&self) -> bool {
        match &self.state {
            CircuitState::Closed { .. } => true,
            CircuitState::Open { opened_at } => opened_at.elapsed() >= self.config.open_duration,
            CircuitState::HalfOpen { .. } => true,
        }
    }
}

// --- Usage Example ---

fn call_external_api(breaker: &mut CircuitBreaker) -> Result<String, BreakerError<ApiError>> {
    breaker.call(|| {
        // This closure only executes if the circuit allows it
        let resp = http_client::get("https://api.example.com/data")
            .timeout(Duration::from_secs(5))
            .send()
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(resp.body().to_string())
        } else {
            Err(ApiError::Http(resp.status()))
        }
    })
}

// Example with fallback
fn call_ml_model_with_fallback(
    breaker: &mut CircuitBreaker,
    input: &Tensor,
) -> Prediction {
    match breaker.call(|| {
        py_runtime::invoke("ml.predict", input.clone())
            .map_err(|e| BreakerError::Upstream(ApiError::Python(e.to_string())))
    }) {
        Ok(prediction) => prediction,
        Err(BreakerError::CircuitOpen) => {
            log::warn!("ML model unavailable, using default prediction");
            Prediction::default()  // Fallback: return neutral prediction
        }
        Err(BreakerError::Upstream(e)) => {
            log::error!("ML model error: {}", e);
            Prediction::default()
        }
    }
}
```

---

## Summary

This chapter covered the architectural foundations for polyglot systems:

- **Design patterns** — strangler fig, ACL, API gateway, event-driven, CQRS, circuit breaker, sidecar
- **Project structure** — monorepo vs polyrepo, domain-based organization, shared schema management
- **Pitfalls** — FFI memory leaks, performance cliffs, debugging across runtimes, version lock-in
- **Memory management** — ownership transfer protocols, layout verification, reference counting bridges, leak detection
- **Security** — input sanitization at boundaries, sandboxing, capability-based access, memory safety contracts

Each pattern and example targets a specific failure mode. Apply them selectively — the cost of infrastructure should match the complexity of the polyglot system you are building.

---

> **Navigation**: [Previous: Part 3 — Advanced Patterns](ch30-polyglot-advanced-patterns.md) | [Next: Part 5 — Case Studies](ch32-polyglot-case-studies.md)
