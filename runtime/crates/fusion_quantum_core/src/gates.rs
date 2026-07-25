//! Standard quantum gates
//! Integrated from fusion_core Quantum Operations.rs

use fusion_tensor_core::{Matrix, TensorOps};
use fusion_traits::Unitary;
use num_complex::Complex64;

/// Quantum gate representation
#[derive(Debug, Clone)]
pub struct QuantumGate {
    pub name: String,
    pub matrix: Matrix<Complex64>,
    pub num_qubits: usize,
}

impl Unitary for QuantumGate {
    fn matrix(&self) -> Vec<Vec<Complex64>> {
        // Convert Matrix<Complex64> to Vec<Vec<Complex64>>
        let (rows, cols) = self.matrix.dims();
        let mut result = Vec::with_capacity(rows);

        for i in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for j in 0..cols {
                row.push(self.matrix.at(i, j).unwrap_or(Complex64::new(0.0, 0.0)));
            }
            result.push(row);
        }
        result
    }

    fn adjoint(&self) -> Self {
        // Conjugate transpose: (A†)ᵢⱼ = conj(Aⱼᵢ)
        let transposed = self.matrix.clone().transpose();
        // Conjugate each element (negate imaginary part)
        let (rows, cols) = transposed.dims();
        let mut conjugated = transposed;
        for i in 0..rows {
            for j in 0..cols {
                if let Ok(val) = conjugated.at(i, j) {
                    let _ = conjugated.set_at(i, j, val.conj());
                }
            }
        }
        Self {
            name: format!("{}†", self.name),
            matrix: conjugated,
            num_qubits: self.num_qubits,
        }
    }

    fn num_qubits(&self) -> usize {
        self.num_qubits
    }
}

impl QuantumGate {
    /// Hadamard gate
    pub fn hadamard() -> Self {
        let s = 1.0 / 2.0_f64.sqrt();
        let matrix = Matrix::from_vec(
            vec![
                Complex64::new(s, 0.0),
                Complex64::new(s, 0.0),
                Complex64::new(s, 0.0),
                Complex64::new(-s, 0.0),
            ],
            [2, 2],
        )
        .unwrap();

        Self {
            name: "H".to_string(),
            matrix,
            num_qubits: 1,
        }
    }

    /// Pauli-X (NOT) gate
    pub fn pauli_x() -> Self {
        let matrix = Matrix::from_vec(
            vec![
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [2, 2],
        )
        .unwrap();

        Self {
            name: "X".to_string(),
            matrix,
            num_qubits: 1,
        }
    }

    /// Pauli-Y gate
    pub fn pauli_y() -> Self {
        let matrix = Matrix::from_vec(
            vec![
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, -1.0),
                Complex64::new(0.0, 1.0),
                Complex64::new(0.0, 0.0),
            ],
            [2, 2],
        )
        .unwrap();

        Self {
            name: "Y".to_string(),
            matrix,
            num_qubits: 1,
        }
    }

    /// Pauli-Z gate
    pub fn pauli_z() -> Self {
        let matrix = Matrix::from_vec(
            vec![
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(-1.0, 0.0),
            ],
            [2, 2],
        )
        .unwrap();

        Self {
            name: "Z".to_string(),
            matrix,
            num_qubits: 1,
        }
    }

    /// CNOT (Controlled-NOT) gate
    pub fn cnot() -> Self {
        let matrix = Matrix::from_vec(
            vec![
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [4, 4],
        )
        .unwrap();

        Self {
            name: "CNOT".to_string(),
            matrix,
            num_qubits: 2,
        }
    }

    /// Rotation Y gate
    pub fn ry(theta: f64) -> Self {
        let cos_t = (theta / 2.0).cos();
        let sin_t = (theta / 2.0).sin();

        let matrix = Matrix::from_vec(
            vec![
                Complex64::new(cos_t, 0.0),
                Complex64::new(-sin_t, 0.0),
                Complex64::new(sin_t, 0.0),
                Complex64::new(cos_t, 0.0),
            ],
            [2, 2],
        )
        .unwrap();

        Self {
            name: format!("Ry({:.3})", theta),
            matrix,
            num_qubits: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hadamard() {
        let h = QuantumGate::hadamard();
        assert_eq!(h.num_qubits, 1);
        assert_eq!(h.name, "H");
    }

    #[test]
    fn test_pauli_gates() {
        let x = QuantumGate::pauli_x();
        let y = QuantumGate::pauli_y();
        let z = QuantumGate::pauli_z();

        assert_eq!(x.num_qubits, 1);
        assert_eq!(y.num_qubits, 1);
        assert_eq!(z.num_qubits, 1);
    }

    #[test]
    fn test_cnot() {
        let cnot = QuantumGate::cnot();
        assert_eq!(cnot.num_qubits, 2);
    }

    #[test]
    fn test_unitary_trait() {
        let h = QuantumGate::hadamard();
        assert_eq!(h.num_qubits(), 1);

        let h_adj = h.adjoint();
        assert!(h_adj.name.contains("†"));
    }
}
