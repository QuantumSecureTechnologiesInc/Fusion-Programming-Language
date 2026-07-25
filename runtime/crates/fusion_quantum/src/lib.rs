//! # Fusion Quantum
//!
//! Quantum computing primitives for the Fusion Runtime.
//!
//! ## Features
//!
//! - **Qubit Representation**: High-level qubit abstraction with complex amplitudes
//! - **Quantum Gates**: Hadamard, Pauli-X/Y/Z, CNOT, Rx, Ry, and more
//! - **Circuit Builder**: Fluent API for building quantum circuits
//! - **QEM Middleware**: Automatic Quantum Error Mitigation
//!
//! ## Architecture
//!
//! The quantum module operates in the "Quantum Plane" of the runtime,
//! interfacing with QPU drivers (IBM Q, Rigetti, or Simulators) through
//! the QEM layer which automatically injects error mitigation.

use fusion_core::{FusionType, QuantumState, QuantumType};
use num_complex::Complex64;
use tracing::{debug, trace};

pub mod qem;

pub use qem::{Circuit as QemCircuit, MitigationError, QemConfig, QemLayer};

/// Qubit representation with full complex amplitude tracking.
///
/// Each qubit maintains its state as |ψ⟩ = α|0⟩ + β|1⟩ where α and β
/// are complex amplitudes satisfying |α|² + |β|² = 1.
pub struct Qubit {
    id: usize,
    alpha: Complex64,
    beta: Complex64,
}

impl Qubit {
    /// Create a new qubit in |0⟩ state
    pub fn new() -> Self {
        debug!("Creating new qubit");
        Self {
            id: 0,
            alpha: Complex64::new(1.0, 0.0),
            beta: Complex64::new(0.0, 0.0),
        }
    }

    /// Create a new qubit with a specific ID
    pub fn with_id(id: usize) -> Self {
        debug!("Creating qubit with id {}", id);
        Self {
            id,
            alpha: Complex64::new(1.0, 0.0),
            beta: Complex64::new(0.0, 0.0),
        }
    }

    /// Get the qubit ID
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get probability of measuring |0⟩
    pub fn prob_zero(&self) -> f64 {
        self.alpha.norm_sqr()
    }

    /// Get probability of measuring |1⟩
    pub fn prob_one(&self) -> f64 {
        self.beta.norm_sqr()
    }

    /// Apply Hadamard gate (creates superposition)
    ///
    /// H|0⟩ = (|0⟩ + |1⟩)/√2
    /// H|1⟩ = (|0⟩ - |1⟩)/√2
    pub fn hadamard(&mut self) {
        trace!("Applying Hadamard gate to qubit {}", self.id);
        let s = 1.0 / 2.0_f64.sqrt();
        let new_alpha = s * (self.alpha + self.beta);
        let new_beta = s * (self.alpha - self.beta);
        self.alpha = new_alpha;
        self.beta = new_beta;
    }

    /// Apply Pauli-X gate (bit flip / NOT)
    ///
    /// X|0⟩ = |1⟩, X|1⟩ = |0⟩
    pub fn pauli_x(&mut self) {
        trace!("Applying Pauli-X gate to qubit {}", self.id);
        std::mem::swap(&mut self.alpha, &mut self.beta);
    }

    /// Apply Pauli-Y gate
    ///
    /// Y|0⟩ = i|1⟩, Y|1⟩ = -i|0⟩
    pub fn pauli_y(&mut self) {
        trace!("Applying Pauli-Y gate to qubit {}", self.id);
        let i = Complex64::i();
        let new_alpha = -i * self.beta;
        let new_beta = i * self.alpha;
        self.alpha = new_alpha;
        self.beta = new_beta;
    }

    /// Apply Pauli-Z gate (phase flip)
    ///
    /// Z|0⟩ = |0⟩, Z|1⟩ = -|1⟩
    pub fn pauli_z(&mut self) {
        trace!("Applying Pauli-Z gate to qubit {}", self.id);
        self.beta = -self.beta;
    }

