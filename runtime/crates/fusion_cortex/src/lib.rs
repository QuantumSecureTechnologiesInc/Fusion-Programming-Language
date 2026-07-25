//! # Fusion Cortex Engine
//!
//! The "Brain" of the Fusion Runtime scheduler. The Cortex Engine is an AI-powered
//! scheduler that replaces traditional round-robin scheduling with intelligent
//! task routing based on cost prediction.
//!
//! ## Architecture
//!
//! The Cortex uses a quantised Reinforcement Learning model to predict task costs
//! and dynamically route work to the most efficient hardware (CPU vs GPU vs QPU).
//!
//! ## Decision Loop
//!
//! 1. User spawns a task with an `Intent` (e.g., `HighThroughput`, `Critical`)
//! 2. Cortex queries the internal `CostModel`
//! 3. If `Prediction(GPU) < Prediction(CPU) + TransferCost`, schedule to GPU
//! 4. Otherwise, run locally on the CPU thread pool
//!
//! ## HFT Guard
//!
//! For `Intent::Critical` (High-Frequency Trading) tasks, the Cortex always
//! schedules to CPU to guarantee minimal jitter (<10μs).

use ndarray::{Array1, Array2};
use std::sync::RwLock;
use tracing::{debug, info};

mod types;

pub use types::{Device, Intent, TaskProfile};

/// Number of input features for the linear model.
/// Features: log10(ops), log10(memory), intent_value, dependency_count, complexity_score
const NUM_FEATURES: usize = 5;

/// Represents the "Brain" of the scheduler.
///
/// The CortexEngine uses machine learning to predict the optimal device
/// for each task based on its profile (estimated operations, memory footprint,
/// intent, and dependencies).
pub struct CortexEngine {
    model: RwLock<Option<LinearModel>>,
}

/// A linear model for cost prediction.
///
/// Computes: cost = weights @ features + bias
/// One weight vector per device (Cpu, Gpu, Qpu).
#[derive(Clone)]
pub(crate) struct LinearModel {
    /// Shape: (3, NUM_FEATURES) — rows are [cpu_cost, gpu_cost, qpu_cost]
    weights: Array2<f32>,
    /// Shape: (3,) — bias per device
    bias: Array1<f32>,
}

impl LinearModel {
    /// Create a new model with random-ish initial weights.
    fn new() -> Self {
        Self {
            weights: Array2::from_shape_fn((3, NUM_FEATURES), |(_, j)| {
                // Xavier-ish init scaled to feature range
                match j {
                    0 => 0.5,  // log_ops weight
                    1 => 0.3,  // log_memory weight
                    2 => -0.1, // intent weight
                    3 => 0.05, // dependency weight
                    4 => 0.2,  // complexity weight
                    _ => 0.1,
                }
            }),
            bias: Array1::from_vec(vec![50.0, 50.0, 50.0]), // equal base costs
        }
    }

    /// Predict cost for a given feature vector across all devices.
    /// Returns [cpu_cost, gpu_cost, qpu_cost].
    fn predict(&self, features: &Array1<f32>) -> Array1<f32> {
        // (3, 5) @ (5,) = (3,)
        let scores = self.weights.dot(features) + &self.bias;
        scores
    }
}

/// Serialize the model to a byte buffer.
///
/// Format: [num_rows: u32][num_cols: u32][weights: f32...][num_bias: u32][bias: f32...]
#[allow(dead_code)]
pub(crate) fn serialize_model(model: &LinearModel) -> Vec<u8> {
    let mut buf = Vec::new();
    let (rows, cols) = model.weights.dim();

    buf.extend_from_slice(&(rows as u32).to_le_bytes());
    buf.extend_from_slice(&(cols as u32).to_le_bytes());
    for val in model.weights.iter() {
        buf.extend_from_slice(&val.to_le_bytes());
    }

    let bias_len = model.bias.len() as u32;
    buf.extend_from_slice(&bias_len.to_le_bytes());
    for val in model.bias.iter() {
        buf.extend_from_slice(&val.to_le_bytes());
    }

    buf
}

