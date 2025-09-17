//! Mistral Model Transformations

use serde_json::{json, Value};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::requests::ChatRequest;
use crate::core::providers::bedrock::model_config::ModelConfig;

/// Transform request for Mistral models
pub fn transform_request(
    request: &ChatRequest,
    _model_config: &ModelConfig,
) -> Result<Value, ProviderError> {
    let prompt = format_mistral_prompt(&request.messages);

    let mut body = json!({
        "prompt": prompt,
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

/// Format messages for Mistral prompt format
fn format_mistral_prompt(messages: &[crate::core::types::ChatMessage]) -> String {
    use crate::core::types::{MessageRole, MessageContent};

    let mut prompt = String::new();
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
                    prompt.push_str(&format!("<s>[INST] {} [/INST]", format!("{}\n\n{}", sys, content)));
                    system_prompt = None; // Use system prompt only once
                } else {
                    prompt.push_str(&format!("<s>[INST] {} [/INST]", content));
                }
            }
            MessageRole::Assistant => {
                prompt.push_str(&format!("{}</s>", content));
            }
            _ => {}
        }
    }

    // Add instruction tag for the model to continue
    if !prompt.is_empty() && !prompt.ends_with("[/INST]") {
        prompt.push_str("<s>[INST]");
    }

    prompt
}