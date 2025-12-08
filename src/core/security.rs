//! Security and content filtering system
//!
//! This module provides comprehensive security features including PII detection,
//! content filtering, and compliance tools.

use crate::core::models::openai::*;
use crate::utils::error::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use std::collections::HashMap;
use tracing::warn;

// Pre-compiled regex patterns for PII detection
static SSN_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").expect("Invalid SSN regex"));
static EMAIL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").expect("Invalid email regex")
});
static PHONE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d{3}-\d{3}-\d{4}\b").expect("Invalid phone regex"));
static CREDIT_CARD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").expect("Invalid credit card regex")
});

/// Content filter for detecting and handling sensitive content
pub struct ContentFilter {
    /// PII detection patterns
    pii_patterns: Vec<PIIPattern>,
    /// Content moderation rules
    moderation_rules: Vec<ModerationRule>,
    /// Profanity filter
    profanity_filter: ProfanityFilter,
    /// Custom filters
    custom_filters: Vec<CustomFilter>,
}

/// PII (Personally Identifiable Information) pattern
#[derive(Debug, Clone)]
pub struct PIIPattern {
    /// Pattern name
    pub name: String,
    /// Regex pattern
    pub pattern: Regex,
    /// Replacement strategy
    pub replacement: PIIReplacement,
    /// Confidence level
    pub confidence: f64,
}

/// PII replacement strategies
#[derive(Debug, Clone)]
pub enum PIIReplacement {
    /// Redact with asterisks
    Redact,
    /// Replace with placeholder
    Placeholder(String),
    /// Hash the value
    Hash,
    /// Remove entirely
    Remove,
    /// Mask partially (keep first/last N characters)
    PartialMask {
        /// Number of characters to keep at start
        keep_start: usize,
        /// Number of characters to keep at end
        keep_end: usize,
    },
}

/// Content moderation rule
#[derive(Debug, Clone)]
pub struct ModerationRule {
    /// Rule name
    pub name: String,
    /// Rule type
    pub rule_type: ModerationType,
    /// Action to take
    pub action: ModerationAction,
    /// Severity level
    pub severity: ModerationSeverity,
}

/// Types of content moderation
#[derive(Debug, Clone)]
pub enum ModerationType {
    /// Hate speech detection
    HateSpeech,
    /// Violence detection
    Violence,
    /// Sexual content detection
    Sexual,
    /// Self-harm detection
    SelfHarm,
    /// Harassment detection
    Harassment,
    /// Illegal activity detection
    IllegalActivity,
    /// Custom category
    Custom(String),
}

/// Actions to take when content is flagged
#[derive(Debug, Clone)]
pub enum ModerationAction {
    /// Block the request
    Block,
    /// Warn but allow
    Warn,
    /// Log for review
    Log,
    /// Modify content
    Modify,
    /// Require human review
    HumanReview,
}

/// Severity levels for moderation
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ModerationSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Profanity filter
#[derive(Debug, Clone)]
pub struct ProfanityFilter {
    /// Blocked words list
    blocked_words: Vec<String>,
    /// Replacement character
    replacement_char: char,
    /// Whether to use fuzzy matching
    fuzzy_matching: bool,
}

/// Custom content filter
#[derive(Debug, Clone)]
pub struct CustomFilter {
    /// Filter name
    pub name: String,
    /// Filter pattern
    pub pattern: Regex,
    /// Filter action
    pub action: ModerationAction,
}

/// Content filtering result
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// Whether content should be blocked
    pub blocked: bool,
    /// Detected issues
    pub issues: Vec<ContentIssue>,
    /// Modified content (if applicable)
    pub modified_content: Option<String>,
    /// Confidence score
    pub confidence: f64,
}

/// Detected content issue
#[derive(Debug, Clone)]
pub struct ContentIssue {
    /// Issue type
    pub issue_type: String,
    /// Issue description
    pub description: String,
    /// Severity level
    pub severity: ModerationSeverity,
    /// Location in text (start, end)
    pub location: Option<(usize, usize)>,
    /// Confidence score
    pub confidence: f64,
}

/// GDPR compliance tools
pub struct GDPRCompliance {
    /// Data retention policies
    retention_policies: HashMap<String, RetentionPolicy>,
    /// Consent management
    consent_manager: ConsentManager,
    /// Data export tools
    export_tools: DataExportTools,
}

/// Data retention policy
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Data type
    pub data_type: String,
    /// Retention period in days
    pub retention_days: u32,
    /// Auto-deletion enabled
    pub auto_delete: bool,
    /// Anonymization rules
    pub anonymization: Option<AnonymizationRule>,
}

/// Consent management
#[derive(Debug, Clone)]
pub struct ConsentManager {
    /// User consents
    consents: HashMap<String, UserConsent>,
}

