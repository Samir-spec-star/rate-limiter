pub mod algorithms;
pub mod error;
pub mod middleware;
pub mod storage;
use std::time::Duration;
//configuration for a rate limiter
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    //max requests allowed
    pub max_request: u32,
    //time window
    pub window: Duration,
}
impl RateLimitConfig {
    pub fn new(max_request: u32, window: Duration) -> Self {
        Self {
            max_request,
            window,
        }
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    // whether the request is allowed
    pub allowed: bool,
    // Remaining Request in current window
    pub remaining: u32,
    // time until the bucket resets/refills
    pub retry_after: Duration,
}

//Core trait that all rate limiter must implement
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    //check if a request is allowed for the give key/api/userID..
    async fn check(&self, key: &str) -> RateLimitResult;
}
//re-export main types
pub use algorithms::SharedTokenBucket;
pub use algorithms::TokenBucket;
