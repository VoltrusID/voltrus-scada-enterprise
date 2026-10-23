//! Multi-site hub: connect to remote Voltrus sites and mirror their data.

pub mod connector;
pub mod handlers;

pub use handlers::HubManager;
