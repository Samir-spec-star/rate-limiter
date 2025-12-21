use crate::storage::{RateLimitStorage, WindowState};
use crate::{RateLimitConfig, RateLimitResult, RateLimiter};
use std::time::{Duration, Instant};
/// Sliding Window Counter rate limiter
/// Provides smoother rate limiting than fixed windows
pub struct SlidingWindowLimiter<S: RateLimitStorage<WindowState>> {
    storage: S,
    pub config: RateLimitConfig,
}
impl<S: RateLimitStorage<WindowState>> SlidingWindowLimiter<S> {
    pub fn new(config: RateLimitConfig, storage: S) -> Self {
        Self { storage, config }
    }
    async fn check_and_update(&self, key: &str) -> (bool, u32, Duration) {
        let now = Instant::now();
        let window_size = self.config.window;
        let max_requests = self.config.max_request;
        // Step A: Get existing state or create fresh one
        let mut state = self.storage.get(key).await.unwrap_or_else(|| WindowState {
            prev_count: 0,
            curr_count: 0,
            window_start: now,
        });
        // Step B: Check if window needs to slide
        let elapsed = now.duration_since(state.window_start);
        if elapsed >= window_size {
            // Window expired - need to slide
            let windows_passed = elapsed.as_secs_f64() / window_size.as_secs_f64();
            if windows_passed >= 2.0 {
                // More than 2 windows passed - reset everything
                state.prev_count = 0;
                state.curr_count = 0;
            } else {
                // Exactly 1 window passed - slide: current becomes previous
                state.prev_count = state.curr_count;
                state.curr_count = 0;
            }
            state.window_start = now;
        }
        // Step C: Calculate weight of previous window
        // Weight is 1.0 at start of window, 0.0 at end
        let elapsed_in_current = now.duration_since(state.window_start).as_secs_f64();
        let weight = 1.0 - (elapsed_in_current / window_size.as_secs_f64());
        // Step D: Calculate effective count using sliding window formula
        let effective_count = (state.prev_count as f64 * weight) + state.curr_count as f64;
        // Step E: Check if request is allowed
        let allowed = effective_count < max_requests as f64;
        if allowed {
            state.curr_count += 1;
        }
        // Step F: Calculate remaining requests
        let remaining = if allowed {
            ((max_requests as f64 - effective_count - 1.0).max(0.0)) as u32
        } else {
            0
        };
        // Step G: Calculate retry_after time
        let retry_after = if allowed {
            Duration::ZERO
        } else {
            // Estimate time until weight drops enough
            Duration::from_secs_f64(window_size.as_secs_f64() * 0.1)
        };
        // Step H: Save state back to storage
        let _ = self.storage.set(key, state);
        (allowed, remaining, retry_after)
    }
}
#[async_trait::async_trait]
impl<S: RateLimitStorage<WindowState>> RateLimiter for SlidingWindowLimiter<S> {
    async fn check(&self, key: &str) -> RateLimitResult {
        let (allowed, remaining, retry_after) = self.check_and_update(key).await;
        RateLimitResult {
            allowed,
            remaining,
            retry_after,
        }
    }
}
