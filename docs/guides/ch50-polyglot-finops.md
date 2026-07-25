# Chapter 50: Financial Cost Attribution (FinOps)

In a monolingual system, you know which service is expensive. In a polyglot system, you know which *language* is expensive — but not which *service* using that language is expensive. This chapter covers how to instrument, attribute, and optimize costs across a polyglot architecture.

## The Cost Attribution Problem

A typical polyglot microservice architecture:

```
User API (Python)     ──▶ PostgreSQL    (CPU: $200/mo, Memory: $100/mo)
Auth Service (Go)     ──▶ Redis         (CPU: $50/mo,  Memory: $30/mo)
Core Engine (Rust)    ──▶ S3            (CPU: $30/mo,  Memory: $20/mo)
ML Pipeline (Python)  ──▶ GPU Instances (CPU: $500/mo, Memory: $200/mo)
Web App (Node.js)     ──▶ CDN           (CPU: $100/mo, Memory: $50/mo)
```

The total cloud bill is $1,380/month. But which service is responsible for the GPU cost? Is the ML pipeline using 80% of the GPU, or is it 20% ML + 60% User API + 20% Core Engine? Without per-service attribution, you're guessing.

## Instrumenting CPU/Memory per Language Process

### Runtime Metrics Collection

```python
# finops/metrics_collector.py
"""Collect CPU and memory metrics per service, tagged by language."""
import psutil
import time
import json
from dataclasses import dataclass
from typing import Optional

@dataclass
class ServiceMetrics:
    service_name: str
    language: str
    cpu_percent: float
    memory_rss_mb: float
    memory_vms_mb: float
    thread_count: int
    gc_collections: int
    gc_pause_ms: float
    timestamp: str

class MetricsCollector:
    def __init__(self, service_name: str, language: str):
        self.service_name = service_name
        self.language = language
        self.process = psutil.Process()
        self.gc_stats = self._init_gc_stats()

    def collect(self) -> ServiceMetrics:
        cpu = self.process.cpu_percent(interval=1)
        mem = self.process.memory_info()
        gc_info = self._collect_gc_stats()

        return ServiceMetrics(
            service_name=self.service_name,
            language=self.language,
            cpu_percent=cpu,
            memory_rss_mb=mem.rss / 1024 / 1024,
            memory_vms_mb=mem.vms / 1024 / 1024,
            thread_count=self.process.num_threads(),
            gc_collections=gc_info["collections"],
            gc_pause_ms=gc_info["pause_ms"],
            timestamp=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        )

    def _init_gc_stats(self):
        if self.language == "python":
            import gc
            gc.set_debug(0)
            return {"prev_count": gc.get_count()}
        return {}

    def _collect_gc_stats(self):
        if self.language == "python":
            import gc
            collections = gc.get_count()[0]
            return {
                "collections": collections,
                "pause_ms": 0,  # Requires gc.callbacks or tracemalloc
            }
        return {"collections": 0, "pause_ms": 0}

# Usage
collector = MetricsCollector("user-api", "python")
metrics = collector.collect()
print(json.dumps(metrics.__dict__, indent=2))
```

### Rust Metrics

```rust
// finops/metrics.rs
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use sysinfo::System;
use chrono::Utc;

#[derive(serde::Serialize)]
pub struct ServiceMetrics {
    pub service_name: String,
    pub language: String,
    pub cpu_percent: f32,
    pub memory_rss_mb: f64,
    pub memory_vms_mb: f64,
    pub thread_count: usize,
    pub alloc_count: u64,
    pub alloc_bytes: u64,
    pub timestamp: String,
}

pub struct MetricsCollector {
    service_name: String,
}

impl MetricsCollector {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    pub fn collect(&self) -> ServiceMetrics {
        let mut sys = System::new_all();
        sys.refresh_all();

        let pid = sysinfo::get_current_pid().unwrap();
        let process = sys.process(pid).unwrap();

        ServiceMetrics {
            service_name: self.service_name.clone(),
            language: "rust".to_string(),
            cpu_percent: process.cpu_usage(),
            memory_rss_mb: process.memory() as f64 / 1024.0 / 1024.0,
            memory_vms_mb: process.virtual_memory() as f64 / 1024.0 / 1024.0,
            thread_count: process.num_threads(),
            alloc_count: ALLOC_COUNT.load(Ordering::Relaxed),
            alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

// Track allocations
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;
```

### Go Metrics

