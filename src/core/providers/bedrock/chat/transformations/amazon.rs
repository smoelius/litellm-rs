//! Amazon Titan and Nova Model Transformations

use serde_json::{json, Value};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::requests::ChatRequest;
use crate::core::providers::bedrock::model_config::ModelConfig;

/// Transform request for Amazon Titan models
pub fn transform_titan_request(
    request: &ChatRequest,
    _model_config: &ModelConfig,
) -> Result<Value, ProviderError> {
    let prompt = super::messages_to_prompt(&request.messages);

    let mut text_generation_config = json!({
        "maxTokenCount": request.max_tokens.unwrap_or(4096),
    });

    if let Some(temp) = request.temperature {
        text_generation_config["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        text_generation_config["topP"] = json!(top_p);
    }

    if let Some(stop) = &request.stop {
        text_generation_config["stopSequences"] = json!(stop);
    }

    Ok(json!({
        "inputText": prompt,
        "textGenerationConfig": text_generation_config
    }))
}

/// Transform request for Amazon Nova models
pub fn transform_nova_request(
    request: &ChatRequest,
    _model_config: &ModelConfig,
) -> Result<Value, ProviderError> {
    // Nova models use a format similar to Claude but with some differences
    let mut messages = Vec::new();
    let mut system = None;

    use crate::core::types::{MessageRole, MessageContent};

    for msg in &request.messages {
        match msg.role {
            MessageRole::System => {
                // Extract system message
                if let Some(content) = &msg.content {
                    system = Some(match content {
                        MessageContent::Text(text) => text.clone(),
                        MessageContent::Parts(parts) => {
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
                    });
                }
            }
            MessageRole::User | MessageRole::Assistant => {
                messages.push(json!({
                    "role": match msg.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        _ => continue,
                    },
                    "content": match &msg.content {
                        Some(MessageContent::Text(text)) => vec![json!({
                            "text": text
                        })],
                        Some(MessageContent::Parts(parts)) => {
                            parts.iter().filter_map(|part| {
                                match part {
                                    crate::core::types::requests::ContentPart::Text { text } => {
                                        Some(json!({"text": text}))
                                    }
                                    crate::core::types::requests::ContentPart::Image { .. } => {
                                        // TODO: Handle image content for Nova Canvas
                                        None
                                    }
                                    crate::core::types::requests::ContentPart::ImageUrl { .. } => {
                                        // TODO: Handle image URL content
                                        None
                                    }
                                    crate::core::types::requests::ContentPart::Audio { .. } => {
                                        // TODO: Handle audio content
                                        None
                                    }
                                    crate::core::types::requests::ContentPart::Document { .. } => {
                                        // TODO: Handle document content
                                        None
                                    }
                                    crate::core::types::requests::ContentPart::ToolResult { .. } => {
                                        // TODO: Handle tool result content
                                        None
                                    }
                                    crate::core::types::requests::ContentPart::ToolUse { .. } => {
                                        // TODO: Handle tool use content
                                        None
                                    }
                                }
                            }).collect()
                        }
                        None => vec![],
                    }
                }));
            }
            _ => {
                // Skip function/tool messages for now
            }
        }
    }

    let mut body = json!({
        "messages": messages,
    });

    if let Some(system_text) = system {
        body["system"] = json!([{
            "text": system_text
        }]);
    }

    // Add inference configuration
    let mut inference_config = json!({});

    if let Some(max_tokens) = request.max_tokens {
        inference_config["maxTokens"] = json!(max_tokens);
    }

    if let Some(temp) = request.temperature {
        inference_config["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        inference_config["topP"] = json!(top_p);
    }

    if let Some(stop) = &request.stop {
        inference_config["stopSequences"] = json!(stop);
    }

    if !inference_config.as_object().unwrap().is_empty() {
        body["inferenceConfig"] = inference_config;
    }

    Ok(body)
}