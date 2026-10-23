//! Licence activation, verification and renewal for the official build.
//!
//! A licence is an Ed25519-signed activation file bound to this machine's
//! fingerprint. Verification is fully offline. Only a valid licence unlocks
//! add-on features; any other state runs as Community and never blocks the core.

pub mod activation;
pub mod cli;
pub mod fingerprint;
pub mod online;
pub mod request;
pub mod routes;
pub mod verification;

#[cfg(test)]
mod tests;

pub use verification::{GRACE_DAYS, resolve_license_status};