```go
// finops/metrics.go
package finops

import (
    "encoding/json"
    "fmt"
    "os"
    "runtime"
    "time"
)

type ServiceMetrics struct {
    ServiceName  string  `json:"service_name"`
    Language     string  `json:"language"`
    CPUPercent   float64 `json:"cpu_percent"`
    MemoryRssMB  float64 `json:"memory_rss_mb"`
    MemoryVmsMB  float64 `json:"memory_vms_mb"`
    ThreadCount  int     `json:"thread_count"`
    GCCount      uint32  `json:"gc_count"`
    GCPauseNs    uint64  `json:"gc_pause_ns"`
    HeapAllocMB  float64 `json:"heap_alloc_mb"`
    HeapSysMB    float64 `json:"heap_sys_mb"`
    Timestamp    string  `json:"timestamp"`
}

type MetricsCollector struct {
    serviceName string
}

func NewMetricsCollector(serviceName string) *MetricsCollector {
    return &MetricsCollector{serviceName: serviceName}
}

func (c *MetricsCollector) Collect() ServiceMetrics {
    var m runtime.MemStats
    runtime.ReadMemStats(&m)

    return ServiceMetrics{
        ServiceName: c.serviceName,
        Language:    "go",
        CPUPercent:  getCPUPercent(),
        MemoryRssMB: float64(m.Sys) / 1024 / 1024,
        MemoryVmsMB: 0, // Requires platform-specific code
        ThreadCount: runtime.NumGoroutine(),
        GCCount:     m.NumGC,
        GCPauseNs:   m.PauseNs[(m.NumGC+255)%256],
        HeapAllocMB: float64(m.HeapAlloc) / 1024 / 1024,
        HeapSysMB:   float64(m.HeapSys) / 1024 / 1024,
        Timestamp:   time.Now().UTC().Format(time.RFC3339),
    }
}

func (c *MetricsCollector) ExportJSON() string {
    metrics := c.Collect()
    data, _ := json.Marshal(metrics)
    return string(data)
}
```

## Tagging Costs by Language in Kubernetes

### Resource Quota Labels

```yaml
# kubernetes/namespace-quotas.yaml
# Tag all resources by language for cost attribution
apiVersion: v1
kind: Namespace
metadata:
  name: fusion-polyglot
  labels:
    team: fusion
    cost-center: engineering

---
# Python services
apiVersion: apps/v1
kind: Deployment
metadata:
  name: user-api
  namespace: fusion-polyglot
  labels:
    app: user-api
    language: python          # Language tag for cost tracking
    cost-tier: standard       # Cost tier for budget allocation
    team: backend
spec:
  replicas: 3
  template:
    metadata:
      labels:
        app: user-api
        language: python
        cost-tier: standard
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8000"
    spec:
      containers:
        - name: user-api
          image: fusion/user-api:latest
          resources:
            requests:
              cpu: "200m"
              memory: "256Mi"
            limits:
              cpu: "500m"
              memory: "512Mi"

---
# Rust services
apiVersion: apps/v1
kind: Deployment
metadata:
  name: core-engine
  namespace: fusion-polyglot
  labels:
    app: core-engine
    language: rust            # Language tag
    cost-tier: low            # Rust is cheaper per operation
    team: backend
spec:
  replicas: 2
  template:
    metadata:
      labels:
        app: core-engine
        language: rust
        cost-tier: low
    spec:
      containers:
        - name: core-engine
          image: fusion/core-engine:latest
          resources:
            requests:
              cpu: "100m"
              memory: "64Mi"
            limits:
              cpu: "200m"
              memory: "128Mi"

---
# Python ML pipeline (GPU)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ml-pipeline
  namespace: fusion-polyglot
  labels:
    app: ml-pipeline
    language: python
    cost-tier: premium        # GPU instances are expensive
    team: ml
spec:
  replicas: 1
  template:
    metadata:
      labels:
        app: ml-pipeline
        language: python
        cost-tier: premium
    spec:
      containers:
        - name: ml-pipeline
          image: fusion/ml-pipeline:latest
          resources:
            requests:
              cpu: "2000m"
              memory: "8Gi"
              nvidia.com/gpu: "1"
            limits:
              cpu: "4000m"
              memory: "16Gi"
              nvidia.com/gpu: "1"
```

### Cost Aggregation with Prometheus

