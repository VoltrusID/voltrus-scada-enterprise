mod crypto;
mod token;
mod handlers;
mod users;
mod sso;

#[cfg(test)]
mod tests;

pub use crypto::*;
pub use token::*;
pub use handlers::*;
pub use users::*;
pub use sso::*;

use crate::models::*;

use super::{AppError, AppState};

const ACCESS_TOKEN_EXPIRY_MS: u64 = 15 * 60 * 1000; // 15 minutes
const REFRESH_TOKEN_EXPIRY_SECS: i64 = 7 * 24 * 60 * 60; // 7 days
