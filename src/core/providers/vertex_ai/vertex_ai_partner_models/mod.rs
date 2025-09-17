//! Vertex AI Partner Models Module

pub mod ai21;
pub mod anthropic;
pub mod llama3;

use super::error::VertexAIError;
use serde::{Deserialize, Serialize};

/// Partner provider types
#[derive(Debug, Clone)]
pub enum PartnerProvider {
    AI21,
    Anthropic,
    Meta,
}

/// Partner model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerModelConfig {
    pub provider: String,
    pub model: String,
    pub parameters: serde_json::Value,
}

/// Main partner model handler
pub struct PartnerModelHandler;

impl PartnerModelHandler {
    /// Route request to appropriate partner handler
    pub async fn handle_request(
        provider: PartnerProvider,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, VertexAIError> {
        match provider {
            PartnerProvider::AI21 => ai21::AI21Handler::handle_request(request).await,
            PartnerProvider::Anthropic => {
                anthropic::AnthropicHandler::handle_request(request).await
            }
            PartnerProvider::Meta => llama3::Llama3Handler::handle_request(request).await,
        }
    }
}
