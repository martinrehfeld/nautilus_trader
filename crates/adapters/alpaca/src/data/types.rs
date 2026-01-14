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

//! Data-specific types for Alpaca adapter.
//!
//! This module contains request/response types and state tracking for the data client.

use std::collections::{HashMap, HashSet};

use nautilus_model::identifiers::{InstrumentId, Symbol};
use serde::Serialize;

use crate::common::enums::{AlpacaAssetClass, AlpacaDataFeed};

// ================================================================================================
// Subscription Management
// ================================================================================================

/// Tracks active subscriptions for the data client.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionState {
    /// Maps instrument IDs to their symbols (for quick lookup).
    pub instruments: HashMap<InstrumentId, Symbol>,
    /// Maps instrument IDs to their asset class.
    pub asset_classes: HashMap<InstrumentId, AlpacaAssetClass>,
    /// Trade subscriptions by asset class.
    pub trades: HashMap<AlpacaAssetClass, HashSet<String>>,
    /// Quote subscriptions by asset class.
    pub quotes: HashMap<AlpacaAssetClass, HashSet<String>>,
    /// Bar subscriptions by asset class.
    pub bars: HashMap<AlpacaAssetClass, HashSet<String>>,
}

impl SubscriptionState {
    /// Creates a new empty subscription state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a trade subscription.
    pub fn add_trade(&mut self, asset_class: AlpacaAssetClass, symbol: String) {
        self.trades
            .entry(asset_class)
            .or_default()
            .insert(symbol);
    }

    /// Removes a trade subscription.
    pub fn remove_trade(&mut self, asset_class: AlpacaAssetClass, symbol: &str) -> bool {
        self.trades
            .get_mut(&asset_class)
            .map_or(false, |set| set.remove(symbol))
    }

    /// Adds a quote subscription.
    pub fn add_quote(&mut self, asset_class: AlpacaAssetClass, symbol: String) {
        self.quotes
            .entry(asset_class)
            .or_default()
            .insert(symbol);
    }

    /// Removes a quote subscription.
    pub fn remove_quote(&mut self, asset_class: AlpacaAssetClass, symbol: &str) -> bool {
        self.quotes
            .get_mut(&asset_class)
            .map_or(false, |set| set.remove(symbol))
    }

    /// Adds a bar subscription.
    pub fn add_bar(&mut self, asset_class: AlpacaAssetClass, symbol: String) {
        self.bars
            .entry(asset_class)
            .or_default()
            .insert(symbol);
    }

    /// Removes a bar subscription.
    pub fn remove_bar(&mut self, asset_class: AlpacaAssetClass, symbol: &str) -> bool {
        self.bars
            .get_mut(&asset_class)
            .map_or(false, |set| set.remove(symbol))
    }

    /// Registers an instrument with its asset class.
    pub fn register_instrument(
        &mut self,
        instrument_id: InstrumentId,
        symbol: Symbol,
        asset_class: AlpacaAssetClass,
    ) {
        self.instruments.insert(instrument_id, symbol);
        self.asset_classes.insert(instrument_id, asset_class);
    }

    /// Gets the asset class for an instrument.
    #[must_use]
    pub fn get_asset_class(&self, instrument_id: &InstrumentId) -> Option<AlpacaAssetClass> {
        self.asset_classes.get(instrument_id).copied()
    }

    /// Gets the symbol for an instrument.
    #[must_use]
    pub fn get_symbol(&self, instrument_id: &InstrumentId) -> Option<&Symbol> {
        self.instruments.get(instrument_id)
    }
}

// ================================================================================================
// Historical Data Request Parameters
// ================================================================================================

/// Parameters for historical bars request.
#[derive(Debug, Clone, Serialize)]
pub struct HistoricalBarsRequest {
    /// Comma-separated list of symbols.
    pub symbols: String,
    /// Timeframe (e.g., "1Min", "1Hour", "1Day").
    pub timeframe: String,
    /// Start time (RFC3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// End time (RFC3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// Maximum number of bars to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Page token for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

impl HistoricalBarsRequest {
    /// Creates a new historical bars request.
    #[must_use]
    pub fn new(symbols: String, timeframe: String) -> Self {
        Self {
            symbols,
            timeframe,
            start: None,
            end: None,
            limit: None,
            page_token: None,
        }
    }

    /// Sets the start time.
    #[must_use]
    pub fn with_start(mut self, start: String) -> Self {
        self.start = Some(start);
        self
    }

    /// Sets the end time.
    #[must_use]
    pub fn with_end(mut self, end: String) -> Self {
        self.end = Some(end);
        self
    }

    /// Sets the limit.
    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the page token.
    #[must_use]
    pub fn with_page_token(mut self, page_token: String) -> Self {
        self.page_token = Some(page_token);
        self
    }
}

/// Parameters for historical trades request.
#[derive(Debug, Clone, Serialize)]
pub struct HistoricalTradesRequest {
    /// Comma-separated list of symbols.
    pub symbols: String,
    /// Start time (RFC3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// End time (RFC3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// Maximum number of trades to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Page token for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

impl HistoricalTradesRequest {
    /// Creates a new historical trades request.
    #[must_use]
    pub fn new(symbols: String) -> Self {
        Self {
            symbols,
            start: None,
            end: None,
            limit: None,
            page_token: None,
        }
    }

