//! # Fusion Quantum AWS
//!
//! AWS Braket backend integration for the Fusion v2.0 Vortex runtime.
//!
//! Provides circuit serialization to Amazon Braket IR, device selection
//! across SV1, IonQ, and Rigetti backends, job tracking, and result retrieval.
//!
//! ## Supported Devices
//!
//! | Device        | Provider  | Qubits | Native Gates                    |
//! |---------------|-----------|--------|---------------------------------|
//! | SV1           | Amazon    | 34     | H, CNOT, Rx, Ry, Rz, X, Y, Z  |
//! | IonQ Harmony  | IonQ      | 11     | GPI, GPI2, MS, X, Z            |
//! | Aspen M-12    | Rigetti   | 40     | H, CNOT, Rx, Ry, Rz, CZ       |
//!
//! ## Example
//!
//! ```rust
//! use fusion_quantum_aws::*;
//!
//! let mut circuit = BraketCircuit::new(2);
//! circuit.add_instruction(Instruction::H { target: 0 });
//! circuit.add_instruction(Instruction::CNot { control: 0, target: 1 });
//! circuit.measure_all();
//!
//! let ir = circuit.to_braket_ir().unwrap();
//! assert!(ir.contains("\"type\": \"H\""));
//! ```

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, trace};

// ── Errors ────────────────────────────────────────────────────────────────

/// Errors specific to AWS Braket backend operations.
#[derive(Debug, thiserror::Error)]
pub enum BraketError {
    #[error("no device available matching constraints: {0}")]
    DeviceNotFound(String),

    #[error("circuit validation failed: {0}")]
    CircuitValidation(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("task {0} failed: {1}")]
    TaskFailed(String, String),

    #[error("task {0} was cancelled")]
    TaskCancelled(String),

    #[error("S3 configuration error: {0}")]
    S3Config(String),

    #[error("device {device} does not support gate '{gate}'")]
    UnsupportedGate { device: String, gate: String },

    #[error("qubit {0} out of range for device (max {1})")]
    QubitOutOfRange(usize, usize),

    #[error("connectivity violation: qubits {q0} and {q1} are not connected on {device}")]
    ConnectivityViolation {
        q0: usize,
        q1: usize,
        device: String,
    },
}

// ── Instructions ──────────────────────────────────────────────────────────

/// A single instruction in the Braket IR representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Instruction {
    /// Hadamard gate
    #[serde(rename = "H")]
    H { target: usize },

    /// CNOT (controlled-NOT) gate
    #[serde(rename = "CNot")]
    CNot { control: usize, target: usize },

    /// Rotation around X axis
    #[serde(rename = "Rx")]
    Rx { target: usize, angle: f64 },

    /// Rotation around Y axis
    #[serde(rename = "Ry")]
    Ry { target: usize, angle: f64 },

    /// Rotation around Z axis
    #[serde(rename = "Rz")]
    Rz { target: usize, angle: f64 },

    /// Pauli-X gate
    #[serde(rename = "X")]
    X { target: usize },

    /// Pauli-Y gate
    #[serde(rename = "Y")]
    Y { target: usize },

    /// Pauli-Z gate
    #[serde(rename = "Z")]
    Z { target: usize },

    /// SWAP gate
    #[serde(rename = "Swap")]
    Swap { qubit0: usize, qubit1: usize },

    /// Controlled-Z gate
    #[serde(rename = "CZ")]
    CZ { control: usize, target: usize },

    /// Toffoli (CCNOT) gate
    #[serde(rename = "CCNot")]
    CCNot {
        control0: usize,
        control1: usize,
        target: usize,
    },

    /// Measurement of a qubit into a classical register
    #[serde(rename = "Measure")]
    Measure {
        target: usize,
        classical_target: usize,
    },
}

impl Instruction {
    /// Returns the qubits this instruction operates on.
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            Instruction::H { target }
            | Instruction::Rx { target, .. }
            | Instruction::Ry { target, .. }
            | Instruction::Rz { target, .. }
            | Instruction::X { target }
            | Instruction::Y { target }
            | Instruction::Z { target } => vec![*target],

            Instruction::CNot { control, target }
            | Instruction::CZ { control, target } => vec![*control, *target],

            Instruction::Swap { qubit0, qubit1 } => vec![*qubit0, *qubit1],

            Instruction::CCNot {
                control0,
                control1,
                target,
            } => vec![*control0, *control1, *target],

