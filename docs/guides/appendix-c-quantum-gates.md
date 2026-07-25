> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Appendix C: Quantum Gate Reference

This appendix provides a comprehensive reference for quantum gates available in Fusion's quantum computing library.

---

## Single-Qubit Gates

### Pauli Gates

| Gate        | Symbol | Matrix                                          | Description                            |
| :---------- | :----- | :---------------------------------------------- | :------------------------------------- |
| **Pauli-X** | X      | $\begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix}$  | Bit flip: `\|0⟩ → \|1⟩`, `\|1⟩ → \|0⟩` |
| **Pauli-Y** | Y      | $\begin{pmatrix} 0 & -i \\ i & 0 \end{pmatrix}$ | Bit and phase flip                     |
| **Pauli-Z** | Z      | $\begin{pmatrix} 1 & 0 \\ 0 & -1 \end{pmatrix}$ | Phase flip: `\|1⟩ → -\|1⟩`             |

```fusion
circuit.x(0)      // Apply X to qubit 0
circuit.y(0)      // Apply Y to qubit 0
circuit.z(0)      // Apply Z to qubit 0
```text

### Hadamard Gate

| Gate         | Symbol | Matrix                                                            | Description           |
| :----------- | :----- | :---------------------------------------------------------------- | :-------------------- |
| **Hadamard** | H      | $\frac{1}{\sqrt{2}}\begin{pmatrix} 1 & 1 \\ 1 & -1 \end{pmatrix}$ | Creates superposition |

```fusion
circuit.h(0)      // |0⟩ → (|0⟩ + |1⟩)/√2

// Effect:
// |0⟩ → |+⟩ = (|0⟩ + |1⟩)/√2
// |1⟩ → |-⟩ = (|0⟩ - |1⟩)/√2
```text

### Phase Gates

| Gate   | Symbol | Matrix                                                   | Description    |
| :----- | :----- | :------------------------------------------------------- | :------------- |
| **S**  | S      | $\begin{pmatrix} 1 & 0 \\ 0 & i \end{pmatrix}$           | π/2 phase (√Z) |
| **S†** | Sdg    | $\begin{pmatrix} 1 & 0 \\ 0 & -i \end{pmatrix}$          | -π/2 phase     |
| **T**  | T      | $\begin{pmatrix} 1 & 0 \\ 0 & e^{i\pi/4} \end{pmatrix}$  | π/4 phase (√S) |
| **T†** | Tdg    | $\begin{pmatrix} 1 & 0 \\ 0 & e^{-i\pi/4} \end{pmatrix}$ | -π/4 phase     |

```fusion
circuit.s(0)      // S gate
circuit.sdg(0)    // S-dagger gate
circuit.t(0)      // T gate
circuit.tdg(0)    // T-dagger gate
```text

### Rotation Gates

| Gate      | Symbol | Matrix                                                                                                 | Description            |
| :-------- | :----- | :----------------------------------------------------------------------------------------------------- | :--------------------- |
| **Rx(θ)** | Rx     | $\begin{pmatrix} \cos(\theta/2) & -i\sin(\theta/2) \\ -i\sin(\theta/2) & \cos(\theta/2) \end{pmatrix}$ | Rotation around X-axis |
| **Ry(θ)** | Ry     | $\begin{pmatrix} \cos(\theta/2) & -\sin(\theta/2) \\ \sin(\theta/2) & \cos(\theta/2) \end{pmatrix}$    | Rotation around Y-axis |
| **Rz(θ)** | Rz     | $\begin{pmatrix} e^{-i\theta/2} & 0 \\ 0 & e^{i\theta/2} \end{pmatrix}$                                | Rotation around Z-axis |

```fusion
use std::f64::consts::PI

circuit.rx(0, PI / 4.0)   // Rotate around X by π/4
circuit.ry(0, PI / 2.0)   // Rotate around Y by π/2
circuit.rz(0, PI)         // Rotate around Z by π
```text

### Universal Single-Qubit Gate

| Gate           | Symbol | Description                   |
| :------------- | :----- | :---------------------------- |
| **U(θ, φ, λ)** | U      | General single-qubit rotation |

$U(\theta, \phi, \lambda) = \begin{pmatrix} \cos(\theta/2) & -e^{i\lambda}\sin(\theta/2) \\ e^{i\phi}\sin(\theta/2) & e^{i(\phi+\lambda)}\cos(\theta/2) \end{pmatrix}$

