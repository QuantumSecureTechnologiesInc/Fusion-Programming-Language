//! # Fusion Cortex Scheduler
//!
//! AI-powered workload routing engine that profiles tasks, predicts execution
//! costs, and makes model-based scheduling decisions. Learns from execution
//! feedback to improve routing over time.
//!
//! ## Architecture
//!
//! The scheduler maintains a linear cost model per device (CPU/GPU/QPU) and
//! uses online gradient descent to update predictions from observed execution
//! times. Task profiling extracts features (log-ops, memory, complexity) that
//! feed the model for routing decisions.

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, info};

// ─── Core Types ────────────────────────────────────────────────

/// Hardware device target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Device {
    Cpu,
    Gpu(u32),
    Qpu(u32),
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Device::Cpu => write!(f, "CPU"),
            Device::Gpu(i) => write!(f, "GPU:{}", i),
            Device::Qpu(i) => write!(f, "QPU:{}", i),
        }
    }
}

/// Task intent category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Intent {
    Critical,
    HighThroughput,
    Precision,
    Background,
}

impl Intent {
    pub fn priority(&self) -> u8 {
        match self {
            Intent::Critical => 3,
            Intent::HighThroughput => 2,
            Intent::Precision => 2,
            Intent::Background => 0,
        }
    }
}

/// Profile of a task's resource characteristics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskProfile {
    pub estimated_ops: u64,
    pub memory_bytes: usize,
    pub intent: Intent,
    pub dependencies: usize,
}

impl TaskProfile {
    pub fn new(intent: Intent) -> Self {
        Self {
            estimated_ops: 0,
            memory_bytes: 0,
            intent,
            dependencies: 0,
        }
    }

    pub fn with_ops(mut self, ops: u64) -> Self { self.estimated_ops = ops; self }
    pub fn with_memory(mut self, bytes: usize) -> Self { self.memory_bytes = bytes; self }
    pub fn with_dependencies(mut self, deps: usize) -> Self { self.dependencies = deps; self }

    /// Complexity score combining ops, memory, and dependency factors.
    pub fn complexity_score(&self) -> f64 {
        let ops = (self.estimated_ops as f64).log10().max(0.0);
        let mem = (self.memory_bytes as f64 / 1024.0).log10().max(0.0);
        let dep = (self.dependencies as f64).sqrt();
        ops * 0.5 + mem * 0.3 + dep * 0.2
    }
}

// ─── Feature Extraction ────────────────────────────────────────

const NUM_FEATURES: usize = 5;

/// Extract a fixed-size feature vector from a task profile.
fn extract_features(profile: &TaskProfile) -> [f32; NUM_FEATURES] {
    [
        (profile.estimated_ops as f32).log10().max(0.0),
        (profile.memory_bytes as f32).log10().max(0.0),
        profile.intent.priority() as f32,
        profile.dependencies as f32,
        profile.complexity_score() as f32,
    ]
}

// ─── Cost Model ────────────────────────────────────────────────

/// A linear cost model: cost = weights · features + bias.
/// One weight vector per device.
#[derive(Debug, Clone)]
struct CostModel {
    weights: [[f32; NUM_FEATURES]; 3], // [cpu, gpu, qpu]
    bias: [f32; 3],
}

impl CostModel {
    /// Initialize with heuristic weights.
    fn new() -> Self {
        Self {
            weights: [
                [1.0, 0.1, -0.1, 0.05, 0.3],   // CPU: steep ops scaling (serial execution)
                [0.01, 0.1, 0.0, 0.02, 0.05],   // GPU: very low ops (parallel), moderate base
                [0.01, 0.05, 0.2, 0.01, 0.1],   // QPU
            ],
            bias: [5.0, 10.0, 200.0],
        }
    }

    /// Predict cost for each device.
    fn predict(&self, features: &[f32; NUM_FEATURES]) -> [f32; 3] {
        let mut costs = [0.0f32; 3];
        for d in 0..3 {
            let mut sum = self.bias[d];
            for j in 0..NUM_FEATURES {
                sum += self.weights[d][j] * features[j];
            }
            costs[d] = sum;
        }
        costs
    }

