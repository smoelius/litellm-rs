use crate::core::providers::unified_provider::ProviderError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: ToolFunction,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<MessageContent>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<Value>>,
    pub tool_choice: Option<String>,
}

pub struct RequestUtils;

impl RequestUtils {
    pub fn validate_chat_completion_messages(
        messages: &[MessageContent],
    ) -> Result<(), ProviderError> {
        if messages.is_empty() {
            return Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: "Messages array cannot be empty".to_string(),
            });
        }

        for (i, message) in messages.iter().enumerate() {
            Self::validate_single_message(message, i)?;
        }

        Self::validate_message_sequence(messages)?;
        Ok(())
    }

    fn validate_single_message(
        message: &MessageContent,
        index: usize,
    ) -> Result<(), ProviderError> {
        let valid_roles = ["system", "user", "assistant", "function", "tool"];

        if !valid_roles.contains(&message.role.as_str()) {
            return Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: format!("Invalid role '{}' at message index {}", message.role, index),
            });
        }

        if message.content.is_empty() && message.role != "tool" {
            return Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: format!("Message content cannot be empty at index {}", index),
            });
        }

        if message.content.len() > 100000 {
            return Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: format!(
                    "Message content too long at index {} (max 100k chars)",
                    index
                ),
            });
        }

        Ok(())
    }

    fn validate_message_sequence(messages: &[MessageContent]) -> Result<(), ProviderError> {
        let mut has_user_message = false;

        for message in messages {
            if message.role == "user" {
                has_user_message = true;
                break;
            }
        }

        if !has_user_message {
            return Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: "At least one user message is required".to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_and_fix_openai_messages(
        messages: &mut [MessageContent],
    ) -> Result<(), ProviderError> {
        for message in messages.iter_mut() {
            Self::cleanup_none_fields_in_message(message);
        }

        Self::validate_chat_completion_messages(messages)?;
        Ok(())
    }

    fn cleanup_none_fields_in_message(message: &mut MessageContent) {
        message.content = message.content.trim().to_string();
    }

    pub fn validate_and_fix_openai_tools(
        tools: &mut Option<Vec<Value>>,
    ) -> Result<(), ProviderError> {
        if let Some(tools_vec) = tools {
            for (i, tool) in tools_vec.iter().enumerate() {
                if !tool.is_object() {
                    return Err(ProviderError::InvalidRequest {
                        provider: "unknown",
                        message: format!("Tool at index {} must be an object", i),
                    });
                }

                let tool_obj = tool.as_object().unwrap();

                if !tool_obj.contains_key("type") {
                    return Err(ProviderError::InvalidRequest {
                        provider: "unknown",
                        message: format!("Tool at index {} missing required 'type' field", i),
                    });
                }

                if !tool_obj.contains_key("function") {
                    return Err(ProviderError::InvalidRequest {
                        provider: "unknown",
                        message: format!("Tool at index {} missing required 'function' field", i),
                    });
                }

                let function = tool_obj.get("function").unwrap();
                if !function.is_object() {
                    return Err(ProviderError::InvalidRequest {
                        provider: "unknown",
                        message: format!("Tool function at index {} must be an object", i),
                    });
                }

                let func_obj = function.as_object().unwrap();
                if !func_obj.contains_key("name") {
                    return Err(ProviderError::InvalidRequest {
                        provider: "unknown",
                        message: format!(
                            "Tool function at index {} missing required 'name' field",
                            i
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn validate_tool_choice(
        tool_choice: &Option<String>,
        tools: &Option<Vec<Value>>,
    ) -> Result<(), ProviderError> {
        if let Some(choice) = tool_choice {
            if tools.is_none() || tools.as_ref().unwrap().is_empty() {
                return Err(ProviderError::InvalidRequest {
                    provider: "unknown",
                    message: "tool_choice requires tools to be provided".to_string(),
                });
            }

            match choice.as_str() {
                "none" | "auto" => {}
                _ => {
                    if !Self::is_valid_tool_name(choice, tools.as_ref().unwrap()) {
                        return Err(ProviderError::InvalidRequest {
                            provider: "unknown",
                            message: format!(
                                "Invalid tool_choice '{}'. Must be 'none', 'auto', or a valid tool name",
                                choice
                            ),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn is_valid_tool_name(tool_name: &str, tools: &[Value]) -> bool {
        tools.iter().any(|tool| {
            if let Some(function) = tool.get("function") {
                if let Some(name) = function.get("name") {
                    return name.as_str() == Some(tool_name);
                }
            }
            false
        })
    }

    pub fn process_system_message(
        system_message: &str,
        max_tokens: Option<u32>,
        model: &str,
    ) -> Result<String, ProviderError> {
        let mut processed = system_message.to_string();

        if let Some(max_tokens) = max_tokens {
            if processed.len() > (max_tokens as usize * 4) {
                processed = Self::truncate_message(&processed, max_tokens as usize * 4)?;
            }
        }

        if Self::needs_model_specific_processing(model) {
            processed = Self::apply_model_specific_processing(&processed, model)?;
        }

        Ok(processed)
    }

    pub fn process_messages(
        messages: &mut Vec<MessageContent>,
        max_tokens: Option<u32>,
        model: &str,
    ) -> Result<(), ProviderError> {
        if let Some(token_limit) = max_tokens {
            Self::trim_messages_to_fit_limit(messages, token_limit, model)?;
        }

        for message in messages.iter_mut() {
            if Self::needs_model_specific_processing(model) {
                message.content = Self::apply_model_specific_processing(&message.content, model)?;
            }
        }

        Ok(())
    }

    fn trim_messages_to_fit_limit(
        messages: &mut Vec<MessageContent>,
        max_tokens: u32,
        model: &str,
    ) -> Result<(), ProviderError> {
        let estimated_tokens = Self::estimate_total_tokens(messages, model);

        if estimated_tokens <= max_tokens as usize {
            return Ok(());
        }

        while messages.len() > 1
            && Self::estimate_total_tokens(messages, model) > max_tokens as usize
        {
            messages.remove(0);
        }

        if Self::estimate_total_tokens(messages, model) > max_tokens as usize {
            if let Some(last_message) = messages.last_mut() {
                let target_length = (max_tokens as usize * 3).saturating_sub(100);
                last_message.content =
                    Self::truncate_message(&last_message.content, target_length)?;
            }
        }

        Ok(())
    }

    fn estimate_total_tokens(messages: &[MessageContent], _model: &str) -> usize {
        messages
            .iter()
            .map(|msg| msg.content.split_whitespace().count() + 10)
            .sum()
    }

    fn truncate_message(message: &str, max_length: usize) -> Result<String, ProviderError> {
        if message.len() <= max_length {
            return Ok(message.to_string());
        }

        let mut truncated = message.chars().take(max_length - 3).collect::<String>();
        truncated.push_str("...");
        Ok(truncated)
    }

    fn needs_model_specific_processing(model: &str) -> bool {
        let model_lower = model.to_lowercase();
        model_lower.contains("claude") || model_lower.contains("palm")
    }

    fn apply_model_specific_processing(
        content: &str,
        model: &str,
    ) -> Result<String, ProviderError> {
        let model_lower = model.to_lowercase();

        if model_lower.contains("claude") {
            Ok(Self::process_for_claude(content))
        } else if model_lower.contains("palm") {
            Ok(Self::process_for_palm(content))
        } else {
            Ok(content.to_string())
        }
    }

    fn process_for_claude(content: &str) -> String {
        content
            .replace("Assistant:", "")
            .replace("Human:", "")
            .trim()
            .to_string()
    }

    fn process_for_palm(content: &str) -> String {
        content.trim().to_string()
    }

    pub fn add_dummy_tool(provider: &str) -> Vec<Value> {
        match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => {
                vec![serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": "dummy_function",
                        "description": "A dummy function for testing",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "A dummy query parameter"
                                }
                            },
                            "required": ["query"]
                        }
                    }
                })]
            }
            _ => vec![],
        }
    }

    pub fn convert_messages_to_dict(messages: &[MessageContent]) -> Vec<Map<String, Value>> {
        messages
            .iter()
            .map(|msg| {
                let mut map = Map::new();
                map.insert("role".to_string(), Value::String(msg.role.clone()));
                map.insert("content".to_string(), Value::String(msg.content.clone()));
                map
            })
            .collect()
    }

    pub fn has_tool_call_blocks(messages: &[MessageContent]) -> bool {
        messages.iter().any(|msg| {
            msg.content.contains("tool_calls")
                || msg.content.contains("function_call")
                || msg.role == "tool"
        })
    }

    pub fn get_standard_openai_params(params: &HashMap<String, Value>) -> HashMap<String, Value> {
        let standard_params = [
            "model",
            "messages",
            "temperature",
            "max_tokens",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
            "stop",
            "stream",
            "tools",
            "tool_choice",
            "response_format",
        ];

        params
            .iter()
            .filter(|(key, _)| standard_params.contains(&key.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn get_non_default_completion_params(
        params: &HashMap<String, Value>,
    ) -> HashMap<String, Value> {
        let mut non_default = HashMap::new();

        if let Some(temp) = params.get("temperature") {
            if temp.as_f64() != Some(1.0) {
                non_default.insert("temperature".to_string(), temp.clone());
            }
        }

        if let Some(max_tokens) = params.get("max_tokens") {
            if max_tokens.as_u64().is_some() {
                non_default.insert("max_tokens".to_string(), max_tokens.clone());
            }
        }

        if let Some(top_p) = params.get("top_p") {
            if top_p.as_f64() != Some(1.0) {
                non_default.insert("top_p".to_string(), top_p.clone());
            }
        }

        non_default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_validation() {
        let valid_messages = vec![
            MessageContent {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            MessageContent {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
            },
        ];

        assert!(RequestUtils::validate_chat_completion_messages(&valid_messages).is_ok());

        let empty_messages: Vec<MessageContent> = vec![];
        assert!(RequestUtils::validate_chat_completion_messages(&empty_messages).is_err());
    }

    #[test]
    fn test_invalid_role() {
        let invalid_messages = vec![MessageContent {
            role: "invalid_role".to_string(),
            content: "Hello".to_string(),
        }];

        assert!(RequestUtils::validate_chat_completion_messages(&invalid_messages).is_err());
    }

    #[test]
    fn test_system_message_processing() {
        let system_msg = "You are a helpful assistant.";
        let processed = RequestUtils::process_system_message(system_msg, None, "gpt-4").unwrap();
        assert_eq!(processed, system_msg);

        let claude_processed =
            RequestUtils::process_system_message(system_msg, None, "claude-3").unwrap();
        assert_eq!(claude_processed, system_msg);
    }

    #[test]
    fn test_tool_choice_validation() {
        let tools = Some(vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "test_function",
                "description": "A test function"
            }
        })]);

        assert!(RequestUtils::validate_tool_choice(&Some("auto".to_string()), &tools).is_ok());
        assert!(RequestUtils::validate_tool_choice(&Some("none".to_string()), &tools).is_ok());
        assert!(
            RequestUtils::validate_tool_choice(&Some("test_function".to_string()), &tools).is_ok()
        );
        assert!(
            RequestUtils::validate_tool_choice(&Some("invalid_function".to_string()), &tools)
                .is_err()
        );
        assert!(RequestUtils::validate_tool_choice(&Some("auto".to_string()), &None).is_err());
    }

    #[test]
    fn test_message_truncation() {
        let long_message = "a".repeat(1000);
        let truncated = RequestUtils::truncate_message(&long_message, 100).unwrap();
        assert!(truncated.len() <= 100);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_has_tool_call_blocks() {
        let messages_with_tools = vec![MessageContent {
            role: "assistant".to_string(),
            content: "Here's a tool_calls example".to_string(),
        }];

        let messages_without_tools = vec![MessageContent {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        assert!(RequestUtils::has_tool_call_blocks(&messages_with_tools));
        assert!(!RequestUtils::has_tool_call_blocks(&messages_without_tools));
    }

    #[test]
    fn test_convert_messages_to_dict() {
        let messages = vec![MessageContent {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let dict = RequestUtils::convert_messages_to_dict(&messages);
        assert_eq!(dict.len(), 1);
        assert_eq!(dict[0].get("role").unwrap().as_str().unwrap(), "user");
        assert_eq!(dict[0].get("content").unwrap().as_str().unwrap(), "Hello");
    }
}