/// User consent information
#[derive(Debug, Clone)]
pub struct UserConsent {
    /// User ID
    pub user_id: String,
    /// Consent given
    pub consented: bool,
    /// Consent timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Consent version
    pub version: String,
    /// Specific permissions
    pub permissions: Vec<String>,
}

/// Data export tools
#[derive(Debug, Clone)]
pub struct DataExportTools {
    /// Supported export formats
    formats: Vec<ExportFormat>,
}

/// Export formats
#[derive(Debug, Clone)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// XML format
    Xml,
    /// PDF format
    Pdf,
}

/// Anonymization rule
#[derive(Debug, Clone)]
pub struct AnonymizationRule {
    /// Fields to anonymize
    pub fields: Vec<String>,
    /// Anonymization method
    pub method: AnonymizationMethod,
}

/// Anonymization methods
#[derive(Debug, Clone)]
pub enum AnonymizationMethod {
    /// Replace with random data
    Randomize,
    /// Hash the data
    Hash,
    /// Remove the data
    Remove,
    /// Generalize the data
    Generalize,
}

impl ContentFilter {
    /// Create a new content filter
    pub fn new() -> Self {
        Self {
            pii_patterns: Self::default_pii_patterns(),
            moderation_rules: Self::default_moderation_rules(),
            profanity_filter: ProfanityFilter::new(),
            custom_filters: Vec::new(),
        }
    }

    /// Filter chat completion request
    pub async fn filter_chat_request(
        &self,
        request: &mut ChatCompletionRequest,
    ) -> Result<FilterResult> {
        let mut issues = Vec::new();
        let mut blocked = false;
        let mut modified = false;

        // Check each message
        for message in &mut request.messages {
            if let Some(MessageContent::Text(text)) = &mut message.content {
                let result = self.filter_text(text).await?;

                if result.blocked {
                    blocked = true;
                }

                issues.extend(result.issues);

                if let Some(modified_text) = result.modified_content {
                    *text = modified_text;
                    modified = true;
                }
            }
        }

        let confidence = if issues.is_empty() {
            1.0
        } else {
            issues.iter().map(|i| i.confidence).sum::<f64>() / issues.len() as f64
        };

        Ok(FilterResult {
            blocked,
            issues,
            modified_content: if modified {
                Some("Messages modified".to_string())
            } else {
                None
            },
            confidence,
        })
    }

    /// Filter text content
    pub async fn filter_text(&self, text: &str) -> Result<FilterResult> {
        let mut issues = Vec::new();
        let mut modified_text = text.to_string();
        let mut blocked = false;

        // PII detection
        for pattern in &self.pii_patterns {
            if let Some(captures) = pattern.pattern.captures(text) {
                let issue = ContentIssue {
                    issue_type: format!("PII_{}", pattern.name),
                    description: format!("Detected {} in content", pattern.name),
                    severity: ModerationSeverity::High,
                    location: captures.get(0).map(|m| (m.start(), m.end())),
                    confidence: pattern.confidence,
                };
                issues.push(issue);

                // Apply replacement
                modified_text = self.apply_pii_replacement(&modified_text, pattern)?;
            }
        }

        // Content moderation
        for rule in &self.moderation_rules {
            if self.check_moderation_rule(&modified_text, rule).await? {
                let issue = ContentIssue {
                    issue_type: format!("MODERATION_{:?}", rule.rule_type),
                    description: format!("Content flagged for {:?}", rule.rule_type),
                    severity: rule.severity.clone(),
                    location: None,
                    confidence: 0.8, // Default confidence
                };
                issues.push(issue);

                match rule.action {
                    ModerationAction::Block => blocked = true,
                    ModerationAction::Warn => warn!("Content warning: {:?}", rule.rule_type),
                    _ => {}
                }
            }
        }

        // Profanity filtering
        if self.profanity_filter.contains_profanity(&modified_text) {
            modified_text = self.profanity_filter.filter(&modified_text);
            issues.push(ContentIssue {
                issue_type: "PROFANITY".to_string(),
                description: "Profanity detected and filtered".to_string(),
                severity: ModerationSeverity::Medium,
                location: None,
                confidence: 0.9,
            });
        }

        let confidence = if issues.is_empty() {
            1.0
        } else {
            issues.iter().map(|i| i.confidence).sum::<f64>() / issues.len() as f64
        };

        Ok(FilterResult {
            blocked,
            issues,
            modified_content: if modified_text != text {
                Some(modified_text)
            } else {
                None
            },
            confidence,
        })
    }

