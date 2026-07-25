//! fusion_llm_core — Core LLM abstractions for the Fusion v2.0 Vortex runtime.
//!
//! Provides the foundational traits and types for model definition, tokenization,
//! KV-cache management, and inference execution.

use std::collections::HashMap;
use std::fmt;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("tokenization error: {0}")]
    Tokenization(String),

    #[error("model error: {0}")]
    Model(String),

    #[error("inference error: {0}")]
    Inference(String),

    #[error("format error: {0}")]
    Format(String),

    #[error("shape mismatch: expected {expected}, got {actual}")]
    ShapeMismatch { expected: String, actual: String },

    #[error("weight missing: {0}")]
    WeightMissing(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, LlmError>;

// ---------------------------------------------------------------------------
// Token representation
// ---------------------------------------------------------------------------

/// A single token with its id and optional byte-level representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub id: u32,
    pub text: String,
    pub score: Option<f32>,
}

impl Token {
    pub fn new(id: u32, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
            score: None,
        }
    }
}

/// A sequence of tokens.
pub type TokenSequence = Vec<Token>;

/// Raw token ids (the numeric representation used by models).
pub type TokenIds = Vec<u32>;

// ---------------------------------------------------------------------------
// Tensor — minimal rank-2 f32 tensor used throughout the crate
// ---------------------------------------------------------------------------

/// A simple row-major 2-D tensor backed by `Vec<f32>`.
/// Used as the lingua franca between tokenizer output, model internals,
/// and inference output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
}

impl Tensor {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![0.0; rows * cols],
            rows,
            cols,
        }
    }

    pub fn new(data: Vec<f32>, rows: usize, cols: usize) -> Self {
        assert_eq!(data.len(), rows * cols, "data length must equal rows * cols");
        Self { data, rows, cols }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data
    }

    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.data[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, val: f32) {
        self.data[row * self.cols + col] = val;
    }

    pub fn numel(&self) -> usize {
        self.data.len()
    }

    /// Softmax over the last dimension (each row independently).
    pub fn softmax(&self) -> Self {
        let mut out = Vec::with_capacity(self.data.len());
        for r in 0..self.rows {
            let offset = r * self.cols;
            let row = &self.data[offset..offset + self.cols];
            let max_val = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let exp_sum: f32 = row.iter().map(|v| (v - max_val).exp()).sum();
            for &v in row {
                out.push((v - max_val).exp() / exp_sum);
            }
        }
        Self::new(out, self.rows, self.cols)
    }

    /// Matrix multiply: self (M×K) × other (K×N) → result (M×N).
    pub fn matmul(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.cols, other.rows, "matmul dimension mismatch");
        let m = self.rows;
        let k = self.cols;
        let n = other.cols;
        let mut result = vec![0.0f32; m * n];
        for i in 0..m {
            for p in 0..k {
                let a_val = self.data[i * k + p];
                for j in 0..n {
                    result[i * n + j] += a_val * other.data[p * n + j];
                }
            }
        }
        Tensor::new(result, m, n)
    }

    /// Element-wise addition.
    pub fn add(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.data.len(), other.data.len());
        Tensor::new(
            self.data.iter().zip(other.data.iter()).map(|(a, b)| a + b).collect(),
            self.rows,
            self.cols,
        )
    }
}

impl fmt::Display for Tensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tensor({}×{})", self.rows, self.cols)
    }
}

// ---------------------------------------------------------------------------
// Tokenizer trait + BPE implementation
// ---------------------------------------------------------------------------

/// BPE merge rule — merges a pair of token ids into a single new id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpeMerge {
    pub left: u32,
    pub right: u32,
    pub merged: u32,
}

/// BPE tokenizer vocabulary and merge table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpeVocab {
    /// Maps token id → string representation.
    pub token_to_str: HashMap<u32, String>,
    /// Maps string → token id.
    pub str_to_token: HashMap<String, u32>,
    /// Ordered merge rules (applied greedily by priority).
    pub merges: Vec<BpeMerge>,
    /// The special-token offset: ids >= this value are special tokens.
    pub special_offset: u32,
}

impl BpeVocab {
    pub fn new() -> Self {
        Self {
            token_to_str: HashMap::new(),
            str_to_token: HashMap::new(),
            merges: Vec::new(),
            special_offset: 256,
        }
    }

