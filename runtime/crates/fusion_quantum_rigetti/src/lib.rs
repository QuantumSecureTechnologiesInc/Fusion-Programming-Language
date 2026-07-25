//! # Fusion Quantum Rigetti
//!
//! Rigetti QCS backend integration with QAM (Quantum Abstract Machine) interface
//! for Fusion v2.0 Vortex runtime.
//!
//! This crate provides:
//! - Quantum Abstract Machine (QAM) trait for abstracting quantum execution
//! - Rigetti QCS backend implementation
//! - Quil instruction generation and program representation
//! - QPU reservation management
//! - Native Quil and OpenQASM program support

use async_trait::async_trait;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

// ============================================================================
// Errors
// ============================================================================

/// Errors specific to Rigetti QCS operations
#[derive(Error, Debug)]
pub enum RigettiError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("QPU unavailable: {0}")]
    QPUUnavailable(String),

    #[error("Reservation failed: {0}")]
    ReservationFailed(String),

    #[error("Program compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid program: {0}")]
    InvalidProgram(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

// ============================================================================
// Quantum State
// ============================================================================

/// Represents a quantum state with complex amplitudes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumState {
    /// Number of qubits in the state
    pub num_qubits: usize,
    /// Complex amplitudes for each basis state (2^n for n qubits)
    pub amplitudes: Vec<Complex64>,
}

impl QuantumState {
    /// Create a new quantum state with all qubits in |0⟩
    pub fn new(num_qubits: usize) -> Self {
        let num_states = 1 << num_qubits;
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); num_states];
        amplitudes[0] = Complex64::new(1.0, 0.0); // |00...0⟩ state

        Self {
            num_qubits,
            amplitudes,
        }
    }

    /// Get the probability of measuring a specific basis state
    pub fn probability(&self, basis_state: usize) -> f64 {
        if basis_state < self.amplitudes.len() {
            self.amplitudes[basis_state].norm_sqr()
        } else {
            0.0
        }
    }

    /// Normalize the state vector
    pub fn normalize(&mut self) {
        let norm: f64 = self.amplitudes.iter().map(|a| a.norm_sqr()).sum();
        let norm_sqrt = norm.sqrt();

        if norm_sqrt > 0.0 {
            for amp in &mut self.amplitudes {
                *amp /= norm_sqrt;
            }
        }
    }
}

impl fmt::Display for QuantumState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QuantumState({} qubits, {} amplitudes)",
            self.num_qubits,
            self.amplitudes.len()
        )
    }
}

// ============================================================================
// Program Types and Instructions
// ============================================================================

/// Type of quantum program
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgramType {
    /// Native Quil format (Rigetti's intermediate representation)
    NativeQuil,
    /// OpenQASM format
    OpenQASM,
}

impl fmt::Display for ProgramType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramType::NativeQuil => write!(f, "NativeQuil"),
            ProgramType::OpenQASM => write!(f, "OpenQASM"),
        }
    }
}

/// Quantum instruction with parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    /// Hadamard gate
    H(usize),
    /// Controlled-NOT gate (control, target)
    CNOT(usize, usize),
    /// Rotation around X axis: Rx(theta)(qubit)
    Rx(f64, usize),
    /// Rotation around Y axis: Ry(theta)(qubit)
    Ry(f64, usize),
    /// Rotation around Z axis: Rz(theta)(qubit)
    Rz(f64, usize),
    /// Pauli X gate
    X(usize),
    /// Pauli Y gate
    Y(usize),
    /// Pauli Z gate
    Z(usize),
    /// Measurement into classical register
    MEASURE(usize, usize),
    /// Wait for QPU
    WAIT,
    /// Halt program execution
    HALT,
    /// Fence for memory ordering
    FENCE,
}

impl Instruction {
    /// Convert instruction to Quil string representation
    pub fn to_quil_string(&self) -> String {
        match self {
            Instruction::H(q) => format!("H {}", q),
            Instruction::CNOT(c, t) => format!("CNOT {} {}", c, t),
            Instruction::Rx(theta, q) => format!("Rx({}) {}", theta, q),
            Instruction::Ry(theta, q) => format!("Ry({}) {}", theta, q),
            Instruction::Rz(theta, q) => format!("Rz({}) {}", theta, q),
            Instruction::X(q) => format!("X {}", q),
            Instruction::Y(q) => format!("Y {}", q),
            Instruction::Z(q) => format!("Z {}", q),
            Instruction::MEASURE(q, c) => format!("MEASURE {} [{}]", q, c),
            Instruction::WAIT => "WAIT".to_string(),
            Instruction::HALT => "HALT".to_string(),
            Instruction::FENCE => "FENCE".to_string(),
        }
    }
}

