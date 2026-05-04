// Readback state machine — stub for compilation
// Full implementation in Task 5

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadbackState {
    Pending,
    Sent,
    Polling,
    Verified,
    Timeout,
    Mismatch,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct ReadbackTracker {
    pub state: ReadbackState,
    pub retries_left: u8,
    pub timeout: Duration,
}
