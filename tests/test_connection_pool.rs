//! Integration test for connection pool

use litellm_rs::core::providers::base::{
    ConnectionPool, GlobalPoolManager, HttpMethod, PoolConfig,
};

#[tokio::test]
async fn test_pool_creation() {
    let pool = ConnectionPool::new();
    assert!(pool.is_ok(), "Should create connection pool successfully");
}

#[tokio::test]
async fn test_global_manager_creation() {
    let manager = GlobalPoolManager::new();
    assert!(
        manager.is_ok(),
        "Should create global pool manager successfully"
    );
}

#[tokio::test]
async fn test_pool_config_values() {
    assert_eq!(PoolConfig::TIMEOUT_SECS, 600);
    assert_eq!(PoolConfig::POOL_SIZE, 80);
    assert_eq!(PoolConfig::KEEPALIVE_SECS, 90);
}

#[tokio::test]
async fn test_http_methods() {
    let _ = HttpMethod::GET;
    let _ = HttpMethod::POST;
    let _ = HttpMethod::PUT;
    let _ = HttpMethod::DELETE;
}
