//! Redis operations implementation
//!
//! Provides batch operations for Redis with pipeline support.

use redis::{AsyncCommands, Pipeline, RedisResult, aio::MultiplexedConnection};

/// Batch set operations with pipeline for better performance
///
/// Uses Redis pipeline for atomic batch operations.
pub(super) async fn batch_set_operation(
    mut conn: MultiplexedConnection,
    pairs: &[(String, String)],
    ttl: Option<u64>,
) -> RedisResult<()> {
    let mut pipe = Pipeline::new();
    pipe.atomic();

    for (key, value) in pairs {
        if let Some(ttl_seconds) = ttl {
            pipe.set_ex(key, value, ttl_seconds);
        } else {
            pipe.set(key, value);
        }
    }

    pipe.query_async(&mut conn).await
}

/// Batch get operations using MGET
pub(super) async fn batch_get_operation(
    mut conn: MultiplexedConnection,
    keys: &[String],
) -> RedisResult<Vec<Option<String>>> {
    conn.mget(keys).await
}

/// Batch delete operations
///
/// Returns the number of keys deleted.
pub(super) async fn batch_delete_operation(
    mut conn: MultiplexedConnection,
    keys: &[String],
) -> RedisResult<u64> {
    conn.del(keys).await
}
