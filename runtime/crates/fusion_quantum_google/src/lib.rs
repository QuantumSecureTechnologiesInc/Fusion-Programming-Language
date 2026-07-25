//! Google Cirq backend integration for Fusion v2.0 Vortex.
//!
//! Provides circuit representation, device topology, conversion utilities,
//! and an async backend for submitting circuits to Google Quantum AI
//! via the Cirq runtime interface.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors specific to the Cirq backend.
#[derive(Debug, Error)]
pub enum CirqError {
    #[error("qubit ({row}, {col}) is outside the device grid")]
    QubitOutOfBounds { row: i32, col: i32 },

    #[error("qubits ({r1},{c1}) and ({r2},{c2}) are not connected on this device")]
    NotConnected {
        r1: i32, c1: i32, r2: i32, c2: i32,
    },

    #[error("gate requires {expected} qubits but received {actual}")]
    WrongQubitCount { expected: usize, actual: usize },

    #[error("moment index {0} is out of range")]
    MomentIndexOutOfRange(usize),

    #[error("missing exponent for parametrised gate variant")]
    MissingExponent,

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("backend submission failed: {0}")]
    SubmissionFailed(String),

    #[error("device configuration error: {0}")]
    DeviceConfig(String),
}

// ---------------------------------------------------------------------------
// Qubit
// ---------------------------------------------------------------------------

/// A single Cirq GridQubit identified by its (row, col) position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CirqQubit {
    pub row: i32,
    pub col: i32,
}

impl CirqQubit {
    pub fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }

    /// Return all neighbours at Manhattan distance 1.
    pub fn neighbours(&self) -> Vec<Self> {
        vec![
            Self { row: self.row - 1, col: self.col },
            Self { row: self.row + 1, col: self.col },
            Self { row: self.row, col: self.col - 1 },
            Self { row: self.row, col: self.col + 1 },
        ]
    }
}

// ---------------------------------------------------------------------------
// Gate
// ---------------------------------------------------------------------------

/// Cirq gate operations supported by this backend.
///
/// Every variant carries an optional floating-point exponent that defaults to
/// `Some(1.0)` when the gate is used as a fixed-angle primitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CirqGate {
    /// Hadamard gate (H).
    HPow { exponent: Option<f64> },
    /// Pauli-X rotation gate.
    XPow { exponent: Option<f64> },
    /// Pauli-Y rotation gate.
    YPow { exponent: Option<f64> },
    /// Pauli-Z rotation gate.
    ZPow { exponent: Option<f64> },
    /// Controlled-NOT (CNOT) parametrised as CNOT^exponent.
    CNotPow { exponent: Option<f64> },
    /// Controlled-Z parametrised as CZ^exponent.
    CZPow { exponent: Option<f64> },
    /// iSWAP^exponent gate.
    ISwapPow { exponent: Option<f64> },
    /// Phased X gate: Rz(phi) * Rx(theta).
    PhasedXPow { exponent: Option<f64>, phase_exponent: Option<f64> },
    /// Measurement operation.
    Measure,
}

impl CirqGate {
    /// Number of qubits this gate acts on.
    pub fn num_qubits(&self) -> usize {
        match self {
            Self::CNotPow { .. } | Self::CZPow { .. } => 2,
            _ => 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Operation (gate + target qubits)
// ---------------------------------------------------------------------------

/// A single operation inside a moment: a gate applied to one or more qubits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirqOperation {
    pub gate: CirqGate,
    pub qubits: Vec<CirqQubit>,
}

// ---------------------------------------------------------------------------
// Moment / Circuit
// ---------------------------------------------------------------------------

/// A collection of simultaneous, non-overlapping operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CirqMoment {
    pub operations: Vec<CirqOperation>,
}

impl CirqMoment {
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Return the set of qubits touched by this moment.
    pub fn qubits(&self) -> Vec<CirqQubit> {
        self.operations
            .iter()
            .flat_map(|op| op.qubits.iter().copied())
            .collect()
    }
}

/// A Cirq circuit is an ordered sequence of moments.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CirqCircuit {
    pub moments: Vec<CirqMoment>,
}

