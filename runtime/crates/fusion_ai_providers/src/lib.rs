//! AI provider and model registry for Fusion v2.0 Vortex.
//!
//! Supports 26 model providers with native integration and Ollama fallback.
//! Provides a unified interface for model discovery, registration, and inference.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;


// ─── Errors ──────────────────────────────────────────────────────────────────

/// Errors that can occur in the AI provider registry.
#[derive(Error, Debug, Clone)]
pub enum ProviderError {
    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("provider not supported: {0}")]
    ProviderNotSupported(String),

    #[error("Ollama API error: {0}")]
    OllamaError(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("model already registered: {0}")]
    AlreadyRegistered(String),

    #[error("inference failed: {0}")]
    InferenceFailed(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(e: reqwest::Error) -> Self {
        ProviderError::NetworkError(e.to_string())
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(e: serde_json::Error) -> Self {
        ProviderError::SerializationError(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ProviderError>;

// ─── Model Provider Enum ─────────────────────────────────────────────────────

/// Supported AI model providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelProvider {
    DeepSeek,
    Llama4,
    Qwen3,
    Gemma4,
    GLM,
    Mistral,
    KimiK2,
    Nemotron3,
    Phi,
    Inkling,
    OLMo,
    Qwen3Coder,
    DeepSeekV3_2,
    GptOss120b,
    Codestral,
    StarCoder2,
    NousCoder,
    OmniCoder,
    Lance,
    MiniCPM_V,
    Falcon3,
    YuLan,
    BLOOM,
    BLOOMZ,
    Pythia,
    EuroLLM,
}

impl ModelProvider {
    /// Returns all supported providers.
    pub fn all() -> &'static [ModelProvider] {
        &[
            Self::DeepSeek,
            Self::Llama4,
            Self::Qwen3,
            Self::Gemma4,
            Self::GLM,
            Self::Mistral,
            Self::KimiK2,
            Self::Nemotron3,
            Self::Phi,
            Self::Inkling,
            Self::OLMo,
            Self::Qwen3Coder,
            Self::DeepSeekV3_2,
            Self::GptOss120b,
            Self::Codestral,
            Self::StarCoder2,
            Self::NousCoder,
            Self::OmniCoder,
            Self::Lance,
            Self::MiniCPM_V,
            Self::Falcon3,
            Self::YuLan,
            Self::BLOOM,
            Self::BLOOMZ,
            Self::Pythia,
            Self::EuroLLM,
        ]
    }

    /// Returns the provider's home page or organization.
    pub fn organization(&self) -> &'static str {
        match self {
            Self::DeepSeek => "DeepSeek AI",
            Self::Llama4 => "Meta AI",
            Self::Qwen3 => "Alibaba Cloud",
            Self::Gemma4 => "Google DeepMind",
            Self::GLM => "Zhipu AI",
            Self::Mistral => "Mistral AI",
            Self::KimiK2 => "Moonshot AI",
            Self::Nemotron3 => "NVIDIA",
            Self::Phi => "Microsoft Research",
            Self::Inkling => "Inkling AI",
            Self::OLMo => "Allen Institute for AI",
            Self::Qwen3Coder => "Alibaba Cloud",
            Self::DeepSeekV3_2 => "DeepSeek AI",
            Self::GptOss120b => "OpenAI OSS",
            Self::Codestral => "Mistral AI",
            Self::StarCoder2 => "BigCode",
            Self::NousCoder => "Nous Research",
            Self::OmniCoder => "OmniCoder Team",
            Self::Lance => "Lance AI",
            Self::MiniCPM_V => "OpenBMB",
            Self::Falcon3 => "TII",
            Self::YuLan => "Renmin University",
            Self::BLOOM => "BigScience",
            Self::BLOOMZ => "BigScience",
            Self::Pythia => "EleutherAI",
            Self::EuroLLM => "EURO-LLM Consortium",
        }
    }

    /// Returns the Ollama model name for this provider.
    pub fn ollama_name(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek-coder:33b",
            Self::Llama4 => "llama4:latest",
            Self::Qwen3 => "qwen3:32b",
            Self::Gemma4 => "gemma4:latest",
            Self::GLM => "glm4:latest",
            Self::Mistral => "mistral:latest",
            Self::KimiK2 => "kimi-k2:latest",
            Self::Nemotron3 => "nemotron3:latest",
            Self::Phi => "phi4:latest",
            Self::Inkling => "inkling:latest",
            Self::OLMo => "olmo2:latest",
            Self::Qwen3Coder => "qwen3-coder:latest",
            Self::DeepSeekV3_2 => "deepseek-v3.2:latest",
            Self::GptOss120b => "gpt-oss-120b:latest",
            Self::Codestral => "codestral:latest",
            Self::StarCoder2 => "starcoder2:latest",
            Self::NousCoder => "nous-coder:latest",
            Self::OmniCoder => "omnicoder:latest",
            Self::Lance => "lance:latest",
            Self::MiniCPM_V => "minicpm-v:latest",
            Self::Falcon3 => "falcon3:latest",
            Self::YuLan => "yulan:latest",
            Self::BLOOM => "bloom:latest",
            Self::BLOOMZ => "bloomz:latest",
            Self::Pythia => "pythia:latest",
            Self::EuroLLM => "eurollm:latest",
        }
    }

    /// Attempts to parse a provider from a string name (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        let lower = s.to_lowercase().replace('-', "_").replace(' ', "_");
        match lower.as_str() {
            "deepseek" => Some(Self::DeepSeek),
            "llama4" | "llama_4" => Some(Self::Llama4),
            "qwen3" | "qwen_3" => Some(Self::Qwen3),
            "gemma4" | "gemma_4" => Some(Self::Gemma4),
            "glm" => Some(Self::GLM),
            "mistral" => Some(Self::Mistral),
            "kimi_k2" | "kimi" => Some(Self::KimiK2),
            "nemotron3" | "nemotron_3" => Some(Self::Nemotron3),
            "phi" => Some(Self::Phi),
            "inkling" => Some(Self::Inkling),
            "olmo" => Some(Self::OLMo),
            "qwen3_coder" | "qwen_3_coder" => Some(Self::Qwen3Coder),
            "deepseek_v3_2" | "deepseek_v3.2" | "deepseekv3_2" => Some(Self::DeepSeekV3_2),
            "gpt_oss_120b" | "gpt_oss120b" | "gptoss120b" => Some(Self::GptOss120b),
            "codestral" => Some(Self::Codestral),
            "starcoder2" | "star_coder2" => Some(Self::StarCoder2),
            "nous_coder" => Some(Self::NousCoder),
            "omnicoder" | "omni_coder" => Some(Self::OmniCoder),
            "lance" => Some(Self::Lance),
            "minicpm_v" | "minicpmv" => Some(Self::MiniCPM_V),
            "falcon3" | "falcon_3" => Some(Self::Falcon3),
            "yulan" => Some(Self::YuLan),
            "bloom" => Some(Self::BLOOM),
            "bloomz" => Some(Self::BLOOMZ),
            "pythia" => Some(Self::Pythia),
            "eurollm" | "eu_llm" => Some(Self::EuroLLM),
            _ => None,
        }
    }
}

