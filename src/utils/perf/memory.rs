//! Memory pool utilities for efficient memory management
//!
//! This module provides utilities to reduce memory allocations and improve
//! performance through object pooling and reuse.

#![allow(dead_code)] // Tool module - functions may be used in the future

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

/// A generic object pool for reusing expensive-to-create objects
pub struct ObjectPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> ObjectPool<T>
where
    T: Send + 'static,
{
    /// Create a new object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            factory: Box::new(factory),
            max_size,
        }
    }

    /// Get an object from the pool or create a new one
    pub fn get(&self) -> PooledObject<T> {
        let obj = {
            let mut pool = self.pool.lock();
            pool.pop_front().unwrap_or_else(|| (self.factory)())
        };

        PooledObject {
            obj: Some(obj),
            pool: Arc::clone(&self.pool),
            max_size: self.max_size,
        }
    }

    /// Get the current size of the pool
    pub fn size(&self) -> usize {
        self.pool.lock().len()
    }

    /// Clear the pool
    pub fn clear(&self) {
        self.pool.lock().clear();
    }
}

/// A wrapper around a pooled object that returns it to the pool when dropped
pub struct PooledObject<T> {
    obj: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    max_size: usize,
}

impl<T> PooledObject<T> {
    /// Get a reference to the inner object
    pub fn get_ref(&self) -> &T {
        self.obj.as_ref().expect("Object already taken")
    }

    /// Get a mutable reference to the inner object
    pub fn get_mut(&mut self) -> &mut T {
        self.obj.as_mut().expect("Object already taken")
    }

    /// Take the object out of the pool wrapper (prevents return to pool)
    pub fn take(mut self) -> T {
        self.obj.take().expect("Object already taken")
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            let mut pool = self.pool.lock();
            if pool.len() < self.max_size {
                pool.push_back(obj);
            }
            // If pool is full, just drop the object
        }
    }
}

/// A memory-efficient buffer pool for byte operations
pub struct BufferPool {
    pool: ObjectPool<Vec<u8>>,
    default_capacity: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(default_capacity: usize, max_pooled: usize) -> Self {
        let pool = ObjectPool::new(move || Vec::with_capacity(default_capacity), max_pooled);

        Self {
            pool,
            default_capacity,
        }
    }

    /// Get a buffer from the pool
    pub fn get(&self) -> PooledBuffer {
        let mut buffer = self.pool.get();
        buffer.clear(); // Ensure buffer is empty
        PooledBuffer { buffer }
    }

    /// Get a buffer with a specific capacity
    pub fn get_with_capacity(&self, capacity: usize) -> PooledBuffer {
        let mut buffer = self.pool.get();
        buffer.clear();
        let current_capacity = buffer.capacity();
        if current_capacity < capacity {
            buffer.reserve(capacity - current_capacity);
        }
        PooledBuffer { buffer }
    }
}

/// A pooled buffer wrapper
pub struct PooledBuffer {
    buffer: PooledObject<Vec<u8>>,
}

impl PooledBuffer {
    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.get_mut().clear();
    }

    /// Extend the buffer with data
    pub fn extend_from_slice(&mut self, data: &[u8]) {
        self.buffer.get_mut().extend_from_slice(data);
    }

    /// Get a slice of the buffer
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.get_ref().as_slice()
    }

    /// Take the inner Vec (prevents return to pool)
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer.take()
    }
}

impl std::ops::Deref for PooledBuffer {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl std::ops::DerefMut for PooledBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.get_mut()
    }
}

/// A string pool optimized for specific use cases
pub struct OptimizedStringPool {
    small_strings: ObjectPool<String>,  // For strings < 64 chars
    medium_strings: ObjectPool<String>, // For strings < 256 chars
    large_strings: ObjectPool<String>,  // For strings >= 256 chars
}

impl OptimizedStringPool {
    /// Create a new optimized string pool
    pub fn new() -> Self {
        Self {
            small_strings: ObjectPool::new(|| String::with_capacity(64), 100),
            medium_strings: ObjectPool::new(|| String::with_capacity(256), 50),
            large_strings: ObjectPool::new(|| String::with_capacity(1024), 20),
        }
    }