    /// Sets the start time.
    #[must_use]
    pub fn with_start(mut self, start: String) -> Self {
        self.start = Some(start);
        self
    }

    /// Sets the end time.
    #[must_use]
    pub fn with_end(mut self, end: String) -> Self {
        self.end = Some(end);
        self
    }

    /// Sets the limit.
    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Parameters for historical quotes request.
#[derive(Debug, Clone, Serialize)]
pub struct HistoricalQuotesRequest {
    /// Comma-separated list of symbols.
    pub symbols: String,
    /// Start time (RFC3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// End time (RFC3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// Maximum number of quotes to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Page token for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

impl HistoricalQuotesRequest {
    /// Creates a new historical quotes request.
    #[must_use]
    pub fn new(symbols: String) -> Self {
        Self {
            symbols,
            start: None,
            end: None,
            limit: None,
            page_token: None,
        }
    }

    /// Sets the start time.
    #[must_use]
    pub fn with_start(mut self, start: String) -> Self {
        self.start = Some(start);
        self
    }

    /// Sets the end time.
    #[must_use]
    pub fn with_end(mut self, end: String) -> Self {
        self.end = Some(end);
        self
    }

    /// Sets the limit.
    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

// ================================================================================================
// Data Client Configuration
// ================================================================================================

/// Configuration for the Alpaca data client.
#[derive(Debug, Clone)]
pub struct AlpacaDataClientConfig {
    /// API key for authentication.
    pub api_key: String,
    /// API secret for authentication.
    pub api_secret: String,
    /// Data feed subscription level.
    pub data_feed: AlpacaDataFeed,
    /// Whether to use paper trading environment.
    pub paper_trading: bool,
    /// Request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Proxy URL (optional).
    pub proxy_url: Option<String>,
}

impl AlpacaDataClientConfig {
    /// Creates a new data client configuration.
    #[must_use]
    pub fn new(api_key: String, api_secret: String, data_feed: AlpacaDataFeed) -> Self {
        Self {
            api_key,
            api_secret,
            data_feed,
            paper_trading: false,
            timeout_secs: None,
            proxy_url: None,
        }
    }

    /// Sets paper trading mode.
    #[must_use]
    pub fn with_paper_trading(mut self, paper_trading: bool) -> Self {
        self.paper_trading = paper_trading;
        self
    }

    /// Sets request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    /// Sets proxy URL.
    #[must_use]
    pub fn with_proxy(mut self, proxy_url: String) -> Self {
        self.proxy_url = Some(proxy_url);
        self
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use nautilus_model::identifiers::Venue;

    #[rstest]
    fn test_subscription_state_trades() {
        let mut state = SubscriptionState::new();

        state.add_trade(AlpacaAssetClass::UsEquity, "AAPL".to_string());
        assert!(state.trades.get(&AlpacaAssetClass::UsEquity).unwrap().contains("AAPL"));

        let removed = state.remove_trade(AlpacaAssetClass::UsEquity, "AAPL");
        assert!(removed);
        assert!(!state.trades.get(&AlpacaAssetClass::UsEquity).unwrap().contains("AAPL"));
    }

    #[rstest]
    fn test_subscription_state_register_instrument() {
        let mut state = SubscriptionState::new();
        let instrument_id = InstrumentId::new(
            Symbol::new("AAPL"),
            Venue::new("ALPACA"),
        );
        let symbol = Symbol::new("AAPL");

        state.register_instrument(instrument_id, symbol, AlpacaAssetClass::UsEquity);

        assert_eq!(
            state.get_asset_class(&instrument_id),
            Some(AlpacaAssetClass::UsEquity)
        );
        assert_eq!(state.get_symbol(&instrument_id), Some(&symbol));
    }

    #[rstest]
    fn test_historical_bars_request_builder() {
        let request = HistoricalBarsRequest::new("AAPL".to_string(), "1Min".to_string())
            .with_start("2023-01-01T00:00:00Z".to_string())
            .with_end("2023-01-02T00:00:00Z".to_string())
            .with_limit(1000);

        assert_eq!(request.symbols, "AAPL");
        assert_eq!(request.timeframe, "1Min");
        assert!(request.start.is_some());
        assert!(request.end.is_some());
        assert_eq!(request.limit, Some(1000));
    }

    #[rstest]
    fn test_data_client_config_builder() {
        let config = AlpacaDataClientConfig::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            AlpacaDataFeed::Iex,
        )
        .with_paper_trading(true)
        .with_timeout(30);

        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.data_feed, AlpacaDataFeed::Iex);
        assert!(config.paper_trading);
        assert_eq!(config.timeout_secs, Some(30));
    }
}
