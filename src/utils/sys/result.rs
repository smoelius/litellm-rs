//! Result extension utilities for better error handling
//!
//! This module provides extension traits and utilities to make error handling
//! more ergonomic and reduce the need for unwrap() calls.

#![allow(dead_code)] // Tool module - functions may be used in the future

use crate::utils::error::{GatewayError, Result};
use tracing::{error, warn};

/// Extension trait for Result types to provide better error handling
pub trait ResultExt<T> {
    /// Log an error and return a default value instead of panicking
    fn unwrap_or_log_default(self, context: &str) -> T
    where
        T: Default;

    /// Log an error and return the provided default value
    fn unwrap_or_log(self, default: T, context: &str) -> T;

    /// Convert to a GatewayError with additional context
    fn with_context(self, context: &str) -> Result<T>;

    /// Log the error and continue with a default value
    fn log_and_continue(self, context: &str) -> Option<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn unwrap_or_log_default(self, context: &str) -> T
    where
        T: Default,
    {
        match self {
            Ok(value) => value,
            Err(e) => {
                error!("Error in {}: {}. Using default value.", context, e);
                T::default()
            }
        }
    }

    fn unwrap_or_log(self, default: T, context: &str) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                error!("Error in {}: {}. Using fallback value.", context, e);
                default
            }
        }
    }

    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| GatewayError::Internal(format!("{}: {}", context, e)))
    }

    fn log_and_continue(self, context: &str) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                warn!("Non-fatal error in {}: {}. Continuing...", context, e);
                None
            }
        }
    }
}

/// Extension trait for Option types
pub trait OptionExt<T> {
    /// Convert None to a GatewayError with context
    fn ok_or_context(self, context: &str) -> Result<T>;

    /// Log when None and return default
    fn unwrap_or_log_default(self, context: &str) -> T
    where
        T: Default;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_context(self, context: &str) -> Result<T> {
        self.ok_or_else(|| GatewayError::Internal(format!("Missing required value: {}", context)))
    }

    fn unwrap_or_log_default(self, context: &str) -> T
    where
        T: Default,
    {
        match self {
            Some(value) => value,
            None => {
                warn!("Missing value in {}, using default", context);
                T::default()
            }
        }
    }
}

/// Utility for safe numeric conversions
pub trait SafeConvert<T> {
    /// Safely convert to target type with error context
    fn safe_convert(self, context: &str) -> Result<T>;
}

impl SafeConvert<usize> for u32 {
    fn safe_convert(self, context: &str) -> Result<usize> {
        usize::try_from(self).map_err(|e| {
            GatewayError::Internal(format!("Numeric conversion failed in {}: {}", context, e))
        })
    }
}

impl SafeConvert<u32> for usize {
    fn safe_convert(self, context: &str) -> Result<u32> {
        u32::try_from(self).map_err(|e| {
            GatewayError::Internal(format!("Numeric conversion failed in {}: {}", context, e))
        })
    }
}

/// Macro for safe unwrapping with context
#[macro_export]
macro_rules! safe_unwrap {
    ($expr:expr, $context:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                error!("Error in {}: {}", $context, e);
                return Err($crate::utils::error::GatewayError::Internal(format!(
                    "Failed in {}: {}",
                    $context, e
                )));
            }
        }
    };
}

/// Macro for safe option unwrapping with context
#[macro_export]
macro_rules! safe_unwrap_option {
    ($expr:expr, $context:expr) => {
        match $expr {
            Some(val) => val,
            None => {
                error!("Missing required value in {}", $context);
                return Err($crate::utils::error::GatewayError::Internal(format!(
                    "Missing required value in {}",
                    $context
                )));
            }
        }
    };
}

/// Utility function to create a configuration error
pub fn config_error(msg: &str) -> GatewayError {
    GatewayError::Config(msg.to_string())
}

/// Utility function to create an internal error
pub fn internal_error(msg: &str) -> GatewayError {
    GatewayError::Internal(msg.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_ext() {
        let ok_result: Result<i32> = Ok(42);
        let err_result: Result<i32> = Err(GatewayError::Internal("test error".to_string()));

        assert_eq!(ok_result.unwrap_or_log_default("test"), 42);
        assert_eq!(err_result.unwrap_or_log_default("test"), 0);

        let ok_result: Result<i32> = Ok(42);
        let err_result: Result<i32> = Err(GatewayError::Internal("test error".to_string()));

        assert_eq!(ok_result.unwrap_or_log(99, "test"), 42);
        assert_eq!(err_result.unwrap_or_log(99, "test"), 99);
    }

    #[test]
    fn test_option_ext() {
        let some_val = Some(42);
        let none_val: Option<i32> = None;

        assert_eq!(some_val.unwrap_or_log_default("test"), 42);
        assert_eq!(none_val.unwrap_or_log_default("test"), 0);
    }

    #[test]
    fn test_safe_convert() {
        let val: u32 = 42;
        let converted: Result<usize> = val.safe_convert("test conversion");
        assert!(converted.is_ok());
        assert_eq!(converted.unwrap(), 42);
    }
}