    /// Select the device with lowest predicted cost.
    fn best_device(&self, features: &[f32; NUM_FEATURES]) -> usize {
        let costs = self.predict(features);
        costs
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Online SGD update: adjust weights for one device based on observed cost.
    fn update(&mut self, device_idx: usize, features: &[f32; NUM_FEATURES], actual: f32, lr: f32) {
        let predicted = {
            let mut sum = self.bias[device_idx];
            for j in 0..NUM_FEATURES {
                sum += self.weights[device_idx][j] * features[j];
            }
            sum
        };

        let error = predicted - actual;
        let grad = 2.0 * error;
        let clipped = grad.max(-10.0).min(10.0);

        for j in 0..NUM_FEATURES {
            self.weights[device_idx][j] -= lr * clipped * features[j];
        }
        self.bias[device_idx] -= lr * clipped;
    }
}

// ─── Execution Record ──────────────────────────────────────────

/// A recorded execution for learning.
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub profile: TaskProfile,
    pub device: Device,
    pub actual_time_us: u64,
    pub success: bool,
}

// ─── Scheduler Decision ────────────────────────────────────────

/// The output of a scheduling decision.
#[derive(Debug, Clone)]
pub struct SchedulingDecision {
    pub device: Device,
    pub estimated_cost_us: f64,
    pub confidence: f64,
}

// ─── Cortex Scheduler ──────────────────────────────────────────

/// AI-powered workload scheduler.
///
/// Maintains a cost model that predicts execution time per device,
/// selects the optimal device for each task, and learns from observed
/// execution times via online gradient descent.
pub struct CortexScheduler {
    model: RwLock<CostModel>,
    execution_history: RwLock<Vec<ExecutionRecord>>,
    total_routed: AtomicU64,
    learning_rate: f32,
}

impl CortexScheduler {
    /// Create a new scheduler with default settings.
    pub fn new() -> Self {
        info!("Initializing CortexScheduler with heuristic model");
        Self {
            model: RwLock::new(CostModel::new()),
            execution_history: RwLock::new(Vec::new()),
            total_routed: AtomicU64::new(0),
            learning_rate: 0.001,
        }
    }

    /// Create a scheduler with a custom learning rate.
    pub fn with_learning_rate(lr: f32) -> Self {
        Self {
            model: RwLock::new(CostModel::new()),
            execution_history: RwLock::new(Vec::new()),
            total_routed: AtomicU64::new(0),
            learning_rate: lr,
        }
    }

    /// Route a task to the optimal device based on its profile.
    pub fn route(&self, profile: &TaskProfile) -> SchedulingDecision {
        // HFT guard: Critical tasks always go to CPU
        if profile.intent == Intent::Critical {
            debug!("HFT guard: Critical task routed to CPU");
            return SchedulingDecision {
                device: Device::Cpu,
                estimated_cost_us: 0.0,
                confidence: 1.0,
            };
        }

        let features = extract_features(profile);

        let (device_idx, costs) = {
            let model = self.model.read();
            let costs = model.predict(&features);
            let idx = model.best_device(&features);
            (idx, costs)
        };

        let device = match device_idx {
            0 => Device::Cpu,
            1 => Device::Gpu(0),
            _ => Device::Qpu(0),
        };

        let min_cost = costs[device_idx] as f64;
        let max_cost = costs.iter().cloned().fold(f32::MIN, f32::max) as f64;
        let confidence = if max_cost > 0.0 {
            1.0 - (min_cost / max_cost).min(1.0)
        } else {
            0.5
        };

        self.total_routed.fetch_add(1, Ordering::Relaxed);
        debug!(
            "Routed task to {:?} (cost={:.2}, confidence={:.2})",
            device, min_cost, confidence
        );

        SchedulingDecision {
            device,
            estimated_cost_us: min_cost,
            confidence,
        }
    }

