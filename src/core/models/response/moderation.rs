//! Moderation response types

use serde::{Deserialize, Serialize};

/// Moderation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationResponse {
    /// Response ID
    pub id: String,
    /// Model used
    pub model: String,
    /// Results
    pub results: Vec<ModerationResult>,
}

/// Moderation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationResult {
    /// Whether content is flagged
    pub flagged: bool,
    /// Category flags
    pub categories: ModerationCategories,
    /// Category scores
    pub category_scores: ModerationCategoryScores,
}

/// Moderation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationCategories {
    /// Sexual content
    pub sexual: bool,
    /// Hate speech
    pub hate: bool,
    /// Harassment
    pub harassment: bool,
    /// Self-harm
    #[serde(rename = "self-harm")]
    pub self_harm: bool,
    /// Sexual content involving minors
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: bool,
    /// Hate speech targeting identity
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: bool,
    /// Harassment threatening
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: bool,
    /// Self-harm instructions
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: bool,
    /// Self-harm intent
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: bool,
    /// Violence
    pub violence: bool,
    /// Graphic violence
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: bool,
}

/// Moderation category scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationCategoryScores {
    /// Sexual content score
    pub sexual: f64,
    /// Hate speech score
    pub hate: f64,
    /// Harassment score
    pub harassment: f64,
    /// Self-harm score
    #[serde(rename = "self-harm")]
    pub self_harm: f64,
    /// Sexual content involving minors score
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: f64,
    /// Hate speech targeting identity score
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: f64,
    /// Harassment threatening score
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: f64,
    /// Self-harm instructions score
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: f64,
    /// Self-harm intent score
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: f64,
    /// Violence score
    pub violence: f64,
    /// Graphic violence score
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: f64,
}
