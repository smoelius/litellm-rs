//! JWT utility functions

use super::types::{Claims, JwtHandler};
use std::time::{SystemTime, UNIX_EPOCH};

impl JwtHandler {
    /// Extract token from Authorization header
    pub fn extract_token_from_header(header_value: &str) -> Option<String> {
        header_value
            .strip_prefix("Bearer ")
            .map(|token| token.to_string())
    }

    /// Get token expiration time
    pub fn get_expiration(&self) -> u64 {
        self.expiration
    }

    /// Check if token is expired
    pub fn is_token_expired(&self, claims: &Claims) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX); // If system time is invalid, treat as expired

        claims.exp < now
    }

    /// Get time until token expires
    pub fn time_until_expiry(&self, claims: &Claims) -> Option<u64> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

        if claims.exp > now {
            Some(claims.exp - now)
        } else {
            None
        }
    }
}
