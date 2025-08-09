use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

use crate::errors::ZipError;

#[derive(Serialize, Deserialize)]
pub struct PaymailConfig {
    pub domain: String,
    pub excluded_prefixes: HashMap<String, Decimal>,
}

impl PaymailConfig {
    /// Loads PayMail configuration with excluded prefixes and prices.
    pub fn load() -> Self {
        let mut excluded_prefixes = HashMap::new();
        // 3-digit prefixes (alphabetically sorted)
        excluded_prefixes.insert("ace".to_string(), Decimal::from(1000));
        excluded_prefixes.insert("blue".to_string(), Decimal::from(700));
        excluded_prefixes.insert("bob".to_string(), Decimal::from(1000));
        excluded_prefixes.insert("cat".to_string(), Decimal::from(800));
        excluded_prefixes.insert("cool".to_string(), Decimal::from(2500));
        excluded_prefixes.insert("dog".to_string(), Decimal::from(900));
        excluded_prefixes.insert("fox".to_string(), Decimal::from(900));
        excluded_prefixes.insert("fun".to_string(), Decimal::from(1800));
        excluded_prefixes.insert("gold".to_string(), Decimal::from(4000));
        excluded_prefixes.insert("hot".to_string(), Decimal::from(3000));
        excluded_prefixes.insert("joe".to_string(), Decimal::from(800));
        excluded_prefixes.insert("max".to_string(), Decimal::from(1200));
        excluded_prefixes.insert("moon".to_string(), Decimal::from(600));
        excluded_prefixes.insert("neo".to_string(), Decimal::from(1500));
        excluded_prefixes.insert("pro".to_string(), Decimal::from(1500));
        excluded_prefixes.insert("red".to_string(), Decimal::from(600));
        excluded_prefixes.insert("roy".to_string(), Decimal::from(700));
        excluded_prefixes.insert("sam".to_string(), Decimal::from(900));
        excluded_prefixes.insert("sex".to_string(), Decimal::from(6000));
        excluded_prefixes.insert("sky".to_string(), Decimal::from(700));
        excluded_prefixes.insert("star".to_string(), Decimal::from(800));
        excluded_prefixes.insert("sun".to_string(), Decimal::from(500));
        excluded_prefixes.insert("top".to_string(), Decimal::from(2000));
        excluded_prefixes.insert("vip".to_string(), Decimal::from(5000));
        excluded_prefixes.insert("zen".to_string(), Decimal::from(1200));
        // 4-digit prefixes (alphabetically sorted)
        excluded_prefixes.insert("anna".to_string(), Decimal::from(350));
        excluded_prefixes.insert("bank".to_string(), Decimal::from(3500));
        excluded_prefixes.insert("best".to_string(), Decimal::from(2200));
        excluded_prefixes.insert("blog".to_string(), Decimal::from(700));
        excluded_prefixes.insert("boss".to_string(), Decimal::from(1800));
        excluded_prefixes.insert("cash".to_string(), Decimal::from(4000));
        excluded_prefixes.insert("deal".to_string(), Decimal::from(500));
        excluded_prefixes.insert("easy".to_string(), Decimal::from(1400));
        excluded_prefixes.insert("fast".to_string(), Decimal::from(1600));
        excluded_prefixes.insert("free".to_string(), Decimal::from(2800));
        excluded_prefixes.insert("game".to_string(), Decimal::from(900));
        excluded_prefixes.insert("guru".to_string(), Decimal::from(1200));
        excluded_prefixes.insert("hero".to_string(), Decimal::from(1500));
        excluded_prefixes.insert("jane".to_string(), Decimal::from(450));
        excluded_prefixes.insert("john".to_string(), Decimal::from(300));
        excluded_prefixes.insert("king".to_string(), Decimal::from(2000));
        excluded_prefixes.insert("love".to_string(), Decimal::from(3000));
        excluded_prefixes.insert("mark".to_string(), Decimal::from(600));
        excluded_prefixes.insert("mary".to_string(), Decimal::from(400));
        excluded_prefixes.insert("news".to_string(), Decimal::from(800));
        excluded_prefixes.insert("paul".to_string(), Decimal::from(500));
        excluded_prefixes.insert("rich".to_string(), Decimal::from(2500));
        excluded_prefixes.insert("shop".to_string(), Decimal::from(600));
        excluded_prefixes.insert("tech".to_string(), Decimal::from(1000));

        Self {
            domain: env::var("PAYMAIL_DOMAIN")
                .map_err(|_| ZipError::Blockchain("Missing PAYMAIL_DOMAIN".to_string()))
                .unwrap_or("zip.io".to_string()),
            excluded_prefixes,
        }
    }

    /// Determines price for a PayMail prefix based on length and exclusion list.
    pub fn get_prefix_price(&self, prefix: &str, is_first: bool) -> Decimal {
        if is_first && prefix == "101" {
            return Decimal::ZERO;
        }
        if let Some(price) = self.excluded_prefixes.get(prefix) {
            return *price;
        }
        match prefix.len() {
            3 => Decimal::from(250),
            4 => Decimal::from(25),
            _ => Decimal::from(10),
        }
    }

    /// Validates a PayMail prefix.
    pub fn validate_prefix(&self, prefix: &str) -> Result<(), ZipError> {
        if prefix.is_empty() || prefix.contains('@') || prefix.contains('.') {
            return Err(ZipError::Blockchain("Invalid PayMail prefix".to_string()));
        }
        if prefix.len() < 3 {
            return Err(ZipError::Blockchain("Prefix too short".to_string()));
        }
        Ok(())
    }
}
