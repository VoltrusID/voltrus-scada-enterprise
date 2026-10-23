//! RTSP video: camera management, snapshot capture and stream proxy.

mod client;
pub mod handlers;
mod proxy;
mod types;

pub use client::*;
pub use proxy::*;
pub use types::*;

#[cfg(test)]
mod tests;
