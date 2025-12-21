use axum::{Router, middleware, routing::get};
use rate_limiter::RateLimitConfig;
use rate_limiter::middleware::{
    RateLimitState, RedisRateLimitState, rate_limit_middleware, redis_rate_limit_middleware,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Starting Rate Limited API Server...\n");
    let config = RateLimitConfig::new(5, Duration::from_secs(10));
    // Try Redis first, fall back to memory
    let app = match RedisRateLimitState::new(config.clone(), "redis://127.0.0.1:6379") {
        Ok(redis_state) => Router::new()
            .route("/", get(root_handler))
            .route("/api/data", get(api_data_handler))
            .layer(middleware::from_fn_with_state(
                redis_state.clone(),
                redis_rate_limit_middleware,
            ))
            .with_state(redis_state),
        Err(_) => {
            let state = RateLimitState::new(config).await;
            Router::new()
                .route("/", get(root_handler))
                .route("/api/data", get(api_data_handler))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    rate_limit_middleware,
                ))
                .with_state(state)
        }
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address. Is the port already in use?");
    println!("Server at http://127.0.0.1:3000\n");
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> &'static str {
    "Welcome to Rate Limited API!"
}
async fn api_data_handler() -> &'static str {
    r#"{"data": [1,2,3]}"#
}
