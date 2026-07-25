Fusion Core (Kernel v0.1.0)Fusion Core is the foundational runtime and type system for the Fusion Programming Language. It implements the "Tri-brid" Architecture, unifying Classical, Tensor, and Quantum computing into a single, type-safe framework.This crate serves as the Language Kernel (Era 1), providing the primitives upon which the standard library and ecosystem will be built.🌟 Key Features1. Unified Type System (fusion_core::types)The FusionType enum allows the compiler to reason about interactions between paradigms at compile-time.Classical: Systems programming types (Int, Float, Struct, Vector).Tensor: N-Dimensional arrays (Tensor<T, RANK>) with compile-time shape checks and strided memory layout.Quantum: Physics-compliant types (Qubit, QuantumCircuit, QuantumGate) enforcing the No-Cloning Theorem.2. Quantum Safety & SimulationNo-Cloning Enforcement: The Qubit type does not implement Clone, ensuring physical laws are respected by the Rust borrow checker.State Simulation: Includes a sparse state-vector simulator for validating quantum logic before hardware deployment.Measurement: Explicit conversion from Quantum $\to$ Classical via measure().3. Hybrid Compiler Frontend (fusion_core::compiler)Semantic Analysis: The SemanticAnalyzer validates operations across boundaries (e.g., preventing matrix multiplication on a Qubit register).Type Checking: Enforces rank compatibility for Tensor operations.🚀 Production Example: Hybrid VQEThe crate includes a production-grade implementation of the Variational Quantum Eigensolver (VQE). This example demonstrates:Modular Architecture: Using Ansatz and Optimizer traits.Numerical Differentiation: Finite-difference gradient descent.Tri-brid Execution: Classical optimization loop driving a Quantum simulation, evaluating results via Tensor math.Running the Examplecargo run --example hybrid_vqe
Expected Output:--- Fusion Hybrid VQE (Production Grade) ---
Target: Minimize <ψ|Z|ψ>
Ansatz: Ry(theta)
Optimizer: Gradient Descent (Numerical)
---------------------------------------
|  Iter |      Energy     |
|-------|-----------------|
|   0   |    0.70710678   |
|   1   |    0.63639610   |
...
|  42   |   -0.99999500   | (Converged)
---------------------------------------
✅ Success!
Minimum Energy Found: -0.999995
📦 Architecturefusion_core/
├── types/              # The Unified Type Hierarchy
│   ├── classical.rs    # Primitives & Collections
│   ├── tensor.rs       # ND-Arrays & Strides
│   ├── quantum.rs      # Qubits, Registers & Circuits
│   └── hybrid.rs       # FusionType & Metadata
├── ops/                # Operations
│   ├── tensor_ops.rs   # Matrix Math & Reductions
│   ├── quantum_ops.rs  # Standard Gates (H, X, CNOT) & Simulation
│   └── conversions.rs  # Traits (ToQuantumState, ToTensor)
├── compiler/           # Semantic Analysis & Type Checking
└── traits.rs           # Base traits (Numeric, Unitary, Measurable)
🛠️ Integration GuideUsing Tensorsuse fusion_core::types::tensor::{Matrix, Tensor};

// Create a 2x2 Matrix (Rank 2 Tensor)
let a = Matrix::<f64>::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]);
let b = Matrix::<f64>::ones([2, 2]);

// Perform Matrix Multiplication
let c = a.matmul(&b);
Using Quantum Circuitsuse fusion_core::types::quantum::{QuantumCircuit, QuantumGate};

// Create a 2-qubit circuit
let mut circuit = QuantumCircuit::new(2);

// Apply Hadamard to q0
circuit.apply_gate(QuantumGate::hadamard(), vec![0]);

// Apply CNOT (Control: q0, Target: q1) -> Bell State
circuit.apply_gate(QuantumGate::cnot(), vec![0, 1]);

// Simulate State Vector
let state = circuit.simulate();
🧪 TestingTo run the full suite of unit and integration tests (covering all three paradigms):cargo test
📜 LicensePart of the Fusion Programming Language Ecosystem.