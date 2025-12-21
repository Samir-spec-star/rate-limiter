#[derive(Debug, thiserror::Error)]
pub enum RateLimitError { 
    //redis connection or operation error
    #[error("Redis connection error: {0}")]
    RedisError(#[from] redis::RedisError),
    //storage operation failed (mutex poisoned)
    #[error("Storage lock failed: {0}")]
    StorageLock(String),
    //config error
    #[error("Invaild configuration: {0}")]
    ConfigError(String),
}

// result type alias for convenience
pub type Result<T> = std::result::Result<T, RateLimitError>;

