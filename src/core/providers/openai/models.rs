//! OpenAI Model Registry
//!
//! Dynamic model discovery and capability detection system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::providers::base::get_pricing_db;
use crate::core::types::common::ModelInfo;

/// OpenAI-specific model features
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenAIModelFeature {
    /// Chat completion support
    ChatCompletion,
    /// Streaming response support
    StreamingSupport,
    /// Function/tool calling support
    FunctionCalling,
    /// Vision support (multimodal)
    VisionSupport,
    /// System message support
    SystemMessages,
    /// JSON mode support
    JsonMode,
    /// O-series reasoning mode
    ReasoningMode,
    /// Audio input support
    AudioInput,
    /// Audio output support (TTS)
    AudioOutput,
    /// Image generation (DALL-E)
    ImageGeneration,
    /// Image editing
    ImageEditing,
    /// Audio transcription
    AudioTranscription,
    /// Fine-tuning support
    FineTuning,
    /// Embeddings generation
    Embeddings,
    /// Code completion optimized
    CodeCompletion,
    /// High context window (>32K)
    LargeContext,
    /// Real-time audio processing
    RealtimeAudio,
}

/// OpenAI model specification
#[derive(Debug, Clone)]
pub struct OpenAIModelSpec {
    /// Basic model information
    pub model_info: ModelInfo,
    /// Supported features
    pub features: Vec<OpenAIModelFeature>,
    /// Model family (gpt-4, gpt-3.5, dalle, whisper, etc.)
    pub family: OpenAIModelFamily,
    /// Model configuration
    pub config: OpenAIModelConfig,
}

/// OpenAI model families
#[derive(Debug, Clone, PartialEq)]
pub enum OpenAIModelFamily {
    GPT4,
    GPT4Turbo,
    GPT4O,
    GPT35,
    GPT5, // Future model
    O1,   // Reasoning models
    DALLE2,
    DALLE3,
    Whisper,
    TTS,
    Embedding,
    Moderation,
}

/// Model-specific configuration
#[derive(Debug, Clone)]
pub struct OpenAIModelConfig {
    /// Maximum requests per minute
    pub max_rpm: Option<u32>,
    /// Maximum tokens per minute  
    pub max_tpm: Option<u32>,
    /// Supports batch API
    pub supports_batch: bool,
    /// Default temperature
    pub default_temperature: Option<f32>,
    /// Supports streaming
    pub supports_streaming: bool,
    /// Custom parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for OpenAIModelConfig {
    fn default() -> Self {
        Self {
            max_rpm: None,
            max_tpm: None,
            supports_batch: false,
            default_temperature: None,
            supports_streaming: true,
            custom_params: HashMap::new(),
        }
    }
}

/// OpenAI model registry
#[derive(Debug)]
pub struct OpenAIModelRegistry {
    models: HashMap<String, OpenAIModelSpec>,
}