// ============================================================================
// Quantum Program
// ============================================================================

/// A quantum program with instructions and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QProgram {
    /// Type of program (NativeQuil or OpenQASM)
    pub program_type: ProgramType,
    /// Sequence of quantum instructions
    pub instructions: Vec<Instruction>,
    /// Number of qubits used in the program
    pub num_qubits: usize,
    /// Number of classical registers
    pub num_classical_registers: usize,
    /// Named memory regions (e.g., "ro" for readout)
    pub memory_regions: HashMap<String, usize>,
}

impl QProgram {
    /// Create a new empty program
    pub fn new(program_type: ProgramType, num_qubits: usize) -> Self {
        Self {
            program_type,
            instructions: Vec::new(),
            num_qubits,
            num_classical_registers: 0,
            memory_regions: HashMap::new(),
        }
    }

    /// Add an instruction to the program
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    /// Get the number of instructions
    pub fn num_instructions(&self) -> usize {
        self.instructions.len()
    }

    /// Convert the program to Quil string format
    pub fn to_quil_string(&self) -> String {
        let mut quil = String::new();

        // Add memory declarations
        if self.num_classical_registers > 0 {
            quil.push_str(&format!("DECLARE ro BIT[{}]\n", self.num_classical_registers));
        }

        for (name, size) in &self.memory_regions {
            if name != "ro" {
                quil.push_str(&format!("DECLARE {} BIT[{}]\n", name, size));
            }
        }

        quil.push('\n');

        // Add instructions
        for instruction in &self.instructions {
            quil.push_str(&instruction.to_quil_string());
            quil.push('\n');
        }

        quil
    }
}

// ============================================================================
// QPU Definitions
// ============================================================================

/// Available Rigetti QPU processors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QPU {
    /// Aspen-M-12 processor (12 qubits)
    AspenM12,
    /// Ankaa-2 processor (30 qubits)
    Ankaa2,
    /// Ankaa-3 processor (40 qubits)
    Ankaa3,
}

impl QPU {
    /// Get the number of qubits for this QPU
    pub fn num_qubits(&self) -> usize {
        match self {
            QPU::AspenM12 => 12,
            QPU::Ankaa2 => 30,
            QPU::Ankaa3 => 40,
        }
    }

    /// Get the connectivity topology as adjacency list
    /// Returns pairs of qubit indices that are connected
    pub fn connectivity(&self) -> Vec<(usize, usize)> {
        match self {
            QPU::AspenM12 => {
                // Linear chain connectivity for Aspen-M-12
                vec![
                    (0, 1),
                    (1, 2),
                    (2, 3),
                    (3, 4),
                    (4, 5),
                    (5, 6),
                    (6, 7),
                    (7, 8),
                    (8, 9),
                    (9, 10),
                    (10, 11),
                ]
            }
            QPU::Ankaa2 => {
                // Grid connectivity for Ankaa-2 (6x5)
                let mut connections = Vec::new();
                for row in 0..5 {
                    for col in 0..6 {
                        let q = row * 6 + col;
                        if col < 5 {
                            connections.push((q, q + 1));
                        }
                        if row < 4 {
                            connections.push((q, q + 6));
                        }
                    }
                }
                connections
            }
            QPU::Ankaa3 => {
                // Grid connectivity for Ankaa-3 (8x5)
                let mut connections = Vec::new();
                for row in 0..5 {
                    for col in 0..8 {
                        let q = row * 8 + col;
                        if col < 7 {
                            connections.push((q, q + 1));
                        }
                        if row < 4 {
                            connections.push((q, q + 8));
                        }
                    }
                }
                connections
            }
        }
    }

    /// Get the QPU ID string for API calls
    pub fn id(&self) -> &str {
        match self {
            QPU::AspenM12 => "Aspen-M-12",
            QPU::Ankaa2 => "Ankaa-2",
            QPU::Ankaa3 => "Ankaa-3",
        }
    }

