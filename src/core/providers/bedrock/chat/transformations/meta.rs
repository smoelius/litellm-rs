//! Meta Llama Model Transformations

use serde_json::{json, Value};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::requests::ChatRequest;
use crate::core::providers::bedrock::model_config::ModelConfig;

/// Transform request for Meta Llama models
pub fn transform_request(
    request: &ChatRequest,
    model_config: &ModelConfig,
) -> Result<Value, ProviderError> {
    // Newer Llama models support message format, older ones need prompt format
    if model_config.family == crate::core::providers::bedrock::model_config::BedrockModelFamily::Llama
        && request.model.contains("llama3") {
        // Llama 3 uses message format similar to Claude
        transform_llama3_request(request)
    } else {
        // Llama 2 uses prompt format
        transform_llama2_request(request)
    }
}

/// Transform request for Llama 3 models (message format)
fn transform_llama3_request(request: &ChatRequest) -> Result<Value, ProviderError> {
    let mut body = json!({
        "messages": request.messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
    });

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        body["top_p"] = json!(top_p);
    }

    Ok(body)
}

/// Transform request for Llama 2 models (prompt format)
fn transform_llama2_request(request: &ChatRequest) -> Result<Value, ProviderError> {
    let prompt = format_llama2_prompt(&request.messages);

    let mut body = json!({
        "prompt": prompt,
        "max_gen_len": request.max_tokens.unwrap_or(512),
    });

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }

    if let Some(top_p) = request.top_p {
        body["top_p"] = json!(top_p);
    }

    Ok(body)
}

/// Format messages for Llama 2 prompt format
fn format_llama2_prompt(messages: &[crate::core::types::ChatMessage]) -> String {
    use crate::core::types::{MessageRole, MessageContent};

    let mut prompt = String::from("<s>");
    let mut system_prompt = None;

    for message in messages {
        let content = match &message.content {
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

        match message.role {
            MessageRole::System => {
                system_prompt = Some(content);
            }
            MessageRole::User => {
                if let Some(sys) = &system_prompt {
                    prompt.push_str(&format!("[INST] <<SYS>>\n{}\n<</SYS>>\n\n{} [/INST]", sys, content));
                    system_prompt = None; // Use system prompt only once
                } else {
                    prompt.push_str(&format!("[INST] {} [/INST]", content));
                }
            }
            MessageRole::Assistant => {
                prompt.push_str(&format!(" {} </s><s>", content));
            }
            _ => {}
        }
    }

    prompt
}