            Instruction::Measure { target, .. } => vec![*target],
        }
    }

    /// Returns the gate name as a string.
    pub fn gate_name(&self) -> &str {
        match self {
            Instruction::H { .. } => "H",
            Instruction::CNot { .. } => "CNot",
            Instruction::Rx { .. } => "Rx",
            Instruction::Ry { .. } => "Ry",
            Instruction::Rz { .. } => "Rz",
            Instruction::X { .. } => "X",
            Instruction::Y { .. } => "Y",
            Instruction::Z { .. } => "Z",
            Instruction::Swap { .. } => "Swap",
            Instruction::CZ { .. } => "CZ",
            Instruction::CCNot { .. } => "CCNot",
            Instruction::Measure { .. } => "Measure",
        }
    }

    /// Returns true if this is a measurement instruction.
    pub fn is_measurement(&self) -> bool {
        matches!(self, Instruction::Measure { .. })
    }
}

// ── Braket Circuit ────────────────────────────────────────────────────────

/// A quantum circuit ready for submission to Amazon Braket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraketCircuit {
    /// Number of qubits in the circuit.
    pub num_qubits: usize,
    /// Ordered list of instructions.
    pub instructions: Vec<Instruction>,
    /// Number of classical bits for measurement results.
    pub num_classical_bits: usize,
}

impl BraketCircuit {
    /// Create a new empty circuit with the given number of qubits.
    pub fn new(num_qubits: usize) -> Self {
        debug!("Creating BraketCircuit with {} qubits", num_qubits);
        Self {
            num_qubits,
            instructions: Vec::new(),
            num_classical_bits: 0,
        }
    }

    /// Add a single instruction to the circuit.
    pub fn add_instruction(&mut self, instruction: Instruction) -> Result<(), BraketError> {
        // Validate qubit indices
        for &q in &instruction.qubits() {
            if q >= self.num_qubits {
                return Err(BraketError::QubitOutOfRange(q, self.num_qubits));
            }
        }
        self.instructions.push(instruction);
        Ok(())
    }

    /// Measure all qubits into classical registers starting from index 0.
    pub fn measure_all(&mut self) {
        self.num_classical_bits = self.num_qubits;
        for q in 0..self.num_qubits {
            self.instructions
                .push(Instruction::Measure { target: q, classical_target: q });
        }
    }

    /// Measure specific qubits into sequential classical registers.
    pub fn measure(&mut self, qubits: &[usize]) -> Result<(), BraketError> {
        for &q in qubits {
            if q >= self.num_qubits {
                return Err(BraketError::QubitOutOfRange(q, self.num_qubits));
            }
        }
        let start = self.num_classical_bits;
        for (i, &q) in qubits.iter().enumerate() {
            self.instructions.push(Instruction::Measure {
                target: q,
                classical_target: start + i,
            });
        }
        self.num_classical_bits += qubits.len();
        Ok(())
    }

    /// Get the circuit depth (longest path of sequential gates).
    pub fn depth(&self) -> usize {
        let mut qubit_layers = vec![0usize; self.num_qubits];

        for inst in &self.instructions {
            let qubits = inst.qubits();
            if qubits.is_empty() {
                continue;
            }
            let max_layer = qubits.iter().map(|&q| qubit_layers[q]).max().unwrap_or(0);
            let new_layer = max_layer + 1;
            for &q in &qubits {
                qubit_layers[q] = new_layer;
            }
        }

        qubit_layers.into_iter().max().unwrap_or(0)
    }

    /// Count the number of non-measurement gates.
    pub fn gate_count(&self) -> usize {
        self.instructions.iter().filter(|i| !i.is_measurement()).count()
    }

    /// Serialize the circuit to Amazon Braket IR (JSON format).
    pub fn to_braket_ir(&self) -> Result<String, BraketError> {
        let ir = BraketIr {
            braket_schema: "braket.ir.jaqcd".to_string(),
            instructions: &self.instructions,
            num_qubits: self.num_qubits,
            results: self.build_result_spec(),
        };
        Ok(serde_json::to_string_pretty(&ir)?)
    }

    fn build_result_spec(&self) -> Vec<BraketResultType> {
        if self.num_classical_bits == 0 {
            return vec![];
        }
        vec![BraketResultType {
            result_type: "DensityMatrix".to_string(),
            target: 0,
            state: "SV1".to_string(),
        }]
    }

