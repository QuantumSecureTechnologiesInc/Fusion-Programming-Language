# Chapter 8: Quantum Computing

> Qubits, gates, circuits, simulation, and hybrid quantum-classical programming

---

## Quantum Concepts

### Qubits

A qubit is the fundamental unit of quantum information. Unlike classical bits (0 or 1), a qubit can exist in a **superposition** of both states simultaneously.

```fusion
use std::quantum;

fn main() -> int {
    // Create a single qubit (starts in |0⟩ state)
    let qubit: quantum::Qubit = quantum::Qubit::new();

    println("Qubit created in |0⟩ state");

    // Apply a Hadamard gate to create superposition
    qubit.h();

    // Measure the qubit
    let result: int = qubit.measure();
    println("Measurement result: %d", result);  // 0 or 1

    return 0;
}
```

### State Vectors

A qubit's state is described by a **state vector** — a complex vector whose components represent the probability amplitudes of each computational basis state.

```fusion
use std::quantum;

fn main() -> int {
    let sim: quantum::Simulator = quantum::Simulator::new();
    let q: quantum::SimQubit = sim.allocate_qubit();

    // |0⟩ state: [1, 0]
    let sv0: quantum::StateVector = sim.get_state_vector();
    println("Initial |0⟩: %s", sv0.to_string());
    // Output: |0⟩ (amplitude: 1.000)

    // Apply Hadamard → (|0⟩ + |1⟩) / √2
    q.h();
    let sv1: quantum::StateVector = sim.get_state_vector();
    println("After H: %s", sv1.to_string());
    // Output: 0.707|0⟩ + 0.707|1⟩

    // Apply X gate → |1⟩
    q.x();
    let sv2: quantum::StateVector = sim.get_state_vector();
    println("After X: %s", sv2.to_string());
    // Output: |1⟩ (amplitude: 1.000)

    // Multi-qubit state vectors
    let q1: quantum::SimQubit = sim.allocate_qubit();
    let q2: quantum::SimQubit = sim.allocate_qubit();
    q1.h();
    quantum::cnot(q1, q2);
    let sv3: quantum::StateVector = sim.get_state_vector();
    println("Bell state: %s", sv3.to_string());
    // Output: 0.707|00⟩ + 0.707|11⟩

    return 0;
}
```

### Quantum States

```fusion
use std::quantum;

fn main() -> int {
    // |0⟩ state
    let q0: quantum::Qubit = quantum::Qubit::zero();
    println("q0 = |0⟩");

    // |1⟩ state
    let q1: quantum::Qubit = quantum::Qubit::one();
    println("q1 = |1⟩");

    // Superposition state: (|0⟩ + |1⟩) / √2
    let qsuper: quantum::Qubit = quantum::Qubit::zero();
    qsuper.h();  // Hadamard gate
    println("qsuper = (|0⟩ + |1⟩) / √2");

    return 0;
}
```

---

## Available Gates

### Single-Qubit Gates

| Gate | Symbol | Description |
|------|--------|-------------|
| Hadamard | H | Creates superposition |
| Pauli-X | X | Bit flip (NOT) |
| Pauli-Y | Y | Bit and phase flip |
| Pauli-Z | Z | Phase flip |
| T | T | π/8 rotation |
| S | S | π/4 rotation |
| Rx | Rx(θ) | Rotation around X-axis by angle θ |
| Ry | Ry(θ) | Rotation around Y-axis by angle θ |
| Rz | Rz(θ) | Rotation around Z-axis by angle θ |