    /// Apply Rx(θ) rotation gate (rotation around X axis on Bloch sphere)
    ///
    /// Rx(θ) = cos(θ/2)·I - i·sin(θ/2)·X
    pub fn rx(&mut self, theta: f64) {
        trace!("Applying Rx({}) gate to qubit {}", theta, self.id);
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        let is = Complex64::new(0.0, -sin);
        let new_alpha = cos * self.alpha + is * self.beta;
        let new_beta = is * self.alpha + cos * self.beta;
        self.alpha = new_alpha;
        self.beta = new_beta;
    }

    /// Apply Ry(θ) rotation gate (rotation around Y axis on Bloch sphere)
    ///
    /// Ry(θ) = cos(θ/2)·I - sin(θ/2)·Y
    pub fn ry(&mut self, theta: f64) {
        trace!("Applying Ry({}) gate to qubit {}", theta, self.id);
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        let new_alpha = cos * self.alpha - sin * self.beta;
        let new_beta = sin * self.alpha + cos * self.beta;
        self.alpha = new_alpha;
        self.beta = new_beta;
    }

    /// Measure the qubit (collapses superposition)
    ///
    /// Returns 0 or 1 based on the measurement outcome probability.
    pub fn measure(&mut self) -> u8 {
        trace!("Measuring qubit {}", self.id);

        let prob_zero = self.prob_zero();
        let result = if rand_bool_with_prob(prob_zero) { 0 } else { 1 };

        if result == 0 {
            self.alpha = Complex64::new(1.0, 0.0);
            self.beta = Complex64::new(0.0, 0.0);
        } else {
            self.alpha = Complex64::new(0.0, 0.0);
            self.beta = Complex64::new(1.0, 0.0);
        }
        result
    }
}