    /// Check if two qubits are connected
    pub fn are_connected(&self, q1: usize, q2: usize) -> bool {
        self.connectivity()
            .iter()
            .any(|&(a, b)| (a == q1 && b == q2) || (a == q2 && b == q1))
    }
}

impl fmt::Display for QPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} qubits)", self.id(), self.num_qubits())
    }
}

// ============================================================================
// QCS Configuration
// ============================================================================

/// Configuration for connecting to Rigetti QCS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QCSConfig {
    /// API token for authentication
    pub api_token: String,
    /// QCS API endpoint URL
    pub endpoint_url: String,
    /// Optional reservation ID for QPU access
    pub reservation_id: Option<String>,
    /// Timeout for API calls in seconds
    pub timeout_secs: u64,
}

impl QCSConfig {
    /// Create a new QCS configuration
    pub fn new(api_token: String, endpoint_url: String) -> Self {
        Self {
            api_token,
            endpoint_url,
            reservation_id: None,
            timeout_secs: 30,
        }
    }

    /// Set reservation ID
    pub fn with_reservation(mut self, reservation_id: String) -> Self {
        self.reservation_id = Some(reservation_id);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

// ============================================================================
// QCS Job
// ============================================================================

/// Status of a QCS job
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is queued
    Pending,
    /// Job is executing on QPU
    Executing,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed(String),
    /// Job was cancelled
    Cancelled,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Executing => write!(f, "Executing"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Failed(msg) => write!(f, "Failed: {}", msg),
            JobStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// A QCS job representing a submitted quantum program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QCSJob {
    /// Unique job identifier
    pub job_id: String,
    /// Current status of the job
    pub status: JobStatus,
    /// Reservation ID used for this job
    pub reservation_id: Option<String>,
    /// The program that was submitted
    pub program: Option<QProgram>,
    /// Number of shots requested
    pub shots: u32,
    /// QPU targeted
    pub target_qpu: QPU,
}

impl QCSJob {
    /// Create a new QCS job
    pub fn new(job_id: String, program: QProgram, shots: u32, target_qpu: QPU) -> Self {
        Self {
            job_id,
            status: JobStatus::Pending,
            reservation_id: None,
            program: Some(program),
            shots,
            target_qpu,
        }
    }
}

// ============================================================================
// QAM Result
// ============================================================================

/// Result from a quantum abstract machine execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAMResult {
    /// Readout values: register_name -> vector of bit values
    pub readout_values: HashMap<String, Vec<u8>>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// QPU used for execution
    pub qpu_used: String,
}

impl QAMResult {
    /// Create a new QAM result
    pub fn new(
        readout_values: HashMap<String, Vec<u8>>,
        execution_time_ms: u64,
        qpu_used: String,
    ) -> Self {
        Self {
            readout_values,
            execution_time_ms,
            qpu_used,
        }
    }

    /// Get readout values for a specific register
    pub fn get_register(&self, name: &str) -> Option<&Vec<u8>> {
        self.readout_values.get(name)
    }

    /// Get the total number of measurement outcomes
    pub fn num_outcomes(&self) -> usize {
        self.readout_values.values().map(|v| v.len()).sum()
    }
}

// ============================================================================
// QAM Trait
// ============================================================================

/// Quantum Abstract Machine - core trait for quantum execution
///
/// This trait provides a unified interface for executing quantum programs
/// on different backends (simulators, real QPUs, etc.)
#[async_trait]
pub trait QAM {
    /// Error type for this QAM implementation
    type Error: std::fmt::Display + Send + Sync;

    /// Run a quantum program with the specified number of shots
    async fn run(&self, program: &QProgram, shots: u32) -> Result<QAMResult, Self::Error>;

    /// Measure all specified qubits and return results
    async fn measure_all(&self, qubits: &[usize]) -> Result<Vec<u8>, Self::Error>;

    /// Reset the quantum state to |0...0⟩
    async fn reset(&self) -> Result<(), Self::Error>;

    /// Get the current quantum state (if available)
    fn quantum_state(&self) -> Option<&QuantumState>;

    /// Get the QPU identifier
    fn qpu_id(&self) -> &str;
}

// ============================================================================
// Backend Trait (Fusion v2.0 common interface)
// ============================================================================

/// Backend trait for Fusion v2.0 Vortex runtime
///
/// This is the common interface that all quantum backends must implement
#[async_trait]
pub trait Backend: Send + Sync {
    /// Error type for this backend
    type Error: std::fmt::Display + Send + Sync;