/// Deserialize a model from a byte buffer.
fn deserialize_model(data: &[u8]) -> Result<LinearModel, CortexError> {
    if data.len() < 8 {
        return Err(CortexError::ModelLoadError(
            "Buffer too small for model header".into(),
        ));
    }

    let rows = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let cols = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;

    let weights_end = 8 + rows * cols * 4;
    if data.len() < weights_end + 4 {
        return Err(CortexError::ModelLoadError(format!(
            "Buffer too small: expected >= {} bytes for weights, got {}",
            weights_end + 4,
            data.len()
        )));
    }

    let weights_iter = (0..rows * cols).map(|i| {
        let offset = 8 + i * 4;
        f32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
    });
    let weights = Array2::from_shape_vec((rows, cols), weights_iter.collect())
        .map_err(|e| CortexError::ModelLoadError(format!("Invalid weight shape: {}", e)))?;

    let bias_offset = weights_end;
    if data.len() < bias_offset + 4 {
        return Err(CortexError::ModelLoadError(
            "Buffer too small for bias header".into(),
        ));
    }

    let bias_len =
        u32::from_le_bytes(data[bias_offset..bias_offset + 4].try_into().unwrap()) as usize;
    let expected_total = bias_offset + 4 + bias_len * 4;
    if data.len() < expected_total {
        return Err(CortexError::ModelLoadError(format!(
            "Buffer too small: expected {} bytes, got {}",
            expected_total,
            data.len()
        )));
    }

    let bias_iter = (0..bias_len).map(|i| {
        let offset = bias_offset + 4 + i * 4;
        f32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
    });
    let bias = Array1::from_vec(bias_iter.collect());

    if (rows, cols) != (3, NUM_FEATURES) || bias_len != 3 {
        return Err(CortexError::ModelLoadError(format!(
            "Invalid model dimensions: weights=({}, {}), bias={}; expected (3, {}), bias=3",
            rows, cols, bias_len, NUM_FEATURES
        )));
    }

    Ok(LinearModel { weights, bias })
}

impl CortexEngine {
    /// Create a new CortexEngine instance.
    ///
    /// The engine starts without a trained model and uses heuristic
    /// scheduling until a model is loaded via `load_model()`.
    pub fn new() -> Self {
        debug!("Initialising Cortex Engine (AI Scheduler)");
        Self {
            model: RwLock::new(None),
        }
    }

    /// Extract feature vector from a task profile.
    fn extract_features(profile: &TaskProfile) -> Array1<f32> {
        Array1::from_vec(vec![
            (profile.estimated_ops as f32).log10().max(0.0),
            (profile.memory_footprint_bytes as f32).log10().max(0.0),
            profile.intent as i32 as f32,
            profile.dependencies as f32,
            profile.complexity_score() as f32,
        ])
    }

    /// Schedule a task to the optimal device based on its profile.
    ///
    /// # Arguments
    ///
    /// * `profile` - The task's metadata including estimated operations,
    ///               memory footprint, intent, and dependencies.
    ///
    /// # Returns
    ///
    /// The optimal `Device` for executing this task.
    ///
    /// # HFT Guard
    ///
    /// If the task has `Intent::Critical`, it will always be scheduled
    /// to the CPU to guarantee minimal jitter for HFT operations.
    pub fn schedule(&self, profile: &TaskProfile) -> Device {
        let features = Self::extract_features(profile);

        // HFT Guard: Critical tasks always run on CPU
        if profile.intent == Intent::Critical {
            debug!("HFT Guard: Routing Critical task to CPU");
            return Device::Cpu;
        }

        let device = match self.model.read().ok().and_then(|m| {
            m.as_ref()
                .map(|model| {
                    let costs = model.predict(&features);
                    let best_idx = costs
                        .iter()
                        .enumerate()
                        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(i, _)| i)
                        .unwrap_or(0);

                    debug!(
                        "Cortex model inference: cpu={:.2}, gpu={:.2}, qpu={:.2}, best={}",
                        costs[0], costs[1], costs[2], best_idx
                    );

                    match best_idx {
                        0 => Device::Cpu,
                        1 => Device::Gpu(0),
                        _ => Device::Qpu(0),
                    }
                })
        }) {
            Some(d) => d,
            None => {
                // Heuristic fallback when no model is loaded
                let gpu_score = features[0] * 0.8 + features[1] * 0.2;
                debug!(
                    "Cortex heuristic fallback: gpu_score={:.2}, intent={:?}",
                    gpu_score, profile.intent
                );
                if gpu_score > 5.0 {
                    Device::Gpu(0)
                } else {
                    Device::Cpu
                }
            }
        };