impl Default for Qubit {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Qubit> for FusionType {
    fn from(qubit: Qubit) -> Self {
        FusionType::Quantum(QuantumType {
            num_qubits: 1,
            state: QuantumState::Simulated(vec![
                (qubit.alpha.re, qubit.alpha.im),
                (qubit.beta.re, qubit.beta.im),
            ]),
            qpu_device: "simulator".to_string(),
        })
    }
}

/// Quantum gate representation
#[derive(Debug, Clone)]
pub enum QuantumGate {
    Hadamard(usize),
    PauliX(usize),
    PauliY(usize),
    PauliZ(usize),
    CNOT(usize, usize),
    Toffoli(usize, usize, usize),
    Measure(usize),
    Rz(usize, f64),
    Rx(usize, f64),
    Ry(usize, f64),
}

impl QuantumGate {
    /// Get the qubit indices this gate operates on.
    fn target_qubits(&self) -> Vec<usize> {
        match self {
            QuantumGate::Hadamard(q)
            | QuantumGate::PauliX(q)
            | QuantumGate::PauliY(q)
            | QuantumGate::PauliZ(q)
            | QuantumGate::Measure(q)
            | QuantumGate::Rz(q, _)
            | QuantumGate::Rx(q, _)
            | QuantumGate::Ry(q, _) => vec![*q],
            QuantumGate::CNOT(c, t) => vec![*c, *t],
            QuantumGate::Toffoli(c1, c2, t) => vec![*c1, *c2, *t],
        }
    }
}

/// Quantum circuit builder
pub struct Circuit {
    num_qubits: usize,
    gates: Vec<QuantumGate>,
}

impl Circuit {
    /// Create a new quantum circuit
    pub fn new(num_qubits: usize) -> Self {
        debug!("Creating quantum circuit with {} qubits", num_qubits);
        Self {
            num_qubits,
            gates: Vec::new(),
        }
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get the number of gates
    pub fn num_gates(&self) -> usize {
        self.gates.len()
    }

    /// Apply Hadamard gate
    pub fn h(&mut self, qubit: usize) -> &mut Self {
        self.gates.push(QuantumGate::Hadamard(qubit));
        self
    }

    /// Apply Pauli-X gate
    pub fn x(&mut self, qubit: usize) -> &mut Self {
        self.gates.push(QuantumGate::PauliX(qubit));
        self
    }

    /// Apply Pauli-Y gate
    pub fn y(&mut self, qubit: usize) -> &mut Self {
        self.gates.push(QuantumGate::PauliY(qubit));
        self
    }

    /// Apply Pauli-Z gate
    pub fn z(&mut self, qubit: usize) -> &mut Self {
        self.gates.push(QuantumGate::PauliZ(qubit));
        self
    }

    /// Apply CNOT gate
    pub fn cx(&mut self, control: usize, target: usize) -> &mut Self {
        self.gates.push(QuantumGate::CNOT(control, target));
        self
    }

    /// Apply Toffoli (CCNOT) gate
    pub fn ccx(&mut self, control1: usize, control2: usize, target: usize) -> &mut Self {
        self.gates
            .push(QuantumGate::Toffoli(control1, control2, target));
        self
    }

    /// Apply measurement
    pub fn measure(&mut self, qubit: usize) -> &mut Self {
        self.gates.push(QuantumGate::Measure(qubit));
        self
    }

    /// Apply Rz rotation (rotation around Z axis)
    pub fn rz(&mut self, qubit: usize, angle: f64) -> &mut Self {
        self.gates.push(QuantumGate::Rz(qubit, angle));
        self
    }

    /// Apply Rx rotation (rotation around X axis)
    pub fn rx(&mut self, qubit: usize, angle: f64) -> &mut Self {
        self.gates.push(QuantumGate::Rx(qubit, angle));
        self
    }

    /// Apply Ry rotation (rotation around Y axis)
    pub fn ry(&mut self, qubit: usize, angle: f64) -> &mut Self {
        self.gates.push(QuantumGate::Ry(qubit, angle));
        self
    }

    /// Get the circuit depth (longest path of dependent gates).
    ///
    /// Gates on the same qubit cannot execute in parallel, so the depth
    /// is the maximum number of sequential gates on any single qubit.
    pub fn depth(&self) -> usize {
        if self.gates.is_empty() {
            return 0;
        }
        // Track the current layer for each qubit
        let mut qubit_layers = vec![0usize; self.num_qubits];

        for gate in &self.gates {
            let qubits_used = gate.target_qubits();
            if qubits_used.is_empty() {
                continue;
            }
            // The layer for this gate is one past the max layer of its target qubits
            let max_layer = qubits_used.iter().map(|&q| qubit_layers[q]).max().unwrap_or(0);
            let new_layer = max_layer + 1;
            for &q in &qubits_used {
                qubit_layers[q] = new_layer;
            }
        }

        qubit_layers.into_iter().max().unwrap_or(0)
    }

    /// Execute circuit (simulation mode) using full state vector simulation.
    ///
    /// Initializes the state vector to |0...0⟩, applies each gate as a
    /// unitary transformation, then samples `shots` measurements from the
    /// resulting probability distribution.
    ///
    /// Returns measurement counts as a map of bitstring to count.
    pub fn execute(&self, shots: u32) -> CircuitResult {
        debug!(
            "Executing circuit with {} gates, {} qubits, {} shots",
            self.gates.len(),
            self.num_qubits,
            shots
        );

        let n = self.num_qubits;
        let dim = 1usize << n;

        // Initialize state vector to |0...0⟩
        let mut state = vec![Complex64::new(0.0, 0.0); dim];
        state[0] = Complex64::new(1.0, 0.0);

        // Apply each gate to the state vector
        for gate in &self.gates {
            match gate {
                QuantumGate::Hadamard(q) => apply_single_qubit_gate(
                    &mut state,
                    *q,
                    n,
                    Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0),
                    Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0),
                    Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0),
                    Complex64::new(-1.0 / 2.0_f64.sqrt(), 0.0),
                ),
                QuantumGate::PauliX(q) => apply_single_qubit_gate(
                    &mut state,
                    *q,
                    n,
                    Complex64::new(0.0, 0.0),
                    Complex64::new(1.0, 0.0),
                    Complex64::new(1.0, 0.0),
                    Complex64::new(0.0, 0.0),
                ),
                QuantumGate::PauliY(q) => apply_single_qubit_gate(
                    &mut state,
                    *q,
                    n,
                    Complex64::new(0.0, 0.0),
                    Complex64::new(0.0, -1.0),
                    Complex64::new(0.0, 1.0),
                    Complex64::new(0.0, 0.0),
                ),
                QuantumGate::PauliZ(q) => apply_single_qubit_gate(
                    &mut state,
                    *q,
                    n,
                    Complex64::new(1.0, 0.0),
                    Complex64::new(0.0, 0.0),
                    Complex64::new(0.0, 0.0),
                    Complex64::new(-1.0, 0.0),
                ),
                QuantumGate::Rz(q, angle) => {
                    let phase0 = Complex64::new(0.0, -*angle / 2.0).exp();
                    let phase1 = Complex64::new(0.0, *angle / 2.0).exp();
                    apply_single_qubit_gate(
                        &mut state,
                        *q,
                        n,
                        phase0,
                        Complex64::new(0.0, 0.0),
                        Complex64::new(0.0, 0.0),
                        phase1,
                    );
                }
                QuantumGate::Rx(q, angle) => {
                    let cos = (angle / 2.0).cos();
                    let sin = (angle / 2.0).sin();
                    apply_single_qubit_gate(
                        &mut state,
                        *q,
                        n,
                        Complex64::new(cos, 0.0),
                        Complex64::new(0.0, -sin),
                        Complex64::new(0.0, -sin),
                        Complex64::new(cos, 0.0),
                    );
                }
                QuantumGate::Ry(q, angle) => {
                    let cos = (angle / 2.0).cos();
                    let sin = (angle / 2.0).sin();
                    apply_single_qubit_gate(
                        &mut state,
                        *q,
                        n,
                        Complex64::new(cos, 0.0),
                        Complex64::new(-sin, 0.0),
                        Complex64::new(sin, 0.0),
                        Complex64::new(cos, 0.0),
                    );
                }
                QuantumGate::CNOT(control, target) => {
                    apply_cnot(&mut state, *control, *target, n);
                }
                QuantumGate::Toffoli(c1, c2, target) => {
                    apply_toffoli(&mut state, *c1, *c2, *target, n);
                }
                QuantumGate::Measure(_) => {
                    // Measurement gates are handled during sampling
                }
            }
        }