    pub fn with_byte_fallback() -> Self {
        let mut vocab = Self::new();
        // Single-byte tokens: 0x00..0xFF
        for i in 0..256u32 {
            let s = format!("<0x{:02X}>", i);
            vocab.token_to_str.insert(i, s.clone());
            vocab.str_to_token.insert(s, i);
        }
        vocab
    }

    pub fn add_token(&mut self, token: &str) -> u32 {
        if let Some(&id) = self.str_to_token.get(token) {
            return id;
        }
        let id = self.token_to_str.len() as u32;
        self.token_to_str.insert(id, token.to_string());
        self.str_to_token.insert(token.to_string(), id);
        id
    }

    pub fn token_str(&self, id: u32) -> Option<&str> {
        self.token_to_str.get(&id).map(|s| s.as_str())
    }
}

/// Trait for any tokenizer that can encode text to token ids and decode back.
pub trait Tokenizer: Send + Sync {
    /// Encode a string into token ids.
    fn encode(&self, text: &str) -> Result<TokenIds>;

    /// Encode with scores (for logit analysis).
    fn encode_with_scores(&self, text: &str) -> Result<TokenSequence>;

    /// Decode token ids back to text.
    fn decode(&self, ids: &[u32]) -> Result<String>;

    /// Vocab size.
    fn vocab_size(&self) -> usize;

    /// Tokenize into byte-level BPE pieces (internal).
    fn tokenize_bytes(&self, text: &str) -> Vec<String>;
}

/// A BPE-based tokenizer operating on byte-level pieces.
pub struct BpeTokenizer {
    vocab: BpeVocab,
    /// Pre-computed merge priority: (left, right) → merge rank (lower = higher priority).
    merge_rank: HashMap<(u32, u32), usize>,
}

impl BpeTokenizer {
    pub fn new(vocab: BpeVocab) -> Self {
        let merge_rank: HashMap<(u32, u32), usize> = vocab
            .merges
            .iter()
            .enumerate()
            .map(|(i, m)| ((m.left, m.right), i))
            .collect();
        Self { vocab, merge_rank }
    }

    pub fn vocab(&self) -> &BpeVocab {
        &self.vocab
    }

    /// Apply BPE merges to a sequence of byte tokens.
    fn apply_merges(&self, mut ids: Vec<u32>) -> Vec<u32> {
        loop {
            // Find the pair with lowest merge rank
            let mut best_rank = usize::MAX;
            let mut best_pos = 0;
            for i in 0..ids.len().saturating_sub(1) {
                if let Some(&rank) = self.merge_rank.get(&(ids[i], ids[i + 1])) {
                    if rank < best_rank {
                        best_rank = rank;
                        best_pos = i;
                    }
                }
            }
            if best_rank == usize::MAX {
                break;
            }
            // Merge at best_pos
            let merged_id = self.vocab.merges[best_pos].merged;
            ids[best_pos] = merged_id;
            ids.remove(best_pos + 1);
        }
        ids
    }
}

impl Tokenizer for BpeTokenizer {
    fn encode(&self, text: &str) -> Result<TokenIds> {
        let pieces = self.tokenize_bytes(text);
        let ids: Vec<u32> = pieces
            .iter()
            .map(|p| {
                self.vocab
                    .str_to_token
                    .get(p)
                    .copied()
                    .unwrap_or(0) // unknown → fallback to id 0
            })
            .collect();
        Ok(self.apply_merges(ids))
    }

    fn encode_with_scores(&self, text: &str) -> Result<TokenSequence> {
        let ids = self.encode(text)?;
        Ok(ids
            .into_iter()
            .map(|id| Token {
                id,
                text: self.vocab.token_str(id).unwrap_or("<unk>").to_string(),
                score: None,
            })
            .collect())
    }

    fn decode(&self, ids: &[u32]) -> Result<String> {
        let mut out = String::new();
        for &id in ids {
            match self.vocab.token_str(id) {
                Some(s) => out.push_str(s),
                None => return Err(LlmError::Tokenization(format!("unknown token id {id}"))),
            }
        }
        Ok(out)
    }

    fn vocab_size(&self) -> usize {
        self.vocab.token_to_str.len()
    }

