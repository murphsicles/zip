use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

use crate::errors::ZipError;

/// Generic in-memory cache with expiration.
pub struct Cache<K: std::hash::Hash + Eq + Clone, V: Clone> {
    data: Arc<RwLock<HashMap<K, (V, Instant)>>>,
    ttl: Duration,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> Cache<K, V> {
    /// Initializes cache with time-to-live (TTL) duration.
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Inserts a key-value pair into the cache with expiration.
    pub async fn insert(&self, key: K, value: V) {
        let mut cache = self.data.write().await;
        cache.insert(key, (value, Instant::now()));
    }

    /// Retrieves a value from the cache if not expired.
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.data.write().await;
        if let Some((value, timestamp)) = cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(value.clone());
            } else {
                cache.remove(key);
            }
        }
        None
    }

    /// Removes a key from the cache.
    pub async fn remove(&self, key: &K) {
        let mut cache = self.data.write().await;
        cache.remove(key);
    }

    /// Clears the entire cache.
    pub async fn clear(&self) {
        let mut cache = self.data.write().await;
        cache.clear();
    }
}
