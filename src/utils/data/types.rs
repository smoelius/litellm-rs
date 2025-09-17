//! Type-safe wrappers and utilities
//!
//! This module provides type-safe wrappers for common values to prevent
//! mixing up different types of IDs, keys, and other values.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use uuid::Uuid;

/// A type-safe wrapper for user IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    /// Create a new user ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a user ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Get the string representation
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UserId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}

/// A type-safe wrapper for API keys
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiKey(String);

impl ApiKey {
    /// Create a new API key from a string
    pub fn new(key: String) -> Self {
        Self(key)
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the string representation (for logging, truncated)
    pub fn as_display_str(&self) -> String {
        if self.0.len() > 8 {
            format!("{}...", &self.0[..8])
        } else {
            self.0.clone()
        }
    }

    /// Check if the key is valid (basic validation)
    pub fn is_valid(&self) -> bool {
        !self.0.is_empty() && self.0.len() >= 16
    }
}

impl Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_display_str())
    }
}

impl FromStr for ApiKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = Self(s.to_string());
        if key.is_valid() {
            Ok(key)
        } else {
            Err("Invalid API key format".to_string())
        }
    }
}

impl From<String> for ApiKey {
    fn from(key: String) -> Self {
        Self(key)
    }
}

impl From<ApiKey> for String {
    fn from(api_key: ApiKey) -> Self {
        api_key.0
    }
}

/// A type-safe wrapper for model names
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelName(String);

impl ModelName {
    /// Create a new model name
    pub fn new(name: String) -> Self {
        Self(name)
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract provider from model name (e.g., "openai/gpt-4" -> "openai")
    pub fn provider(&self) -> Option<&str> {
        self.0.find('/').map(|pos| &self.0[..pos])
    }

    /// Extract model from model name (e.g., "openai/gpt-4" -> "gpt-4")
    pub fn model(&self) -> &str {
        if let Some(pos) = self.0.find('/') {
            &self.0[pos + 1..]
        } else {
            &self.0
        }
    }

    /// Check if this is a valid model name
    pub fn is_valid(&self) -> bool {
        !self.0.is_empty() && !self.0.contains(' ')
    }
}

impl Display for ModelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ModelName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let model = Self(s.to_string());
        if model.is_valid() {
            Ok(model)
        } else {
            Err("Invalid model name format".to_string())
        }
    }
}

impl From<String> for ModelName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<ModelName> for String {
    fn from(model_name: ModelName) -> Self {
        model_name.0
    }
}

/// A type-safe wrapper for request IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(String);

impl RequestId {
    /// Create a new request ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a request ID from a string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RequestId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<RequestId> for String {
    fn from(request_id: RequestId) -> Self {
        request_id.0
    }
}

/// A type-safe wrapper for organization IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationId(Uuid);

impl OrganizationId {
    /// Create a new organization ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an organization ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Get the string representation
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for OrganizationId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for OrganizationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for OrganizationId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for OrganizationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<OrganizationId> for Uuid {
    fn from(org_id: OrganizationId) -> Self {
        org_id.0
    }
}

/// A type-safe wrapper for team IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamId(Uuid);

impl TeamId {
    /// Create a new team ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a team ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Get the string representation
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for TeamId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TeamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TeamId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for TeamId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<TeamId> for Uuid {
    fn from(team_id: TeamId) -> Self {
        team_id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id() {
        let user_id = UserId::new();
        let uuid = user_id.as_uuid();
        let user_id2 = UserId::from_uuid(uuid);
        assert_eq!(user_id, user_id2);

        let user_id_str = user_id.to_string();
        let user_id3: UserId = user_id_str.parse().unwrap();
        assert_eq!(user_id, user_id3);
    }

    #[test]
    fn test_api_key() {
        let key = ApiKey::new("sk-1234567890abcdef".to_string());
        assert!(key.is_valid());
        assert_eq!(key.as_str(), "sk-1234567890abcdef");
        assert_eq!(key.as_display_str(), "sk-12345...");

        let invalid_key = ApiKey::new("short".to_string());
        assert!(!invalid_key.is_valid());
    }

    #[test]
    fn test_model_name() {
        let model = ModelName::new("openai/gpt-4".to_string());
        assert!(model.is_valid());
        assert_eq!(model.provider(), Some("openai"));
        assert_eq!(model.model(), "gpt-4");

        let simple_model = ModelName::new("gpt-4".to_string());
        assert_eq!(simple_model.provider(), None);
        assert_eq!(simple_model.model(), "gpt-4");
    }

    #[test]
    fn test_request_id() {
        let req_id = RequestId::new();
        assert!(!req_id.as_str().is_empty());

        let custom_id = RequestId::from_string("custom-123".to_string());
        assert_eq!(custom_id.as_str(), "custom-123");
    }
}
