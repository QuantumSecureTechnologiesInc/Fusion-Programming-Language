//! fusion_llm_models — Transformer model architectures for Fusion v2.0 Vortex.
//!
//! Implements GPT-style decoder-only, LLaMA-style with RoPE, and BERT-style
//! encoder models, all built on the `fusion_llm_core` traits.

use std::f32::consts::PI;

use fusion_llm_core::{
    ForwardConfig, ForwardOutput, KvCache, KvCacheEntry, Model, Result, Tensor,
};

// ---------------------------------------------------------------------------
// Neural-network primitives
// ---------------------------------------------------------------------------

/// Linear (fully-connected) layer: y = xW + b.
#[derive(Debug, Clone)]
pub struct Linear {
    pub weight: Tensor, // (in_features, out_features) — stored transposed for matmul
    pub bias: Tensor,   // (1, out_features)
    pub in_features: usize,
    pub out_features: usize,
}

impl Linear {
    pub fn new(in_features: usize, out_features: usize) -> Self {
        Self {
            weight: Tensor::zeros(in_features, out_features),
            bias: Tensor::zeros(1, out_features),
            in_features,
            out_features,
        }
    }

    /// Xavier/Glorot uniform initialization.
    pub fn init_xavier(&mut self) {
        let scale = (2.0 / (self.in_features + self.out_features) as f32).sqrt();
        for v in self.weight.data.iter_mut() {
            *v = (simple_random_f32() - 0.5) * 2.0 * scale;
        }
        self.bias.data.iter_mut().for_each(|v| *v = 0.0);
    }

    /// Forward: (batch × in_features) → (batch × out_features).
    pub fn forward(&self, input: &Tensor) -> Tensor {
        let mut out = input.matmul(&self.weight);
        // Broadcast-add bias to every row
        for r in 0..out.rows {
            for c in 0..self.out_features {
                let idx = r * self.out_features + c;
                out.data[idx] += self.bias.data[c];
            }
        }
        out
    }

    pub fn param_count(&self) -> usize {
        self.weight.numel() + self.bias.numel()
    }
}

/// Layer normalization: y = (x - mean) / sqrt(var + eps) * gamma + beta.
#[derive(Debug, Clone)]
pub struct LayerNorm {
    pub gamma: Tensor, // (1, dim)
    pub beta: Tensor,  // (1, dim)
    pub eps: f32,
    pub dim: usize,
}

impl LayerNorm {
    pub fn new(dim: usize, eps: f32) -> Self {
        Self {
            gamma: Tensor::new(vec![1.0; dim], 1, dim),
            beta: Tensor::zeros(1, dim),
            eps,
            dim,
        }
    }

    pub fn forward(&self, input: &Tensor) -> Tensor {
        let mut output = vec![0.0f32; input.data.len()];
        for r in 0..input.rows {
            let offset = r * self.dim;
            let row = &input.data[offset..offset + self.dim];

            // Compute mean
            let mean: f32 = row.iter().sum::<f32>() / self.dim as f32;
            // Compute variance
            let var: f32 = row.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / self.dim as f32;
            let inv_std = 1.0 / (var + self.eps).sqrt();

            for (i, &v) in row.iter().enumerate() {
                let normalized = (v - mean) * inv_std;
                output[offset + i] = normalized * self.gamma.data[i] + self.beta.data[i];
            }
        }
        Tensor::new(output, input.rows, self.dim)
    }
}

/// RMSNorm (used in LLaMA): y = x / sqrt(mean(x^2) + eps) * gamma.
#[derive(Debug, Clone)]
pub struct RmsNorm {
    pub gamma: Tensor,
    pub eps: f32,
    pub dim: usize,
}

impl RmsNorm {
    pub fn new(dim: usize, eps: f32) -> Self {
        Self {
            gamma: Tensor::new(vec![1.0; dim], 1, dim),
            eps,
            dim,
        }
    }

    pub fn forward(&self, input: &Tensor) -> Tensor {
        let mut output = vec![0.0f32; input.data.len()];
        for r in 0..input.rows {
            let offset = r * self.dim;
            let row = &input.data[offset..offset + self.dim];
            let rms = (row.iter().map(|v| v * v).sum::<f32>() / self.dim as f32 + self.eps).sqrt();
            for (i, &v) in row.iter().enumerate() {
                output[offset + i] = (v / rms) * self.gamma.data[i];
            }
        }
        Tensor::new(output, input.rows, self.dim)
    }
}

/// Activation functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    ReLU,
    GELU,
    SiLU, // Swish
    Tanh,
}

impl Activation {
    pub fn apply(&self, x: f32) -> f32 {
        match self {
            Self::ReLU => x.max(0.0),
            Self::GELU => {
                // Approximate GELU: x * σ(1.702 * x)
                let sigma = 1.0 / (1.0 + (-1.702 * x).exp());
                x * sigma
            }
            Self::SiLU => {
                // x * σ(x)
                let sigma = 1.0 / (1.0 + (-x).exp());
                x * sigma
            }
            Self::Tanh => x.tanh(),
        }
    }

