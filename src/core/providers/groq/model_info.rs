//! Groq Model Information
//!
//! Contains model configurations and capabilities for Groq-supported models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Groq model identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroqModel {
    // Llama 3.3 models
    Llama33_70B,

    // Llama 3.2 models
    Llama32_90BTextPreview,
    Llama32_11BTextPreview,
    Llama32_3BPreview,
    Llama32_1BPreview,

    // Llama 3.1 models
    Llama31_405B,
    Llama31_70B,
    Llama31_8B,

    // Llama 3 models
    Llama3_70B,
    Llama3_8B,

    // Mixtral models
    Mixtral8x7B,

    // Gemma models
    Gemma2_9B,
    Gemma7B,

    // Distilled models
    Llama3GroqToolUse70B,
    Llama3GroqToolUse8B,

    // Audio models
    WhisperLargeV3,
    WhisperLargeV3Turbo,
    DistilWhisperLargeV3,
}

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model ID as used in API calls
    pub model_id: &'static str,

    /// Human-friendly model name
    pub display_name: &'static str,

    /// Maximum context length (tokens)
    pub context_length: u32,

    /// Maximum output tokens
    pub max_output_tokens: u32,

    /// Whether the model supports tool/function calling
    pub supports_tools: bool,

    /// Whether this is a reasoning model
    pub is_reasoning: bool,

    /// Whether the model supports vision
    pub supports_vision: bool,

    /// Whether this is an audio model
    pub is_audio: bool,

    /// Cost per 1M input tokens (USD)
    pub input_cost_per_million: f64,

    /// Cost per 1M output tokens (USD)
    pub output_cost_per_million: f64,
}