```fusion
use std::quantum;

fn main() -> int {
    let q: quantum::Qubit = quantum::Qubit::zero();

    // Pauli-X gate (NOT gate) — flips |0⟩ to |1⟩ and vice versa
    q.x();
    println("After X: |1⟩");

    // Pauli-Y gate — equivalent to iXZ, rotates π around Y-axis
    q.y();

    // Pauli-Z gate — applies phase flip, |1⟩ → -|1⟩
    q.z();

    // Hadamard gate — creates equal superposition
    q.h();
    println("After H: superposition");

    // T gate (π/8 rotation) — phase gate, adds π/4 phase to |1⟩
    q.t();

    // S gate (π/4 rotation) — square root of Z, adds π/2 phase to |1⟩
    q.s();

    return 0;
}
```

### Rotation Gates (Rx, Ry, Rz)

Rotation gates allow arbitrary single-qubit rotations around the X, Y, or Z axes of the Bloch sphere. These are essential for parameterized quantum circuits and variational algorithms.

```fusion
use std::quantum;

fn main() -> int {
    // Rx(θ) — rotation around X-axis by angle θ
    // Rx(π/2) rotates |0⟩ to (|0⟩ - i|1⟩) / √2
    let q1: quantum::Qubit = quantum::Qubit::zero();
    q1.rx(3.14159 / 2.0);  // π/2 rotation

    // Ry(θ) — rotation around Y-axis by angle θ
    // Ry(π/2) rotates |0⟩ to (|0⟩ + |1⟩) / √2
    let q2: quantum::Qubit = quantum::Qubit::zero();
    q2.ry(3.14159 / 2.0);

    // Rz(θ) — rotation around Z-axis by angle θ
    // Rz(π) adds a phase of e^(iπ) = -1 to |1⟩
    let q3: quantum::Qubit = quantum::Qubit::zero();
    q3.h();  // put in superposition first
    q3.rz(3.14159);  // π rotation

    println("Rotation gates applied");

    return 0;
}
```

### Rotation Gates in Circuits

```fusion
use std::quantum;

fn main() -> int {
    let circuit: quantum::Circuit = quantum::Circuit::new(2, 2);

    // Parameterized rotations on each qubit
    circuit.rx(0, 0.5);      // Rx(0.5) on qubit 0
    circuit.ry(0, 1.0);      // Ry(1.0) on qubit 0
    circuit.rz(1, 1.5);      // Rz(1.5) on qubit 1

    // Entangle after rotations
    circuit.cnot(0, 1);

    circuit.measure(0, 0);
    circuit.measure(1, 1);

    circuit.draw();

    let results: quantum::ShotResults = circuit.execute_shots(1000);
    println("Rotation circuit results: %s", results.to_string());

    return 0;
}
```

### Multi-Qubit Gates

| Gate | Description |
|------|-------------|
| CNOT | Controlled-NOT (entangles two qubits) |
| CZ | Controlled-Z |
| Toffoli | Controlled-Controlled-NOT (3 qubits) |
| SWAP | Swaps two qubits |

```fusion
use std::quantum;

fn main() -> int {
    // Create two qubits
    let q1: quantum::Qubit = quantum::Qubit::zero();
    let q2: quantum::Qubit = quantum::Qubit::zero();

    // Put first qubit in superposition
    q1.h();

    // CNOT gate: if q1 is |1⟩, flip q2
    quantum::cnot(q1, q2);

    // Now q1 and q2 are entangled
    // Measuring q1 determines q2

    let r1: int = q1.measure();
    let r2: int = q2.measure();
    println("q1=%d, q2=%d (should be equal)", r1, r2);

    return 0;
}
```

### Toffoli Gate (3-Qubit)

```fusion
use std::quantum;

fn main() -> int {
    let q1: quantum::Qubit = quantum::Qubit::one();
    let q2: quantum::Qubit = quantum::Qubit::one();
    let q3: quantum::Qubit = quantum::Qubit::zero();

    // Toffoli: if q1 AND q2 are |1⟩, flip q3
    quantum::toffoli(q1, q2, q3);

    let result: int = q3.measure();
    println("Toffoli result: %d", result);  // 1 (flipped)

    return 0;
}
```

### SWAP Gate

