//! Background health check tasks

use std::time::Duration;
use tracing::error;

use super::checker::HealthChecker;

impl HealthChecker {
    /// Start background health check tasks
    pub(super) async fn start_health_check_tasks(&self) {
        let health_checker = self.clone();

        // Main health check task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                if !health_checker.is_active() {
                    break;
                }

                if let Err(e) = health_checker.check_all().await {
                    error!("Health check failed: {}", e);
                }
            }
        });

        // Component-specific health checks can be added here
        // with different intervals for different components
    }
}
