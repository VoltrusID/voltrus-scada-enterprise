mod client;
mod poller;
#[cfg(test)]
mod tests;
mod types;

pub use client::*;
pub use poller::*;

// Surface the S7 value/tag types so the gateway client glue can name them
// (e.g. `crate::s7::S7Tag`) when S7 is routed through the gateway. Targeted
// re-export avoids glob collisions with the client/poller globs above.
#[cfg(feature = "gateway-s7")]
pub use types::{S7Area, S7DataType, S7Tag};

// Re-exported for test modules using `use super::*`
#[cfg(test)]
pub use types::*;