    /// Validate circuit against a device's constraints.
    pub fn validate(&self, device: &Device) -> Result<(), BraketError> {
        if self.num_qubits > device.num_qubits() {
            return Err(BraketError::CircuitValidation(format!(
                "circuit uses {} qubits but device {} has {}",
                self.num_qubits,
                device.name(),
                device.num_qubits()
            )));
        }

        let native_gates = device.native_gates();
        for inst in &self.instructions {
            let name = inst.gate_name();
            if !native_gates.contains(&name) {
                return Err(BraketError::UnsupportedGate {
                    device: device.name().to_string(),
                    gate: name.to_string(),
                });
            }
        }

        // Check connectivity for 2+ qubit gates
        let connectivity = device.connectivity();
        for inst in &self.instructions {
            let qubits = inst.qubits();
            if qubits.len() == 2 {
                let (q0, q1) = (qubits[0], qubits[1]);
                if !connectivity.contains(&(q0, q1)) && !connectivity.contains(&(q1, q0)) {
                    return Err(BraketError::ConnectivityViolation {
                        q0,
                        q1,
                        device: device.name().to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct BraketIr<'a> {
    #[serde(rename = "braketSchemaHeader")]
    braket_schema: String,
    instructions: &'a [Instruction],
    #[serde(rename = "qubitCount")]
    num_qubits: usize,
    results: Vec<BraketResultType>,
}

#[derive(Serialize)]
struct BraketResultType {
    #[serde(rename = "resultType")]
    result_type: String,
    target: usize,
    state: String,
}

// ── Devices ───────────────────────────────────────────────────────────────

/// AWS Braket quantum computing device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Device {
    /// Amazon SV1 - 34-qubit state vector simulator
    SV1,
    /// IonQ Harmony - 11-qubit trapped-ion QPU
    IonQHarmony,
    /// Rigetti Aspen M-12 - 40-qubit superconducting QPU
    RigettiAspen,
}

/// Information about a Braket device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub arn: String,
    pub provider: String,
    pub num_qubits: usize,
    pub native_gates: Vec<String>,
    pub connectivity: Vec<(usize, usize)>,
    pub status: DeviceStatus,
}

/// Device operational status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Maintenance,
}

impl Device {
    /// Get the device name.
    pub fn name(&self) -> &str {
        match self {
            Device::SV1 => "SV1",
            Device::IonQHarmony => "IonQ Harmony",
            Device::RigettiAspen => "Aspen M-12",
        }
    }

    /// Get the Braket ARN for this device.
    pub fn arn(&self) -> &str {
        match self {
            Device::SV1 => "arn:aws:braket:::device/quantum-simulator/amazon/sv1",
            Device::IonQHarmony => "arn:aws:braket:::device/qpu/ionq/ionQ_Harmony",
            Device::RigettiAspen => "arn:aws:braket:::device/qpu/rigetti/Aspen-M-12",
        }
    }

    /// Get the number of qubits.
    pub fn num_qubits(&self) -> usize {
        match self {
            Device::SV1 => 34,
            Device::IonQHarmony => 11,
            Device::RigettiAspen => 40,
        }
    }

    /// Get the native gate set.
    pub fn native_gates(&self) -> &[&str] {
        match self {
            Device::SV1 => &["H", "CNot", "Rx", "Ry", "Rz", "X", "Y", "Z", "Swap", "Measure"],
            Device::IonQHarmony => &["Rx", "Ry", "Rz", "CNot", "X", "Z", "Measure"],
            Device::RigettiAspen => &["H", "CNot", "Rx", "Ry", "Rz", "CZ", "Measure"],
        }
    }

    /// Get the device connectivity (pairs of connected qubits).
    /// Returns a linear chain for simplicity; real devices have more complex topologies.
    pub fn connectivity(&self) -> Vec<(usize, usize)> {
        let n = self.num_qubits();
        (0..n - 1).map(|i| (i, i + 1)).collect()
    }

    /// Get full device info.
    pub fn info(&self) -> DeviceInfo {
        DeviceInfo {
            name: self.name().to_string(),
            arn: self.arn().to_string(),
            provider: match self {
                Device::SV1 => "Amazon".to_string(),
                Device::IonQHarmony => "IonQ".to_string(),
                Device::RigettiAspen => "Rigetti".to_string(),
            },
            num_qubits: self.num_qubits(),
            native_gates: self.native_gates().iter().map(|s| s.to_string()).collect(),
            connectivity: self.connectivity(),
            status: DeviceStatus::Online,
        }
    }

    /// List all available devices.
    pub fn all() -> Vec<Device> {
        vec![Device::SV1, Device::IonQHarmony, Device::RigettiAspen]
    }

    /// Select the best device for a given circuit based on constraints.
    /// Picks the smallest device that has enough qubits AND supports all gates.
    pub fn select_for_circuit(circuit: &BraketCircuit) -> Option<Device> {
        Self::all()
            .into_iter()
            .filter(|d| {
                d.num_qubits() >= circuit.num_qubits
                    && circuit
                        .instructions
                        .iter()
                        .all(|i| d.native_gates().contains(&i.gate_name()))
            })
            .min_by_key(|d| d.num_qubits())
    }
}

// ── Task Management ───────────────────────────────────────────────────────

/// Status of a Braket task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Created,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Created => write!(f, "CREATED"),
            TaskStatus::Queued => write!(f, "QUEUED"),
            TaskStatus::Running => write!(f, "RUNNING"),
            TaskStatus::Completed => write!(f, "COMPLETED"),
            TaskStatus::Failed => write!(f, "FAILED"),
            TaskStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

/// A tracked Braket task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraketTask {
    pub task_arn: String,
    pub status: TaskStatus,
    pub device: String,
    pub shots: u32,
    pub created_at: Option<String>,
    pub completed_at: Option<String>,
}

// ── Results ───────────────────────────────────────────────────────────────

/// Result from an AWS Braket task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraketBackendResult {
    pub measurement_counts: HashMap<String, u32>,
    pub task_arn: String,
    pub device_used: String,
    pub execution_time_ms: u64,
    pub meter_usage: f64,
}

impl BraketBackendResult {
    /// Get the most frequent outcome.
    pub fn most_frequent(&self) -> Option<(&str, u32)> {
        self.measurement_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(k, &v)| (k.as_str(), v))
    }

