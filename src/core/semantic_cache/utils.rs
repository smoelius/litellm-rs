//! Utility functions for semantic caching

use crate::core::models::openai::{ChatMessage, ContentPart, MessageContent};

/// Extract prompt text from messages
pub fn extract_prompt_text(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .filter_map(|msg| match &msg.content {
            Some(MessageContent::Text(text)) => Some(text.clone()),
            Some(MessageContent::Parts(parts)) => {
                let text = parts
                    .iter()
                    .filter_map(|part| match part {
                        ContentPart::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<String>>()
                    .join(" ");
                if text.is_empty() { None } else { Some(text) }
            }
            None => None,
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Hash a prompt for quick lookup
pub fn hash_prompt(prompt: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    format!("{:x}", hasher.finalize())
}
