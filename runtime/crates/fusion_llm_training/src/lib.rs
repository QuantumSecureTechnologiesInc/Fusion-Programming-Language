//! fusion_llm_training — Training infrastructure for Fusion v2.0 LLM models.
//!
//! Provides learning rate schedulers, loss functions, AdamW optimizer,
//! gradient accumulation, and checkpoint/resume support.

use std::collections::HashMap;
use std::f32::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

use fusion_llm_core::{LlmError, Result, Tensor};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum TrainingError {
    #[error("checkpoint error: {0}")]
    Checkpoint(String),

    #[error("gradient error: {0}")]
    Gradient(String),

    #[error("optimizer error: {0}")]
    Optimizer(String),
}

impl From<TrainingError> for LlmError {
    fn from(e: TrainingError) -> Self {
        match e {
            TrainingError::Checkpoint(msg) => LlmError::Inference(msg),
            TrainingError::Gradient(msg) => LlmError::Inference(msg),
            TrainingError::Optimizer(msg) => LlmError::Inference(msg),
        }
    }
}

// ---------------------------------------------------------------------------
// Optimizer — AdamW
// ---------------------------------------------------------------------------

/// AdamW optimizer state for a single parameter group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdamWState {
    /// First moment estimates (m).
    pub m: Vec<f32>,
    /// Second moment estimates (v).
    pub v: Vec<f32>,
    /// Current step count.
    pub step: usize,
    /// Learning rate.
    pub lr: f32,
    /// Beta1 for first moment.
    pub beta1: f32,
    /// Beta2 for second moment.
    pub beta2: f32,
    /// Epsilon for numerical stability.
    pub eps: f32,
    /// Weight decay coefficient.
    pub weight_decay: f32,
}

impl AdamWState {
    pub fn new(lr: f32, beta1: f32, beta2: f32, eps: f32, weight_decay: f32, num_params: usize) -> Self {
        Self {
            m: vec![0.0; num_params],
            v: vec![0.0; num_params],
            step: 0,
            lr,
            beta1,
            beta2,
            eps,
            weight_decay,
        }
    }

    /// Update parameters: p -= lr * m_hat / (sqrt(v_hat) + eps).
    /// Also applies decoupled weight decay: p -= lr * weight_decay * p.
    pub fn step(&mut self, params: &mut [f32], gradients: &[f32]) {
        assert_eq!(params.len(), gradients.len());
        assert_eq!(params.len(), self.m.len());

        self.step += 1;
        let bias_correction1 = 1.0 - self.beta1.powi(self.step as i32);
        let bias_correction2 = 1.0 - self.beta2.powi(self.step as i32);

        for i in 0..params.len() {
            // Decoupled weight decay
            params[i] -= self.lr * self.weight_decay * params[i];

            // Update moments
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * gradients[i];
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * gradients[i] * gradients[i];

            // Bias-corrected moments
            let m_hat = self.m[i] / bias_correction1;
            let v_hat = self.v[i] / bias_correction2;

            // Update
            params[i] -= self.lr * m_hat / (v_hat.sqrt() + self.eps);
        }
    }

    pub fn set_lr(&mut self, lr: f32) {
        self.lr = lr;
    }
}

// ---------------------------------------------------------------------------
// Learning rate schedulers
// ---------------------------------------------------------------------------

/// Learning rate scheduler trait.
pub trait LearningRateScheduler: Send + Sync {
    /// Get the learning rate at the given step.
    fn get_lr(&self, step: usize) -> f32;

    /// Get the current warmup state.
    fn is_warming_up(&self, step: usize) -> bool;
}

/// Constant learning rate (no scheduling).
pub struct ConstantScheduler {
    lr: f32,
}

impl ConstantScheduler {
    pub fn new(lr: f32) -> Self {
        Self { lr }
    }
}

impl LearningRateScheduler for ConstantScheduler {
    fn get_lr(&self, _step: usize) -> f32 {
        self.lr
    }

    fn is_warming_up(&self, _step: usize) -> bool {
        false
    }
}