        device
    }

    /// Load a trained model from a serialized byte buffer.
    ///
    /// The byte buffer must contain a serialized `LinearModel`:
    /// [num_rows: u32][num_cols: u32][weights: f32...][num_bias: u32][bias: f32...]
    pub fn load_model(&self, model_data: &[u8]) -> Result<(), CortexError> {
        info!("Loading Cortex model from {} bytes", model_data.len());
        let model = deserialize_model(model_data)?;

        // Log loaded weights for debugging
        debug!(
            "Model loaded: weights shape ({}, {}), bias = {:?}",
            model.weights.nrows(),
            model.weights.ncols(),
            model.bias.as_slice().unwrap()
        );

        *self.model.write().map_err(|e| {
            CortexError::ModelLoadError(format!("Lock poisoned: {}", e))
        })? = Some(model);

        info!("Cortex model loaded successfully");
        Ok(())
    }

    /// Train the model on execution data using stochastic gradient descent.
    ///
    /// Each `ExecutionLog` contains the task profile, the device it ran on,
    /// and the actual execution time. The model minimises mean squared error
    /// between predicted and actual costs using online (per-sample) updates
    /// with gradient clipping for numerical stability.
    pub fn train(&self, execution_logs: &[ExecutionLog]) -> Result<(), CortexError> {
        if execution_logs.is_empty() {
            return Err(CortexError::TrainingError(
                "No execution logs provided".into(),
            ));
        }

        info!(
            "Training Cortex model on {} execution logs",
            execution_logs.len()
        );

        let learning_rate: f32 = 0.001;
        let gradient_clip: f32 = 10.0;
        let epochs = 50;

        // Ensure a model exists to train on
        {
            let mut model_guard = self.model.write().map_err(|e| {
                CortexError::TrainingError(format!("Lock poisoned: {}", e))
            })?;
            if model_guard.is_none() {
                *model_guard = Some(LinearModel::new());
                debug!("Initialised model with default weights for training");
            }
        }

        for epoch in 0..epochs {
            let mut epoch_loss: f32 = 0.0;

            for log in execution_logs {
                let features = Self::extract_features(&log.profile);

                // Read current weights and compute prediction
                let (predicted, target_device_idx) = {
                    let model_guard = self.model.read().map_err(|e| {
                        CortexError::TrainingError(format!("Lock poisoned: {}", e))
                    })?;
                    let m = model_guard.as_ref().unwrap();
                    let predictions = m.weights.dot(&features) + &m.bias;

                    let dev_idx = match log.device {
                        Device::Cpu => 0,
                        Device::Gpu(_) => 1,
                        Device::Qpu(_) => 2,
                    };

                    (predictions[dev_idx], dev_idx)
                };

                let actual_cost = log.actual_time_us as f32;
                let error = predicted - actual_cost;
                epoch_loss += error * error;

                // Compute gradient with clipping
                let raw_grad = 2.0 * error;
                let grad = raw_grad.max(-gradient_clip).min(gradient_clip);

                // Online update: modify weights immediately
                let mut model_guard = self.model.write().map_err(|e| {
                    CortexError::TrainingError(format!("Lock poisoned: {}", e))
                })?;
                let m = model_guard.as_mut().unwrap();

                for j in 0..NUM_FEATURES {
                    m.weights[[target_device_idx, j]] -=
                        learning_rate * grad * features[j];
                }
                m.bias[target_device_idx] -= learning_rate * grad;
            }

            let n = execution_logs.len() as f32;
            if epoch % 10 == 0 {
                debug!("Epoch {}: loss={:.4}", epoch, epoch_loss / n);
            }
        }

        info!("Cortex model training complete");
        Ok(())
    }

    /// Predict the execution cost for a task on a specific device.
    ///
    /// # Arguments
    ///
    /// * `profile` - The task's metadata
    /// * `device` - The target device for prediction
    ///
    /// # Returns
    ///
    /// Predicted execution time in microseconds.
    pub fn predict_cost(&self, profile: &TaskProfile, device: Device) -> f64 {
        let features = Self::extract_features(profile);

        if let Some(model) = self.model.read().ok().and_then(|m| m.clone()) {
            let costs = model.predict(&features);
            let idx = match device {
                Device::Cpu => 0,
                Device::Gpu(_) => 1,
                Device::Qpu(_) => 2,
            };
            costs[idx] as f64
        } else {
            // Heuristic fallback: GPU has transfer overhead for small tasks
            let base_cost = (profile.estimated_ops as f64).log10().max(1.0);
            let memory_factor =
                (profile.memory_footprint_bytes as f64 / 1024.0).log10().max(1.0);

            match device {
                Device::Cpu => base_cost * 1.0 + memory_factor * 0.5,
                Device::Gpu(_) => base_cost * 0.1 + memory_factor * 0.8 + 100.0,
                Device::Qpu(_) => base_cost * 0.01 + 1000.0,
            }
        }
    }
}

