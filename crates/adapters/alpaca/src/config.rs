// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
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

//! Configuration structures for the Alpaca adapter.

use nautilus_model::identifiers::Venue;

use crate::common::enums::{AlpacaAssetClass, AlpacaDataFeed};

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Configuration for the Alpaca instrument provider.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaInstrumentProviderConfig {
    /// Whether to load all instruments on initialization.
    pub load_all: bool,
    /// Asset classes to load instruments for.
    pub asset_classes: Option<Vec<AlpacaAssetClass>>,
    /// Underlying symbols to load option contracts for (required for options).
    pub option_underlying_symbols: Option<Vec<String>>,
    /// Whether to log warnings for parsing errors.
    pub log_warnings: bool,
}

impl Default for AlpacaInstrumentProviderConfig {
    fn default() -> Self {
        Self {
            load_all: false,
            asset_classes: Some(vec![AlpacaAssetClass::UsEquity]),
            option_underlying_symbols: None,
            log_warnings: true,
        }
    }
}

impl AlpacaInstrumentProviderConfig {
    /// Creates a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to load all instruments on initialization.
    #[must_use]
    pub fn with_load_all(mut self, load_all: bool) -> Self {
        self.load_all = load_all;
        self
    }

    /// Sets the asset classes to load.
    #[must_use]
    pub fn with_asset_classes(mut self, asset_classes: Vec<AlpacaAssetClass>) -> Self {
        self.asset_classes = Some(asset_classes);
        self
    }

    /// Sets the option underlying symbols.
    #[must_use]
    pub fn with_option_underlying_symbols(mut self, symbols: Vec<String>) -> Self {
        self.option_underlying_symbols = Some(symbols);
        self
    }

    /// Sets whether to log warnings.
    #[must_use]
    pub fn with_log_warnings(mut self, log_warnings: bool) -> Self {
        self.log_warnings = log_warnings;
        self
    }
}

/// Configuration for the Alpaca data client.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaDataClientConfig {
    /// The venue for the client.
    pub venue: Venue,
    /// The Alpaca API public key.
    pub api_key: Option<String>,
    /// The Alpaca API secret key.
    pub api_secret: Option<String>,
    /// The market data feed subscription level.
    pub data_feed: AlpacaDataFeed,
    /// If the client should connect to paper trading endpoints.
    pub paper_trading: bool,
    /// Optional HTTP client custom endpoint override.
    pub base_url_http: Option<String>,
    /// Optional WebSocket client custom endpoint override.
    pub base_url_ws: Option<String>,
    /// Optional proxy URL for HTTP requests.
    pub proxy_url: Option<String>,
    /// The timeout (seconds) for HTTP requests.
    pub http_timeout_secs: Option<u64>,
    /// The interval (minutes) between instrument updates.
    pub update_instruments_interval_mins: Option<u64>,
    /// Instrument provider configuration.
    pub instrument_provider: Option<AlpacaInstrumentProviderConfig>,
}

impl Default for AlpacaDataClientConfig {
    fn default() -> Self {
        Self {
            venue: Venue::from("ALPACA"),
            api_key: None,
            api_secret: None,
            data_feed: AlpacaDataFeed::Iex,
            paper_trading: true,
            base_url_http: None,
            base_url_ws: None,
            proxy_url: None,
            http_timeout_secs: Some(30),
            update_instruments_interval_mins: Some(60),
            instrument_provider: Some(AlpacaInstrumentProviderConfig::default()),
        }
    }
}

impl AlpacaDataClientConfig {
    /// Creates a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the API credentials.
    #[must_use]
    pub fn with_credentials(mut self, api_key: String, api_secret: String) -> Self {
        self.api_key = Some(api_key);
        self.api_secret = Some(api_secret);
        self
    }

    /// Sets the data feed.
    #[must_use]
    pub fn with_data_feed(mut self, data_feed: AlpacaDataFeed) -> Self {
        self.data_feed = data_feed;
        self
    }

    /// Sets whether to use paper trading.
    #[must_use]
    pub fn with_paper_trading(mut self, paper_trading: bool) -> Self {
        self.paper_trading = paper_trading;
        self
    }

    /// Sets the HTTP base URL override.
    #[must_use]
    pub fn with_base_url_http(mut self, url: String) -> Self {
        self.base_url_http = Some(url);
        self
    }

    /// Sets the WebSocket base URL override.
    #[must_use]
    pub fn with_base_url_ws(mut self, url: String) -> Self {
        self.base_url_ws = Some(url);
        self
    }