        // Sample from the probability distribution
        let probabilities: Vec<f64> = state.iter().map(|a| a.norm_sqr()).collect();
        let mut counts: std::collections::HashMap<Vec<u8>, u32> = std::collections::HashMap::new();

        for _ in 0..shots {
            let outcome = sample_from_probabilities(&probabilities, n);
            *counts.entry(outcome).or_insert(0) += 1;
        }

        CircuitResult {
            counts: counts.into_iter().collect(),
        }
    }
}

/// Apply a single-qubit gate to the state vector.
///
/// The gate matrix is:
/// ```text
/// | g00  g01 |
/// | g10  g11 |
/// ```
///
/// For each pair of basis states that differ only in the target qubit bit,
/// the gate transforms them according to the matrix multiplication.
fn apply_single_qubit_gate(
    state: &mut [Complex64],
    qubit: usize,
    num_qubits: usize,
    g00: Complex64,
    g01: Complex64,
    g10: Complex64,
    g11: Complex64,
) {
    let dim = state.len();
    let mask = 1usize << (num_qubits - 1 - qubit);

    for i in 0..dim {
        // Only process the lower bit of each pair (where target qubit is 0)
        if i & mask != 0 {
            continue;
        }
        let j = i | mask; // The paired index with target qubit = 1

        let a = state[i];
        let b = state[j];

        state[i] = g00 * a + g01 * b;
        state[j] = g10 * a + g11 * b;
    }
}

/// Apply CNOT gate: flip target qubit if control qubit is |1⟩.
fn apply_cnot(state: &mut [Complex64], control: usize, target: usize, num_qubits: usize) {
    let dim = state.len();
    let ctrl_mask = 1usize << (num_qubits - 1 - control);
    let tgt_mask = 1usize << (num_qubits - 1 - target);

    for i in 0..dim {
        // Only process states where control is 1 and target is 0
        if i & ctrl_mask == 0 || i & tgt_mask != 0 {
            continue;
        }
        let j = i | tgt_mask; // Flip the target bit
        // Swap amplitudes to apply NOT on target when control is 1
        state.swap(i, j);
    }
}

/// Apply Toffoli (CCNOT) gate: flip target if both controls are |1⟩.
fn apply_toffoli(
    state: &mut [Complex64],
    ctrl1: usize,
    ctrl2: usize,
    target: usize,
    num_qubits: usize,
) {
    let dim = state.len();
    let c1_mask = 1usize << (num_qubits - 1 - ctrl1);
    let c2_mask = 1usize << (num_qubits - 1 - ctrl2);
    let tgt_mask = 1usize << (num_qubits - 1 - target);

    for i in 0..dim {
        if i & c1_mask == 0 || i & c2_mask == 0 || i & tgt_mask != 0 {
            continue;
        }
        let j = i | tgt_mask;
        state.swap(i, j);
    }
}

