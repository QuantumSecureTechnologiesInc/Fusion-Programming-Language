//! Quantum state vector simulator
//! Integrated from fusion_core Quantum Core.rs

use num_complex::Complex64;

/// Quantum state vector
#[derive(Debug, Clone)]
pub struct QuantumState {
    pub amplitudes: Vec<Complex64>,
    pub num_qubits: usize,
}

impl QuantumState {
    /// Create state in |0...0⟩
    pub fn zeros(num_qubits: usize) -> Self {
        let size = 1 << num_qubits; // 2^num_qubits
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); size];
        amplitudes[0] = Complex64::new(1.0, 0.0); // |0⟩ state

        Self {
            amplitudes,
            num_qubits,
        }
    }

    /// Create a Bell state (|00⟩ + |11⟩)/√2 between two qubits
    pub fn bell_state(num_qubits: usize, qubit_a: usize, qubit_b: usize) -> Self {
        assert!(
            qubit_a < num_qubits && qubit_b < num_qubits,
            "Qubit indices out of range"
        );
        assert_ne!(qubit_a, qubit_b, "Qubits must be different for Bell state");

        let size = 1 << num_qubits;
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); size];
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();

        for i in 0..size {
            // Set amplitude for states where both qubits match
            let bit_a = (i >> (num_qubits - 1 - qubit_a)) & 1;
            let bit_b = (i >> (num_qubits - 1 - qubit_b)) & 1;
            if bit_a == bit_b {
                amplitudes[i] = Complex64::new(inv_sqrt2, 0.0);
            }
        }

        Self {
            amplitudes,
            num_qubits,
        }
    }

    /// Get probability of measuring |i⟩
    pub fn probability(&self, state_index: usize) -> f64 {
        if state_index < self.amplitudes.len() {
            self.amplitudes[state_index].norm_sqr()
        } else {
            0.0
        }
    }

    /// Get total probability (should be 1.0 for normalized states)
    pub fn total_probability(&self) -> f64 {
        self.amplitudes.iter().map(|a| a.norm_sqr()).sum()
    }

    /// Normalize the state vector
    pub fn normalize(&mut self) {
        let norm = self.total_probability().sqrt();
        if norm > 1e-10 {
            for amp in &mut self.amplitudes {
                *amp /= norm;
            }
        }
    }

    /// Apply a single-qubit gate to a specific qubit.
    ///
    /// The gate is specified as a 2x2 matrix [g00, g01, g10, g11].
    /// For each pair of basis states differing only in the target qubit,
    /// the gate matrix is applied to transform their amplitudes.
    pub fn apply_single_qubit_gate(
        &mut self,
        qubit: usize,
        g00: Complex64,
        g01: Complex64,
        g10: Complex64,
        g11: Complex64,
    ) {
        assert!(qubit < self.num_qubits, "Qubit index out of range");
        let dim = self.amplitudes.len();
        let mask = 1usize << (self.num_qubits - 1 - qubit);

        for i in 0..dim {
            if i & mask != 0 {
                continue;
            }
            let j = i | mask;
            let a = self.amplitudes[i];
            let b = self.amplitudes[j];
            self.amplitudes[i] = g00 * a + g01 * b;
            self.amplitudes[j] = g10 * a + g11 * b;
        }
    }

    /// Apply Hadamard gate to a specific qubit.
    ///
    /// H = (1/√2) * [[1, 1], [1, -1]]
    pub fn apply_hadamard(&mut self, qubit: usize) {
        let s = 1.0 / 2.0_f64.sqrt();
        self.apply_single_qubit_gate(
            qubit,
            Complex64::new(s, 0.0),
            Complex64::new(s, 0.0),
            Complex64::new(s, 0.0),
            Complex64::new(-s, 0.0),
        );
    }

    /// Apply Pauli-X (NOT) gate to a specific qubit.
    ///
    /// X = [[0, 1], [1, 0]]
    pub fn apply_pauli_x(&mut self, qubit: usize) {
        self.apply_single_qubit_gate(
            qubit,
            Complex64::new(0.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
        );
    }

    /// Apply Pauli-Y gate to a specific qubit.
    ///
    /// Y = [[0, -i], [i, 0]]
    pub fn apply_pauli_y(&mut self, qubit: usize) {
        self.apply_single_qubit_gate(
            qubit,
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, -1.0),
            Complex64::new(0.0, 1.0),
            Complex64::new(0.0, 0.0),
        );
    }

    /// Apply Pauli-Z gate to a specific qubit.
    ///
    /// Z = [[1, 0], [0, -1]]
    pub fn apply_pauli_z(&mut self, qubit: usize) {
        self.apply_single_qubit_gate(
            qubit,
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(-1.0, 0.0),
        );
    }

    /// Apply Rx(θ) rotation gate to a specific qubit.
    ///
    /// Rx(θ) = cos(θ/2)·I - i·sin(θ/2)·X
    pub fn apply_rotation_x(&mut self, qubit: usize, theta: f64) {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        self.apply_single_qubit_gate(
            qubit,
            Complex64::new(cos, 0.0),
            Complex64::new(0.0, -sin),
            Complex64::new(0.0, -sin),
            Complex64::new(cos, 0.0),
        );
    }

    /// Apply Ry(θ) rotation gate to a specific qubit.
    ///
    /// Ry(θ) = cos(θ/2)·I - sin(θ/2)·Y
    pub fn apply_rotation_y(&mut self, qubit: usize, theta: f64) {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        self.apply_single_qubit_gate(
            qubit,
            Complex64::new(cos, 0.0),
            Complex64::new(-sin, 0.0),
            Complex64::new(sin, 0.0),
            Complex64::new(cos, 0.0),
        );
    }

    /// Apply Rz(θ) rotation gate to a specific qubit.
    ///
    /// Rz(θ) = [[e^(-iθ/2), 0], [0, e^(iθ/2)]]
    pub fn apply_rotation_z(&mut self, qubit: usize, theta: f64) {
        let phase0 = Complex64::new(0.0, -theta / 2.0).exp();
        let phase1 = Complex64::new(0.0, theta / 2.0).exp();
        self.apply_single_qubit_gate(
            qubit,
            phase0,
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            phase1,
        );
    }

    /// Apply CNOT gate (control-target pair).
    ///
    /// Flips the target qubit when the control qubit is |1⟩.
    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        assert!(
            control < self.num_qubits && target < self.num_qubits,
            "Qubit indices out of range"
        );
        let dim = self.amplitudes.len();
        let ctrl_mask = 1usize << (self.num_qubits - 1 - control);
        let tgt_mask = 1usize << (self.num_qubits - 1 - target);

        for i in 0..dim {
            if i & ctrl_mask == 0 || i & tgt_mask != 0 {
                continue;
            }
            let j = i | tgt_mask;
            self.amplitudes.swap(i, j);
        }
    }

    /// Apply Toffoli (CCNOT) gate (two controls + target).
    ///
    /// Flips the target qubit when both control qubits are |1⟩.
    pub fn apply_toffoli(&mut self, ctrl1: usize, ctrl2: usize, target: usize) {
        assert!(
            ctrl1 < self.num_qubits
                && ctrl2 < self.num_qubits
                && target < self.num_qubits,
            "Qubit indices out of range"
        );
        let dim = self.amplitudes.len();
        let c1_mask = 1usize << (self.num_qubits - 1 - ctrl1);
        let c2_mask = 1usize << (self.num_qubits - 1 - ctrl2);
        let tgt_mask = 1usize << (self.num_qubits - 1 - target);

        for i in 0..dim {
            if i & c1_mask == 0 || i & c2_mask == 0 || i & tgt_mask != 0 {
                continue;
            }
            let j = i | tgt_mask;
            self.amplitudes.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = QuantumState::zeros(2);
        assert_eq!(state.num_qubits, 2);
        assert_eq!(state.amplitudes.len(), 4);
        assert_eq!(state.probability(0), 1.0);
    }

    #[test]
    fn test_normalization() {
        let mut state = QuantumState {
            amplitudes: vec![Complex64::new(1.0, 0.0), Complex64::new(1.0, 0.0)],
            num_qubits: 1,
        };

        state.normalize();
        let total = state.total_probability();
        assert!((total - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_hadamard_on_zero() {
        let mut state = QuantumState::zeros(1);
        state.apply_hadamard(0);

        // After H|0⟩, both amplitudes should be 1/√2
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        assert!((state.amplitudes[0].re - inv_sqrt2).abs() < 1e-10);
        assert!((state.amplitudes[1].re - inv_sqrt2).abs() < 1e-10);
        assert!((state.total_probability() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_x_on_zero() {
        let mut state = QuantumState::zeros(1);
        state.apply_pauli_x(0);

        // X|0⟩ = |1⟩
        assert!((state.probability(0)).abs() < 1e-10);
        assert!((state.probability(1) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cnot_creates_bell_state() {
        let mut state = QuantumState::zeros(2);
        state.apply_hadamard(0);
        state.apply_cnot(0, 1);

        // Bell state: |00⟩/√2 + |11⟩/√2
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        assert!((state.probability(0) - 0.5).abs() < 1e-10); // |00⟩
        assert!((state.probability(1)).abs() < 1e-10); // |01⟩
        assert!((state.probability(2)).abs() < 1e-10); // |10⟩
        assert!((state.probability(3) - 0.5).abs() < 1e-10); // |11⟩
    }

    #[test]
    fn test_bell_state_constructor() {
        let state = QuantumState::bell_state(2, 0, 1);
        assert!((state.probability(0) - 0.5).abs() < 1e-10);
        assert!((state.probability(3) - 0.5).abs() < 1e-10);
        assert!((state.probability(1)).abs() < 1e-10);
        assert!((state.probability(2)).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_x_pi_flips() {
        let mut state = QuantumState::zeros(1);
        state.apply_rotation_x(0, std::f64::consts::PI);
        // Rx(π)|0⟩ = -i|1⟩
        assert!((state.probability(0)).abs() < 1e-10);
        assert!((state.probability(1) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_y_pi_flips() {
        let mut state = QuantumState::zeros(1);
        state.apply_rotation_y(0, std::f64::consts::PI);
        // Ry(π)|0⟩ = |1⟩
        assert!((state.probability(0)).abs() < 1e-10);
        assert!((state.probability(1) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_z_on_zero_no_change() {
        let mut state = QuantumState::zeros(1);
        state.apply_rotation_z(0, std::f64::consts::PI);
        // Rz(π)|0⟩ = e^(-iπ/2)|0⟩ → still |0⟩ with phase
        assert!((state.probability(0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_toffoli() {
        let mut state = QuantumState::zeros(3);
        // Set qubits 1 and 2 to |1⟩ (qubit 0 stays |0⟩)
        // State: q0=0, q1=1, q2=1 → index = 0*4 + 1*2 + 1*1 = 3 (|011⟩)
        state.apply_pauli_x(1);
        state.apply_pauli_x(2);

        // Apply Toffoli: ctrl1=qubit 2, ctrl2=qubit 1, target=qubit 0
        // Both controls are 1, so target flips → q0=1, q1=1, q2=1 → index 7 (|111⟩)
        state.apply_toffoli(2, 1, 0);
        assert!((state.probability(7) - 1.0).abs() < 1e-10);

        // Now test with only one control set (ctrl1=qubit 2 = 1, ctrl2=qubit 1 = 0)
        let mut state2 = QuantumState::zeros(3);
        state2.apply_pauli_x(2); // q2=1, q0=0, q1=0 → index 1 (|001⟩)
        state2.apply_toffoli(2, 1, 0);
        // Only ctrl1 is 1, ctrl2 is 0 → no flip, stays |001⟩
        assert!((state2.probability(1) - 1.0).abs() < 1e-10);
    }
}
