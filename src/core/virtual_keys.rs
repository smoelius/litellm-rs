//! Virtual Keys management system
//!
//! This module provides comprehensive virtual key management for the LiteLLM proxy.

use crate::storage::database::Database;
use crate::utils::error::{GatewayError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Virtual key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualKey {
    /// Unique key identifier
    pub key_id: String,
    /// The actual API key (hashed)
    pub key_hash: String,
    /// Human-readable key alias
    pub key_alias: Option<String>,
    /// User ID who owns this key
    pub user_id: String,
    /// Team ID (if applicable)
    pub team_id: Option<String>,
    /// Organization ID
    pub organization_id: Option<String>,
    /// Models this key can access
    pub models: Vec<String>,
    /// Maximum spend limit
    pub max_budget: Option<f64>,
    /// Current spend
    pub spend: f64,
    /// Budget duration (e.g., "1d", "1w", "1m")
    pub budget_duration: Option<String>,
    /// Budget reset timestamp
    pub budget_reset_at: Option<DateTime<Utc>>,
    /// Rate limits
    pub rate_limits: Option<RateLimits>,
    /// Key permissions
    pub permissions: Vec<Permission>,
    /// Key metadata
    pub metadata: HashMap<String, String>,
    /// Key expiration
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether key is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last used timestamp
    pub last_used_at: Option<DateTime<Utc>>,
    /// Usage count
    pub usage_count: u64,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Requests per minute
    pub rpm: Option<u32>,
    /// Requests per hour
    pub rph: Option<u32>,
    /// Requests per day
    pub rpd: Option<u32>,
    /// Tokens per minute
    pub tpm: Option<u32>,
    /// Tokens per hour
    pub tph: Option<u32>,
    /// Tokens per day
    pub tpd: Option<u32>,
    /// Maximum parallel requests
    pub max_parallel_requests: Option<u32>,
}

/// Permission types for virtual keys
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    /// Can make chat completion requests
    ChatCompletion,
    /// Can make text completion requests
    TextCompletion,
    /// Can make embedding requests
    Embedding,
    /// Can make image generation requests
    ImageGeneration,
    /// Can access specific models
    ModelAccess(String),
    /// Can access admin endpoints
    Admin,
    /// Can create other keys
    KeyManagement,
    /// Can view usage statistics
    ViewUsage,
    /// Can modify team settings
    TeamManagement,
    /// Custom permission
    Custom(String),
}

/// Virtual key creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    /// Key alias (optional)
    pub key_alias: Option<String>,
    /// User ID
    pub user_id: String,
    /// Team ID (optional)
    pub team_id: Option<String>,
    /// Models to allow
    pub models: Vec<String>,
    /// Maximum budget
    pub max_budget: Option<f64>,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Rate limits
    pub rate_limits: Option<RateLimits>,
    /// Permissions
    pub permissions: Vec<Permission>,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Tags
    pub tags: Vec<String>,
}

/// Virtual key update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKeyRequest {
    /// Key alias
    pub key_alias: Option<String>,
    /// Models to allow
    pub models: Option<Vec<String>>,
    /// Maximum budget
    pub max_budget: Option<f64>,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Rate limits
    pub rate_limits: Option<RateLimits>,
    /// Permissions
    pub permissions: Option<Vec<Permission>>,
    /// Metadata
    pub metadata: Option<HashMap<String, String>>,
    /// Expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether key is active
    pub is_active: Option<bool>,
    /// Tags
    pub tags: Option<Vec<String>>,
}

/// Virtual key manager
pub struct VirtualKeyManager {
    /// Database connection
    database: Arc<Database>,
    /// In-memory cache for frequently accessed keys
    key_cache: Arc<RwLock<HashMap<String, VirtualKey>>>,
    /// Rate limiting tracker
    rate_limiter: Arc<RwLock<HashMap<String, RateLimitState>>>,
    /// Key generation settings
    key_settings: KeyGenerationSettings,
}

/// Rate limit state tracking
#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// Request count in current window
    pub request_count: u32,
    /// Token count in current window
    pub token_count: u32,
    /// Window start time
    pub window_start: DateTime<Utc>,
    /// Current parallel requests
    pub parallel_requests: u32,
}