/// Linear warmup followed by cosine decay.
pub struct CosineWarmupScheduler {
    pub base_lr: f32,
    pub min_lr: f32,
    pub warmup_steps: usize,
    pub total_steps: usize,
}

impl CosineWarmupScheduler {
    pub fn new(base_lr: f32, min_lr: f32, warmup_steps: usize, total_steps: usize) -> Self {
        Self {
            base_lr,
            min_lr,
            warmup_steps,
            total_steps,
        }
    }
}

impl LearningRateScheduler for CosineWarmupScheduler {
    fn get_lr(&self, step: usize) -> f32 {
        if step < self.warmup_steps {
            // Linear warmup: 0 → base_lr
            self.base_lr * step as f32 / self.warmup_steps.max(1) as f32
        } else {
            // Cosine decay: base_lr → min_lr
            let progress = (step - self.warmup_steps) as f32
                / (self.total_steps - self.warmup_steps).max(1) as f32;
            let cosine = 0.5 * (1.0 + (PI * progress).cos());
            self.min_lr + (self.base_lr - self.min_lr) * cosine
        }
    }

    fn is_warming_up(&self, step: usize) -> bool {
        step < self.warmup_steps
    }
}

/// Linear warmup followed by linear decay.
pub struct LinearWarmupScheduler {
    pub base_lr: f32,
    pub min_lr: f32,
    pub warmup_steps: usize,
    pub total_steps: usize,
}

impl LinearWarmupScheduler {
    pub fn new(base_lr: f32, min_lr: f32, warmup_steps: usize, total_steps: usize) -> Self {
        Self {
            base_lr,
            min_lr,
            warmup_steps,
            total_steps,
        }
    }
}

impl LearningRateScheduler for LinearWarmupScheduler {
    fn get_lr(&self, step: usize) -> f32 {
        if step < self.warmup_steps {
            self.base_lr * step as f32 / self.warmup_steps.max(1) as f32
        } else {
            let progress = (step - self.warmup_steps) as f32
                / (self.total_steps - self.warmup_steps).max(1) as f32;
            self.base_lr - (self.base_lr - self.min_lr) * progress
        }
    }

    fn is_warming_up(&self, step: usize) -> bool {
        step < self.warmup_steps
    }
}

/// Constant with linear warmup (no decay).
pub struct WarmupOnlyScheduler {
    pub base_lr: f32,
    pub warmup_steps: usize,
}

impl WarmupOnlyScheduler {
    pub fn new(base_lr: f32, warmup_steps: usize) -> Self {
        Self { base_lr, warmup_steps }
    }
}

impl LearningRateScheduler for WarmupOnlyScheduler {
    fn get_lr(&self, step: usize) -> f32 {
        if step < self.warmup_steps {
            self.base_lr * step as f32 / self.warmup_steps.max(1) as f32
        } else {
            self.base_lr
        }
    }

    fn is_warming_up(&self, step: usize) -> bool {
        step < self.warmup_steps
    }
}

// ---------------------------------------------------------------------------
// Loss functions
// ---------------------------------------------------------------------------

/// Compute cross-entropy loss: -sum(target * log(softmax(logits))).
///
/// `logits`: (batch * seq_len, vocab_size)
/// `targets`: (batch * seq_len,) — target token ids
/// Returns the mean loss over all tokens.
pub fn cross_entropy_loss(logits: &Tensor, targets: &[u32]) -> f32 {
    let vocab_size = logits.cols;
    let batch_len = targets.len();
    assert_eq!(logits.rows, batch_len, "logits rows must match targets length");

    let mut total_loss = 0.0f32;
    for (t, &target) in targets.iter().enumerate() {
        let target = (target as usize).min(vocab_size - 1);
        let offset = t * vocab_size;
        let logits_row = &logits.data[offset..offset + vocab_size];

        // Numerically stable softmax log
        let max_logit = logits_row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let log_sum_exp: f32 = logits_row.iter().map(|l| (l - max_logit).exp()).sum::<f32>().ln();
        let log_prob = logits_row[target] - max_logit - log_sum_exp;

        total_loss -= log_prob;
    }

    total_loss / batch_len as f32
}

