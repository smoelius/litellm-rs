//! Bounded histogram for metrics collection

use std::collections::VecDeque;

/// Maximum number of samples to keep in histogram (prevents unbounded memory growth)
pub const HISTOGRAM_MAX_SAMPLES: usize = 1000;

/// Bounded histogram that maintains a rolling window of samples
#[derive(Debug, Clone)]
pub struct BoundedHistogram {
    /// Rolling window of duration samples
    samples: VecDeque<f64>,
    /// Maximum number of samples to retain
    max_samples: usize,
    /// Running sum for efficient mean calculation
    sum: f64,
    /// Total count of all samples ever recorded (for accurate counting)
    total_count: u64,
}

impl BoundedHistogram {
    /// Create a new bounded histogram with specified capacity
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            sum: 0.0,
            total_count: 0,
        }
    }

    /// Record a new duration sample
    pub fn record(&mut self, value: f64) {
        self.total_count += 1;
        self.sum += value;

        // If at capacity, remove oldest sample from sum
        if self.samples.len() >= self.max_samples {
            if let Some(oldest) = self.samples.pop_front() {
                self.sum -= oldest;
            }
        }

        self.samples.push_back(value);
    }

    /// Get the mean of current samples
    pub fn mean(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.sum / self.samples.len() as f64
        }
    }

    /// Get percentile value (0-100)
    pub fn percentile(&self, p: f64) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f64> = self.samples.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Use linear interpolation for more accurate percentiles
        let n = sorted.len();
        if n == 1 {
            return sorted[0];
        }

        // Calculate position using the standard percentile formula
        let pos = (p / 100.0) * (n - 1) as f64;
        let lower = pos.floor() as usize;
        let upper = pos.ceil() as usize;

        if lower == upper {
            sorted[lower]
        } else {
            // Linear interpolation between lower and upper
            let frac = pos - lower as f64;
            sorted[lower] * (1.0 - frac) + sorted[upper] * frac
        }
    }

    /// Get the total count of samples ever recorded
    pub fn count(&self) -> u64 {
        self.total_count
    }

    /// Get current number of samples in the window
    pub fn window_size(&self) -> usize {
        self.samples.len()
    }

    /// Get min value in current window
    pub fn min(&self) -> f64 {
        self.samples
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get max value in current window
    pub fn max(&self) -> f64 {
        self.samples
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }
}

impl Default for BoundedHistogram {
    fn default() -> Self {
        Self::new(HISTOGRAM_MAX_SAMPLES)
    }
}
