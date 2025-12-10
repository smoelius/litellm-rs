//! End-to-end tests for litellm-rs
//!
//! These tests verify complete user flows and require real API keys.
//! Run with: cargo test --all-features -- --ignored
//!
//! Required environment variables:
//! - OPENAI_API_KEY: For OpenAI tests
//! - GROQ_API_KEY: For Groq tests
//! - ANTHROPIC_API_KEY: For Anthropic tests

pub mod audio;
pub mod chat_completion;