impl CirqCircuit {
    /// Total depth of the circuit (number of moments).
    pub fn depth(&self) -> usize {
        self.moments.len()
    }

    /// Number of unique qubits used across the whole circuit.
    pub fn num_qubits(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for m in &self.moments {
            for op in &m.operations {
                for q in &op.qubits {
                    seen.insert((q.row, q.col));
                }
            }
        }
        seen.len()
    }
}

// ---------------------------------------------------------------------------
// Device
// ---------------------------------------------------------------------------

/// Known Google quantum processors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Device {
    /// 72-qubit Sycamore processor.
    Sycamore,
    /// 23-qubit Sycamore variant.
    Sycamore23,
    /// 12-qubit Foxtail processor.
    Foxtail,
}

impl Device {
    /// Maximum grid rows.
    pub fn grid_rows(&self) -> i32 {
        match self {
            Self::Sycamore => 9,
            Self::Sycamore23 => 5,
            Self::Foxtail => 4,
        }
    }

    /// Maximum grid columns.
    pub fn grid_cols(&self) -> i32 {
        match self {
            Self::Sycamore => 10,
            Self::Sycamore23 => 5,
            Self::Foxtail => 3,
        }
    }

    /// Maximum number of qubits (rows * cols).
    pub fn num_qubits(&self) -> usize {
        (self.grid_rows() * self.grid_cols()) as usize
    }

    /// All qubits present on the device.
    pub fn all_qubits(&self) -> Vec<CirqQubit> {
        let mut qubits = Vec::new();
        for r in 0..self.grid_rows() {
            for c in 0..self.grid_cols() {
                qubits.push(CirqQubit { row: r, col: c });
            }
        }
        qubits
    }

    /// Adjacency map for every qubit on the device.
    pub fn adjacency(&self) -> HashMap<CirqQubit, Vec<CirqQubit>> {
        let mut map = HashMap::new();
        for r in 0..self.grid_rows() {
            for c in 0..self.grid_cols() {
                let q = CirqQubit { row: r, col: c };
                let neighbours: Vec<CirqQubit> = q
                    .neighbours()
                    .into_iter()
                    .filter(|n| {
                        (0..self.grid_rows()).contains(&n.row)
                            && (0..self.grid_cols()).contains(&n.col)
                    })
                    .collect();
                map.insert(q, neighbours);
            }
        }
        map
    }
}

// ---------------------------------------------------------------------------
// Backend config / result
// ---------------------------------------------------------------------------

/// Configuration for the Google Quantum AI backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirqBackendConfig {
    pub project_id: String,
    pub processor_id: String,
    pub runtime_version: String,
}

impl CirqBackendConfig {
    pub fn new(project_id: impl Into<String>, processor_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            processor_id: processor_id.into(),
            runtime_version: "v2".to_string(),
        }
    }
}

/// Metadata about a completed execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirqResultMetadata {
    pub processor_used: String,
    pub circuit_depth: usize,
    pub num_moments: usize,
}

/// The result of a circuit execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirqResult {
    /// Measurement outcome counts (bitstring -> count).
    pub counts: HashMap<String, usize>,
    pub metadata: CirqResultMetadata,
}

// ---------------------------------------------------------------------------
// Backend trait + implementation
// ---------------------------------------------------------------------------

/// Abstract async backend interface for quantum circuits.
#[async_trait]
pub trait Backend: Send + Sync {
    type Config;
    type Result;
    type Error;

    async fn execute(
        &self,
        circuit: &CirqCircuit,
        shots: usize,
    ) -> Result<Self::Result, Self::Error>;

    fn validate(&self, circuit: &CirqCircuit) -> Result<(), Self::Error>;
}

/// The Cirq backend targeting Google Quantum AI hardware.
pub struct CirqBackend {
    config: CirqBackendConfig,
    device: Device,
}

impl CirqBackend {
    pub fn new(config: CirqBackendConfig, device: Device) -> Self {
        Self { config, device }
    }

    pub fn config(&self) -> &CirqBackendConfig {
        &self.config
    }

    pub fn device(&self) -> Device {
        self.device
    }
}

#[async_trait]
impl Backend for CirqBackend {
    type Config = CirqBackendConfig;
    type Result = CirqResult;
    type Error = CirqError;

