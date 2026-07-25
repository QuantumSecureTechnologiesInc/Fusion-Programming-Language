> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 17: Quantum Computing in Fusion

This chapter introduces Fusion's quantum computing capabilities—the third pillar of Tri-brid computing. Fusion is the world's first general-purpose programming language with **native quantum types**. Unlike languages that treat quantum computing as a library concern, Fusion's type system understands qubits, enforces the no-cloning theorem, and provides compile-time guarantees about quantum circuit validity.

Whether you're exploring quantum algorithms, building hybrid quantum-classical applications, or preparing for the post-quantum era, this chapter provides the foundation you need.

We'll cover:

- Theoretical foundations of quantum computing
- Qubits, quantum gates, and the `QuantumCircuit` type
- Building and simulating quantum algorithms
- Connecting to real quantum hardware
- Best practices for quantum software development

---

## 17.1 Quantum Computing Fundamentals

Before diving into code, let's establish the conceptual foundations. If you're already familiar with quantum mechanics, feel free to skim this section.

### 17.1.1 Classical Bits vs. Qubits

A classical bit is either 0 or 1. A **qubit** (quantum bit) can be in a **superposition** of both states simultaneously:

$$|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$$

Here:
- $|0\rangle$ and $|1\rangle$ are the computational basis states
- $\alpha$ and $\beta$ are complex probability amplitudes
- $|\alpha|^2 + |\beta|^2 = 1$ (normalisation)
- When measured, the qubit collapses to $|0\rangle$ with probability $|\alpha|^2$ or $|1\rangle$ with probability $|\beta|^2$

This superposition enables quantum parallelism: a system of $n$ qubits can represent $2^n$ states simultaneously.

### 17.1.2 Entanglement

Two qubits can become **entangled**, meaning their states are correlated in ways that have no classical analogue. The classic example is the Bell state:

$$|\Phi^+\rangle = \frac{1}{\sqrt{2}}(|00\rangle + |11\rangle)$$