    /// Total number of measurement shots.
    pub fn total_shots(&self) -> u32 {
        self.measurement_counts.values().sum()
    }
}

// ── Configuration ─────────────────────────────────────────────────────────

/// Configuration for the AWS Braket backend.
#[derive(Debug, Clone)]
pub struct BraketConfig {
    /// AWS region (e.g., "us-east-1")
    pub region: String,
    /// S3 bucket for task results
    pub s3_bucket: String,
    /// S3 key prefix for task results
    pub s3_prefix: String,
    /// Path to AWS credentials file (optional; uses default credential chain if None)
    pub credentials_file: Option<String>,
    /// Default device to use
    pub default_device: Device,
    /// Default number of shots
    pub default_shots: u32,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for BraketConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            s3_bucket: "braket-fusion-results".to_string(),
            s3_prefix: "vortex/".to_string(),
            credentials_file: None,
            default_device: Device::SV1,
            default_shots: 1024,
            timeout: Duration::from_secs(300),
        }
    }
}

// ── Backend Trait ─────────────────────────────────────────────────────────

/// Unified quantum backend trait for all providers.
#[async_trait]
pub trait Backend {
    /// Backend-specific error type.
    type Error: fmt::Display + Send + Sync;

    /// Submit a circuit for execution.
    async fn submit_circuit(
        &self,
        circuit: &BraketCircuit,
        shots: u32,
    ) -> Result<String, Self::Error>;

    /// Get the status of a submitted task.
    async fn get_task_status(&self, task_arn: &str) -> Result<TaskStatus, Self::Error>;

    /// Retrieve results for a completed task.
    async fn get_task_results(&self, task_arn: &str) -> Result<BraketBackendResult, Self::Error>;

    /// Cancel a running task.
    async fn cancel_task(&self, task_arn: &str) -> Result<(), Self::Error>;

    /// Return the backend name.
    fn backend_name(&self) -> &str;
}

// ── AWS Braket Backend ────────────────────────────────────────────────────

/// AWS Braket backend implementation.
pub struct BraketBackend {
    config: BraketConfig,
    tasks: HashMap<String, BraketTask>,
}

impl BraketBackend {
    /// Create a new Braket backend with the given configuration.
    pub fn new(config: BraketConfig) -> Self {
        info!("Creating Braket backend for region {}", config.region);
        Self {
            config,
            tasks: HashMap::new(),
        }
    }

