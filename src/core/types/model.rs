//! Model information types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Provider capability enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    /// Chat completion
    ChatCompletion,
    /// Streaming chat completion
    ChatCompletionStream,
    /// Embeddings generation
    Embeddings,
    /// Image generation
    ImageGeneration,
    /// Image editing
    ImageEdit,
    /// Image variation
    ImageVariation,
    /// Audio transcription
    AudioTranscription,
    /// Audio translation
    AudioTranslation,
    /// Text to speech
    TextToSpeech,
    /// Tool calling
    ToolCalling,
    /// Function calling (backward compatibility)
    FunctionCalling,
    /// Code execution
    CodeExecution,
    /// File upload
    FileUpload,
    /// Fine-tuning
    FineTuning,
    /// Batch processing
    BatchProcessing,
    /// Real-time API
    RealtimeApi,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    /// Model name
    pub name: String,
    /// Provider name
    pub provider: String,
    /// Maximum context length
    pub max_context_length: u32,
    /// Maximum output length
    pub max_output_length: Option<u32>,
    /// Supports streaming
    pub supports_streaming: bool,
    /// Supports tool calling
    pub supports_tools: bool,
    /// Supports multimodal
    pub supports_multimodal: bool,
    /// Input price (per 1K tokens)
    pub input_cost_per_1k_tokens: Option<f64>,
    /// Output price (per 1K tokens)
    pub output_cost_per_1k_tokens: Option<f64>,
    /// Currency unit
    pub currency: String,
    /// Supported features
    pub capabilities: Vec<ProviderCapability>,
    /// Created at
    pub created_at: Option<SystemTime>,
    /// Updated at
    pub updated_at: Option<SystemTime>,
    /// Extra metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            provider: String::new(),
            max_context_length: 4096,
            max_output_length: None,
            supports_streaming: false,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: Vec::new(),
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }
    }
}