When you measure the first qubit, you instantly know the second qubit's state—regardless of the physical distance between them. This isn't faster-than-light communication (you can't control the measurement outcome), but it's a resource that quantum algorithms exploit.

### 17.1.3 Quantum Gates

Just as classical circuits use logic gates (AND, OR, NOT), quantum circuits use **quantum gates**—unitary transformations that evolve qubit states. Common gates include:

| Gate              | Symbol     | Effect                                                                              |
| :---------------- | :--------- | :---------------------------------------------------------------------------------- |
| **Hadamard (H)**  | H          | Creates superposition: $\|0\rangle \to \frac{1}{\sqrt{2}}(\|0\rangle + \|1\rangle)$ |
| **Pauli-X (NOT)** | X          | Bit flip: $\|0\rangle \leftrightarrow \|1\rangle$                                   |
| **Pauli-Z**       | Z          | Phase flip: $\|1\rangle \to -\|1\rangle$                                            |
| **CNOT**          | CX         | Flips target if control is $\|1\rangle$                                             |
| **Rotation**      | Rx, Ry, Rz | Rotates around X, Y, or Z axis                                                      |

### 17.1.4 Measurement

**Measurement** extracts classical information from a qubit, but destroys the superposition. After measurement, the qubit is in a definite state ($|0\rangle$ or $|1\rangle$), and the probability amplitudes are gone.

This is irreversible: you cannot "unmeasure" a qubit. Quantum algorithms must carefully balance computation (in superposition) with measurement (to extract results).

### 17.1.5 The No-Cloning Theorem

A fundamental result of quantum mechanics: **unknown quantum states cannot be copied**. There's no quantum equivalent of `memcpy`. This has profound implications:
- Error correction requires redundancy without copying
- Quantum cryptography exploits this for secure communication
- Fusion's type system enforces this at compile time

---

## 17.2 Qubits in Fusion

Fusion provides a `Qubit` type that enforces quantum mechanical constraints.

### 17.2.1 The Qubit Type

```fusion
use fusion_quantum_sdk::Qubit

fn main() {
    // Create a qubit in the |0⟩ state
    let q = Qubit::zero()

    // Create a qubit in the |1⟩ state
    let q1 = Qubit::one()

    // The qubit type is non-copyable (enforces no-cloning)
    // let q_copy = q  // Error! Cannot move out of `q` after use
}
```text

### 17.2.2 No-Cloning Enforcement

The `Qubit` type does not implement `Copy` or `Clone`. This enforces the no-cloning theorem at the type level:

```fusion
fn try_to_clone(q: Qubit) -> (Qubit, Qubit) {
    // (q, q)  // Error! Cannot use `q` more than once
    //         // (it was moved into the first tuple position)
}
```text

This is not a limitation—it's a feature. The compiler catches violations of quantum mechanics before your code ever runs.

### 17.2.3 Qubit Registers

For algorithms on multiple qubits, use `QubitRegister`:

```fusion
use fusion_quantum_sdk::QubitRegister

fn main() {
    // Create a register of 5 qubits (all initialised to |0⟩)
    let register = QubitRegister::new(5)

    println("Number of qubits: {}", register.size())  // 5
    println("State: {}", register.state())  // |00000⟩
}
```text

### 17.2.4 QuantumState Representation

For simulation purposes, you can access the full state vector:

```fusion
use fusion_quantum_sdk::QuantumState

fn main() {
    // Create a 2-qubit state
    let state = QuantumState::new(2)  // |00⟩

    println("Number of qubits: {}", state.num_qubits())        // 2
    println("Dimension: {}", state.dimension())                // 2^2 = 4
    println("Amplitudes: {:?}", state.amplitudes())  // [1+0i, 0, 0, 0]

    // Check probabilities
    println("P(|00⟩): {}", state.probability(0))  // 1.0
    println("P(|01⟩): {}", state.probability(1))  // 0.0
}
```text

---

## 17.3 Quantum Gates

Gates are the building blocks of quantum circuits. Fusion provides all standard gates with a clean, composable API.

### 17.3.1 Single-Qubit Gates

```fusion
use fusion_quantum_sdk::gates::*
use std::f64::consts::PI

// Pauli gates
let x = PauliX::new()  // Bit flip
let y = PauliY::new()  // Bit + phase flip
let z = PauliZ::new()  // Phase flip

// Hadamard gate (creates superposition)
let h = Hadamard::new()

// Phase gates
let s = SGate::new()   // π/2 phase rotation (√Z)
let t = TGate::new()   // π/4 phase rotation (√S)

// Rotation gates (parameterised)
let rx = RotationX::new(PI / 4.0)   // Rotate around X by π/4
let ry = RotationY::new(PI / 2.0)   // Rotate around Y by π/2
let rz = RotationZ::new(PI)         // Rotate around Z by π

// General single-qubit rotation (Euler angles)
let u = UGate::new(theta: PI/2.0, phi: 0.0, lambda: PI)
```text

### 17.3.2 Understanding Gate Effects

Let's trace through a simple example:

```fusion
use fusion_quantum_sdk::*

fn main() {
    let mut circuit = QuantumCircuit::new(1)

    // Start: |0⟩
    circuit.h(0)        // After H: (|0⟩ + |1⟩) / √2
    circuit.z(0)        // After Z: (|0⟩ - |1⟩) / √2 = |-⟩
    circuit.h(0)        // After H: |1⟩ (H converts |+⟩↔|0⟩, |-⟩↔|1⟩)

    circuit.measure(0, 0)

    let sim = Simulator::new()
    let result = sim.run(&circuit)

    println("Result: {}", result.bits()[0])  // Always 1
}
```text

### 17.3.3 Two-Qubit Gates

```fusion
// CNOT (Controlled-NOT): flips target if control is |1⟩
let cnot = CNOT::new()

// CZ (Controlled-Z): applies Z to target if control is |1⟩
let cz = ControlledZ::new()

// SWAP: exchanges two qubits
let swap = SWAP::new()

// iSWAP: exchanges with additional phase
let iswap = iSWAP::new()

// Controlled rotations
let crx = ControlledRx::new(PI / 4.0)
let cry = ControlledRy::new(PI / 2.0)
let crz = ControlledRz::new(PI)

// ZZ interaction (used in QAOA)
let rzz = RZZ::new(PI / 4.0)
```text

### 17.3.4 Multi-Qubit Gates

```fusion
// Toffoli (CCNOT): 3-qubit AND gate
let toffoli = Toffoli::new()

// Fredkin (CSWAP): conditional swap
let fredkin = Fredkin::new()

// MCX: Multi-controlled X (generalised Toffoli)
let mcx = MultiControlledX::new(num_controls: 3)
```text

### 17.3.5 Custom Gates

Define your own unitary:

```fusion
use fusion_quantum_sdk::gates::CustomGate
use num_complex::Complex64

// Define a custom 2×2 unitary matrix
let matrix = [
    [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
    [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
]

let custom = CustomGate::from_matrix(&matrix)?  // Validates unitarity
```text

---

## 17.4 Quantum Circuits

A `QuantumCircuit` is a sequence of gates applied to qubits—the quantum equivalent of a program.

### 17.4.1 Building Circuits

```fusion
use fusion_quantum_sdk::QuantumCircuit

fn main() {
    // Create a circuit with 3 qubits
    let mut circuit = QuantumCircuit::new(3)

    // Apply gates by qubit index
    circuit.h(0)           // Hadamard on qubit 0
    circuit.x(1)           // X on qubit 1
    circuit.y(2)           // Y on qubit 2

    // Two-qubit gates take (control, target) or (qubit1, qubit2)
    circuit.cnot(0, 1)     // CNOT: control=0, target=1
    circuit.cz(1, 2)       // CZ between qubits 1 and 2

    // Parameterised gates
    circuit.rx(0, PI / 4.0)
    circuit.ry(1, PI / 2.0)
    circuit.rz(2, PI)

    // Multi-qubit gates
    circuit.ccnot(0, 1, 2)  // Toffoli: controls=0,1, target=2

    // Measurements
    circuit.measure(0, 0)   // Measure qubit 0, store in classical bit 0
    circuit.measure(1, 1)
    circuit.measure(2, 2)

    // Or measure all at once
    circuit.measure_all()
}
```text

### 17.4.2 Circuit Visualisation

```fusion
let circuit = /* ... build circuit ... */

// Text-based diagram
println("{}", circuit.draw())
```text

Output:

```text
     ┌───┐          ┌────────┐┌─┐
q_0: ┤ H ├───●──────┤ Rx(π/4)├┤M├
     └───┘┌──┴──┐   └────────┘└╥┘
q_1: ─────┤ NOT ├──────────────╫─
          └─────┘              ║
c: 1/══════════════════════════╩═
```text

### 17.4.3 Circuit Composition

Build complex circuits from simpler ones:

```fusion
fn bell_state_circuit() -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(2)
    circuit.h(0)
    circuit.cnot(0, 1)
    circuit
}

fn teleportation_circuit() -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(3)

    // Prepare state to teleport (optional)
    circuit.rx(0, PI / 3.0)

    // Create Bell pair between qubits 1 and 2
    circuit.append(&bell_state_circuit(), qubits: [1, 2])

    // Bell measurement on qubits 0 and 1
    circuit.cnot(0, 1)
    circuit.h(0)
    circuit.measure(0, 0)
    circuit.measure(1, 1)

    // Conditional corrections on qubit 2
    circuit.x(2).c_if(classical_bit: 1, value: 1)
    circuit.z(2).c_if(classical_bit: 0, value: 1)

    circuit
}
```text

### 17.4.4 Parameterised Circuits

For variational algorithms:

```fusion
fn ansatz(params: &[f64]) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(4)

    // Layer of rotation gates
    for (i, &theta) in params[0..4].iter().enumerate() {
        circuit.ry(i, theta)
    }

    // Entangling layer
    for i in 0..3 {
        circuit.cnot(i, i + 1)
    }

    // Another rotation layer
    for (i, &theta) in params[4..8].iter().enumerate() {
        circuit.ry(i, theta)
        circuit.rz(i, params[8 + i])
    }

    circuit
}

fn main() {
    let params = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2]
    let circuit = ansatz(&params)

    println("{}", circuit.draw())
}
```text

---

## 17.5 Simulation

Fusion includes a high-performance state vector simulator for testing quantum algorithms without needing hardware access.

### 17.5.1 Running Simulations

```fusion
use fusion_quantum_sdk::{QuantumCircuit, Simulator}

fn main() {
    let mut circuit = QuantumCircuit::new(2)
    circuit.h(0)
    circuit.cnot(0, 1)
    circuit.measure_all()

    let sim = Simulator::new()

    // Single execution
    let result = sim.run(&circuit)
    println("Result: {:?}", result.bits())  // e.g., [0, 0] or [1, 1]

    // Multiple executions (for probabilistic results)
    let results = sim.run_shots(&circuit, shots: 1000)
    println("Counts: {:?}", results.counts())
    // e.g., {"00": 498, "11": 502}
}
```text

### 17.5.2 Accessing the State Vector

For debugging and analysis, inspect the quantum state directly:

```fusion
fn main() {
    let mut circuit = QuantumCircuit::new(2)
    circuit.h(0)
    circuit.cnot(0, 1)
    // No measurement—preserve superposition

    let sim = Simulator::new()
    let state = sim.simulate_state(&circuit)

    println("State vector:")
    for (i, amp) in state.amplitudes().iter().enumerate() {
        if amp.norm() > 1e-10 {
            println("  |{:02b}⟩: {:.4}", i, amp)
        }
    }
    // Output:
    //   |00⟩: 0.7071+0i
    //   |11⟩: 0.7071+0i

    println("P(|00⟩) = {:.4}", state.probability(0))  // 0.5
    println("P(|11⟩) = {:.4}", state.probability(3))  // 0.5
}
```text

### 17.5.3 Expectation Values

Compute $\langle\psi|O|\psi\rangle$ for observables:

```fusion
use fusion_quantum_sdk::Observable

fn main() {
    let circuit = /* prepare state */

    // Define observable (as Pauli string)
    let z0_z1 = Observable::from_pauli("ZZ")  // Z₀ ⊗ Z₁

    let sim = Simulator::new()
    let expectation = sim.expectation_value(&circuit, &z0_z1)

    println("⟨ZZ⟩ = {:.4}", expectation)
}
```text

### 17.5.4 Noise Models

Simulate realistic quantum hardware:

```fusion
use fusion_quantum_sdk::noise::*

// Build a noise model
let noise = NoiseModel::new()
    .add_depolarising(probability: 0.01)        // 1% depolarising error
    .add_amplitude_damping(gamma: 0.005)        // T1 decay
    .add_phase_damping(gamma: 0.01)             // T2 decay
    .add_readout_error(p0_given_1: 0.02, p1_given_0: 0.01)

let sim = Simulator::with_noise(noise)

let results = sim.run_shots(&circuit, shots: 10000)
println("Noisy counts: {:?}", results.counts())
// Results will show errors compared to ideal simulation
```text

---

## 17.6 Hardware Backends

Fusion connects to real quantum computers through cloud providers.

### 17.6.1 IBM Quantum

```fusion
use fusion_quantum_sdk::backends::IBMQuantum

#[tokio::main]

async fn main() {
    // Authenticate with IBM Quantum
    let backend = IBMQuantum::new(api_key: env!("IBM_QUANTUM_API_KEY"))
        .await
        .device("ibm_brisbane")  // Choose a device

    // Build circuit
    let mut circuit = QuantumCircuit::new(2)
    circuit.h(0)
    circuit.cnot(0, 1)
    circuit.measure_all()

    // Submit job
    let job = backend.submit(&circuit).await?
    println("Job ID: {}", job.id())

    // Wait for results
    let result = job.wait().await?
    println("Results: {:?}", result.counts())
}
```text

### 17.6.2 AWS Braket

```fusion
use fusion_quantum_sdk::backends::AWSBraket

#[tokio::main]

async fn main() {
    let backend = AWSBraket::new()
        .await
        .device("arn:aws:braket:::device/quantum-simulator/amazon/sv1")

    let result = backend.run(&circuit, shots: 1000).await?
    println("Results: {:?}", result.counts())
}
```text

### 17.6.3 Backend Abstraction

Write hardware-agnostic code:

```fusion
use fusion_quantum_sdk::Backend

async fn run_on_any_backend(
    circuit: &QuantumCircuit,
    backend: &dyn Backend,
) -> Result<Counts, QuantumError> {
    let result = backend.run(circuit, shots: 1000).await?
    Ok(result.counts())
}

#[tokio::main]

async fn main() {
    let circuit = /* ... */

    // Run on simulator (free, fast)
    let sim = Simulator::new()
    let sim_result = run_on_any_backend(&circuit, &sim).await?

    // Run on real hardware (costs $$$, slow)
    let hw = IBMQuantum::new(api_key: "...").await
    let hw_result = run_on_any_backend(&circuit, &hw).await?

    // Compare results
    compare_distributions(&sim_result, &hw_result)
}
```text

---

## 17.7 Common Quantum Algorithms

Let's implement some fundamental quantum algorithms.

### 17.7.1 Bell State Preparation

The simplest entanglement:

```fusion
fn bell_state() -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(2)
    circuit.h(0)           // |0⟩ → |+⟩ = (|0⟩ + |1⟩)/√2
    circuit.cnot(0, 1)     // |+0⟩ → (|00⟩ + |11⟩)/√2
    circuit.measure_all()
    circuit
}

fn main() {
    let sim = Simulator::new()
    let results = sim.run_shots(&bell_state(), 1000)

    // Expect roughly 50% |00⟩, 50% |11⟩, never |01⟩ or |10⟩
    println("{:?}", results.counts())
}
```text

### 17.7.2 Quantum Teleportation

Transfer a quantum state using entanglement:

```fusion
fn quantum_teleportation() {
    let mut circuit = QuantumCircuit::new(3)

    // Qubit 0: State to teleport (prepare arbitrary state)
    circuit.rx(0, PI / 4.0)
    circuit.rz(0, PI / 3.0)

    // Qubits 1 and 2: Create Bell pair
    circuit.h(1)
    circuit.cnot(1, 2)

    // Bell measurement on qubits 0 and 1
    circuit.cnot(0, 1)
    circuit.h(0)
    circuit.measure(0, 0)
    circuit.measure(1, 1)

    // Classical communication and correction
    // (In real teleportation, this happens after classical bits are transmitted)
    circuit.x(2).c_if(1, 1)  // Apply X if classical bit 1 is 1
    circuit.z(2).c_if(0, 1)  // Apply Z if classical bit 0 is 1

    circuit
}
```text

### 17.7.3 Grover's Search Algorithm

Quantum search with quadratic speedup:

```fusion
fn grover_search(n_qubits: int, target: int) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(n_qubits + 1)  // +1 for ancilla

    // Initialise
    for i in 0..n_qubits {
        circuit.h(i)
    }
    circuit.x(n_qubits)
    circuit.h(n_qubits)

    // Grover iterations (optimal: ~π/4 * √N)
    let iterations = ((PI / 4.0) * (2.0_f64).powi(n_qubits / 2)).round() as int

    for _ in 0..iterations {
        // Oracle (marks target state)
        oracle(&mut circuit, target, n_qubits)

        // Diffusion (amplitude amplification)
        diffusion(&mut circuit, n_qubits)
    }

    // Measure
    for i in 0..n_qubits {
        circuit.measure(i, i)
    }

    circuit
}

fn oracle(circuit: &mut QuantumCircuit, target: int, n_qubits: int) {
    // Apply X gates to qubits where target bit is 0
    for i in 0..n_qubits {
        if (target >> i) & 1 == 0 {
            circuit.x(i)
        }
    }

    // Multi-controlled Z (implemented via H-Toffoli-H on ancilla)
    circuit.h(n_qubits)
    circuit.mcx((0..n_qubits).collect(), n_qubits)
    circuit.h(n_qubits)

    // Undo X gates
    for i in 0..n_qubits {
        if (target >> i) & 1 == 0 {
            circuit.x(i)
        }
    }
}

fn diffusion(circuit: &mut QuantumCircuit, n_qubits: int) {
    for i in 0..n_qubits {
        circuit.h(i)
        circuit.x(i)
    }

    circuit.h(n_qubits - 1)
    circuit.mcx((0..n_qubits - 1).collect(), n_qubits - 1)
    circuit.h(n_qubits - 1)

    for i in 0..n_qubits {
        circuit.x(i)
        circuit.h(i)
    }
}
```text

### 17.7.4 Quantum Fourier Transform

Foundation for many quantum algorithms:

```fusion
fn qft(n_qubits: int) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(n_qubits)

    for i in 0..n_qubits {
        circuit.h(i)

        for j in (i + 1)..n_qubits {
            let k = j - i + 1
            let phase = PI / (2.0_f64).powi(k as i32)
            circuit.cp(j, i, phase)  // Controlled phase rotation
        }
    }

    // Reverse qubit order
    for i in 0..(n_qubits / 2) {
        circuit.swap(i, n_qubits - 1 - i)
    }

    circuit
}

fn inverse_qft(n_qubits: int) -> QuantumCircuit {
    // QFT is unitary, so inverse is the Hermitian conjugate
    qft(n_qubits).inverse()
}
```text

---

## 17.8 Best Practices

### 17.8.1 Circuit Optimisation

Fusion's quantum compiler optimises circuits automatically, but you can help:

```fusion
// Before optimisation
let mut circuit = QuantumCircuit::new(2)
circuit.h(0)
circuit.h(0)  // Redundant: H·H = I
circuit.x(1)
circuit.x(1)  // Redundant: X·X = I

// Manually optimise
circuit.optimise()

// Or use specific optimisation passes
circuit.optimise_with([
    OptimisationPass::CancelInverses,
    OptimisationPass::MergeSingleQubitGates,
    OptimisationPass::CommuteControlledGates,
])
```text

### 17.8.2 Error Mitigation

On noisy hardware, apply error mitigation:

```fusion
use fusion_quantum_sdk::mitigation::*

// Measurement error mitigation
let mit = ReadoutErrorMitigation::calibrate(&backend).await?
let mitigated_counts = mit.apply(&raw_counts)

// Zero-noise extrapolation
let zne = ZNE::new(noise_factors: [1.0, 2.0, 3.0])
let extrapolated = zne.run(&circuit, &backend).await?
```text

### 17.8.3 Testing Quantum Code

```fusion

#[cfg(test)]

mod tests {
    use super::*

    #[test]
    fn test_bell_state() {
        let circuit = bell_state()
        let sim = Simulator::new()
        let results = sim.run_shots(&circuit, 10000)

        let counts = results.counts()

        // Should only have |00⟩ and |11⟩
        assert!(counts.get("01").unwrap_or(&0) < &50)
        assert!(counts.get("10").unwrap_or(&0) < &50)

        // Should be roughly 50-50
        let count_00 = *counts.get("00").unwrap_or(&0) as f64
        let count_11 = *counts.get("11").unwrap_or(&0) as f64
        assert!((count_00 / count_11).abs() < 1.2)
    }

    #[test]
    fn test_state_vector() {
        let circuit = bell_state()  // Without measurement
        let sim = Simulator::new()
        let state = sim.simulate_state(&circuit)

        // |00⟩ and |11⟩ should have amplitude 1/√2 ≈ 0.707
        assert!((state.amplitudes()[0].re - 0.707).abs() < 0.01)
        assert!((state.amplitudes()[3].re - 0.707).abs() < 0.01)
    }
}
```text

---

## 17.9 Summary

This chapter covered Fusion's quantum computing capabilities:

| Feature               | Description                           |
| :-------------------- | :------------------------------------ |
| **Qubit**             | Non-copyable quantum bit type         |
| **QuantumCircuit**    | Sequence of quantum gates             |
| **Gates**             | H, X, Y, Z, CNOT, rotations, and more |
| **Simulator**         | State vector simulation               |
| **Hardware Backends** | IBM Quantum, AWS Braket               |
| **Noise Models**      | Realistic error simulation            |

Key takeaways:

1. **Fusion enforces quantum mechanics at the type level**—no-cloning is a compile error
2. **Circuits are first-class values**—compose, optimise, and pass them around
3. **Simulation and hardware share the same API**—develop locally, deploy to QPUs
4. **Quantum and classical code interoperate seamlessly**—no FFI, no context switches

Quantum computing is no longer a niche research topic. With Fusion, it's a natural part of the programming landscape.

---

## 17.10 Exercises

1. **GHZ State**: Create a 5-qubit GHZ state ($\frac{1}{\sqrt{2}}(|00000\rangle + |11111\rangle)$) and verify with simulation.

2. **Deutsch-Jozsa Algorithm**: Implement the algorithm that determines whether a function is constant or balanced.

3. **Quantum Random Number Generator**: Use a qubit in superposition to generate cryptographically random bits.

4. **Error Analysis**: Run the same circuit on a noisy simulation and compare to ideal results.

5. **Hardware Submission**: If you have access, submit a Bell state circuit to IBM Quantum and analyse the results.

---

[Next: Chapter 18 - Hybrid Quantum-Classical Algorithms →](./chapter-18-hybrid.md)