/// Key generation settings
#[derive(Debug, Clone)]
pub struct KeyGenerationSettings {
    /// Key length
    pub key_length: usize,
    /// Key prefix
    pub key_prefix: String,
    /// Default permissions
    pub default_permissions: Vec<Permission>,
    /// Default budget
    pub default_budget: Option<f64>,
    /// Default rate limits
    pub default_rate_limits: Option<RateLimits>,
}

impl Default for KeyGenerationSettings {
    fn default() -> Self {
        Self {
            key_length: 32,
            key_prefix: "sk-".to_string(),
            default_permissions: vec![
                Permission::ChatCompletion,
                Permission::TextCompletion,
                Permission::Embedding,
            ],
            default_budget: Some(100.0),
            default_rate_limits: Some(RateLimits {
                rpm: Some(60),
                rph: Some(3600),
                rpd: Some(86400),
                tpm: Some(100000),
                tph: Some(6000000),
                tpd: Some(144000000),
                max_parallel_requests: Some(10),
            }),
        }
    }
}

impl VirtualKeyManager {
    /// Create a new virtual key manager
    pub async fn new(database: Arc<Database>) -> Result<Self> {
        Ok(Self {
            database,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            key_settings: KeyGenerationSettings::default(),
        })
    }

    /// Create a new virtual key
    pub async fn create_key(&self, request: CreateKeyRequest) -> Result<(String, VirtualKey)> {
        info!("Creating virtual key for user: {}", request.user_id);

        // Generate new API key
        let api_key = self.generate_api_key();
        let key_hash = self.hash_key(&api_key);

        // Create virtual key
        let virtual_key = VirtualKey {
            key_id: Uuid::new_v4().to_string(),
            key_hash: key_hash.clone(),
            key_alias: request.key_alias,
            user_id: request.user_id,
            team_id: request.team_id,
            organization_id: None, // TODO: Add organization support
            models: request.models,
            max_budget: request.max_budget.or(self.key_settings.default_budget),
            spend: 0.0,
            budget_duration: request.budget_duration,
            budget_reset_at: self.calculate_budget_reset(&request.budget_duration),
            rate_limits: request.rate_limits.or(self.key_settings.default_rate_limits.clone()),
            permissions: if request.permissions.is_empty() {
                self.key_settings.default_permissions.clone()
            } else {
                request.permissions
            },
            metadata: request.metadata,
            expires_at: request.expires_at,
            is_active: true,
            created_at: Utc::now(),
            last_used_at: None,
            usage_count: 0,
            tags: request.tags,
        };

        // Store in database
        self.database.store_virtual_key(&virtual_key).await?;

        // Cache the key
        {
            let mut cache = self.key_cache.write().await;
            cache.insert(key_hash, virtual_key.clone());
        }

        info!("Virtual key created successfully: {}", virtual_key.key_id);
        Ok((api_key, virtual_key))
    }

    /// Validate and retrieve virtual key
    pub async fn validate_key(&self, api_key: &str) -> Result<VirtualKey> {
        let key_hash = self.hash_key(api_key);

        // Check cache first
        {
            let cache = self.key_cache.read().await;
            if let Some(key) = cache.get(&key_hash) {
                if self.is_key_valid(key) {
                    return Ok(key.clone());
                }
            }
        }

        // Load from database
        let mut virtual_key = self.database.get_virtual_key(&key_hash).await?
            .ok_or_else(|| GatewayError::Unauthorized("Invalid API key".to_string()))?;

        // Validate key
        if !self.is_key_valid(&virtual_key) {
            return Err(GatewayError::Unauthorized("API key is expired or inactive".to_string()));
        }

        // Update last used
        virtual_key.last_used_at = Some(Utc::now());
        virtual_key.usage_count += 1;

        // Update in database (async)
        let db = self.database.clone();
        let key_for_update = virtual_key.clone();
        tokio::spawn(async move {
            if let Err(e) = db.update_virtual_key_usage(&key_for_update).await {
                error!("Failed to update key usage: {}", e);
            }
        });

        // Update cache
        {
            let mut cache = self.key_cache.write().await;
            cache.insert(key_hash, virtual_key.clone());
        }

        Ok(virtual_key)
    }

