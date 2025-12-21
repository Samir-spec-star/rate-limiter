use super::RateLimitStorage;
use dashmap::DashMap;
use std::sync::Arc;
/// In-memory storage using HashMap (same as before, but abstracted)
pub struct MemoryStorage<T> {
    data: Arc<DashMap<String, T>>,
}
impl<T> MemoryStorage<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }
}
#[async_trait::async_trait]
impl<T> RateLimitStorage<T> for MemoryStorage<T> 
where 
    T: Clone + Send + Sync, 

{
    async fn get(&self, key: &str) -> Option<T> {
        self.data.get(key).map(|entry| entry.value().clone())
    }

    async fn set(&self, key: &str, state: T) {
        self.data.insert(key.to_string(), state);
    }
}