impl fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::DeepSeek => "DeepSeek",
            Self::Llama4 => "Llama 4",
            Self::Qwen3 => "Qwen 3",
            Self::Gemma4 => "Gemma 4",
            Self::GLM => "GLM",
            Self::Mistral => "Mistral",
            Self::KimiK2 => "Kimi K2",
            Self::Nemotron3 => "Nemotron 3",
            Self::Phi => "Phi",
            Self::Inkling => "Inkling",
            Self::OLMo => "OLMo",
            Self::Qwen3Coder => "Qwen 3 Coder",
            Self::DeepSeekV3_2 => "DeepSeek V3.2",
            Self::GptOss120b => "GPT-OSS 120B",
            Self::Codestral => "Codestral",
            Self::StarCoder2 => "StarCoder 2",
            Self::NousCoder => "Nous Coder",
            Self::OmniCoder => "OmniCoder",
            Self::Lance => "Lance",
            Self::MiniCPM_V => "MiniCPM-V",
            Self::Falcon3 => "Falcon 3",
            Self::YuLan => "YuLan",
            Self::BLOOM => "BLOOM",
            Self::BLOOMZ => "BLOOMZ",
            Self::Pythia => "Pythia",
            Self::EuroLLM => "EuroLLM",
        };
        write!(f, "{}", name)
    }
}

// ─── Modality ────────────────────────────────────────────────────────────────

/// Input/output modalities a model supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    Text,
    Code,
    Image,
    Audio,
    Video,
    Multimodal,
}

impl Modality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Code => "code",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Multimodal => "multimodal",
        }
    }
}

impl fmt::Display for Modality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ─── Quantization ────────────────────────────────────────────────────────────

/// Quantization level for a model variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quantization {
    FP32,
    FP16,
    BF16,
    INT8,
    INT4,
    GGUF_Q4_0,
    GGUF_Q4_K_M,
    GGUF_Q5_K_M,
    GGUF_Q8_0,
    GPTQ,
    AWQ,
    EXL2,
    None,
}

impl Quantization {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FP32 => "FP32",
            Self::FP16 => "FP16",
            Self::BF16 => "BF16",
            Self::INT8 => "INT8",
            Self::INT4 => "INT4",
            Self::GGUF_Q4_0 => "GGUF_Q4_0",
            Self::GGUF_Q4_K_M => "GGUF_Q4_K_M",
            Self::GGUF_Q5_K_M => "GGUF_Q5_K_M",
            Self::GGUF_Q8_0 => "GGUF_Q8_0",
            Self::GPTQ => "GPTQ",
            Self::AWQ => "AWQ",
            Self::EXL2 => "EXL2",
            Self::None => "None",
        }
    }

    /// Returns approximate bits-per-weight for memory estimation.
    pub fn bits_per_weight(&self) -> f64 {
        match self {
            Self::FP32 => 32.0,
            Self::FP16 | Self::BF16 => 16.0,
            Self::INT8 => 8.0,
            Self::INT4 | Self::GGUF_Q4_0 | Self::GGUF_Q4_K_M | Self::GGUF_Q5_K_M | Self::GPTQ | Self::AWQ | Self::EXL2 => {
                4.5
            }
            Self::GGUF_Q8_0 => 8.5,
            Self::None => 16.0,
        }
    }
}

impl fmt::Display for Quantization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ─── Model Info ──────────────────────────────────────────────────────────────

/// Core model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: ModelProvider,
    pub params_b: f64,
    pub context_window: u32,
    pub modalities: Vec<Modality>,
    pub quantization: Quantization,
}

impl ModelInfo {
    pub fn new(
        name: impl Into<String>,
        provider: ModelProvider,
        params_b: f64,
        context_window: u32,
        modalities: Vec<Modality>,
        quantization: Quantization,
    ) -> Self {
        Self {
            name: name.into(),
            provider,
            params_b,
            context_window,
            modalities,
            quantization,
        }
    }

    /// Estimated VRAM requirement in GB.
    pub fn estimated_vram_gb(&self) -> f64 {
        let params = self.params_b * 1e9;
        let bpw = self.quantization.bits_per_weight();
        let bytes = params * bpw / 8.0;
        let overhead = 1.1; // 10% overhead for KV cache
        (bytes / 1e9) * overhead
    }

    /// Returns true if the model supports the given modality.
    pub fn supports(&self, modality: Modality) -> bool {
        self.modalities.contains(&modality)
    }
}

impl fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}, {:.1}B params, {} ctx, {} quant)",
            self.name,
            self.provider,
            self.params_b,
            self.context_window,
            self.quantization
        )
    }
}

// ─── Model Entry ─────────────────────────────────────────────────────────────

/// Complete model entry with metadata and runtime state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub info: ModelInfo,
    pub description: String,
    pub license: String,
    pub ollama_tag: String,
    pub native_support: bool,
    pub api_endpoint: Option<String>,
    pub capabilities: Vec<String>,
}

impl ModelEntry {
    pub fn new(info: ModelInfo, description: impl Into<String>, license: impl Into<String>) -> Self {
        let ollama_tag = info.provider.ollama_name().to_string();
        Self {
            info,
            description: description.into(),
            license: license.into(),
            ollama_tag,
            native_support: false,
            api_endpoint: None,
            capabilities: Vec::new(),
        }
    }

    pub fn with_native_support(mut self) -> Self {
        self.native_support = true;
        self
    }

    pub fn with_api_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.api_endpoint = Some(endpoint.into());
        self
    }

    pub fn with_capabilities(mut self, caps: Vec<String>) -> Self {
        self.capabilities = caps;
        self
    }
}

// ─── Ollama API Types ────────────────────────────────────────────────────────

/// Response from Ollama's /api/tags endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: OllamaModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelDetails {
    pub parameter_size: String,
    pub quantization_level: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub format: String,
}

/// Request body for Ollama's /api/generate endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
}

/// Response from Ollama's /api/generate endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

/// Request body for Ollama's /api/pull endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaPullRequest {
    pub name: String,
    #[serde(default)]
    pub insecure: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Response from Ollama's /api/pull endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaPullResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<u64>,
}

// ─── Ollama Client ───────────────────────────────────────────────────────────

/// Client for interacting with a local Ollama instance.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    http: Client,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new("http://localhost:11434")
    }
}

