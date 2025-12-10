//! Test fixtures and data factories
//!
//! Provides factory methods for creating test data with sensible defaults.
//! All factories create real objects, not mocks.

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

/// Factory for creating test users
pub struct UserFactory;

impl UserFactory {
    /// Create a basic test user
    pub fn create() -> TestUser {
        TestUser {
            id: Uuid::new_v4().to_string(),
            username: format!("user_{}", &Uuid::new_v4().to_string()[..8]),
            email: format!("test-{}@example.com", &Uuid::new_v4().to_string()[..8]),
            display_name: Some("Test User".to_string()),
            password_hash: "hashed_password".to_string(),
            role: "user".to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
        }
    }

    /// Create an admin user
    pub fn admin() -> TestUser {
        let mut user = Self::create();
        user.username = format!("admin_{}", &Uuid::new_v4().to_string()[..8]);
        user.role = "admin".to_string();
        user
    }

    /// Create a user with specific email
    pub fn with_email(email: &str) -> TestUser {
        let mut user = Self::create();
        user.email = email.to_string();
        user
    }

    /// Create a pending (unverified) user
    pub fn pending() -> TestUser {
        let mut user = Self::create();
        user.status = "pending".to_string();
        user
    }
}

/// Test user data structure
#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub password_hash: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// Factory for creating chat requests
pub struct ChatRequestFactory;

impl ChatRequestFactory {
    /// Create a simple chat request with user message
    pub fn simple(model: &str, content: &str) -> SimpleChatRequest {
        SimpleChatRequest {
            model: model.to_string(),
            content: content.to_string(),
            stream: false,
        }
    }

    /// Create a streaming chat request
    pub fn streaming(model: &str, content: &str) -> SimpleChatRequest {
        SimpleChatRequest {
            model: model.to_string(),
            content: content.to_string(),
            stream: true,
        }
    }
}

/// Simple chat request for testing
#[derive(Debug, Clone)]
pub struct SimpleChatRequest {
    pub model: String,
    pub content: String,
    pub stream: bool,
}

/// Factory for creating API keys
pub struct ApiKeyFactory;

impl ApiKeyFactory {
    /// Create a basic test API key
    pub fn create() -> TestApiKey {
        TestApiKey {
            id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            name: "Test API Key".to_string(),
            key_hash: format!("hash_{}", Uuid::new_v4()),
            key: format!("sk-{}", &Uuid::new_v4().to_string().replace("-", "")[..32]),
            permissions: vec!["chat:read".to_string(), "chat:write".to_string()],
            rate_limit_rpm: Some(100),
            rate_limit_tpm: Some(10000),
            expires_at: None,
            created_at: Utc::now(),
        }
    }

    /// Create an expired API key
    pub fn expired() -> TestApiKey {
        let mut key = Self::create();
        key.expires_at = Some(Utc::now() - Duration::hours(1));
        key
    }

    /// Create an API key expiring in the future
    pub fn expiring_in(hours: i64) -> TestApiKey {
        let mut key = Self::create();
        key.expires_at = Some(Utc::now() + Duration::hours(hours));
        key
    }
}

/// Test API key data structure
#[derive(Debug, Clone)]
pub struct TestApiKey {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub key_hash: String,
    pub key: String,
    pub permissions: Vec<String>,
    pub rate_limit_rpm: Option<i32>,
    pub rate_limit_tpm: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_factory() {
        let user = UserFactory::create();
        assert!(!user.id.is_empty());
        assert!(user.email.contains('@'));
        assert_eq!(user.role, "user");
        assert_eq!(user.status, "active");
    }

    #[test]
    fn test_admin_factory() {
        let admin = UserFactory::admin();
        assert_eq!(admin.role, "admin");
    }

    #[test]
    fn test_chat_request_factory() {
        let request = ChatRequestFactory::simple("gpt-4", "Hello");
        assert_eq!(request.model, "gpt-4");
        assert!(!request.stream);
    }

    #[test]
    fn test_streaming_request() {
        let request = ChatRequestFactory::streaming("gpt-4", "Hello");
        assert!(request.stream);
    }

    #[test]
    fn test_api_key_factory() {
        let key = ApiKeyFactory::create();
        assert!(key.key.starts_with("sk-"));
        assert_eq!(key.key.len(), 35); // "sk-" + 32 chars
    }

    #[test]
    fn test_expired_key() {
        let key = ApiKeyFactory::expired();
        assert!(key.expires_at.unwrap() < Utc::now());
    }
}