    /// Get the backend name
    fn name(&self) -> &str;

    /// Get supported program types
    fn supported_program_types(&self) -> Vec<ProgramType>;

    /// Submit a program for execution
    async fn submit(&self, program: &QProgram, shots: u32) -> Result<QCSJob, Self::Error>;

    /// Check if the backend is available
    async fn is_available(&self) -> bool;

    /// Get estimated execution time
    fn estimated_execution_time(&self, program: &QProgram, shots: u32) -> Option<u64>;
}

// ============================================================================
// Reservation System
// ============================================================================

/// Reservation for QPU access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reservation {
    /// Unique reservation identifier
    pub reservation_id: String,
    /// QPU reserved
    pub qpu: QPU,
    /// Start time (Unix timestamp)
    pub start_time: u64,
    /// Duration in seconds
    pub duration_secs: u64,
    /// Reservation status
    pub active: bool,
}

impl Reservation {
    /// Check if reservation is currently valid
    pub fn is_valid(&self, current_time: u64) -> bool {
        self.active && current_time >= self.start_time
            && current_time < self.start_time + self.duration_secs
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn uuid_simple() -> String {
    // Simple UUID-like identifier for demonstration
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        current_timestamp(),
        0x1234u32,
        0x5678u32,
        0x9abcu32,
        0xdef012345678u64
    )
}

// ============================================================================
// Rigetti QCS Backend
// ============================================================================

/// Rigetti QCS backend implementing both QAM and Backend traits
pub struct RigettiQCSBackend {
    /// Configuration for QCS connection
    config: QCSConfig,
    /// Target QPU
    qpu: QPU,
    /// Current quantum state (for simulation)
    state: Option<QuantumState>,
    /// Active reservation
    reservation: Option<Reservation>,
}

impl RigettiQCSBackend {
    /// Create a new Rigetti QCS backend
    pub fn new(config: QCSConfig, qpu: QPU) -> Self {
        Self {
            config,
            qpu,
            state: None,
            reservation: None,
        }
    }

    /// Reserve a QPU for execution
    pub async fn reserve_qpu(&mut self, duration_secs: u64) -> Result<Reservation, RigettiError> {
        tracing::info!("Reserving {} for {} seconds", self.qpu, duration_secs);

        // In a real implementation, this would call the QCS API
        // For now, we simulate the reservation
        let reservation = Reservation {
            reservation_id: format!("res_{}", uuid_simple()),
            qpu: self.qpu,
            start_time: current_timestamp(),
            duration_secs,
            active: true,
        };

        self.reservation = Some(reservation.clone());
        self.config.reservation_id = Some(reservation.reservation_id.clone());

        tracing::info!("Reservation created: {}", reservation.reservation_id);
        Ok(reservation)
    }

    /// Release the current reservation
    pub async fn release_reservation(&mut self) -> Result<(), RigettiError> {
        if let Some(mut reservation) = self.reservation.take() {
            tracing::info!("Releasing reservation: {}", reservation.reservation_id);
            reservation.active = false;

            // In a real implementation, this would call the QCS API
            Ok(())
        } else {
            Err(RigettiError::ReservationFailed(
                "No active reservation".to_string(),
            ))
        }
    }

    /// Check if there's an active valid reservation
    pub fn has_valid_reservation(&self) -> bool {
        self.reservation.as_ref().map_or(false, |r| {
            let now = current_timestamp();
            r.active && now >= r.start_time && now < r.start_time + r.duration_secs
        })
    }

    /// Validate that a program is compatible with the QPU
    pub fn validate_program(&self, program: &QProgram) -> Result<(), RigettiError> {
        if program.num_qubits > self.qpu.num_qubits() {
            return Err(RigettiError::InvalidProgram(format!(
                "Program requires {} qubits but QPU {} has only {}",
                program.num_qubits,
                self.qpu.id(),
                self.qpu.num_qubits()
            )));
        }

        // Validate two-qubit gate connectivity
        for instruction in &program.instructions {
            if let Instruction::CNOT(control, target) = instruction {
                if !self.qpu.are_connected(*control, *target) {
                    return Err(RigettiError::InvalidProgram(format!(
                        "Qubits {} and {} are not connected on {}",
                        control,
                        target,
                        self.qpu.id()
                    )));
                }
            }
        }

        Ok(())
    }