```fusion
use std::quantum;

fn main() -> int {
    let q1: quantum::Qubit = quantum::Qubit::zero();
    let q2: quantum::Qubit = quantum::Qubit::one();

    // Before swap: q1=|0⟩, q2=|1⟩
    quantum::swap(q1, q2);

    // After swap: q1=|1⟩, q2=|0⟩
    let r1: int = q1.measure();
    let r2: int = q2.measure();
    println("After SWAP: q1=%d, q2=%d", r1, r2);

    return 0;
}
```

---

## Building Circuits

### Basic Circuit

```fusion
use std::quantum;

fn main() -> int {
    // Create a circuit with 2 qubits and 2 classical bits
    let circuit: quantum::Circuit = quantum::Circuit::new(2, 2);

    // Add gates to the circuit
    circuit.h(0);                    // Hadamard on qubit 0
    circuit.cnot(0, 1);              // CNOT: qubit 0 → qubit 1
    circuit.measure(0, 0);           // Measure qubit 0 → classical bit 0
    circuit.measure(1, 1);           // Measure qubit 1 → classical bit 1

    // Print circuit diagram
    circuit.draw();

    // Execute the circuit
    let result: quantum::Result = circuit.execute();
    println("Result: %s", result.to_string());

    return 0;
}
```

### Complex Circuit

```fusion
use std::quantum;

fn main() -> int {
    let circuit: quantum::Circuit = quantum::Circuit::new(3, 3);

    // Create a GHZ state (|000⟩ + |111⟩) / √2
    circuit.h(0);              // Superposition on qubit 0
    circuit.cnot(0, 1);        // Entangle qubit 0 and 1
    circuit.cnot(1, 2);        // Entangle qubit 1 and 2

    // Measure all qubits
    circuit.measure(0, 0);
    circuit.measure(1, 1);
    circuit.measure(2, 2);

    circuit.draw();

    // Run multiple shots
    let results: quantum::ShotResults = circuit.execute_shots(1000);
    println("GHZ state results:");
    println("  |000⟩: %d shots", results.count("000"));
    println("  |111⟩: %d shots", results.count("111"));

    return 0;
}
```

### Parameterized Circuit (VQE-Style)

```fusion
use std::quantum;

fn main() -> int {
    let circuit: quantum::Circuit = quantum::Circuit::new(2, 2);

    // Parameterized rotation layer
    circuit.ry(0, 0.5);       // Ry(0.5) on qubit 0
    circuit.rx(1, 0.3);       // Rx(0.3) on qubit 1

    // Entangling layer
    circuit.cnot(0, 1);

    // Second rotation layer
    circuit.rz(0, 0.7);
    circuit.rz(1, 0.2);

    // Second entangling layer
    circuit.cnot(1, 0);

    // Final rotations
    circuit.ry(0, 0.4);
    circuit.rx(1, 0.6);

    circuit.measure(0, 0);
    circuit.measure(1, 1);

    circuit.draw();

    let results: quantum::ShotResults = circuit.execute_shots(1000);
    println("Parameterized circuit results: %s", results.to_string());

    return 0;
}
```

---

## Measurement and Analysis

### Single Measurement

```fusion
use std::quantum;

fn main() -> int {
    let q: quantum::Qubit = quantum::Qubit::zero();
    q.h();  // Superposition

    // Measure once
    let result: int = q.measure();
    println("Measurement: %d", result);

    return 0;
}
```

### Multiple Shots

```fusion
use std::quantum;

fn main() -> int {
    // Run 1000 measurements
    let results: quantum::ShotResults = quantum::run_shots(|| {
        let q: quantum::Qubit = quantum::Qubit::zero();
        q.h();
        return q.measure();
    }, 1000);

    // Analyze results
    println("Total shots: %d", results.total());
    println("Zeros: %d (%.1f%%)", results.count(0), results.percentage(0));
    println("Ones: %d (%.1f%%)", results.count(1), results.percentage(1));

    return 0;
}
```