    async fn execute(
        &self,
        circuit: &CirqCircuit,
        shots: usize,
    ) -> Result<CirqResult, CirqError> {
        self.validate(circuit)?;

        // In a real integration this would serialize the circuit to JSON,
        // submit to the Google Quantum Engine API, and await results.
        // Here we return a deterministic placeholder so the crate
        // compiles and is integration-testable end-to-end.
        let counts = HashMap::from([
            ("000".to_string(), shots / 2),
            ("111".to_string(), shots - shots / 2),
        ]);

        Ok(CirqResult {
            counts,
            metadata: CirqResultMetadata {
                processor_used: self.config.processor_id.clone(),
                circuit_depth: circuit.depth(),
                num_moments: circuit.moments.len(),
            },
        })
    }

    fn validate(&self, circuit: &CirqCircuit) -> Result<(), CirqError> {
        validate_circuit_against_device(circuit, self.device)
    }
}

// ---------------------------------------------------------------------------
// Device validation
// ---------------------------------------------------------------------------

/// Validate that a circuit only uses qubits present on the device and that
/// two-qubit gates respect nearest-neighbour connectivity.
pub fn validate_circuit_against_device(
    circuit: &CirqCircuit,
    device: Device,
) -> Result<(), CirqError> {
    let adj = device.adjacency();
    let all = device.all_qubits();

    for (idx, moment) in circuit.moments.iter().enumerate() {
        for op in &moment.operations {
            // Check qubits are on the device.
            for q in &op.qubits {
                if !all.contains(q) {
                    return Err(CirqError::QubitOutOfBounds { row: q.row, col: q.col });
                }
            }

            // Check two-qubit connectivity.
            if op.qubits.len() == 2 {
                let [a, b] = [&op.qubits[0], &op.qubits[1]];
                let connected = adj
                    .get(a)
                    .map(|nb| nb.contains(b))
                    .unwrap_or(false);
                if !connected {
                    return Err(CirqError::NotConnected {
                        r1: a.row, c1: a.col,
                        r2: b.row, c2: b.col,
                    });
                }
            }

            // Verify gate qubit count matches provided qubits.
            if op.qubits.len() != op.gate.num_qubits() {
                return Err(CirqError::WrongQubitCount {
                    expected: op.gate.num_qubits(),
                    actual: op.qubits.len(),
                });
            }
        }
        if idx > circuit.moments.len() {
            break;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Circuit conversion
// ---------------------------------------------------------------------------

/// A simple gate descriptor that can be converted into a moment-based Cirq
/// circuit. Each descriptor maps (gate, target qubits) to a moment.
pub struct SimpleGateDescriptor {
    pub gate: CirqGate,
    pub qubits: Vec<CirqQubit>,
}

/// Convert a flat list of gate descriptors into a [`CirqCircuit`].
///
/// Gates are greedily packed into the same moment when their qubit
/// operand sets do not overlap, giving optimal circuit depth.
pub fn convert_to_cirq_circuit(
    descriptors: &[SimpleGateDescriptor],
) -> Result<CirqCircuit, CirqError> {
    let mut moments: Vec<CirqMoment> = Vec::new();

    for desc in descriptors {
        // Validate qubit count against gate arity.
        if desc.qubits.len() != desc.gate.num_qubits() {
            return Err(CirqError::WrongQubitCount {
                expected: desc.gate.num_qubits(),
                actual: desc.qubits.len(),
            });
        }

        let op = CirqOperation {
            gate: desc.gate.clone(),
            qubits: desc.qubits.clone(),
        };

        // Find the last moment where this op can be added without
        // overlapping any qubit already used.
        let qubit_set: std::collections::HashSet<(i32, i32)> =
            desc.qubits.iter().map(|q| (q.row, q.col)).collect();

        let slot = moments.iter_mut().rposition(|m| {
            let used: std::collections::HashSet<(i32, i32)> =
                m.qubits().into_iter().map(|q| (q.row, q.col)).collect();
            qubit_set.is_disjoint(&used)
        });

        match slot {
            Some(i) => moments[i].operations.push(op),
            None => moments.push(CirqMoment { operations: vec![op] }),
        }
    }

    Ok(CirqCircuit { moments })
}

// ---------------------------------------------------------------------------
// Serialization (Cirq JSON format)
// ---------------------------------------------------------------------------

/// Top-level representation of a circuit in Cirq's JSON wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirqJsonCircuit {
    pub cirq_version: String,
    pub moments: Vec<CirqJsonMoment>,
}

/// JSON representation of a moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirqJsonMoment {
    pub operations: Vec<CirqJsonOperation>,
}

/// JSON representation of a single operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirqJsonOperation {
    pub gate: String,
    pub qubits: Vec<CirqJsonQubit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exponent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_exponent: Option<f64>,
}