    /// Get a string from the appropriate pool
    pub fn get_string(&self, estimated_size: usize) -> PooledString {
        let pooled = if estimated_size < 64 {
            PooledStringType::Small(self.small_strings.get())
        } else if estimated_size < 256 {
            PooledStringType::Medium(self.medium_strings.get())
        } else {
            PooledStringType::Large(self.large_strings.get())
        };

        let mut string = PooledString { inner: pooled };
        string.clear();
        string
    }
}

impl Default for OptimizedStringPool {
    fn default() -> Self {
        Self::new()
    }
}

/// A pooled string wrapper
pub struct PooledString {
    inner: PooledStringType,
}

enum PooledStringType {
    Small(PooledObject<String>),
    Medium(PooledObject<String>),
    Large(PooledObject<String>),
}

impl PooledString {
    /// Clear the string
    pub fn clear(&mut self) {
        match &mut self.inner {
            PooledStringType::Small(s) => s.clear(),
            PooledStringType::Medium(s) => s.clear(),
            PooledStringType::Large(s) => s.clear(),
        }
    }

    /// Get the string length
    pub fn len(&self) -> usize {
        match &self.inner {
            PooledStringType::Small(s) => s.len(),
            PooledStringType::Medium(s) => s.len(),
            PooledStringType::Large(s) => s.len(),
        }
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Push a string slice
    pub fn push_str(&mut self, s: &str) {
        match &mut self.inner {
            PooledStringType::Small(string) => string.push_str(s),
            PooledStringType::Medium(string) => string.push_str(s),
            PooledStringType::Large(string) => string.push_str(s),
        }
    }

    /// Take the inner string (prevents return to pool)
    pub fn into_string(self) -> String {
        match self.inner {
            PooledStringType::Small(s) => s.take(),
            PooledStringType::Medium(s) => s.take(),
            PooledStringType::Large(s) => s.take(),
        }
    }
}

impl std::ops::Deref for PooledString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            PooledStringType::Small(s) => s.as_str(),
            PooledStringType::Medium(s) => s.as_str(),
            PooledStringType::Large(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for PooledString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            PooledStringType::Small(s) => write!(f, "{}", s.as_str()),
            PooledStringType::Medium(s) => write!(f, "{}", s.as_str()),
            PooledStringType::Large(s) => write!(f, "{}", s.as_str()),
        }
    }
}

/// Global instances for common use cases
use once_cell::sync::Lazy;

/// Global buffer pool
pub static BUFFER_POOL: Lazy<BufferPool> = Lazy::new(|| BufferPool::new(1024, 50));

/// Global string pool
pub static STRING_POOL: Lazy<OptimizedStringPool> = Lazy::new(OptimizedStringPool::new);

/// Convenience functions
///
/// Get a buffer from the global buffer pool
pub fn get_buffer() -> PooledBuffer {
    BUFFER_POOL.get()
}

/// Get a buffer with specific capacity from the global buffer pool
pub fn get_buffer_with_capacity(capacity: usize) -> PooledBuffer {
    BUFFER_POOL.get_with_capacity(capacity)
}

/// Get a string from the global string pool
pub fn get_string(estimated_size: usize) -> PooledString {
    STRING_POOL.get_string(estimated_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::new(|| Vec::<i32>::new(), 2);

        {
            let mut obj1 = pool.get();
            obj1.push(1);
            assert_eq!(obj1.len(), 1);
        } // obj1 returned to pool

        assert_eq!(pool.size(), 1);

        let obj2 = pool.get();
        assert_eq!(obj2.len(), 0); // Should be cleared when reused
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(64, 5);

        {
            let mut buffer = pool.get();
            buffer.extend_from_slice(b"hello");
            assert_eq!(buffer.len(), 5);
        }

        let buffer2 = pool.get();
        assert_eq!(buffer2.len(), 0); // Should be cleared
    }

    #[test]
    fn test_string_pool() {
        let pool = OptimizedStringPool::new();

        {
            let mut string = pool.get_string(10);
            string.push_str("hello");
            assert_eq!(string.len(), 5);
        }

        let string2 = pool.get_string(10);
        assert_eq!(string2.len(), 0); // Should be cleared
    }
}
