//! Helper functions for metrics calculations

use std::collections::VecDeque;

/// Calculate percentile from sorted values
pub(super) fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    if percentile >= 1.0 {
        // Safe: we checked is_empty() above
        return sorted_values.last().copied().unwrap_or(0.0);
    }

    let index = percentile * (sorted_values.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = (index.ceil() as usize).min(sorted_values.len() - 1);

    if lower == upper || lower >= sorted_values.len() {
        sorted_values.get(lower).copied().unwrap_or(0.0)
    } else {
        let weight = index - lower as f64;
        let lower_val = sorted_values.get(lower).copied().unwrap_or(0.0);
        let upper_val = sorted_values.get(upper).copied().unwrap_or(0.0);
        lower_val * (1.0 - weight) + upper_val * weight
    }
}

/// Calculate average of f64 values from any iterable
pub(super) fn calculate_average(values: &VecDeque<f64>) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

/// Calculate average of u64 values from any iterable
pub(super) fn calculate_average_u64(values: &VecDeque<u64>) -> u64 {
    if values.is_empty() {
        0
    } else {
        values.iter().sum::<u64>() / values.len() as u64
    }
}

/// Calculate average of u32 values from any iterable
pub(super) fn calculate_average_u32(values: &VecDeque<u32>) -> u32 {
    if values.is_empty() {
        0
    } else {
        values.iter().sum::<u32>() / values.len() as u32
    }
}
