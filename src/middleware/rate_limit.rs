use crate::algorithms::TokenBucketLimiter;
use crate::storage::RedisStorage;
use crate::storage::{BucketState, MemoryStorage};
use crate::{RateLimitConfig, RateLimiter};
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::sync::Arc;
/// Shared state for the rate limiter middleware
#[derive(Clone)]
pub struct RateLimitState {
    pub limiter: Arc<TokenBucketLimiter<MemoryStorage<BucketState>>>,
}

impl RateLimitState {
    pub async fn new(config: RateLimitConfig) -> Self {
        let storage = MemoryStorage::new();
        let limiter = TokenBucketLimiter::new(config, storage);
        Self {
            limiter: Arc::new(limiter),
        }
    }
}
/// Rate limiting middleware function
/// This runs BEFORE your route handlers
pub async fn rate_limit_middleware(
    State(state): State<RateLimitState>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let client_id = extract_client_id(&request);
    // Step 2: Check rate limit
    let result = state.limiter.check(&client_id).await;
    // Step 3: Build rate limit headers
    let limit_header = result.remaining + if result.allowed { 1 } else { 0 };

    if result.allowed {
        // Step 4a: Request allowed - continue to route handler
        let mut response = next.run(request).await;

        // Add rate limit headers to response
        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            limit_header.to_string().parse().expect("valid header"),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            result.remaining.to_string().parse().expect("valid header"),
        );

        response
    } else {
        // Step 4b: Rate limited - return 429 immediately
        let mut response = Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(Body::from("Rate limit exceeded. Please try again later."))
            .expect("response build");
        // Add rate limit headers
        let headers = response.headers_mut();
        headers.insert("X-RateLimit-Limit", limit_header.to_string().parse().expect("valid header"));
        headers.insert("X-RateLimit-Remaining", "0".parse().expect("valid header"));
        headers.insert(
            "Retry-After",
            result
                .retry_after
                .as_secs()
                .to_string()
                .parse()
                .expect("valid header"),
        );
        response
    }
}
/// Extract a client identifier from the request
/// In production, use: client IP, API key header, or JWT token
  fn extract_client_id(request: &Request<Body>) -> String {
    // Try to get client IP from headers (for proxied requests)
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(ip) = forwarded.to_str() {
            return ip.split(',').next().unwrap_or("unknown").trim().to_string();
        }
    }

    // Fallback: use a default key (in production, get real IP)
    "default_client".to_string()
}
// Type alias for cleaner code
pub type RateLimitLayer = axum::middleware::FromFnLayer<
    fn(
        State<RateLimitState>,
        Request<Body>,
        Next,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response<Body>> + Send>>,
    RateLimitState,
    (State<RateLimitState>, Request<Body>, Next),
>;
//redis backend rate limit state
#[derive(Clone)]
pub struct RedisRateLimitState {
    pub limiter: Arc<TokenBucketLimiter<RedisStorage>>,
}

impl RedisRateLimitState {
    pub fn new(config: RateLimitConfig, redis_url: &str) -> Result<Self, redis::RedisError> {
        let storage = RedisStorage::new(redis_url, config.window.as_secs())?;
        let limiter = TokenBucketLimiter::new(config, storage);
        Ok(Self {
            limiter: Arc::new(limiter),
        })
    }
}

//redis middleware function
pub async fn redis_rate_limit_middleware(
    State(state): State<RedisRateLimitState>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let client_id = extract_client_id(&request);
    let result = state.limiter.check(&client_id).await;
    if result.allowed {
        let mut response = next.run(request).await;
        let headers = response.headers_mut();
        headers.insert("X-RateLimit-Limit", state.limiter.config.max_request.to_string().parse().expect("valid header"));
        headers.insert(
            "X-RateLimit-Remaining",
            result.remaining.to_string().parse().expect("valid header"),
        );
        response
    } else {
        let mut response = Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(Body::from("Rate limit exceeded."))
            .expect("response build ");
        let headers = response.headers_mut();
        headers.insert("X-RateLimit-Limit", state.limiter.config.max_request.to_string().parse().expect("valid header"));
        headers.insert("X-RateLimit-Remaining", result.remaining.to_string().parse().expect("valid header"));
        headers.insert(
            "Retry-After",
            result
                .retry_after
                .as_secs()
                .to_string()
                .parse()
                .expect("valid header"),
        );
        response
    }
}