    fn tokenize_bytes(&self, text: &str) -> Vec<String> {
        // Byte-level tokenization: each byte → <0xXX> piece
        text.bytes()
            .map(|b| format!("<0x{:02X}>", b))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// KV-Cache for autoregressive inference
// ---------------------------------------------------------------------------

/// Key-value cache entry for a single attention layer.
#[derive(Debug, Clone)]
pub struct KvCacheEntry {
    /// Cached key vectors: (num_heads, seq_len, head_dim).
    pub keys: Vec<f32>,
    /// Cached value vectors: (num_heads, seq_len, head_dim).
    pub values: Vec<f32>,
    /// Current sequence length stored in cache.
    pub seq_len: usize,
    /// Head dimension.
    pub head_dim: usize,
    /// Number of attention heads.
    pub num_heads: usize,
}

impl KvCacheEntry {
    pub fn new(num_heads: usize, head_dim: usize, max_seq_len: usize) -> Self {
        Self {
            keys: vec![0.0; num_heads * max_seq_len * head_dim],
            values: vec![0.0; num_heads * max_seq_len * head_dim],
            seq_len: 0,
            head_dim,
            num_heads,
        }
    }

    /// Append new key/value tensors for a single position.
    pub fn append(&mut self, keys: &[f32], values: &[f32]) {
        let offset = self.seq_len * self.head_dim;
        for h in 0..self.num_heads {
            let k_base = h * (self.keys.len() / self.num_heads) + offset;
            let v_base = h * (self.values.len() / self.num_heads) + offset;
            let k_in = h * self.head_dim;
            let v_in = h * self.head_dim;
            for d in 0..self.head_dim {
                self.keys[k_base + d] = keys[k_in + d];
                self.values[v_base + d] = values[v_in + d];
            }
        }
        self.seq_len += 1;
    }

    /// Get cached keys for a specific head.
    pub fn keys_for_head(&self, head: usize) -> &[f32] {
        let layer_size = self.keys.len() / self.num_heads;
        let start = head * layer_size;
        &self.keys[start..start + self.seq_len * self.head_dim]
    }

    /// Get cached values for a specific head.
    pub fn values_for_head(&self, head: usize) -> &[f32] {
        let layer_size = self.values.len() / self.num_heads;
        let start = head * layer_size;
        &self.values[start..start + self.seq_len * self.head_dim]
    }

    pub fn clear(&mut self) {
        self.keys.iter_mut().for_each(|v| *v = 0.0);
        self.values.iter_mut().for_each(|v| *v = 0.0);
        self.seq_len = 0;
    }
}

/// Multi-layer KV-cache managed with interior mutability.
pub struct KvCache {
    layers: Vec<RwLock<KvCacheEntry>>,
}

impl KvCache {
    pub fn new(num_layers: usize, num_heads: usize, head_dim: usize, max_seq_len: usize) -> Self {
        Self {
            layers: (0..num_layers)
                .map(|_| RwLock::new(KvCacheEntry::new(num_heads, head_dim, max_seq_len)))
                .collect(),
        }
    }

    pub fn num_layers(&self) -> usize {
        self.layers.len()
    }

    pub fn get_layer(&self, layer_idx: usize) -> &RwLock<KvCacheEntry> {
        &self.layers[layer_idx]
    }

    pub fn clear_all(&self) {
        for layer in &self.layers {
            layer.write().clear();
        }
    }
}

// ---------------------------------------------------------------------------
// Model trait
// ---------------------------------------------------------------------------

/// Configuration for a model forward pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardConfig {
    /// If true, use the KV-cache for autoregressive decoding.
    pub use_cache: bool,
    /// Maximum new tokens to generate.
    pub max_new_tokens: usize,
    /// Sampling temperature (0 = greedy).
    pub temperature: f32,
    /// Top-k sampling cutoff.
    pub top_k: Option<usize>,
    /// Top-p (nucleus) sampling cutoff.
    pub top_p: Option<f32>,
}

impl Default for ForwardConfig {
    fn default() -> Self {
        Self {
            use_cache: true,
            max_new_tokens: 128,
            temperature: 0.0,
            top_k: None,
            top_p: None,
        }
    }
}

/// The output of a single forward pass.
#[derive(Debug, Clone)]
pub struct ForwardOutput {
    /// Raw logits: (batch_size * seq_len, vocab_size) or just (vocab_size,) for single token.
    pub logits: Tensor,
    /// Optional: per-token log-probabilities.
    pub log_probs: Option<Vec<f32>>,
}

/// Trait that all Fusion LLM models must implement.
pub trait Model: Send + Sync {
    /// Model name (e.g. "gpt2-small", "llama-7b").
    fn name(&self) -> &str;