```fusion
circuit.u(0, theta, phi, lambda)
```text

---

## Two-Qubit Gates

### Controlled Gates

| Gate          | Symbol | Description         |
| :------------ | :----- | :------------------ |
| **CNOT** (CX) | ●─X    | Controlled-NOT      |
| **CY**        | ●─Y    | Controlled-Y        |
| **CZ**        | ●─Z    | Controlled-Z        |
| **CH**        | ●─H    | Controlled-Hadamard |

```fusion
circuit.cnot(0, 1)    // Control: 0, Target: 1
circuit.cx(0, 1)      // Alias for CNOT
circuit.cy(0, 1)      // Controlled-Y
circuit.cz(0, 1)      // Controlled-Z
circuit.ch(0, 1)      // Controlled-Hadamard
```text

**CNOT Matrix (control=0, target=1):**

$CNOT = \begin{pmatrix} 1 & 0 & 0 & 0 \\ 0 & 1 & 0 & 0 \\ 0 & 0 & 0 & 1 \\ 0 & 0 & 1 & 0 \end{pmatrix}$

### Controlled Rotations

```fusion
circuit.crx(0, 1, PI / 4.0)   // Controlled-Rx
circuit.cry(0, 1, PI / 2.0)   // Controlled-Ry
circuit.crz(0, 1, PI)         // Controlled-Rz
circuit.cp(0, 1, PI / 4.0)    // Controlled-Phase
```text

### SWAP Gate

| Gate      | Symbol | Description         |
| :-------- | :----- | :------------------ |
| **SWAP**  | ×─×    | Exchange two qubits |
| **iSWAP** | ×─×    | SWAP with i phase   |
| **√SWAP** | ×─×    | Square root of SWAP |

$SWAP = \begin{pmatrix} 1 & 0 & 0 & 0 \\ 0 & 0 & 1 & 0 \\ 0 & 1 & 0 & 0 \\ 0 & 0 & 0 & 1 \end{pmatrix}$

```fusion
circuit.swap(0, 1)
circuit.iswap(0, 1)
```text

### Two-Qubit Rotations

```fusion
circuit.rxx(0, 1, theta)  // XX rotation
circuit.ryy(0, 1, theta)  // YY rotation
circuit.rzz(0, 1, theta)  // ZZ rotation
```text

---

## Three-Qubit Gates

### Toffoli Gate (CCNOT)

| Gate        | Symbol | Description               |
| :---------- | :----- | :------------------------ |
| **Toffoli** | ●─●─X  | Controlled-Controlled-NOT |

```fusion
circuit.ccnot(0, 1, 2)    // Controls: 0, 1; Target: 2
circuit.toffoli(0, 1, 2)  // Alias
circuit.ccx(0, 1, 2)      // Alias
```text

### Fredkin Gate (CSWAP)

| Gate        | Symbol | Description     |
| :---------- | :----- | :-------------- |
| **Fredkin** | ●─×─×  | Controlled-SWAP |

```fusion
circuit.cswap(0, 1, 2)    // Control: 0; Swap: 1, 2
circuit.fredkin(0, 1, 2)  // Alias
```text

---

## Multi-Qubit Gates

### Multi-Controlled Gates

```fusion
// Multi-controlled X (MCX)
circuit.mcx([0, 1, 2], 3)    // Controls: 0,1,2; Target: 3

// Multi-controlled phase
circuit.mcp([0, 1], 2, PI / 4.0)
```text

### Parameterized Multi-Qubit

```fusion
// Apply same gate to multiple qubits
for q in 0..n {
    circuit.h(q)
}

// Barrier (prevents optimization across boundary)
circuit.barrier([0, 1, 2])
```text

---

## Special Operations

### Measurement

```fusion
// Measure single qubit
circuit.measure(0, 0)     // Qubit 0 → Classical bit 0

// Measure all qubits
circuit.measure_all()

// Measure subset
circuit.measure([0, 1, 2], [0, 1, 2])
```text

### Reset

```fusion
// Reset qubit to |0⟩
circuit.reset(0)

// Reset all qubits
circuit.reset_all()
```text

### Classical Conditioning

```fusion
// Apply gate conditioned on classical bit
circuit.measure(0, 0)
circuit.x(1).c_if(0, 1)   // Apply X if classical bit 0 == 1

// Conditional on register value
circuit.z(2).c_if(classical_reg, 3)
```text

