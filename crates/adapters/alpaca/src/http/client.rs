// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2026 Andrew Crum. All rights reserved.
//  https://github.com/agcrum
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

#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::{
    error::{AlpacaHttpError, AlpacaHttpResult},
    models::*,
};
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
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
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

    // ============================================================================
    // Trading API Endpoints
    // ============================================================================

    /// Get account details.
    ///
    /// Retrieves the current account information including buying power,
    /// equity, cash, and account status.
    pub async fn get_account(&self) -> AlpacaHttpResult<AlpacaAccount> {
        self.get::<HashMap<String, String>, AlpacaAccount>("/v2/account", None)
            .await
    }

    /// Get all positions.
    ///
    /// Retrieves all open positions for the account.
    pub async fn get_positions(&self) -> AlpacaHttpResult<Vec<AlpacaPosition>> {
        self.get::<HashMap<String, String>, Vec<AlpacaPosition>>("/v2/positions", None)
            .await
    }

    /// Get a specific position by symbol or asset ID.
    ///
    /// # Arguments
    ///
    /// * `symbol_or_asset_id` - The symbol (e.g., "AAPL") or asset ID
    pub async fn get_position(&self, symbol_or_asset_id: &str) -> AlpacaHttpResult<AlpacaPosition> {
        let path = format!("/v2/positions/{symbol_or_asset_id}");
        self.get::<HashMap<String, String>, AlpacaPosition>(&path, None)
            .await
    }

    /// Close a position.
    ///
    /// # Arguments
    ///
    /// * `symbol_or_asset_id` - The symbol or asset ID
    /// * `qty` - Optional quantity to close (None closes entire position)
    /// * `percentage` - Optional percentage to close
    pub async fn close_position(
        &self,
        symbol_or_asset_id: &str,
        qty: Option<&str>,
        percentage: Option<&str>,
    ) -> AlpacaHttpResult<AlpacaOrder> {
        let path = format!("/v2/positions/{symbol_or_asset_id}");
        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(q) = qty {
            params.insert("qty".to_string(), q.to_string());
        }
        if let Some(p) = percentage {
            params.insert("percentage".to_string(), p.to_string());
        }
        self.delete::<HashMap<String, String>, AlpacaOrder>(&path, Some(&params))
            .await
    }

    /// Close all positions.
    ///
    /// # Arguments
    ///
    /// * `cancel_orders` - Whether to cancel all open orders before closing positions
    pub async fn close_all_positions(
        &self,
        cancel_orders: bool,
    ) -> AlpacaHttpResult<Vec<AlpacaOrder>> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("cancel_orders".to_string(), cancel_orders.to_string());
        self.delete::<HashMap<String, String>, Vec<AlpacaOrder>>("/v2/positions", Some(&params))
            .await
    }

    /// Submit a new order.
    ///
    /// # Arguments
    ///
    /// * `request` - Order request details
    pub async fn submit_order(&self, request: &AlpacaOrderRequest) -> AlpacaHttpResult<AlpacaOrder> {
        let body = serde_json::to_vec(request)
            .map_err(|e| AlpacaHttpError::JsonError(e.to_string()))?;
        self.post::<HashMap<String, String>, AlpacaOrder>("/v2/orders", None, Some(body))
            .await
    }

    /// Get all orders.
    ///
    /// # Arguments
    ///
    /// * `status` - Filter by order status (e.g., "open", "closed", "all")
    /// * `limit` - Maximum number of orders to return
    /// * `after` - Return orders after this timestamp
    /// * `until` - Return orders until this timestamp
    /// * `nested` - If true, roll up multi-leg orders
    pub async fn get_orders(
        &self,
        status: Option<&str>,
        limit: Option<u32>,
        after: Option<&str>,
        until: Option<&str>,
        nested: Option<bool>,
    ) -> AlpacaHttpResult<Vec<AlpacaOrder>> {
        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(s) = status {
            params.insert("status".to_string(), s.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        if let Some(a) = after {
            params.insert("after".to_string(), a.to_string());
        }
        if let Some(u) = until {
            params.insert("until".to_string(), u.to_string());
        }
        if let Some(n) = nested {
            params.insert("nested".to_string(), n.to_string());
        }
        self.get::<HashMap<String, String>, Vec<AlpacaOrder>>("/v2/orders", Some(&params))
            .await
    }

    /// Get a specific order by ID.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID
    /// * `nested` - If true, roll up multi-leg orders
    pub async fn get_order(&self, order_id: &str, nested: Option<bool>) -> AlpacaHttpResult<AlpacaOrder> {
        let path = format!("/v2/orders/{order_id}");
        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(n) = nested {
            params.insert("nested".to_string(), n.to_string());
        }
        self.get::<HashMap<String, String>, AlpacaOrder>(&path, Some(&params))
            .await
    }

    /// Cancel an order by ID.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to cancel
    pub async fn cancel_order(&self, order_id: &str) -> AlpacaHttpResult<()> {
        let path = format!("/v2/orders/{order_id}");
        self.delete::<HashMap<String, String>, ()>(&path, None)
            .await
    }

    /// Cancel all open orders.
    pub async fn cancel_all_orders(&self) -> AlpacaHttpResult<Vec<AlpacaCancelStatus>> {
        self.delete::<HashMap<String, String>, Vec<AlpacaCancelStatus>>("/v2/orders", None)
            .await
    }

    /// Cancel an order by client order ID.
    pub async fn cancel_order_by_client_id(&self, client_order_id: &str) -> AlpacaHttpResult<()> {
        let path = format!("/v2/orders:by_client_order_id?client_order_id={client_order_id}");
        self.delete::<HashMap<String, String>, ()>(&path, None)
            .await
    }

    /// Modify an existing order.
    pub async fn modify_order(
        &self,
        order_id: &str,
        body: &serde_json::Map<String, serde_json::Value>,
    ) -> AlpacaHttpResult<AlpacaOrder> {
        let path = format!("/v2/orders/{order_id}");
        let body_bytes = serde_json::to_vec(body)
            .map_err(|e| AlpacaHttpError::JsonError(e.to_string()))?;
        self.patch::<HashMap<String, String>, AlpacaOrder>(&path, None, Some(body_bytes))
            .await
    }

    /// Get account activities (alias for get_activities).
    pub async fn get_account_activities(
        &self,
        activity_types: Option<&str>,
        after: Option<&str>,
        until: Option<&str>,
        _direction: Option<&str>,
        page_size: Option<u32>,
    ) -> AlpacaHttpResult<Vec<AlpacaActivity>> {
        self.get_activities(activity_types, after, until, page_size)
            .await
    }

    /// Get account activities.
    ///
    /// # Arguments
    ///
    /// * `activity_types` - Optional comma-separated list of activity types
    /// * `after` - Return activities after this timestamp
    /// * `until` - Return activities until this timestamp
    /// * `page_size` - Number of activities per page
    pub async fn get_activities(
        &self,
        activity_types: Option<&str>,
        after: Option<&str>,
        until: Option<&str>,
        page_size: Option<u32>,
    ) -> AlpacaHttpResult<Vec<AlpacaActivity>> {
        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(types) = activity_types {
            params.insert("activity_types".to_string(), types.to_string());
        }
        if let Some(a) = after {
            params.insert("after".to_string(), a.to_string());
        }
        if let Some(u) = until {
            params.insert("until".to_string(), u.to_string());
        }
        if let Some(size) = page_size {
            params.insert("page_size".to_string(), size.to_string());
        }
        self.get::<HashMap<String, String>, Vec<AlpacaActivity>>("/v2/account/activities", Some(&params))
            .await
    }

    // ============================================================================
    // Asset API Endpoints
    // ============================================================================

    /// Get all assets.
    ///
    /// # Arguments
    ///
    /// * `status` - Filter by asset status ("active" or "inactive")
    /// * `asset_class` - Filter by asset class (e.g., "us_equity", "crypto")
    pub async fn get_assets(
        &self,
        status: Option<&str>,
        asset_class: Option<&str>,
    ) -> AlpacaHttpResult<Vec<AlpacaAsset>> {
        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(s) = status {
            params.insert("status".to_string(), s.to_string());
        }
        if let Some(c) = asset_class {
            params.insert("asset_class".to_string(), c.to_string());
        }
        let assets = self.get::<HashMap<String, String>, Vec<AlpacaAsset>>("/v2/assets", Some(&params))
            .await?;

        // Filter out unsupported asset classes that Alpaca doesn't actually offer
        // crypto_perp appears in API responses but is not a real product
        let filtered_assets: Vec<AlpacaAsset> = assets
            .into_iter()
            .filter(|asset| asset.class.to_lowercase() != "crypto_perp")
            .collect();

        Ok(filtered_assets)
    }

    /// Get a specific asset by symbol or ID.
    ///
    /// # Arguments
    ///
    /// * `symbol_or_asset_id` - The asset symbol or ID
    pub async fn get_asset(&self, symbol_or_asset_id: &str) -> AlpacaHttpResult<AlpacaAsset> {
        let path = format!("/v2/assets/{symbol_or_asset_id}");
        self.get::<HashMap<String, String>, AlpacaAsset>(&path, None)
            .await
    }

    // ============================================================================
    // Options API Endpoints
    // ============================================================================

    /// Get option contracts.
    ///
    /// # Arguments
    ///
    /// * `underlying_symbols` - Comma-separated list of underlying symbols
    /// * `status` - Filter by contract status
    /// * `expiration_date` - Filter by specific expiration date (YYYY-MM-DD)
    /// * `expiration_date_gte` - Filter by expiration date greater than or equal to
    /// * `expiration_date_lte` - Filter by expiration date less than or equal to
    /// * `strike_price_gte` - Filter by strike price greater than or equal to
    /// * `strike_price_lte` - Filter by strike price less than or equal to
    /// * `contract_type` - Filter by contract type ("call" or "put")
    pub async fn get_option_contracts(
        &self,
        underlying_symbols: Option<&str>,
        status: Option<&str>,
        expiration_date: Option<&str>,
        expiration_date_gte: Option<&str>,
        expiration_date_lte: Option<&str>,
        strike_price_gte: Option<&str>,
        strike_price_lte: Option<&str>,
        contract_type: Option<&str>,
    ) -> AlpacaHttpResult<AlpacaOptionContractsResponse> {
        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(s) = underlying_symbols {
            params.insert("underlying_symbols".to_string(), s.to_string());
        }
        if let Some(s) = status {
            params.insert("status".to_string(), s.to_string());
        }
        if let Some(d) = expiration_date {
            params.insert("expiration_date".to_string(), d.to_string());
        }
        if let Some(d) = expiration_date_gte {
            params.insert("expiration_date_gte".to_string(), d.to_string());
        }
        if let Some(d) = expiration_date_lte {
            params.insert("expiration_date_lte".to_string(), d.to_string());
        }
        if let Some(p) = strike_price_gte {
            params.insert("strike_price_gte".to_string(), p.to_string());
        }
        if let Some(p) = strike_price_lte {
            params.insert("strike_price_lte".to_string(), p.to_string());
        }
        if let Some(t) = contract_type {
            params.insert("type".to_string(), t.to_string());
        }
        self.get::<HashMap<String, String>, AlpacaOptionContractsResponse>("/v2/options/contracts", Some(&params))
            .await
    }

    /// Exercise an options position.
    ///
    /// # Arguments
    ///
    /// * `symbol_or_contract_id` - The option symbol or contract ID
    pub async fn exercise_option(&self, symbol_or_contract_id: &str) -> AlpacaHttpResult<()> {
        let path = format!("/v2/positions/{symbol_or_contract_id}/exercise");
        self.post::<HashMap<String, String>, ()>(&path, None, None)
            .await
    }

    // ============================================================================
    // Market Data API Endpoints
    // ============================================================================

    /// Get historical bars for stocks.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Comma-separated list of symbols
    /// * `timeframe` - Bar timeframe (e.g., "1Min", "1Hour", "1Day")
    /// * `start` - Start time (RFC3339 format)
    /// * `end` - End time (RFC3339 format)
    /// * `limit` - Maximum number of bars per symbol
    pub async fn get_stock_bars(
        &self,
        symbols: &str,
        timeframe: &str,
        start: Option<&str>,
        end: Option<&str>,
        limit: Option<u32>,
    ) -> AlpacaHttpResult<AlpacaBarsResponse> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("symbols".to_string(), symbols.to_string());
        params.insert("timeframe".to_string(), timeframe.to_string());
        if let Some(s) = start {
            params.insert("start".to_string(), s.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get_data::<HashMap<String, String>, AlpacaBarsResponse>("/v2/stocks/bars", Some(&params))
            .await
    }

    /// Get historical trades for stocks.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Comma-separated list of symbols
    /// * `start` - Start time (RFC3339 format)
    /// * `end` - End time (RFC3339 format)
    /// * `limit` - Maximum number of trades per symbol
    pub async fn get_stock_trades(
        &self,
        symbols: &str,
        start: Option<&str>,
        end: Option<&str>,
        limit: Option<u32>,
    ) -> AlpacaHttpResult<AlpacaTradesResponse> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("symbols".to_string(), symbols.to_string());
        if let Some(s) = start {
            params.insert("start".to_string(), s.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get_data::<HashMap<String, String>, AlpacaTradesResponse>("/v2/stocks/trades", Some(&params))
            .await
    }

    /// Get historical quotes for stocks.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Comma-separated list of symbols
    /// * `start` - Start time (RFC3339 format)
    /// * `end` - End time (RFC3339 format)
    /// * `limit` - Maximum number of quotes per symbol
    pub async fn get_stock_quotes(
        &self,
        symbols: &str,
        start: Option<&str>,
        end: Option<&str>,
        limit: Option<u32>,
    ) -> AlpacaHttpResult<AlpacaQuotesResponse> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("symbols".to_string(), symbols.to_string());
        if let Some(s) = start {
            params.insert("start".to_string(), s.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get_data::<HashMap<String, String>, AlpacaQuotesResponse>("/v2/stocks/quotes", Some(&params))
            .await
    }

    /// Get historical bars for crypto.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Comma-separated list of crypto symbols (e.g., "BTC/USD")
    /// * `timeframe` - Bar timeframe (e.g., "1Min", "1Hour", "1Day")
    /// * `start` - Start time (RFC3339 format)
    /// * `end` - End time (RFC3339 format)
    /// * `limit` - Maximum number of bars per symbol
    pub async fn get_crypto_bars(
        &self,
        symbols: &str,
        timeframe: &str,
        start: Option<&str>,
        end: Option<&str>,
        limit: Option<u32>,
    ) -> AlpacaHttpResult<AlpacaBarsResponse> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("symbols".to_string(), symbols.to_string());
        params.insert("timeframe".to_string(), timeframe.to_string());
        if let Some(s) = start {
            params.insert("start".to_string(), s.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get_data::<HashMap<String, String>, AlpacaBarsResponse>("/v1beta3/crypto/us/bars", Some(&params))
            .await
    }

    /// Get historical trades for crypto.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Comma-separated list of crypto symbols
    /// * `start` - Start time (RFC3339 format)
    /// * `end` - End time (RFC3339 format)
    /// * `limit` - Maximum number of trades per symbol
    pub async fn get_crypto_trades(
        &self,
        symbols: &str,
        start: Option<&str>,
        end: Option<&str>,
        limit: Option<u32>,
    ) -> AlpacaHttpResult<AlpacaTradesResponse> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("symbols".to_string(), symbols.to_string());
        if let Some(s) = start {
            params.insert("start".to_string(), s.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get_data::<HashMap<String, String>, AlpacaTradesResponse>("/v1beta3/crypto/us/trades", Some(&params))
            .await
    }

    // ============================================================================
    // Private Helper Methods
    // ============================================================================

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
        let mut headers: HashMap<String, String> = HashMap::new();
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
        let mut headers: HashMap<String, String> = HashMap::new();
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
    use rstest::rstest;


    #[rstest]
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

    #[rstest]
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

    #[rstest]
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