/// Sample a measurement outcome from the probability distribution.
fn sample_from_probabilities(probabilities: &[f64], num_qubits: usize) -> Vec<u8> {
    let r = pseudo_random_f64();
    let mut cumulative = 0.0;
    let dim = probabilities.len();

    for i in 0..dim {
        cumulative += probabilities[i];
        if r <= cumulative {
            return index_to_bitstring(i, num_qubits);
        }
    }
    // Fallback (should not reach here if probabilities sum to 1)
    index_to_bitstring(dim - 1, num_qubits)
}

/// Convert a state index to a bitstring (MSB first).
fn index_to_bitstring(index: usize, num_qubits: usize) -> Vec<u8> {
    (0..num_qubits)
        .map(|q| ((index >> (num_qubits - 1 - q)) & 1) as u8)
        .collect()
}

/// Simple xorshift64 PRNG state
static mut RNG_STATE: u64 = 0x1234_5678_9ABC_DEF0;

/// xorshift64 PRNG - produces a full u64, then we take top 53 bits for f64.
fn xorshift64() -> u64 {
    unsafe {
        RNG_STATE ^= RNG_STATE << 13;
        RNG_STATE ^= RNG_STATE >> 7;
        RNG_STATE ^= RNG_STATE << 17;
        RNG_STATE
    }
}

/// Pseudo-random f64 in [0, 1) using xorshift64.
fn pseudo_random_f64() -> f64 {
    let bits = xorshift64();
    // Use top 53 bits for double precision
    (bits >> 11) as f64 / (1u64 << 53) as f64
}

/// Random boolean with configurable probability of true.
fn rand_bool_with_prob(prob_true: f64) -> bool {
    pseudo_random_f64() >= prob_true
}

/// Result from circuit execution
pub struct CircuitResult {
    pub counts: Vec<(Vec<u8>, u32)>,
}

impl CircuitResult {
    /// Get the most frequent measurement outcome
    pub fn most_frequent(&self) -> Option<&Vec<u8>> {
        self.counts
            .iter()
            .max_by_key(|(_, count)| count)
            .map(|(bits, _)| bits)
    }

    /// Get the total number of shots
    pub fn total_shots(&self) -> u32 {
        self.counts.iter().map(|(_, count)| count).sum()
    }

