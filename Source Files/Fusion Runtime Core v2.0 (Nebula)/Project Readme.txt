Fusion Runtime Core v2.0 (Nebula)Version: 2.0.0 "Nebula"Status: Active DevelopmentArchitecture: Micro-kernel / WASM-basedOverviewFusion Runtime Core is a hyperscale execution engine designed for distributed systems. It allows for the safe, sandboxed execution of dynamic logic (Plugins) written in Rust and compiled to WebAssembly.Project Structureserver/: The Rust-based gRPC Host.src/main.rs: Entry point.src/engine.rs: WASM execution logic (Wasmtime).proto/: Protocol Buffer definitions (.proto).Strict contract for API interactions.sdk/: Client libraries.python/: Python client for managing the core.examples/: Reference implementations.plugin/: A basic Rust plugin template.docs/: Documentation.developer_guide.md: How to build plugins.Quick StartStart the Server:cd server
cargo run
Build the Example Plugin:cd examples/plugin
cargo build --target wasm32-unknown-unknown --release
Run the SDK Demo:cd sdk/python
# Ensure proto files are generated (see Developer Guide)
python fusion_client.py
LicenseProprietary / Internal Use Only.