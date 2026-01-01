// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Alpaca API credential handling.

use std::fmt;

/// Alpaca API credentials for authentication.
///
/// Alpaca uses a simple API key/secret authentication scheme via HTTP headers:
/// - `APCA-API-KEY-ID`: The API key
/// - `APCA-API-SECRET-KEY`: The API secret
#[derive(Clone)]
pub struct AlpacaCredential {
    api_key: String,
    api_secret: String,
}

impl AlpacaCredential {
    /// Creates a new `AlpacaCredential` with the given API key and secret.
    #[must_use]
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
        }
    }

    /// Returns the API key.
    #[must_use]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the API secret.
    #[must_use]
    pub fn api_secret(&self) -> &str {
        &self.api_secret
    }

    /// Returns the header name for the API key.
    #[must_use]
    pub const fn api_key_header() -> &'static str {
        "APCA-API-KEY-ID"
    }

    /// Returns the header name for the API secret.
    #[must_use]
    pub const fn api_secret_header() -> &'static str {
        "APCA-API-SECRET-KEY"
    }
}

impl fmt::Debug for AlpacaCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlpacaCredential")
            .field("api_key", &"[REDACTED]")
            .field("api_secret", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_creation() {
        let cred = AlpacaCredential::new("test_key", "test_secret");
        assert_eq!(cred.api_key(), "test_key");
        assert_eq!(cred.api_secret(), "test_secret");
    }

    #[test]
    fn test_header_names() {
        assert_eq!(AlpacaCredential::api_key_header(), "APCA-API-KEY-ID");
        assert_eq!(AlpacaCredential::api_secret_header(), "APCA-API-SECRET-KEY");
    }

    #[test]
    fn test_debug_redaction() {
        let cred = AlpacaCredential::new("secret_key", "secret_value");
        let debug_str = format!("{cred:?}");
        assert!(!debug_str.contains("secret_key"));
        assert!(!debug_str.contains("secret_value"));
        assert!(debug_str.contains("[REDACTED]"));
    }
}
