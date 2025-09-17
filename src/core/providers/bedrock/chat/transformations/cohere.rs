//! Cohere Model Transformations

use serde_json::{json, Value};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::requests::ChatRequest;
use crate::core::providers::bedrock::model_config::ModelConfig;

/// Transform request for Cohere models
pub fn transform_request(
    request: &ChatRequest,
    _model_config: &ModelConfig,
) -> Result<Value, ProviderError> {
    // Newer Cohere models (Command R) support chat format
    if request.model.contains("command-r") {
        transform_command_r_request(request)
    } else {
        // Older Cohere models use prompt format
        transform_command_request(request)
    }
}

/// Transform request for Command R models (chat format)
fn transform_command_r_request(request: &ChatRequest) -> Result<Value, ProviderError> {
    use crate::core::types::{MessageRole, MessageContent};

    let mut chat_history = Vec::new();
    let mut message = String::new();
    let mut preamble = None;

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

        match msg.role {
            MessageRole::System => {
                preamble = Some(content);
            }
            MessageRole::User => {
                // If there's a previous message, add it to history
                if !message.is_empty() {
                    chat_history.push(json!({
                        "role": "USER",
                        "message": message.clone()
                    }));
                }
                message = content;
            }
            MessageRole::Assistant => {
                // Add user message to history if exists
                if !message.is_empty() {
                    chat_history.push(json!({
                        "role": "USER",
                        "message": message.clone()
                    }));
                    message.clear();
                }
                // Add assistant message
                chat_history.push(json!({
                    "role": "CHATBOT",
                    "message": content
                }));
            }
            _ => {}
        }
    }

    let mut body = json!({
        "message": message,
        "max_tokens": request.max_tokens.unwrap_or(4096),
    });

    if !chat_history.is_empty() {
        body["chat_history"] = json!(chat_history);
    }

    if let Some(preamble_text) = preamble {
        body["preamble"] = json!(preamble_text);
    }

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        body["p"] = json!(top_p);
    }

    if let Some(stop) = &request.stop {
        body["stop_sequences"] = json!(stop);
    }

    Ok(body)
}

/// Transform request for older Command models (prompt format)
fn transform_command_request(request: &ChatRequest) -> Result<Value, ProviderError> {
    let prompt = super::messages_to_prompt(&request.messages);

    let mut body = json!({
        "prompt": prompt,
        "max_tokens": request.max_tokens.unwrap_or(4096),
    });

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        body["p"] = json!(top_p);
    }

    if let Some(stop) = &request.stop {
        body["stop_sequences"] = json!(stop);
    }

    Ok(body)
}