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

//! Alpaca API URL resolution utilities.

use super::enums::{AlpacaAssetClass, AlpacaDataFeed, AlpacaEnvironment};

// Trading API base URLs
const TRADING_API_LIVE: &str = "https://api.alpaca.markets";
const TRADING_API_PAPER: &str = "https://paper-api.alpaca.markets";

// Market Data API base URL
const DATA_API_BASE: &str = "https://data.alpaca.markets";

// WebSocket URLs for trading updates
const WS_TRADING_LIVE: &str = "wss://api.alpaca.markets/stream";
const WS_TRADING_PAPER: &str = "wss://paper-api.alpaca.markets/stream";

// WebSocket URLs for market data
const WS_STOCKS_IEX: &str = "wss://stream.data.alpaca.markets/v2/iex";
const WS_STOCKS_SIP: &str = "wss://stream.data.alpaca.markets/v2/sip";
const WS_CRYPTO: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us";
const WS_OPTIONS: &str = "wss://stream.data.alpaca.markets/v1beta1/options";

/// Returns the HTTP base URL for the trading API.
#[must_use]
pub fn get_http_base_url(environment: AlpacaEnvironment) -> &'static str {
    match environment {
        AlpacaEnvironment::Live => TRADING_API_LIVE,
        AlpacaEnvironment::Paper => TRADING_API_PAPER,
    }
}

/// Returns the HTTP base URL for the market data API.
#[must_use]
pub fn get_data_api_url() -> &'static str {
    DATA_API_BASE
}

/// Returns the WebSocket URL for trading updates (order fills, account updates).
#[must_use]
pub fn get_ws_trading_url(environment: AlpacaEnvironment) -> &'static str {
    match environment {
        AlpacaEnvironment::Live => WS_TRADING_LIVE,
        AlpacaEnvironment::Paper => WS_TRADING_PAPER,
    }
}

/// Returns the WebSocket URL for market data streaming.
#[must_use]
pub fn get_ws_data_url(asset_class: AlpacaAssetClass, data_feed: AlpacaDataFeed) -> &'static str {
    match asset_class {
        AlpacaAssetClass::UsEquity => match data_feed {
            AlpacaDataFeed::Iex => WS_STOCKS_IEX,
            AlpacaDataFeed::Sip => WS_STOCKS_SIP,
        },
        AlpacaAssetClass::Crypto => WS_CRYPTO,
        AlpacaAssetClass::Option => WS_OPTIONS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_url_live() {
        assert_eq!(
            get_http_base_url(AlpacaEnvironment::Live),
            "https://api.alpaca.markets"
        );
    }

    #[test]
    fn test_trading_url_paper() {
        assert_eq!(
            get_http_base_url(AlpacaEnvironment::Paper),
            "https://paper-api.alpaca.markets"
        );
    }

    #[test]
    fn test_ws_trading_url_live() {
        assert_eq!(
            get_ws_trading_url(AlpacaEnvironment::Live),
            "wss://api.alpaca.markets/stream"
        );
    }

    #[test]
    fn test_ws_trading_url_paper() {
        assert_eq!(
            get_ws_trading_url(AlpacaEnvironment::Paper),
            "wss://paper-api.alpaca.markets/stream"
        );
    }

    #[test]
    fn test_ws_data_url_stocks_iex() {
        assert_eq!(
            get_ws_data_url(AlpacaAssetClass::UsEquity, AlpacaDataFeed::Iex),
            "wss://stream.data.alpaca.markets/v2/iex"
        );
    }

    #[test]
    fn test_ws_data_url_stocks_sip() {
        assert_eq!(
            get_ws_data_url(AlpacaAssetClass::UsEquity, AlpacaDataFeed::Sip),
            "wss://stream.data.alpaca.markets/v2/sip"
        );
    }

    #[test]
    fn test_ws_data_url_crypto() {
        assert_eq!(
            get_ws_data_url(AlpacaAssetClass::Crypto, AlpacaDataFeed::Iex),
            "wss://stream.data.alpaca.markets/v1beta3/crypto/us"
        );
    }

    #[test]
    fn test_ws_data_url_options() {
        assert_eq!(
            get_ws_data_url(AlpacaAssetClass::Option, AlpacaDataFeed::Iex),
            "wss://stream.data.alpaca.markets/v1beta1/options"
        );
    }
}