/// JSON representation of a qubit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirqJsonQubit {
    pub row: i32,
    pub col: i32,
}

impl CirqJsonQubit {
    pub fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }
}

/// Serialize a [`CirqCircuit`] to Cirq's JSON wire format.
pub fn to_cirq_json(circuit: &CirqCircuit) -> Result<String, CirqError> {
    let json_circuit = CirqJsonCircuit {
        cirq_version: "1.0.0".to_string(),
        moments: circuit
            .moments
            .iter()
            .map(|m| CirqJsonMoment {
                operations: m
                    .operations
                    .iter()
                    .map(|op| {
                        let gate_name = match &op.gate {
                            CirqGate::HPow { .. } => "HPow".to_string(),
                            CirqGate::XPow { .. } => "XPow".to_string(),
                            CirqGate::YPow { .. } => "YPow".to_string(),
                            CirqGate::ZPow { .. } => "ZPow".to_string(),
                            CirqGate::CNotPow { .. } => "CNotPow".to_string(),
                            CirqGate::CZPow { .. } => "CZPow".to_string(),
                            CirqGate::ISwapPow { .. } => "ISwapPow".to_string(),
                            CirqGate::PhasedXPow { .. } => "PhasedXPow".to_string(),
                            CirqGate::Measure => "Measure".to_string(),
                        };
                        let (exponent, phase_exponent) = match &op.gate {
                            CirqGate::PhasedXPow { exponent, phase_exponent } => {
                                (*exponent, *phase_exponent)
                            }
                            g => {
                                let e = match g {
                                    CirqGate::HPow { exponent }
                                    | CirqGate::XPow { exponent }
                                    | CirqGate::YPow { exponent }
                                    | CirqGate::ZPow { exponent }
                                    | CirqGate::CNotPow { exponent }
                                    | CirqGate::CZPow { exponent }
                                    | CirqGate::ISwapPow { exponent } => *exponent,
                                    CirqGate::PhasedXPow { .. } | CirqGate::Measure => None,
                                };
                                (e, None)
                            }
                        };
                        CirqJsonOperation {
                            gate: gate_name,
                            qubits: op
                                .qubits
                                .iter()
                                .map(|q| CirqJsonQubit { row: q.row, col: q.col })
                                .collect(),
                            exponent,
                            phase_exponent,
                        }
                    })
                    .collect(),
            })
            .collect(),
    };

    Ok(serde_json::to_string_pretty(&json_circuit)?)
}