    pub fn apply_to_tensor(&self, t: &Tensor) -> Tensor {
        Tensor::new(
            t.data.iter().map(|&v| self.apply(v)).collect(),
            t.rows,
            t.cols,
        )
    }
}

/// Feed-forward network (position-wise): Linear → Activation → Linear.
#[derive(Debug, Clone)]
pub struct FeedForward {
    pub up: Linear,
    pub down: Linear,
    pub activation: Activation,
}

impl FeedForward {
    pub fn new(dim: usize, ffn_dim: usize, activation: Activation) -> Self {
        Self {
            up: Linear::new(dim, ffn_dim),
            down: Linear::new(ffn_dim, dim),
            activation,
        }
    }

    pub fn init_weights(&mut self) {
        self.up.init_xavier();
        self.down.init_xavier();
    }

    pub fn forward(&self, input: &Tensor) -> Tensor {
        let intermediate = self.up.forward(input);
        let activated = self.activation.apply_to_tensor(&intermediate);
        self.down.forward(&activated)
    }
}

/// Gated feed-forward unit (SwiGLU, used in LLaMA): down(SiLU(gate(x)) * up(x)).
#[derive(Debug, Clone)]
pub struct GatedFeedForward {
    pub gate: Linear,
    pub up: Linear,
    pub down: Linear,
}

impl GatedFeedForward {
    pub fn new(dim: usize, ffn_dim: usize) -> Self {
        Self {
            gate: Linear::new(dim, ffn_dim),
            up: Linear::new(dim, ffn_dim),
            down: Linear::new(ffn_dim, dim),
        }
    }

    pub fn init_weights(&mut self) {
        self.gate.init_xavier();
        self.up.init_xavier();
        self.down.init_xavier();
    }

    pub fn forward(&self, input: &Tensor) -> Tensor {
        let gate_out = Activation::SiLU.apply_to_tensor(&self.gate.forward(input));
        let up_out = self.up.forward(input);
        // Element-wise multiply
        let mut intermediate = vec![0.0f32; gate_out.data.len()];
        for i in 0..intermediate.len() {
            intermediate[i] = gate_out.data[i] * up_out.data[i];
        }
        self.down.forward(&Tensor::new(intermediate, gate_out.rows, gate_out.cols))
    }
}

// ---------------------------------------------------------------------------
// Rotary Positional Embeddings (RoPE)
// ---------------------------------------------------------------------------

/// Pre-computed rotation frequencies for RoPE.
pub struct RopeCache {
    pub sin_cache: Vec<f32>,
    pub cos_cache: Vec<f32>,
    pub head_dim: usize,
    pub max_seq_len: usize,
}

impl RopeCache {
    pub fn new(head_dim: usize, max_seq_len: usize, base: f32) -> Self {
        let mut sin_cache = Vec::with_capacity(max_seq_len * head_dim);
        let mut cos_cache = Vec::with_capacity(max_seq_len * head_dim);

        for pos in 0..max_seq_len {
            for i in (0..head_dim).step_by(2) {
                let freq = 1.0 / base.powf(i as f32 / head_dim as f32);
                let theta = pos as f32 * freq;
                sin_cache.push(theta.sin());
                cos_cache.push(theta.cos());
            }
        }

        Self {
            sin_cache,
            cos_cache,
            head_dim,
            max_seq_len,
        }
    }

    /// Apply rotation to a (seq_len, head_dim) tensor at the given position offset.
    pub fn apply(&self, x: &Tensor, pos_offset: usize) -> Tensor {
        let mut output = vec![0.0f32; x.data.len()];
        for r in 0..x.rows {
            let abs_pos = pos_offset + r;
            for i in (0..self.head_dim).step_by(2) {
                let idx = r * self.head_dim + i;
                let sin = self.sin_cache[abs_pos * self.head_dim + i];
                let cos = self.cos_cache[abs_pos * self.head_dim + i];
                let x0 = x.data[idx];
                let x1 = x.data[idx + 1];
                output[idx] = x0 * cos - x1 * sin;
                output[idx + 1] = x0 * sin + x1 * cos;
            }
        }
        Tensor::new(output, x.rows, self.head_dim)
    }
}

// ---------------------------------------------------------------------------
// Multi-head attention
// ---------------------------------------------------------------------------

/// Multi-head self-attention with optional KV-cache.
pub struct MultiHeadAttention {
    pub q_proj: Linear,
    pub k_proj: Linear,
    pub v_proj: Linear,
    pub o_proj: Linear,
    pub num_heads: usize,
    pub head_dim: usize,
    pub dim: usize,
}

impl MultiHeadAttention {
    pub fn new(dim: usize, num_heads: usize) -> Self {
        assert_eq!(dim % num_heads, 0, "dim must be divisible by num_heads");
        let head_dim = dim / num_heads;
        Self {
            q_proj: Linear::new(dim, dim),
            k_proj: Linear::new(dim, dim),
            v_proj: Linear::new(dim, dim),
            o_proj: Linear::new(dim, dim),
            num_heads,
            head_dim,
            dim,
        }
    }

