use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use crate::errors::ZipError;

/// Rate limiter for controlling request frequency.
#[derive(Clone)]
pub struct RateLimiter {
    limits: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    /// Initializes rate limiter with max requests per window (seconds).
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Checks if a key (e.g., user_id) can make a request.
    pub async fn check(&self, key: &str) -> Result<(), ZipError> {
        let mut limits = self.limits.lock().await;
        let now = Instant::now();
        let (count, timestamp) = limits.entry(key.to_string()).or_insert((0, now));
        if now.duration_since(*timestamp) > self.window {
            *count = 0;
            *timestamp = now;
        }
        if *count >= self.max_requests {
            return Err(ZipError::RateLimit(format!(
                "Rate limit exceeded for {}. Try again later.",
                key
            )));
        }
        *count += 1;
        Ok(())
    }

    /// Clears rate limit for a key.
    pub async fn clear(&self, key: &str) {
        let mut limits = self.limits.lock().await;
        limits.remove(key);
    }

    /// Resets all rate limits.
    pub async fn reset(&self) {
        let mut limits = self.limits.lock().await;
        limits.clear();
    }
}
