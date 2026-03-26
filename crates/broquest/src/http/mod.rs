//! HTTP client service module

mod client;
pub mod jwt;
pub mod oauth2;

#[cfg(test)]
mod auth_tests;

pub use client::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current Unix timestamp in seconds
pub fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Check if a token is expired based on its expiration timestamp
/// Returns true if the token has no value or expires within 60 seconds
pub fn is_token_expired(expires_at: Option<i64>, has_token: bool) -> bool {
    match expires_at {
        Some(expires_at) => current_unix_timestamp() >= expires_at - 60,
        None => !has_token,
    }
}
