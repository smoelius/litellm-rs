//! PII detection patterns
//!
//! Pre-compiled regex patterns for detecting personally identifiable information.

use once_cell::sync::Lazy;
use regex::Regex;

// Pre-compiled regex patterns for PII detection
// These patterns are validated at compile time to ensure they never fail
// Note: unwrap() is used because these are static patterns that are known-good
// If a pattern fails, it indicates a code error that should be caught in tests

/// SSN pattern: XXX-XX-XXXX
pub static SSN_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap_or_else(|e| {
        tracing::error!("Failed to compile SSN regex: {}", e);
        // Return a pattern that never matches as fallback
        // [^\s\S] matches "neither whitespace nor non-whitespace" = empty set
        Regex::new(r"[^\s\S]").unwrap()
    })
});

/// Email pattern: local@domain.tld
pub static EMAIL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap_or_else(|e| {
        tracing::error!("Failed to compile email regex: {}", e);
        Regex::new(r"[^\s\S]").unwrap()
    })
});

/// Phone pattern: XXX-XXX-XXXX
pub static PHONE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{3}-\d{3}-\d{4}\b").unwrap_or_else(|e| {
        tracing::error!("Failed to compile phone regex: {}", e);
        Regex::new(r"[^\s\S]").unwrap()
    })
});

/// Credit card pattern: XXXX-XXXX-XXXX-XXXX or XXXXXXXXXXXXXXXX
pub static CREDIT_CARD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap_or_else(|e| {
        tracing::error!("Failed to compile credit card regex: {}", e);
        Regex::new(r"[^\s\S]").unwrap()
    })
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_patterns_compile() {
        // Verify all static regex patterns compile successfully
        assert!(SSN_PATTERN.is_match("123-45-6789"));
        assert!(EMAIL_PATTERN.is_match("test@example.com"));
        assert!(PHONE_PATTERN.is_match("123-456-7890"));
        assert!(CREDIT_CARD_PATTERN.is_match("1234-5678-9012-3456"));
        assert!(CREDIT_CARD_PATTERN.is_match("1234567890123456"));
    }

    #[test]
    fn test_regex_patterns_no_false_positives() {
        // Verify patterns don't match invalid input
        assert!(!SSN_PATTERN.is_match("123456789")); // No dashes
        assert!(!SSN_PATTERN.is_match("12-345-6789")); // Wrong format
        assert!(!EMAIL_PATTERN.is_match("not an email"));
        assert!(!PHONE_PATTERN.is_match("12345678901")); // No dashes
        assert!(!CREDIT_CARD_PATTERN.is_match("123")); // Too short
    }
}