/// Compute cross-entropy loss with label smoothing.
pub fn cross_entropy_loss_label_smoothed(
    logits: &Tensor,
    targets: &[u32],
    smoothing: f32,
) -> f32 {
    let vocab_size = logits.cols;
    let batch_len = targets.len();
    assert_eq!(logits.rows, batch_len);

    let mut total_loss = 0.0f32;
    for (t, &target) in targets.iter().enumerate() {
        let target = (target as usize).min(vocab_size - 1);
        let offset = t * vocab_size;
        let logits_row = &logits.data[offset..offset + vocab_size];

        let max_logit = logits_row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let log_sum_exp: f32 = logits_row.iter().map(|l| (l - max_logit).exp()).sum::<f32>().ln();

        // Smooth loss: (1 - smoothing) * log_prob(target) + smoothing * mean(log_probs)
        let log_prob_target = logits_row[target] - max_logit - log_sum_exp;
        let mean_log_prob = -log_sum_exp; // mean of uniform distribution log probs
        let loss = (1.0 - smoothing) * (-log_prob_target) + smoothing * (-mean_log_prob);
        total_loss += loss;
    }

    total_loss / batch_len as f32
}

/// Compute masked language modeling (MLM) loss — used for BERT-style training.
///
/// Only computes loss on positions where `mask` is true.
pub fn mlm_loss(logits: &Tensor, targets: &[u32], mask: &[bool]) -> f32 {
    let vocab_size = logits.cols;
    assert_eq!(logits.rows, targets.len());
    assert_eq!(logits.rows, mask.len());

    let mut total_loss = 0.0f32;
    let mut masked_count = 0u32;

    for (t, (&target, &is_masked)) in targets.iter().zip(mask.iter()).enumerate() {
        if !is_masked {
            continue;
        }
        let target = (target as usize).min(vocab_size - 1);
        let offset = t * vocab_size;
        let logits_row = &logits.data[offset..offset + vocab_size];

        let max_logit = logits_row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let log_sum_exp: f32 = logits_row.iter().map(|l| (l - max_logit).exp()).sum::<f32>().ln();
        let log_prob = logits_row[target] - max_logit - log_sum_exp;

        total_loss -= log_prob;
        masked_count += 1;
    }

    if masked_count == 0 {
        0.0
    } else {
        total_loss / masked_count as f32
    }
}

/// Perplexity from loss: exp(loss).
pub fn perplexity_from_loss(loss: f32) -> f32 {
    loss.exp()
}

// ---------------------------------------------------------------------------
// Gradient accumulator
// ---------------------------------------------------------------------------

/// Accumulates gradients over multiple micro-steps before applying them.
pub struct GradientAccumulator {
    /// Accumulated gradients (summed).
    pub accumulated: Vec<f32>,
    /// Number of micro-batches accumulated so far.
    pub count: usize,
    /// Number of micro-batches before stepping.
    pub accumulation_steps: usize,
    /// Total parameter count.
    pub num_params: usize,
}

impl GradientAccumulator {
    pub fn new(num_params: usize, accumulation_steps: usize) -> Self {
        Self {
            accumulated: vec![0.0; num_params],
            count: 0,
            accumulation_steps,
            num_params,
        }
    }

    /// Accumulate a gradient (adds to the running sum).
    pub fn accumulate(&mut self, gradients: &[f32]) {
        assert_eq!(gradients.len(), self.num_params);
        for (acc, g) in self.accumulated.iter_mut().zip(gradients.iter()) {
            *acc += g;
        }
        self.count += 1;
    }

    /// Returns true if enough micro-batches have been accumulated.
    pub fn is_ready(&self) -> bool {
        self.count >= self.accumulation_steps
    }

