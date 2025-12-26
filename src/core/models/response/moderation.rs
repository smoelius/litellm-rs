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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ModerationCategories Tests ====================

    #[test]
    fn test_moderation_categories_all_false() {
        let categories = ModerationCategories {
            sexual: false,
            hate: false,
            harassment: false,
            self_harm: false,
            sexual_minors: false,
            hate_threatening: false,
            harassment_threatening: false,
            self_harm_instructions: false,
            self_harm_intent: false,
            violence: false,
            violence_graphic: false,
        };

        assert!(!categories.sexual);
        assert!(!categories.hate);
        assert!(!categories.violence);
    }

    #[test]
    fn test_moderation_categories_some_flagged() {
        let categories = ModerationCategories {
            sexual: false,
            hate: true,
            harassment: true,
            self_harm: false,
            sexual_minors: false,
            hate_threatening: false,
            harassment_threatening: true,
            self_harm_instructions: false,
            self_harm_intent: false,
            violence: false,
            violence_graphic: false,
        };

        assert!(categories.hate);
        assert!(categories.harassment);
        assert!(categories.harassment_threatening);
    }

    #[test]
    fn test_moderation_categories_serialization() {
        let categories = ModerationCategories {
            sexual: false,
            hate: true,
            harassment: false,
            self_harm: false,
            sexual_minors: false,
            hate_threatening: false,
            harassment_threatening: false,
            self_harm_instructions: false,
            self_harm_intent: false,
            violence: true,
            violence_graphic: false,
        };

        let json = serde_json::to_value(&categories).unwrap();
        assert_eq!(json["hate"], true);
        assert_eq!(json["violence"], true);
        assert_eq!(json["self-harm"], false);
    }

    #[test]
    fn test_moderation_categories_deserialization() {
        let json = r#"{
            "sexual": false,
            "hate": true,
            "harassment": false,
            "self-harm": false,
            "sexual/minors": false,
            "hate/threatening": true,
            "harassment/threatening": false,
            "self-harm/instructions": false,
            "self-harm/intent": false,
            "violence": false,
            "violence/graphic": false
        }"#;

        let categories: ModerationCategories = serde_json::from_str(json).unwrap();
        assert!(categories.hate);
        assert!(categories.hate_threatening);
        assert!(!categories.violence);
    }

    #[test]
    fn test_moderation_categories_clone() {
        let categories = ModerationCategories {
            sexual: true,
            hate: false,
            harassment: false,
            self_harm: false,
            sexual_minors: false,
            hate_threatening: false,
            harassment_threatening: false,
            self_harm_instructions: false,
            self_harm_intent: false,
            violence: false,
            violence_graphic: false,
        };

        let cloned = categories.clone();
        assert_eq!(categories.sexual, cloned.sexual);
    }

    // ==================== ModerationCategoryScores Tests ====================

    #[test]
    fn test_moderation_category_scores_zeros() {
        let scores = ModerationCategoryScores {
            sexual: 0.0,
            hate: 0.0,
            harassment: 0.0,
            self_harm: 0.0,
            sexual_minors: 0.0,
            hate_threatening: 0.0,
            harassment_threatening: 0.0,
            self_harm_instructions: 0.0,
            self_harm_intent: 0.0,
            violence: 0.0,
            violence_graphic: 0.0,
        };

        assert!((scores.sexual - 0.0).abs() < f64::EPSILON);
        assert!((scores.violence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_moderation_category_scores_high_values() {
        let scores = ModerationCategoryScores {
            sexual: 0.1,
            hate: 0.85,
            harassment: 0.72,
            self_harm: 0.05,
            sexual_minors: 0.001,
            hate_threatening: 0.65,
            harassment_threatening: 0.55,
            self_harm_instructions: 0.02,
            self_harm_intent: 0.03,
            violence: 0.45,
            violence_graphic: 0.25,
        };

        assert!(scores.hate > 0.8);
        assert!(scores.harassment > 0.7);
    }

    #[test]
    fn test_moderation_category_scores_serialization() {
        let scores = ModerationCategoryScores {
            sexual: 0.1,
            hate: 0.2,
            harassment: 0.3,
            self_harm: 0.4,
            sexual_minors: 0.05,
            hate_threatening: 0.15,
            harassment_threatening: 0.25,
            self_harm_instructions: 0.35,
            self_harm_intent: 0.45,
            violence: 0.5,
            violence_graphic: 0.55,
        };

        let json = serde_json::to_value(&scores).unwrap();
        assert_eq!(json["sexual"], 0.1);
        assert_eq!(json["self-harm"], 0.4);
        assert_eq!(json["violence/graphic"], 0.55);
    }

    #[test]
    fn test_moderation_category_scores_deserialization() {
        let json = r#"{
            "sexual": 0.01,
            "hate": 0.02,
            "harassment": 0.03,
            "self-harm": 0.04,
            "sexual/minors": 0.005,
            "hate/threatening": 0.015,
            "harassment/threatening": 0.025,
            "self-harm/instructions": 0.035,
            "self-harm/intent": 0.045,
            "violence": 0.05,
            "violence/graphic": 0.055
        }"#;

        let scores: ModerationCategoryScores = serde_json::from_str(json).unwrap();
        assert!((scores.sexual - 0.01).abs() < f64::EPSILON);
        assert!((scores.self_harm - 0.04).abs() < f64::EPSILON);
    }

    #[test]
    fn test_moderation_category_scores_clone() {
        let scores = ModerationCategoryScores {
            sexual: 0.5,
            hate: 0.0,
            harassment: 0.0,
            self_harm: 0.0,
            sexual_minors: 0.0,
            hate_threatening: 0.0,
            harassment_threatening: 0.0,
            self_harm_instructions: 0.0,
            self_harm_intent: 0.0,
            violence: 0.0,
            violence_graphic: 0.0,
        };

        let cloned = scores.clone();
        assert!((scores.sexual - cloned.sexual).abs() < f64::EPSILON);
    }

    // ==================== ModerationResult Tests ====================

    #[test]
    fn test_moderation_result_not_flagged() {
        let result = ModerationResult {
            flagged: false,
            categories: ModerationCategories {
                sexual: false,
                hate: false,
                harassment: false,
                self_harm: false,
                sexual_minors: false,
                hate_threatening: false,
                harassment_threatening: false,
                self_harm_instructions: false,
                self_harm_intent: false,
                violence: false,
                violence_graphic: false,
            },
            category_scores: ModerationCategoryScores {
                sexual: 0.001,
                hate: 0.002,
                harassment: 0.003,
                self_harm: 0.001,
                sexual_minors: 0.0001,
                hate_threatening: 0.0005,
                harassment_threatening: 0.001,
                self_harm_instructions: 0.0002,
                self_harm_intent: 0.0003,
                violence: 0.002,
                violence_graphic: 0.001,
            },
        };

        assert!(!result.flagged);
    }

    #[test]
    fn test_moderation_result_flagged() {
        let result = ModerationResult {
            flagged: true,
            categories: ModerationCategories {
                sexual: false,
                hate: true,
                harassment: false,
                self_harm: false,
                sexual_minors: false,
                hate_threatening: false,
                harassment_threatening: false,
                self_harm_instructions: false,
                self_harm_intent: false,
                violence: false,
                violence_graphic: false,
            },
            category_scores: ModerationCategoryScores {
                sexual: 0.01,
                hate: 0.92,
                harassment: 0.15,
                self_harm: 0.01,
                sexual_minors: 0.001,
                hate_threatening: 0.05,
                harassment_threatening: 0.02,
                self_harm_instructions: 0.005,
                self_harm_intent: 0.003,
                violence: 0.08,
                violence_graphic: 0.02,
            },
        };

        assert!(result.flagged);
        assert!(result.categories.hate);
    }

    #[test]
    fn test_moderation_result_clone() {
        let result = ModerationResult {
            flagged: true,
            categories: ModerationCategories {
                sexual: false,
                hate: false,
                harassment: false,
                self_harm: false,
                sexual_minors: false,
                hate_threatening: false,
                harassment_threatening: false,
                self_harm_instructions: false,
                self_harm_intent: false,
                violence: true,
                violence_graphic: false,
            },
            category_scores: ModerationCategoryScores {
                sexual: 0.0,
                hate: 0.0,
                harassment: 0.0,
                self_harm: 0.0,
                sexual_minors: 0.0,
                hate_threatening: 0.0,
                harassment_threatening: 0.0,
                self_harm_instructions: 0.0,
                self_harm_intent: 0.0,
                violence: 0.85,
                violence_graphic: 0.0,
            },
        };

        let cloned = result.clone();
        assert_eq!(result.flagged, cloned.flagged);
    }

    // ==================== ModerationResponse Tests ====================

    #[test]
    fn test_moderation_response_structure() {
        let response = ModerationResponse {
            id: "modr-123".to_string(),
            model: "text-moderation-007".to_string(),
            results: vec![],
        };

        assert_eq!(response.id, "modr-123");
        assert_eq!(response.model, "text-moderation-007");
        assert!(response.results.is_empty());
    }

    #[test]
    fn test_moderation_response_with_results() {
        let result = ModerationResult {
            flagged: false,
            categories: ModerationCategories {
                sexual: false,
                hate: false,
                harassment: false,
                self_harm: false,
                sexual_minors: false,
                hate_threatening: false,
                harassment_threatening: false,
                self_harm_instructions: false,
                self_harm_intent: false,
                violence: false,
                violence_graphic: false,
            },
            category_scores: ModerationCategoryScores {
                sexual: 0.0,
                hate: 0.0,
                harassment: 0.0,
                self_harm: 0.0,
                sexual_minors: 0.0,
                hate_threatening: 0.0,
                harassment_threatening: 0.0,
                self_harm_instructions: 0.0,
                self_harm_intent: 0.0,
                violence: 0.0,
                violence_graphic: 0.0,
            },
        };

        let response = ModerationResponse {
            id: "modr-456".to_string(),
            model: "text-moderation-latest".to_string(),
            results: vec![result],
        };

        assert_eq!(response.results.len(), 1);
        assert!(!response.results[0].flagged);
    }

    #[test]
    fn test_moderation_response_serialization() {
        let response = ModerationResponse {
            id: "modr-ser".to_string(),
            model: "model".to_string(),
            results: vec![],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "modr-ser");
        assert_eq!(json["model"], "model");
        assert!(json["results"].is_array());
    }

    #[test]
    fn test_moderation_response_clone() {
        let response = ModerationResponse {
            id: "modr-clone".to_string(),
            model: "model".to_string(),
            results: vec![],
        };

        let cloned = response.clone();
        assert_eq!(response.id, cloned.id);
    }
}