### Expectation Values

```fusion
use std::quantum;

fn main() -> int {
    let sim: quantum::Simulator = quantum::Simulator::new();
    let q: quantum::SimQubit = sim.allocate_qubit();

    q.h();

    // Measure Z operator expectation value
    let exp_z: float = sim.expectation_z(q);
    println("⟨Z⟩ = %f", exp_z);  // Should be ~0 for superposition

    // Measure X operator expectation value
    let exp_x: float = sim.expectation_x(q);
    println("⟨X⟩ = %f", exp_x);

    // Measure Y operator expectation value
    let exp_y: float = sim.expectation_y(q);
    println("⟨Y⟩ = %f", exp_y);

    return 0;
}
```

### Density Matrix

```fusion
use std::quantum;

fn main() -> int {
    let sim: quantum::Simulator = quantum::Simulator::new();
    let q: quantum::SimQubit = sim.allocate_qubit();

    // Pure |0⟩ state
    let dm0: quantum::DensityMatrix = sim.get_density_matrix();
    println("Density matrix |0⟩: %s", dm0.to_string());

    // After Hadamard — maximally mixed on diagonal
    q.h();
    let dm1: quantum::DensityMatrix = sim.get_density_matrix();
    println("Density matrix after H: %s", dm1.to_string());

    // Check purity (should be 1.0 for pure states)
    let purity: float = dm1.purity();
    println("State purity: %f", purity);

    return 0;
}
```

---

## Backend Selection

Fusion supports multiple quantum backends — from local simulators to cloud-based quantum hardware providers.

### Available Backends

| Backend | Type | Qubits | Use Case |
|---------|------|--------|----------|
| `local_simulator` | Simulator | Unlimited | Development, testing |
| `statevector_sim` | Simulator | Up to 30 | State vector simulation |
| `ibm_quantum` | Hardware | 100+ | IBM Quantum hardware |
| `aws_braket` | Hardware | 100+ | Amazon Braket (IonQ, Rigetti) |
| `azure_quantum` | Hardware | 100+ | Azure Quantum providers |
| `google_cirq` | Hardware | 70+ | Google Sycamore/Eagle |
| `rigetti_qc` | Hardware | 40+ | Rigetti QPU |

### Backend Selection

```fusion
use std::quantum;

fn main() -> int {
    // Use local simulator (default)
    let sim1: quantum::Simulator = quantum::Simulator::new();

    // Use statevector simulator
    let sim2: quantum::Simulator = quantum::Simulator::with_backend(
        quantum::Backend::StatevectorSim,
    );

    // Use IBM Quantum backend
    let sim3: quantum::Simulator = quantum::Simulator::with_backend(
        quantum::Backend::IBMQuantum,
    );

    // Use AWS Braket backend
    let sim4: quantum::Simulator = quantum::Simulator::with_backend(
        quantum::Backend::AWSBraket,
    );

    // Use Azure Quantum backend
    let sim5: quantum::Simulator = quantum::Simulator::with_backend(
        quantum::Backend::AzureQuantum,
    );

    // Use Google Cirq backend
    let sim6: quantum::Simulator = quantum::Simulator::with_backend(
        quantum::Backend::GoogleCirq,
    );

    // Use Rigetti backend
    let sim7: quantum::Simulator = quantum::Simulator::with_backend(
        quantum::Backend::RigettiQC,
    );

    println("Backend selection demonstrated");

    return 0;
}
```

### Backend with Options

