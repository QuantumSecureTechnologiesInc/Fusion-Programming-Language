# HAFT Quick Reference

## Installation
```bash
cd "C:\Projects\Fusion - Programming Language"
cargo build -p haft_fusion
```

## CLI Usage
```bash
# Basic
haft-fusion --shape 100,100,100

# With options
haft-fusion -s 256,256,64 -l 1000000 -t 1.5
```

## API Quick Start
```rust
use haft_fusion::*;
use std::sync::Arc;

// Create tensor
let tensor = Arc::new(FluxTensor::new(vec![1024, 768]));

// Set values
tensor.set(vec![10, 20], 3.14);

// Get values
let val = tensor.get(&[10, 20]); // Some(3.14)

// Spawn agents
spawn_agents(tensor.clone()).await;
```

## Key Metrics
| Metric            | Value              |
| ----------------- | ------------------ |
| Memory savings    | 90-99%             |
| Cache hit rate    | 70-99%             |
| Compression speed | 50ms/100k elements |

## Agent Reference
| Agent      | Interval | Purpose                         |
| ---------- | -------- | ------------------------------- |
| Researcher | 5s       | Statistics & anomaly detection  |
| Builder    | 10s      | Memory management & compression |
| Optimizer  | 15s      | Access pattern optimization     |

## Common Patterns

### ML Training
```rust
let weights = Arc::new(FluxTensor::new(vec![10000, 5000]));
// 10% hot storage for active layers
spawn_agents(weights).await;
```

### Analytics
```rust
let events = Arc::new(FluxTensor::new(vec![1000000]));
// Recent events hot, historical cold
spawn_agents(events).await;
```

## Configuration Tips
- **Hot Limit**: 10-20% of tensor size for ML
- **Variance Threshold**: 1.0-2.0 for stable computations
- **Intervals**: Default optimal for 1M-100M elements