    /// Embedding dimension.
    fn embedding_dim(&self) -> usize;

    /// Number of transformer layers.
    fn num_layers(&self) -> usize;

    /// Vocabulary size.
    fn vocab_size(&self) -> usize;

    /// Run a forward pass on the given token ids.
    fn forward(&self, input_ids: &[u32], config: &ForwardConfig) -> Result<ForwardOutput>;

    /// Access the KV-cache (if present).
    fn kv_cache(&self) -> Option<&KvCache>;

    /// Reset the KV-cache.
    fn reset_cache(&self);

    /// Total parameter count.
    fn param_count(&self) -> usize;
}

// ---------------------------------------------------------------------------
// Model loading — GGUF and SafeTensors concepts
// ---------------------------------------------------------------------------

/// Supported weight file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeightFormat {
    /// GGUF quantized format (used by llama.cpp ecosystem).
    Gguf,
    /// SafeTensors safetensors (HuggingFace standard).
    SafeTensors,
    /// Raw f32 weights (for testing).
    RawF32,
}

/// Metadata about a loaded model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub format: WeightFormat,
    pub vocab_size: usize,
    pub embedding_dim: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub head_dim: usize,
    pub ffn_dim: usize,
    pub max_seq_len: usize,
    pub param_count: usize,
    pub quantization: Option<String>,
}

/// Descriptor for a weight tensor inside a model file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightDescriptor {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: WeightDtype,
    pub offset: u64,
    pub byte_len: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeightDtype {
    F32,
    F16,
    Q8_0,
    Q4_0,
    Q4_1,
    Q5_0,
    Q5_1,
}

impl WeightDtype {
    pub fn byte_width(&self) -> usize {
        match self {
            Self::F32 => 4,
            Self::F16 => 2,
            Self::Q8_0 => 1, // per-block quantization metadata omitted for simplicity
            Self::Q4_0 | Self::Q4_1 | Self::Q5_0 | Self::Q5_1 => 1,
        }
    }
}

/// A loaded weight file — maps tensor names to raw byte data.
pub struct WeightFile {
    pub metadata: ModelMetadata,
    pub tensors: HashMap<String, Vec<f32>>,
}

impl WeightFile {
    pub fn get_tensor(&self, name: &str) -> Option<&Vec<f32>> {
        self.tensors.get(name)
    }

    pub fn tensor_names(&self) -> Vec<&str> {
        self.tensors.keys().map(|s| s.as_str()).collect()
    }
}

/// Concept: parse GGUF header. In production this would read the binary GGUF format;
/// here we provide the structural skeleton.
pub fn parse_gguf_header(_data: &[u8]) -> Result<ModelMetadata> {
    // GGUF magic: 0x46554747 ("GGUF")
    // In production: parse magic, version, tensor count, metadata key-value pairs.
    // Returning a placeholder that demonstrates the expected structure.
    Err(LlmError::Format(
        "GGUF parser: provide binary data from a .gguf file — \
         header parsing stub (magic 0x46554747, version, tensor count, kv-pairs)"
            .to_string(),
    ))
}

