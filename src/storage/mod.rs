pub mod memory;
pub mod redis_storage;

pub use memory::MemoryStorage;
pub use redis_storage::RedisStorage;

use std::time::Instant;

/// Represents the state of a token bucket
#[derive(Debug, Clone)]
pub struct BucketState {
    pub tokens: f64,
    pub last_refill: Instant,
}

//state for sliding window algorithim
#[derive(Debug, Clone)]
pub struct WindowState {
    pub prev_count: u32,
    pub curr_count: u32,
    pub window_start: Instant,
}
//Generic trait for rate limit storage backends
// T is the state type (BucketState or WindowState)
#[async_trait::async_trait]
pub trait RateLimitStorage<T>: Send + Sync
where
    T: Clone + Send + Sync,
{
    //Get the bucket state for a key, returns None if don't exist
    async fn get(&self, key: &str) -> Option<T>;

    //set the bucket state for a key
    async fn set(&self, key: &str, state: T);
}