---

## Gate Decompositions

### CNOT Decomposition

CNOT can be decomposed into native gates:

```fusion
// Equivalent to CNOT(0, 1):
circuit.h(1)
circuit.cz(0, 1)
circuit.h(1)
```text

### SWAP Decomposition

```fusion
// SWAP = 3 CNOTs
circuit.cnot(0, 1)
circuit.cnot(1, 0)
circuit.cnot(0, 1)
```text

### Toffoli Decomposition

```fusion
// Toffoli requires ~6 CNOTs and single-qubit gates
// Usually provided natively for efficiency
```text

---

## Gate Properties

### Hermitian Gates

These gates are their own inverse (G² = I):

- X, Y, Z (Pauli gates)
- H (Hadamard)
- CNOT, CZ
- SWAP

```fusion
circuit.h(0)
circuit.h(0)  // Back to original state
```text

### Gate Inverses

```fusion
// Get inverse of a gate
let gate = RY::new(theta)
let inverse = gate.inverse()  // RY(-theta)

// Apply inverse
circuit.ry(0, theta)
circuit.ry(0, -theta)  // Cancels out
```text

### Gate Composition

```fusion
// Compose gates
let gate1 = Hadamard::new()
let gate2 = PauliX::new()
let composed = gate1.compose(&gate2)  // X * H

// Apply composed gate
circuit.apply(composed, 0)
```text

---

## Common Circuit Patterns

### Bell State

```fusion
fn bell_state() -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(2)
    circuit.h(0)
    circuit.cnot(0, 1)
    circuit
}
// Creates: (|00⟩ + |11⟩) / √2
```text

### GHZ State

```fusion
fn ghz_state(n: int) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(n)
    circuit.h(0)
    for i in 0..(n - 1) {
        circuit.cnot(i, i + 1)
    }
    circuit
}
// Creates: (|00...0⟩ + |11...1⟩) / √2
```text

### Quantum Fourier Transform

```fusion
fn qft(n: int) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(n)

    for j in 0..n {
        circuit.h(j)
        for k in (j + 1)..n {
            let phase = PI / (1 << (k - j)) as f64
            circuit.cp(k, j, phase)
        }
    }

    // Swap qubits for correct output ordering
    for i in 0..(n / 2) {
        circuit.swap(i, n - 1 - i)
    }

    circuit
}
```text

### Phase Estimation

```fusion
fn phase_estimation(precision: int, unitary: &QuantumCircuit) -> QuantumCircuit {
    let n = precision + 1  // Precision bits + ancilla
    let mut circuit = QuantumCircuit::new(n)

    // Initialize ancilla in eigenstate (assumed |1⟩)
    circuit.x(n - 1)

    // Hadamard on precision qubits
    for i in 0..precision {
        circuit.h(i)
    }

    // Controlled unitary applications
    for i in 0..precision {
        let power = 1 << (precision - 1 - i)
        for _ in 0..power {
            circuit.append_controlled(&unitary, i, precision)
        }
    }

    // Inverse QFT on precision qubits
    circuit.append(&qft_inverse(precision))

    circuit
}
```text

---

## Quick Reference Table

| Gate | Syntax           | Qubits | Use Case         |
| :--- | :--------------- | :----- | :--------------- |
| H    | `h(q)`           | 1      | Superposition    |
| X    | `x(q)`           | 1      | Bit flip         |
| Y    | `y(q)`           | 1      | Bit+phase flip   |
| Z    | `z(q)`           | 1      | Phase flip       |
| S    | `s(q)`           | 1      | π/2 phase        |
| T    | `t(q)`           | 1      | π/4 phase        |
| Rx   | `rx(q, θ)`       | 1      | X rotation       |
| Ry   | `ry(q, θ)`       | 1      | Y rotation       |
| Rz   | `rz(q, θ)`       | 1      | Z rotation       |
| CNOT | `cnot(c, t)`     | 2      | Entanglement     |
| CZ   | `cz(c, t)`       | 2      | Controlled phase |
| SWAP | `swap(a, b)`     | 2      | Exchange qubits  |
| CCX  | `ccx(c1, c2, t)` | 3      | Toffoli          |

---

This appendix provides a reference for quantum gates in Fusion. For detailed usage, see Chapter 17: Quantum Computing.

[Back to Table of Contents](./README.md)