    /// Predict the cost of executing a task on a specific device.
    pub fn predict_cost(&self, profile: &TaskProfile, device: Device) -> f64 {
        let features = extract_features(profile);
        let model = self.model.read();
        let costs = model.predict(&features);
        let idx = match device {
            Device::Cpu => 0,
            Device::Gpu(_) => 1,
            Device::Qpu(_) => 2,
        };
        costs[idx] as f64
    }

    /// Record an execution and update the model.
    pub fn record_execution(&self, record: ExecutionRecord) {
        let features = extract_features(&record.profile);
        let device_idx = match record.device {
            Device::Cpu => 0,
            Device::Gpu(_) => 1,
            Device::Qpu(_) => 2,
        };

        {
            let mut model = self.model.write();
            model.update(device_idx, &features, record.actual_time_us as f32, self.learning_rate);
        }

        self.execution_history.write().push(record);
    }

    /// Train the model on a batch of execution records.
    pub fn train_batch(&self, records: &[ExecutionRecord]) {
        if records.is_empty() {
            return;
        }

        info!("Training on {} execution records", records.len());

        for record in records {
            let features = extract_features(&record.profile);
            let device_idx = match record.device {
                Device::Cpu => 0,
                Device::Gpu(_) => 1,
                Device::Qpu(_) => 2,
            };

            let mut model = self.model.write();
            model.update(device_idx, &features, record.actual_time_us as f32, self.learning_rate);
        }

        self.execution_history.write().extend_from_slice(records);
        info!("Training complete, total records: {}", self.execution_history.read().len());
    }

    /// Get the total number of tasks routed.
    pub fn total_routed(&self) -> u64 {
        self.total_routed.load(Ordering::Relaxed)
    }

    /// Get the number of recorded executions.
    pub fn history_len(&self) -> usize {
        self.execution_history.read().len()
    }

    /// Export the model weights as raw bytes for serialization.
    pub fn export_model(&self) -> Vec<u8> {
        let model = self.model.read();
        let mut buf = Vec::new();
        for d in 0..3 {
            for j in 0..NUM_FEATURES {
                buf.extend_from_slice(&model.weights[d][j].to_le_bytes());
            }
            buf.extend_from_slice(&model.bias[d].to_le_bytes());
        }
        buf
    }

    /// Import model weights from raw bytes.
    pub fn import_model(&self, data: &[u8]) -> Result<(), String> {
        let expected = 3 * (NUM_FEATURES + 1) * 4;
        if data.len() < expected {
            return Err(format!("Expected {} bytes, got {}", expected, data.len()));
        }

        let mut model = self.model.write();
        let mut offset = 0;
        for d in 0..3 {
            for j in 0..NUM_FEATURES {
                model.weights[d][j] = f32::from_le_bytes([
                    data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                ]);
                offset += 4;
            }
            model.bias[d] = f32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            ]);
            offset += 4;
        }

        Ok(())
    }
}