    /// Check rate limits for a key
    pub async fn check_rate_limits(
        &self,
        key: &VirtualKey,
        tokens_requested: u32,
    ) -> Result<()> {
        if let Some(rate_limits) = &key.rate_limits {
            let mut rate_limiter = self.rate_limiter.write().await;
            let state = rate_limiter.entry(key.key_id.clone())
                .or_insert_with(|| RateLimitState {
                    request_count: 0,
                    token_count: 0,
                    window_start: Utc::now(),
                    parallel_requests: 0,
                });

            let now = Utc::now();
            
            // Reset window if needed (1 minute window)
            if now.signed_duration_since(state.window_start) > Duration::minutes(1) {
                state.request_count = 0;
                state.token_count = 0;
                state.window_start = now;
            }

            // Check RPM
            if let Some(rpm) = rate_limits.rpm {
                if state.request_count >= rpm {
                    return Err(GatewayError::RateLimit(
                        format!("Rate limit exceeded: {} requests per minute", rpm)
                    ));
                }
            }

            // Check TPM
            if let Some(tpm) = rate_limits.tpm {
                if state.token_count + tokens_requested > tpm {
                    return Err(GatewayError::RateLimit(
                        format!("Token rate limit exceeded: {} tokens per minute", tpm)
                    ));
                }
            }

            // Check parallel requests
            if let Some(max_parallel) = rate_limits.max_parallel_requests {
                if state.parallel_requests >= max_parallel {
                    return Err(GatewayError::RateLimit(
                        format!("Too many parallel requests: max {}", max_parallel)
                    ));
                }
            }

            // Update counters
            state.request_count += 1;
            state.token_count += tokens_requested;
            state.parallel_requests += 1;
        }

        Ok(())
    }

    /// Record request completion (for parallel request tracking)
    pub async fn record_request_completion(&self, key_id: &str) {
        let mut rate_limiter = self.rate_limiter.write().await;
        if let Some(state) = rate_limiter.get_mut(key_id) {
            if state.parallel_requests > 0 {
                state.parallel_requests -= 1;
            }
        }
    }

    /// Check budget limits
    pub async fn check_budget(&self, key: &VirtualKey, cost: f64) -> Result<()> {
        if let Some(max_budget) = key.max_budget {
            if key.spend + cost > max_budget {
                return Err(GatewayError::BudgetExceeded(
                    format!("Budget exceeded: ${:.2} + ${:.2} > ${:.2}", 
                           key.spend, cost, max_budget)
                ));
            }
        }
        Ok(())
    }

    /// Update key spend
    pub async fn update_spend(&self, key_id: &str, cost: f64) -> Result<()> {
        self.database.update_key_spend(key_id, cost).await?;

        // Update cache
        {
            let mut cache = self.key_cache.write().await;
            for (_, key) in cache.iter_mut() {
                if key.key_id == key_id {
                    key.spend += cost;
                    break;
                }
            }
        }

        Ok(())
    }

    /// List keys for a user
    pub async fn list_user_keys(&self, user_id: &str) -> Result<Vec<VirtualKey>> {
        self.database.list_user_keys(user_id).await
    }

    /// Update virtual key
    pub async fn update_key(&self, key_id: &str, request: UpdateKeyRequest) -> Result<VirtualKey> {
        let mut key = self.database.get_virtual_key_by_id(key_id).await?
            .ok_or_else(|| GatewayError::NotFound("Virtual key not found".to_string()))?;

        // Update fields
        if let Some(alias) = request.key_alias {
            key.key_alias = Some(alias);
        }
        if let Some(models) = request.models {
            key.models = models;
        }
        if let Some(budget) = request.max_budget {
            key.max_budget = Some(budget);
        }
        if let Some(duration) = request.budget_duration {
            key.budget_duration = Some(duration.clone());
            key.budget_reset_at = self.calculate_budget_reset(&Some(duration));
        }
        if let Some(rate_limits) = request.rate_limits {
            key.rate_limits = Some(rate_limits);
        }
        if let Some(permissions) = request.permissions {
            key.permissions = permissions;
        }
        if let Some(metadata) = request.metadata {
            key.metadata = metadata;
        }
        if let Some(expires_at) = request.expires_at {
            key.expires_at = Some(expires_at);
        }
        if let Some(is_active) = request.is_active {
            key.is_active = is_active;
        }
        if let Some(tags) = request.tags {
            key.tags = tags;
        }

        // Update in database
        self.database.update_virtual_key(&key).await?;

        // Update cache
        {
            let mut cache = self.key_cache.write().await;
            cache.insert(key.key_hash.clone(), key.clone());
        }

        Ok(key)
    }

