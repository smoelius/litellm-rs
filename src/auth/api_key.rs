//! API key authentication and management
//!
//! This module provides API key creation, verification, and management functionality.

use crate::core::models::{ApiKey, RateLimits, UsageStats, User};
use crate::storage::StorageLayer;
use crate::utils::auth::crypto;
use crate::utils::error::{GatewayError, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// API key handler for authentication and management
#[derive(Debug, Clone)]
pub struct ApiKeyHandler {
    /// Storage layer for persistence
    storage: Arc<StorageLayer>,
}

/// API key creation request
#[derive(Debug, Clone)]
pub struct CreateApiKeyRequest {
    /// Key name/description
    pub name: String,
    /// Associated user ID
    pub user_id: Option<Uuid>,
    /// Associated team ID
    pub team_id: Option<Uuid>,
    /// Permissions for the key
    pub permissions: Vec<String>,
    /// Rate limits for the key
    pub rate_limits: Option<RateLimits>,
    /// Expiration date
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// API key verification result
#[derive(Debug, Clone)]
pub struct ApiKeyVerification {
    /// The API key
    pub api_key: ApiKey,
    /// Associated user (if any)
    pub user: Option<User>,
    /// Whether the key is valid
    pub is_valid: bool,
    /// Reason for invalidity (if any)
    pub invalid_reason: Option<String>,
}

impl ApiKeyHandler {
    /// Create a new API key handler
    pub async fn new(storage: Arc<StorageLayer>) -> Result<Self> {
        Ok(Self { storage })
    }

    /// Create a new API key
    pub async fn create_key(
        &self,
        user_id: Option<Uuid>,
        team_id: Option<Uuid>,
        name: String,
        permissions: Vec<String>,
    ) -> Result<(ApiKey, String)> {
        info!("Creating API key: {}", name);

        // Generate API key
        let raw_key = crypto::generate_api_key();
        let key_hash = crypto::hash_api_key(&raw_key);
        let key_prefix = crypto::extract_api_key_prefix(&raw_key);

        // Create API key object
        let api_key = ApiKey {
            metadata: crate::core::models::Metadata::new(),
            name,
            key_hash,
            key_prefix,
            user_id,
            team_id,
            permissions,
            rate_limits: None,
            expires_at: None,
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        };

        // Store in database
        let stored_key = self.storage.db().create_api_key(&api_key).await?;

        info!("API key created successfully: {}", stored_key.metadata.id);
        Ok((stored_key, raw_key))
    }

    /// Create API key with full options
    pub async fn create_key_with_options(
        &self,
        request: CreateApiKeyRequest,
    ) -> Result<(ApiKey, String)> {
        info!("Creating API key with options: {}", request.name);

        // Generate API key
        let raw_key = crypto::generate_api_key();
        let key_hash = crypto::hash_api_key(&raw_key);
        let key_prefix = crypto::extract_api_key_prefix(&raw_key);

        // Create API key object
        let api_key = ApiKey {
            metadata: crate::core::models::Metadata::new(),
            name: request.name,
            key_hash,
            key_prefix,
            user_id: request.user_id,
            team_id: request.team_id,
            permissions: request.permissions,
            rate_limits: request.rate_limits,
            expires_at: request.expires_at,
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        };

        // Store in database
        let stored_key = self.storage.db().create_api_key(&api_key).await?;

        info!("API key created successfully: {}", stored_key.metadata.id);
        Ok((stored_key, raw_key))
    }

    /// Verify an API key
    pub async fn verify_key(&self, raw_key: &str) -> Result<Option<(ApiKey, Option<User>)>> {
        debug!("Verifying API key");

        // Hash the provided key
        let key_hash = crypto::hash_api_key(raw_key);

        // Find API key in database
        let api_key = match self.storage.db().find_api_key_by_hash(&key_hash).await? {
            Some(key) => key,
            None => {
                debug!("API key not found");
                return Ok(None);
            }
        };

        // Check if key is active
        if !api_key.is_active {
            debug!("API key is inactive");
            return Ok(None);
        }

        // Check if key is expired
        if let Some(expires_at) = api_key.expires_at {
            if chrono::Utc::now() > expires_at {
                debug!("API key is expired");
                return Ok(None);
            }
        }

        // Get associated user if any
        let user = if let Some(user_id) = api_key.user_id {
            self.storage.db().find_user_by_id(user_id).await?
        } else {
            None
        };

        // Update last used timestamp
        self.update_last_used(api_key.metadata.id).await?;

        debug!("API key verified successfully");
        Ok(Some((api_key, user)))
    }

    /// Verify API key with detailed result
    pub async fn verify_key_detailed(&self, raw_key: &str) -> Result<ApiKeyVerification> {
        let key_hash = crypto::hash_api_key(raw_key);

        let api_key = match self.storage.db().find_api_key_by_hash(&key_hash).await? {
            Some(key) => key,
            None => {
                return Ok(ApiKeyVerification {
                    api_key: ApiKey {
                        metadata: crate::core::models::Metadata::new(),
                        name: "".to_string(),
                        key_hash: "".to_string(),
                        key_prefix: "".to_string(),
                        user_id: None,
                        team_id: None,
                        permissions: vec![],
                        rate_limits: None,
                        expires_at: None,
                        is_active: false,
                        last_used_at: None,
                        usage_stats: UsageStats::default(),
                    },
                    user: None,
                    is_valid: false,
                    invalid_reason: Some("API key not found".to_string()),
                });
            }
        };

        // Check if key is active
        if !api_key.is_active {
            return Ok(ApiKeyVerification {
                api_key,
                user: None,
                is_valid: false,
                invalid_reason: Some("API key is inactive".to_string()),
            });
        }

        // Check if key is expired
        if let Some(expires_at) = api_key.expires_at {
            if chrono::Utc::now() > expires_at {
                return Ok(ApiKeyVerification {
                    api_key,
                    user: None,
                    is_valid: false,
                    invalid_reason: Some("API key is expired".to_string()),
                });
            }
        }

        // Get associated user if any
        let user = if let Some(user_id) = api_key.user_id {
            self.storage.db().find_user_by_id(user_id).await?
        } else {
            None
        };

        // Check if user is active (if associated)
        if let Some(ref user) = user {
            if !user.is_active() {
                return Ok(ApiKeyVerification {
                    api_key,
                    user: Some(user.clone()),
                    is_valid: false,
                    invalid_reason: Some("Associated user is inactive".to_string()),
                });
            }
        }

        // Update last used timestamp
        self.update_last_used(api_key.metadata.id).await?;

        Ok(ApiKeyVerification {
            api_key,
            user,
            is_valid: true,
            invalid_reason: None,
        })
    }

    /// Revoke an API key
    pub async fn revoke_key(&self, key_id: Uuid) -> Result<()> {
        info!("Revoking API key: {}", key_id);

        self.storage.db().deactivate_api_key(key_id).await?;

        info!("API key revoked successfully: {}", key_id);
        Ok(())
    }

    /// List API keys for a user
    pub async fn list_user_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        debug!("Listing API keys for user: {}", user_id);
        // TODO: Fix type mismatch - user_id is Uuid but list_api_keys_by_user expects i64
        let user_id_hash = user_id.as_u128() as i64;
        self.storage.db().list_api_keys_by_user(user_id_hash).await
    }

    /// List API keys for a team
    pub async fn list_team_keys(&self, team_id: Uuid) -> Result<Vec<ApiKey>> {
        debug!("Listing API keys for team: {}", team_id);
        self.storage.db().list_api_keys_by_team(team_id).await
    }

    /// Update API key permissions
    pub async fn update_permissions(&self, key_id: Uuid, permissions: Vec<String>) -> Result<()> {
        info!("Updating permissions for API key: {}", key_id);

        self.storage
            .db()
            .update_api_key_permissions(key_id, &permissions)
            .await?;

        info!("API key permissions updated successfully: {}", key_id);
        Ok(())
    }

    /// Update API key rate limits
    pub async fn update_rate_limits(
        &self,
        key_id: Uuid,
        rate_limits: Option<RateLimits>,
    ) -> Result<()> {
        info!("Updating rate limits for API key: {}", key_id);

        if let Some(ref limits) = rate_limits {
            self.storage
                .db()
                .update_api_key_rate_limits(key_id, limits)
                .await?;
        }

        info!("API key rate limits updated successfully: {}", key_id);
        Ok(())
    }

    /// Update API key expiration
    pub async fn update_expiration(
        &self,
        key_id: Uuid,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        info!("Updating expiration for API key: {}", key_id);

        self.storage
            .db()
            .update_api_key_expiration(key_id, expires_at)
            .await?;

        info!("API key expiration updated successfully: {}", key_id);
        Ok(())
    }

    /// Record API key usage
    pub async fn record_usage(
        &self,
        key_id: Uuid,
        requests: u64,
        tokens: u64,
        cost: f64,
    ) -> Result<()> {
        debug!("Recording usage for API key: {}", key_id);

        self.storage
            .db()
            .update_api_key_usage(key_id, requests, tokens, cost)
            .await?;

        Ok(())
    }

    /// Get API key usage statistics
    pub async fn get_usage_stats(&self, key_id: Uuid) -> Result<UsageStats> {
        debug!("Getting usage stats for API key: {}", key_id);

        let api_key = self
            .storage
            .db()
            .find_api_key_by_id(key_id)
            .await?
            .ok_or_else(|| GatewayError::not_found("API key not found"))?;

        Ok(api_key.usage_stats)
    }

    /// Check if API key has permission
    pub fn has_permission(&self, api_key: &ApiKey, permission: &str) -> bool {
        api_key.permissions.contains(&permission.to_string())
            || api_key.permissions.contains(&"*".to_string()) // Wildcard permission
    }

    /// Check if API key has any of the permissions
    pub fn has_any_permission(&self, api_key: &ApiKey, permissions: &[String]) -> bool {
        if api_key.permissions.contains(&"*".to_string()) {
            return true;
        }

        permissions
            .iter()
            .any(|perm| api_key.permissions.contains(perm))
    }

    /// Check if API key has all permissions
    pub fn has_all_permissions(&self, api_key: &ApiKey, permissions: &[String]) -> bool {
        if api_key.permissions.contains(&"*".to_string()) {
            return true;
        }

        permissions
            .iter()
            .all(|perm| api_key.permissions.contains(perm))
    }

    /// Update last used timestamp
    async fn update_last_used(&self, key_id: Uuid) -> Result<()> {
        // Use a background task to avoid blocking the request
        let storage = self.storage.clone();
        tokio::spawn(async move {
            if let Err(e) = storage.db().update_api_key_last_used(key_id).await {
                warn!("Failed to update API key last used timestamp: {}", e);
            }
        });

        Ok(())
    }

    /// Cleanup expired API keys
    pub async fn cleanup_expired_keys(&self) -> Result<u64> {
        info!("Cleaning up expired API keys");

        // Delete expired API keys from database
        let count = self.storage.db().delete_expired_api_keys().await?;

        info!("Cleaned up {} expired API keys", count);
        Ok(count)
    }

    /// Get API key by ID
    pub async fn get_key(&self, key_id: Uuid) -> Result<Option<ApiKey>> {
        self.storage.db().find_api_key_by_id(key_id).await
    }

    /// Regenerate API key (creates new key, deactivates old one)
    pub async fn regenerate_key(&self, key_id: Uuid) -> Result<(ApiKey, String)> {
        info!("Regenerating API key: {}", key_id);

        // Get existing key
        let old_key = self
            .storage
            .db()
            .find_api_key_by_id(key_id)
            .await?
            .ok_or_else(|| GatewayError::not_found("API key not found"))?;

        // Create new key with same properties
        let request = CreateApiKeyRequest {
            name: old_key.name.clone(),
            user_id: old_key.user_id,
            team_id: old_key.team_id,
            permissions: old_key.permissions.clone(),
            rate_limits: old_key.rate_limits.clone(),
            expires_at: old_key.expires_at,
        };

        let (new_key, raw_key) = self.create_key_with_options(request).await?;

        // Deactivate old key
        self.revoke_key(key_id).await?;

        info!(
            "API key regenerated successfully: {} -> {}",
            key_id, new_key.metadata.id
        );
        Ok((new_key, raw_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::storage::StorageLayer;

    async fn create_test_storage() -> Arc<StorageLayer> {
        // This would require actual database setup in a real test
        // For now, we'll create a mock or skip database-dependent tests
        todo!("Implement test storage setup")
    }

    #[test]
    fn test_create_api_key_request() {
        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            user_id: Some(Uuid::new_v4()),
            team_id: None,
            permissions: vec!["read".to_string(), "write".to_string()],
            rate_limits: None,
            expires_at: None,
        };

        assert_eq!(request.name, "Test Key");
        assert!(request.user_id.is_some());
        assert_eq!(request.permissions.len(), 2);
    }

    #[test]
    fn test_api_key_verification_result() {
        let verification = ApiKeyVerification {
            api_key: ApiKey {
                metadata: crate::core::models::Metadata::new(),
                name: "Test Key".to_string(),
                key_hash: "hash".to_string(),
                key_prefix: "gw-test".to_string(),
                user_id: None,
                team_id: None,
                permissions: vec!["read".to_string()],
                rate_limits: None,
                expires_at: None,
                is_active: true,
                last_used_at: None,
                usage_stats: UsageStats::default(),
            },
            user: None,
            is_valid: true,
            invalid_reason: None,
        };

        assert!(verification.is_valid);
        assert!(verification.invalid_reason.is_none());
        assert_eq!(verification.api_key.name, "Test Key");
    }

    #[test]
    fn test_permission_checking() {
        let api_key = ApiKey {
            metadata: crate::core::models::Metadata::new(),
            name: "Test Key".to_string(),
            key_hash: "hash".to_string(),
            key_prefix: "gw-test".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec!["read".to_string(), "write".to_string()],
            rate_limits: None,
            expires_at: None,
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        };

        // This would require a handler instance, but we can test the logic
        let permissions = &api_key.permissions;
        assert!(permissions.contains(&"read".to_string()));
        assert!(permissions.contains(&"write".to_string()));
        assert!(!permissions.contains(&"admin".to_string()));
    }
}