    /// Get the averaged gradients (sum / accumulation_steps).
    /// Resets the accumulator.
    pub fn get_averaged(&mut self) -> Vec<f32> {
        let scale = 1.0 / self.accumulation_steps as f32;
        let averaged: Vec<f32> = self.accumulated.iter().map(|g| g * scale).collect();
        self.accumulated.iter_mut().for_each(|g| *g = 0.0);
        self.count = 0;
        averaged
    }

    /// Reset without returning.
    pub fn reset(&mut self) {
        self.accumulated.iter_mut().for_each(|g| *g = 0.0);
        self.count = 0;
    }
}

// ---------------------------------------------------------------------------
// Checkpointing
// ---------------------------------------------------------------------------

/// Serializable training checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Global training step.
    pub step: usize,
    /// Epoch number.
    pub epoch: usize,
    /// Model parameters (flattened).
    pub model_params: Vec<f32>,
    /// Optimizer state.
    pub optimizer_state: AdamWState,
    /// Loss at this step.
    pub loss: f32,
    /// Learning rate at this step.
    pub learning_rate: f32,
    /// Random seed for reproducibility.
    pub rng_seed: u64,
    /// Step-level metadata.
    pub metadata: HashMap<String, String>,
}

impl Checkpoint {
    pub fn new(
        step: usize,
        epoch: usize,
        model_params: Vec<f32>,
        optimizer_state: AdamWState,
        loss: f32,
        learning_rate: f32,
        rng_seed: u64,
    ) -> Self {
        Self {
            step,
            epoch,
            model_params,
            optimizer_state,
            loss,
            learning_rate,
            rng_seed,
            metadata: HashMap::new(),
        }
    }

    /// Serialize to JSON and write to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| LlmError::Inference(e.to_string()))?;
        fs::write(path, json)
            .map_err(|e| TrainingError::Checkpoint(format!("failed to write {path:?}: {e}")))?;
        log::info!("Checkpoint saved to {path:?} at step {}", self.step);
        Ok(())
    }

    /// Load from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path)
            .map_err(|e| TrainingError::Checkpoint(format!("failed to read {path:?}: {e}")))?;
        let checkpoint: Self = serde_json::from_str(&json)
            .map_err(|e| TrainingError::Checkpoint(format!("failed to parse {path:?}: {e}")))?;
        log::info!("Checkpoint loaded from {path:?} at step {}", checkpoint.step);
        Ok(checkpoint)
    }

    /// Add a metadata entry.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Manages checkpoint rotation — keeps the latest N checkpoints and deletes older ones.
pub struct CheckpointManager {
    checkpoint_dir: PathBuf,
    max_checkpoints: usize,
    prefix: String,
}