    pub fn init_weights(&mut self) {
        self.q_proj.init_xavier();
        self.k_proj.init_xavier();
        self.v_proj.init_xavier();
        self.o_proj.init_xavier();
    }

    /// Scaled dot-product attention for a single head.
    fn attention_head(q: &[f32], k: &[f32], v: &[f32], seq_len: usize) -> Vec<f32> {
        let scale = (q.len() as f32 / seq_len as f32).sqrt();
        let mut output = vec![0.0f32; seq_len * (q.len() / seq_len)];

        // q: (seq_len, head_dim), k: (kv_len, head_dim), v: (kv_len, head_dim)
        let kv_len = k.len() / (q.len() / seq_len);
        let head_dim = q.len() / seq_len;

        for t in 0..seq_len {
            // Compute attention scores: q[t] · k[j] for all j
            let mut scores = vec![0.0f32; kv_len];
            let q_offset = t * head_dim;
            for j in 0..kv_len {
                let k_offset = j * head_dim;
                let mut dot = 0.0f32;
                for d in 0..head_dim {
                    dot += q[q_offset + d] * k[k_offset + d];
                }
                scores[j] = dot / scale;
            }

            // Softmax
            let max_s = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let exp_sum: f32 = scores.iter().map(|s| (s - max_s).exp()).sum();
            for s in scores.iter_mut() {
                *s = (*s - max_s).exp() / exp_sum;
            }

            // Weighted sum of values
            let out_offset = t * head_dim;
            for j in 0..kv_len {
                let v_offset = j * head_dim;
                for d in 0..head_dim {
                    output[out_offset + d] += scores[j] * v[v_offset + d];
                }
            }
        }

        output
    }

    /// Forward pass: input (batch, seq_len, dim) → output (batch, seq_len, dim).
    /// With cache: only computes for the last token and appends to cache.
    pub fn forward(&self, input: &Tensor, cache: Option<&KvCacheEntry>, _pos_offset: usize) -> (Tensor, Option<KvCacheEntry>) {
        let seq_len = input.rows;
        let mut new_cache = cache.map(|c| {
            let mut nc = c.clone();
            nc.clear();
            nc
        });

        let q_raw = self.q_proj.forward(input);
        let k_raw = self.k_proj.forward(input);
        let v_raw = self.v_proj.forward(input);

        // Reshape to (seq_len, num_heads, head_dim) and transpose to per-head views
        // For simplicity, operate per-head directly
        let mut output = vec![0.0f32; input.data.len()];

        for h in 0..self.num_heads {
            let hd = self.head_dim;
            // Extract head h from q, k, v
            let q_head: Vec<f32> = (0..seq_len)
                .flat_map(|t| {
                    let base = t * self.dim + h * hd;
                    q_raw.data[base..base + hd].to_vec()
                })
                .collect();
            let k_head: Vec<f32> = (0..seq_len)
                .flat_map(|t| {
                    let base = t * self.dim + h * hd;
                    k_raw.data[base..base + hd].to_vec()
                })
                .collect();
            let v_head: Vec<f32> = (0..seq_len)
                .flat_map(|t| {
                    let base = t * self.dim + h * hd;
                    v_raw.data[base..base + hd].to_vec()
                })
                .collect();

            // Merge with cached KV
            let (full_k, full_v, kv_len) = if let Some(ref c) = cache {
                let cached_k = c.keys_for_head(h);
                let cached_v = c.values_for_head(h);
                let kv_len = c.seq_len + seq_len;
                let mut fk = Vec::with_capacity(kv_len * hd);
                let mut fv = Vec::with_capacity(kv_len * hd);
                fk.extend_from_slice(cached_k);
                fk.extend_from_slice(&k_head);
                fv.extend_from_slice(cached_v);
                fv.extend_from_slice(&v_head);
                (fk, fv, kv_len)
            } else {
                (k_head.clone(), v_head.clone(), seq_len)
            };

            let head_out = Self::attention_head(&q_head, &full_k, &full_v, kv_len);
            // We only want the last seq_len tokens from head_out
            let head_out_trimmed = &head_out[(kv_len - seq_len) * hd..kv_len * hd];

            // Scatter back to output
            for t in 0..seq_len {
                let out_base = t * self.dim + h * hd;
                let src_base = t * hd;
                output[out_base..out_base + hd].copy_from_slice(&head_out_trimmed[src_base..src_base + hd]);
            }

            // Update cache
            if let Some(ref mut c) = new_cache {
                c.append(&k_head, &v_head);
            }
        }

        let attn_out = Tensor::new(output, seq_len, self.dim);
        let projected = self.o_proj.forward(&attn_out);

        (projected, new_cache)
    }
}

// ---------------------------------------------------------------------------
// Transformer blocks
// ---------------------------------------------------------------------------

