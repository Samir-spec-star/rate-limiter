use super::{BucketState, RateLimitStorage};
use redis::{Client, Connection};
use std::sync::{Arc, Mutex};
use std::time::Instant;
/// Redis-backed storage for distributed rate limiting
pub struct RedisStorage {
    connection: Arc<Mutex<Connection>>,
    window_secs: u64,
}
impl RedisStorage {
    /// Create new Redis storage
    /// url: "redis://127.0.0.1:6379"
    pub fn new(url: &str, window_secs: u64) -> Result<Self, redis::RedisError> {
        let client = Client::open(url)?;
        let connection = client.get_connection()?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            window_secs,
        })
    }

    fn build_key(&self, key: &str) -> String {
        format!("ratelimit:{}", key)
    }
}
#[async_trait::async_trait]
impl RateLimitStorage<BucketState> for RedisStorage {
    async fn get(&self, key: &str) -> Option<BucketState> {
        let redis_key = self.build_key(key);
        let Ok(mut conn) = self.connection.lock() else {
            return None;
        };

        // Get tokens and timestamp from Redis
        let result: Result<(Option<f64>, Option<i64>), _> = redis::pipe()
            .get(format!("{}:tokens", redis_key))
            .get(format!("{}:last_refill", redis_key))
            .query(&mut *conn);

        match result {
            Ok((Some(tokens), Some(last_refill_ms))) => {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("System before unix epoch")
                    .as_millis() as i64;

                let elapsed_ms = (now_ms - last_refill_ms).max(0) as u64;
                let last_refill = Instant::now() - std::time::Duration::from_millis(elapsed_ms);

                Some(BucketState {
                    tokens,
                    last_refill,
                })
            }
            _ => None,
        }
    }
    async fn set(&self, key: &str, state: BucketState) {
        let redis_key = self.build_key(key);
        let Ok(mut conn) = self.connection.lock() else {
            return;
        };

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System before unix epoch")
            .as_millis() as i64;

        let elapsed = Instant::now().duration_since(state.last_refill);
        let last_refill_ms = now_ms - elapsed.as_millis() as i64;

        let _: Result<(), _> = redis::pipe()
            .set_ex(
                format!("{}:tokens", redis_key),
                state.tokens,
                self.window_secs,
            )
            .set_ex(
                format!("{}:last_refill", redis_key),
                last_refill_ms,
                self.window_secs,
            )
            .query(&mut *conn);
    }
}
