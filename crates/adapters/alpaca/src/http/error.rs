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

//! Alpaca HTTP client error types.

use thiserror::Error;

/// Result type for Alpaca HTTP operations.
pub type AlpacaHttpResult<T> = Result<T, AlpacaHttpError>;

/// Alpaca HTTP client error types.
#[derive(Debug, Error)]
pub enum AlpacaHttpError {
    /// Missing API credentials.
    #[error("Missing API credentials: both api_key and api_secret are required")]
    MissingCredentials,

    /// HTTP client error.
    #[error("HTTP client error: {0}")]
    ClientError(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(String),

    /// API returned an error response.
    #[error("API error {code}: {message}")]
    ApiError {
        /// HTTP status code.
        code: u16,
        /// Error message from API.
        message: String,
    },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Request validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Connection error.
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Timeout error.
    #[error("Request timeout: {0}")]
    Timeout(String),
}

impl From<nautilus_network::http::HttpClientError> for AlpacaHttpError {
    fn from(err: nautilus_network::http::HttpClientError) -> Self {
        Self::ClientError(err.to_string())
    }
}

impl From<serde_json::Error> for AlpacaHttpError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err.to_string())
    }
}
