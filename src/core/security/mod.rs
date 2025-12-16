//! Security and content filtering system
//!
//! This module provides comprehensive security features including PII detection,
//! content filtering, and compliance tools.

// Module declarations
mod filter;
mod gdpr;
mod patterns;
mod profanity;
mod types;

// Re-export all public types and functions for backward compatibility
pub use filter::ContentFilter;
pub use gdpr::GDPRCompliance;
pub use patterns::{CREDIT_CARD_PATTERN, EMAIL_PATTERN, PHONE_PATTERN, SSN_PATTERN};
pub use profanity::ProfanityFilter;
pub use types::{
    AnonymizationMethod, AnonymizationRule, ConsentManager, ContentIssue, CustomFilter,
    DataExportTools, ExportFormat, FilterResult, ModerationAction, ModerationRule,
    ModerationSeverity, ModerationType, PIIPattern, PIIReplacement, RetentionPolicy, UserConsent,
};
