//! Response models for the Gateway
//!
//! This module defines internal response structures used by the gateway.

mod completion;
mod embedding;
mod error;
mod media;
mod metadata;
mod moderation;
mod rerank;
mod types;

// Re-export all types for backward compatibility
pub use completion::{CompletionChoice, CompletionLogprobs, CompletionResponse};
pub use embedding::{EmbeddingData, EmbeddingResponse, EmbeddingUsage};
pub use error::{ErrorDetail, ErrorResponse};
pub use media::{
    AudioTranscriptionResponse, ImageData, ImageGenerationResponse, TranscriptionSegment,
    TranscriptionWord,
};
pub use metadata::{CacheInfo, ProviderInfo, ResponseMetrics};
pub use moderation::{
    ModerationCategories, ModerationCategoryScores, ModerationResponse, ModerationResult,
};
pub use rerank::{RerankResponse, RerankResult, RerankUsage};
pub use types::{GatewayResponse, ResponseData, ResponseType};