    /// Create a backend with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(BraketConfig::default())
    }

    /// Get the backend configuration.
    pub fn config(&self) -> &BraketConfig {
        &self.config
    }

    /// Build the S3 result path for a task.
    fn s3_result_path(&self, task_arn: &str) -> String {
        let short_id = task_arn.split('/').last().unwrap_or(task_arn);
        format!("s3://{}/{}/{}", self.config.s3_bucket, self.config.s3_prefix, short_id)
    }

    /// Build the Braket API endpoint URL for task submission.
    fn task_endpoint(&self) -> String {
        format!(
            "https://braket.{}.amazonaws.com/tasks",
            self.config.region
        )
    }

    /// Build the API endpoint for a specific task.
    fn task_detail_endpoint(&self, task_arn: &str) -> String {
        format!(
            "https://braket.{}.amazonaws.com{}",
            self.config.region,
            task_arn.replace("arn:aws:braket", "")
        )
    }

    /// Submit a circuit using a specific device.
    pub async fn submit_to_device(
        &self,
        circuit: &BraketCircuit,
        device: &Device,
        shots: u32,
    ) -> Result<String, BraketError> {
        // Validate circuit against the device
        circuit.validate(device)?;

        debug!(
            "Submitting circuit ({} gates, {} qubits) to {} with {} shots",
            circuit.gate_count(),
            circuit.num_qubits,
            device.name(),
            shots
        );

        // In a real implementation, this would make an HTTP POST to the Braket API.
        // For now, we generate a mock task ARN.
        let task_id = format!("task-{}", uuid_simple());
        let task_arn = format!(
            "arn:aws:braket:{}::task/{}",
            self.config.region, task_id
        );

        info!("Submitted task: {}", task_arn);
        Ok(task_arn)
    }

    /// List tasks tracked by this backend.
    pub fn list_tasks(&self) -> Vec<&BraketTask> {
        self.tasks.values().collect()
    }
}

impl Default for BraketBackend {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[async_trait]
impl Backend for BraketBackend {
    type Error = BraketError;

    async fn submit_circuit(
        &self,
        circuit: &BraketCircuit,
        shots: u32,
    ) -> Result<String, BraketError> {
        let device = Device::select_for_circuit(circuit)
            .ok_or_else(|| BraketError::DeviceNotFound("no suitable device".to_string()))?;

        self.submit_to_device(circuit, &device, shots).await
    }

    async fn get_task_status(&self, task_arn: &str) -> Result<TaskStatus, BraketError> {
        trace!("Polling status for task {}", task_arn);

        // In a real implementation, this would query the Braket API.
        if let Some(task) = self.tasks.get(task_arn) {
            return Ok(task.status.clone());
        }

        // For tasks not yet tracked, return Queued as default
        Ok(TaskStatus::Queued)
    }

    async fn get_task_results(&self, task_arn: &str) -> Result<BraketBackendResult, BraketError> {
        debug!("Retrieving results for task {}", task_arn);

        let task = self.tasks.get(task_arn).ok_or_else(|| {
            BraketError::TaskFailed(
                task_arn.to_string(),
                "task not found".to_string(),
            )
        })?;

        if task.status != TaskStatus::Completed {
            return Err(BraketError::TaskFailed(
                task_arn.to_string(),
                format!("task is in status {}", task.status),
            ));
        }

        // In a real implementation, this would fetch results from S3.
        Ok(BraketBackendResult {
            measurement_counts: HashMap::new(),
            task_arn: task_arn.to_string(),
            device_used: task.device.clone(),
            execution_time_ms: 0,
            meter_usage: 0.0,
        })
    }

    async fn cancel_task(&self, task_arn: &str) -> Result<(), BraketError> {
        info!("Cancelling task {}", task_arn);

        if let Some(task) = self.tasks.get(task_arn) {
            if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
                return Err(BraketError::TaskFailed(
                    task_arn.to_string(),
                    format!("cannot cancel task in status {}", task.status),
                ));
            }
        }

        Ok(())
    }

    fn backend_name(&self) -> &str {
        "aws-braket"
    }
}

// ── Device Filtering ──────────────────────────────────────────────────────