    /// Get the count for a specific bitstring outcome
    pub fn count_for(&self, outcome: &[u8]) -> u32 {
        self.counts
            .iter()
            .find(|(bits, _)| bits == outcome)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qubit_creation() {
        let qubit = Qubit::new();
        assert_eq!(qubit.id(), 0);
        assert!((qubit.prob_zero() - 1.0).abs() < 1e-10);
        assert!((qubit.prob_one()).abs() < 1e-10);
    }

    #[test]
    fn test_qubit_with_id() {
        let qubit = Qubit::with_id(5);
        assert_eq!(qubit.id(), 5);
    }

    #[test]
    fn test_hadamard_gate() {
        let mut qubit = Qubit::new();
        qubit.hadamard();
        assert!((qubit.prob_zero() - 0.5).abs() < 1e-10);
        assert!((qubit.prob_one() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_x_gate() {
        let mut qubit = Qubit::new();
        qubit.pauli_x();
        assert!((qubit.prob_zero()).abs() < 1e-10);
        assert!((qubit.prob_one() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_y_gate() {
        let mut qubit = Qubit::new();
        qubit.pauli_y();
        // Y|0⟩ = i|1⟩ → probability of |1⟩ is 1
        assert!((qubit.prob_zero()).abs() < 1e-10);
        assert!((qubit.prob_one() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_z_gate() {
        let mut qubit = Qubit::new();
        // Z|0⟩ = |0⟩ (no change)
        qubit.pauli_z();
        assert!((qubit.prob_zero() - 1.0).abs() < 1e-10);

        // Z|1⟩ = -|1⟩ (phase flip, but |1⟩ probability is still 1)
        let mut qubit = Qubit::new();
        qubit.pauli_x(); // flip to |1⟩
        qubit.pauli_z();
        assert!((qubit.prob_one() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rx_gate() {
        let mut qubit = Qubit::new();
        qubit.rx(std::f64::consts::PI); // Rx(π) = -iX
        assert!((qubit.prob_one() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ry_gate() {
        let mut qubit = Qubit::new();
        qubit.ry(std::f64::consts::PI); // Ry(π) flips |0⟩ to |1⟩
        assert!((qubit.prob_one() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_qubit_measurement() {
        let mut qubit = Qubit::new();
        qubit.hadamard();
        let result = qubit.measure();
        assert!(result == 0 || result == 1);
        // After measurement, qubit should be in a definite state
        assert!((qubit.prob_zero() - 1.0).abs() < 1e-10
            || (qubit.prob_one() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_circuit_builder() {
        let mut circuit = Circuit::new(2);
        circuit.h(0).cx(0, 1).measure(0).measure(1);

        assert_eq!(circuit.num_gates(), 4);
        assert_eq!(circuit.num_qubits(), 2);
    }

    #[test]
    fn test_circuit_depth_single_qubit() {
        let mut circuit = Circuit::new(1);
        circuit.h(0).x(0).z(0);
        assert_eq!(circuit.depth(), 3);
    }

    #[test]
    fn test_circuit_depth_parallel() {
        let mut circuit = Circuit::new(2);
        circuit.h(0).h(1); // These can run in parallel
        assert_eq!(circuit.depth(), 1);

        circuit.cx(0, 1); // This depends on both
        assert_eq!(circuit.depth(), 2);
    }

    #[test]
    fn test_circuit_execution_zero_state() {
        let circuit = Circuit::new(1);
        let result = circuit.execute(1000);
        assert_eq!(result.total_shots(), 1000);
        // Should always measure |0⟩
        assert_eq!(result.count_for(&vec![0]), 1000);
    }

    #[test]
    fn test_circuit_execution_pauli_x() {
        let mut circuit = Circuit::new(1);
        circuit.x(0);
        let result = circuit.execute(1000);
        // Should always measure |1⟩
        assert_eq!(result.count_for(&vec![1]), 1000);
    }

    #[test]
    fn test_bell_state_circuit() {
        let mut circuit = Circuit::new(2);
        circuit.h(0).cx(0, 1);

        let result = circuit.execute(10000);
        assert_eq!(result.total_shots(), 10000);

        // Bell state should produce ~50% |00⟩ and ~50% |11⟩
        let count_00 = result.count_for(&vec![0, 0]);
        let count_11 = result.count_for(&vec![1, 1]);
        assert!(count_00 > 4000, "Expected ~5000 |00⟩, got {}", count_00);
        assert!(count_11 > 4000, "Expected ~5000 |11⟩, got {}", count_11);
        // Should NOT see |01⟩ or |10⟩ in Bell state
        assert_eq!(result.count_for(&vec![0, 1]), 0);
        assert_eq!(result.count_for(&vec![1, 0]), 0);
    }

    #[test]
    fn test_bell_state_with_measurements() {
        let mut circuit = Circuit::new(2);
        circuit.h(0).cx(0, 1).measure(0).measure(1);

        assert_eq!(circuit.num_gates(), 4);
        assert!(circuit.depth() >= 3);

        let result = circuit.execute(1000);
        assert_eq!(result.total_shots(), 1000);
    }

    #[test]
    fn test_rx_rotation_to_superposition() {
        let mut circuit = Circuit::new(1);
        // Rx(π/2) on |0⟩ should put it in a superposition
        circuit.rx(0, std::f64::consts::FRAC_PI_2);
        let result = circuit.execute(10000);
        let count_0 = result.count_for(&vec![0]);
        let count_1 = result.count_for(&vec![1]);
        // Both outcomes should appear
        assert!(count_0 > 1000, "Expected some |0⟩ outcomes");
        assert!(count_1 > 1000, "Expected some |1⟩ outcomes");
    }

    #[test]
    fn test_ry_rotation() {
        let mut circuit = Circuit::new(1);
        circuit.ry(0, std::f64::consts::PI);
        let result = circuit.execute(1000);
        assert_eq!(result.count_for(&vec![1]), 1000);
    }
}