    /// Compile a program for the target QPU
    pub fn compile_program(&self, program: &QProgram) -> Result<QProgram, RigettiError> {
        self.validate_program(program)?;

        // In a real implementation, this would perform:
        // - Gate decomposition for native gateset
        // - Qubit routing
        // - Optimization passes

        tracing::info!(
            "Compiled program with {} instructions for {}",
            program.num_instructions(),
            self.qpu
        );
        Ok(program.clone())
    }

    /// Get the QPU being used
    pub fn qpu(&self) -> QPU {
        self.qpu
    }

    /// Get the current configuration
    pub fn config(&self) -> &QCSConfig {
        &self.config
    }

    /// Simulate a simple quantum execution (for testing)
    fn simulate_execution(&self, program: &QProgram, shots: u32) -> QAMResult {
        let mut state = QuantumState::new(program.num_qubits);

        // Apply instructions to simulate
        for instruction in &program.instructions {
            match instruction {
                Instruction::H(q) => {
                    // Apply Hadamard to qubit q
                    let idx = 1 << q;
                    let sqrt2_inv = 1.0 / 2.0_f64.sqrt();
                    for i in 0..state.amplitudes.len() {
                        if i & idx == 0 {
                            let i1 = i | idx;
                            let a0 = state.amplitudes[i];
                            let a1 = state.amplitudes[i1];
                            state.amplitudes[i] =
                                Complex64::new(sqrt2_inv, 0.0) * (a0 + a1);
                            state.amplitudes[i1] =
                                Complex64::new(sqrt2_inv, 0.0) * (a0 - a1);
                        }
                    }
                }
                Instruction::X(q) => {
                    // Apply Pauli-X (NOT) gate
                    let idx = 1 << q;
                    for i in 0..state.amplitudes.len() {
                        if i & idx == 0 {
                            let i1 = i | idx;
                            state.amplitudes.swap(i, i1);
                        }
                    }
                }
                _ => {
                    // Other gates not fully simulated here
                }
            }
        }

        // Generate random readout based on final state
        let mut readout = HashMap::new();
        let mut ro_values = Vec::new();

        for _ in 0..shots {
            // Simple random sampling based on probabilities
            let mut cumulative = 0.0;
            let rand_val = (current_timestamp() as f64 % 1000.0) / 1000.0;

            for (idx, amp) in state.amplitudes.iter().enumerate() {
                cumulative += amp.norm_sqr();
                if cumulative >= rand_val {
                    // Extract measurement bits
                    for q in 0..program.num_qubits {
                        ro_values.push(((idx >> q) & 1) as u8);
                    }
                    break;
                }
            }
        }

        readout.insert("ro".to_string(), ro_values);

        QAMResult {
            readout_values: readout,
            execution_time_ms: 100, // Simulated
            qpu_used: self.qpu.id().to_string(),
        }
    }
}

// ============================================================================
// QAM Implementation for RigettiQCSBackend
// ============================================================================

#[async_trait]
impl QAM for RigettiQCSBackend {
    type Error = RigettiError;

    async fn run(&self, program: &QProgram, shots: u32) -> Result<QAMResult, Self::Error> {
        tracing::info!(
            "Running program with {} instructions, {} shots on {}",
            program.num_instructions(),
            shots,
            self.qpu
        );

        // Validate program
        self.validate_program(program)?;

        // Check reservation if required
        if self.config.reservation_id.is_some() && !self.has_valid_reservation() {
            return Err(RigettiError::ReservationFailed(
                "No valid reservation for QPU access".to_string(),
            ));
        }

        // In a real implementation, this would:
        // 1. Compile the program
        // 2. Submit to QCS API
        // 3. Poll for completion
        // 4. Return results

        // For now, simulate execution
        let result = self.simulate_execution(program, shots);

        tracing::info!("Execution completed in {}ms", result.execution_time_ms);
        Ok(result)
    }

    async fn measure_all(&self, qubits: &[usize]) -> Result<Vec<u8>, Self::Error> {
        tracing::info!("Measuring qubits: {:?}", qubits);

        // Validate qubits
        for &q in qubits {
            if q >= self.qpu.num_qubits() {
                return Err(RigettiError::InvalidProgram(format!(
                    "Qubit index {} out of range for {}",
                    q,
                    self.qpu
                )));
            }
        }

        // In a real implementation, this would trigger measurement
        // For now, return simulated results
        let results: Vec<u8> = qubits
            .iter()
            .map(|_| {
                // Pseudo-random based on timestamp
                (current_timestamp() % 2) as u8
            })
            .collect();

        Ok(results)
    }

