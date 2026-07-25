> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 18: Hybrid Quantum-Classical Algorithms

In Chapter 17, we introduced Quantum Computing in isolation. However, the most powerful near-term applications of quantum computers are **hybrid**: they combine classical processing (for optimization, data handling, and control flow) with quantum processing (for evaluating complex cost functions or sampling probability distributions).

Fusion is uniquely positioned for this because it integrates both paradigms in a single language, runtime, and type system.

In this chapter, we will cover:
- variational quantum algorithms (VQE, QAOA).
- Quantum Machine Learning (QML).
- Passing data between `Tensor` (AI) and `Qubit` (Quantum) types.

---

## 18.1 Variational Quantum Eigensolver (VQE)

VQE is an algorithm used to find the ground state energy of a molecule (or minimum eigenvalue of a Hamiltonian). It uses a classical optimizer to "tune" the parameters of a quantum circuit.

### 18.1.1 The Ansible (Parameterized Circuit)

First, we define a quantum circuit that depends on classical parameters (angles). This is called an **ansatz**.

```fusion
use fusion_quantum_sdk::{QuantumCircuit, QubitRegister}
use fusion_ai_core::{Tensor, Device}
use std::f64::consts::PI

fn hardware_efficient_ansatz(num_qubits: i32, depth: i32, params: &Tensor) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(num_qubits)
    let param_data = params.to_vec_f64() // Move tensor data to CPU for circuit construction
    let mut idx = 0

    for _ in 0..depth {
        // Ry rotations on all qubits
        for q in 0..num_qubits {
            circuit.ry(q, param_data[idx])
            circuit.rz(q, param_data[idx + 1])
            idx += 2
        }

        // Entangling layer (CNOT ring)
        for q in 0..num_qubits-1 {
            circuit.cnot(q, q + 1)
        }
    }

    circuit
}
```text

### 18.1.2 The Hamiltonian (Cost Function)

We define the problem we are solving. For H2 molecule at bond distance 0.74A:

```fusion
use fusion_quantum_sdk::Observable

fn h2_hamiltonian() -> Observable {
    Observable::from_pauli_sum(vec![
        ("II", -1.052),
        ("IZ",  0.397),
        ("ZI", -0.397),
        ("ZZ", -0.011),
        ("XX",  0.180),
    ])
}
```text

### 18.1.3 The Optimization Loop

Here's the hybrid magic. We use Fusion's AI optimizer (typically used for Neural Networks) to optimize the Quantum Circuit!

```fusion
use fusion_ai_core::optim::Adam
use fusion_quantum_sdk::Simulator

fn main() {
    let num_qubits = 2
    let depth = 2
    let num_params = num_qubits * 2 * depth

    // 1. Initialize parameters as a Tensor requiring gradients
    let mut params = Tensor::randn(&[num_params], Device::Cpu).requires_grad(true)

    // 2. Setup Classical Optimizer
    let mut optimizer = Adam::new(&[&params], 0.1) // learning rate 0.1
    let hamiltonian = h2_hamiltonian()
    let sim = Simulator::new()

    for epoch in 0..100 {
        optimizer.zero_grad()

        // 3. Quantum Forward Pass (calculating Expectation Value)
        // Fusion supports "autograd" through quantum circuits via parameter shift rule!
        let energy = sim.expectation_value_autograd(
            |p| hardware_efficient_ansatz(num_qubits, depth, p), // Circuit builder closure
            &params,
            &hamiltonian
        )

        println!("Epoch {}: Energy = {:.6}", epoch, energy.item::<f64>())

        // 4. Backward Pass (calculates gradients combining quantum and classical chains)
        energy.backward()

        // 5. Update Parameters
        optimizer.step()
    }
}
```text

This code is revolutionary. In other languages, you'd need PyTorch + Qiskit + a bridge library. In Fusion, `Tensor` and `QuantumCircuit` speak the same autodiff language.

---

## 18.2 Quantum Approximate Optimization Algorithm (QAOA)

QAOA is used for combinatorial optimization (like MaxCut).

### 18.2.1 Mixing and Cost Hamiltonians

QAOA alternates between two Hamiltonians: $H_C$ (Cost) and $H_B$ (Mixer).