    /// Delete virtual key
    pub async fn delete_key(&self, key_id: &str) -> Result<()> {
        let key = self.database.get_virtual_key_by_id(key_id).await?
            .ok_or_else(|| GatewayError::NotFound("Virtual key not found".to_string()))?;

        // Delete from database
        self.database.delete_virtual_key(key_id).await?;

        // Remove from cache
        {
            let mut cache = self.key_cache.write().await;
            cache.remove(&key.key_hash);
        }

        info!("Virtual key deleted: {}", key_id);
        Ok(())
    }

    /// Generate a new API key
    fn generate_api_key(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        let random_string: String = (0..self.key_settings.key_length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        format!("{}{}", self.key_settings.key_prefix, random_string)
    }

    /// Hash an API key
    fn hash_key(&self, key: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if a key is valid
    fn is_key_valid(&self, key: &VirtualKey) -> bool {
        if !key.is_active {
            return false;
        }

        if let Some(expires_at) = key.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        true
    }

    /// Calculate budget reset time
    fn calculate_budget_reset(&self, duration: &Option<String>) -> Option<DateTime<Utc>> {
        duration.as_ref().and_then(|d| {
            let now = Utc::now();
            match d.as_str() {
                "1d" => Some(now + Duration::days(1)),
                "1w" => Some(now + Duration::weeks(1)),
                "1m" => Some(now + Duration::days(30)),
                _ => None,
            }
        })
    }

    /// Reset budgets for expired keys
    pub async fn reset_expired_budgets(&self) -> Result<()> {
        let keys_to_reset = self.database.get_keys_with_expired_budgets().await?;
        
        for mut key in keys_to_reset {
            key.spend = 0.0;
            key.budget_reset_at = self.calculate_budget_reset(&key.budget_duration);
            
            self.database.update_virtual_key(&key).await?;
            
            // Update cache
            {
                let mut cache = self.key_cache.write().await;
                cache.insert(key.key_hash.clone(), key);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let manager = VirtualKeyManager {
            database: Arc::new(Database::new_mock()),
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            key_settings: KeyGenerationSettings::default(),
        };

        let key = manager.generate_api_key();
        assert!(key.starts_with("sk-"));
        assert_eq!(key.len(), 35); // "sk-" + 32 chars
    }

    #[test]
    fn test_key_hashing() {
        let manager = VirtualKeyManager {
            database: Arc::new(Database::new_mock()),
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            key_settings: KeyGenerationSettings::default(),
        };

        let key = "sk-test123";
        let hash1 = manager.hash_key(key);
        let hash2 = manager.hash_key(key);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, key);
    }

    #[test]
    fn test_key_validation() {
        let manager = VirtualKeyManager {
            database: Arc::new(Database::new_mock()),
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            key_settings: KeyGenerationSettings::default(),
        };

        let active_key = VirtualKey {
            key_id: "test".to_string(),
            key_hash: "hash".to_string(),
            key_alias: None,
            user_id: "user1".to_string(),
            team_id: None,
            organization_id: None,
            models: vec![],
            max_budget: None,
            spend: 0.0,
            budget_duration: None,
            budget_reset_at: None,
            rate_limits: None,
            permissions: vec![],
            metadata: HashMap::new(),
            expires_at: None,
            is_active: true,
            created_at: Utc::now(),
            last_used_at: None,
            usage_count: 0,
            tags: vec![],
        };

        assert!(manager.is_key_valid(&active_key));

        let inactive_key = VirtualKey {
            is_active: false,
            ..active_key.clone()
        };

        assert!(!manager.is_key_valid(&inactive_key));

        let expired_key = VirtualKey {
            expires_at: Some(Utc::now() - Duration::hours(1)),
            ..active_key
        };

        assert!(!manager.is_key_valid(&expired_key));
    }
}