```fusion
use std::quantum;

fn main() -> int {
    // Configure IBM Quantum backend
    let ibm_config: quantum::BackendConfig = quantum::BackendConfig {
        backend: quantum::Backend::IBMQuantum,
        provider: "ibm-q",
        device: "ibm_brisbane",
        shots: 4096,
        api_token: std::env::var("IBM_QUANTUM_TOKEN"),
    };

    let sim: quantum::Simulator = quantum::Simulator::with_config(ibm_config);

    // Configure AWS Braket backend
    let aws_config: quantum::BackendConfig = quantum::BackendConfig {
        backend: quantum::Backend::AWSBraket,
        provider: "ionq",
        device: "ionq_harmony",
        shots: 1000,
        region: "us-east-1",
    };

    let sim2: quantum::Simulator = quantum::Simulator::with_config(aws_config);

    // Configure Google Cirq backend
    let google_config: quantum::BackendConfig = quantum::BackendConfig {
        backend: quantum::Backend::GoogleCirq,
        provider: "google",
        device: "sycamore",
        shots: 5000,
    };

    let sim3: quantum::Simulator = quantum::Simulator::with_config(google_config);

    println("Backend configuration demonstrated");

    return 0;
}
```

---

## Configuration in Fusion.toml

```toml
[quantum]
# Default backend for quantum circuits
backend = "local_simulator"

# Number of shots for circuit execution
shots = 1024

# Enable noise model for simulation
noise_model = false

# Random seed for reproducibility
seed = 42

[quantum.backends]
# Local simulator settings
[quantum.backends.local_simulator]
max_qubits = 30
memory_limit = "4GB"

# IBM Quantum settings
[quantum.backends.ibm_quantum]
provider = "ibm-q"
device = "ibm_brisbane"
api_token_env = "IBM_QUANTUM_TOKEN"
hub = "ibm-q"
group = "open"
project = "main"

# AWS Braket settings
[quantum.backends.aws_braket]
region = "us-east-1"
s3_bucket = "fusion-quantum-results"
s3_prefix = "jobs/"

# Azure Quantum settings
[quantum.backends.azure_quantum]
resource_group = "fusion-rg"
workspace = "fusion-quantum-ws"
location = "eastus"

# Google Cirq settings
[quantum.backends.google_cirq]
project_id = "fusion-quantum-project"
processor = "sycamore"

# Rigetti settings
[quantum.backends.rigetti]
api_token_env = "RIGETTI_API_TOKEN"
endpoint = "https://api.rigetti.com"

[quantum.noise]
# Noise model parameters
depolarizing_rate = 0.01
t1 = 50.0       # T1 relaxation time (microseconds)
t2 = 70.0       # T2 dephasing time (microseconds)
gate_error = 0.005
measurement_error = 0.02
```

---

## Hybrid Quantum-Classical Programming

### Variational Quantum Eigensolver (VQE)

```fusion
use std::quantum;

fn ansatz(params: [float], qubit: quantum::SimQubit) {
    // Parameterized quantum circuit
    qubit.rx(params[0]);
    qubit.rz(params[1]);
}

fn cost_function(params: [float]) -> float {
    let sim: quantum::Simulator = quantum::Simulator::new();
    let q: quantum::SimQubit = sim.allocate_qubit();

    ansatz(params, q);

    // Measure energy
    return sim.expectation_z(q);
}

fn main() -> int {
    // Classical optimization loop
    let mut params: [float] = [0.0, 0.0];
    let learning_rate: float = 0.1;

    for i in 0..100 {
        let energy: float = cost_function(params);

        // Gradient descent (simplified)
        let grad: float = (cost_function([params[0] + 0.01, params[1]])
                         - cost_function([params[0] - 0.01, params[1]])) / 0.02;

        params[0] = params[0] - learning_rate * grad;

        if i %% 10 == 0 {
            println("Iteration %d: energy = %f", i, energy);
        }
    }

    println("Final energy: %f", cost_function(params));
    return 0;
}
```

### Quantum Neural Network

