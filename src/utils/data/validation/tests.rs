//! Tests for validation utilities

#[cfg(test)]
mod tests {
    use crate::core::models::openai::{ChatMessage, MessageContent, MessageRole};
    use crate::utils::data::validation::{ApiValidator, DataValidator, RequestValidator};

    #[test]
    fn test_model_name_validation() {
        // Test model name validation through validate_chat_completion_request
        let message = ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("test".to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
            audio: None,
        };

        assert!(RequestValidator::validate_chat_completion_request(
            "gpt-4",
            &[message.clone()],
            None,
            None
        )
        .is_ok());
        assert!(RequestValidator::validate_chat_completion_request(
            "claude-3.5-sonnet",
            &[message.clone()],
            None,
            None
        )
        .is_ok());
        assert!(RequestValidator::validate_chat_completion_request("", &[message.clone()], None, None)
            .is_err());
        assert!(RequestValidator::validate_chat_completion_request(
            "invalid@model",
            &[message],
            None,
            None
        )
        .is_err());
    }

    #[test]
    fn test_api_key_validation() {
        assert!(ApiValidator::validate_api_key("valid_api_key_123").is_ok());
        assert!(ApiValidator::validate_api_key("").is_err());
        assert!(ApiValidator::validate_api_key("short").is_err());
    }

    #[test]
    fn test_uuid_validation() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(ApiValidator::validate_uuid(valid_uuid).is_ok());
        assert!(ApiValidator::validate_uuid("invalid-uuid").is_err());
    }

    #[test]
    fn test_pagination_validation() {
        assert!(ApiValidator::validate_pagination(Some(1), Some(20)).is_ok());
        assert!(ApiValidator::validate_pagination(Some(0), Some(20)).is_err());
        assert!(ApiValidator::validate_pagination(Some(1), Some(0)).is_err());
        assert!(ApiValidator::validate_pagination(Some(1), Some(2000)).is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(DataValidator::validate_username("valid_user").is_ok());
        assert!(DataValidator::validate_username("user123").is_ok());
        assert!(DataValidator::validate_username("").is_err());
        assert!(DataValidator::validate_username("ab").is_err());
        assert!(DataValidator::validate_username("invalid@user").is_err());
    }

    #[test]
    fn test_password_validation() {
        assert!(DataValidator::validate_password("StrongPass123!").is_ok());
        assert!(DataValidator::validate_password("NoSpecialChars123").is_ok()); // Has 3 types: upper, lower, digit
        assert!(DataValidator::validate_password("weak").is_err()); // Too short
        assert!(DataValidator::validate_password("onlylowercase").is_err()); // Only 1 type
        assert!(DataValidator::validate_password("ONLYUPPERCASE").is_err()); // Only 1 type
        assert!(DataValidator::validate_password("OnlyTwoTypes").is_err()); // Only 2 types: upper, lower
    }

    #[test]
    fn test_tags_validation() {
        assert!(DataValidator::validate_tags(&["tag1".to_string(), "tag2".to_string()]).is_ok());
        assert!(DataValidator::validate_tags(&["".to_string()]).is_err());
        assert!(DataValidator::validate_tags(&["tag1".to_string(), "tag1".to_string()]).is_err());
    }
}
