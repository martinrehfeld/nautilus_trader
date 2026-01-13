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

//! Alpaca HTTP client implementation.

use std::{collections::HashMap, num::NonZeroU32};

use nautilus_network::{
    http::{HttpClient, HttpResponse, Method},
    ratelimiter::quota::Quota,
};
use serde::{Serialize, de::DeserializeOwned};

use super::error::{AlpacaHttpError, AlpacaHttpResult};
use crate::common::{
    credential::AlpacaCredential,
    enums::AlpacaEnvironment,
    urls::{get_data_api_url, get_http_base_url},
};

const ALPACA_RATE_LIMIT_KEY: &str = "alpaca:trading";

/// Alpaca HTTP client for REST API access.
///
/// Handles:
/// - Base URL resolution by environment (live/paper).
/// - API key authentication via headers.
/// - Rate limiting (200 requests per minute for trading API).
/// - Error deserialization for Alpaca error payloads.
#[derive(Debug, Clone)]
pub struct AlpacaHttpClient {
    client: HttpClient,
    trading_base_url: String,
    data_base_url: String,
    credential: AlpacaCredential,
}

impl AlpacaHttpClient {
    /// Creates a new Alpaca HTTP client.
    ///
    /// # Arguments
    ///
    /// * `environment` - Trading environment (Live or Paper).
    /// * `api_key` - Alpaca API key.
    /// * `api_secret` - Alpaca API secret.
    /// * `timeout_secs` - Optional request timeout in seconds.
    /// * `proxy_url` - Optional proxy URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying [`HttpClient`] fails to build.
    ///
    /// # Panics
    ///
    /// Panics if `NonZeroU32::new(200)` returns `None` (which should never happen).
    pub fn new(
        environment: AlpacaEnvironment,
        api_key: String,
        api_secret: String,
        timeout_secs: Option<u64>,
        proxy_url: Option<String>,
    ) -> AlpacaHttpResult<Self> {
        let credential = AlpacaCredential::new(api_key, api_secret);
        let trading_base_url = get_http_base_url(environment).to_string();
        let data_base_url = get_data_api_url().to_string();

        // Default headers with authentication
        let headers = Self::build_headers_map(&credential);

        // Rate limiting: 200 requests per minute for trading API
        let default_quota = Some(Quota::per_minute(NonZeroU32::new(200).expect("Non-zero")));

        let keyed_quotas = vec![(
            ALPACA_RATE_LIMIT_KEY.to_string(),
            Quota::per_minute(NonZeroU32::new(200).expect("Non-zero")),
        )];

        let client = HttpClient::new(
            headers,
            vec![
                AlpacaCredential::api_key_header().to_string(),
                AlpacaCredential::api_secret_header().to_string(),
            ],
            keyed_quotas,
            default_quota,
            timeout_secs,
            proxy_url,
        )?;

        Ok(Self {
            client,
            trading_base_url,
            data_base_url,
            credential,
        })
    }

    /// Returns the trading API base URL.
    #[must_use]
    pub fn trading_base_url(&self) -> &str {
        &self.trading_base_url
    }

    /// Returns the market data API base URL.
    #[must_use]
    pub fn data_base_url(&self) -> &str {
        &self.data_base_url
    }