```yaml
# prometheus/cost-rules.yml
# Aggregate costs by language and service
groups:
  - name: fusion_cost_attribution
    interval: 60s
    rules:
      # CPU cost by language
      - record: fusion:cpu_cost_by_language
        expr: |
          sum by (language) (
            rate(container_cpu_usage_seconds_total{
              namespace="fusion-polyglot"
            }[1h])
          ) * 0.04  # $0.04 per CPU-hour on AWS

      # Memory cost by language
      - record: fusion:memory_cost_by_language
        expr: |
          sum by (language) (
            container_memory_working_set_bytes{
              namespace="fusion-polyglot"
            }
          ) / 1024 / 1024 * 0.005  # $0.005 per GB-hour

      # Total cost by service
      - record: fusion:total_cost_by_service
        expr: |
          fusion:cpu_cost_by_service + fusion:memory_cost_by_service

      # Cost per request by language
      - record: fusion:cost_per_request_by_language
        expr: |
          fusion:total_cost_by_language
          / fusion:requests_by_language
```

## Cost Attribution Across Microservices

### Request-Level Cost Tracking

```python
# finops/request_cost_tracker.py
"""Track cost per request across microservices."""
import time
import json
from dataclasses import dataclass, field, asdict
from typing import List

@dataclass
class ServiceCost:
    service: str
    language: str
    cpu_ms: float
    memory_mb_seconds: float
    cost_usd: float

@dataclass
class RequestCost:
    request_id: str
    trace_id: str
    services: List[ServiceCost] = field(default_factory=list)
    total_cost_usd: float = 0.0

    def add_service(self, service: str, language: str, cpu_ms: float, memory_mb_seconds: float):
        # Cost calculation (AWS us-east-1 pricing)
        cpu_cost = (cpu_ms / 1000) * 0.0000166667  # $0.04/vCPU-hour
        memory_cost = memory_mb_seconds * 0.0000000023  # $0.0008/GB-hour

        self.services.append(ServiceCost(
            service=service,
            language=language,
            cpu_ms=cpu_ms,
            memory_mb_seconds=memory_mb_seconds,
            cost_usd=cpu_cost + memory_cost,
        ))
        self.total_cost_usd += cpu_cost + memory_cost

    def to_dict(self):
        return asdict(self)

# Middleware to track costs
class CostTrackingMiddleware:
    def __init__(self, app, service_name: str, language: str):
        self.app = app
        self.service_name = service_name
        self.language = language

    async def __call__(self, scope, receive, send):
        if scope["type"] != "http":
            await self.app(scope, receive, send)
            return

        start_time = time.perf_counter()
        start_memory = self._get_memory_mb()

        await self.app(scope, receive, send)

        elapsed_ms = (time.perf_counter() - start_time) * 1000
        end_memory = self._get_memory_mb()
        memory_mb_seconds = ((start_memory + end_memory) / 2) * (elapsed_ms / 1000)

        # Attach cost info to response headers
        cost = ServiceCost(
            service=self.service_name,
            language=self.language,
            cpu_ms=elapsed_ms,
            memory_mb_seconds=memory_mb_seconds,
            cost_usd=(elapsed_ms / 1000) * 0.0000166667 + memory_mb_seconds * 0.0000000023,
        )

        # Log cost for aggregation
        print(json.dumps({
            "type": "cost_attribution",
            "request_id": scope.get("request_id", ""),
            "service": self.service_name,
            "language": self.language,
            "cost_usd": cost.cost_usd,
            "cpu_ms": elapsed_ms,
            "memory_mb_seconds": memory_mb_seconds,
        }))
```

### Cost Dashboard Query (Grafana)

```sql
-- Total cost by language (last 30 days)
SELECT
    language,
    SUM(cost_usd) as total_cost,
    SUM(cost_usd) / COUNT(DISTINCT DATE(timestamp)) as daily_avg,
    SUM(cpu_ms) as total_cpu_ms,
    SUM(memory_mb_seconds) as total_memory_mb_s
FROM cost_attributions
WHERE timestamp >= NOW() - INTERVAL '30 days'
GROUP BY language
ORDER BY total_cost DESC;

-- Cost per request by service
SELECT
    service,
    language,
    AVG(cost_usd) as avg_cost_per_request,
    P95(cost_usd) as p95_cost_per_request,
    COUNT(*) as request_count
FROM cost_attributions
WHERE timestamp >= NOW() - INTERVAL '7 days'
GROUP BY service, language
ORDER BY avg_cost_per_request DESC;

-- Cost trend by language
SELECT
    DATE(timestamp) as date,
    language,
    SUM(cost_usd) as daily_cost
FROM cost_attributions
WHERE timestamp >= NOW() - INTERVAL '30 days'
GROUP BY date, language
ORDER BY date;
```

