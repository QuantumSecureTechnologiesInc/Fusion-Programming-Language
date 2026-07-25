# Flux-Resolve Engine

**Location:** `runtime/crates/fusion_flux_resolve`
**Status:** ⚠️ Migrated to Fusion Runtime — Implementation Partial (bridge + SAT solver; CUDA/Redis/registry stubs)
**Version:** 0.3.0

## Overview

The Flux-Resolve Engine is a Fusion-native dependency resolution module relocated to the Fusion runtime workspace. The Rust bridge layer is implemented and tested. Core components:

- **SAT solver** (DPLL + VSIDS heuristics) — ✅ Fully implemented
- **Cycle detection** (Kahn's algorithm) — ✅ Fully implemented
- **CAS cache** (L1 DashMap + L2 disk, TTL) — ✅ Fully implemented
- **Version constraint solver** (SemVer) — ✅ Fully implemented
- **GPU bridge** — ⚠️ Delegates to CPU solver (no CUDA kernel)
- **Registry bridge** — 🔴 Stub (hardcoded return values)
- **`stdlib/flux_resolve.fu`** — 🔴 Not created (referenced in docs but does not exist)

## Architecture

```text
Fusion Module (stdlib/flux_resolve.fu) - Core logic in Fusion    🔴 NOT CREATED
    ↓ FFI
Rust Bridge (runtime/crates/fusion_flux_resolve) - System ops    ✅ Implemented
    ↓
OS (File I/O, GPU, Network)                                      ⚠️ Partial
```

## Components

### Rust Bridge (`fusion_flux_resolve`)

Provides FFI exports for:
- **CacheBridge** - File I/O for L2 disk cache — ✅ Working
- **GpuBridge** - GPU offloading logic — ⚠️ CPU fallback only
- **RegistryBridge** - HTTP requests to package registry — 🔴 Stub

### FFI Exports

```rust
extern "C" fn flux_resolve_bridge_create() -> *mut FluxResolveBridge
extern "C" fn flux_resolve_bridge_destroy(bridge: *mut FluxResolveBridge)
extern "C" fn flux_resolve_cache_get(...) -> *mut u8
extern "C" fn flux_resolve_cache_put(...)
```

## What's NOT Implemented (but referenced in docs)

- CUDA kernel compilation (`build.rs`, `cuda_sat_kernel_v2.cu`)
- Redis distributed CAS
- CLI binary (`flux resolve`, `flux cache clear`, `flux inspect`)
- Package registry HTTP client
- Python bindings
- Docker deployment
- `stdlib/flux_resolve.fu` Fusion module

## Building

```bash
cd runtime
cargo build -p fusion_flux_resolve
cargo test -p fusion_flux_resolve
```

## Configuration

Environment variables:
- `FUSION_CUDA_ENABLE` - Enable GPU acceleration (default: true)
- `FUSION_REGISTRY_URL` - Package registry URL

Default config:
- GPU threshold: 10,000 nodes
- VSIDS decay: 0.95
- Cache path: `.fusion/cache_db`

## See Also

- Fusion Runtime documentation
- `fusion_runtime_core` - Core runtime
- `fusion_traits` - Shared traits