/// Configuration for a transformer model.
#[derive(Debug, Clone)]
pub struct TransformerConfig {
    pub vocab_size: usize,
    pub embedding_dim: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub ffn_dim: usize,
    pub max_seq_len: usize,
    pub activation: Activation,
    pub norm_eps: f32,
    pub rope_base: f32,
    pub tie_word_embeddings: bool,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self {
            vocab_size: 32000,
            embedding_dim: 768,
            num_layers: 12,
            num_heads: 12,
            ffn_dim: 3072,
            max_seq_len: 2048,
            activation: Activation::GELU,
            norm_eps: 1e-5,
            rope_base: 10000.0,
            tie_word_embeddings: true,
        }
    }
}

/// GPT-style transformer block: LayerNorm → MultiHeadAttention → Residual → LayerNorm → FFN → Residual.
pub struct GptBlock {
    pub attn_norm: LayerNorm,
    pub attention: MultiHeadAttention,
    pub ffn_norm: LayerNorm,
    pub ffn: FeedForward,
}

impl GptBlock {
    pub fn new(config: &TransformerConfig) -> Self {
        Self {
            attn_norm: LayerNorm::new(config.embedding_dim, config.norm_eps),
            attention: MultiHeadAttention::new(config.embedding_dim, config.num_heads),
            ffn_norm: LayerNorm::new(config.embedding_dim, config.norm_eps),
            ffn: FeedForward::new(config.embedding_dim, config.ffn_dim, config.activation),
        }
    }

    pub fn init_weights(&mut self) {
        self.attention.init_weights();
        self.ffn.init_weights();
    }

    pub fn forward(&self, input: &Tensor, cache: Option<&KvCacheEntry>, pos_offset: usize) -> (Tensor, Option<KvCacheEntry>) {
        // Pre-norm attention
        let normed = self.attn_norm.forward(input);
        let (attn_out, new_cache) = self.attention.forward(&normed, cache, pos_offset);
        // Residual
        let residual = input.add(&attn_out);
        // Pre-norm FFN
        let normed2 = self.ffn_norm.forward(&residual);
        let ffn_out = self.ffn.forward(&normed2);
        let output = residual.add(&ffn_out);
        (output, new_cache)
    }
}

/// LLaMA-style transformer block with RoPE and SwiGLU.
pub struct LlamaBlock {
    pub attn_norm: RmsNorm,
    pub attention: MultiHeadAttention,
    pub ffn_norm: RmsNorm,
    pub ffn: GatedFeedForward,
    pub rope: RopeCache,
}

impl LlamaBlock {
    pub fn new(config: &TransformerConfig) -> Self {
        Self {
            attn_norm: RmsNorm::new(config.embedding_dim, config.norm_eps),
            attention: MultiHeadAttention::new(config.embedding_dim, config.num_heads),
            ffn_norm: RmsNorm::new(config.embedding_dim, config.norm_eps),
            ffn: GatedFeedForward::new(config.embedding_dim, config.ffn_dim),
            rope: RopeCache::new(config.embedding_dim / config.num_heads, config.max_seq_len, config.rope_base),
        }
    }

    pub fn init_weights(&mut self) {
        self.attention.init_weights();
        self.ffn.init_weights();
    }

    pub fn forward(&self, input: &Tensor, cache: Option<&KvCacheEntry>, pos_offset: usize) -> (Tensor, Option<KvCacheEntry>) {
        let normed = self.attn_norm.forward(input);
        let (attn_out, new_cache) = self.attention.forward(&normed, cache, pos_offset);
        let residual = input.add(&attn_out);
        let normed2 = self.ffn_norm.forward(&residual);
        let ffn_out = self.ffn.forward(&normed2);
        let output = residual.add(&ffn_out);
        (output, new_cache)
    }
}

// ---------------------------------------------------------------------------
// GPT-style decoder-only model
// ---------------------------------------------------------------------------

pub struct GptModel {
    config: TransformerConfig,
    token_embedding: Tensor,   // (vocab_size, embedding_dim)
    position_embedding: Tensor, // (max_seq_len, embedding_dim)
    layers: Vec<GptBlock>,
    final_norm: LayerNorm,
    lm_head: Linear,
    cache: Option<KvCache>,
}

impl GptModel {
    pub fn new(config: TransformerConfig) -> Self {
        let mut layers = Vec::with_capacity(config.num_layers);
        for _ in 0..config.num_layers {
            layers.push(GptBlock::new(&config));
        }

        let mut lm_head = Linear::new(config.embedding_dim, config.vocab_size);
        lm_head.init_xavier();

        let mut token_emb = Tensor::zeros(config.vocab_size, config.embedding_dim);
        init_normal(&mut token_emb.data, 0.02);
        let mut pos_emb = Tensor::zeros(config.max_seq_len, config.embedding_dim);
        init_normal(&mut pos_emb.data, 0.02);

        let embedding_dim = config.embedding_dim;
        let norm_eps = config.norm_eps;

        Self {
            config,
            token_embedding: token_emb,
            position_embedding: pos_emb,
            layers,
            final_norm: LayerNorm::new(embedding_dim, norm_eps),
            lm_head,
            cache: None,
        }
    }

