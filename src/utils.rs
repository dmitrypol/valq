use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn now_as_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
