//! Pricing database entities

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Model pricing entity for database storage
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "model_pricing")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Provider name (openai, anthropic, glm, etc.)
    #[sea_orm(column_type = "String(Some(50))")]
    pub provider: String,
    /// Model name
    #[sea_orm(column_type = "String(Some(100))")]
    pub model: String,
    /// Input token cost per 1K tokens
    pub input_cost_per_1k: f64,
    /// Output token cost per 1K tokens  
    pub output_cost_per_1k: f64,
    /// Currency code
    #[sea_orm(column_type = "String(Some(10))")]
    pub currency: String,
    /// Whether this is the default pricing for unknown models
    pub is_default: bool,
    /// Additional metadata (JSON)
    #[sea_orm(column_type = "Json", nullable)]
    pub metadata: Option<serde_json::Value>,
    /// Data source (config, api, manual)
    #[sea_orm(column_type = "String(Some(20))", nullable)]
    pub source: Option<String>,
    /// Created timestamp
    pub created_at: DateTimeUtc,
    /// Updated timestamp  
    pub updated_at: DateTimeUtc,
    /// Expiry timestamp (for cached external data)
    pub expires_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::pricing_history::Entity")]
    PricingHistory,
}

impl Related<super::pricing_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PricingHistory.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Pricing history entity for tracking price changes
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "pricing_history")]
pub struct PricingHistoryModel {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Reference to model_pricing
    pub pricing_id: i32,
    /// Provider name
    #[sea_orm(column_type = "String(Some(50))")]
    pub provider: String,
    /// Model name
    #[sea_orm(column_type = "String(Some(100))")]
    pub model: String,
    /// Previous input cost
    pub old_input_cost_per_1k: f64,
    /// New input cost
    pub new_input_cost_per_1k: f64,
    /// Previous output cost
    pub old_output_cost_per_1k: f64,
    /// New output cost
    pub new_output_cost_per_1k: f64,
    /// Change reason
    #[sea_orm(column_type = "Text", nullable)]
    pub change_reason: Option<String>,
    /// Changed by (user/system)
    #[sea_orm(column_type = "String(Some(50))", nullable)]
    pub changed_by: Option<String>,
    /// Created timestamp
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum PricingHistoryRelation {
    #[sea_orm(
        belongs_to = "super::pricing::Entity",
        from = "super::pricing_history::Column::PricingId",
        to = "super::pricing::Column::Id"
    )]
    ModelPricing,
}

impl Related<Entity> for super::pricing_history::Entity {
    fn to() -> RelationDef {
        PricingHistoryRelation::ModelPricing.def()
    }
}

impl ActiveModelBehavior for super::pricing_history::ActiveModel {}

// Re-export for convenience
pub use super::pricing_history::{
    Entity as PricingHistoryEntity,
    Model as PricingHistoryModel,
    ActiveModel as PricingHistoryActiveModel,
    Column as PricingHistoryColumn,
};