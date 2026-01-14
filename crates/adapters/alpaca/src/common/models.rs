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

//! Common models for Alpaca adapter.
//!
//! This module contains shared data structures used across both data and execution clients.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Market Data Models (WebSocket Messages)

/// Trade data from Alpaca WebSocket streams.
///
/// This is the raw trade message format received from Alpaca's data streams.
/// The structure is the same across all asset classes (equity/crypto/options).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlpacaWsTrade {
    /// Message type ("t" for trade).
    #[serde(rename = "T")]
    pub msg_type: String,
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade ID.
    #[serde(rename = "i")]
    pub trade_id: Option<u64>,
    /// Exchange code where trade occurred.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Trade size (number of shares/units).
    #[serde(rename = "s")]
    pub size: u64,
    /// Trade timestamp (RFC3339 format).
    #[serde(rename = "t")]
    pub timestamp: String,
    /// Trade conditions (optional).
    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,
    /// Tape (optional, for equities: A, B, or C).
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Quote data from Alpaca WebSocket streams.
///
/// This is the raw quote message format received from Alpaca's data streams.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlpacaWsQuote {
    /// Message type ("q" for quote).
    #[serde(rename = "T")]
    pub msg_type: String,
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,
    /// Ask price.
    #[serde(rename = "ap")]
    pub ask_price: Decimal,
    /// Ask size.
    #[serde(rename = "as")]
    pub ask_size: u64,
    /// Bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,
    /// Bid price.
    #[serde(rename = "bp")]
    pub bid_price: Decimal,
    /// Bid size.
    #[serde(rename = "bs")]
    pub bid_size: u64,
    /// Quote timestamp (RFC3339 format).
    #[serde(rename = "t")]
    pub timestamp: String,
    /// Quote conditions (optional).
    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,
    /// Tape (optional, for equities).
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Bar (OHLCV) data from Alpaca WebSocket streams.
///
/// This is the raw bar message format received from Alpaca's data streams.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlpacaWsBar {
    /// Message type ("b" for bar).
    #[serde(rename = "T")]
    pub msg_type: String,
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Open price.
    #[serde(rename = "o")]
    pub open: Decimal,
    /// High price.
    #[serde(rename = "h")]
    pub high: Decimal,
    /// Low price.
    #[serde(rename = "l")]
    pub low: Decimal,
    /// Close price.
    #[serde(rename = "c")]
    pub close: Decimal,
    /// Volume.
    #[serde(rename = "v")]
    pub volume: u64,
    /// Bar start timestamp (RFC3339 format).
    #[serde(rename = "t")]
    pub timestamp: String,
    /// Number of trades in bar (optional).
    #[serde(rename = "n")]
    pub trade_count: Option<u64>,
    /// Volume-weighted average price (optional).
    #[serde(rename = "vw")]
    pub vwap: Option<Decimal>,
}

// ================================================================================================
// Instrument Models
// ================================================================================================

/// Represents an equity instrument.
///
/// This is used for parsing Alpaca assets into Nautilus instruments.
#[derive(Debug, Clone)]
pub struct AlpacaEquityInstrument {
    /// Symbol (e.g., "AAPL").
    pub symbol: String,
    /// Full name of the security.
    pub name: String,
    /// Exchange where the asset is traded.
    pub exchange: String,
    /// Asset ID from Alpaca.
    pub asset_id: String,
    /// Whether the asset is tradable.
    pub tradable: bool,
    /// Whether the asset is marginable.
    pub marginable: bool,
    /// Whether the asset is shortable.
    pub shortable: bool,
    /// Maintenance margin requirement (optional).
    pub maintenance_margin_requirement: Option<Decimal>,
}

/// Represents a cryptocurrency instrument.
#[derive(Debug, Clone)]
pub struct AlpacaCryptoInstrument {
    /// Symbol (e.g., "BTC/USD").
    pub symbol: String,
    /// Full name of the crypto asset.
    pub name: String,
    /// Asset ID from Alpaca.
    pub asset_id: String,
    /// Minimum order size.
    pub min_order_size: Option<Decimal>,
    /// Minimum price increment.
    pub min_price_increment: Option<Decimal>,
}

/// Represents an options contract instrument.
#[derive(Debug, Clone)]
pub struct AlpacaOptionInstrument {
    /// Option symbol (OCC format).
    pub symbol: String,
    /// Contract ID from Alpaca.
    pub contract_id: String,
    /// Underlying asset symbol.
    pub underlying_symbol: String,
    /// Contract type ("call" or "put").
    pub contract_type: String,
    /// Strike price.
    pub strike_price: Decimal,
    /// Expiration date (YYYY-MM-DD).
    pub expiration_date: String,
    /// Contract multiplier (typically 100).
    pub multiplier: String,
    /// Trading status.
    pub status: String,
    /// Whether the contract is tradable.
    pub tradable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_trade() {
        let json = r#"{
            "T": "t",
            "S": "AAPL",
            "i": 123456,
            "x": "V",
            "p": 150.25,
            "s": 100,
            "t": "2023-08-25T14:30:00Z",
            "c": ["@"],
            "z": "C"
        }"#;

        let trade: AlpacaWsTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.symbol, "AAPL");
        assert_eq!(trade.price.to_string(), "150.25");
        assert_eq!(trade.size, 100);
        assert_eq!(trade.msg_type, "t");
    }

    #[test]
    fn test_deserialize_quote() {
        let json = r#"{
            "T": "q",
            "S": "MSFT",
            "ax": "Q",
            "ap": 330.50,
            "as": 200,
            "bx": "Q",
            "bp": 330.45,
            "bs": 150,
            "t": "2023-08-25T14:30:00Z",
            "c": ["R"],
            "z": "C"
        }"#;

        let quote: AlpacaWsQuote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.symbol, "MSFT");
        assert_eq!(quote.ask_price.to_string(), "330.5");
        assert_eq!(quote.bid_price.to_string(), "330.45");
        assert_eq!(quote.ask_size, 200);
        assert_eq!(quote.bid_size, 150);
    }

    #[test]
    fn test_deserialize_bar() {
        let json = r#"{
            "T": "b",
            "S": "SPY",
            "o": 440.10,
            "h": 440.50,
            "l": 440.00,
            "c": 440.25,
            "v": 1000000,
            "t": "2023-08-25T14:30:00Z",
            "n": 5000,
            "vw": 440.22
        }"#;

        let bar: AlpacaWsBar = serde_json::from_str(json).unwrap();
        assert_eq!(bar.symbol, "SPY");
        assert_eq!(bar.open.to_string(), "440.1");
        assert_eq!(bar.high.to_string(), "440.5");
        assert_eq!(bar.low.to_string(), "440");
        assert_eq!(bar.close.to_string(), "440.25");
        assert_eq!(bar.volume, 1000000);
        assert_eq!(bar.trade_count, Some(5000));
    }

    #[test]
    fn test_crypto_trade() {
        let json = r#"{
            "T": "t",
            "S": "BTC/USD",
            "p": 45000.5,
            "s": 100,
            "t": "2023-08-25T14:30:00Z"
        }"#;

        let trade: AlpacaWsTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.symbol, "BTC/USD");
        assert_eq!(trade.price.to_string(), "45000.5");
        assert!(trade.exchange.is_none());
    }
}