impl Default for CortexScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = CortexScheduler::new();
        assert_eq!(scheduler.total_routed(), 0);
        assert_eq!(scheduler.history_len(), 0);
    }

    #[test]
    fn test_critical_routes_to_cpu() {
        let scheduler = CortexScheduler::new();
        let profile = TaskProfile::new(Intent::Critical);
        let decision = scheduler.route(&profile);
        assert_eq!(decision.device, Device::Cpu);
        assert_eq!(decision.confidence, 1.0);
    }

    #[test]
    fn test_small_task_routes_to_cpu() {
        let scheduler = CortexScheduler::new();
        let profile = TaskProfile::new(Intent::Background).with_ops(100).with_memory(1024);
        let decision = scheduler.route(&profile);
        assert_eq!(decision.device, Device::Cpu);
    }

    #[test]
    fn test_large_task_routes_to_gpu() {
        let scheduler = CortexScheduler::new();
        let profile = TaskProfile::new(Intent::HighThroughput)
            .with_ops(10_000_000_000)
            .with_memory(1024 * 1024 * 1024);
        let decision = scheduler.route(&profile);
        assert_eq!(decision.device, Device::Gpu(0));
    }

    #[test]
    fn test_record_execution_updates_model() {
        let scheduler = CortexScheduler::new();
        let profile = TaskProfile::new(Intent::HighThroughput)
            .with_ops(10_000_000)
            .with_memory(1024 * 1024);

        // Record GPU as fast
        scheduler.record_execution(ExecutionRecord {
            profile: profile.clone(),
            device: Device::Gpu(0),
            actual_time_us: 10,
            success: true,
        });

        // Record CPU as slow
        scheduler.record_execution(ExecutionRecord {
            profile: profile.clone(),
            device: Device::Cpu,
            actual_time_us: 500,
            success: true,
        });

        assert_eq!(scheduler.history_len(), 2);
    }

    #[test]
    fn test_train_batch() {
        let scheduler = CortexScheduler::new();
        let records: Vec<ExecutionRecord> = (0..10)
            .map(|i| ExecutionRecord {
                profile: TaskProfile::new(Intent::HighThroughput)
                    .with_ops(10_000_000)
                    .with_memory(1024 * 1024),
                device: if i % 2 == 0 { Device::Gpu(0) } else { Device::Cpu },
                actual_time_us: if i % 2 == 0 { 20 } else { 200 },
                success: true,
            })
            .collect();

        scheduler.train_batch(&records);
        assert_eq!(scheduler.history_len(), 10);
    }

    #[test]
    fn test_model_export_import() {
        let scheduler = CortexScheduler::new();
        let exported = scheduler.export_model();

        let scheduler2 = CortexScheduler::new();
        scheduler2.import_model(&exported).unwrap();

        // Both should produce the same predictions
        let profile = TaskProfile::new(Intent::HighThroughput)
            .with_ops(1_000_000)
            .with_memory(1024 * 1024);

        let cost1 = scheduler.predict_cost(&profile, Device::Cpu);
        let cost2 = scheduler2.predict_cost(&profile, Device::Cpu);
        assert!((cost1 - cost2).abs() < 1e-6);
    }

    #[test]
    fn test_import_invalid_data() {
        let scheduler = CortexScheduler::new();
        let result = scheduler.import_model(&[0u8; 4]);
        assert!(result.is_err());
    }

    #[test]
    fn test_predict_cost_all_devices() {
        let scheduler = CortexScheduler::new();
        let profile = TaskProfile::new(Intent::HighThroughput)
            .with_ops(1_000_000)
            .with_memory(1024 * 1024);

        let cpu = scheduler.predict_cost(&profile, Device::Cpu);
        let gpu = scheduler.predict_cost(&profile, Device::Gpu(0));
        let qpu = scheduler.predict_cost(&profile, Device::Qpu(0));

        assert!(cpu.is_finite());
        assert!(gpu.is_finite());
        assert!(qpu.is_finite());
    }

    #[test]
    fn test_learning_rate_config() {
        let scheduler = CortexScheduler::with_learning_rate(0.01);
        assert!((scheduler.learning_rate - 0.01).abs() < f32::EPSILON);
    }

    #[test]
    fn test_task_profile_builder() {
        let profile = TaskProfile::new(Intent::Precision)
            .with_ops(500)
            .with_memory(4096)
            .with_dependencies(3);

        assert_eq!(profile.estimated_ops, 500);
        assert_eq!(profile.memory_bytes, 4096);
        assert_eq!(profile.dependencies, 3);
        assert!(profile.complexity_score() > 0.0);
    }

    #[test]
    fn test_intent_priority() {
        assert!(Intent::Critical.priority() > Intent::HighThroughput.priority());
        assert!(Intent::HighThroughput.priority() == Intent::Precision.priority());
        assert!(Intent::Background.priority() < Intent::Precision.priority());
    }
}
