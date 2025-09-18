//! Anthropic Claude Model Transformations

use crate::core::providers::bedrock::model_config::ModelConfig;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::requests::ChatRequest;
use serde_json::{Value, json};

/// Transform request for Anthropic Claude models
pub fn transform_request(
    request: &ChatRequest,
    _model_config: &ModelConfig,
) -> Result<Value, ProviderError> {
    // Claude models on Bedrock use anthropic messages format
    let mut body = json!({
        "messages": request.messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "anthropic_version": "bedrock-2023-05-20"
    });

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        body["top_p"] = json!(top_p);
    }

    if let Some(stop) = &request.stop {
        body["stop_sequences"] = json!(stop);
    }

    if let Some(system) = extract_system_message(request) {
        body["system"] = json!(system);
    }

    Ok(body)
}

/// Extract system message from chat messages
fn extract_system_message(request: &ChatRequest) -> Option<String> {
    use crate::core::types::{MessageContent, MessageRole};

    request
        .messages
        .iter()
        .find(|msg| msg.role == MessageRole::System)
        .and_then(|msg| msg.content.as_ref())
        .map(|content| match content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Parts(parts) => parts
                .iter()
                .filter_map(|part| {
                    if let crate::core::types::requests::ContentPart::Text { text } = part {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
        })
}
