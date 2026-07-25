// src/quantum/circuit.fu - Core Structures for Quantum Circuits

use fusion::collections::List;
use fusion::runtime::Result;

// --- Circuit Primitives ---

// Represents a single quantum bit
struct Qubit:
    id: u32

// Represents a classical bit used for measurement results
struct ClassicalRegister:
    id: u32
    size: u32

// Base trait for all quantum operations
trait QuantumOperation:
    fn apply(self, circuit: &mut QuantumCircuit);

// --- Quantum Gates (Single-Qubit Operations) ---

// Hadamard gate: creates superposition (H)
struct H { target: Qubit }
implements QuantumOperation for H { ... }

// Pauli X gate: NOT gate (X)
struct X { target: Qubit }
implements QuantumOperation for X { ... }

// Rotation around Z-axis (Rz)
struct Rz { target: Qubit, angle: f64 }
implements QuantumOperation for Rz { ... }

// --- Multi-Qubit Operations ---

// Controlled-NOT gate (CNOT)
struct CNOT { control: Qubit, target: Qubit }
implements QuantumOperation for CNOT { ... }

// Controlled Rotation gate (CRX)
struct CRX { control: Qubit, target: Qubit, angle: f64 }
implements QuantumOperation for CRX { ... }

// --- Measurement ---

// Measures a quantum state and stores the result in a classical register
struct Measure { qubit: Qubit, register: ClassicalRegister }
implements QuantumOperation for Measure { ... }

// --- Main Circuit Structure ---

/// Container for all quantum operations, registers, and hardware targeting.
struct QuantumCircuit:
    name: String
    num_qubits: u32
    operations: List<QuantumOperation>
    
    static fn new(num_qubits: u32) -> QuantumCircuit:
        return QuantumCircuit { name: "default", num_qubits: num_qubits, operations: [] }
    
    // Fluent API for adding gates
    fn h(self, target_idx: u32) -> QuantumCircuit:
        let qubit = Qubit { id: target_idx }
        self.operations.push(H { target: qubit })
        return self
        
    fn cnot(self, control_idx: u32, target_idx: u32) -> QuantumCircuit:
        let control = Qubit { id: control_idx }
        let target = Qubit { id: target_idx }
        self.operations.push(CNOT { control: control, target: target })
        return self

    fn measure_all(self, register: ClassicalRegister) -> QuantumCircuit:
        for i in 0..self.num_qubits:
            self.operations.push(Measure { qubit: Qubit { id: i }, register: register })
        return self