## Cloud Bill Optimization Strategies

### Strategy 1: Right-Size by Language

```yaml
# Optimization: Match resource requests to actual needs
# Rust needs less memory than Python for the same work

# Before: Over-provisioned Python
resources:
  requests:
    cpu: "1000m"
    memory: "2Gi"

# After: Right-sized Python
resources:
  requests:
    cpu: "200m"
    memory: "256Mi"

# Before: Over-provisioned Rust
resources:
  requests:
    cpu: "500m"
    memory: "1Gi"

# After: Right-sized Rust
resources:
  requests:
    cpu: "100m"
    memory: "64Mi"
```

### Strategy 2: Move Workloads to Cheaper Languages

```
Workload              Current Language   Optimized Language   Savings
──────────────────────────────────────────────────────────────────────
JSON validation       Python             Rust                 80% CPU
String processing     Python             Rust (SIMD)          90% CPU
HTTP routing          Node.js            Go                   60% memory
Data transformation   Python             Rust                 70% CPU
Caching layer         Python             Go                   50% memory
```

### Strategy 3: Spot Instances for Stateless Services

```yaml
# Use spot instances for stateless Python services
apiVersion: apps/v1
kind: Deployment
metadata:
  name: user-api
spec:
  template:
    spec:
      tolerations:
        - key: "spot"
          operator: "Equal"
          value: "true"
          effect: "NoSchedule"
      nodeSelector:
        node-type: "spot"
      # Spot instances: 60-90% cheaper
      # Risk: can be terminated with 2-minute notice
      # Mitigation: graceful shutdown, multiple replicas
```

### Strategy 4: Reserved Instances for Steady-State

```yaml
# Reserved instances for always-on services
# 1-year reserved: ~40% savings
# 3-year reserved: ~60% savings

# Identify candidates: services with >70% utilization over 30 days
# query: fusion:utilization_by_service{language="rust"} > 0.7

# Reserved: Core Engine (Rust), always-on, predictable load
# Reserved: Database (PostgreSQL), always-on, predictable load
# On-demand: ML Pipeline (Python), bursty, GPU-intensive
# Spot: User API (Python), stateless, fault-tolerant
```

### Strategy 5: Auto-Scaling Based on Cost Metrics

```yaml
# HorizontalPodAutoscaler with cost awareness
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: user-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: user-api
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Pods
      pods:
        metric:
          name: cost_per_request_usd
        target:
          type: AverageValue
          averageValue: "0.0001"  # Scale up if cost per request exceeds $0.0001
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 50
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

## Cost Optimization Report Template

```markdown
# Monthly FinOps Report — Fusion Polyglot System

## Period: January 2024

### Total Cost: $4,250

| Service          | Language | CPU Cost | Memory Cost | GPU Cost | Total    | % of Total |
|------------------|----------|----------|-------------|----------|----------|------------|
| ML Pipeline      | Python   | $200     | $150        | $2,100   | $2,450   | 57.6%      |
| User API         | Python   | $300     | $200        | $0       | $500     | 11.8%      |
| Core Engine      | Rust     | $50      | $30         | $0       | $80      | 1.9%       |
| Auth Service     | Go       | $80      | $50         | $0       | $130     | 3.1%       |
| Web App          | Node.js  | $150     | $100        | $0       | $250     | 5.9%       |
| Databases        | Mixed    | $200     | $300        | $0       | $500     | 11.8%      |
| Infrastructure   | Mixed    | $100     | $140        | $0       | $240     | 5.6%       |
| **Total**        |          | **$1,080** | **$970**  | **$2,100** | **$4,150** | **100%** |

### Optimization Actions

1. **ML Pipeline GPU**: Moving from on-demand to reserved instances saves $840/month (40%)
2. **User API**: Right-sizing from 1Gi to 256Mi saves $120/month
3. **Core Engine**: Already optimal (Rust is 10x cheaper than Python for CPU-bound work)

### Projected Savings: $960/month (23%)
```

## Summary

FinOps in a polyglot system requires language-level cost attribution. Tag every Kubernetes resource by language, collect per-process CPU/memory metrics, and track cost per request across service boundaries. The optimization strategies are straightforward: right-size by language, move workloads to cheaper languages where possible, and use reserved/spot instances strategically. The key insight: Rust and Go are 5-10x cheaper per operation than Python. Use them for high-throughput paths.
