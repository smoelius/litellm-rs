//! Google Vertex AI Provider Implementation
//!
//! Comprehensive support for Google Vertex AI including:
//! - Gemini models (Pro, Flash, Ultra)
//! - Partner models (Anthropic, AI21, Meta Llama)
//! - Model Garden
//! - Multimodal embeddings
//! - Image generation
//! - Text-to-speech
//! - Context caching
//! - Batch operations

pub mod auth;
pub mod batches;
pub mod client;
pub mod common_utils;
pub mod context_caching;
pub mod count_tokens;
pub mod embeddings;
pub mod error;
pub mod files;
pub mod fine_tuning;
pub mod gemini;
pub mod gemini_embeddings;
pub mod google_genai;
pub mod image_generation;
pub mod models;
pub mod multimodal_embeddings;
pub mod partner_models;
pub mod text_to_speech;
pub mod transformers;
pub mod vector_stores;
pub mod vertex_ai_partner_models;
pub mod vertex_embeddings;
pub mod vertex_model_garden;

pub use auth::{VertexAuth, VertexCredentials};
pub use client::VertexAIProvider;
pub use common_utils::VertexAIConfig;
pub use error::VertexAIError;

/// Main VertexAI Provider Configuration
#[derive(Debug, Clone)]
pub struct VertexAIProviderConfig {
    /// Google Cloud Project ID
    pub project_id: String,

    /// Vertex AI region (e.g., "us-central1")
    pub location: String,

    /// API version to use ("v1" or "v1beta1")
    pub api_version: String,

    /// Authentication credentials
    pub credentials: VertexCredentials,

    /// Custom API endpoint (optional)
    pub api_base: Option<String>,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Enable experimental features
    pub enable_experimental: bool,
}

impl Default for VertexAIProviderConfig {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            location: "us-central1".to_string(),
            api_version: "v1".to_string(),
            credentials: VertexCredentials::ApplicationDefault,
            api_base: None,
            timeout_seconds: 60,
            max_retries: 3,
            enable_experimental: false,
        }
    }
}