/// Concept: parse SafeTensors header (JSON-based index).
pub fn parse_safetensors_header(json_data: &[u8]) -> Result<HashMap<String, WeightDescriptor>> {
    let header: HashMap<String, serde_json::Value> =
        serde_json::from_slice(json_data).map_err(|e| LlmError::Format(e.to_string()))?;
    let mut descriptors = HashMap::new();
    for (key, val) in &header {
        if key == "__metadata__" {
            continue;
        }
        if let Some(arr) = val.as_array() {
            if arr.len() >= 3 {
                let shape: Vec<usize> = arr[0]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_u64().map(|n| n as usize)).collect())
                    .unwrap_or_default();
                let dtype_str = arr[1].as_str().unwrap_or("F32");
                let offset = arr[2].as_u64().unwrap_or(0);
                let dtype = match dtype_str {
                    "F16" => WeightDtype::F16,
                    "BF16" => WeightDtype::F16,
                    "Q8_0" => WeightDtype::Q8_0,
                    "Q4_0" => WeightDtype::Q4_0,
                    _ => WeightDtype::F32,
                };
                descriptors.insert(
                    key.clone(),
                    WeightDescriptor {
                        name: key.clone(),
                        shape,
                        dtype,
                        offset,
                        byte_len: 0,
                    },
                );
            }
        }
    }
    Ok(descriptors)
}

// ---------------------------------------------------------------------------
// Inference engine
// ---------------------------------------------------------------------------

/// Configuration for the inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub model_name: String,
    pub max_seq_len: usize,
    pub batch_size: usize,
    pub use_gpu: bool,
    pub quantization: Option<String>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_name: String::new(),
            max_seq_len: 2048,
            batch_size: 1,
            use_gpu: false,
            quantization: None,
        }
    }
}

/// The inference engine holds a model + tokenizer and drives generation.
pub struct InferenceEngine {
    config: InferenceConfig,
    tokenizer: Box<dyn Tokenizer>,
}

impl InferenceEngine {
    pub fn new(config: InferenceConfig, tokenizer: Box<dyn Tokenizer>) -> Self {
        Self { config, tokenizer }
    }

    pub fn tokenizer(&self) -> &dyn Tokenizer {
        self.tokenizer.as_ref()
    }

    pub fn config(&self) -> &InferenceConfig {
        &self.config
    }

    /// Generate text given a prompt string.
    ///
    /// Uses greedy decoding when temperature == 0.
    pub fn generate(&self, model: &dyn Model, prompt: &str, gen_config: &ForwardConfig) -> Result<String> {
        let prompt_ids = self.tokenizer.encode(prompt)?;
        let mut all_ids = prompt_ids.clone();
        let mut generated = Vec::new();

        for _ in 0..gen_config.max_new_tokens {
            let output = model.forward(&all_ids, gen_config)?;
            // Take the last token's logits
            let vocab_size = output.logits.cols;
            let last_offset = (output.logits.rows - 1) * vocab_size;
            let last_logits = &output.logits.data[last_offset..last_offset + vocab_size];

            let next_id = if gen_config.temperature == 0.0 {
                // Greedy
                argmax(last_logits)
            } else {
                // Temperature-scaled sampling
                sample_with_temperature(last_logits, gen_config.temperature)
            };

            all_ids.push(next_id);
            generated.push(next_id);

            // Check for end-of-sequence (convention: id 0 = EOS in many tokenizers)
            if next_id == 0 {
                break;
            }
        }

        // Decode only the generated tokens (not the prompt)
        let generated_str = self.tokenizer.decode(&generated)?;
        Ok(generated_str)
    }
}

// ---------------------------------------------------------------------------
// Sampling helpers
// ---------------------------------------------------------------------------

pub fn argmax(slice: &[f32]) -> u32 {
    let mut best = 0usize;
    for i in 1..slice.len() {
        if slice[i] > slice[best] {
            best = i;
        }
    }
    best as u32
}

pub fn sample_with_temperature(logits: &[f32], temperature: f32) -> u32 {
    // Apply temperature scaling
    let scaled: Vec<f32> = logits.iter().map(|l| l / temperature).collect();
    // Softmax
    let max_val = scaled.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exp_sum: f32 = scaled.iter().map(|v| (v - max_val).exp()).sum();
    let probs: Vec<f32> = scaled.iter().map(|v| (v - max_val).exp() / exp_sum).collect();

    // Sample from distribution using a simple linear congruential generator
    // (in production: use rand crate)
    let r: f32 = simple_random_f32();
    let mut cumulative = 0.0;
    for (i, p) in probs.iter().enumerate() {
        cumulative += p;
        if r <= cumulative {
            return i as u32;
        }
    }
    (probs.len() - 1) as u32
}