```fusion
fn qaoa_circuit(graph: &Graph, gammas: &[f64], betas: &[f64], p: i32) -> QuantumCircuit {
    let n = graph.num_nodes()
    let mut circuit = QuantumCircuit::new(n)

    // Initial superposition
    for i in 0..n { circuit.h(i) }

    for layer in 0..p {
        // Cost Hamiltonian (ZZ interactions for edges)
        let gamma = gammas[layer]
        for (u, v) in graph.edges() {
            circuit.cnot(u, v)
            circuit.rz(v, gamma)
            circuit.cnot(u, v)
        }

        // Mixer Hamiltonian (X rotations)
        let beta = betas[layer]
        for i in 0..n {
            circuit.rx(i, 2.0 * beta)
        }
    }

    circuit.measure_all()
    circuit
}
```text

The rest of the optimization loop looks very similar to VQE.

---

## 18.3 Quantum Machine Learning (QML)

We can embed quantum layers inside classical neural networks.

### 18.3.1 Quantum Layer as a Module

We define a struct that implements the `Module` trait from `fusion_ai_core`.

```fusion
use fusion_ai_core::nn::Module

struct QuantumLayer {
    params: Tensor,
    simulator: Simulator,
}

impl Module for QuantumLayer {
    fn forward(&self, inputs: &Tensor) -> Tensor {
        // Encode classical input data into quantum state (Data Encoding)
        // Apply parameterized variational circuit (Trainable Weights)
        // Measure expectation values (Decode to Classical)

        // This function returns a Tensor resulting from the quantum measurement
        // connected to the computation graph.
        self.simulator.run_hybrid_batch(inputs, &self.params)
    }
}
```text

### 18.3.2 The Hybrid Neural Network

```fusion
use fusion_ai_core::nn

struct HybridClassifier {
    fc1: nn::Linear,
    q_layer: QuantumLayer,
    fc2: nn::Linear,
}

impl HybridClassifier {
    fn new() -> Self {
        Self {
            fc1: nn::Linear::new(784, 4), // Reduce MNIST 784 -> 4 dims
            q_layer: QuantumLayer::new(4), // 4 Qubits processing
            fc2: nn::Linear::new(4, 10),   // 4 -> 10 classes
        }
    }

    fn forward(&self, x: &Tensor) -> Tensor {
        let x = self.fc1.forward(x).relu() // Classical reduction
        let x = self.q_layer.forward(&x)   // Quantum processing
        let x = self.fc2.forward(&x)       // Classical classification
        x
    }
}
```text

This allows us to leverage quantum Hilbert space for feature processing while keeping standard classical interfaces for I/O.

---

## 18.4 Handling Shot Noise

Real quantum computers are probabilistic. When running QML on real hardware (not statevector simulation), gradients are estimated via sampling ("shots").

Fusion provides statistical utilities to handle this variance.

```fusion
let options = SamplingOptions {
    shots: 1000,
    noise_model: NoiseModel::IBMQ_Brisbane,
}

// Automatic handling of variance in the optimizer
optimizer.set_stochastic(true)
```text

---

## 18.5 Summary

Hybrid Quantum-Classical computing is a key workflow in the Fusion v2.0 Vortex ecosystem.
- **Unified Autograd**: Gradients flow seamlessly between Classical Tensors and Quantum Parameters (via parameter-shift rules).
- **VQE/QAOA**: Standard algorithms are easy to implement using the AI optimizer library.
- **QML**: Quantum circuits can act as layers in neural networks.
- **Tri-brid**: We just combined Classical control flow + AI Optimization + Quantum Circuits.

This capability makes Fusion the premier language for quantum research and development.

---

## 18.6 Exercises

1. **MaxCut**: Implement MaxCut solver using QAOA for a simple 5-node graph.
2. **Quantum Kernel**: Instead of a variational circuit, implement a Quantum Kernel Estimation method to serve as a kernel for a classical Support Vector Machine (SVM).
3. **Barren Plateaus**: Experiment with deep circuits and observe the "vanishing gradient" problem specific to quantum landscapes, then try to mitigate it by initializing parameters in a restricted way.

---

[Next: Chapter 19 - Security and Post-Quantum Cryptography →](./chapter-19-security.md)