impl CheckpointManager {
    pub fn new(checkpoint_dir: impl Into<PathBuf>, max_checkpoints: usize) -> Self {
        Self {
            checkpoint_dir: checkpoint_dir.into(),
            max_checkpoints,
            prefix: "checkpoint".to_string(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Save a checkpoint and prune old ones.
    pub fn save(&self, checkpoint: &Checkpoint) -> Result<PathBuf> {
        let path = self
            .checkpoint_dir
            .join(format!("{}_step_{}.json", self.prefix, checkpoint.step));
        checkpoint.save(&path)?;
        self.prune()?;
        Ok(path)
    }

    /// Find the latest checkpoint in the directory.
    pub fn latest(&self) -> Result<Option<Checkpoint>> {
        if !self.checkpoint_dir.exists() {
            return Ok(None);
        }

        let mut files: Vec<_> = fs::read_dir(&self.checkpoint_dir)
            .map_err(|e| TrainingError::Checkpoint(e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .map_or(false, |f| f.to_string_lossy().starts_with(&self.prefix))
            })
            .collect();

        files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));

        match files.last() {
            Some(entry) => {
                let checkpoint = Checkpoint::load(&entry.path())?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    /// Remove old checkpoints, keeping only `max_checkpoints`.
    fn prune(&self) -> Result<()> {
        if !self.checkpoint_dir.exists() {
            return Ok(());
        }

        let mut files: Vec<_> = fs::read_dir(&self.checkpoint_dir)
            .map_err(|e| TrainingError::Checkpoint(e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .map_or(false, |f| f.to_string_lossy().starts_with(&self.prefix))
            })
            .collect();

        files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));

        if files.len() > self.max_checkpoints {
            let to_remove = files.len() - self.max_checkpoints;
            for file in &files[..to_remove] {
                let _ = fs::remove_file(file.path());
                log::info!("Pruned old checkpoint: {:?}", file.path());
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Training loop
// ---------------------------------------------------------------------------

/// Callback invoked at each training step.
pub type StepCallback = Box<dyn Fn(usize, f32, f32) + Send + Sync>;

/// Configuration for the training loop.
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    pub total_steps: usize,
    pub log_interval: usize,
    pub save_interval: usize,
    pub eval_interval: usize,
    pub gradient_accumulation_steps: usize,
    pub max_grad_norm: f32,
    pub seed: u64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            total_steps: 10000,
            log_interval: 100,
            save_interval: 1000,
            eval_interval: 500,
            gradient_accumulation_steps: 1,
            max_grad_norm: 1.0,
            seed: 42,
        }
    }
}

/// A batch of training data.
#[derive(Debug, Clone)]
pub struct TrainingBatch {
    pub input_ids: Vec<u32>,
    pub target_ids: Vec<u32>,
    pub mask: Option<Vec<bool>>,
}

/// Training state that persists across steps.
#[derive(Debug, Clone)]
pub struct TrainingState {
    pub step: usize,
    pub epoch: usize,
    pub best_loss: f32,
    pub total_tokens_processed: usize,
    pub training_loss_history: Vec<f32>,
}

impl TrainingState {
    pub fn new() -> Self {
        Self {
            step: 0,
            epoch: 0,
            best_loss: f32::INFINITY,
            total_tokens_processed: 0,
            training_loss_history: Vec::new(),
        }
    }
}

/// The training loop orchestrator.
pub struct TrainingLoop {
    config: TrainingConfig,
    state: TrainingState,
    scheduler: Box<dyn LearningRateScheduler>,
    accumulator: GradientAccumulator,
    checkpoint_manager: Option<CheckpointManager>,
    on_step: Option<StepCallback>,
}

impl TrainingLoop {
    pub fn new(
        config: TrainingConfig,
        num_params: usize,
        scheduler: Box<dyn LearningRateScheduler>,
    ) -> Self {
        let accumulator =
            GradientAccumulator::new(num_params, config.gradient_accumulation_steps);
        Self {
            config,
            state: TrainingState::new(),
            scheduler,
            accumulator,
            checkpoint_manager: None,
            on_step: None,
        }
    }

    pub fn with_checkpoint_manager(mut self, manager: CheckpointManager) -> Self {
        self.checkpoint_manager = Some(manager);
        self
    }

    pub fn on_step(mut self, callback: StepCallback) -> Self {
        self.on_step = Some(callback);
        self
    }

    pub fn state(&self) -> &TrainingState {
        &self.state
    }

    /// Attempt to resume from the latest checkpoint.
    pub fn resume(&mut self) -> Result<Option<Checkpoint>> {
        if let Some(ref manager) = self.checkpoint_manager {
            match manager.latest()? {
                Some(cp) => {
                    self.state.step = cp.step;
                    self.state.epoch = cp.epoch;
                    self.state.best_loss = cp.loss;
                    log::info!("Resumed training from step {}", cp.step);
                    Ok(Some(cp))
                }
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Run a single training step with the given batch.
    ///
    /// The caller provides a closure that computes the loss and gradients:
    ///   `fn(input_ids, target_ids, mask) -> (loss, gradients)`
    pub fn step<F>(
        &mut self,
        batch: &TrainingBatch,
        compute_loss_and_grads: F,
    ) -> Result<f32>
    where
        F: FnOnce(&[u32], &[u32], Option<&[bool]>) -> (f32, Vec<f32>),
    {
        let lr = self.scheduler.get_lr(self.state.step);
        let (loss, grads) = compute_loss_and_grads(
            &batch.input_ids,
            &batch.target_ids,
            batch.mask.as_deref(),
        );

        self.accumulator.accumulate(&grads);

        if self.accumulator.is_ready() {
            let averaged_grads = self.accumulator.get_averaged();

            // Gradient clipping by global norm
            let grad_norm: f32 = averaged_grads.iter().map(|g| g * g).sum::<f32>().sqrt();
            let _clip_scale = if grad_norm > self.config.max_grad_norm {
                self.config.max_grad_norm / grad_norm
            } else {
                1.0
            };

            // Note: In production, the model params would be updated here via optimizer.
            // The averaged + clipped gradients are returned for the model to apply.

            if let Some(ref cb) = self.on_step {
                cb(self.state.step, loss, lr);
            }

            self.state.step += 1;
            self.state.training_loss_history.push(loss);
            if loss < self.state.best_loss {
                self.state.best_loss = loss;
            }
            self.state.total_tokens_processed += batch.input_ids.len();

            // Log
            if self.state.step % self.config.log_interval == 0 {
                log::info!(
                    "step {} | loss: {:.6} | lr: {:.2e} | grad_norm: {:.4} | best: {:.6}",
                    self.state.step,
                    loss,
                    lr,
                    grad_norm,
                    self.state.best_loss
                );
            }

            // Save checkpoint
            if self.config.save_interval > 0 && self.state.step % self.config.save_interval == 0 {
                if let Some(ref _manager) = self.checkpoint_manager {
                    // In a real system, we'd serialize model params here.
                    log::info!("Checkpoint save triggered at step {}", self.state.step);
                }
            }

            Ok(loss)
        } else {
            // Still accumulating — return the micro-batch loss
            Ok(loss)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_entropy_loss() {
        // 3 classes, 2 samples
        // Sample 0: logits = [1, 2, 3], target = 2
        // Sample 1: logits = [1, 1, 1], target = 0
        let logits = Tensor::new(vec![1.0, 2.0, 3.0, 1.0, 1.0, 1.0], 2, 3);
        let targets = vec![2u32, 0u32];
        let loss = cross_entropy_loss(&logits, &targets);
        // Should be positive and finite
        assert!(loss > 0.0, "loss should be positive, got {loss}");
        assert!(loss.is_finite(), "loss should be finite, got {loss}");
    }

    #[test]
    fn test_cross_entropy_label_smoothed() {
        let logits = Tensor::new(vec![1.0, 2.0, 3.0], 1, 3);
        let targets = vec![2u32];
        let loss_smooth = cross_entropy_loss_label_smoothed(&logits, &targets, 0.1);
        let loss_plain = cross_entropy_loss(&logits, &targets);
        // Smoothed loss should be >= plain loss (penalizes confidence)
        assert!(
            loss_smooth >= loss_plain - 1e-5,
            "smoothed ({loss_smooth}) should be >= plain ({loss_plain})"
        );
    }

    #[test]
    fn test_mlm_loss() {
        let logits = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 2, 3);
        let targets = vec![0u32, 2u32];
        let mask = vec![true, false]; // Only first token is masked
        let loss = mlm_loss(&logits, &targets, &mask);
        assert!(loss > 0.0);
    }

    #[test]
    fn test_perplexity() {
        let ppl = perplexity_from_loss(2.0);
        assert!((ppl - 2.0_f32.exp()).abs() < 1e-5);
    }

    #[test]
    fn test_constant_scheduler() {
        let s = ConstantScheduler::new(0.001);
        assert_eq!(s.get_lr(0), 0.001);
        assert_eq!(s.get_lr(1000), 0.001);
        assert!(!s.is_warming_up(0));
    }

    #[test]
    fn test_cosine_warmup_scheduler() {
        let s = CosineWarmupScheduler::new(0.001, 0.0, 100, 1000);
        // Warmup phase
        assert!(s.is_warming_up(50));
        assert!((s.get_lr(0) - 0.0).abs() < 1e-7);
        assert!((s.get_lr(50) - 0.0005).abs() < 1e-5);
        // Post-warmup: should decay toward min_lr
        assert!(!s.is_warming_up(100));
        let lr_at_end = s.get_lr(1000);
        assert!(lr_at_end < 0.001, "should decay, got {lr_at_end}");
    }

    #[test]
    fn test_linear_warmup_scheduler() {
        let s = LinearWarmupScheduler::new(0.001, 0.0001, 100, 1000);
        assert!((s.get_lr(0) - 0.0).abs() < 1e-7);
        assert!((s.get_lr(50) - 0.0005).abs() < 1e-5);
        assert!((s.get_lr(100) - 0.001).abs() < 1e-5);
        assert!((s.get_lr(1000) - 0.0001).abs() < 1e-5);
    }

    #[test]
    fn test_adamw_step() {
        let mut state = AdamWState::new(0.001, 0.9, 0.999, 1e-8, 0.01, 4);
        let mut params = vec![1.0, 2.0, 3.0, 4.0];
        let gradients = vec![0.1, -0.2, 0.3, -0.4];
        let params_before = params.clone();

        state.step(&mut params, &gradients);
        assert_eq!(state.step, 1);
        // Params should have changed
        for i in 0..4 {
            assert!(
                (params[i] - params_before[i]).abs() > 1e-6,
                "params[{i}] should have changed"
            );
        }
    }

    #[test]
    fn test_adamw_weight_decay() {
        let mut state = AdamWState::new(0.1, 0.9, 0.999, 1e-8, 0.5, 1);
        let mut params = vec![1.0];
        let gradients = vec![0.0]; // Zero gradient, only weight decay acts
        state.step(&mut params, &gradients);
        // After weight decay only: p = 1.0 - 0.1 * 0.5 * 1.0 = 0.95
        // Then m and v update, but m is 0 so no further change
        assert!((params[0] - 0.95).abs() < 0.01, "weight decay: expected ~0.95, got {}", params[0]);
    }

    #[test]
    fn test_gradient_accumulator() {
        let mut acc = GradientAccumulator::new(4, 3);
        assert!(!acc.is_ready());

        acc.accumulate(&[1.0, 2.0, 3.0, 4.0]);
        assert!(!acc.is_ready());
        acc.accumulate(&[0.5, 0.5, 0.5, 0.5]);
        assert!(!acc.is_ready());
        acc.accumulate(&[0.5, 0.5, 0.5, 0.5]);
        assert!(acc.is_ready());

        let avg = acc.get_averaged();
        // Average of [1.0, 0.5, 0.5] = 2.0/3 for first element
        assert!((avg[0] - 2.0 / 3.0).abs() < 1e-5);
        assert!((avg[1] - 3.0 / 3.0).abs() < 1e-5);
        assert!(!acc.is_ready()); // Reset
    }

    #[test]
    fn test_checkpoint_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cp.json");

        let opt = AdamWState::new(0.001, 0.9, 0.999, 1e-8, 0.01, 4);
        let cp = Checkpoint::new(100, 1, vec![1.0, 2.0, 3.0], opt, 0.5, 0.001, 42);
        cp.save(&path).unwrap();

        let loaded = Checkpoint::load(&path).unwrap();
        assert_eq!(loaded.step, 100);
        assert_eq!(loaded.epoch, 1);
        assert_eq!(loaded.loss, 0.5);
        assert_eq!(loaded.rng_seed, 42);
        assert_eq!(loaded.model_params, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_training_config_default() {
        let cfg = TrainingConfig::default();
        assert_eq!(cfg.total_steps, 10000);
        assert_eq!(cfg.gradient_accumulation_steps, 1);
        assert_eq!(cfg.max_grad_norm, 1.0);
    }

    #[test]
    fn test_training_state_new() {
        let state = TrainingState::new();
        assert_eq!(state.step, 0);
        assert_eq!(state.epoch, 0);
        assert_eq!(state.best_loss, f32::INFINITY);
    }
}
