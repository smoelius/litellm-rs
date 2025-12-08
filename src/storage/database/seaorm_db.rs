use crate::config::DatabaseConfig;
use crate::core::models::user::User;
use crate::utils::error::{GatewayError, Result};
use sea_orm::*;
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::entities::{self, password_reset_token, user};
use super::migration::Migrator;

/// SeaORM-based database implementation
#[derive(Debug)]
pub struct SeaOrmDatabase {
    db: DatabaseConnection,
}

impl SeaOrmDatabase {
    /// Create a new database connection
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let mut opt = ConnectOptions::new(config.url.clone());
        opt.max_connections(config.max_connections)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(config.connection_timeout))
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(3600))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Debug);

        let db = Database::connect(opt)
            .await
            .map_err(GatewayError::Database)?;

        info!("Database connection established");
        Ok(Self { db })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        Migrator::up(&self.db, None).await.map_err(|e| {
            warn!("Migration failed: {}", e);
            GatewayError::Database(e)
        })?;
        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Find user by ID
    pub async fn find_user_by_id(&self, user_id: uuid::Uuid) -> Result<Option<User>> {
        debug!("Finding user by ID: {}", user_id);

        let user_model = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(user_model.map(|model| model.to_domain_user()))
    }

    /// Find user by username
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        debug!("Finding user by username: {}", username);

        let user_model = entities::User::find()
            .filter(user::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(user_model.map(|model| model.to_domain_user()))
    }

    /// Find user by email
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>> {
        debug!("Finding user by email: {}", email);

        let user_model = entities::User::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(user_model.map(|model| model.to_domain_user()))
    }

    /// Create a new user
    pub async fn create_user(&self, user: &User) -> Result<User> {
        debug!("Creating user: {}", user.username);

        let active_model = user::Model::from_domain_user(user);

        let _result = entities::User::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        // Return the created user
        Ok(user.clone())
    }

    /// Update user password
    pub async fn update_user_password(
        &self,
        user_id: uuid::Uuid,
        password_hash: &str,
    ) -> Result<()> {
        debug!("Updating password for user: {}", user_id);

        let mut user: user::ActiveModel = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?
            .into();

        user.password_hash = Set(password_hash.to_string());
        user.updated_at = Set(chrono::Utc::now().into());

        user.update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Store password reset token
    pub async fn store_password_reset_token(
        &self,
        user_id: uuid::Uuid,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        debug!("Storing password reset token for user: {}", user_id);

        // First, delete any existing tokens for this user
        entities::PasswordResetToken::delete_many()
            .filter(password_reset_token::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        // Insert new token
        let active_model = password_reset_token::ActiveModel {
            id: NotSet,
            user_id: Set(user_id),
            token: Set(token.to_string()),
            expires_at: Set(expires_at.into()),
            created_at: Set(chrono::Utc::now().into()),
            used_at: Set(None),
        };

        entities::PasswordResetToken::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Verify and consume password reset token
    pub async fn verify_password_reset_token(&self, token: &str) -> Result<Option<uuid::Uuid>> {
        debug!("Verifying password reset token");

        let token_model = entities::PasswordResetToken::find()
            .filter(password_reset_token::Column::Token.eq(token))
            .filter(password_reset_token::Column::UsedAt.is_null())
            .filter(password_reset_token::Column::ExpiresAt.gt(chrono::Utc::now()))
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        if let Some(token_model) = token_model {
            // Mark token as used
            let mut active_model: password_reset_token::ActiveModel = token_model.clone().into();
            active_model.used_at = Set(Some(chrono::Utc::now().into()));

            active_model
                .update(&self.db)
                .await
                .map_err(GatewayError::Database)?;

            Ok(Some(token_model.user_id))
        } else {
            Ok(None)
        }
    }

    /// Invalidate password reset token
    pub async fn invalidate_password_reset_token(&self, token: &str) -> Result<()> {
        debug!("Invalidating password reset token");

        let token_model = entities::PasswordResetToken::find()
            .filter(password_reset_token::Column::Token.eq(token))
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        if let Some(token_model) = token_model {
            let mut active_model: password_reset_token::ActiveModel = token_model.into();
            active_model.used_at = Set(Some(chrono::Utc::now().into()));

            active_model
                .update(&self.db)
                .await
                .map_err(GatewayError::Database)?;
        }

        Ok(())
    }

    /// Verify user email
    pub async fn verify_user_email(&self, user_id: uuid::Uuid) -> Result<()> {
        debug!("Verifying email for user: {}", user_id);

        let user_model = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let mut active_model: user::ActiveModel = user_model.into();
        active_model.email_verified = Set(true);
        active_model.updated_at = Set(chrono::Utc::now().into());

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Clean up expired password reset tokens
    /// Clean up expired password reset tokens
    #[allow(dead_code)] // Reserved for future token cleanup functionality
    pub async fn cleanup_expired_tokens(&self) -> Result<u64> {
        debug!("Cleaning up expired password reset tokens");

        let result = entities::PasswordResetToken::delete_many()
            .filter(password_reset_token::Column::ExpiresAt.lt(chrono::Utc::now()))
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(result.rows_affected)
    }

    /// Get database connection for advanced operations
    /// Get the underlying database connection
    #[allow(dead_code)] // Reserved for future direct database access
    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        debug!("Performing database health check");

        // Simple query to check database connectivity
        let _result = entities::User::find()
            .limit(1)
            .all(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        debug!("Database health check passed");
        Ok(())
    }

    /// Update user last login
    pub async fn update_user_last_login(&self, user_id: uuid::Uuid) -> Result<()> {
        debug!("Updating last login for user: {}", user_id);

        let user_model = entities::User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let mut active_model: user::ActiveModel = user_model.into();
        active_model.last_login_at = Set(Some(chrono::Utc::now().into()));
        active_model.updated_at = Set(chrono::Utc::now().into());

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Close database connection
    /// Close the database connection
    #[allow(dead_code)] // Reserved for future connection cleanup
    pub async fn close(self) -> Result<()> {
        self.db.close().await.map_err(GatewayError::Database)?;
        Ok(())
    }

    // Batch operations (placeholder implementations)
    /// Create a new batch
    pub async fn create_batch(&self, batch: &crate::core::batch::BatchRequest) -> Result<String> {
        debug!("Creating batch: {}", batch.batch_id);

        let active_model = entities::batch::ActiveModel {
            id: Set(batch.batch_id.clone()),
            object: Set("batch".to_string()),
            endpoint: Set(match batch.batch_type {
                crate::core::batch::BatchType::ChatCompletion => "/v1/chat/completions".to_string(),
                crate::core::batch::BatchType::Embedding => "/v1/embeddings".to_string(),
                crate::core::batch::BatchType::ImageGeneration => {
                    "/v1/images/generations".to_string()
                }
                crate::core::batch::BatchType::Custom(ref endpoint) => endpoint.clone(),
            }),
            input_file_id: Set(None),
            completion_window: Set(format!("{}h", batch.completion_window.unwrap_or(24))),
            status: Set("validating".to_string()),
            output_file_id: Set(None),
            error_file_id: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            in_progress_at: Set(None),
            finalizing_at: Set(None),
            completed_at: Set(None),
            failed_at: Set(None),
            expired_at: Set(None),
            cancelling_at: Set(None),
            cancelled_at: Set(None),
            request_counts_total: Set(Some(batch.requests.len() as i32)),
            request_counts_completed: Set(Some(0)),
            request_counts_failed: Set(Some(0)),
            metadata: Set(Some(
                serde_json::to_string(&batch.metadata).unwrap_or_default(),
            )),
        };

        entities::Batch::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(batch.batch_id.clone())
    }

    /// Update batch status
    pub async fn update_batch_status(&self, batch_id: &str, status: &str) -> Result<()> {
        debug!("Updating batch status: {} -> {}", batch_id, status);

        let batch_model = entities::Batch::find_by_id(batch_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("Batch not found".to_string()))?;

        let mut active_model: entities::batch::ActiveModel = batch_model.into();
        active_model.status = Set(status.to_string());

        // Update timestamp based on status
        let now = chrono::Utc::now().into();
        match status {
            "in_progress" => active_model.in_progress_at = Set(Some(now)),
            "finalizing" => active_model.finalizing_at = Set(Some(now)),
            "completed" => active_model.completed_at = Set(Some(now)),
            "failed" => active_model.failed_at = Set(Some(now)),
            "expired" => active_model.expired_at = Set(Some(now)),
            "cancelling" => active_model.cancelling_at = Set(Some(now)),
            "cancelled" => active_model.cancelled_at = Set(Some(now)),
            _ => {}
        }

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// List batches with pagination
    pub async fn list_batches(
        &self,
        limit: Option<i32>,
        after: Option<&str>,
    ) -> Result<Vec<crate::core::batch::BatchRecord>> {
        debug!(
            "Listing batches with limit: {:?}, after: {:?}",
            limit, after
        );

        let mut query = entities::Batch::find();

        if let Some(after_id) = after {
            query = query.filter(entities::batch::Column::Id.gt(after_id));
        }

        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }

        let batch_models = query
            .order_by_desc(entities::batch::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        let batch_records = batch_models
            .into_iter()
            .map(|model| {
                // Parse status string to BatchStatus enum
                let status = match model.status.as_str() {
                    "validating" => crate::core::batch::BatchStatus::Validating,
                    "failed" => crate::core::batch::BatchStatus::Failed,
                    "in_progress" => crate::core::batch::BatchStatus::InProgress,
                    "finalizing" => crate::core::batch::BatchStatus::Finalizing,
                    "completed" => crate::core::batch::BatchStatus::Completed,
                    "expired" => crate::core::batch::BatchStatus::Expired,
                    "cancelling" => crate::core::batch::BatchStatus::Cancelling,
                    "cancelled" => crate::core::batch::BatchStatus::Cancelled,
                    _ => crate::core::batch::BatchStatus::Failed,
                };

                crate::core::batch::BatchRecord {
                    id: model.id,
                    object: model.object,
                    endpoint: model.endpoint,
                    input_file_id: model.input_file_id,
                    completion_window: model.completion_window,
                    status,
                    output_file_id: model.output_file_id,
                    error_file_id: model.error_file_id,
                    created_at: model.created_at.with_timezone(&chrono::Utc),
                    in_progress_at: model.in_progress_at.map(|t| t.with_timezone(&chrono::Utc)),
                    expires_at: None, // TODO: Add expires_at field to database schema
                    finalizing_at: model.finalizing_at.map(|t| t.with_timezone(&chrono::Utc)),
                    completed_at: model.completed_at.map(|t| t.with_timezone(&chrono::Utc)),
                    failed_at: model.failed_at.map(|t| t.with_timezone(&chrono::Utc)),
                    expired_at: model.expired_at.map(|t| t.with_timezone(&chrono::Utc)),
                    cancelling_at: model.cancelling_at.map(|t| t.with_timezone(&chrono::Utc)),
                    cancelled_at: model.cancelled_at.map(|t| t.with_timezone(&chrono::Utc)),
                    request_counts: crate::core::batch::BatchRequestCounts {
                        total: model.request_counts_total.unwrap_or(0),
                        completed: model.request_counts_completed.unwrap_or(0),
                        failed: model.request_counts_failed.unwrap_or(0),
                    },
                    metadata: model.metadata.and_then(|m| serde_json::from_str(&m).ok()),
                }
            })
            .collect();

        Ok(batch_records)
    }

    /// Get batch results
    pub async fn get_batch_results(
        &self,
        _batch_id: &str,
    ) -> Result<Option<Vec<serde_json::Value>>> {
        // TODO: Implement batch results retrieval
        warn!("get_batch_results not implemented yet");
        Ok(None)
    }

    /// Get batch request
    pub async fn get_batch_request(
        &self,
        _batch_id: &str,
    ) -> Result<Option<crate::core::batch::BatchRequest>> {
        // TODO: Implement batch request retrieval
        warn!("get_batch_request not implemented yet");
        Ok(None)
    }

    /// Store batch results
    pub async fn store_batch_results(
        &self,
        _batch_id: &str,
        _results: &[serde_json::Value],
    ) -> Result<()> {
        // TODO: Implement batch results storage
        warn!("store_batch_results not implemented yet");
        Ok(())
    }

    /// Update batch progress
    pub async fn update_batch_progress(
        &self,
        batch_id: &str,
        completed: i32,
        failed: i32,
    ) -> Result<()> {
        debug!(
            "Updating batch progress: {} - completed: {}, failed: {}",
            batch_id, completed, failed
        );

        let batch_model = entities::Batch::find_by_id(batch_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("Batch not found".to_string()))?;

        let mut active_model: entities::batch::ActiveModel = batch_model.into();
        active_model.request_counts_completed = Set(Some(completed));
        active_model.request_counts_failed = Set(Some(failed));

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    /// Mark batch as completed
    pub async fn mark_batch_completed(&self, batch_id: &str) -> Result<()> {
        debug!("Marking batch as completed: {}", batch_id);

        let batch_model = entities::Batch::find_by_id(batch_id)
            .one(&self.db)
            .await
            .map_err(GatewayError::Database)?
            .ok_or_else(|| GatewayError::NotFound("Batch not found".to_string()))?;

        let mut active_model: entities::batch::ActiveModel = batch_model.into();
        active_model.status = Set("completed".to_string());
        active_model.completed_at = Set(Some(chrono::Utc::now().into()));

        active_model
            .update(&self.db)
            .await
            .map_err(GatewayError::Database)?;

        Ok(())
    }

    // Analytics operations (placeholder implementations)
    /// Get user usage statistics
    pub async fn get_user_usage(
        &self,
        _user_id: &str,
        _start: chrono::DateTime<chrono::Utc>,
        _end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<serde_json::Value>> {
        // TODO: Implement user usage retrieval
        warn!("get_user_usage not implemented yet");
        Ok(vec![])
    }

    // API Key operations (placeholder implementations)
    /// Create a new API key
    pub async fn create_api_key(
        &self,
        _api_key: &crate::core::models::ApiKey,
    ) -> Result<crate::core::models::ApiKey> {
        // TODO: Implement API key creation
        warn!("create_api_key not implemented yet");
        Err(GatewayError::NotImplemented(
            "create_api_key not implemented".to_string(),
        ))
    }

    /// Find API key by hash
    pub async fn find_api_key_by_hash(
        &self,
        _key_hash: &str,
    ) -> Result<Option<crate::core::models::ApiKey>> {
        // TODO: Implement API key lookup by hash
        warn!("find_api_key_by_hash not implemented yet");
        Ok(None)
    }

    /// Find API key by ID
    pub async fn find_api_key_by_id(
        &self,
        _key_id: uuid::Uuid,
    ) -> Result<Option<crate::auth::ApiKey>> {
        // TODO: Implement API key lookup by ID
        warn!("find_api_key_by_id not implemented yet");
        Ok(None)
    }

    /// Deactivate API key
    pub async fn deactivate_api_key(&self, _key_id: uuid::Uuid) -> Result<()> {
        // TODO: Implement API key deactivation
        warn!("deactivate_api_key not implemented yet");
        Ok(())
    }

    /// List API keys by user
    /// Note: Changed from i64 to Uuid to avoid lossy conversion from Uuid->i64
    pub async fn list_api_keys_by_user(&self, _user_id: uuid::Uuid) -> Result<Vec<crate::auth::ApiKey>> {
        // TODO: Implement API key listing by user
        warn!("list_api_keys_by_user not implemented yet");
        Ok(vec![])
    }

    /// List API keys by team
    pub async fn list_api_keys_by_team(
        &self,
        _team_id: uuid::Uuid,
    ) -> Result<Vec<crate::auth::ApiKey>> {
        // TODO: Implement API key listing by team
        warn!("list_api_keys_by_team not implemented yet");
        Ok(vec![])
    }

    /// Update API key permissions
    pub async fn update_api_key_permissions(
        &self,
        _key_id: uuid::Uuid,
        _permissions: &[String],
    ) -> Result<()> {
        // TODO: Implement API key permissions update
        warn!("update_api_key_permissions not implemented yet");
        Ok(())
    }

    /// Update API key rate limits
    pub async fn update_api_key_rate_limits(
        &self,
        _key_id: uuid::Uuid,
        _rate_limits: &crate::core::models::RateLimits,
    ) -> Result<()> {
        // TODO: Implement API key rate limits update
        warn!("update_api_key_rate_limits not implemented yet");
        Ok(())
    }

    /// Update API key expiration
    pub async fn update_api_key_expiration(
        &self,
        _key_id: uuid::Uuid,
        _expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        // TODO: Implement API key expiration update
        warn!("update_api_key_expiration not implemented yet");
        Ok(())
    }

    /// Update API key usage statistics
    pub async fn update_api_key_usage(
        &self,
        _key_id: uuid::Uuid,
        _requests: u64,
        _tokens: u64,
        _cost: f64,
    ) -> Result<()> {
        // TODO: Implement API key usage update
        warn!("update_api_key_usage not implemented yet");
        Ok(())
    }

    /// Update API key last used timestamp
    pub async fn update_api_key_last_used(&self, _key_id: uuid::Uuid) -> Result<()> {
        // TODO: Implement API key last used update
        warn!("update_api_key_last_used not implemented yet");
        Ok(())
    }

    /// Delete expired API keys
    pub async fn delete_expired_api_keys(&self) -> Result<u64> {
        // TODO: Implement expired API key deletion
        warn!("delete_expired_api_keys not implemented yet");
        Ok(0)
    }

    // Monitoring operations (placeholder implementations)
    /// Store request metrics
    /// Store request metrics
    #[allow(dead_code)] // Reserved for future metrics storage functionality
    pub async fn store_metrics(
        &self,
        _metrics: &crate::core::models::metrics::RequestMetrics,
    ) -> Result<()> {
        // TODO: Implement metrics storage
        warn!("store_metrics not implemented yet");
        Ok(())
    }

    /// Get database statistics
    pub fn stats(&self) -> DatabaseStats {
        // TODO: Implement database stats
        warn!("stats not implemented yet");
        DatabaseStats {
            total_users: 0,
            size: 0,
            idle: 0,
        }
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Total number of users
    #[allow(dead_code)] // Reserved for future database statistics
    pub total_users: u64,
    /// Database size
    pub size: u32,
    /// Number of idle connections
    pub idle: usize,
}