    /// Default PII patterns
    fn default_pii_patterns() -> Vec<PIIPattern> {
        vec![
            PIIPattern {
                name: "SSN".to_string(),
                pattern: SSN_PATTERN.clone(),
                replacement: PIIReplacement::Placeholder("XXX-XX-XXXX".to_string()),
                confidence: 0.95,
            },
            PIIPattern {
                name: "Email".to_string(),
                pattern: EMAIL_PATTERN.clone(),
                replacement: PIIReplacement::PartialMask {
                    keep_start: 2,
                    keep_end: 0,
                },
                confidence: 0.9,
            },
            PIIPattern {
                name: "Phone".to_string(),
                pattern: PHONE_PATTERN.clone(),
                replacement: PIIReplacement::Placeholder("XXX-XXX-XXXX".to_string()),
                confidence: 0.85,
            },
            PIIPattern {
                name: "CreditCard".to_string(),
                pattern: CREDIT_CARD_PATTERN.clone(),
                replacement: PIIReplacement::Placeholder("XXXX-XXXX-XXXX-XXXX".to_string()),
                confidence: 0.9,
            },
        ]
    }

    /// Default moderation rules
    fn default_moderation_rules() -> Vec<ModerationRule> {
        vec![
            ModerationRule {
                name: "Hate Speech".to_string(),
                rule_type: ModerationType::HateSpeech,
                action: ModerationAction::Block,
                severity: ModerationSeverity::High,
            },
            ModerationRule {
                name: "Violence".to_string(),
                rule_type: ModerationType::Violence,
                action: ModerationAction::Warn,
                severity: ModerationSeverity::Medium,
            },
        ]
    }

    /// Apply PII replacement
    fn apply_pii_replacement(&self, text: &str, pattern: &PIIPattern) -> Result<String> {
        let result = match &pattern.replacement {
            PIIReplacement::Redact => pattern.pattern.replace_all(text, "***").to_string(),
            PIIReplacement::Placeholder(placeholder) => pattern
                .pattern
                .replace_all(text, placeholder.as_str())
                .to_string(),
            PIIReplacement::Hash => {
                // Simple hash replacement (in production, use proper hashing)
                pattern.pattern.replace_all(text, "[HASHED]").to_string()
            }
            PIIReplacement::Remove => pattern.pattern.replace_all(text, "").to_string(),
            PIIReplacement::PartialMask {
                keep_start,
                keep_end,
            } => {
                // Implement partial masking logic
                pattern
                    .pattern
                    .replace_all(text, |caps: &regex::Captures| {
                        // caps.get(0) should always succeed for a match, but handle gracefully
                        let matched = match caps.get(0) {
                            Some(m) => m.as_str(),
                            None => return String::new(),
                        };
                        let len = matched.len();
                        if len <= keep_start + keep_end {
                            "*".repeat(len)
                        } else {
                            let start = &matched[..*keep_start];
                            let end = if *keep_end > 0 {
                                &matched[len - keep_end..]
                            } else {
                                ""
                            };
                            let middle = "*".repeat(len - keep_start - keep_end);
                            format!("{}{}{}", start, middle, end)
                        }
                    })
                    .to_string()
            }
        };
        Ok(result)
    }

    /// Check moderation rule
    async fn check_moderation_rule(&self, text: &str, rule: &ModerationRule) -> Result<bool> {
        // Simplified moderation check - in production, integrate with external services
        match rule.rule_type {
            ModerationType::HateSpeech => {
                Ok(text.to_lowercase().contains("hate") || text.to_lowercase().contains("racist"))
            }
            ModerationType::Violence => Ok(
                text.to_lowercase().contains("violence") || text.to_lowercase().contains("kill")
            ),
            _ => Ok(false),
        }
    }
}

impl Default for ProfanityFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfanityFilter {
    /// Create a new profanity filter
    pub fn new() -> Self {
        Self {
            blocked_words: vec![
                "badword1".to_string(),
                "badword2".to_string(),
                // Add more blocked words
            ],
            replacement_char: '*',
            fuzzy_matching: true,
        }
    }

    /// Check if text contains profanity
    pub fn contains_profanity(&self, text: &str) -> bool {
        let lower_text = text.to_lowercase();
        self.blocked_words
            .iter()
            .any(|word| lower_text.contains(word))
    }

    /// Filter profanity from text
    pub fn filter(&self, text: &str) -> String {
        let mut result = text.to_string();
        for word in &self.blocked_words {
            let replacement = self.replacement_char.to_string().repeat(word.len());
            result = result.replace(word, &replacement);
        }
        result
    }
}

impl Default for ContentFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pii_detection() {
        let filter = ContentFilter::new();
        let text = "My SSN is 123-45-6789 and email is test@example.com";

        let result = filter.filter_text(text).await.unwrap();
        assert!(!result.issues.is_empty());
        assert!(result.modified_content.is_some());
    }

    #[tokio::test]
    async fn test_profanity_filter() {
        let filter = ProfanityFilter::new();
        assert!(filter.contains_profanity("This contains badword1"));

        let filtered = filter.filter("This contains badword1");
        assert!(!filtered.contains("badword1"));
    }
}
