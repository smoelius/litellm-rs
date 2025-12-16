//! Tests for logging module

#[cfg(test)]
mod tests {
    use crate::utils::logging::logging::async_logger::AsyncLogger;
    use crate::utils::logging::logging::sampler::LogSampler;
    use crate::utils::logging::logging::types::AsyncLoggerConfig;
    use tracing::Level;

    #[test]
    fn test_log_sampler() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("test", 0.5);

        // Should sample approximately half the logs
        let mut sampled_count = 0;
        for _ in 0..1000 {
            if sampler.should_log("test") {
                sampled_count += 1;
            }
        }

        // Allow some variance due to sampling
        assert!(sampled_count > 400 && sampled_count < 600);
    }

    #[test]
    fn test_log_sampler_edge_cases() {
        let mut sampler = LogSampler::new();

        // Test 100% sampling
        sampler.set_sample_rate("full", 1.0);
        let mut count = 0;
        for _ in 0..100 {
            if sampler.should_log("full") {
                count += 1;
            }
        }
        assert_eq!(count, 100);

        // Test 0% sampling
        sampler.set_sample_rate("none", 0.0);
        count = 0;
        for _ in 0..100 {
            if sampler.should_log("none") {
                count += 1;
            }
        }
        assert_eq!(count, 0);

        // Test 10% sampling
        sampler.set_sample_rate("ten_percent", 0.1);
        count = 0;
        for _ in 0..1000 {
            if sampler.should_log("ten_percent") {
                count += 1;
            }
        }
        // Should be exactly 100 (every 10th log)
        assert_eq!(count, 100);
    }

    #[test]
    fn test_async_logger_config() {
        let config = AsyncLoggerConfig {
            buffer_size: 5000,
            drop_on_overflow: true,
            sample_rate: 0.8,
            max_message_length: 512,
        };

        assert_eq!(config.buffer_size, 5000);
        assert!(config.drop_on_overflow);
        assert_eq!(config.sample_rate, 0.8);
        assert_eq!(config.max_message_length, 512);
    }

    #[tokio::test]
    async fn test_async_logger_creation() {
        let config = AsyncLoggerConfig::default();
        let logger = AsyncLogger::new(config);

        // Test basic logging
        logger.log(Level::INFO, "test", "test message");

        // Give background task time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_async_logger_bounded_channel() {
        // Create logger with small buffer to test backpressure
        let config = AsyncLoggerConfig {
            buffer_size: 10,
            drop_on_overflow: true,
            sample_rate: 1.0,
            max_message_length: 100,
        };
        let logger = AsyncLogger::new(config);

        // Send more messages than buffer can hold
        for i in 0..100 {
            logger.log(Level::INFO, "test", &format!("message {}", i));
        }

        // Should not panic or hang - messages are dropped when buffer full
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_async_logger_sampling() {
        let config = AsyncLoggerConfig {
            buffer_size: 1000,
            drop_on_overflow: false,
            sample_rate: 0.5, // 50% sampling
            max_message_length: 100,
        };
        let logger = AsyncLogger::new(config);

        // The sampling counter is internal, so we just verify no panic
        for i in 0..100 {
            logger.log(Level::INFO, "test", &format!("sampled message {}", i));
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
}
