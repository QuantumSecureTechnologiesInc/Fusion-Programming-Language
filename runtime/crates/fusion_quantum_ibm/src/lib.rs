use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for IBM Quantum operations
#[derive(Error, Debug)]
pub enum IBMError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("API authentication failed: {0}")]
    Authentication(String),
    
    #[error("Job not found: {0}")]
    JobNotFound(String),
    
    #[error("Job failed: {0}")]
    JobFailed(String),
    
    #[error("Transpilation error: {0}")]
    TranspilationError(String),
    
    #[error("Invalid circuit: {0}")]
    InvalidCircuit(String),
    
    #[error("Backend error: {0}")]
    BackendError(String),
}

/// Configuration for IBM Quantum backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBMConfig {
    pub api_token: String,
    pub base_url: String,
    pub instance: String,
    pub hub: String,
    pub group: String,
    pub project: String,
}

impl Default for IBMConfig {
    fn default() -> Self {
        Self {
            api_token: String::new(),
            base_url: "https://auth.quantum-computing.ibm.com/api".to_string(),
            instance: "ibm-q".to_string(),
            hub: "ibm-q".to_string(),
            group: "open".to_string(),
            project: "main".to_string(),
        }
    }
}

/// Quantum gate types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuantumGate {
    H(usize),           // Hadamard gate
    X(usize),           // Pauli-X gate
    Y(usize),           // Pauli-Y gate
    Z(usize),           // Pauli-Z gate
    CNOT(usize, usize), // Controlled-NOT gate
    T(usize),           // T gate
    S(usize),           // S gate
    Rx(usize, f64),     // Rotation around X-axis
    Ry(usize, f64),     // Rotation around Y-axis
    Rz(usize, f64),     // Rotation around Z-axis
    Measure(usize, usize), // Measurement
}

/// Quantum circuit representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCircuit {
    pub name: String,
    pub num_qubits: usize,
    pub num_clbits: usize,
    pub gates: Vec<QuantumGate>,
    pub metadata: HashMap<String, String>,
}