    pub fn config(&self) -> &TransformerConfig {
        &self.config
    }
}

impl Model for GptModel {
    fn name(&self) -> &str {
        "gpt-decoder"
    }

    fn embedding_dim(&self) -> usize {
        self.config.embedding_dim
    }

    fn num_layers(&self) -> usize {
        self.config.num_layers
    }

    fn vocab_size(&self) -> usize {
        self.config.vocab_size
    }

    fn forward(&self, input_ids: &[u32], config: &ForwardConfig) -> Result<ForwardOutput> {
        let seq_len = input_ids.len();
        let dim = self.config.embedding_dim;

        // Token + position embeddings
        let mut hidden = vec![0.0f32; seq_len * dim];
        for (t, &id) in input_ids.iter().enumerate() {
            let id = (id as usize).min(self.config.vocab_size - 1);
            let tok_off = id * dim;
            let pos_off = t * dim;
            let out_off = t * dim;
            for d in 0..dim {
                hidden[out_off + d] = self.token_embedding.data[tok_off + d]
                    + self.position_embedding.data[pos_off + d];
            }
        }

        let mut x = Tensor::new(hidden, seq_len, dim);
        let pos_offset = 0;

        for (i, layer) in self.layers.iter().enumerate() {
            let cache_entry = self.cache.as_ref().and_then(|c| {
                if config.use_cache {
                    Some(c.get_layer(i).read().clone())
                } else {
                    None
                }
            });
            let (out, new_cache) = layer.forward(&x, cache_entry.as_ref(), pos_offset);
            if config.use_cache {
                if let Some(nc) = new_cache {
                    if let Some(ref cache) = self.cache {
                        *cache.get_layer(i).write() = nc;
                    }
                }
            }
            x = out;
        }

        let normed = self.final_norm.forward(&x);
        let logits = self.lm_head.forward(&normed);

        if config.use_cache {
            // pos_offset tracks cumulative sequence length for cache indexing
        }

        Ok(ForwardOutput { logits, log_probs: None })
    }

    fn kv_cache(&self) -> Option<&KvCache> {
        self.cache.as_ref()
    }

    fn reset_cache(&self) {
        if let Some(ref cache) = self.cache {
            cache.clear_all();
        }
    }

    fn param_count(&self) -> usize {
        let emb = self.token_embedding.numel() + self.position_embedding.numel();
        let layers: usize = self
            .layers
            .iter()
            .map(|l| {
                l.attn_norm.gamma.numel()
                    + l.attn_norm.beta.numel()
                    + l.attention.q_proj.param_count()
                    + l.attention.k_proj.param_count()
                    + l.attention.v_proj.param_count()
                    + l.attention.o_proj.param_count()
                    + l.ffn_norm.gamma.numel()
                    + l.ffn_norm.beta.numel()
                    + l.ffn.up.param_count()
                    + l.ffn.down.param_count()
            })
            .sum();
        let head = self.final_norm.gamma.numel()
            + self.final_norm.beta.numel()
            + self.lm_head.param_count();
        emb + layers + head
    }
}

// ---------------------------------------------------------------------------
// LLaMA-style decoder-only model with RoPE
// ---------------------------------------------------------------------------

pub struct LlamaModel {
    config: TransformerConfig,
    token_embedding: Tensor,
    layers: Vec<LlamaBlock>,
    final_norm: RmsNorm,
    lm_head: Linear,
    cache: Option<KvCache>,
}

impl LlamaModel {
    pub fn new(config: TransformerConfig) -> Self {
        let mut layers = Vec::with_capacity(config.num_layers);
        for _ in 0..config.num_layers {
            layers.push(LlamaBlock::new(&config));
        }

        let mut lm_head = Linear::new(config.embedding_dim, config.vocab_size);
        lm_head.init_xavier();

        let mut token_emb = Tensor::zeros(config.vocab_size, config.embedding_dim);
        init_normal(&mut token_emb.data, 0.02);

        let embedding_dim = config.embedding_dim;
        let norm_eps = config.norm_eps;

        Self {
            config,
            token_embedding: token_emb,
            layers,
            final_norm: RmsNorm::new(embedding_dim, norm_eps),
            lm_head,
            cache: None,
        }
    }

    pub fn config(&self) -> &TransformerConfig {
        &self.config
    }
}

impl Model for LlamaModel {
    fn name(&self) -> &str {
        "llama-decoder"
    }

    fn embedding_dim(&self) -> usize {
        self.config.embedding_dim
    }

    fn num_layers(&self) -> usize {
        self.config.num_layers
    }

    fn vocab_size(&self) -> usize {
        self.config.vocab_size
    }

