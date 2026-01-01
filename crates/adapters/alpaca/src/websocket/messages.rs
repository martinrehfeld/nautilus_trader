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

//! Alpaca WebSocket message types.

use serde::{Deserialize, Serialize};

/// Alpaca WebSocket action types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlpacaWsAction {
    /// Authentication action.
    Auth,
    /// Subscribe to streams.
    Subscribe,
    /// Unsubscribe from streams.
    Unsubscribe,
}

/// Alpaca WebSocket authentication message.
#[derive(Debug, Clone, Serialize)]
pub struct AlpacaWsAuthMessage {
    /// Action type.
    pub action: AlpacaWsAction,
    /// API key.
    pub key: String,
    /// API secret.
    pub secret: String,
}

impl AlpacaWsAuthMessage {
    /// Creates a new authentication message.
    #[must_use]
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            action: AlpacaWsAction::Auth,
            key: api_key,
            secret: api_secret,
        }
    }
}

/// Alpaca WebSocket subscription message.
#[derive(Debug, Clone, Serialize)]
pub struct AlpacaWsSubscribeMessage {
    /// Action type.
    pub action: AlpacaWsAction,
    /// Trade symbols to subscribe to.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trades: Vec<String>,
    /// Quote symbols to subscribe to.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quotes: Vec<String>,
    /// Bar symbols to subscribe to.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bars: Vec<String>,
}

impl AlpacaWsSubscribeMessage {
    /// Creates a new subscription message.
    #[must_use]
    pub fn subscribe(trades: Vec<String>, quotes: Vec<String>, bars: Vec<String>) -> Self {
        Self {
            action: AlpacaWsAction::Subscribe,
            trades,
            quotes,
            bars,
        }
    }

    /// Creates a new unsubscription message.
    #[must_use]
    pub fn unsubscribe(trades: Vec<String>, quotes: Vec<String>, bars: Vec<String>) -> Self {
        Self {
            action: AlpacaWsAction::Unsubscribe,
            trades,
            quotes,
            bars,
        }
    }
}

/// Alpaca WebSocket message types from server.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "T")]
pub enum AlpacaWsMessage {
    /// Connection success.
    #[serde(rename = "success")]
    Success { msg: String },
    /// Subscription confirmation.
    #[serde(rename = "subscription")]
    Subscription {
        trades: Vec<String>,
        quotes: Vec<String>,
        bars: Vec<String>,
    },
    /// Error message.
    #[serde(rename = "error")]
    Error { msg: String, code: i32 },
    /// Trade tick data.
    #[serde(rename = "t")]
    Trade(AlpacaWsTrade),
    /// Quote tick data.
    #[serde(rename = "q")]
    Quote(AlpacaWsQuote),
    /// Bar data.
    #[serde(rename = "b")]
    Bar(AlpacaWsBar),
}

/// Alpaca WebSocket trade message.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaWsTrade {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade ID.
    pub i: u64,
    /// Exchange code.
    pub x: String,
    /// Trade price.
    pub p: f64,
    /// Trade size.
    pub s: u64,
    /// Trade timestamp (RFC3339).
    pub t: String,
    /// Trade conditions.
    #[serde(default)]
    pub c: Vec<String>,
    /// Tape.
    pub z: String,
}

/// Alpaca WebSocket quote message.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaWsQuote {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Ask exchange code.
    pub ax: String,
    /// Ask price.
    pub ap: f64,
    /// Ask size.
    #[serde(rename = "as")]
    pub ask_size: u64,
    /// Bid exchange code.
    pub bx: String,
    /// Bid price.
    pub bp: f64,
    /// Bid size.
    pub bs: u64,
    /// Quote timestamp (RFC3339).
    pub t: String,
    /// Quote conditions.
    #[serde(default)]
    pub c: Vec<String>,
    /// Tape.
    pub z: String,
}

/// Alpaca WebSocket bar message.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaWsBar {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Open price.
    pub o: f64,
    /// High price.
    pub h: f64,
    /// Low price.
    pub l: f64,
    /// Close price.
    pub c: f64,
    /// Volume.
    pub v: u64,
    /// Bar timestamp (RFC3339).
    pub t: String,
    /// Number of trades.
    pub n: u64,
    /// Volume weighted average price.
    pub vw: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_message_serialization() {
        let msg = AlpacaWsAuthMessage::new("key123".to_string(), "secret456".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"action\":\"auth\""));
        assert!(json.contains("\"key\":\"key123\""));
        assert!(json.contains("\"secret\":\"secret456\""));
    }

    #[test]
    fn test_subscribe_message_serialization() {
        let msg = AlpacaWsSubscribeMessage::subscribe(
            vec!["AAPL".to_string()],
            vec!["MSFT".to_string()],
            vec![],
        );
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"action\":\"subscribe\""));
        assert!(json.contains("\"trades\":[\"AAPL\"]"));
        assert!(json.contains("\"quotes\":[\"MSFT\"]"));
        // Empty bars should be skipped
        assert!(!json.contains("bars"));
    }

    #[test]
    fn test_trade_message_deserialization() {
        let json = r#"{"T":"t","S":"AAPL","i":123,"x":"V","p":150.25,"s":100,"t":"2024-01-15T14:30:00Z","c":["@"],"z":"A"}"#;
        let msg: AlpacaWsMessage = serde_json::from_str(json).unwrap();

        if let AlpacaWsMessage::Trade(trade) = msg {
            assert_eq!(trade.symbol, "AAPL");
            assert_eq!(trade.p, 150.25);
            assert_eq!(trade.s, 100);
        } else {
            panic!("Expected Trade message");
        }
    }

    #[test]
    fn test_quote_message_deserialization() {
        let json = r#"{"T":"q","S":"AAPL","ax":"V","ap":150.30,"as":500,"bx":"N","bp":150.25,"bs":300,"t":"2024-01-15T14:30:00Z","c":[],"z":"A"}"#;
        let msg: AlpacaWsMessage = serde_json::from_str(json).unwrap();

        if let AlpacaWsMessage::Quote(quote) = msg {
            assert_eq!(quote.symbol, "AAPL");
            assert_eq!(quote.ap, 150.30);
            assert_eq!(quote.bp, 150.25);
        } else {
            panic!("Expected Quote message");
        }
    }
}