impl QuantumCircuit {
    /// Create a new quantum circuit
    pub fn new(name: &str, num_qubits: usize, num_clbits: usize) -> Self {
        Self {
            name: name.to_string(),
            num_qubits,
            num_clbits,
            gates: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a gate to the circuit
    pub fn add_gate(&mut self, gate: QuantumGate) {
        self.gates.push(gate);
    }
    
    /// Convert circuit to OpenQASM 2.0 string
    pub fn to_qasm(&self) -> String {
        let mut qasm = String::new();
        qasm.push_str("OPENQASM 2.0;\n");
        qasm.push_str("include \"qelib1.inc\";\n\n");
        qasm.push_str(&format!("qreg q[{}];\n", self.num_qubits));
        if self.num_clbits > 0 {
            qasm.push_str(&format!("creg c[{}];\n", self.num_clbits));
        }
        qasm.push_str("\n");
        
        for gate in &self.gates {
            match gate {
                QuantumGate::H(qubit) => {
                    qasm.push_str(&format!("h q[{}];\n", qubit));
                }
                QuantumGate::X(qubit) => {
                    qasm.push_str(&format!("x q[{}];\n", qubit));
                }
                QuantumGate::Y(qubit) => {
                    qasm.push_str(&format!("y q[{}];\n", qubit));
                }
                QuantumGate::Z(qubit) => {
                    qasm.push_str(&format!("z q[{}];\n", qubit));
                }
                QuantumGate::CNOT(control, target) => {
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", control, target));
                }
                QuantumGate::T(qubit) => {
                    qasm.push_str(&format!("t q[{}];\n", qubit));
                }
                QuantumGate::S(qubit) => {
                    qasm.push_str(&format!("s q[{}];\n", qubit));
                }
                QuantumGate::Rx(qubit, theta) => {
                    qasm.push_str(&format!("rx({}) q[{}];\n", theta, qubit));
                }
                QuantumGate::Ry(qubit, theta) => {
                    qasm.push_str(&format!("ry({}) q[{}];\n", theta, qubit));
                }
                QuantumGate::Rz(qubit, theta) => {
                    qasm.push_str(&format!("rz({}) q[{}];\n", theta, qubit));
                }
                QuantumGate::Measure(qubit, clbit) => {
                    qasm.push_str(&format!("measure q[{}] -> c[{}];\n", qubit, clbit));
                }
            }
        }
        
        qasm
    }
}

/// Job status enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "QUEUED"),
            JobStatus::Running => write!(f, "RUNNING"),
            JobStatus::Completed => write!(f, "COMPLETED"),
            JobStatus::Failed => write!(f, "FAILED"),
            JobStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

/// IBM Quantum job representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBMJob {
    pub job_id: String,
    pub status: JobStatus,
    pub result: Option<IBMBackendResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Result from IBM Quantum backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBMBackendResult {
    pub counts: HashMap<String, u32>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, String>,
}

/// Backend trait for quantum computing
#[async_trait]
pub trait Backend {
    type Error: std::fmt::Display;
    
    async fn submit_circuit(&self, circuit: &QuantumCircuit, shots: u32) -> Result<String, Self::Error>;
    async fn get_job_status(&self, job_id: &str) -> Result<JobStatus, Self::Error>;
    async fn get_job_results(&self, job_id: &str) -> Result<IBMBackendResult, Self::Error>;
    async fn cancel_job(&self, job_id: &str) -> Result<(), Self::Error>;
    fn backend_name(&self) -> &str;
}

/// IBM Quantum REST API backend
#[derive(Debug)]
pub struct IBMBackend {
    config: IBMConfig,
    client: Client,
}

impl IBMBackend {
    /// Create a new IBM Quantum backend
    pub fn new(config: IBMConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
    
    /// Build authorization header
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.api_token)
    }
    
    /// Build base URL for API calls
    fn base_api_url(&self) -> String {
        format!(
            "{}/Networks/{}/Groups/{}/Projects/{}",
            self.config.base_url, self.config.hub, self.config.group, self.config.project
        )
    }
    
    /// Transpile a generic circuit description to OpenQASM 2.0
    pub fn transpile_circuit(&self, circuit: &QuantumCircuit) -> Result<String, IBMError> {
        // Validate circuit
        if circuit.num_qubits == 0 {
            return Err(IBMError::TranspilationError(
                "Circuit must have at least 1 qubit".to_string(),
            ));
        }
        
        if circuit.num_clbits > circuit.num_qubits {
            return Err(IBMError::TranspilationError(
                "Classical bits cannot exceed qubits".to_string(),
            ));
        }
        
        // Validate gate indices
        for gate in &circuit.gates {
            match gate {
                QuantumGate::H(q) | QuantumGate::X(q) | QuantumGate::Y(q) | QuantumGate::Z(q) |
                QuantumGate::T(q) | QuantumGate::S(q) | QuantumGate::Rx(q, _) | 
                QuantumGate::Ry(q, _) | QuantumGate::Rz(q, _) => {
                    if *q >= circuit.num_qubits {
                        return Err(IBMError::TranspilationError(
                            format!("Gate qubit index {} exceeds circuit qubit count {}", q, circuit.num_qubits),
                        ));
                    }
                }
                QuantumGate::CNOT(c, t) => {
                    if *c >= circuit.num_qubits || *t >= circuit.num_qubits {
                        return Err(IBMError::TranspilationError(
                            format!("CNOT qubit indices ({}, {}) exceed circuit qubit count {}", c, t, circuit.num_qubits),
                        ));
                    }
                }
                QuantumGate::Measure(q, cl) => {
                    if *q >= circuit.num_qubits {
                        return Err(IBMError::TranspilationError(
                            format!("Measurement qubit index {} exceeds circuit qubit count {}", q, circuit.num_qubits),
                        ));
                    }
                    if *cl >= circuit.num_clbits {
                        return Err(IBMError::TranspilationError(
                            format!("Measurement classical bit index {} exceeds circuit clbit count {}", cl, circuit.num_clbits),
                        ));
                    }
                }
            }
        }
        
        Ok(circuit.to_qasm())
    }
    
    /// Parse IBM Quantum API job status response
    fn parse_job_status(status: &str) -> Result<JobStatus, IBMError> {
        match status.to_uppercase().as_str() {
            "QUEUED" | "JOB_STATE_QUEUED" => Ok(JobStatus::Queued),
            "RUNNING" | "JOB_STATE_RUNNING" => Ok(JobStatus::Running),
            "COMPLETED" | "DONE" | "JOB_STATE_DONE" => Ok(JobStatus::Completed),
            "FAILED" | "ERROR" | "JOB_STATE_ERROR" => Ok(JobStatus::Failed),
            "CANCELLED" | "CANCELED" | "JOB_STATE_CANCELLED" => Ok(JobStatus::Cancelled),
            _ => Err(IBMError::BackendError(format!("Unknown job status: {}", status))),
        }
    }
}

#[async_trait]
impl Backend for IBMBackend {
    type Error = IBMError;
    
