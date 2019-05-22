use std::time::{SystemTime, Duration};

pub fn duration_to_nano(duration: &Duration) -> u128 {
    let in_nanos = duration.as_secs() as u128 * 1000_000_000 +
            duration.subsec_nanos() as u128;
    in_nanos
}