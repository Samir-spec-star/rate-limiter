use crate::storage::{BucketState, RateLimitStorage};
use crate::{RateLimitConfig, RateLimitResult, RateLimiter};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// A Token bucket rate limiter
pub struct TokenBucket {
    ///Maximum tokens the bucket can hold
    capacity: f64,
    /// current number of tokens in the bucket
    tokens: f64,
    ///Tokens added per second
    refill_rate: f64,
    ///last time we refilled tokens
    last_refill: Instant,
}

impl TokenBucket {
    //capacity - Maximum tokens the bucket can hold
    //refill_rate - Tokens added per second

    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity: capacity as f64,
            tokens: capacity as f64, // Start with full bucket
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    ///Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        //calculate new tokens: elapsed time * refill rate
        let new_tokens = elapsed * self.refill_rate;

        //add tokens but don't exceed capacity
        self.tokens = (self.tokens + new_tokens).min(self.capacity.into());
        self.last_refill = now;
    }

    /// Try to acquire a token. Returns true if allowed, false if rate limited.
    pub fn try_acquire(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true // Request allowed
        } else {
            false // Rate limited
        }
    }

    //get remaining tokens
    pub fn remaining(&mut self) -> u32 {
        self.refill();
        self.tokens as u32
    }

    //Time until next token is available
    pub fn retry_after(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let needed = 1.0 - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }

    /// Create a thread-safe wrapper
    pub fn into_shared(self) -> SharedTokenBucket {
        SharedTokenBucket {
            inner: Arc::new(Mutex::new(self)),
        }
    }
}

// Thread-safe wrapper around TokenBucket
#[derive(Clone)]
pub struct SharedTokenBucket {
    inner: Arc<Mutex<TokenBucket>>,
}

impl SharedTokenBucket {
    // Create a new shared token bucket
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        TokenBucket::new(capacity, refill_rate).into_shared()
    }

    // Try to acquire a token (thread-safe)
    pub async fn try_acquire(&self) -> bool {
        // lock the mutex - blocks until we have exclusive access
        let mut bucket = self.inner.lock().unwrap(); 
        bucket.try_acquire()
    }
}
pub struct TokenBucketLimiter<S: RateLimitStorage<BucketState>> {
    storage: S,
    pub config: RateLimitConfig,
}
impl<S: RateLimitStorage<BucketState>> TokenBucketLimiter<S> {
    pub fn new(config: RateLimitConfig, storage: S) -> Self {
        Self { storage, config }
    }
    async fn get_or_create_bucket(&self, key: &str) -> (bool, u32, Duration) {
        use crate::storage::BucketState;

        // Get existing state or create new one
        let mut state = self.storage.get(key).await.unwrap_or_else(|| BucketState {
            tokens: self.config.max_request as f64,
            last_refill: Instant::now(),
        });
        // Refill logic
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill).as_secs_f64();
        let refill_rate = self.config.max_request as f64 / self.config.window.as_secs_f64();
        let new_tokens = elapsed * refill_rate;
        state.tokens = (state.tokens + new_tokens).min(self.config.max_request as f64);
        state.last_refill = now;
        // Try to acquire
        let allowed = if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            true
        } else {
            false
        };
        let remaining = state.tokens as u32;
        let retry_after = if state.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let needed = 1.0 - state.tokens;
            Duration::from_secs_f64(needed / refill_rate)
        };
        // Save state back
        self.storage.set(key, state).await;
        (allowed, remaining, retry_after)
    }
}
#[async_trait::async_trait]
impl<S: RateLimitStorage<BucketState>> RateLimiter for TokenBucketLimiter<S> {
    async fn check(&self, key: &str) -> RateLimitResult {
        let (allowed, remaining, retry_after) = self.get_or_create_bucket(key).await;
        RateLimitResult {
            allowed,
            remaining,
            retry_after,
        }
    }
}