/// Static model configurations
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Llama 3.3 models
    configs.insert(
        "llama-3.3-70b-versatile",
        ModelInfo {
            model_id: "llama-3.3-70b-versatile",
            display_name: "Llama 3.3 70B",
            context_length: 128000,
            max_output_tokens: 32768,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    // Llama 3.2 models
    configs.insert(
        "llama-3.2-90b-text-preview",
        ModelInfo {
            model_id: "llama-3.2-90b-text-preview",
            display_name: "Llama 3.2 90B Text Preview",
            context_length: 128000,
            max_output_tokens: 8192,
            supports_tools: false,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.90,
            output_cost_per_million: 0.90,
        },
    );

    configs.insert(
        "llama-3.2-11b-text-preview",
        ModelInfo {
            model_id: "llama-3.2-11b-text-preview",
            display_name: "Llama 3.2 11B Text Preview",
            context_length: 128000,
            max_output_tokens: 8192,
            supports_tools: false,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.18,
            output_cost_per_million: 0.18,
        },
    );

    // Llama 3.1 models
    configs.insert(
        "llama-3.1-405b-reasoning",
        ModelInfo {
            model_id: "llama-3.1-405b-reasoning",
            display_name: "Llama 3.1 405B Reasoning",
            context_length: 131072,
            max_output_tokens: 16384,
            supports_tools: true,
            is_reasoning: true,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 3.00,
            output_cost_per_million: 3.00,
        },
    );

    configs.insert(
        "llama-3.1-70b-versatile",
        ModelInfo {
            model_id: "llama-3.1-70b-versatile",
            display_name: "Llama 3.1 70B",
            context_length: 131072,
            max_output_tokens: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    configs.insert(
        "llama-3.1-8b-instant",
        ModelInfo {
            model_id: "llama-3.1-8b-instant",
            display_name: "Llama 3.1 8B",
            context_length: 131072,
            max_output_tokens: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.05,
            output_cost_per_million: 0.08,
        },
    );

    // Mixtral
    configs.insert(
        "mixtral-8x7b-32768",
        ModelInfo {
            model_id: "mixtral-8x7b-32768",
            display_name: "Mixtral 8x7B",
            context_length: 32768,
            max_output_tokens: 32768,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.24,
            output_cost_per_million: 0.24,
        },
    );

    // Gemma models
    configs.insert(
        "gemma2-9b-it",
        ModelInfo {
            model_id: "gemma2-9b-it",
            display_name: "Gemma2 9B",
            context_length: 8192,
            max_output_tokens: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.20,
            output_cost_per_million: 0.20,
        },
    );

    // Tool use optimized models
    configs.insert(
        "llama3-groq-70b-8192-tool-use-preview",
        ModelInfo {
            model_id: "llama3-groq-70b-8192-tool-use-preview",
            display_name: "Llama 3 Groq 70B Tool Use",
            context_length: 8192,
            max_output_tokens: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.89,
            output_cost_per_million: 0.89,
        },
    );

    configs.insert(
        "llama3-groq-8b-8192-tool-use-preview",
        ModelInfo {
            model_id: "llama3-groq-8b-8192-tool-use-preview",
            display_name: "Llama 3 Groq 8B Tool Use",
            context_length: 8192,
            max_output_tokens: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.19,
            output_cost_per_million: 0.19,
        },
    );

    // Audio models
    configs.insert(
        "whisper-large-v3",
        ModelInfo {
            model_id: "whisper-large-v3",
            display_name: "Whisper Large v3",
            context_length: 0,    // Audio model
            max_output_tokens: 0, // Audio model
            supports_tools: false,
            is_reasoning: false,
            supports_vision: false,
            is_audio: true,
            input_cost_per_million: 0.111, // Per hour of audio
            output_cost_per_million: 0.0,
        },
    );

    configs.insert(
        "whisper-large-v3-turbo",
        ModelInfo {
            model_id: "whisper-large-v3-turbo",
            display_name: "Whisper Large v3 Turbo",
            context_length: 0,
            max_output_tokens: 0,
            supports_tools: false,
            is_reasoning: false,
            supports_vision: false,
            is_audio: true,
            input_cost_per_million: 0.04, // Per hour of audio
            output_cost_per_million: 0.0,
        },
    );

    configs.insert(
        "distil-whisper-large-v3-en",
        ModelInfo {
            model_id: "distil-whisper-large-v3-en",
            display_name: "Distil Whisper Large v3",
            context_length: 0,
            max_output_tokens: 0,
            supports_tools: false,
            is_reasoning: false,
            supports_vision: false,
            is_audio: true,
            input_cost_per_million: 0.02, // Per hour of audio
            output_cost_per_million: 0.0,
        },
    );

    // Reasoning models (matching Python LiteLLM configuration)
    configs.insert(
        "deepseek-r1-distill-llama-70b",
        ModelInfo {
            model_id: "deepseek-r1-distill-llama-70b",
            display_name: "DeepSeek R1 Distill Llama 70B",
            context_length: 131072,
            max_output_tokens: 131072,
            supports_tools: true,
            is_reasoning: true,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    configs.insert(
        "qwen3-32b",
        ModelInfo {
            model_id: "qwen3-32b",
            display_name: "Qwen 3 32B",
            context_length: 131072,
            max_output_tokens: 131072,
            supports_tools: true,
            is_reasoning: true,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    configs.insert(
        "gpt-oss-20b",
        ModelInfo {
            model_id: "gpt-oss-20b",
            display_name: "GPT OSS 20B",
            context_length: 131072,
            max_output_tokens: 32766,
            supports_tools: true,
            is_reasoning: true,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.15,
            output_cost_per_million: 0.75,
        },
    );

    configs.insert(
        "gpt-oss-120b",
        ModelInfo {
            model_id: "gpt-oss-120b",
            display_name: "GPT OSS 120B",
            context_length: 131072,
            max_output_tokens: 32766,
            supports_tools: true,
            is_reasoning: true,
            supports_vision: false,
            is_audio: false,
            input_cost_per_million: 0.15,
            output_cost_per_million: 0.75,
        },
    );

    configs
});

/// Get model information for a given model ID
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    MODEL_CONFIGS.get(model_id)
}

/// Check if a model supports reasoning
pub fn is_reasoning_model(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.is_reasoning)
        .unwrap_or(false)
}

/// Get all available model IDs
pub fn get_available_models() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

/// Get all models that support tool/function calling
pub fn get_tool_capable_models() -> Vec<&'static str> {
    MODEL_CONFIGS
        .iter()
        .filter(|(_, info)| info.supports_tools)
        .map(|(id, _)| *id)
        .collect()
}