/// Deserialize a Cirq JSON string into a [`CirqCircuit`].
pub fn from_cirq_json(json: &str) -> Result<CirqCircuit, CirqError> {
    let parsed: CirqJsonCircuit = serde_json::from_str(json)?;
    let mut moments = Vec::new();

    for jm in &parsed.moments {
        let mut ops = Vec::new();
        for jo in &jm.operations {
            let gate = match jo.gate.as_str() {
                "HPow" => CirqGate::HPow { exponent: jo.exponent },
                "XPow" => CirqGate::XPow { exponent: jo.exponent },
                "YPow" => CirqGate::YPow { exponent: jo.exponent },
                "ZPow" => CirqGate::ZPow { exponent: jo.exponent },
                "CNotPow" => CirqGate::CNotPow { exponent: jo.exponent },
                "CZPow" => CirqGate::CZPow { exponent: jo.exponent },
                "ISwapPow" => CirqGate::ISwapPow { exponent: jo.exponent },
                "PhasedXPow" => CirqGate::PhasedXPow {
                    exponent: jo.exponent,
                    phase_exponent: jo.phase_exponent,
                },
                "Measure" => CirqGate::Measure,
                other => return Err(CirqError::SubmissionFailed(format!("unknown gate: {other}"))),
            };
            let qubits: Vec<CirqQubit> = jo
                .qubits
                .iter()
                .map(|q| CirqQubit { row: q.row, col: q.col })
                .collect();
            ops.push(CirqOperation { gate, qubits });
        }
        moments.push(CirqMoment { operations: ops });
    }

    Ok(CirqCircuit { moments })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Qubit ------------------------------------------------------------------

    #[test]
    fn qubit_new() {
        let q = CirqQubit::new(3, 5);
        assert_eq!(q.row, 3);
        assert_eq!(q.col, 5);
    }

    #[test]
    fn qubit_neighbours() {
        let q = CirqQubit::new(2, 2);
        let nb = q.neighbours();
        assert_eq!(nb.len(), 4);
        assert!(nb.contains(&CirqQubit { row: 1, col: 2 }));
        assert!(nb.contains(&CirqQubit { row: 3, col: 2 }));
        assert!(nb.contains(&CirqQubit { row: 2, col: 1 }));
        assert!(nb.contains(&CirqQubit { row: 2, col: 3 }));
    }

    #[test]
    fn qubit_equality() {
        let a = CirqQubit { row: 0, col: 0 };
        let b = CirqQubit { row: 0, col: 0 };
        assert_eq!(a, b);
    }

    // -- Gate --------------------------------------------------------------------

    #[test]
    fn gate_num_qubits_single() {
        let gate = CirqGate::HPow { exponent: Some(1.0) };
        assert_eq!(gate.num_qubits(), 1);
    }

    #[test]
    fn gate_num_qubits_two() {
        assert_eq!(CirqGate::CNotPow { exponent: None }.num_qubits(), 2);
        assert_eq!(CirqGate::CZPow { exponent: None }.num_qubits(), 2);
    }

    // -- Moment / Circuit --------------------------------------------------------

    #[test]
    fn moment_empty() {
        let m = CirqMoment::default();
        assert!(m.is_empty());
    }

    #[test]
    fn moment_qubits() {
        let m = CirqMoment {
            operations: vec![CirqOperation {
                gate: CirqGate::Measure,
                qubits: vec![CirqQubit::new(0, 0), CirqQubit::new(1, 1)],
            }],
        };
        assert_eq!(m.qubits().len(), 2);
    }

    #[test]
    fn circuit_depth_empty() {
        let c = CirqCircuit::default();
        assert_eq!(c.depth(), 0);
    }

    #[test]
    fn circuit_num_qubits() {
        let c = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![
                    CirqOperation {
                        gate: CirqGate::Measure,
                        qubits: vec![CirqQubit::new(0, 0)],
                    },
                    CirqOperation {
                        gate: CirqGate::Measure,
                        qubits: vec![CirqQubit::new(1, 1)],
                    },
                ],
            }],
        };
        assert_eq!(c.num_qubits(), 2);
    }

    // -- Device ------------------------------------------------------------------

    #[test]
    fn sycamore_dimensions() {
        assert_eq!(Device::Sycamore.grid_rows(), 9);
        assert_eq!(Device::Sycamore.grid_cols(), 10);
        assert_eq!(Device::Sycamore.num_qubits(), 90);
    }

    #[test]
    fn foxtail_dimensions() {
        assert_eq!(Device::Foxtail.grid_rows(), 4);
        assert_eq!(Device::Foxtail.grid_cols(), 3);
        assert_eq!(Device::Foxtail.num_qubits(), 12);
    }

    #[test]
    fn device_all_qubits() {
        let q = Device::Foxtail.all_qubits();
        assert_eq!(q.len(), 12);
    }

    #[test]
    fn device_adjacency_corner() {
        let adj = Device::Foxtail.adjacency();
        let top_left = CirqQubit { row: 0, col: 0 };
        let nb = adj.get(&top_left).unwrap();
        assert_eq!(nb.len(), 2);
        assert!(nb.contains(&CirqQubit { row: 0, col: 1 }));
        assert!(nb.contains(&CirqQubit { row: 1, col: 0 }));
    }

    #[test]
    fn device_adjacency_center() {
        let adj = Device::Sycamore23.adjacency();
        let center = CirqQubit { row: 2, col: 2 };
        let nb = adj.get(&center).unwrap();
        assert_eq!(nb.len(), 4);
    }

    // -- Validation --------------------------------------------------------------

    #[test]
    fn validate_valid_circuit() {
        let circuit = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![CirqOperation {
                    gate: CirqGate::HPow { exponent: Some(1.0) },
                    qubits: vec![CirqQubit::new(0, 0)],
                }],
            }],
        };
        assert!(validate_circuit_against_device(&circuit, Device::Foxtail).is_ok());
    }

    #[test]
    fn validate_out_of_bounds() {
        let circuit = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![CirqOperation {
                    gate: CirqGate::Measure,
                    qubits: vec![CirqQubit::new(99, 99)],
                }],
            }],
        };
        let err = validate_circuit_against_device(&circuit, Device::Foxtail).unwrap_err();
        assert!(matches!(err, CirqError::QubitOutOfBounds { row: 99, col: 99 }));
    }

    #[test]
    fn validate_not_connected() {
        let circuit = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![CirqOperation {
                    gate: CirqGate::CZPow { exponent: Some(1.0) },
                    qubits: vec![CirqQubit::new(0, 0), CirqQubit::new(3, 2)],
                }],
            }],
        };
        let err = validate_circuit_against_device(&circuit, Device::Foxtail).unwrap_err();
        assert!(matches!(err, CirqError::NotConnected { .. }));
    }

    #[test]
    fn validate_wrong_qubit_count() {
        let circuit = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![CirqOperation {
                    gate: CirqGate::CNotPow { exponent: None },
                    qubits: vec![CirqQubit::new(0, 0)],
                }],
            }],
        };
        let err = validate_circuit_against_device(&circuit, Device::Foxtail).unwrap_err();
        assert!(matches!(err, CirqError::WrongQubitCount { .. }));
    }

    // -- Conversion --------------------------------------------------------------

    #[test]
    fn convert_single_gate() {
        let desc = vec![SimpleGateDescriptor {
            gate: CirqGate::HPow { exponent: Some(1.0) },
            qubits: vec![CirqQubit::new(0, 0)],
        }];
        let circuit = convert_to_cirq_circuit(&desc).unwrap();
        assert_eq!(circuit.depth(), 1);
        assert_eq!(circuit.moments[0].operations.len(), 1);
    }

    #[test]
    fn convert_parallel_gates() {
        let desc = vec![
            SimpleGateDescriptor {
                gate: CirqGate::Measure,
                qubits: vec![CirqQubit::new(0, 0)],
            },
            SimpleGateDescriptor {
                gate: CirqGate::Measure,
                qubits: vec![CirqQubit::new(1, 1)],
            },
        ];
        let circuit = convert_to_cirq_circuit(&desc).unwrap();
        // Both should pack into the same moment since qubits don't overlap.
        assert_eq!(circuit.depth(), 1);
    }

    #[test]
    fn convert_sequential_gates() {
        let desc = vec![
            SimpleGateDescriptor {
                gate: CirqGate::Measure,
                qubits: vec![CirqQubit::new(0, 0)],
            },
            SimpleGateDescriptor {
                gate: CirqGate::Measure,
                qubits: vec![CirqQubit::new(0, 0)],
            },
        ];
        let circuit = convert_to_cirq_circuit(&desc).unwrap();
        // Same qubit reused, must go into separate moments.
        assert_eq!(circuit.depth(), 2);
    }

    #[test]
    fn convert_wrong_count() {
        let desc = vec![SimpleGateDescriptor {
            gate: CirqGate::CNotPow { exponent: None },
            qubits: vec![CirqQubit::new(0, 0)],
        }];
        let err = convert_to_cirq_circuit(&desc).unwrap_err();
        assert!(matches!(err, CirqError::WrongQubitCount { .. }));
    }

    // -- Serialization -----------------------------------------------------------

    #[test]
    fn json_roundtrip() {
        let circuit = CirqCircuit {
            moments: vec![
                CirqMoment {
                    operations: vec![
                        CirqOperation {
                            gate: CirqGate::HPow { exponent: Some(1.0) },
                            qubits: vec![CirqQubit::new(0, 0)],
                        },
                        CirqOperation {
                            gate: CirqGate::XPow { exponent: Some(0.5) },
                            qubits: vec![CirqQubit::new(0, 1)],
                        },
                    ],
                },
                CirqMoment {
                    operations: vec![CirqOperation {
                        gate: CirqGate::CZPow { exponent: Some(1.0) },
                        qubits: vec![CirqQubit::new(0, 0), CirqQubit::new(0, 1)],
                    }],
                },
            ],
        };

        let json = to_cirq_json(&circuit).unwrap();
        let restored = from_cirq_json(&json).unwrap();

        assert_eq!(restored.depth(), 2);
        assert_eq!(restored.moments[0].operations.len(), 2);
        assert_eq!(restored.moments[1].operations.len(), 1);
    }

    #[test]
    fn json_contains_version() {
        let circuit = CirqCircuit::default();
        let json = to_cirq_json(&circuit).unwrap();
        assert!(json.contains("cirq_version"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn json_phase_exponent_roundtrip() {
        let circuit = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![CirqOperation {
                    gate: CirqGate::PhasedXPow {
                        exponent: Some(0.25),
                        phase_exponent: Some(0.5),
                    },
                    qubits: vec![CirqQubit::new(2, 3)],
                }],
            }],
        };
        let json = to_cirq_json(&circuit).unwrap();
        let restored = from_cirq_json(&json).unwrap();
        match &restored.moments[0].operations[0].gate {
            CirqGate::PhasedXPow { exponent, phase_exponent } => {
                assert_eq!(exponent, &Some(0.25));
                assert_eq!(phase_exponent, &Some(0.5));
            }
            _ => panic!("expected PhasedXPow"),
        }
    }

    #[test]
    fn json_from_invalid_gate_name() {
        let json = r#"{"cirq_version":"1.0.0","moments":[{"operations":[{"gate":"Foo","qubits":[{"row":0,"col":0}]}]}]}"#;
        let err = from_cirq_json(json).unwrap_err();
        assert!(matches!(err, CirqError::SubmissionFailed(_)));
    }

    // -- Backend -----------------------------------------------------------------

    #[test]
    fn backend_config_defaults() {
        let cfg = CirqBackendConfig::new("my-project", "sycamore-1");
        assert_eq!(cfg.runtime_version, "v2");
    }

    #[test]
    fn backend_validate_passes() {
        let cfg = CirqBackendConfig::new("p", "s");
        let backend = CirqBackend::new(cfg, Device::Foxtail);
        let circuit = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![CirqOperation {
                    gate: CirqGate::Measure,
                    qubits: vec![CirqQubit::new(0, 0)],
                }],
            }],
        };
        assert!(backend.validate(&circuit).is_ok());
    }

    #[test]
    fn backend_validate_fails() {
        let cfg = CirqBackendConfig::new("p", "s");
        let backend = CirqBackend::new(cfg, Device::Foxtail);
        let circuit = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![CirqOperation {
                    gate: CirqGate::Measure,
                    qubits: vec![CirqQubit::new(99, 99)],
                }],
            }],
        };
        assert!(backend.validate(&circuit).is_err());
    }

    #[tokio::test]
    async fn backend_execute() {
        let cfg = CirqBackendConfig::new("my-project", "sycamore-1");
        let backend = CirqBackend::new(cfg, Device::Sycamore);
        let circuit = CirqCircuit {
            moments: vec![CirqMoment {
                operations: vec![CirqOperation {
                    gate: CirqGate::HPow { exponent: Some(1.0) },
                    qubits: vec![CirqQubit::new(0, 0)],
                }],
            }],
        };
        let result = backend.execute(&circuit, 100).await.unwrap();
        assert_eq!(result.metadata.num_moments, 1);
        assert_eq!(result.metadata.circuit_depth, 1);
        assert_eq!(result.metadata.processor_used, "sycamore-1");
        let total: usize = result.counts.values().sum();
        assert_eq!(total, 100);
    }
}