impl Default for OpenAIModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAIModelRegistry {
    /// Create new registry instance
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.load_models();
        registry
    }

    /// Load models from pricing database and add static definitions
    fn load_models(&mut self) {
        let pricing_db = get_pricing_db();
        let model_ids = pricing_db.get_provider_models("openai");

        // Load from pricing database
        for model_id in &model_ids {
            if let Some(model_info) = pricing_db.to_model_info(model_id, "openai") {
                let features = self.detect_features(&model_info);
                let family = self.determine_family(&model_info);
                let config = self.create_config(&model_info);

                self.models.insert(
                    model_id.clone(),
                    OpenAIModelSpec {
                        model_info,
                        features,
                        family,
                        config,
                    },
                );
            }
        }

        // Add static model definitions as fallback
        if self.models.is_empty() {
            self.add_static_models();
        }
    }

    /// Detect model features based on model info
    fn detect_features(&self, model_info: &ModelInfo) -> Vec<OpenAIModelFeature> {
        let mut features = vec![
            OpenAIModelFeature::SystemMessages,
            OpenAIModelFeature::StreamingSupport,
        ];

        let model_id = &model_info.id;

        // Chat models
        if model_id.starts_with("gpt-") {
            features.push(OpenAIModelFeature::ChatCompletion);
            features.push(OpenAIModelFeature::JsonMode);
        }

        // Function calling support
        if model_info.supports_tools {
            features.push(OpenAIModelFeature::FunctionCalling);
        }

        // Vision support
        if model_info.supports_multimodal || model_id.contains("vision") {
            features.push(OpenAIModelFeature::VisionSupport);
        }

        // O-series reasoning models
        if model_id.starts_with("o1-") {
            features.push(OpenAIModelFeature::ReasoningMode);
        }

        // GPT-4O audio features
        if model_id.contains("gpt-4o-audio") {
            features.push(OpenAIModelFeature::AudioInput);
            features.push(OpenAIModelFeature::AudioOutput);
        }

        // DALL-E models
        if model_id.starts_with("dall-e") {
            features.push(OpenAIModelFeature::ImageGeneration);
            if model_id.contains("dall-e-3") {
                features.push(OpenAIModelFeature::ImageEditing);
            }
        }

        // Whisper models
        if model_id.starts_with("whisper") {
            features.push(OpenAIModelFeature::AudioTranscription);
        }

        // TTS models
        if model_id.starts_with("tts") {
            features.push(OpenAIModelFeature::AudioOutput);
        }

        // Embedding models
        if model_id.contains("embedding") {
            features.push(OpenAIModelFeature::Embeddings);
        }

        // Code-optimized models
        if model_id.contains("code") || model_id.contains("codex") {
            features.push(OpenAIModelFeature::CodeCompletion);
        }

        // Large context models
        if model_info.max_context_length > 32000 {
            features.push(OpenAIModelFeature::LargeContext);
        }

        // Fine-tuning support (selected models)
        if matches!(
            model_id.as_str(),
            "gpt-3.5-turbo" | "gpt-4" | "gpt-4-turbo" | "babbage-002" | "davinci-002"
        ) {
            features.push(OpenAIModelFeature::FineTuning);
        }

        features
    }

    /// Determine model family
    fn determine_family(&self, model_info: &ModelInfo) -> OpenAIModelFamily {
        let model_id = &model_info.id;

        if model_id.starts_with("gpt-4o") {
            OpenAIModelFamily::GPT4O
        } else if model_id.starts_with("gpt-4-turbo")
            || model_id.starts_with("gpt-4-1106")
            || model_id.starts_with("gpt-4-0125")
        {
            OpenAIModelFamily::GPT4Turbo
        } else if model_id.starts_with("gpt-4") {
            OpenAIModelFamily::GPT4
        } else if model_id.starts_with("gpt-3.5") {
            OpenAIModelFamily::GPT35
        } else if model_id.starts_with("gpt-5") {
            OpenAIModelFamily::GPT5
        } else if model_id.starts_with("o1-") {
            OpenAIModelFamily::O1
        } else if model_id.starts_with("dall-e-2") {
            OpenAIModelFamily::DALLE2
        } else if model_id.starts_with("dall-e-3") {
            OpenAIModelFamily::DALLE3
        } else if model_id.starts_with("whisper") {
            OpenAIModelFamily::Whisper
        } else if model_id.starts_with("tts") {
            OpenAIModelFamily::TTS
        } else if model_id.contains("embedding") {
            OpenAIModelFamily::Embedding
        } else {
            OpenAIModelFamily::GPT4 // Default fallback
        }
    }

    /// Create model configuration
    fn create_config(&self, model_info: &ModelInfo) -> OpenAIModelConfig {
        let mut config = OpenAIModelConfig::default();
        let model_id = &model_info.id;

        // Set rate limits based on model
        match model_id.as_str() {
            m if m.starts_with("gpt-4") => {
                config.max_rpm = Some(10000);
                config.max_tpm = Some(300000);
            }
            m if m.starts_with("gpt-3.5") => {
                config.max_rpm = Some(10000);
                config.max_tpm = Some(1000000);
            }
            m if m.starts_with("o1-") => {
                config.max_rpm = Some(5000);
                config.max_tpm = Some(100000);
                config.default_temperature = Some(1.0); // O1 models use temperature=1
            }
            _ => {
                config.max_rpm = Some(5000);
                config.max_tpm = Some(200000);
            }
        }

        // Batch API support
        config.supports_batch = matches!(
            model_id.as_str(),
            "gpt-4"
                | "gpt-4-turbo"
                | "gpt-3.5-turbo"
                | "text-embedding-ada-002"
                | "text-embedding-3-small"
                | "text-embedding-3-large"
        );

        // Streaming support
        config.supports_streaming =
            !model_id.contains("embedding") && !model_id.contains("whisper");

        config
    }

    /// Add static model definitions as fallback
    fn add_static_models(&mut self) {
        let static_models = vec![
            // GPT-4 models
            (
                "gpt-4",
                "GPT-4",
                OpenAIModelFamily::GPT4,
                8192,
                Some(8192),
                0.03,
                0.06,
            ),
            (
                "gpt-4-turbo",
                "GPT-4 Turbo",
                OpenAIModelFamily::GPT4Turbo,
                128000,
                Some(4096),
                0.01,
                0.03,
            ),
            (
                "gpt-4o",
                "GPT-4O",
                OpenAIModelFamily::GPT4O,
                128000,
                Some(4096),
                0.005,
                0.015,
            ),
            // GPT-3.5 models
            (
                "gpt-3.5-turbo",
                "GPT-3.5 Turbo",
                OpenAIModelFamily::GPT35,
                16385,
                Some(4096),
                0.0005,
                0.0015,
            ),
            // O1 models
            (
                "o1-preview",
                "O1 Preview",
                OpenAIModelFamily::O1,
                128000,
                Some(32768),
                0.015,
                0.06,
            ),
            (
                "o1-mini",
                "O1 Mini",
                OpenAIModelFamily::O1,
                128000,
                Some(65536),
                0.003,
                0.012,
            ),
            // DALL-E models
            (
                "dall-e-2",
                "DALL-E 2",
                OpenAIModelFamily::DALLE2,
                1000,
                None,
                0.02,
                0.02,
            ),
            (
                "dall-e-3",
                "DALL-E 3",
                OpenAIModelFamily::DALLE3,
                4000,
                None,
                0.04,
                0.08,
            ),
            // Embedding models
            (
                "text-embedding-ada-002",
                "Embedding Ada 002",
                OpenAIModelFamily::Embedding,
                8191,
                None,
                0.0001,
                0.0001,
            ),
            (
                "text-embedding-3-small",
                "Embedding 3 Small",
                OpenAIModelFamily::Embedding,
                8191,
                None,
                0.00002,
                0.00002,
            ),
            (
                "text-embedding-3-large",
                "Embedding 3 Large",
                OpenAIModelFamily::Embedding,
                8191,
                None,
                0.00013,
                0.00013,
            ),
            // Whisper models
            (
                "whisper-1",
                "Whisper",
                OpenAIModelFamily::Whisper,
                25000000,
                None,
                0.006,
                0.006,
            ),
            // TTS models
            (
                "tts-1",
                "TTS 1",
                OpenAIModelFamily::TTS,
                4096,
                None,
                0.015,
                0.015,
            ),
            (
                "tts-1-hd",
                "TTS 1 HD",
                OpenAIModelFamily::TTS,
                4096,
                None,
                0.03,
                0.03,
            ),
        ];

        for (id, name, family, max_context, max_output, input_cost, output_cost) in static_models {
            let model_info = ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                provider: "openai".to_string(),
                max_context_length: max_context,
                max_output_length: max_output,
                supports_streaming: family != OpenAIModelFamily::Embedding
                    && family != OpenAIModelFamily::Whisper,
                supports_tools: matches!(
                    family,
                    OpenAIModelFamily::GPT4
                        | OpenAIModelFamily::GPT4Turbo
                        | OpenAIModelFamily::GPT4O
                        | OpenAIModelFamily::GPT35
                ),
                supports_multimodal: matches!(family, OpenAIModelFamily::GPT4O)
                    || id.contains("vision"),
                input_cost_per_1k_tokens: Some(input_cost),
                output_cost_per_1k_tokens: Some(output_cost),
                currency: "USD".to_string(),
                capabilities: vec![], // Will be set from features
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            };

            let features = self.detect_features(&model_info);
            let config = self.create_config(&model_info);

            self.models.insert(
                id.to_string(),
                OpenAIModelSpec {
                    model_info,
                    features,
                    family,
                    config,
                },
            );
        }
    }

    /// Get all model information
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.models
            .values()
            .map(|spec| spec.model_info.clone())
            .collect()
    }

    /// Get specific model specification
    pub fn get_model_spec(&self, model_id: &str) -> Option<&OpenAIModelSpec> {
        self.models.get(model_id)
    }

    /// Check if model supports a feature
    pub fn supports_feature(&self, model_id: &str, feature: &OpenAIModelFeature) -> bool {
        self.models
            .get(model_id)
            .map(|spec| spec.features.contains(feature))
            .unwrap_or(false)
    }

    /// Get models by family
    pub fn get_models_by_family(&self, family: &OpenAIModelFamily) -> Vec<String> {
        self.models
            .iter()
            .filter_map(|(id, spec)| {
                if &spec.family == family {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get models supporting specific feature
    pub fn get_models_with_feature(&self, feature: &OpenAIModelFeature) -> Vec<String> {
        self.models
            .iter()
            .filter_map(|(id, spec)| {
                if spec.features.contains(feature) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the best model for a specific use case
    pub fn get_recommended_model(&self, use_case: OpenAIUseCase) -> Option<String> {
        match use_case {
            OpenAIUseCase::GeneralChat => Some("gpt-4o".to_string()),
            OpenAIUseCase::CodeGeneration => Some("gpt-4o".to_string()),
            OpenAIUseCase::Reasoning => Some("o1-preview".to_string()),
            OpenAIUseCase::Vision => Some("gpt-4o".to_string()),
            OpenAIUseCase::ImageGeneration => Some("dall-e-3".to_string()),
            OpenAIUseCase::AudioTranscription => Some("whisper-1".to_string()),
            OpenAIUseCase::TextToSpeech => Some("tts-1-hd".to_string()),
            OpenAIUseCase::Embeddings => Some("text-embedding-3-large".to_string()),
            OpenAIUseCase::CostOptimized => Some("gpt-3.5-turbo".to_string()),
        }
    }
}

/// OpenAI use cases for model recommendation
#[derive(Debug, Clone)]
pub enum OpenAIUseCase {
    GeneralChat,
    CodeGeneration,
    Reasoning,
    Vision,
    ImageGeneration,
    AudioTranscription,
    TextToSpeech,
    Embeddings,
    CostOptimized,
}

/// Global model registry instance
static OPENAI_REGISTRY: OnceLock<OpenAIModelRegistry> = OnceLock::new();

/// Get global OpenAI model registry
pub fn get_openai_registry() -> &'static OpenAIModelRegistry {
    OPENAI_REGISTRY.get_or_init(OpenAIModelRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = OpenAIModelRegistry::new();
        let models = registry.get_all_models();
        assert!(!models.is_empty());
    }

    #[test]
    fn test_feature_detection() {
        let registry = get_openai_registry();

        // Test GPT-4 features
        assert!(registry.supports_feature("gpt-4", &OpenAIModelFeature::ChatCompletion));
        assert!(registry.supports_feature("gpt-4", &OpenAIModelFeature::FunctionCalling));
        assert!(registry.supports_feature("gpt-4", &OpenAIModelFeature::StreamingSupport));

        // Test O1 features
        assert!(registry.supports_feature("o1-preview", &OpenAIModelFeature::ReasoningMode));

        // Test DALL-E features
        assert!(registry.supports_feature("dall-e-3", &OpenAIModelFeature::ImageGeneration));
    }

    #[test]
    fn test_model_families() {
        let registry = get_openai_registry();
        let gpt4_models = registry.get_models_by_family(&OpenAIModelFamily::GPT4);
        assert!(!gpt4_models.is_empty());
    }

    #[test]
    fn test_model_recommendations() {
        let registry = get_openai_registry();

        assert_eq!(
            registry.get_recommended_model(OpenAIUseCase::GeneralChat),
            Some("gpt-4o".to_string())
        );
        assert_eq!(
            registry.get_recommended_model(OpenAIUseCase::Reasoning),
            Some("o1-preview".to_string())
        );
        assert_eq!(
            registry.get_recommended_model(OpenAIUseCase::CostOptimized),
            Some("gpt-3.5-turbo".to_string())
        );
    }
}

// ============================================================================
// OpenAI API Request/Response Types
// ============================================================================

/// OpenAI Chat Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<OpenAIResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
}

/// OpenAI Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<OpenAIFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<serde_json::Value>,
}

/// OpenAI Tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<OpenAIFunction>,
}

/// OpenAI Function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// OpenAI Tool Call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunctionCall,
}

/// OpenAI Function Call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCall {
    pub name: String,
    pub arguments: String,
}