impl OllamaClient {
    /// Creates a new Ollama client pointing to the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("failed to build HTTP client");
        Self {
            base_url: base_url.into(),
            http,
        }
    }

    /// Creates a client with a custom timeout.
    pub fn with_timeout(base_url: impl Into<String>, timeout_secs: u64) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("failed to build HTTP client");
        Self {
            base_url: base_url.into(),
            http,
        }
    }

    /// Returns the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Checks if Ollama is running and reachable.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/version", self.base_url);
        match self.http.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Lists all locally available models.
    pub async fn list(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let body: serde_json::Value = resp.json().await?;

        let models = body["models"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    /// Pulls (downloads) a model from the Ollama registry.
    pub async fn pull(&self, model_name: &str) -> Result<Vec<OllamaPullResponse>> {
        let url = format!("{}/api/pull", self.base_url);
        let req = OllamaPullRequest {
            name: model_name.to_string(),
            insecure: false,
            stream: Some(true),
        };

        let resp = self.http.post(&url).json(&req).send().await?;
        let full_text = resp.text().await?;

        let mut results = Vec::new();
        for line in full_text.lines() {
            if !line.trim().is_empty() {
                if let Ok(parsed) = serde_json::from_str::<OllamaPullResponse>(line) {
                    results.push(parsed);
                }
            }
        }

        if results.is_empty() && !full_text.is_empty() {
            results.push(OllamaPullResponse {
                status: full_text,
                digest: None,
                total: None,
                completed: None,
            });
        }

        Ok(results)
    }

    /// Generates a completion from a model.
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        system: Option<&str>,
        options: Option<OllamaOptions>,
    ) -> Result<OllamaGenerateResponse> {
        let url = format!("{}/api/generate", self.base_url);
        let req = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.map(String::from),
            template: None,
            context: None,
            stream: false,
            options,
        };

        let resp = self.http.post(&url).json(&req).send().await?;
        let body: OllamaGenerateResponse = resp.json().await?;
        Ok(body)
    }

    /// Generates a completion from a model using a ModelProvider.
    pub async fn generate_with_provider(
        &self,
        provider: ModelProvider,
        prompt: &str,
        system: Option<&str>,
    ) -> Result<OllamaGenerateResponse> {
        self.generate(provider.ollama_name(), prompt, system, None)
            .await
    }

    /// Checks if a specific model is available locally.
    pub async fn is_model_available(&self, model_name: &str) -> Result<bool> {
        let models = self.list().await?;
        Ok(models.iter().any(|m| {
            m.name == model_name
                || m.model == model_name
                || m.name.starts_with(&format!("{}:", model_name))
        }))
    }
}

// ─── Inference Backend ───────────────────────────────────────────────────────

/// Where to route inference for a model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceBackend {
    Native,
    Ollama,
    Api(String),
}

// ─── Inference Engine ────────────────────────────────────────────────────────

/// Routes inference requests to native or Ollama backends.
#[derive(Debug, Clone)]
pub struct InferenceEngine {
    ollama: OllamaClient,
    backend_map: HashMap<ModelProvider, InferenceBackend>,
}

impl InferenceEngine {
    pub fn new(ollama: OllamaClient) -> Self {
        Self {
            ollama,
            backend_map: HashMap::new(),
        }
    }

    /// Sets the inference backend for a provider.
    pub fn set_backend(&mut self, provider: ModelProvider, backend: InferenceBackend) {
        self.backend_map.insert(provider, backend);
    }

    /// Gets the current backend for a provider (defaults to Ollama).
    pub fn get_backend(&self, provider: &ModelProvider) -> &InferenceBackend {
        self.backend_map
            .get(provider)
            .unwrap_or(&InferenceBackend::Ollama)
    }

    /// Runs inference for the given provider and prompt.
    pub async fn infer(
        &self,
        provider: ModelProvider,
        prompt: &str,
        system: Option<&str>,
    ) -> Result<String> {
        let backend = self.get_backend(&provider).clone();

        match backend {
            InferenceBackend::Native => {
                log::info!("routing to native backend for {}", provider);
                Err(ProviderError::InferenceFailed(format!(
                    "native backend not implemented for {}",
                    provider
                )))
            }
            InferenceBackend::Ollama => {
                log::info!("routing to Ollama for {}", provider);
                let resp = self
                    .ollama
                    .generate_with_provider(provider, prompt, system)
                    .await?;
                Ok(resp.response)
            }
            InferenceBackend::Api(endpoint) => {
                log::info!("routing to API endpoint {} for {}", endpoint, provider);
                Err(ProviderError::InferenceFailed(format!(
                    "API backend not yet implemented: {}",
                    endpoint
                )))
            }
        }
    }

    /// Generates a response for a specific model name via Ollama.
    pub async fn infer_model(
        &self,
        model_name: &str,
        prompt: &str,
        system: Option<&str>,
    ) -> Result<String> {
        let resp = self
            .ollama
            .generate(model_name, prompt, system, None)
            .await?;
        Ok(resp.response)
    }

    /// Checks which configured providers are currently reachable.
    pub async fn check_health(&self) -> HashMap<ModelProvider, bool> {
        let mut results = HashMap::new();
        let ollama_ok = self.ollama.health_check().await.unwrap_or(false);

        for provider in ModelProvider::all() {
            let backend = self.get_backend(provider);
            let ok = match backend {
                InferenceBackend::Ollama => ollama_ok,
                InferenceBackend::Native => true,
                InferenceBackend::Api(_) => false,
            };
            results.insert(*provider, ok);
        }

        results
    }

    /// Returns the Ollama client.
    pub fn ollama_client(&self) -> &OllamaClient {
        &self.ollama
    }
}

// ─── Model Registry ──────────────────────────────────────────────────────────

