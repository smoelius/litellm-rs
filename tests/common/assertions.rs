//! Custom test assertions
//!
//! Provides domain-specific assertions for testing litellm-rs components.

use litellm_rs::core::types::responses::ChatResponse;

/// Assertions for ChatResponse
pub trait ChatResponseAssertions {
    /// Assert response has at least one choice
    fn assert_has_choices(&self);

    /// Assert response has usage information
    fn assert_has_usage(&self);
}

impl ChatResponseAssertions for ChatResponse {
    fn assert_has_choices(&self) {
        assert!(
            !self.choices.is_empty(),
            "Expected response to have at least one choice, got none"
        );
    }

    fn assert_has_usage(&self) {
        assert!(
            self.usage.is_some(),
            "Expected response to have usage information"
        );
        let usage = self.usage.as_ref().unwrap();
        assert!(usage.prompt_tokens > 0, "Expected positive prompt_tokens");
    }
}

/// Assert two values are approximately equal (for floats)
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr) => {
        assert_approx_eq!($left, $right, 1e-6_f64)
    };
    ($left:expr, $right:expr, $epsilon:expr) => {
        let left_val: f64 = $left as f64;
        let right_val: f64 = $right as f64;
        let diff = (left_val - right_val).abs();
        assert!(
            diff < $epsilon,
            "assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`,\n  diff: `{:?}` (epsilon: `{:?}`)",
            left_val,
            right_val,
            diff,
            $epsilon
        );
    };
}

/// Assert a duration is within bounds
#[macro_export]
macro_rules! assert_duration_within {
    ($duration:expr, $max_ms:expr) => {
        let millis = $duration.as_millis();
        assert!(
            millis <= $max_ms,
            "Duration {} ms exceeded maximum {} ms",
            millis,
            $max_ms
        );
    };
}

/// Assert a collection contains an item matching a predicate
#[macro_export]
macro_rules! assert_contains {
    ($collection:expr, $predicate:expr) => {
        assert!(
            $collection.iter().any($predicate),
            "Collection does not contain expected item"
        );
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_approx_eq_macro() {
        assert_approx_eq!(1.0, 1.0);
        assert_approx_eq!(1.0, 1.0000001);
        assert_approx_eq!(0.1 + 0.2, 0.3, 1e-10_f64);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_approx_eq_failure() {
        assert_approx_eq!(1.0, 2.0);
    }

    #[test]
    fn test_duration_within_macro() {
        use std::time::Duration;
        assert_duration_within!(Duration::from_millis(50), 100);
    }

    #[test]
    fn test_contains_macro() {
        let items = [1, 2, 3, 4, 5];
        assert_contains!(items, |&x| x == 3);
    }
}