```fusion
use std::quantum;

fn quantum_layer(qubits: [quantum::SimQubit], params: [float]) {
    for i in 0..qubits.len() {
        qubits[i].rx(params[i * 2]);
        qubits[i].rz(params[i * 2 + 1]);
    }

    for i in 0..qubits.len() - 1 {
        quantum::cnot(qubits[i], qubits[i + 1]);
    }
}

fn main() -> int {
    let sim: quantum::Simulator = quantum::Simulator::new();
    let qubits: [quantum::SimQubit] = [
        sim.allocate_qubit(),
        sim.allocate_qubit(),
        sim.allocate_qubit(),
    ];

    // Apply parameterized layers
    let params1: [float] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
    quantum_layer(qubits, params1);

    let params2: [float] = [0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
    quantum_layer(qubits, params2);

    // Measure
    let result: quantum::Measurement = sim.measure_all();
    println("QNN output: %s", result.to_string());

    return 0;
}
```

---

## Complete Examples

### Bell State Creation

```fusion
use std::quantum;

fn main() -> int {
    // Create Bell state: (|00⟩ + |11⟩) / √2
    let circuit: quantum::Circuit = quantum::Circuit::new(2, 2);

    circuit.h(0);        // Superposition on qubit 0
    circuit.cnot(0, 1);  // Entangle with qubit 1
    circuit.measure(0, 0);
    circuit.measure(1, 1);

    println("Bell State Circuit:");
    circuit.draw();

    // Run and verify entanglement
    let results: quantum::ShotResults = circuit.execute_shots(1000);

    // Bell state should only produce |00⟩ or |11⟩
    let zeros: int = results.count("00");
    let ones: int = results.count("11");
    let mixed: int = results.total() - zeros - ones;

    println("Results:");
    println("  |00⟩: %d (%.1f%%)", zeros, results.percentage("00"));
    println("  |11⟩: %d (%.1f%%)", ones, results.percentage("11"));
    println("  Mixed: %d (%.1f%%)", mixed, (mixed as float) / (results.total() as float) * 100.0);

    return 0;
}
```

### GHZ State

```fusion
use std::quantum;

fn main() -> int {
    // Create GHZ state for 3 qubits: (|000⟩ + |111⟩) / √2
    let circuit: quantum::Circuit = quantum::Circuit::new(3, 3);

    circuit.h(0);
    circuit.cnot(0, 1);
    circuit.cnot(1, 2);
    circuit.measure(0, 0);
    circuit.measure(1, 1);
    circuit.measure(2, 2);

    println("GHZ State Circuit:");
    circuit.draw();

    let results: quantum::ShotResults = circuit.execute_shots(1000);

    println("GHZ Results:");
    println("  |000⟩: %d", results.count("000"));
    println("  |111⟩: %d", results.count("111"));

    return 0;
}
```

### N-Qubit GHZ State (Generalized)

```fusion
use std::quantum;

fn create_nghz(n: int) -> quantum::Circuit {
    let circuit: quantum::Circuit = quantum::Circuit::new(n, n);

    // Hadamard on first qubit
    circuit.h(0);

    // Chain of CNOTs to entangle all qubits
    for i in 0..n - 1 {
        circuit.cnot(i, i + 1);
    }

    // Measure all qubits
    for i in 0..n {
        circuit.measure(i, i);
    }

    return circuit;
}

fn main() -> int {
    let n: int = 5;  // 5-qubit GHZ state
    let circuit: quantum::Circuit = create_nghz(n);

    println("5-qubit GHZ state:");
    circuit.draw();

    let results: quantum::ShotResults = circuit.execute_shots(1000);
    println("|00000⟩: %d", results.count("00000"));
    println("|11111⟩: %d", results.count("11111"));

    return 0;
}
```

### Simple Quantum Algorithm: Deutsch-Jozsa

