//! AI21 Labs Model Transformations

use serde_json::{json, Value};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::requests::ChatRequest;
use crate::core::providers::bedrock::model_config::ModelConfig;

/// Transform request for AI21 models
pub fn transform_request(
    request: &ChatRequest,
    _model_config: &ModelConfig,
) -> Result<Value, ProviderError> {
    // AI21 Jamba models use their own format
    if request.model.contains("jamba") {
        transform_jamba_request(request)
    } else {
        // Older Jurassic models
        transform_jurassic_request(request)
    }
}

/// Transform request for Jamba models
fn transform_jamba_request(request: &ChatRequest) -> Result<Value, ProviderError> {
    use crate::core::types::{MessageRole, MessageContent};

    let mut messages = Vec::new();

    for msg in &request.messages {
        let content = match &msg.content {
            Some(MessageContent::Text(text)) => text.clone(),
            Some(MessageContent::Parts(parts)) => {
                parts.iter()
                    .filter_map(|part| {
                        if let crate::core::types::requests::ContentPart::Text { text } = part {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            None => continue,
        };

        let role = match msg.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            _ => continue,
        };

        messages.push(json!({
            "role": role,
            "content": content
        }));
    }

    let mut body = json!({
        "messages": messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
    });

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        body["top_p"] = json!(top_p);
    }

    if let Some(stop) = &request.stop {
        body["stop"] = json!(stop);
    }

    Ok(body)
}

/// Transform request for Jurassic models
fn transform_jurassic_request(request: &ChatRequest) -> Result<Value, ProviderError> {
    let prompt = super::messages_to_prompt(&request.messages);

    let mut body = json!({
        "prompt": prompt,
        "maxTokens": request.max_tokens.unwrap_or(4096),
    });

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        body["topP"] = json!(top_p);
    }

    if let Some(stop) = &request.stop {
        body["stopSequences"] = json!(stop);
    }

    Ok(body)
}