    async fn submit_circuit(&self, circuit: &QuantumCircuit, shots: u32) -> Result<String, IBMError> {
        let qasm = self.transpile_circuit(circuit)?;
        
        let url = format!("{}/Jobs", self.base_api_url());
        
        let payload = serde_json::json!({
            "circuits": [{"qasm": qasm}],
            "backend": "ibmq_manila",
            "shots": shots,
            "hub": self.config.hub,
            "group": self.config.group,
            "project": self.config.project,
        });
        
        let response = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IBMError::BackendError(format!(
                "Job submission failed ({}): {}",
                status, body
            )));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        let job_id = response_json["id"]
            .as_str()
            .ok_or_else(|| IBMError::BackendError("Missing job ID in response".to_string()))?
            .to_string();
        
        tracing::info!("Submitted job {} to IBM Quantum", job_id);
        Ok(job_id)
    }
    
    async fn get_job_status(&self, job_id: &str) -> Result<JobStatus, IBMError> {
        let url = format!("{}/Jobs/{}", self.base_api_url(), job_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(IBMError::JobNotFound(job_id.to_string()));
            }
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IBMError::BackendError(format!(
                "Failed to get job status ({}): {}",
                status, body
            )));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        let status_str = response_json["status"]
            .as_str()
            .ok_or_else(|| IBMError::BackendError("Missing status in response".to_string()))?;
        
