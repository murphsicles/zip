use tracing::info;

use crate::config::EnvConfig;
use crate::errors::ZipError;

pub struct Metrics {
    enabled: bool,
}

impl Metrics {
    /// Initializes metrics with optional telemetry based on environment config.
    pub fn new(config: &EnvConfig) -> Self {
        let enabled = config.log_level == "debug";
        Self { enabled }
    }

    /// Tracks an authentication event (e.g., login success/failure).
    pub fn track_auth_event(&self, user_id: &str, event: &str, success: bool) {
        if self.enabled {
            info!(
                event = event,
                user_id = user_id,
                success = success,
                "Authentication event"
            );
        }
    }

    /// Tracks a payment event (e.g., transaction success/failure).
    pub fn track_payment_event(&self, user_id: &str, tx_id: &str, amount: u64, success: bool) {
        if self.enabled {
            info!(
                event = "payment",
                user_id = user_id,
                tx_id = tx_id,
                amount = amount,
                success = success,
                "Payment event"
            );
        }
    }
}
