//! Test suite for litellm-rs
//!
//! This module organizes tests into three categories:
//!
//! ## Test Categories
//!
//! ### 1. Common Utilities (`common/`)
//! Shared test infrastructure including:
//! - In-memory database helpers
//! - Test fixtures and factories
//! - Provider test utilities
//! - Custom assertions
//!
//! ### 2. Integration Tests (`integration/`)
//! Tests that verify component interactions:
//! - Database operations
//! - Router and load balancer
//! - Provider configuration
//!
//! ### 3. End-to-End Tests (`e2e/`)
//! Full system tests requiring real API keys:
//! - Run with: `cargo test --all-features -- --ignored`
//! - Set environment variables for providers (GROQ_API_KEY, etc.)
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all fast tests (default)
//! cargo test --all-features
//!
//! # Run only unit tests
//! cargo test --lib --all-features
//!
//! # Run integration tests
//! cargo test --test lib --all-features
//!
//! # Run E2E tests (requires API keys)
//! cargo test --all-features -- --ignored
//!
//! # Run tests with coverage
//! cargo llvm-cov --all-features
//! ```

pub mod common;
pub mod e2e;
pub mod integration;