    /// Sets the HTTP timeout.
    #[must_use]
    pub fn with_http_timeout_secs(mut self, timeout: u64) -> Self {
        self.http_timeout_secs = Some(timeout);
        self
    }

    /// Sets the instrument update interval.
    #[must_use]
    pub fn with_update_instruments_interval_mins(mut self, interval: u64) -> Self {
        self.update_instruments_interval_mins = Some(interval);
        self
    }

    /// Sets the instrument provider configuration.
    #[must_use]
    pub fn with_instrument_provider(mut self, config: AlpacaInstrumentProviderConfig) -> Self {
        self.instrument_provider = Some(config);
        self
    }

    /// Returns the HTTP base URL, considering paper trading and overrides.
    #[must_use]
    pub fn http_base_url(&self) -> String {
        self.base_url_http.clone().unwrap_or_else(|| {
            if self.paper_trading {
                "https://paper-api.alpaca.markets".to_string()
            } else {
                "https://api.alpaca.markets".to_string()
            }
        })
    }

    /// Returns the WebSocket base URL for a specific asset class.
    #[must_use]
    pub fn ws_base_url(&self, asset_class: AlpacaAssetClass) -> String {
        self.base_url_ws.clone().unwrap_or_else(|| {
            match asset_class {
                AlpacaAssetClass::UsEquity => match self.data_feed {
                    AlpacaDataFeed::Iex => "wss://stream.data.alpaca.markets/v2/iex".to_string(),
                    AlpacaDataFeed::Sip => "wss://stream.data.alpaca.markets/v2/sip".to_string(),
                },
                AlpacaAssetClass::Crypto => {
                    "wss://stream.data.alpaca.markets/v1beta3/crypto/us".to_string()
                }
                AlpacaAssetClass::Option => {
                    "wss://stream.data.alpaca.markets/v1beta1/options".to_string()
                }
            }
        })
    }

    /// Returns `true` if both API key and secret are available.
    #[must_use]
    pub fn has_credentials(&self) -> bool {
        self.api_key.is_some() && self.api_secret.is_some()
    }
}

/// Configuration for the Alpaca execution client.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaExecClientConfig {
    /// The venue for the client.
    pub venue: Venue,
    /// The Alpaca API public key.
    pub api_key: Option<String>,
    /// The Alpaca API secret key.
    pub api_secret: Option<String>,
    /// If the client should connect to paper trading endpoints.
    pub paper_trading: bool,
    /// Optional HTTP client custom endpoint override.
    pub base_url_http: Option<String>,
    /// Optional WebSocket client custom endpoint override.
    pub base_url_ws: Option<String>,
    /// Optional proxy URL for HTTP requests.
    pub proxy_url: Option<String>,
    /// The timeout (seconds) for HTTP requests.
    pub http_timeout_secs: Option<u64>,
    /// The maximum number of times to retry submitting an order on failure.
    pub max_retries: Option<u32>,
    /// The delay (seconds) between order submit retries.
    pub retry_delay_secs: Option<f64>,
    /// Instrument provider configuration.
    pub instrument_provider: Option<AlpacaInstrumentProviderConfig>,
}

impl Default for AlpacaExecClientConfig {
    fn default() -> Self {
        Self {
            venue: Venue::from("ALPACA"),
            api_key: None,
            api_secret: None,
            paper_trading: true,
            base_url_http: None,
            base_url_ws: None,
            proxy_url: None,
            http_timeout_secs: Some(30),
            max_retries: Some(3),
            retry_delay_secs: Some(1.0),
            instrument_provider: Some(AlpacaInstrumentProviderConfig::default()),
        }
    }
}

impl AlpacaExecClientConfig {
    /// Creates a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the API credentials.
    #[must_use]
    pub fn with_credentials(mut self, api_key: String, api_secret: String) -> Self {
        self.api_key = Some(api_key);
        self.api_secret = Some(api_secret);
        self
    }

    /// Sets whether to use paper trading.
    #[must_use]
    pub fn with_paper_trading(mut self, paper_trading: bool) -> Self {
        self.paper_trading = paper_trading;
        self
    }

    /// Sets the HTTP base URL override.
    #[must_use]
    pub fn with_base_url_http(mut self, url: String) -> Self {
        self.base_url_http = Some(url);
        self
    }

    /// Sets the WebSocket base URL override.
    #[must_use]
    pub fn with_base_url_ws(mut self, url: String) -> Self {
        self.base_url_ws = Some(url);
        self
    }

