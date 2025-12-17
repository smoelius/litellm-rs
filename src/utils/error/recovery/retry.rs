//! Retry mechanism with exponential backoff

use super::types::RetryConfig;
use std::time::Duration;
use tracing::{debug, error};

/// Retry mechanism with exponential backoff
#[allow(dead_code)]
pub struct RetryPolicy {
    config: RetryConfig,
}

#[allow(dead_code)]
impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn call<F, Fut, R, E>(&self, mut f: F) -> std::result::Result<R, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<R, E>>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        let mut attempt = 0;
        let mut delay = self.config.base_delay;

        loop {
            attempt += 1;

            match f().await {
                Ok(result) => {
                    if attempt > 1 {
                        debug!("Retry succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    if attempt >= self.config.max_attempts {
                        error!("Retry failed after {} attempts: {}", attempt, error);
                        return Err(error);
                    }

                    debug!(
                        "Attempt {} failed: {}, retrying in {:?}",
                        attempt, error, delay
                    );

                    // Sleep with optional jitter
                    let actual_delay = if self.config.jitter {
                        let jitter_factor = 0.1;
                        let jitter = delay.as_millis() as f64
                            * jitter_factor
                            * (rand::random::<f64>() - 0.5);
                        Duration::from_millis((delay.as_millis() as f64 + jitter) as u64)
                    } else {
                        delay
                    };

                    tokio::time::sleep(actual_delay).await;

                    // Calculate next delay with exponential backoff
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64,
                        ),
                        self.config.max_delay,
                    );
                }
            }
        }
    }
}
