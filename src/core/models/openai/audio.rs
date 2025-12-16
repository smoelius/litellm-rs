//! Audio types for OpenAI-compatible API
//!
//! This module defines audio-related structures for multimodal interactions
//! including audio content, parameters, and delta updates for streaming.

use serde::{Deserialize, Serialize};

/// Audio parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParams {
    /// Voice to use
    pub voice: String,
    /// Audio format
    pub format: String,
}

/// Audio content
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct AudioContent {
    /// Audio data (base64 encoded)
    pub data: String,
    /// Audio format
    pub format: String,
}

/// Audio delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDelta {
    /// Audio data delta
    pub data: Option<String>,
    /// Transcript delta
    pub transcript: Option<String>,
}