        Self::parse_job_status(status_str)
    }
    
    async fn get_job_results(&self, job_id: &str) -> Result<IBMBackendResult, IBMError> {
        let url = format!("{}/Jobs/{}/result", self.base_api_url(), job_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(IBMError::JobNotFound(job_id.to_string()));
            }
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IBMError::BackendError(format!(
                "Failed to get job results ({}): {}",
                status, body
            )));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        // Parse counts from response
        let mut counts = HashMap::new();
        if let Some(counts_obj) = response_json["counts"].as_object() {
            for (key, value) in counts_obj {
                if let Some(count) = value.as_u64() {
                    counts.insert(key.clone(), count as u32);
                }
            }
        }
        
        let execution_time_ms = response_json["execution_time"]
            .as_u64()
            .unwrap_or(0);
        
        Ok(IBMBackendResult {
            counts,
            execution_time_ms,
            metadata: HashMap::new(),
        })
    }
    
    async fn cancel_job(&self, job_id: &str) -> Result<(), IBMError> {
        let url = format!("{}/Jobs/{}/cancel", self.base_api_url(), job_id);
        
        let response = self.client
            .put(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(IBMError::JobNotFound(job_id.to_string()));
            }
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IBMError::BackendError(format!(
                "Failed to cancel job ({}): {}",
                status, body
            )));
        }
        
        tracing::info!("Cancelled job {}", job_id);
        Ok(())
    }
    
    fn backend_name(&self) -> &str {
        "ibm_quantum"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_circuit_creation() {
        let mut circuit = QuantumCircuit::new("bell_state", 2, 2);
        circuit.add_gate(QuantumGate::H(0));
        circuit.add_gate(QuantumGate::CNOT(0, 1));
        circuit.add_gate(QuantumGate::Measure(0, 0));
        circuit.add_gate(QuantumGate::Measure(1, 1));

        assert_eq!(circuit.name, "bell_state");
        assert_eq!(circuit.num_qubits, 2);
        assert_eq!(circuit.num_clbits, 2);
        assert_eq!(circuit.gates.len(), 4);
    }
    
    #[test]
    fn test_qasm_generation() {
        let mut circuit = QuantumCircuit::new("bell_state", 2, 2);
        circuit.add_gate(QuantumGate::H(0));
        circuit.add_gate(QuantumGate::CNOT(0, 1));
        circuit.add_gate(QuantumGate::Measure(0, 0));
        circuit.add_gate(QuantumGate::Measure(1, 1));
        
        let qasm = circuit.to_qasm();
        
        assert!(qasm.contains("OPENQASM 2.0;"));
        assert!(qasm.contains("qreg q[2];"));
        assert!(qasm.contains("creg c[2];"));
        assert!(qasm.contains("h q[0];"));
        assert!(qasm.contains("cx q[0], q[1];"));
        assert!(qasm.contains("measure q[0] -> c[0];"));
        assert!(qasm.contains("measure q[1] -> c[1];"));
    }
    
    #[test]
    fn test_job_status_parsing() {
        assert_eq!(IBMBackend::parse_job_status("QUEUED").unwrap(), JobStatus::Queued);
        assert_eq!(IBMBackend::parse_job_status("RUNNING").unwrap(), JobStatus::Running);
        assert_eq!(IBMBackend::parse_job_status("COMPLETED").unwrap(), JobStatus::Completed);
        assert_eq!(IBMBackend::parse_job_status("FAILED").unwrap(), JobStatus::Failed);
        assert_eq!(IBMBackend::parse_job_status("CANCELLED").unwrap(), JobStatus::Cancelled);
        
        assert!(IBMBackend::parse_job_status("UNKNOWN").is_err());
    }
    
    #[test]
    fn test_error_display() {
        let error = IBMError::JobNotFound("test-123".to_string());
        assert_eq!(error.to_string(), "Job not found: test-123");
        
        let error = IBMError::Authentication("invalid token".to_string());
        assert_eq!(error.to_string(), "API authentication failed: invalid token");
        
        let error = IBMError::TranspilationError("bad gate".to_string());
        assert_eq!(error.to_string(), "Transpilation error: bad gate");
        
        let error = IBMError::InvalidCircuit("empty".to_string());
        assert_eq!(error.to_string(), "Invalid circuit: empty");
        
        let error = IBMError::BackendError("timeout".to_string());
        assert_eq!(error.to_string(), "Backend error: timeout");
    }
    
    #[test]
    fn test_transpilation_validation() {
        let backend = IBMBackend::new(IBMConfig::default());
        
        let mut circuit = QuantumCircuit::new("empty", 0, 0);
        assert!(backend.transpile_circuit(&circuit).is_err());
        
        circuit.num_qubits = 2;
        circuit.num_clbits = 3;
        assert!(backend.transpile_circuit(&circuit).is_err());
        
        let mut circuit = QuantumCircuit::new("valid", 2, 2);
        circuit.add_gate(QuantumGate::H(0));
        circuit.add_gate(QuantumGate::CNOT(0, 1));
        assert!(backend.transpile_circuit(&circuit).is_ok());
    }
    
    #[test]
    fn test_backend_name() {
        let backend = IBMBackend::new(IBMConfig::default());
        assert_eq!(backend.backend_name(), "ibm_quantum");
    }
    
    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Queued.to_string(), "QUEUED");
        assert_eq!(JobStatus::Running.to_string(), "RUNNING");
        assert_eq!(JobStatus::Completed.to_string(), "COMPLETED");
        assert_eq!(JobStatus::Failed.to_string(), "FAILED");
        assert_eq!(JobStatus::Cancelled.to_string(), "CANCELLED");
    }
    
    #[test]
    fn test_circuit_gate_operations() {
        let mut circuit = QuantumCircuit::new("multi_gate", 3, 1);
        circuit.add_gate(QuantumGate::Rx(0, std::f64::consts::PI));
        circuit.add_gate(QuantumGate::Ry(1, 0.5));
        circuit.add_gate(QuantumGate::Rz(2, 1.0));
        
        assert_eq!(circuit.gates.len(), 3);
        
        let qasm = circuit.to_qasm();
        assert!(qasm.contains("rx(3.141592653589793) q[0];"));
        assert!(qasm.contains("ry(0.5) q[1];"));
        assert!(qasm.contains("rz(1) q[2];"));
    }
    
    #[tokio::test]
    async fn test_mock_submit_circuit() {
        let backend = IBMBackend::new(IBMConfig::default());
        let mut circuit = QuantumCircuit::new("test", 2, 2);
        circuit.add_gate(QuantumGate::H(0));
        circuit.add_gate(QuantumGate::Measure(0, 0));
        
        // This will fail with network error, but tests the function signature
        let result = backend.submit_circuit(&circuit, 1024).await;
        assert!(result.is_err());
    }
}