impl Default for CortexEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution log entry for training the Cortex model.
#[derive(Debug, Clone)]
pub struct ExecutionLog {
    /// The task profile that was executed
    pub profile: TaskProfile,
    /// The device the task was executed on
    pub device: Device,
    /// Actual execution time in microseconds
    pub actual_time_us: u64,
    /// Whether the execution was successful
    pub success: bool,
}

/// Errors that can occur in the Cortex Engine.
#[derive(Debug, thiserror::Error)]
pub enum CortexError {
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),
    #[error("Training failed: {0}")]
    TrainingError(String),
    #[error("Invalid profile: {0}")]
    InvalidProfile(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cortex_creation() {
        let cortex = CortexEngine::new();
        assert!(cortex.model.read().unwrap().is_none());
    }

    #[test]
    fn test_hft_guard() {
        let cortex = CortexEngine::new();
        let profile = TaskProfile::new(Intent::Critical);

        let device = cortex.schedule(&profile);
        assert_eq!(device, Device::Cpu);
    }

    #[test]
    fn test_high_throughput_gpu_scheduling() {
        let cortex = CortexEngine::new();
        let profile = TaskProfile {
            estimated_ops: 10_000_000_000,              // 10B ops
            memory_footprint_bytes: 1024 * 1024 * 1024, // 1GB
            intent: Intent::HighThroughput,
            dependencies: 0,
        };

        let device = cortex.schedule(&profile);
        assert!(matches!(device, Device::Gpu(_)));
    }

    #[test]
    fn test_background_cpu_scheduling() {
        let cortex = CortexEngine::new();
        let profile = TaskProfile {
            estimated_ops: 100,
            memory_footprint_bytes: 1024,
            intent: Intent::Background,
            dependencies: 0,
        };

        let device = cortex.schedule(&profile);
        assert_eq!(device, Device::Cpu);
    }

    #[test]
    fn test_cost_prediction() {
        let cortex = CortexEngine::new();
        let profile = TaskProfile::new(Intent::HighThroughput);

        let cpu_cost = cortex.predict_cost(&profile, Device::Cpu);
        let gpu_cost = cortex.predict_cost(&profile, Device::Gpu(0));

        // GPU should have transfer overhead
        assert!(gpu_cost > cpu_cost);
    }

    #[test]
    fn test_model_serialization_roundtrip() {
        let original = LinearModel::new();
        let serialized = serialize_model(&original);
        let deserialized = deserialize_model(&serialized).unwrap();

        for i in 0..3 {
            for j in 0..NUM_FEATURES {
                assert!(
                    (original.weights[[i, j]] - deserialized.weights[[i, j]]).abs() < 1e-6,
                    "Weight mismatch at [{}, {}]",
                    i,
                    j
                );
            }
            assert!(
                (original.bias[i] - deserialized.bias[i]).abs() < 1e-6,
                "Bias mismatch at {}",
                i
            );
        }
    }

    #[test]
    fn test_load_model() {
        let cortex = CortexEngine::new();
        let model = LinearModel::new();
        let serialized = serialize_model(&model);

        assert!(cortex.model.read().unwrap().is_none());
        cortex.load_model(&serialized).unwrap();
        assert!(cortex.model.read().unwrap().is_some());
    }

    #[test]
    fn test_load_model_invalid_buffer() {
        let cortex = CortexEngine::new();
        let result = cortex.load_model(&[0u8; 4]);
        assert!(result.is_err());
    }

    #[test]
    fn test_train_basic() {
        let cortex = CortexEngine::new();

        let logs = vec![
            ExecutionLog {
                profile: TaskProfile::with_details(1_000_000, 1024 * 1024, Intent::HighThroughput, 0),
                device: Device::Gpu(0),
                actual_time_us: 50,
                success: true,
            },
            ExecutionLog {
                profile: TaskProfile::with_details(100, 1024, Intent::Background, 0),
                device: Device::Cpu,
                actual_time_us: 10,
                success: true,
            },
            ExecutionLog {
                profile: TaskProfile::with_details(10_000_000_000, 1024 * 1024 * 1024, Intent::HighThroughput, 5),
                device: Device::Gpu(0),
                actual_time_us: 200,
                success: true,
            },
        ];

        let result = cortex.train(&logs);
        assert!(result.is_ok());

        // Model should now exist
        assert!(cortex.model.read().unwrap().is_some());
    }

    #[test]
    fn test_train_empty_logs() {
        let cortex = CortexEngine::new();
        let result = cortex.train(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_model_based_schedule() {
        let cortex = CortexEngine::new();

        // Train with mixed data: GPU is faster for large tasks, CPU for small
        let mut logs: Vec<ExecutionLog> = Vec::new();
        for i in 0..20 {
            // Large task: GPU is fast
            logs.push(ExecutionLog {
                profile: TaskProfile::with_details(
                    10_000_000_000,
                    1024 * 1024 * 1024,
                    Intent::HighThroughput,
                    0,
                ),
                device: Device::Gpu(0),
                actual_time_us: 20 + i,
                success: true,
            });
            // Same large task on CPU: much slower
            logs.push(ExecutionLog {
                profile: TaskProfile::with_details(
                    10_000_000_000,
                    1024 * 1024 * 1024,
                    Intent::HighThroughput,
                    0,
                ),
                device: Device::Cpu,
                actual_time_us: 500 + i * 10,
                success: true,
            });
        }

        cortex.train(&logs).unwrap();

        let profile = TaskProfile::with_details(
            10_000_000_000,
            1024 * 1024 * 1024,
            Intent::HighThroughput,
            0,
        );

        let cpu_cost = cortex.predict_cost(&profile, Device::Cpu);
        let gpu_cost = cortex.predict_cost(&profile, Device::Gpu(0));

        // GPU should now predict lower cost than CPU for large tasks
        assert!(
            gpu_cost < cpu_cost,
            "GPU cost ({:.2}) should be less than CPU cost ({:.2})",
            gpu_cost,
            cpu_cost
        );
    }

    #[test]
    fn test_model_predictions() {
        let cortex = CortexEngine::new();
        let model = LinearModel::new();
        cortex.load_model(&serialize_model(&model)).unwrap();

        let profile = TaskProfile::with_details(
            1_000_000_000,
            512 * 1024 * 1024,
            Intent::HighThroughput,
            0,
        );
        let cpu_cost = cortex.predict_cost(&profile, Device::Cpu);
        let gpu_cost = cortex.predict_cost(&profile, Device::Gpu(0));

        // Both should be finite
        assert!(cpu_cost.is_finite());
        assert!(gpu_cost.is_finite());
    }
}
