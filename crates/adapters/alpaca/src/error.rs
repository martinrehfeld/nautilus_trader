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

//! Alpaca adapter error types.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Alpaca API error response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaApiError {
    /// Error code from Alpaca API.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
}

impl fmt::Display for AlpacaApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Alpaca API Error {}: {}", self.code, self.message)
    }
}

/// Alpaca adapter error types.
#[derive(Debug, Error)]
pub enum AlpacaError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(String),

    /// WebSocket connection error.
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// API returned an error response.
    #[error("API error: {0}")]
    Api(AlpacaApiError),

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Invalid request parameters.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Timeout error.
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Unknown or unexpected error.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl AlpacaError {
    /// Creates an HTTP error.
    #[must_use]
    pub fn http(msg: impl Into<String>) -> Self {
        Self::Http(msg.into())
    }

    /// Creates a WebSocket error.
    #[must_use]
    pub fn websocket(msg: impl Into<String>) -> Self {
        Self::WebSocket(msg.into())
    }

    /// Creates an API error from an error response.
    #[must_use]
    pub fn api(code: i32, message: impl Into<String>) -> Self {
        Self::Api(AlpacaApiError {
            code,
            message: message.into(),
        })
    }

    /// Creates an authentication error.
    #[must_use]
    pub fn authentication(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }

    /// Creates a rate limit error.
    #[must_use]
    pub fn rate_limit(msg: impl Into<String>) -> Self {
        Self::RateLimit(msg.into())
    }

    /// Creates an invalid request error.
    #[must_use]
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest(msg.into())
    }

    /// Creates a serialization error.
    #[must_use]
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Creates a connection error.
    #[must_use]
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Creates a timeout error.
    #[must_use]
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let error = AlpacaApiError {
            code: 40010001,
            message: "Invalid symbol".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Alpaca API Error 40010001: Invalid symbol"
        );
    }

    #[test]
    fn test_error_creation() {
        let error = AlpacaError::api(400, "Bad request");
        assert!(matches!(error, AlpacaError::Api(_)));
        assert!(error.to_string().contains("Bad request"));
    }

    #[test]
    fn test_http_error() {
        let error = AlpacaError::http("Connection refused");
        assert!(matches!(error, AlpacaError::Http(_)));
    }
}