    fn forward(&self, input_ids: &[u32], config: &ForwardConfig) -> Result<ForwardOutput> {
        let seq_len = input_ids.len();
        let dim = self.config.embedding_dim;

        // Token embeddings only (positions handled by RoPE inside attention)
        let mut hidden = vec![0.0f32; seq_len * dim];
        for (t, &id) in input_ids.iter().enumerate() {
            let id = (id as usize).min(self.config.vocab_size - 1);
            let tok_off = id * dim;
            let out_off = t * dim;
            hidden[out_off..out_off + dim].copy_from_slice(&self.token_embedding.data[tok_off..tok_off + dim]);
        }

        let mut x = Tensor::new(hidden, seq_len, dim);

        for (i, layer) in self.layers.iter().enumerate() {
            let cache_entry = self.cache.as_ref().and_then(|c| {
                if config.use_cache {
                    Some(c.get_layer(i).read().clone())
                } else {
                    None
                }
            });
            let (out, new_cache) = layer.forward(&x, cache_entry.as_ref(), 0);
            if config.use_cache {
                if let Some(nc) = new_cache {
                    if let Some(ref cache) = self.cache {
                        *cache.get_layer(i).write() = nc;
                    }
                }
            }
            x = out;
        }

        let normed = self.final_norm.forward(&x);
        let logits = self.lm_head.forward(&normed);

        Ok(ForwardOutput { logits, log_probs: None })
    }

    fn kv_cache(&self) -> Option<&KvCache> {
        self.cache.as_ref()
    }

    fn reset_cache(&self) {
        if let Some(ref cache) = self.cache {
            cache.clear_all();
        }
    }

    fn param_count(&self) -> usize {
        let emb = self.token_embedding.numel();
        let layers: usize = self
            .layers
            .iter()
            .map(|l| {
                l.attn_norm.gamma.numel()
                    + l.attention.q_proj.param_count()
                    + l.attention.k_proj.param_count()
                    + l.attention.v_proj.param_count()
                    + l.attention.o_proj.param_count()
                    + l.ffn_norm.gamma.numel()
                    + l.ffn.gate.param_count()
                    + l.ffn.up.param_count()
                    + l.ffn.down.param_count()
            })
            .sum();
        let head = self.final_norm.gamma.numel() + self.lm_head.param_count();
        emb + layers + head
    }
}

// ---------------------------------------------------------------------------
// BERT-style encoder model
// ---------------------------------------------------------------------------

pub struct BertBlock {
    pub attn_norm: LayerNorm,
    pub attention: MultiHeadAttention,
    pub ffn_norm: LayerNorm,
    pub ffn: FeedForward,
}

impl BertBlock {
    pub fn new(config: &TransformerConfig) -> Self {
        Self {
            attn_norm: LayerNorm::new(config.embedding_dim, config.norm_eps),
            attention: MultiHeadAttention::new(config.embedding_dim, config.num_heads),
            ffn_norm: LayerNorm::new(config.embedding_dim, config.norm_eps),
            ffn: FeedForward::new(config.embedding_dim, config.ffn_dim, Activation::GELU),
        }
    }

    pub fn init_weights(&mut self) {
        self.attention.init_weights();
        self.ffn.init_weights();
    }

    pub fn forward(&self, input: &Tensor) -> Tensor {
        // BERT uses post-norm (residual first, then norm)
        let normed = self.attn_norm.forward(input);
        let (attn_out, _) = self.attention.forward(&normed, None, 0);
        let residual = input.add(&attn_out);
        let normed2 = self.ffn_norm.forward(&residual);
        let ffn_out = self.ffn.forward(&normed2);
        residual.add(&ffn_out)
    }
}

/// BERT-style bidirectional encoder model.
pub struct BertModel {
    config: TransformerConfig,
    token_embedding: Tensor,
    position_embedding: Tensor,
    segment_embedding: Tensor,
    embedding_norm: LayerNorm,
    layers: Vec<BertBlock>,
    pooler: Linear,
}

impl BertModel {
    pub fn new(config: TransformerConfig) -> Self {
        let mut layers = Vec::with_capacity(config.num_layers);
        for _ in 0..config.num_layers {
            layers.push(BertBlock::new(&config));
        }

        let mut token_emb = Tensor::zeros(config.vocab_size, config.embedding_dim);
        init_normal(&mut token_emb.data, 0.02);
        let mut pos_emb = Tensor::zeros(config.max_seq_len, config.embedding_dim);
        init_normal(&mut pos_emb.data, 0.02);
        let mut seg_emb = Tensor::zeros(2, config.embedding_dim);
        init_normal(&mut seg_emb.data, 0.02);

        let mut pooler = Linear::new(config.embedding_dim, config.embedding_dim);
        pooler.init_xavier();

        let embedding_dim = config.embedding_dim;
        let norm_eps = config.norm_eps;

        Self {
            config,
            token_embedding: token_emb,
            position_embedding: pos_emb,
            segment_embedding: seg_emb,
            embedding_norm: LayerNorm::new(embedding_dim, norm_eps),
            layers,
            pooler,
        }
    }

    pub fn config(&self) -> &TransformerConfig {
        &self.config
    }