    /// Performs a GET request to the trading API.
    pub async fn get<P, T>(&self, path: &str, params: Option<&P>) -> AlpacaHttpResult<T>
    where
        P: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.request(Method::GET, &self.trading_base_url, path, params, None)
            .await
    }

    /// Performs a POST request to the trading API.
    pub async fn post<P, T>(
        &self,
        path: &str,
        params: Option<&P>,
        body: Option<Vec<u8>>,
    ) -> AlpacaHttpResult<T>
    where
        P: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.request(Method::POST, &self.trading_base_url, path, params, body)
            .await
    }

    /// Performs a DELETE request to the trading API.
    pub async fn delete<P, T>(&self, path: &str, params: Option<&P>) -> AlpacaHttpResult<T>
    where
        P: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.request(Method::DELETE, &self.trading_base_url, path, params, None)
            .await
    }

    /// Performs a PATCH request to the trading API.
    pub async fn patch<P, T>(
        &self,
        path: &str,
        params: Option<&P>,
        body: Option<Vec<u8>>,
    ) -> AlpacaHttpResult<T>
    where
        P: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.request(Method::PATCH, &self.trading_base_url, path, params, body)
            .await
    }

    /// Performs a GET request to the market data API.
    pub async fn get_data<P, T>(&self, path: &str, params: Option<&P>) -> AlpacaHttpResult<T>
    where
        P: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.request(Method::GET, &self.data_base_url, path, params, None)
            .await
    }

    async fn request<P, T>(
        &self,
        method: Method,
        base_url: &str,
        path: &str,
        params: Option<&P>,
        body: Option<Vec<u8>>,
    ) -> AlpacaHttpResult<T>
    where
        P: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let query = params
            .map(serde_urlencoded::to_string)
            .transpose()
            .map_err(|e| AlpacaHttpError::ValidationError(e.to_string()))?
            .unwrap_or_default();

        let url = self.build_url(base_url, path, &query);

        // Add auth headers to request
        let mut headers = HashMap::new();
        headers.insert(
            AlpacaCredential::api_key_header().to_string(),
            self.credential.api_key().to_string(),
        );
        headers.insert(
            AlpacaCredential::api_secret_header().to_string(),
            self.credential.api_secret().to_string(),
        );

        let keys = vec![ALPACA_RATE_LIMIT_KEY.to_string()];

        let response = self
            .client
            .request(
                method,
                url,
                None::<&HashMap<String, Vec<String>>>,
                Some(headers),
                body,
                None,
                Some(keys),
            )
            .await?;

        if !response.status.is_success() {
            return self.parse_error_response(response);
        }

        serde_json::from_slice::<T>(&response.body)
            .map_err(|e| AlpacaHttpError::JsonError(e.to_string()))
    }

    fn build_url(&self, base_url: &str, path: &str, query: &str) -> String {
        let normalized_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };

        let mut url = format!("{base_url}{normalized_path}");
        if !query.is_empty() {
            url.push('?');
            url.push_str(query);
        }
        url
    }

    fn build_headers_map(credential: &AlpacaCredential) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(
            AlpacaCredential::api_key_header().to_string(),
            credential.api_key().to_string(),
        );
        headers.insert(
            AlpacaCredential::api_secret_header().to_string(),
            credential.api_secret().to_string(),
        );
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers
    }

    fn parse_error_response<T>(&self, response: HttpResponse) -> AlpacaHttpResult<T> {
        let code = response.status.as_u16();

        // Try to parse Alpaca error response
        #[derive(serde::Deserialize)]
        struct AlpacaErrorResponse {
            message: Option<String>,
            #[allow(dead_code)]
            code: Option<i64>,
        }

        let message = serde_json::from_slice::<AlpacaErrorResponse>(&response.body)
            .ok()
            .and_then(|e| e.message)
            .unwrap_or_else(|| String::from_utf8_lossy(&response.body).to_string());

        // Check for specific error types
        match code {
            401 | 403 => Err(AlpacaHttpError::AuthenticationError(message)),
            429 => Err(AlpacaHttpError::RateLimitExceeded(message)),
            _ => Err(AlpacaHttpError::ApiError { code, message }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_with_query() {
        let client = AlpacaHttpClient::new(
            AlpacaEnvironment::Paper,
            "test_key".to_string(),
            "test_secret".to_string(),
            None,
            None,
        )
        .unwrap();

        let url = client.build_url("https://api.example.com", "/v2/account", "");
        assert_eq!(url, "https://api.example.com/v2/account");

        let url = client.build_url("https://api.example.com", "v2/orders", "status=open");
        assert_eq!(url, "https://api.example.com/v2/orders?status=open");
    }

    #[test]
    fn test_build_headers() {
        let credential = AlpacaCredential::new("my_key", "my_secret");
        let headers = AlpacaHttpClient::build_headers_map(&credential);

        assert_eq!(headers.get("APCA-API-KEY-ID"), Some(&"my_key".to_string()));
        assert_eq!(
            headers.get("APCA-API-SECRET-KEY"),
            Some(&"my_secret".to_string())
        );
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_base_urls() {
        let client = AlpacaHttpClient::new(
            AlpacaEnvironment::Paper,
            "key".to_string(),
            "secret".to_string(),
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            client.trading_base_url(),
            "https://paper-api.alpaca.markets"
        );
        assert_eq!(client.data_base_url(), "https://data.alpaca.markets");
    }
}
