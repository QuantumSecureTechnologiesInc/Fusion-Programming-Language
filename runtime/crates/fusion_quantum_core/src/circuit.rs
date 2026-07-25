//! Quantum circuit construction
//! Integrated from fusion_core Quantum Core.rs

use crate::error::{QuantumError, QuantumResult};
use crate::gates::QuantumGate;

/// Quantum circuit
#[derive(Debug, Clone)]
pub struct QuantumCircuit {
    pub num_qubits: usize,
    pub gates: Vec<(QuantumGate, Vec<usize>)>,
}

impl QuantumCircuit {
    /// Create a new quantum circuit with specified number of qubits
    pub fn new(num_qubits: usize) -> Self {
        Self {
            num_qubits,
            gates: Vec::new(),
        }
    }

    /// Apply a gate to specified qubits
    pub fn apply_gate(&mut self, gate: QuantumGate, targets: Vec<usize>) -> QuantumResult<()> {
        // Validate gate arity
        if gate.num_qubits != targets.len() {
            return Err(QuantumError::GateArityMismatch {
                gate: gate.name.clone(),
                required: gate.num_qubits,
                provided: targets.len(),
            });
        }

        // Validate qubit indices
        for &t in &targets {
            if t >= self.num_qubits {
                return Err(QuantumError::InvalidQubitAccess(t));
            }
        }

        self.gates.push((gate, targets));
        Ok(())
    }

    /// Get the number of gates in the circuit
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Get circuit depth (longest path of sequential gate dependencies).
    ///
    /// Two gates conflict if they operate on the same qubit. The depth is
    /// the maximum number of sequential gates on any single qubit, counting
    /// gates on different qubits as parallelizable.
    pub fn depth(&self) -> usize {
        if self.gates.is_empty() {
            return 0;
        }

        // Track the current layer (time step) for each qubit.
        // A gate can execute only after all its target qubits have completed
        // the previous layer. It then advances each target qubit by one layer.
        let mut qubit_layers = vec![0usize; self.num_qubits];

        for (_gate, targets) in &self.gates {
            // The earliest this gate can execute is one past the max layer
            // of all its target qubits
            let max_layer = targets.iter().map(|&q| qubit_layers[q]).max().unwrap_or(0);
            let new_layer = max_layer + 1;
            for &q in targets {
                qubit_layers[q] = new_layer;
            }
        }

        qubit_layers.into_iter().max().unwrap_or(0)
    }

    /// Get the qubits touched by a gate at the given index.
    pub fn gate_qubits(&self, index: usize) -> Option<&[usize]> {
        self.gates.get(index).map(|(_, targets)| targets.as_slice())
    }

    /// Get a list of all gates that operate on a specific qubit.
    pub fn gates_on_qubit(&self, qubit: usize) -> Vec<(usize, &QuantumGate)> {
        self.gates
            .iter()
            .enumerate()
            .filter(|(_, (_, targets))| targets.contains(&qubit))
            .map(|(idx, (gate, _))| (idx, gate))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::QuantumGate;
    use fusion_tensor_core::Matrix;
    use num_complex::Complex64;

    #[test]
    fn test_circuit_creation() {
        let circuit = QuantumCircuit::new(2);
        assert_eq!(circuit.num_qubits, 2);
        assert_eq!(circuit.gate_count(), 0);
    }

    #[test]
    fn test_apply_gate() {
        let mut circuit = QuantumCircuit::new(2);

        let h_gate = QuantumGate {
            name: "H".to_string(),
            matrix: Matrix::from_vec(
                vec![
                    Complex64::new(1.0, 0.0),
                    Complex64::new(1.0, 0.0),
                    Complex64::new(1.0, 0.0),
                    Complex64::new(-1.0, 0.0),
                ],
                [2, 2],
            )
            .unwrap(),
            num_qubits: 1,
        };

        let result = circuit.apply_gate(h_gate, vec![0]);
        assert!(result.is_ok());
        assert_eq!(circuit.gate_count(), 1);
    }

    #[test]
    fn test_invalid_qubit() {
        let mut circuit = QuantumCircuit::new(2);

        let h_gate = QuantumGate {
            name: "H".to_string(),
            matrix: Matrix::zeros([2, 2]),
            num_qubits: 1,
        };

        let result = circuit.apply_gate(h_gate, vec![5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_depth_empty() {
        let circuit = QuantumCircuit::new(2);
        assert_eq!(circuit.depth(), 0);
    }

    #[test]
    fn test_depth_sequential() {
        let mut circuit = QuantumCircuit::new(1);
        let gate = || QuantumGate {
            name: "X".to_string(),
            matrix: Matrix::zeros([2, 2]),
            num_qubits: 1,
        };

        circuit.apply_gate(gate(), vec![0]).unwrap();
        circuit.apply_gate(gate(), vec![0]).unwrap();
        circuit.apply_gate(gate(), vec![0]).unwrap();

        // Three gates on same qubit = depth 3
        assert_eq!(circuit.depth(), 3);
    }

    #[test]
    fn test_depth_parallel() {
        let mut circuit = QuantumCircuit::new(2);
        let gate = || QuantumGate {
            name: "H".to_string(),
            matrix: Matrix::zeros([2, 2]),
            num_qubits: 1,
        };

        // H on qubit 0 and H on qubit 1 can run in parallel
        circuit.apply_gate(gate(), vec![0]).unwrap();
        circuit.apply_gate(gate(), vec![1]).unwrap();

        assert_eq!(circuit.depth(), 1);

        // CNOT depends on both, so depth increases by 1
        let cnot = QuantumGate {
            name: "CNOT".to_string(),
            matrix: Matrix::zeros([4, 4]),
            num_qubits: 2,
        };
        circuit.apply_gate(cnot, vec![0, 1]).unwrap();

        assert_eq!(circuit.depth(), 2);
    }

    #[test]
    fn test_gates_on_qubit() {
        let mut circuit = QuantumCircuit::new(3);
        let gate = |name: &str| QuantumGate {
            name: name.to_string(),
            matrix: Matrix::zeros([2, 2]),
            num_qubits: 1,
        };

        circuit.apply_gate(gate("H"), vec![0]).unwrap();
        circuit.apply_gate(gate("X"), vec![1]).unwrap();
        circuit.apply_gate(gate("Z"), vec![0]).unwrap();

        let gates_q0 = circuit.gates_on_qubit(0);
        assert_eq!(gates_q0.len(), 2);
        assert_eq!(gates_q0[0].1.name, "H");
        assert_eq!(gates_q0[1].1.name, "Z");

        let gates_q1 = circuit.gates_on_qubit(1);
        assert_eq!(gates_q1.len(), 1);
        assert_eq!(gates_q1[0].1.name, "X");
    }
}