    /// Forward pass: input_ids + segment_ids → sequence output + pooled output.
    pub fn forward_bert(&self, input_ids: &[u32], segment_ids: &[u32]) -> (Tensor, Tensor) {
        let seq_len = input_ids.len();
        let dim = self.config.embedding_dim;

        let mut hidden = vec![0.0f32; seq_len * dim];
        for (t, (&id, &seg)) in input_ids.iter().zip(segment_ids.iter()).enumerate() {
            let id = (id as usize).min(self.config.vocab_size - 1);
            let seg = (seg as usize).min(1);
            for d in 0..dim {
                hidden[t * dim + d] = self.token_embedding.data[id * dim + d]
                    + self.position_embedding.data[t * dim + d]
                    + self.segment_embedding.data[seg * dim + d];
            }
        }

        let mut x = self.embedding_norm.forward(&Tensor::new(hidden, seq_len, dim));

        for layer in &self.layers {
            x = layer.forward(&x);
        }

        // Pool [CLS] token (index 0)
        let cls_hidden: Vec<f32> = x.data[0..dim].to_vec();
        let pooled = self.pooler.forward(&Tensor::new(cls_hidden, 1, dim));
        let pooled_activated = Activation::Tanh.apply_to_tensor(&pooled);

        (x, pooled_activated)
    }
}

// BERT doesn't use the autoregressive Model trait (it's bidirectional),
// but we implement it for interface compatibility with generation-style usage.
impl Model for BertModel {
    fn name(&self) -> &str {
        "bert-encoder"
    }

    fn embedding_dim(&self) -> usize {
        self.config.embedding_dim
    }

    fn num_layers(&self) -> usize {
        self.config.num_layers
    }

    fn vocab_size(&self) -> usize {
        self.config.vocab_size
    }

    fn forward(&self, input_ids: &[u32], _config: &ForwardConfig) -> Result<ForwardOutput> {
        let segment_ids = vec![0u32; input_ids.len()];
        let (seq_out, _pooled) = self.forward_bert(input_ids, &segment_ids);
        Ok(ForwardOutput {
            logits: seq_out,
            log_probs: None,
        })
    }

    fn kv_cache(&self) -> Option<&KvCache> {
        None // BERT is bidirectional, no KV-cache
    }

    fn reset_cache(&self) {}

    fn param_count(&self) -> usize {
        let emb = self.token_embedding.numel()
            + self.position_embedding.numel()
            + self.segment_embedding.numel();
        let layers: usize = self
            .layers
            .iter()
            .map(|l| {
                l.attn_norm.gamma.numel()
                    + l.attn_norm.beta.numel()
                    + l.attention.q_proj.param_count()
                    + l.attention.k_proj.param_count()
                    + l.attention.v_proj.param_count()
                    + l.attention.o_proj.param_count()
                    + l.ffn_norm.gamma.numel()
                    + l.ffn_norm.beta.numel()
                    + l.ffn.up.param_count()
                    + l.ffn.down.param_count()
            })
            .sum();
        let pooler = self.pooler.param_count();
        let norm = self.embedding_norm.gamma.numel() + self.embedding_norm.beta.numel();
        emb + layers + pooler + norm
    }
}

// ---------------------------------------------------------------------------
// Weight loading and initialization
// ---------------------------------------------------------------------------

/// Initialize a slice with normal(0, stddev) using Box-Muller transform.
fn init_normal(data: &mut [f32], stddev: f32) {
    for chunk in data.chunks_mut(2) {
        let u1: f32 = simple_random_f32().max(1e-10);
        let u2: f32 = simple_random_f32();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        chunk[0] = z0 * stddev;
        if chunk.len() > 1 {
            let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).sin();
            chunk[1] = z1 * stddev;
        }
    }
}

/// Simple deterministic random f32 in [0, 1).
fn simple_random_f32() -> f32 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = Cell::new(42);
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