```fusion
use std::quantum;

fn main() -> int {
    // Deutsch-Jozsa algorithm: determine if f(x) is constant or balanced
    // Using 3 qubits: 2 input qubits + 1 output qubit
    let num_qubits: int = 3;
    let circuit: quantum::Circuit = quantum::Circuit::new(num_qubits, 2);

    // Initialize: set qubit 2 (output) to |1⟩
    circuit.x(2);

    // Apply Hadamard to all qubits
    for i in 0..num_qubits {
        circuit.h(i);
    }

    // Oracle for a balanced function: f(00)=0, f(01)=1, f(10)=1, f(11)=0
    // This is implemented with CNOT gates from input qubits to output
    circuit.cnot(0, 2);
    circuit.cnot(1, 2);

    // Apply Hadamard to input qubits
    circuit.h(0);
    circuit.h(1);

    // Measure input qubits
    circuit.measure(0, 0);
    circuit.measure(1, 1);

    println("Deutsch-Jozsa Circuit:");
    circuit.draw();

    let results: quantum::ShotResults = circuit.execute_shots(1000);

    // If result is |00⟩, function is constant; otherwise balanced
    let zeros: int = results.count("00");
    let nonzeros: int = results.total() - zeros;

    if zeros > results.total() / 2 {
        println("Function is CONSTANT (%d shots of |00⟩)", zeros);
    } else {
        println("Function is BALANCED (%d shots of non-|00⟩)", nonzeros);
    }

    return 0;
}
```

### Simple Quantum Algorithm: Quantum Phase Estimation (Simplified)

```fusion
use std::quantum;

fn main() -> int {
    // Simplified QPE: estimate the phase of a unitary U|ψ⟩ = e^(2πiφ)|ψ⟩
    let num_counting_qubits: int = 3;
    let total_qubits: int = num_counting_qubits + 1;
    let circuit: quantum::Circuit = quantum::Circuit::new(total_qubits, num_counting_qubits);

    // Prepare eigenstate on target qubit (qubit 3)
    // For this example, the target is already in the eigenstate |1⟩
    circuit.x(num_counting_qubits);

    // Apply Hadamard to counting qubits
    for i in 0..num_counting_qubits {
        circuit.h(i);
    }

    // Controlled-U operations (simplified: using controlled phase gates)
    // For phase φ = 1/4 (π/2 rotation):
    circuit.cp(0, num_counting_qubits, 3.14159 / 2.0);       // 2^0 * π/2
    circuit.cp(1, num_counting_qubits, 3.14159);              // 2^1 * π/2
    circuit.cp(2, num_counting_qubits, 3.14159 * 2.0);       // 2^2 * π/2

    // Inverse QFT on counting qubits
    circuit.h(2);
    circuit.cp(1, 2, -3.14159 / 2.0);
    circuit.cp(0, 2, -3.14159 / 4.0);
    circuit.h(1);
    circuit.cp(0, 1, -3.14159 / 2.0);
    circuit.h(0);
    circuit.swap(0, 2);

    // Measure counting qubits
    for i in 0..num_counting_qubits {
        circuit.measure(i, i);
    }

    circuit.draw();

    let results: quantum::ShotResults = circuit.execute_shots(1000);
    println("QPE results: %s", results.to_string());
    println("Most likely state: %s", results.most_likely());

    return 0;
}
```

---

## Tips and Best Practices

1. **Start with simulators**: Use the local simulator for development and testing.
2. **Verify entanglement**: Check that entangled qubits produce correlated results.
3. **Optimize circuit depth**: Fewer gates mean less noise on real hardware.
4. **Use variational algorithms**: Hybrid quantum-classical approaches are most practical today.
5. **Consider decoherence**: Real quantum hardware has limited coherence time.
6. **Choose the right backend**: Use simulators for prototyping, hardware for production runs.
7. **Tune shot counts**: More shots improve statistics but increase cost on real hardware.
8. **Use noise models**: Test with noise models before running on real hardware.

---

## Cross-References

- **Chapter 7**: Post-Quantum Cryptography for quantum-safe communication
- **Chapter 9**: Machine Learning for quantum ML (VQE, QNN)
- **Chapter 10**: Concurrency for parallel quantum execution
- **Chapter 14**: Examples for more quantum examples
