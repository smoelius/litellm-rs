//! Content filtering and moderation
//!
//! Main content filtering implementation with PII detection and moderation.

use crate::core::models::openai::*;
use crate::utils::error::Result;
use tracing::warn;

use super::patterns::*;
use super::profanity::ProfanityFilter;
use super::types::*;

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
}