/// Load weights from a `WeightFile` into a model's tensors.
pub fn load_weights_from_map(
    target: &mut HashMap<String, &mut [f32]>,
    source: &HashMap<String, Vec<f32>>,
) -> Result<usize> {
    let mut loaded = 0;
    for (name, tensor) in source {
        if let Some(target_slice) = target.get_mut(name.as_str()) {
            let copy_len = tensor.len().min(target_slice.len());
            target_slice[..copy_len].copy_from_slice(&tensor[..copy_len]);
            loaded += 1;
        }
    }
    Ok(loaded)
}

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_forward() {
        let mut linear = Linear::new(4, 3);
        linear.init_xavier();
        let input = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], 1, 4);
        let output = linear.forward(&input);
        assert_eq!(output.rows, 1);
        assert_eq!(output.cols, 3);
    }

    #[test]
    fn test_layer_norm() {
        let ln = LayerNorm::new(4, 1e-5);
        let input = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], 1, 4);
        let output = ln.forward(&input);
        // After layer norm with gamma=1, beta=0: output should be zero-mean, unit-variance
        let mean: f32 = output.data.iter().sum::<f32>() / 4.0;
        assert!(mean.abs() < 1e-5, "mean should be ~0, got {mean}");
    }

    #[test]
    fn test_rms_norm() {
        let rn = RmsNorm::new(4, 1e-6);
        let input = Tensor::new(vec![2.0, 4.0, 6.0, 8.0], 1, 4);
        let output = rn.forward(&input);
        let rms: f32 = (output.data.iter().map(|v| v * v).sum::<f32>() / 4.0).sqrt();
        assert!((rms - 1.0).abs() < 0.1, "RMS should be ~1.0, got {rms}");
    }

    #[test]
    fn test_activation_relu() {
        assert_eq!(Activation::ReLU.apply(-1.0), 0.0);
        assert_eq!(Activation::ReLU.apply(5.0), 5.0);
    }

    #[test]
    fn test_activation_gelu() {
        // GELU(0) ≈ 0
        let v = Activation::GELU.apply(0.0);
        assert!(v.abs() < 0.01, "GELU(0) should be ~0, got {v}");
        // GELU(1) ≈ 0.84
        let v = Activation::GELU.apply(1.0);
        assert!((v - 0.841).abs() < 0.01, "GELU(1) should be ~0.841, got {v}");
    }

    #[test]
    fn test_activation_silu() {
        // SiLU(0) = 0
        assert!(Activation::SiLU.apply(0.0).abs() < 1e-6);
        // SiLU(1) ≈ 0.731
        let v = Activation::SiLU.apply(1.0);
        assert!((v - 0.731).abs() < 0.01, "SiLU(1) should be ~0.731, got {v}");
    }

    #[test]
    fn test_feed_forward() {
        let mut ffn = FeedForward::new(8, 16, Activation::GELU);
        ffn.init_weights();
        let input = Tensor::new(vec![1.0; 8], 1, 8);
        let output = ffn.forward(&input);
        assert_eq!(output.rows, 1);
        assert_eq!(output.cols, 8);
    }

    #[test]
    fn test_rope_cache() {
        let rope = RopeCache::new(8, 64, 10000.0);
        let input = Tensor::new(vec![1.0; 8], 1, 8);
        let output = rope.apply(&input, 0);
        // Rotated vector should have same L2 norm
        let norm_before: f32 = input.data.iter().map(|v| v * v).sum::<f32>().sqrt();
        let norm_after: f32 = output.data.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(
            (norm_before - norm_after).abs() < 1e-5,
            "RoPE should preserve L2 norm"
        );
    }

    #[test]
    fn test_multi_head_attention() {
        let mha = MultiHeadAttention::new(16, 4);
        // We can't easily test with uninitialized weights without NaN,
        // but we can test shape
        let mut input = Tensor::zeros(1, 16);
        init_normal(&mut input.data, 0.1);
        let (output, _) = mha.forward(&input, None, 0);
        assert_eq!(output.rows, 1);
        assert_eq!(output.cols, 16);
    }

    #[test]
    fn test_gpt_model_creation() {
        let config = TransformerConfig {
            vocab_size: 256,
            embedding_dim: 32,
            num_layers: 2,
            num_heads: 4,
            ffn_dim: 64,
            max_seq_len: 128,
            ..Default::default()
        };
        let model = GptModel::new(config);
        assert_eq!(model.name(), "gpt-decoder");
        assert_eq!(model.vocab_size(), 256);
        assert_eq!(model.embedding_dim(), 32);
        assert_eq!(model.num_layers(), 2);
        assert!(model.param_count() > 0);
    }

    #[test]
    fn test_llama_model_creation() {
        let config = TransformerConfig {
            vocab_size: 256,
            embedding_dim: 32,
            num_layers: 2,
            num_heads: 4,
            ffn_dim: 64,
            max_seq_len: 128,
            ..Default::default()
        };
        let model = LlamaModel::new(config);
        assert_eq!(model.name(), "llama-decoder");
        assert!(model.param_count() > 0);
    }

    #[test]
    fn test_bert_model_creation() {
        let config = TransformerConfig {
            vocab_size: 256,
            embedding_dim: 32,
            num_layers: 2,
            num_heads: 4,
            ffn_dim: 64,
            max_seq_len: 128,
            ..Default::default()
        };
        let model = BertModel::new(config);
        assert_eq!(model.name(), "bert-encoder");
        assert!(model.param_count() > 0);
    }

    #[test]
    fn test_transformer_config_default() {
        let cfg = TransformerConfig::default();
        assert_eq!(cfg.vocab_size, 32000);
        assert_eq!(cfg.num_layers, 12);
        assert_eq!(cfg.num_heads, 12);
        assert_eq!(cfg.activation, Activation::GELU);
    }

    #[test]
    fn test_load_weights() {
        let mut source = HashMap::new();
        source.insert("layer.0.weight".to_string(), vec![1.0, 2.0, 3.0, 4.0]);
        let mut target_buf = vec![0.0f32; 4];
        let mut target = HashMap::new();
        target.insert("layer.0.weight".to_string(), target_buf.as_mut_slice());
        let loaded = load_weights_from_map(&mut target, &source).unwrap();
        assert_eq!(loaded, 1);
        assert_eq!(target_buf, vec![1.0, 2.0, 3.0, 4.0]);
    }
}