/// Central registry for all known models.
#[derive(Debug)]
pub struct ModelRegistry {
    models: HashMap<String, ModelEntry>,
    by_provider: HashMap<ModelProvider, Vec<String>>,
    inference: Option<Arc<InferenceEngine>>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            by_provider: HashMap::new(),
            inference: None,
        }
    }

    /// Creates a registry pre-populated with all built-in models.
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();
        reg.register_builtins();
        reg
    }

    /// Sets the inference engine for this registry.
    pub fn set_inference_engine(&mut self, engine: Arc<InferenceEngine>) {
        self.inference = Some(engine);
    }

    /// Registers a model entry. Returns an error if the name is already taken.
    pub fn register(&mut self, entry: ModelEntry) -> Result<()> {
        let name = entry.info.name.clone();
        if self.models.contains_key(&name) {
            return Err(ProviderError::AlreadyRegistered(name));
        }

        self.by_provider
            .entry(entry.info.provider)
            .or_default()
            .push(name.clone());
        self.models.insert(name, entry);
        Ok(())
    }

    /// Registers a model, overwriting any existing entry with the same name.
    pub fn register_or_update(&mut self, entry: ModelEntry) {
        let name = entry.info.name.clone();
        let provider = entry.info.provider;

        if let Some(old) = self.models.get(&name) {
            if let Some(vec) = self.by_provider.get_mut(&old.info.provider) {
                vec.retain(|n| n != &name);
            }
        }

        self.by_provider
            .entry(provider)
            .or_default()
            .push(name.clone());
        self.models.insert(name, entry);
    }

    /// Returns a model by exact name.
    pub fn get_model(&self, name: &str) -> Option<&ModelEntry> {
        self.models.get(name)
    }

    /// Returns a mutable reference to a model by name.
    pub fn get_model_mut(&mut self, name: &str) -> Option<&mut ModelEntry> {
        self.models.get_mut(name)
    }

    /// Lists all registered models.
    pub fn list_models(&self) -> Vec<&ModelEntry> {
        self.models.values().collect()
    }

    /// Lists all model names.
    pub fn list_model_names(&self) -> Vec<&str> {
        self.models.keys().map(|s| s.as_str()).collect()
    }

    /// Returns models for a specific provider.
    pub fn models_by_provider(&self, provider: ModelProvider) -> Vec<&ModelEntry> {
        self.by_provider
            .get(&provider)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|n| self.models.get(n))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Searches models by name substring (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&ModelEntry> {
        let lower = query.to_lowercase();
        self.models
            .values()
            .filter(|e| e.info.name.to_lowercase().contains(&lower))
            .collect()
    }

    /// Returns models that support a given modality.
    pub fn models_with_modality(&self, modality: Modality) -> Vec<&ModelEntry> {
        self.models
            .values()
            .filter(|e| e.info.supports(modality))
            .collect()
    }

    /// Returns models within a parameter count range.
    pub fn models_by_size(&self, min_b: f64, max_b: f64) -> Vec<&ModelEntry> {
        self.models
            .values()
            .filter(|e| e.info.params_b >= min_b && e.info.params_b <= max_b)
            .collect()
    }

    /// Returns models with at least a given context window size.
    pub fn models_with_context(&self, min_ctx: u32) -> Vec<&ModelEntry> {
        self.models
            .values()
            .filter(|e| e.info.context_window >= min_ctx)
            .collect()
    }

    /// Returns the number of registered models.
    pub fn count(&self) -> usize {
        self.models.len()
    }

    /// Returns the number of registered providers.
    pub fn provider_count(&self) -> usize {
        self.by_provider.len()
    }

    /// Removes a model by name. Returns the removed entry if it existed.
    pub fn remove(&mut self, name: &str) -> Option<ModelEntry> {
        if let Some(entry) = self.models.remove(name) {
            if let Some(vec) = self.by_provider.get_mut(&entry.info.provider) {
                vec.retain(|n| n != name);
                if vec.is_empty() {
                    self.by_provider.remove(&entry.info.provider);
                }
            }
            Some(entry)
        } else {
            None
        }
    }

    /// Returns the inference engine if configured.
    pub fn inference_engine(&self) -> Option<&InferenceEngine> {
        self.inference.as_deref()
    }

    /// Returns all registered providers.
    pub fn registered_providers(&self) -> Vec<ModelProvider> {
        self.by_provider.keys().copied().collect()
    }

    /// Returns the total estimated VRAM for all registered models.
    pub fn total_estimated_vram_gb(&self) -> f64 {
        self.models
            .values()
            .map(|e| e.info.estimated_vram_gb())
            .sum()
    }

    /// Finds the smallest model that supports the given modality.
    pub fn smallest_model_with_modality(&self, modality: Modality) -> Option<&ModelEntry> {
        self.models_with_modality(modality)
            .into_iter()
            .min_by(|a, b| a.info.params_b.partial_cmp(&b.info.params_b).unwrap())
    }

    /// Finds the model with the largest context window.
    pub fn largest_context_model(&self) -> Option<&ModelEntry> {
        self.models
            .values()
            .max_by_key(|e| e.info.context_window)
    }

    /// Registers all built-in model entries.
    fn register_builtins(&mut self) {
        let builtins = create_builtin_models();
        for entry in builtins {
            let name = entry.info.name.clone();
            let provider = entry.info.provider;
            self.by_provider
                .entry(provider)
                .or_default()
                .push(name.clone());
            self.models.insert(name, entry);
        }
    }
}

// ─── Built-in Model Definitions ──────────────────────────────────────────────