/// Simple deterministic random f32 in [0, 1) for sampling (not cryptographically secure).
fn simple_random_f32() -> f32 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = Cell::new(123456789);
    }
    STATE.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x as f32) / (u64::MAX as f32)
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_zeros() {
        let t = Tensor::zeros(3, 4);
        assert_eq!(t.rows, 3);
        assert_eq!(t.cols, 4);
        assert_eq!(t.numel(), 12);
        assert!(t.data.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_tensor_matmul() {
        // 2×3 × 3×2 = 2×2
        let a = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 2, 3);
        let b = Tensor::new(vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0], 3, 2);
        let c = a.matmul(&b);
        assert_eq!(c.rows, 2);
        assert_eq!(c.cols, 2);
        // 1*7+2*9+3*11 = 7+18+33 = 58
        assert!((c.get(0, 0) - 58.0).abs() < 1e-5);
        // 1*8+2*10+3*12 = 8+20+36 = 64
        assert!((c.get(0, 1) - 64.0).abs() < 1e-5);
    }

    #[test]
    fn test_tensor_add() {
        let a = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let b = Tensor::new(vec![5.0, 6.0, 7.0, 8.0], 2, 2);
        let c = a.add(&b);
        assert_eq!(c.data, vec![6.0, 8.0, 10.0, 12.0]);
    }

    #[test]
    fn test_tensor_softmax() {
        let t = Tensor::new(vec![1.0, 2.0, 3.0, 1.0, 2.0, 3.0], 2, 3);
        let s = t.softmax();
        // Each row should sum to 1.0
        for r in 0..2 {
            let sum: f32 = (0..3).map(|c| s.get(r, c)).sum();
            assert!((sum - 1.0).abs() < 1e-5, "row {r} sums to {sum}");
        }
    }

    #[test]
    fn test_bpe_vocab_roundtrip() {
        let mut vocab = BpeVocab::with_byte_fallback();
        let id = vocab.add_token("hello");
        assert_eq!(vocab.token_str(id), Some("hello"));
    }

    #[test]
    fn test_kv_cache_append() {
        let mut cache = KvCacheEntry::new(2, 4, 8); // 2 heads, dim 4
        // Append one position: 2 heads × 4 dims = 8 values
        let keys = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let values = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        cache.append(&keys, &values);
        assert_eq!(cache.seq_len, 1);
        let k_head0 = cache.keys_for_head(0);
        assert!((k_head0[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_argmax() {
        let logits = vec![1.0, 5.0, 3.0, 2.0];
        assert_eq!(argmax(&logits), 1);
    }

    #[test]
    fn test_forward_config_default() {
        let cfg = ForwardConfig::default();
        assert!(cfg.use_cache);
        assert_eq!(cfg.max_new_tokens, 128);
        assert_eq!(cfg.temperature, 0.0);
    }

    #[test]
    fn test_tensor_display() {
        let t = Tensor::zeros(4, 8);
        assert_eq!(format!("{t}"), "Tensor(4×8)");
    }

    #[test]
    fn test_weight_dtype_byte_width() {
        assert_eq!(WeightDtype::F32.byte_width(), 4);
        assert_eq!(WeightDtype::F16.byte_width(), 2);
        assert_eq!(WeightDtype::Q4_0.byte_width(), 1);
    }

    #[test]
    fn test_weight_file_lookup() {
        let mut tensors = HashMap::new();
        tensors.insert("layer.0.weight".to_string(), vec![1.0, 2.0, 3.0]);
        let wf = WeightFile {
            metadata: ModelMetadata {
                name: "test".to_string(),
                format: WeightFormat::RawF32,
                vocab_size: 256,
                embedding_dim: 64,
                num_layers: 2,
                num_heads: 2,
                head_dim: 32,
                ffn_dim: 256,
                max_seq_len: 512,
                param_count: 1000,
                quantization: None,
            },
            tensors,
        };
        assert!(wf.get_tensor("layer.0.weight").is_some());
        assert!(wf.get_tensor("missing").is_none());
    }

    #[test]
    fn test_sample_deterministic() {
        let logits = vec![0.0, 0.0, 100.0, 0.0];
        // Even with temperature, 100.0 should dominate
        for _ in 0..10 {
            let id = sample_with_temperature(&logits, 1.0);
            assert_eq!(id, 2);
        }
    }
}