    async fn reset(&self) -> Result<(), Self::Error> {
        tracing::info!("Resetting quantum state");
        // In a real implementation, this would reset the QPU state
        Ok(())
    }

    fn quantum_state(&self) -> Option<&QuantumState> {
        self.state.as_ref()
    }

    fn qpu_id(&self) -> &str {
        self.qpu.id()
    }
}

// ============================================================================
// Backend Implementation for RigettiQCSBackend
// ============================================================================

#[async_trait]
impl Backend for RigettiQCSBackend {
    type Error = RigettiError;

    fn name(&self) -> &str {
        "RigettiQCS"
    }

    fn supported_program_types(&self) -> Vec<ProgramType> {
        vec![ProgramType::NativeQuil, ProgramType::OpenQASM]
    }

    async fn submit(&self, program: &QProgram, shots: u32) -> Result<QCSJob, Self::Error> {
        tracing::info!("Submitting program to {}", self.qpu);

        // Validate program
        self.validate_program(program)?;

        // Create job
        let job = QCSJob::new(uuid_simple(), program.clone(), shots, self.qpu);

        // In a real implementation, this would submit to QCS API
        tracing::info!("Job submitted: {}", job.job_id);
        Ok(job)
    }

    async fn is_available(&self) -> bool {
        // In a real implementation, this would check QPU availability
        true
    }

    fn estimated_execution_time(&self, program: &QProgram, shots: u32) -> Option<u64> {
        // Rough estimation: base time + per-instruction + per-shot
        let base_time_ms = 50;
        let per_instruction_ms = 10;
        let per_shot_ms = 5;

        Some(
            base_time_ms
                + (program.num_instructions() as u64 * per_instruction_ms)
                + (shots as u64 * per_shot_ms),
        )
    }
}

// ============================================================================
// Builder Pattern
// ============================================================================

/// Builder for constructing RigettiQCSBackend
pub struct RigettiQCSBackendBuilder {
    config: Option<QCSConfig>,
    qpu: QPU,
}

impl RigettiQCSBackendBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: None,
            qpu: QPU::Ankaa3,
        }
    }

    /// Set the QCS configuration
    pub fn config(mut self, config: QCSConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the target QPU
    pub fn qpu(mut self, qpu: QPU) -> Self {
        self.qpu = qpu;
        self
    }

    /// Build the backend
    pub fn build(self) -> Result<RigettiQCSBackend, RigettiError> {
        let config = self.config.ok_or_else(|| {
            RigettiError::ConnectionFailed("QCS configuration is required".to_string())
        })?;

        Ok(RigettiQCSBackend::new(config, self.qpu))
    }
}

