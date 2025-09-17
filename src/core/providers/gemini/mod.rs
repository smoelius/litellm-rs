//! Google Gemini Provider
//!
//! 支持Google AI Studio和Vertex AI的Gemini模型系列
//! 
//! # 支持的模型
//! - Gemini 2.0 Flash (最新)
//! - Gemini 1.5 Pro
//! - Gemini 1.5 Flash  
//! - Gemini 1.0 Pro
//! 
//! # 特性
//! - 多模态支持（文本、图像、视频、音频）
//! Utilities
//! - 上下文cache
//! - 批process
//! - 实时流式响应

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod provider;
pub mod streaming;

// Re-export main types
pub use client::GeminiClient;
pub use config::GeminiConfig;
pub use error::GeminiError;
pub use models::{get_gemini_registry, GeminiModelFamily, ModelFeature};
pub use provider::GeminiProvider;
pub use streaming::GeminiStream;

// 便捷函数

/// Create
pub fn create_gemini_provider(config: GeminiConfig) -> Result<GeminiProvider, error::GeminiError> {
    GeminiProvider::new(config)
}

/// Create
pub fn create_gemini_provider_from_env() -> Result<GeminiProvider, error::GeminiError> {
    let config = GeminiConfig::from_env()?;
    GeminiProvider::new(config)
}

/// Model
pub fn supported_models() -> Vec<String> {
    get_gemini_registry()
        .list_models()
        .into_iter()
        .map(|spec| spec.model_info.id.clone())
        .collect()
}

/// Check
pub fn is_model_supported(model_id: &str) -> bool {
    get_gemini_registry().get_model_spec(model_id).is_some()
}

/// Model
pub fn get_model_pricing(model_id: &str) -> Option<(f64, f64)> {
    get_gemini_registry()
        .get_model_pricing(model_id)
        .map(|p| (p.input_price, p.output_price))
}