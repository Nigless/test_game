use std::time::{SystemTime, UNIX_EPOCH};

pub fn date_now() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
}
