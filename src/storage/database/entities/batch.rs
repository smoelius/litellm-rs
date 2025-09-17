use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Batch processing database model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "batches")]
pub struct Model {
    /// Batch ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    /// Object type (always "batch")
    pub object: String,

    /// API endpoint for batch processing
    pub endpoint: String,

    /// Input file ID (optional)
    pub input_file_id: Option<String>,

    /// Completion window (e.g., "24h")
    pub completion_window: String,

    /// Batch processing status
    pub status: String,

    /// Output file ID (when completed)
    pub output_file_id: Option<String>,

    /// Error file ID (when failed)
    pub error_file_id: Option<String>,

    /// Batch creation timestamp
    pub created_at: DateTimeWithTimeZone,

    /// In progress timestamp
    pub in_progress_at: Option<DateTimeWithTimeZone>,

    /// Finalizing timestamp
    pub finalizing_at: Option<DateTimeWithTimeZone>,

    /// Completion timestamp
    pub completed_at: Option<DateTimeWithTimeZone>,

    /// Failure timestamp
    pub failed_at: Option<DateTimeWithTimeZone>,

    /// Expiration timestamp
    pub expired_at: Option<DateTimeWithTimeZone>,

    /// Cancelling timestamp
    pub cancelling_at: Option<DateTimeWithTimeZone>,

    /// Cancelled timestamp
    pub cancelled_at: Option<DateTimeWithTimeZone>,

    /// Total request count
    pub request_counts_total: Option<i32>,

    /// Completed request count
    pub request_counts_completed: Option<i32>,

    /// Failed request count
    pub request_counts_failed: Option<i32>,

    /// Batch metadata (JSON)
    pub metadata: Option<String>,
}

/// Batch entity relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