/// Creates all built-in model entries with full metadata.
fn create_builtin_models() -> Vec<ModelEntry> {
    vec![
        // DeepSeek
        ModelEntry::new(
            ModelInfo::new(
                "deepseek-coder-33b",
                ModelProvider::DeepSeek,
                33.0,
                16_384,
                vec![Modality::Code, Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "DeepSeek Coder 33B — top-tier open code generation model",
            "DeepSeek License",
        )
        .with_native_support()
        .with_capabilities(vec![
            "code-generation".into(),
            "infilling".into(),
            "long-context".into(),
        ]),
        ModelEntry::new(
            ModelInfo::new(
                "deepseek-v3.2",
                ModelProvider::DeepSeekV3_2,
                671.0,
                131_072,
                vec![Modality::Text, Modality::Code],
                Quantization::FP16,
            ),
            "DeepSeek V3.2 — 671B MoE model with top benchmarks",
            "DeepSeek License",
        )
        .with_capabilities(vec![
            "moe".into(),
            "reasoning".into(),
            "code-generation".into(),
        ]),

        // Llama 4
        ModelEntry::new(
            ModelInfo::new(
                "llama-4-405b",
                ModelProvider::Llama4,
                405.0,
                131_072,
                vec![Modality::Text, Modality::Multimodal],
                Quantization::FP16,
            ),
            "Meta Llama 4 405B — flagship multimodal model",
            "Llama 4 Community License",
        )
        .with_native_support()
        .with_capabilities(vec![
            "multimodal".into(),
            "reasoning".into(),
            "instruction-following".into(),
        ]),

        // Qwen 3
        ModelEntry::new(
            ModelInfo::new(
                "qwen3-32b",
                ModelProvider::Qwen3,
                32.0,
                131_072,
                vec![Modality::Text, Modality::Code],
                Quantization::GGUF_Q4_K_M,
            ),
            "Qwen 3 32B — strong multilingual model with code abilities",
            "Apache 2.0",
        )
        .with_native_support()
        .with_capabilities(vec![
            "multilingual".into(),
            "code-generation".into(),
            "tool-use".into(),
        ]),

        // Gemma 4
        ModelEntry::new(
            ModelInfo::new(
                "gemma4-27b",
                ModelProvider::Gemma4,
                27.0,
                131_072,
                vec![Modality::Text, Modality::Code],
                Quantization::BF16,
            ),
            "Google Gemma 4 27B — compact high-performance model",
            "Gemma License",
        )
        .with_native_support()
        .with_capabilities(vec![
            "instruction-following".into(),
            "summarization".into(),
            "retrieval".into(),
        ]),

        // GLM
        ModelEntry::new(
            ModelInfo::new(
                "glm-4-9b",
                ModelProvider::GLM,
                9.0,
                128_000,
                vec![Modality::Text, Modality::Code],
                Quantization::GGUF_Q4_K_M,
            ),
            "Zhipu GLM-4 9B — efficient Chinese-English bilingual model",
            "GLM-4 License",
        )
        .with_capabilities(vec![
            "bilingual".into(),
            "tool-use".into(),
            "agent".into(),
        ]),

        // Mistral
        ModelEntry::new(
            ModelInfo::new(
                "mistral-large",
                ModelProvider::Mistral,
                123.0,
                128_000,
                vec![Modality::Text, Modality::Code],
                Quantization::FP16,
            ),
            "Mistral Large — flagship model with strong reasoning",
            "Apache 2.0",
        )
        .with_native_support()
        .with_capabilities(vec![
            "reasoning".into(),
            "code-generation".into(),
            "instruction-following".into(),
        ]),

        // Kimi K2
        ModelEntry::new(
            ModelInfo::new(
                "kimi-k2-1t",
                ModelProvider::KimiK2,
                1000.0,
                1_000_000,
                vec![Modality::Text, Modality::Code],
                Quantization::FP16,
            ),
            "Moonshot Kimi K2 — 1T MoE with 1M context",
            "Kimi License",
        )
        .with_capabilities(vec![
            "moe".into(),
            "long-context".into(),
            "reasoning".into(),
        ]),

        // Nemotron 3
        ModelEntry::new(
            ModelInfo::new(
                "nemotron3-8b",
                ModelProvider::Nemotron3,
                8.0,
                4_096,
                vec![Modality::Text],
                Quantization::FP16,
            ),
            "NVIDIA Nemotron 3 8B — compact inference-optimized model",
            "NVIDIA License",
        )
        .with_capabilities(vec![
            "inference-optimized".into(),
            "tool-use".into(),
        ]),

        // Phi
        ModelEntry::new(
            ModelInfo::new(
                "phi-4-14b",
                ModelProvider::Phi,
                14.0,
                16_384,
                vec![Modality::Text, Modality::Code],
                Quantization::GGUF_Q4_K_M,
            ),
            "Microsoft Phi-4 14B — reasoning-focused small model",
            "MIT License",
        )
        .with_native_support()
        .with_capabilities(vec![
            "reasoning".into(),
            "math".into(),
            "code-generation".into(),
        ]),

        // Inkling
        ModelEntry::new(
            ModelInfo::new(
                "inkling-8b",
                ModelProvider::Inkling,
                8.0,
                8_192,
                vec![Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "Inkling 8B — creative writing and storytelling model",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "creative-writing".into(),
            "storytelling".into(),
        ]),

        // OLMo
        ModelEntry::new(
            ModelInfo::new(
                "olmo2-32b",
                ModelProvider::OLMo,
                32.0,
                4_096,
                vec![Modality::Text, Modality::Code],
                Quantization::GGUF_Q4_K_M,
            ),
            "Allen AI OLMo 2 32B — fully open model with training data",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "fully-open".into(),
            "reproducible".into(),
            "research".into(),
        ]),

        // Qwen 3 Coder
        ModelEntry::new(
            ModelInfo::new(
                "qwen3-coder-32b",
                ModelProvider::Qwen3Coder,
                32.0,
                256_000,
                vec![Modality::Code, Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "Qwen 3 Coder 32B — specialized code generation model",
            "Apache 2.0",
        )
        .with_native_support()
        .with_capabilities(vec![
            "code-generation".into(),
            "long-context".into(),
            "multi-language".into(),
        ]),

        // GPT-OSS 120B
        ModelEntry::new(
            ModelInfo::new(
                "gpt-oss-120b",
                ModelProvider::GptOss120b,
                120.0,
                128_000,
                vec![Modality::Text, Modality::Code],
                Quantization::FP16,
            ),
            "GPT-OSS 120B — large-scale open GPT architecture model",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "reasoning".into(),
            "code-generation".into(),
            "instruction-following".into(),
        ]),

        // Codestral
        ModelEntry::new(
            ModelInfo::new(
                "codestral-22b",
                ModelProvider::Codestral,
                22.0,
                32_768,
                vec![Modality::Code],
                Quantization::GGUF_Q4_K_M,
            ),
            "Mistral Codestral 22B — code-specialized model",
            "Apache 2.0",
        )
        .with_native_support()
        .with_capabilities(vec![
            "code-generation".into(),
            "fill-in-middle".into(),
            "code-explanation".into(),
        ]),

        // StarCoder 2
        ModelEntry::new(
            ModelInfo::new(
                "starcoder2-15b",
                ModelProvider::StarCoder2,
                15.0,
                16_384,
                vec![Modality::Code],
                Quantization::GGUF_Q4_K_M,
            ),
            "BigCode StarCoder 2 15B — trained on 3.3T tokens of code",
            "BigCode OpenRAIL-M",
        )
        .with_capabilities(vec![
            "code-generation".into(),
            "619 languages".into(),
            "fill-in-middle".into(),
        ]),

        // Nous Coder
        ModelEntry::new(
            ModelInfo::new(
                "nous-coder-34b",
                ModelProvider::NousCoder,
                34.0,
                32_768,
                vec![Modality::Code],
                Quantization::GGUF_Q4_K_M,
            ),
            "Nous Research Coder 34B — fine-tuned for code tasks",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "code-generation".into(),
            "refactoring".into(),
            "debugging".into(),
        ]),

        // OmniCoder
        ModelEntry::new(
            ModelInfo::new(
                "omnicoder-13b",
                ModelProvider::OmniCoder,
                13.0,
                16_384,
                vec![Modality::Code, Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "OmniCoder 13B — versatile code and text model",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "code-generation".into(),
            "explanation".into(),
            "documentation".into(),
        ]),

        // Lance
        ModelEntry::new(
            ModelInfo::new(
                "lance-7b",
                ModelProvider::Lance,
                7.0,
                8_192,
                vec![Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "Lance 7B — fast inference model for embeddings and retrieval",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "embeddings".into(),
            "retrieval".into(),
            "fast-inference".into(),
        ]),

        // MiniCPM-V
        ModelEntry::new(
            ModelInfo::new(
                "minicpm-v-8b",
                ModelProvider::MiniCPM_V,
                8.0,
                4_096,
                vec![Modality::Text, Modality::Image, Modality::Multimodal],
                Quantization::GGUF_Q4_K_M,
            ),
            "OpenBMB MiniCPM-V 8B — small multimodal model",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "vision".into(),
            "ocr".into(),
            "image-understanding".into(),
        ]),

        // Falcon 3
        ModelEntry::new(
            ModelInfo::new(
                "falcon3-10b",
                ModelProvider::Falcon3,
                10.0,
                8_192,
                vec![Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "TII Falcon 3 10B — fast inference model",
            "Falcon License",
        )
        .with_capabilities(vec![
            "fast-inference".into(),
            "multilingual".into(),
            "chat".into(),
        ]),

        // YuLan
        ModelEntry::new(
            ModelInfo::new(
                "yulan-13b",
                ModelProvider::YuLan,
                13.0,
                8_192,
                vec![Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "YuLan 13B — Chinese dialogue model from Renmin University",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "chinese-dialogue".into(),
            "instruction-following".into(),
        ]),

        // BLOOM
        ModelEntry::new(
            ModelInfo::new(
                "bloom-176b",
                ModelProvider::BLOOM,
                176.0,
                2_048,
                vec![Modality::Text],
                Quantization::INT4,
            ),
            "BigScience BLOOM 176B — multilingual language model trained on 46 languages",
            "BigScience OpenRAIL-M",
        )
        .with_capabilities(vec![
            "multilingual-46-languages".into(),
            "text-generation".into(),
            "few-shot".into(),
        ]),

        // BLOOMZ
        ModelEntry::new(
            ModelInfo::new(
                "bloomz-176b",
                ModelProvider::BLOOMZ,
                176.0,
                2_048,
                vec![Modality::Text],
                Quantization::INT4,
            ),
            "BigScience BLOOMZ 176B — instruction-tuned BLOOM",
            "BigScience OpenRAIL-M",
        )
        .with_capabilities(vec![
            "instruction-following".into(),
            "multilingual-46-languages".into(),
            "zero-shot".into(),
        ]),

        // Pythia
        ModelEntry::new(
            ModelInfo::new(
                "pythia-12b",
                ModelProvider::Pythia,
                12.0,
                2_048,
                vec![Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "EleutherAI Pythia 12B — designed for research and reproducibility",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "research".into(),
            "reproducible".into(),
            "training-dynamics".into(),
        ]),

        // EuroLLM
        ModelEntry::new(
            ModelInfo::new(
                "eurollm-9b",
                ModelProvider::EuroLLM,
                9.0,
                8_192,
                vec![Modality::Text],
                Quantization::GGUF_Q4_K_M,
            ),
            "EURO-LLM 9B — European languages focused language model",
            "Apache 2.0",
        )
        .with_capabilities(vec![
            "european-languages".into(),
            "multilingual".into(),
            "compliant".into(),
        ]),
    ]
}

// ─── Model Search Filters ────────────────────────────────────────────────────

/// Builder for composing model search queries.
pub struct ModelFilter {
    provider: Option<ModelProvider>,
    modality: Option<Modality>,
    min_params: Option<f64>,
    max_params: Option<f64>,
    min_context: Option<u32>,
    native_only: bool,
    name_query: Option<String>,
}

impl ModelFilter {
    pub fn new() -> Self {
        Self {
            provider: None,
            modality: None,
            min_params: None,
            max_params: None,
            min_context: None,
            native_only: false,
            name_query: None,
        }
    }

    pub fn provider(mut self, p: ModelProvider) -> Self {
        self.provider = Some(p);
        self
    }

    pub fn modality(mut self, m: Modality) -> Self {
        self.modality = Some(m);
        self
    }

    pub fn min_params(mut self, min: f64) -> Self {
        self.min_params = Some(min);
        self
    }

    pub fn max_params(mut self, max: f64) -> Self {
        self.max_params = Some(max);
        self
    }

    pub fn min_context(mut self, min: u32) -> Self {
        self.min_context = Some(min);
        self
    }

    pub fn native_only(mut self) -> Self {
        self.native_only = true;
        self
    }

    pub fn name(mut self, query: impl Into<String>) -> Self {
        self.name_query = Some(query.into());
        self
    }

    /// Applies this filter to a registry and returns matching entries.
    pub fn apply<'a>(&self, registry: &'a ModelRegistry) -> Vec<&'a ModelEntry> {
        registry
            .list_models()
            .into_iter()
            .filter(|e| {
                if let Some(p) = self.provider {
                    if e.info.provider != p {
                        return false;
                    }
                }
                if let Some(m) = self.modality {
                    if !e.info.supports(m) {
                        return false;
                    }
                }
                if let Some(min) = self.min_params {
                    if e.info.params_b < min {
                        return false;
                    }
                }
                if let Some(max) = self.max_params {
                    if e.info.params_b > max {
                        return false;
                    }
                }
                if let Some(min) = self.min_context {
                    if e.info.context_window < min {
                        return false;
                    }
                }
                if self.native_only && !e.native_support {
                    return false;
                }
                if let Some(ref q) = self.name_query {
                    if !e.info.name.to_lowercase().contains(&q.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

impl Default for ModelFilter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Statistics ──────────────────────────────────────────────────────────────

/// Aggregate statistics about the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_models: usize,
    pub total_providers: usize,
    pub models_by_provider: HashMap<String, usize>,
    pub modality_counts: HashMap<String, usize>,
    pub total_vram_gb: f64,
    pub min_params: f64,
    pub max_params: f64,
    pub avg_params: f64,
    pub min_context: u32,
    pub max_context: u32,
}

impl RegistryStats {
    /// Computes statistics from a registry.
    pub fn from_registry(reg: &ModelRegistry) -> Self {
        let models = reg.list_models();

        let mut models_by_provider: HashMap<String, usize> = HashMap::new();
        let mut modality_counts: HashMap<String, usize> = HashMap::new();
        let mut total_vram = 0.0;
        let mut params_sum = 0.0;
        let mut min_params = f64::MAX;
        let mut max_params: f64 = 0.0;
        let mut min_ctx = u32::MAX;
        let mut max_ctx = 0u32;

        for entry in &models {
            let pname = entry.info.provider.to_string();
            *models_by_provider.entry(pname).or_insert(0) += 1;

            for m in &entry.info.modalities {
                *modality_counts.entry(m.as_str().to_string()).or_insert(0) += 1;
            }

            total_vram += entry.info.estimated_vram_gb();
            params_sum += entry.info.params_b;
            min_params = min_params.min(entry.info.params_b);
            max_params = max_params.max(entry.info.params_b);
            min_ctx = min_ctx.min(entry.info.context_window);
            max_ctx = max_ctx.max(entry.info.context_window);
        }

        let count = models.len() as f64;
        let avg_params = if count > 0.0 {
            params_sum / count
        } else {
            0.0
        };

        Self {
            total_models: models.len(),
            total_providers: reg.provider_count(),
            models_by_provider,
            modality_counts,
            total_vram_gb: total_vram,
            min_params: if min_params == f64::MAX { 0.0 } else { min_params },
            max_params,
            avg_params,
            min_context: if min_ctx == u32::MAX { 0 } else { min_ctx },
            max_context: max_ctx,
        }
    }
}

impl fmt::Display for RegistryStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Registry Statistics")?;
        writeln!(f, "==================")?;
        writeln!(f, "Models:     {}", self.total_models)?;
        writeln!(f, "Providers:  {}", self.total_providers)?;
        writeln!(
            f,
            "Params:     {:.1}B — {:.1}B (avg {:.1}B)",
            self.min_params, self.max_params, self.avg_params
        )?;
        writeln!(
            f,
            "Context:    {} — {}",
            self.min_context, self.max_context
        )?;
        writeln!(f, "VRAM:       {:.1} GB total", self.total_vram_gb)?;
        writeln!(f)?;
        writeln!(f, "By Provider:")?;
        let mut sorted: Vec<_> = self.models_by_provider.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (name, count) in sorted {
            writeln!(f, "  {:<20} {}", name, count)?;
        }
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> ModelRegistry {
        ModelRegistry::with_defaults()
    }

    #[test]
    fn test_provider_count() {
        assert_eq!(ModelProvider::all().len(), 26);
    }

    #[test]
    fn test_provider_display() {
        let p = ModelProvider::DeepSeek;
        assert_eq!(p.to_string(), "DeepSeek");
        assert_eq!(p.organization(), "DeepSeek AI");
    }

    #[test]
    fn test_provider_from_str() {
        assert_eq!(
            ModelProvider::from_str_loose("deepseek"),
            Some(ModelProvider::DeepSeek)
        );
        assert_eq!(
            ModelProvider::from_str_loose("Llama4"),
            Some(ModelProvider::Llama4)
        );
        assert_eq!(
            ModelProvider::from_str_loose("qwen-3"),
            Some(ModelProvider::Qwen3)
        );
        assert_eq!(
            ModelProvider::from_str_loose("gemma_4"),
            Some(ModelProvider::Gemma4)
        );
        assert_eq!(ModelProvider::from_str_loose("nonexistent"), None);
    }

    #[test]
    fn test_provider_ollama_name() {
        assert_eq!(ModelProvider::DeepSeek.ollama_name(), "deepseek-coder:33b");
        assert_eq!(ModelProvider::Llama4.ollama_name(), "llama4:latest");
        assert_eq!(ModelProvider::Qwen3.ollama_name(), "qwen3:32b");
    }

    #[test]
    fn test_modality_display() {
        assert_eq!(Modality::Text.as_str(), "text");
        assert_eq!(Modality::Code.as_str(), "code");
        assert_eq!(Modality::Image.as_str(), "image");
        assert_eq!(Modality::Multimodal.as_str(), "multimodal");
    }

    #[test]
    fn test_quantization_display() {
        assert_eq!(Quantization::FP32.as_str(), "FP32");
        assert_eq!(Quantization::GGUF_Q4_K_M.as_str(), "GGUF_Q4_K_M");
        assert_eq!(Quantization::None.as_str(), "None");
    }

    #[test]
    fn test_quantization_bits_per_weight() {
        assert_eq!(Quantization::FP32.bits_per_weight(), 32.0);
        assert_eq!(Quantization::FP16.bits_per_weight(), 16.0);
        assert_eq!(Quantization::INT4.bits_per_weight(), 4.5);
    }

    #[test]
    fn test_model_info_creation() {
        let info = ModelInfo::new(
            "test-model",
            ModelProvider::DeepSeek,
            7.0,
            8192,
            vec![Modality::Text],
            Quantization::GGUF_Q4_K_M,
        );
        assert_eq!(info.name, "test-model");
        assert_eq!(info.provider, ModelProvider::DeepSeek);
        assert_eq!(info.params_b, 7.0);
        assert_eq!(info.context_window, 8192);
        assert!(info.supports(Modality::Text));
        assert!(!info.supports(Modality::Code));
    }

    #[test]
    fn test_model_info_vram_estimate() {
        let info = ModelInfo::new(
            "test",
            ModelProvider::DeepSeek,
            7.0,
            8192,
            vec![Modality::Text],
            Quantization::GGUF_Q4_K_M,
        );
        let vram = info.estimated_vram_gb();
        assert!(vram > 0.0 && vram < 100.0, "VRAM estimate out of range: {}", vram);
    }

    #[test]
    fn test_model_entry_creation() {
        let entry = ModelEntry::new(
            ModelInfo::new("test", ModelProvider::Llama4, 7.0, 8192, vec![], Quantization::None),
            "A test model",
            "MIT",
        )
        .with_native_support()
        .with_capabilities(vec!["chat".into()]);

        assert_eq!(entry.info.name, "test");
        assert!(entry.native_support);
        assert_eq!(entry.capabilities, vec!["chat"]);
        assert!(!entry.ollama_tag.is_empty());
    }

    #[test]
    fn test_registry_creation() {
        let reg = ModelRegistry::new();
        assert_eq!(reg.count(), 0);
        assert_eq!(reg.provider_count(), 0);
    }

    #[test]
    fn test_registry_with_defaults() {
        let reg = test_registry();
        assert_eq!(reg.count(), 26);
        assert!(reg.provider_count() >= 20);
    }

    #[test]
    fn test_register_model() {
        let mut reg = ModelRegistry::new();
        let entry = ModelEntry::new(
            ModelInfo::new(
                "custom-model",
                ModelProvider::DeepSeek,
                7.0,
                4096,
                vec![Modality::Text],
                Quantization::None,
            ),
            "Custom model",
            "MIT",
        );
        assert!(reg.register(entry).is_ok());
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn test_register_duplicate_fails() {
        let mut reg = ModelRegistry::new();
        let entry1 = ModelEntry::new(
            ModelInfo::new(
                "dup-model",
                ModelProvider::DeepSeek,
                7.0,
                4096,
                vec![],
                Quantization::None,
            ),
            "First",
            "MIT",
        );
        let entry2 = ModelEntry::new(
            ModelInfo::new(
                "dup-model",
                ModelProvider::Llama4,
                13.0,
                8192,
                vec![],
                Quantization::None,
            ),
            "Second",
            "MIT",
        );
        assert!(reg.register(entry1).is_ok());
        assert!(reg.register(entry2).is_err());
    }

    #[test]
    fn test_register_or_update() {
        let mut reg = ModelRegistry::new();
        let e1 = ModelEntry::new(
            ModelInfo::new(
                "update-model",
                ModelProvider::DeepSeek,
                7.0,
                4096,
                vec![],
                Quantization::None,
            ),
            "v1",
            "MIT",
        );
        let e2 = ModelEntry::new(
            ModelInfo::new(
                "update-model",
                ModelProvider::DeepSeek,
                13.0,
                8192,
                vec![],
                Quantization::None,
            ),
            "v2",
            "MIT",
        );
        reg.register_or_update(e1);
        assert_eq!(reg.count(), 1);
        reg.register_or_update(e2);
        assert_eq!(reg.count(), 1);
        assert_eq!(reg.get_model("update-model").unwrap().info.params_b, 13.0);
    }

    #[test]
    fn test_get_model() {
        let reg = test_registry();
        assert!(reg.get_model("deepseek-coder-33b").is_some());
        assert!(reg.get_model("nonexistent").is_none());
    }

    #[test]
    fn test_list_models() {
        let reg = test_registry();
        let models = reg.list_models();
        assert_eq!(models.len(), 26);
    }

    #[test]
    fn test_list_model_names() {
        let reg = test_registry();
        let names = reg.list_model_names();
        assert!(names.contains(&"deepseek-coder-33b"));
        assert!(names.contains(&"bloom-176b"));
    }

    #[test]
    fn test_models_by_provider() {
        let reg = test_registry();
        let deepseek = reg.models_by_provider(ModelProvider::DeepSeek);
        assert!(!deepseek.is_empty());
        assert!(deepseek.iter().all(|e| e.info.provider == ModelProvider::DeepSeek));
    }

    #[test]
    fn test_search() {
        let reg = test_registry();
        let results = reg.search("deepseek");
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .all(|e| e.info.name.to_lowercase().contains("deepseek")));
    }

    #[test]
    fn test_search_case_insensitive() {
        let reg = test_registry();
        let results = reg.search("BLOOM");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_models_with_modality() {
        let reg = test_registry();
        let code_models = reg.models_with_modality(Modality::Code);
        assert!(!code_models.is_empty());
        let image_models = reg.models_with_modality(Modality::Image);
        assert_eq!(image_models.len(), 1); // MiniCPM-V
    }

    #[test]
    fn test_models_by_size() {
        let reg = test_registry();
        let small = reg.models_by_size(0.0, 10.0);
        assert!(!small.is_empty());
        let large = reg.models_by_size(100.0, 1000.0);
        assert!(!large.is_empty());
    }

    #[test]
    fn test_models_with_context() {
        let reg = test_registry();
        let long_ctx = reg.models_with_context(128_000);
        assert!(!long_ctx.is_empty());
    }

    #[test]
    fn test_remove_model() {
        let mut reg = test_registry();
        let removed = reg.remove("deepseek-coder-33b");
        assert!(removed.is_some());
        assert!(reg.get_model("deepseek-coder-33b").is_none());
        assert!(reg.remove("nonexistent").is_none());
    }

    #[test]
    fn test_smallest_model_with_modality() {
        let reg = test_registry();
        let smallest = reg.smallest_model_with_modality(Modality::Code);
        assert!(smallest.is_some());
    }

    #[test]
    fn test_largest_context_model() {
        let reg = test_registry();
        let largest = reg.largest_context_model();
        assert!(largest.is_some());
        assert_eq!(largest.unwrap().info.name, "kimi-k2-1t");
    }

    #[test]
    fn test_total_vram() {
        let reg = test_registry();
        let total = reg.total_estimated_vram_gb();
        assert!(total > 0.0);
    }

    #[test]
    fn test_registry_stats() {
        let reg = test_registry();
        let stats = RegistryStats::from_registry(&reg);
        assert_eq!(stats.total_models, 26);
        assert!(stats.total_providers >= 20);
        assert!(stats.max_params > stats.min_params);
        assert!(stats.max_context > stats.min_context);
    }

    #[test]
    fn test_model_filter() {
        let reg = test_registry();

        // Provider filter
        let filtered = ModelFilter::new()
            .provider(ModelProvider::DeepSeek)
            .apply(&reg);
        assert!(filtered.iter().all(|e| e.info.provider == ModelProvider::DeepSeek));

        // Modality filter
        let code = ModelFilter::new()
            .modality(Modality::Code)
            .apply(&reg);
        assert!(!code.is_empty());
        assert!(code.iter().all(|e| e.info.supports(Modality::Code)));

        // Size filter
        let small = ModelFilter::new()
            .max_params(10.0)
            .apply(&reg);
        assert!(small.iter().all(|e| e.info.params_b <= 10.0));

        // Native only
        let native = ModelFilter::new()
            .native_only()
            .apply(&reg);
        assert!(native.iter().all(|e| e.native_support));

        // Name filter
        let qwen = ModelFilter::new()
            .name("qwen")
            .apply(&reg);
        assert!(qwen.iter().all(|e| e.info.name.contains("qwen")));

        // Combined
        let combined = ModelFilter::new()
            .modality(Modality::Code)
            .min_params(10.0)
            .native_only()
            .apply(&reg);
        assert!(!combined.is_empty());
        assert!(combined.iter().all(|e| {
            e.info.supports(Modality::Code) && e.info.params_b >= 10.0 && e.native_support
        }));
    }

    #[test]
    fn test_inference_engine_backends() {
        let client = OllamaClient::default();
        let mut engine = InferenceEngine::new(client);

        assert_eq!(
            engine.get_backend(&ModelProvider::DeepSeek),
            &InferenceBackend::Ollama
        );

        engine.set_backend(ModelProvider::DeepSeek, InferenceBackend::Native);
        assert_eq!(
            engine.get_backend(&ModelProvider::DeepSeek),
            &InferenceBackend::Native
        );

        engine.set_backend(
            ModelProvider::DeepSeek,
            InferenceBackend::Api("https://api.example.com".into()),
        );
        assert!(matches!(
            engine.get_backend(&ModelProvider::DeepSeek),
            InferenceBackend::Api(_)
        ));
    }

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://custom:9999");
        assert_eq!(client.base_url(), "http://custom:9999");
    }

    #[test]
    fn test_ollama_client_default() {
        let client = OllamaClient::default();
        assert_eq!(client.base_url(), "http://localhost:11434");
    }

    #[test]
    fn test_model_entry_with_api_endpoint() {
        let entry = ModelEntry::new(
            ModelInfo::new(
                "api-model",
                ModelProvider::DeepSeek,
                7.0,
                4096,
                vec![],
                Quantization::None,
            ),
            "API model",
            "MIT",
        )
        .with_api_endpoint("https://api.deepseek.com");
        assert_eq!(
            entry.api_endpoint.as_deref(),
            Some("https://api.deepseek.com")
        );
    }

    #[test]
    fn test_provider_all_returns_26() {
        let all = ModelProvider::all();
        assert_eq!(all.len(), 26);

        // Check uniqueness
        let mut seen = std::collections::HashSet::new();
        for p in all {
            assert!(seen.insert(*p), "duplicate provider: {:?}", p);
        }
    }

    #[test]
    fn test_model_info_display() {
        let info = ModelInfo::new(
            "test-model",
            ModelProvider::DeepSeek,
            7.0,
            8192,
            vec![Modality::Text],
            Quantization::GGUF_Q4_K_M,
        );
        let display = format!("{}", info);
        assert!(display.contains("test-model"));
        assert!(display.contains("DeepSeek"));
    }

    #[test]
    fn test_registry_stats_display() {
        let reg = test_registry();
        let stats = RegistryStats::from_registry(&reg);
        let display = format!("{}", stats);
        assert!(display.contains("Registry Statistics"));
        assert!(display.contains("26"));
    }

    #[test]
    fn test_quantization_all_variants() {
        let variants = [
            Quantization::FP32,
            Quantization::FP16,
            Quantization::BF16,
            Quantization::INT8,
            Quantization::INT4,
            Quantization::GGUF_Q4_0,
            Quantization::GGUF_Q4_K_M,
            Quantization::GGUF_Q5_K_M,
            Quantization::GGUF_Q8_0,
            Quantization::GPTQ,
            Quantization::AWQ,
            Quantization::EXL2,
            Quantization::None,
        ];
        assert_eq!(variants.len(), 13);
        for v in &variants {
            assert!(!v.as_str().is_empty());
            assert!(v.bits_per_weight() > 0.0);
        }
    }

    #[test]
    fn test_modality_all_variants() {
        let variants = [
            Modality::Text,
            Modality::Code,
            Modality::Image,
            Modality::Audio,
            Modality::Video,
            Modality::Multimodal,
        ];
        assert_eq!(variants.len(), 6);
        for v in &variants {
            assert!(!v.as_str().is_empty());
        }
    }
}
