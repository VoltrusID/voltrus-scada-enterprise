pub fn now_secs() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

pub fn now_millis() -> i64 {
    (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64
}