/// OpenAI Response Format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
}

/// OpenAI Chat Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAIUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// OpenAI Choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// OpenAI Usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<OpenAITokenDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<OpenAITokenDetails>,
}

/// OpenAI Token Details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITokenDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}

/// OpenAI Stream Chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIStreamChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAIUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// OpenAI Stream Choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamChoice {
    pub index: u32,
    pub delta: OpenAIDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// OpenAI Delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCallDelta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<OpenAIFunctionCallDelta>,
}

/// OpenAI Tool Call Delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCallDelta {
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub tool_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<OpenAIFunctionCallDelta>,
}

/// OpenAI Function Call Delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCallDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// OpenAI Content Part
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAIImageUrl },
    #[serde(rename = "input_audio")]
    InputAudio { input_audio: OpenAIInputAudio },
}

/// OpenAI Image URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// OpenAI Input Audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIInputAudio {
    pub data: String,
    pub format: String,
}

/// OpenAI Tool Choice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIToolChoice {
    String(String), // "none", "auto", "required"
    Function {
        #[serde(rename = "type")]
        r#type: String,
        function: OpenAIFunctionChoice,
    },
}

impl OpenAIToolChoice {
    pub fn none() -> Self {
        Self::String("none".to_string())
    }

    pub fn auto() -> Self {
        Self::String("auto".to_string())
    }

    pub fn required() -> Self {
        Self::String("required".to_string())
    }
}

/// OpenAI Function Choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionChoice {
    pub name: String,
}

/// OpenAI Logprobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAILogprobs {
    pub content: Option<Vec<OpenAITokenLogprob>>,
    pub refusal: Option<serde_json::Value>,
}

/// OpenAI Token Logprob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITokenLogprob {
    pub token: String,
    pub logprob: f64,
    pub bytes: Option<Vec<u8>>,
    pub top_logprobs: Vec<OpenAITopLogprob>,
}

/// OpenAI Top Logprob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITopLogprob {
    pub token: String,
    pub logprob: f64,
    pub bytes: Option<Vec<u8>>,
}
