//! Helper functions for message creation

use crate::core::types::{ChatMessage, MessageContent, MessageRole};

/// Convert messages to chat messages (no-op since Message is an alias)
pub fn convert_messages_to_chat_messages(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    messages
}

/// Helper function to create user message
pub fn user_message(content: impl Into<String>) -> ChatMessage {
    ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Text(content.into())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

/// Helper function to create system message
pub fn system_message(content: impl Into<String>) -> ChatMessage {
    ChatMessage {
        role: MessageRole::System,
        content: Some(MessageContent::Text(content.into())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

/// Helper function to create assistant message
pub fn assistant_message(content: impl Into<String>) -> ChatMessage {
    ChatMessage {
        role: MessageRole::Assistant,
        content: Some(MessageContent::Text(content.into())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}