impl crate::core::traits::provider::ProviderConfig for VertexAIProviderConfig {
    fn validate(&self) -> Result<(), String> {
        if self.project_id.is_empty() {
            return Err("Project ID is required".to_string());
        }
        if self.location.is_empty() {
            return Err("Location is required".to_string());
        }
        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        None // Vertex AI uses credentials, not API keys
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base
            .as_deref()
            .or(Some("https://aiplatform.googleapis.com"))
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Supported Vertex AI models
#[derive(Debug, Clone)]
pub enum VertexAIModel {
    // Gemini models
    GeminiPro,
    GeminiProVision,
    GeminiFlash,
    GeminiFlashThinking,
    GeminiUltra,

    // Partner models
    Claude3Opus,
    Claude3Sonnet,
    Claude3Haiku,

    // Meta models
    Llama3_70B,
    Llama3_8B,

    // AI21 models
    Jamba15Large,
    Jamba15Mini,

    // Custom model
    Custom(String),
}

impl VertexAIModel {
    /// Get the model ID string for API calls
    pub fn model_id(&self) -> String {
        match self {
            Self::GeminiPro => "gemini-1.5-pro".to_string(),
            Self::GeminiProVision => "gemini-1.5-pro-vision".to_string(),
            Self::GeminiFlash => "gemini-1.5-flash".to_string(),
            Self::GeminiFlashThinking => "gemini-2.0-flash-thinking-exp".to_string(),
            Self::GeminiUltra => "gemini-ultra".to_string(),
            Self::Claude3Opus => "claude-3-opus@20240229".to_string(),
            Self::Claude3Sonnet => "claude-3-sonnet@20240229".to_string(),
            Self::Claude3Haiku => "claude-3-haiku@20240307".to_string(),
            Self::Llama3_70B => "meta/llama3-70b-instruct-maas".to_string(),
            Self::Llama3_8B => "meta/llama3-8b-instruct-maas".to_string(),
            Self::Jamba15Large => "ai21/jamba-1.5-large".to_string(),
            Self::Jamba15Mini => "ai21/jamba-1.5-mini".to_string(),
            Self::Custom(id) => id.clone(),
        }
    }

    /// Check if this is a Gemini model
    pub fn is_gemini(&self) -> bool {
        matches!(
            self,
            Self::GeminiPro
                | Self::GeminiProVision
                | Self::GeminiFlash
                | Self::GeminiFlashThinking
                | Self::GeminiUltra
        )
    }

    /// Check if this is a partner model
    pub fn is_partner_model(&self) -> bool {
        matches!(
            self,
            Self::Claude3Opus
                | Self::Claude3Sonnet
                | Self::Claude3Haiku
                | Self::Llama3_70B
                | Self::Llama3_8B
                | Self::Jamba15Large
                | Self::Jamba15Mini
        )
    }

    /// Check if model supports vision/multimodal
    pub fn supports_vision(&self) -> bool {
        matches!(
            self,
            Self::GeminiProVision | Self::GeminiFlash | Self::GeminiPro
        )
    }

    /// Check if model supports system messages
    pub fn supports_system_messages(&self) -> bool {
        match self {
            Self::GeminiPro | Self::GeminiFlash | Self::GeminiFlashThinking | Self::GeminiUltra => {
                true
            }
            Self::Claude3Opus | Self::Claude3Sonnet | Self::Claude3Haiku => true,
            Self::Llama3_70B | Self::Llama3_8B => true,
            _ => false,
        }
    }

    /// Check if model supports response schema/JSON mode
    pub fn supports_response_schema(&self) -> bool {
        matches!(
            self,
            Self::GeminiPro | Self::GeminiFlash | Self::GeminiFlashThinking | Self::GeminiUltra
        )
    }

    /// Check if model supports function calling
    pub fn supports_function_calling(&self) -> bool {
        self.is_gemini()
    }

    /// Get maximum context window
    pub fn max_context_tokens(&self) -> usize {
        match self {
            Self::GeminiPro => 2_097_152,   // 2M tokens
            Self::GeminiFlash => 1_048_576, // 1M tokens
            Self::GeminiFlashThinking => 1_048_576,
            Self::GeminiProVision => 2_097_152,
            Self::GeminiUltra => 1_048_576,
            Self::Claude3Opus => 200_000,
            Self::Claude3Sonnet => 200_000,
            Self::Claude3Haiku => 200_000,
            Self::Llama3_70B => 32_768,
            Self::Llama3_8B => 8_192,
            Self::Jamba15Large => 256_000,
            Self::Jamba15Mini => 256_000,
            Self::Custom(_) => 32_768, // Default
        }
    }
}

/// Parse model string to VertexAIModel enum
pub fn parse_vertex_model(model: &str) -> VertexAIModel {
    match model {
        "gemini-pro" | "gemini-1.5-pro" | "gemini-1.5-pro-001" => VertexAIModel::GeminiPro,
        "gemini-pro-vision" | "gemini-1.5-pro-vision" => VertexAIModel::GeminiProVision,
        "gemini-flash" | "gemini-1.5-flash" | "gemini-1.5-flash-001" => VertexAIModel::GeminiFlash,
        "gemini-2.0-flash-thinking-exp" => VertexAIModel::GeminiFlashThinking,
        "gemini-ultra" => VertexAIModel::GeminiUltra,

        model if model.contains("claude-3-opus") => VertexAIModel::Claude3Opus,
        model if model.contains("claude-3-sonnet") => VertexAIModel::Claude3Sonnet,
        model if model.contains("claude-3-haiku") => VertexAIModel::Claude3Haiku,

        model if model.contains("llama3-70b") => VertexAIModel::Llama3_70B,
        model if model.contains("llama3-8b") => VertexAIModel::Llama3_8B,

        model if model.contains("jamba-1.5-large") => VertexAIModel::Jamba15Large,
        model if model.contains("jamba-1.5-mini") => VertexAIModel::Jamba15Mini,

        _ => VertexAIModel::Custom(model.to_string()),
    }
}