    /// Sets the HTTP timeout.
    #[must_use]
    pub fn with_http_timeout_secs(mut self, timeout: u64) -> Self {
        self.http_timeout_secs = Some(timeout);
        self
    }

    /// Sets the max retries.
    #[must_use]
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Sets the retry delay.
    #[must_use]
    pub fn with_retry_delay_secs(mut self, delay: f64) -> Self {
        self.retry_delay_secs = Some(delay);
        self
    }

    /// Sets the instrument provider configuration.
    #[must_use]
    pub fn with_instrument_provider(mut self, config: AlpacaInstrumentProviderConfig) -> Self {
        self.instrument_provider = Some(config);
        self
    }

    /// Returns the HTTP base URL, considering paper trading and overrides.
    #[must_use]
    pub fn http_base_url(&self) -> String {
        self.base_url_http.clone().unwrap_or_else(|| {
            if self.paper_trading {
                "https://paper-api.alpaca.markets".to_string()
            } else {
                "https://api.alpaca.markets".to_string()
            }
        })
    }

    /// Returns the WebSocket base URL for trading.
    #[must_use]
    pub fn ws_base_url(&self) -> String {
        self.base_url_ws.clone().unwrap_or_else(|| {
            "wss://stream.data.alpaca.markets/v2/trading".to_string()
        })
    }

    /// Returns `true` if both API key and secret are available.
    #[must_use]
    pub fn has_credentials(&self) -> bool {
        self.api_key.is_some() && self.api_secret.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    fn test_data_client_config_default() {
        let config = AlpacaDataClientConfig::default();
        assert_eq!(config.venue.as_str(), "ALPACA");
        assert!(config.paper_trading);
        assert!(matches!(config.data_feed, AlpacaDataFeed::Iex));
        assert_eq!(config.http_timeout_secs, Some(30));
    }

    #[rstest]
    fn test_data_client_config_builder() {
        let config = AlpacaDataClientConfig::new()
            .with_credentials("key".to_string(), "secret".to_string())
            .with_paper_trading(false)
            .with_data_feed(AlpacaDataFeed::Sip)
            .with_http_timeout_secs(60);

        assert!(config.has_credentials());
        assert!(!config.paper_trading);
        assert!(matches!(config.data_feed, AlpacaDataFeed::Sip));
        assert_eq!(config.http_timeout_secs, Some(60));
    }

    #[rstest]
    fn test_data_client_config_urls() {
        let config = AlpacaDataClientConfig::default();
        assert_eq!(
            config.http_base_url(),
            "https://paper-api.alpaca.markets"
        );
        assert_eq!(
            config.ws_base_url(AlpacaAssetClass::UsEquity),
            "wss://stream.data.alpaca.markets/v2/iex"
        );

        let live_config = AlpacaDataClientConfig::new().with_paper_trading(false);
        assert_eq!(live_config.http_base_url(), "https://api.alpaca.markets");
    }

    #[rstest]
    fn test_exec_client_config_default() {
        let config = AlpacaExecClientConfig::default();
        assert_eq!(config.venue.as_str(), "ALPACA");
        assert!(config.paper_trading);
        assert_eq!(config.max_retries, Some(3));
        assert_eq!(config.retry_delay_secs, Some(1.0));
    }

    #[rstest]
    fn test_exec_client_config_builder() {
        let config = AlpacaExecClientConfig::new()
            .with_credentials("key".to_string(), "secret".to_string())
            .with_paper_trading(false)
            .with_max_retries(5)
            .with_retry_delay_secs(2.0);

        assert!(config.has_credentials());
        assert!(!config.paper_trading);
        assert_eq!(config.max_retries, Some(5));
        assert_eq!(config.retry_delay_secs, Some(2.0));
    }

    #[rstest]
    fn test_instrument_provider_config_default() {
        let config = AlpacaInstrumentProviderConfig::default();
        assert!(!config.load_all);
        assert!(config.asset_classes.is_some());
        assert!(config.log_warnings);
    }

    #[rstest]
    fn test_instrument_provider_config_builder() {
        let config = AlpacaInstrumentProviderConfig::new()
            .with_load_all(true)
            .with_asset_classes(vec![AlpacaAssetClass::Crypto])
            .with_option_underlying_symbols(vec!["AAPL".to_string()])
            .with_log_warnings(false);

        assert!(config.load_all);
        assert!(config.option_underlying_symbols.is_some());
        assert!(!config.log_warnings);
    }
}
