//! Bounded collection utilities for metrics

use std::collections::VecDeque;

/// Maximum number of samples to retain for time-series metrics
pub(super) const MAX_METRIC_SAMPLES: usize = 10_000;

/// Maximum number of recent requests/errors to track (for rate calculations)
pub(super) const MAX_RECENT_EVENTS: usize = 1_000;

/// Helper trait for bounded VecDeque operations
pub(super) trait BoundedPush<T> {
    fn push_bounded(&mut self, value: T, max_size: usize);
}

impl<T> BoundedPush<T> for VecDeque<T> {
    /// Push a value while maintaining a maximum size (O(1) amortized)
    #[inline]
    fn push_bounded(&mut self, value: T, max_size: usize) {
        if self.len() >= max_size {
            self.pop_front();
        }
        self.push_back(value);
    }
}
