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

//! Alpaca WebSocket client implementation.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use nautilus_network::websocket::WebSocketClient;
use tokio::sync::mpsc;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::messages::{AlpacaWsAuthMessage, AlpacaWsMessage, AlpacaWsSubscribeMessage};
use crate::common::{
    credential::AlpacaCredential,
    enums::{AlpacaAssetClass, AlpacaDataFeed},
    urls::get_ws_data_url,
};

/// Alpaca WebSocket client for market data streaming.
#[derive(Debug)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaWebSocketClient {
    url: String,
    credential: AlpacaCredential,
    asset_class: AlpacaAssetClass,
    #[allow(dead_code)]
    inner: Arc<tokio::sync::RwLock<Option<WebSocketClient>>>,
    signal: Arc<AtomicBool>,
    #[allow(dead_code)]
    message_tx: Option<mpsc::UnboundedSender<AlpacaWsMessage>>,
}

impl AlpacaWebSocketClient {
    /// Creates a new Alpaca WebSocket client.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Alpaca API key.
    /// * `api_secret` - Alpaca API secret.
    /// * `asset_class` - Asset class for the data stream.
    /// * `data_feed` - Data feed subscription level (IEX or SIP).
    /// * `url_override` - Optional URL override for testing.
    #[must_use]
    pub fn new(
        api_key: String,
        api_secret: String,
        asset_class: AlpacaAssetClass,
        data_feed: AlpacaDataFeed,
        url_override: Option<String>,
    ) -> Self {
        let credential = AlpacaCredential::new(api_key, api_secret);
        let url =
            url_override.unwrap_or_else(|| get_ws_data_url(asset_class, data_feed).to_string());

        Self {
            url,
            credential,
            asset_class,
            inner: Arc::new(tokio::sync::RwLock::new(None)),
            signal: Arc::new(AtomicBool::new(false)),
            message_tx: None,
        }
    }

    /// Returns the WebSocket URL being used.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the asset class for this client.
    #[must_use]
    pub const fn asset_class(&self) -> AlpacaAssetClass {
        self.asset_class
    }

    /// Returns whether the client is connected.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.signal.load(Ordering::SeqCst)
    }

    /// Creates the authentication message for the WebSocket connection.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the auth message fails.
    #[must_use]
    pub fn auth_message(&self) -> String {
        let msg = AlpacaWsAuthMessage::new(
            self.credential.api_key().to_string(),
            self.credential.api_secret().to_string(),
        );
        serde_json::to_string(&msg).expect("Failed to serialize auth message")
    }

    /// Creates a subscription message for the given symbols.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the subscribe message fails.
    #[must_use]
    pub fn subscribe_trades_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::subscribe(symbols, vec![], vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize subscribe message")
    }

    /// Creates a subscription message for quotes.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the subscribe message fails.
    #[must_use]
    pub fn subscribe_quotes_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::subscribe(vec![], symbols, vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize subscribe message")
    }

    /// Creates a subscription message for bars.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the subscribe message fails.
    #[must_use]
    pub fn subscribe_bars_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::subscribe(vec![], vec![], symbols);
        serde_json::to_string(&msg).expect("Failed to serialize subscribe message")
    }

    /// Creates an unsubscription message for trades.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the unsubscribe message fails.
    #[must_use]
    pub fn unsubscribe_trades_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::unsubscribe(symbols, vec![], vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize unsubscribe message")
    }

    /// Creates an unsubscription message for quotes.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the unsubscribe message fails.
    #[must_use]
    pub fn unsubscribe_quotes_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::unsubscribe(vec![], symbols, vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize unsubscribe message")
    }

    /// Creates an unsubscription message for bars.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the unsubscribe message fails.
    #[must_use]
    pub fn unsubscribe_bars_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::unsubscribe(vec![], vec![], symbols);
        serde_json::to_string(&msg).expect("Failed to serialize unsubscribe message")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AlpacaWebSocketClient::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            AlpacaAssetClass::UsEquity,
            AlpacaDataFeed::Iex,
            None,
        );

        assert_eq!(client.url(), "wss://stream.data.alpaca.markets/v2/iex");
        assert_eq!(client.asset_class(), AlpacaAssetClass::UsEquity);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_auth_message() {
        let client = AlpacaWebSocketClient::new(
            "my_key".to_string(),
            "my_secret".to_string(),
            AlpacaAssetClass::UsEquity,
            AlpacaDataFeed::Iex,
            None,
        );

        let msg = client.auth_message();
        assert!(msg.contains("\"action\":\"auth\""));
        assert!(msg.contains("\"key\":\"my_key\""));
        assert!(msg.contains("\"secret\":\"my_secret\""));
    }

    #[test]
    fn test_subscribe_message() {
        let msg = AlpacaWebSocketClient::subscribe_trades_message(vec![
            "AAPL".to_string(),
            "MSFT".to_string(),
        ]);
        assert!(msg.contains("\"action\":\"subscribe\""));
        assert!(msg.contains("\"trades\":[\"AAPL\",\"MSFT\"]"));
    }

    #[test]
    fn test_crypto_url() {
        let client = AlpacaWebSocketClient::new(
            "key".to_string(),
            "secret".to_string(),
            AlpacaAssetClass::Crypto,
            AlpacaDataFeed::Iex,
            None,
        );

        assert_eq!(
            client.url(),
            "wss://stream.data.alpaca.markets/v1beta3/crypto/us"
        );
    }

    #[test]
    fn test_options_url() {
        let client = AlpacaWebSocketClient::new(
            "key".to_string(),
            "secret".to_string(),
            AlpacaAssetClass::Option,
            AlpacaDataFeed::Iex,
            None,
        );

        assert_eq!(
            client.url(),
            "wss://stream.data.alpaca.markets/v1beta1/options"
        );
    }
}
