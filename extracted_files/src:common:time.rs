// Shared time utilities
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn timestamp_to_string(timestamp: u64) -> String {
    let datetime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
    format!("{:?}", datetime)
}

pub fn time_since(timestamp: u64) -> u64 {
    current_timestamp().saturating_sub(timestamp)
}