/// Filter devices by circuit constraints.
pub fn filter_devices<'a>(
    devices: &'a [Device],
    min_qubits: usize,
    required_gates: &[&str],
) -> Vec<&'a Device> {
    devices
        .iter()
        .filter(|d| {
            d.num_qubits() >= min_qubits
                && required_gates
                    .iter()
                    .all(|g| d.native_gates().contains(g))
        })
        .collect()
}

// ── Utility ───────────────────────────────────────────────────────────────

/// Simple deterministic ID generator for testing (no uuid dependency).
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:016x}", t)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Instruction Tests ─────────────────────────────────────────

    #[test]
    fn test_instruction_qubits() {
        let h = Instruction::H { target: 0 };
        assert_eq!(h.qubits(), vec![0]);
        assert_eq!(h.gate_name(), "H");
        assert!(!h.is_measurement());

        let cnot = Instruction::CNot {
            control: 0,
            target: 1,
        };
        assert_eq!(cnot.qubits(), vec![0, 1]);
        assert_eq!(cnot.gate_name(), "CNot");

        let ccnot = Instruction::CCNot {
            control0: 0,
            control1: 1,
            target: 2,
        };
        assert_eq!(ccnot.qubits(), vec![0, 1, 2]);

        let meas = Instruction::Measure {
            target: 0,
            classical_target: 0,
        };
        assert!(meas.is_measurement());
    }

    #[test]
    fn test_instruction_serialization() {
        let h = Instruction::H { target: 0 };
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains("\"type\":\"H\""));

        let cnot = Instruction::CNot {
            control: 0,
            target: 1,
        };
        let json = serde_json::to_string(&cnot).unwrap();
        assert!(json.contains("\"type\":\"CNot\""));

        let rx = Instruction::Rx {
            target: 2,
            angle: std::f64::consts::PI,
        };
        let json = serde_json::to_string(&rx).unwrap();
        assert!(json.contains("\"type\":\"Rx\""));
        assert!(json.contains("3.141592653589793"));
    }

    #[test]
    fn test_instruction_deserialization() {
        let json = r#"{"type":"H","target":0}"#;
        let inst: Instruction = serde_json::from_str(json).unwrap();
        assert_eq!(inst, Instruction::H { target: 0 });

        let json = r#"{"type":"CNot","control":0,"target":1}"#;
        let inst: Instruction = serde_json::from_str(json).unwrap();
        assert_eq!(
            inst,
            Instruction::CNot {
                control: 0,
                target: 1
            }
        );
    }

    // ── Circuit Tests ─────────────────────────────────────────────

    #[test]
    fn test_circuit_creation() {
        let circuit = BraketCircuit::new(3);
        assert_eq!(circuit.num_qubits, 3);
        assert_eq!(circuit.gate_count(), 0);
        assert_eq!(circuit.depth(), 0);
    }

    #[test]
    fn test_circuit_add_instruction() {
        let mut circuit = BraketCircuit::new(2);
        circuit
            .add_instruction(Instruction::H { target: 0 })
            .unwrap();
        circuit
            .add_instruction(Instruction::CNot {
                control: 0,
                target: 1,
            })
            .unwrap();
        assert_eq!(circuit.gate_count(), 2);
    }

    #[test]
    fn test_circuit_qubit_out_of_range() {
        let mut circuit = BraketCircuit::new(2);
        let result = circuit.add_instruction(Instruction::H { target: 5 });
        assert!(result.is_err());
        match result.unwrap_err() {
            BraketError::QubitOutOfRange(q, max) => {
                assert_eq!(q, 5);
                assert_eq!(max, 2);
            }
            _ => panic!("expected QubitOutOfRange"),
        }
    }

    #[test]
    fn test_circuit_measure_all() {
        let mut circuit = BraketCircuit::new(3);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit.measure_all();
        assert_eq!(circuit.num_classical_bits, 3);
        // 1 gate + 3 measurements = 4 instructions
        assert_eq!(circuit.instructions.len(), 4);
    }

    #[test]
    fn test_circuit_measure_specific() {
        let mut circuit = BraketCircuit::new(3);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit.measure(&[0, 2]).unwrap();
        assert_eq!(circuit.num_classical_bits, 2);
    }

    #[test]
    fn test_circuit_depth() {
        let mut circuit = BraketCircuit::new(2);
        // Two parallel single-qubit gates
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit.add_instruction(Instruction::H { target: 1 }).unwrap();
        assert_eq!(circuit.depth(), 1);

        // CNOT depends on both qubits
        circuit
            .add_instruction(Instruction::CNot {
                control: 0,
                target: 1,
            })
            .unwrap();
        assert_eq!(circuit.depth(), 2);
    }

    #[test]
    fn test_circuit_depth_sequential() {
        let mut circuit = BraketCircuit::new(1);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit.add_instruction(Instruction::X { target: 0 }).unwrap();
        circuit.add_instruction(Instruction::Z { target: 0 }).unwrap();
        assert_eq!(circuit.depth(), 3);
    }

    #[test]
    fn test_bell_state_circuit() {
        let mut circuit = BraketCircuit::new(2);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit
            .add_instruction(Instruction::CNot {
                control: 0,
                target: 1,
            })
            .unwrap();
        circuit.measure_all();

        assert_eq!(circuit.num_qubits, 2);
        assert_eq!(circuit.gate_count(), 2);
        assert_eq!(circuit.depth(), 3); // H, CNOT, then 2 measurements
        assert_eq!(circuit.num_classical_bits, 2);
    }

    #[test]
    fn test_to_braket_ir() {
        let mut circuit = BraketCircuit::new(2);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit
            .add_instruction(Instruction::CNot {
                control: 0,
                target: 1,
            })
            .unwrap();

        let ir = circuit.to_braket_ir().unwrap();
        assert!(ir.contains("\"type\": \"H\""));
        assert!(ir.contains("\"type\": \"CNot\""));
        assert!(ir.contains("\"qubitCount\": 2"));
    }

    // ── Device Tests ──────────────────────────────────────────────

    #[test]
    fn test_device_properties() {
        assert_eq!(Device::SV1.name(), "SV1");
        assert_eq!(Device::SV1.num_qubits(), 34);
        assert_eq!(Device::IonQHarmony.num_qubits(), 11);
        assert_eq!(Device::RigettiAspen.num_qubits(), 40);
    }

    #[test]
    fn test_device_native_gates() {
        let sv1_gates = Device::SV1.native_gates();
        assert!(sv1_gates.contains(&"H"));
        assert!(sv1_gates.contains(&"CNot"));
        assert!(sv1_gates.contains(&"Measure"));

        let ionq_gates = Device::IonQHarmony.native_gates();
        assert!(ionq_gates.contains(&"Rx"));
        assert!(!ionq_gates.contains(&"H")); // IonQ doesn't natively have H
    }

    #[test]
    fn test_device_connectivity() {
        let sv1_conn = Device::SV1.connectivity();
        assert_eq!(sv1_conn.len(), 33); // 34 qubits => 33 connections
        assert!(sv1_conn.contains(&(0, 1)));
        assert!(sv1_conn.contains(&(32, 33)));
    }

    #[test]
    fn test_device_select_for_circuit() {
        let small = BraketCircuit::new(5);
        let device = Device::select_for_circuit(&small).unwrap();
        assert_eq!(device.num_qubits(), 11); // IonQ is smallest that fits

        let large = BraketCircuit::new(35);
        let device = Device::select_for_circuit(&large).unwrap();
        assert_eq!(device.num_qubits(), 40); // Only Aspen fits
    }

    #[test]
    fn test_device_info() {
        let info = Device::SV1.info();
        assert_eq!(info.name, "SV1");
        assert_eq!(info.provider, "Amazon");
        assert_eq!(info.status, DeviceStatus::Online);
    }

    #[test]
    fn test_filter_devices() {
        let all = Device::all();
        let h_cnot_devices = filter_devices(&all, 0, &["H", "CNot"]);
        assert!(h_cnot_devices.contains(&&Device::SV1));
        assert!(h_cnot_devices.contains(&&Device::RigettiAspen));
        assert!(!h_cnot_devices.contains(&&Device::IonQHarmony)); // no H

        let large_enough = filter_devices(&all, 30, &[]);
        assert_eq!(large_enough.len(), 2); // SV1(34) and Aspen(40)
    }

    // ── Circuit Validation Tests ──────────────────────────────────

    #[test]
    fn test_validate_circuit_too_many_qubits() {
        let circuit = BraketCircuit::new(50);
        let result = circuit.validate(&Device::SV1);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_circuit_unsupported_gate() {
        let mut circuit = BraketCircuit::new(2);
        circuit
            .add_instruction(Instruction::CZ {
                control: 0,
                target: 1,
            })
            .unwrap();
        let result = circuit.validate(&Device::SV1);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_circuit_valid() {
        let mut circuit = BraketCircuit::new(2);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit
            .add_instruction(Instruction::CNot {
                control: 0,
                target: 1,
            })
            .unwrap();
        assert!(circuit.validate(&Device::SV1).is_ok());
    }

    // ── Task Tests ────────────────────────────────────────────────

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Completed.to_string(), "COMPLETED");
        assert_eq!(TaskStatus::Running.to_string(), "RUNNING");
        assert_eq!(TaskStatus::Failed.to_string(), "FAILED");
    }

    // ── Result Tests ──────────────────────────────────────────────

    #[test]
    fn test_result_most_frequent() {
        let mut counts = HashMap::new();
        counts.insert("00".to_string(), 512);
        counts.insert("01".to_string(), 128);
        counts.insert("10".to_string(), 128);
        counts.insert("11".to_string(), 256);

        let result = BraketBackendResult {
            measurement_counts: counts,
            task_arn: "test-arn".to_string(),
            device_used: "SV1".to_string(),
            execution_time_ms: 150,
            meter_usage: 1.0,
        };

        let (best, count) = result.most_frequent().unwrap();
        assert_eq!(best, "00");
        assert_eq!(count, 512);
        assert_eq!(result.total_shots(), 1024);
    }

    #[test]
    fn test_result_empty() {
        let result = BraketBackendResult {
            measurement_counts: HashMap::new(),
            task_arn: "test-arn".to_string(),
            device_used: "SV1".to_string(),
            execution_time_ms: 0,
            meter_usage: 0.0,
        };
        assert!(result.most_frequent().is_none());
        assert_eq!(result.total_shots(), 0);
    }

    // ── Backend Tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_backend_submit_circuit() {
        let backend = BraketBackend::with_defaults();
        let mut circuit = BraketCircuit::new(2);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit
            .add_instruction(Instruction::CNot {
                control: 0,
                target: 1,
            })
            .unwrap();

        let task_arn = backend.submit_circuit(&circuit, 1024).await.unwrap();
        assert!(task_arn.contains("arn:aws:braket"));
        assert_eq!(backend.backend_name(), "aws-braket");
    }

    #[tokio::test]
    async fn test_backend_submit_too_many_qubits() {
        let backend = BraketBackend::with_defaults();
        let circuit = BraketCircuit::new(50);
        let result = backend.submit_circuit(&circuit, 1024).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backend_task_lifecycle() {
        let backend = BraketBackend::with_defaults();
        let mut circuit = BraketCircuit::new(1);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();

        let task_arn = backend.submit_circuit(&circuit, 100).await.unwrap();
        let status = backend.get_task_status(&task_arn).await.unwrap();
        assert_eq!(status, TaskStatus::Queued);
    }

    #[tokio::test]
    async fn test_backend_cancel() {
        let backend = BraketBackend::with_defaults();
        let result = backend.cancel_task("arn:nonexistent").await;
        assert!(result.is_ok());
    }

    // ── Config Tests ──────────────────────────────────────────────

    #[test]
    fn test_default_config() {
        let config = BraketConfig::default();
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.s3_bucket, "braket-fusion-results");
        assert_eq!(config.default_shots, 1024);
        assert!(config.credentials_file.is_none());
    }

    #[test]
    fn test_s3_result_path() {
        let backend = BraketBackend::with_defaults();
        let path = backend.s3_result_path("arn:aws:braket:us-east-1::task/abc123");
        assert!(path.starts_with("s3://braket-fusion-results/vortex/"));
        assert!(path.contains("abc123"));
    }

    // ── Roundtrip Test ────────────────────────────────────────────

    #[test]
    fn test_circuit_ir_roundtrip() {
        let mut circuit = BraketCircuit::new(3);
        circuit.add_instruction(Instruction::H { target: 0 }).unwrap();
        circuit.add_instruction(Instruction::Ry {
            target: 1,
            angle: std::f64::consts::FRAC_PI_4,
        });
        circuit
            .add_instruction(Instruction::CNot {
                control: 0,
                target: 2,
            })
            .unwrap();

        let ir = circuit.to_braket_ir().unwrap();
        assert!(ir.contains("\"qubitCount\": 3"));
        assert!(ir.contains("braketSchemaHeader"));
    }
}