impl Default for RigettiQCSBackendBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_state_creation() {
        let state = QuantumState::new(2);
        assert_eq!(state.num_qubits, 2);
        assert_eq!(state.amplitudes.len(), 4);
        // Initial state should be |00⟩
        assert!((state.amplitudes[0].re - 1.0).abs() < 1e-10);
        assert!((state.amplitudes[1].norm()).abs() < 1e-10);
    }

    #[test]
    fn test_quantum_state_probability() {
        let state = QuantumState::new(1);
        assert!((state.probability(0) - 1.0).abs() < 1e-10);
        assert!((state.probability(1) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_instruction_to_quil() {
        assert_eq!(Instruction::H(0).to_quil_string(), "H 0");
        assert_eq!(Instruction::CNOT(0, 1).to_quil_string(), "CNOT 0 1");
        assert_eq!(Instruction::Rx(3.14, 2).to_quil_string(), "Rx(3.14) 2");
        assert_eq!(
            Instruction::MEASURE(0, 0).to_quil_string(),
            "MEASURE 0 [0]"
        );
        assert_eq!(Instruction::WAIT.to_quil_string(), "WAIT");
        assert_eq!(Instruction::HALT.to_quil_string(), "HALT");
    }

    #[test]
    fn test_qprogram_creation() {
        let mut program = QProgram::new(ProgramType::NativeQuil, 2);
        program.add_instruction(Instruction::H(0));
        program.add_instruction(Instruction::CNOT(0, 1));
        program.num_classical_registers = 2;

        assert_eq!(program.num_qubits, 2);
        assert_eq!(program.num_instructions(), 2);
    }

    #[test]
    fn test_qprogram_to_quil() {
        let mut program = QProgram::new(ProgramType::NativeQuil, 2);
        program.add_instruction(Instruction::H(0));
        program.add_instruction(Instruction::CNOT(0, 1));
        program.num_classical_registers = 2;

        let quil = program.to_quil_string();
        assert!(quil.contains("DECLARE ro BIT[2]"));
        assert!(quil.contains("H 0"));
        assert!(quil.contains("CNOT 0 1"));
    }

    #[test]
    fn test_qpu_properties() {
        assert_eq!(QPU::AspenM12.num_qubits(), 12);
        assert_eq!(QPU::Ankaa2.num_qubits(), 30);
        assert_eq!(QPU::Ankaa3.num_qubits(), 40);

        assert_eq!(QPU::AspenM12.id(), "Aspen-M-12");
        assert_eq!(QPU::Ankaa2.id(), "Ankaa-2");
        assert_eq!(QPU::Ankaa3.id(), "Ankaa-3");
    }

    #[test]
    fn test_qpu_connectivity() {
        let qpu = QPU::AspenM12;
        assert!(qpu.are_connected(0, 1));
        assert!(!qpu.are_connected(0, 2));

        let qpu = QPU::Ankaa2;
        assert!(qpu.are_connected(0, 1));
        assert!(qpu.are_connected(0, 6)); // Below in grid
    }

    #[test]
    fn test_qcs_config() {
        let config = QCSConfig::new(
            "token123".to_string(),
            "https://qcs.rigetti.com".to_string(),
        )
        .with_reservation("res_123".to_string())
        .with_timeout(60);

        assert_eq!(config.api_token, "token123");
        assert_eq!(config.endpoint_url, "https://qcs.rigetti.com");
        assert_eq!(config.reservation_id, Some("res_123".to_string()));
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_qam_result() {
        let mut readout = HashMap::new();
        readout.insert("ro".to_string(), vec![0, 1, 1, 0]);

        let result = QAMResult::new(readout, 150, "Ankaa-3".to_string());
        assert_eq!(result.execution_time_ms, 150);
        assert_eq!(result.qpu_used, "Ankaa-3");
        assert_eq!(
            result.get_register("ro"),
            Some(&vec![0, 1, 1, 0])
        );
        assert_eq!(result.get_register("missing"), None);
        assert_eq!(result.num_outcomes(), 4);
    }

    #[test]
    fn test_qcs_job_creation() {
        let program = QProgram::new(ProgramType::NativeQuil, 4);
        let job = QCSJob::new("job_123".to_string(), program, 1000, QPU::Ankaa3);

        assert_eq!(job.job_id, "job_123");
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.shots, 1000);
    }

    #[test]
    fn test_reservation_validity() {
        let mut reservation = Reservation {
            reservation_id: "res_1".to_string(),
            qpu: QPU::Ankaa3,
            start_time: 1000,
            duration_secs: 600,
            active: true,
        };

        assert!(reservation.is_valid(1500)); // Within duration
        assert!(!reservation.is_valid(900)); // Before start
        assert!(!reservation.is_valid(1700)); // After end

        reservation.active = false;
        assert!(!reservation.is_valid(1500)); // Inactive
    }

    #[tokio::test]
    async fn test_backend_creation() {
        let config = QCSConfig::new("test_token".to_string(), "https://test.qcs.com".to_string());
        let backend = RigettiQCSBackend::new(config, QPU::Ankaa3);

        assert_eq!(backend.name(), "RigettiQCS");
        assert_eq!(backend.qpu_id(), "Ankaa-3");
        assert!(backend.is_available().await);
    }

    #[tokio::test]
    async fn test_backend_run() {
        let config = QCSConfig::new("test_token".to_string(), "https://test.qcs.com".to_string());
        let backend = RigettiQCSBackend::new(config, QPU::Ankaa3);

        let mut program = QProgram::new(ProgramType::NativeQuil, 2);
        program.add_instruction(Instruction::H(0));
        program.add_instruction(Instruction::CNOT(0, 1));
        program.num_classical_registers = 2;

        let result = backend.run(&program, 100).await.unwrap();
        assert_eq!(result.qpu_used, "Ankaa-3");
        assert!(result.readout_values.contains_key("ro"));
    }

    #[tokio::test]
    async fn test_backend_submit() {
        let config = QCSConfig::new("test_token".to_string(), "https://test.qcs.com".to_string());
        let backend = RigettiQCSBackend::new(config, QPU::Ankaa3);

        let mut program = QProgram::new(ProgramType::NativeQuil, 2);
        program.add_instruction(Instruction::H(0));
        program.add_instruction(Instruction::CNOT(0, 1));

        let job = backend.submit(&program, 1000).await.unwrap();
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.shots, 1000);
    }

    #[test]
    fn test_program_validation() {
        let config = QCSConfig::new("test_token".to_string(), "https://test.qcs.com".to_string());
        let backend = RigettiQCSBackend::new(config, QPU::AspenM12);

        // Valid program
        let mut valid_program = QProgram::new(ProgramType::NativeQuil, 8);
        valid_program.add_instruction(Instruction::H(0));
        assert!(backend.validate_program(&valid_program).is_ok());

        // Invalid: too many qubits
        let mut invalid_program = QProgram::new(ProgramType::NativeQuil, 20);
        invalid_program.add_instruction(Instruction::H(0));
        assert!(backend.validate_program(&invalid_program).is_err());

        // Invalid: disconnected qubits
        let mut disconnected = QProgram::new(ProgramType::NativeQuil, 12);
        disconnected.add_instruction(Instruction::CNOT(0, 2)); // Not connected on Aspen-M-12
        assert!(backend.validate_program(&disconnected).is_err());
    }

    #[test]
    fn test_backend_builder() {
        let config = QCSConfig::new("test_token".to_string(), "https://test.qcs.com".to_string());

        let backend = RigettiQCSBackendBuilder::new()
            .config(config)
            .qpu(QPU::Ankaa3)
            .build()
            .unwrap();

        assert_eq!(backend.qpu(), QPU::Ankaa3);
    }

    #[test]
    fn test_backend_builder_no_config() {
        let result = RigettiQCSBackendBuilder::new()
            .qpu(QPU::Ankaa3)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_supported_program_types() {
        let config = QCSConfig::new("test_token".to_string(), "https://test.qcs.com".to_string());
        let backend = RigettiQCSBackend::new(config, QPU::Ankaa3);

        let types = backend.supported_program_types();
        assert!(types.contains(&ProgramType::NativeQuil));
        assert!(types.contains(&ProgramType::OpenQASM));
    }

    #[test]
    fn test_estimated_execution_time() {
        let config = QCSConfig::new("test_token".to_string(), "https://test.qcs.com".to_string());
        let backend = RigettiQCSBackend::new(config, QPU::Ankaa3);

        let mut program = QProgram::new(ProgramType::NativeQuil, 2);
        program.add_instruction(Instruction::H(0));
        program.add_instruction(Instruction::CNOT(0, 1));

        let time = backend.estimated_execution_time(&program, 100).unwrap();
        assert!(time > 0);
    }

    #[tokio::test]
    async fn test_reservation_system() {
        let config = QCSConfig::new("test_token".to_string(), "https://test.qcs.com".to_string());
        let mut backend = RigettiQCSBackend::new(config, QPU::Ankaa3);

        // Reserve QPU
        let reservation = backend.reserve_qpu(600).await.unwrap();
        assert!(reservation.active);
        assert_eq!(reservation.qpu, QPU::Ankaa3);
        assert!(backend.has_valid_reservation());

        // Release reservation
        backend.release_reservation().await.unwrap();
        assert!(!backend.has_valid_reservation());
    }

    #[test]
    fn test_error_display() {
        let errors = vec![
            RigettiError::ConnectionFailed("timeout".to_string()),
            RigettiError::AuthenticationFailed("invalid token".to_string()),
            RigettiError::QPUUnavailable("maintenance".to_string()),
            RigettiError::ReservationFailed("no slots".to_string()),
            RigettiError::CompilationFailed("syntax error".to_string()),
            RigettiError::ExecutionFailed("hardware error".to_string()),
            RigettiError::InvalidProgram("too many qubits".to_string()),
            RigettiError::Timeout(30),
        ];

        for error in errors {
            let msg = error.to_string();
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn test_serialization() {
        let config = QCSConfig::new(
            "token".to_string(),
            "https://qcs.rigetti.com".to_string(),
        );
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: QCSConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.api_token, deserialized.api_token);
    }
}
