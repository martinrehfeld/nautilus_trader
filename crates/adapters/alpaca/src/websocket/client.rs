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

use anyhow::anyhow;
use std::{sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
}};

use nautilus_common::log_error;
use nautilus_network::{ratelimiter::quota::Quota, websocket::{MessageHandler, PingHandler, WebSocketClient, WebSocketConfig}};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use ustr::Ustr;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::messages::{AlpacaWsAuthMessage, AlpacaWsMessage, AlpacaWsSubscribeMessage};
use crate::{common::{
    AlpacaEnvironment, credential::AlpacaCredential, enums::{AlpacaAssetClass, AlpacaDataFeed}, urls::get_ws_url
}, websocket::{AlpacaAbstractMesssageHandler, AlpacaMessageHandler, AlpacaTradingSubscriptionMessage}};

/// Alpaca WebSocket client for market data streaming.
#[derive(Debug, Clone)]
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
    /// Creates a new connected Alpaca WebSocket client.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Alpaca API key.
    /// * `api_secret` - Alpaca API secret.
    /// * `asset_class` - Asset class for the data stream.
    /// * `data_feed` - Data feed subscription level (IEX or SIP).
    /// * `url_override` - Optional URL override for testing.
    /// Connect to the WebSocket server.
    /// * `message_handler` - Callback for incoming messages
    /// * ... more optional args, same as nautilus_network::websocket::client::WebSocketClient
    ///
    /// # Errors
    ///
    /// Returns an error if the connection process fails.
    ///
    /// # Panics
    ///
    /// Panics if subscription arguments fail to serialize to JSON.
    pub async fn connect(
        environment: AlpacaEnvironment,
        api_key: String,
        api_secret: String,
        asset_class: AlpacaAssetClass,
        data_feed: AlpacaDataFeed,
        url_override: Option<String>,
        message_handler: MessageHandler,
        ping_handler: Option<PingHandler>,
        post_reconnection: Option<Arc<dyn Fn() + Send + Sync>>,
        keyed_quotas: Vec<(String, Quota)>,
        default_quota: Option<Quota>,
    ) -> anyhow::Result<Self> {
        let credential = AlpacaCredential::new(api_key, api_secret);
        let url =
            url_override.unwrap_or_else(|| get_ws_url(environment, asset_class, data_feed).to_string());

        let config = WebSocketConfig {
            url: url.clone(),
            headers: vec![],
            heartbeat: Some(20),
            heartbeat_msg: None,
            reconnect_timeout_ms: None,
            reconnect_delay_initial_ms: None,
            reconnect_delay_max_ms: None,
            reconnect_backoff_factor: None,
            reconnect_jitter_ms: None,
            reconnect_max_attempts: None
        };

        let signal = Arc::new(AtomicBool::new(false));
        let signal_clone = signal.clone();

        let message_handler_wrapper: MessageHandler = Arc::new(move |msg: Message| {
            if matches!(msg, Message::Text(ref text) if text.as_str() == nautilus_network::RECONNECTED) {
                return;
            }
    
            // TODO: move this out of the closure and make it part of the AlpacaWebsocketClient struct, so a
            //       a upstream client could query for subscriptions.
            let mut alpaca_message_handler = AlpacaMessageHandler::for_feed(data_feed);
            let json_parse_result: Result<serde_json::Value, serde_json::Error> =
                serde_json::from_slice(msg.clone().into_data().as_ref());
            let values: Vec<serde_json::Value> = match json_parse_result {
                Ok(value) => {
                    match value {
                        serde_json::Value::Array(list) => {
                            list
                        },
                        _  => vec![value]
                    }
                },
                Err(e) => {
                    log_error!("Ignoring incoming non-JSON websocket message: {e}");
                    vec![]
                }
            };

            for value in values {
                let value_string: &str = &value.to_string();
                let msg: Message = value_string.into();
                let bytes = value_string.as_bytes();
                match alpaca_message_handler.process_message(bytes) {
                    Ok(None) => {
                        if alpaca_message_handler.is_authenticated() {
                            signal_clone.store(true, Ordering::SeqCst);
                        } else {
                            signal_clone.store(false, Ordering::SeqCst);
                        }
                    },
                    Ok(_) => message_handler(msg),
                    Err(e) => log_error!("Unable to process incoming websocket message: {e}")
                }
            }
        });

        let ws_client = WebSocketClient::connect(
            config,
            Some(message_handler_wrapper),
            ping_handler,
            post_reconnection,
            keyed_quotas,
            default_quota,
        )
        .await?;

        let alpaca_ws_client = Self {
            url,
            credential,
            asset_class,
            inner: Arc::new(tokio::sync::RwLock::new(Some(ws_client))),
            signal: signal,
            message_tx: None,
        };

        // initiate auth
        let auth_message = alpaca_ws_client.auth_message();
        alpaca_ws_client.send_text(auth_message, None).await?;

        alpaca_ws_client.wait_until_active(10.0).await?;

        // TODO: initial subscribe message needs to be added as new argument, String type?
        if data_feed == AlpacaDataFeed::Trading {
            let subscribe_message = AlpacaTradingSubscriptionMessage::new(
                vec!["trade_updates".to_string()]
            );
            let subscribe_message_string = serde_json::to_string(&subscribe_message).expect("Failed to serialize subscribe message");
            alpaca_ws_client.send_text(subscribe_message_string, None).await?;
        }

        Ok(alpaca_ws_client)
    }

    /// Creates a new unconnected Alpaca WebSocket client.
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
        environment: AlpacaEnvironment,
        api_key: String,
        api_secret: String,
        asset_class: AlpacaAssetClass,
        data_feed: AlpacaDataFeed,
        url_override: Option<String>,
    ) -> Self {
        let credential = AlpacaCredential::new(api_key, api_secret);
        let url =
            url_override.unwrap_or_else(|| get_ws_url(environment, asset_class, data_feed).to_string());

        Self {
            url,
            credential,
            asset_class,
            inner: Arc::new(tokio::sync::RwLock::new(None)),
            signal: Arc::new(AtomicBool::new(false)),
            message_tx: None,
        }
    }

    /// Wait until the WebSocket connection is active.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection times out.
    pub async fn wait_until_active(&self, timeout_secs: f64) -> anyhow::Result<()> {
        let timeout = tokio::time::Duration::from_secs_f64(timeout_secs);

        tokio::time::timeout(timeout, async {
            while !self.is_connected() {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| {
            anyhow!("WebSocket connection timeout after {timeout_secs} seconds")
        })?;

        Ok(())
    }

    /// Disconect the Websocket connection by dropping the inner client.
    /// TODO: make this async to properly close the inner clients socket
    pub fn disconnect(&self) -> () {
        self.signal.store(false, Ordering::SeqCst);
        let mut write_lock = self.inner.blocking_write();
        *write_lock = None;
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

    /// Return the inner Webclient
    // pub websocket

    /// Sends the given text `data` to the server.
    ///
    /// # Errors
    ///
    /// Returns a websocket error if unable to send.
    #[allow(unused_variables)]
    pub async fn send_text(&self, data: String, keys: Option<&[Ustr]>) -> Result<(), nautilus_network::error::SendError> {
        let read_lock = self.inner.read().await;
        let client: &WebSocketClient = read_lock.as_ref().expect("no WebSocketClient");
        client.send_text(data, keys).await
    }

    /// Sends the given bytes `data` to the server.
    ///
    /// # Errors
    ///
    /// Returns a websocket error if unable to send.
    #[allow(unused_variables)]
    pub async fn send_bytes(&self, data: Vec<u8>, keys: Option<&[Ustr]>) -> Result<(), nautilus_network::error::SendError> {
        let read_lock = self.inner.read().await;
        let client: &WebSocketClient = read_lock.as_ref().expect("no WebSocketClient");
        client.send_bytes(data, keys).await
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
        let msg = AlpacaWsSubscribeMessage::subscribe(symbols, vec![], vec![], vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize subscribe message")
    }

    /// Creates a subscription message for quotes.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the subscribe message fails.
    #[must_use]
    pub fn subscribe_quotes_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::subscribe(vec![], symbols, vec![], vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize subscribe message")
    }

    /// Creates a subscription message for bars.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the subscribe message fails.
    #[must_use]
    pub fn subscribe_bars_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::subscribe(vec![], vec![], symbols, vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize subscribe message")
    }

    /// Creates a subscription message for orderbooks.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the subscribe message fails.
    #[must_use]
    pub fn subscribe_orderbooks_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::subscribe(vec![], vec![], vec![], symbols);
        serde_json::to_string(&msg).expect("Failed to serialize subscribe message")
    }

    /// Creates an unsubscription message for trades.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the unsubscribe message fails.
    #[must_use]
    pub fn unsubscribe_trades_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::unsubscribe(symbols, vec![], vec![], vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize unsubscribe message")
    }

    /// Creates an unsubscription message for quotes.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the unsubscribe message fails.
    #[must_use]
    pub fn unsubscribe_quotes_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::unsubscribe(vec![], symbols, vec![], vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize unsubscribe message")
    }

    /// Creates an unsubscription message for bars.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the unsubscribe message fails.
    #[must_use]
    pub fn unsubscribe_bars_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::unsubscribe(vec![], vec![], symbols, vec![]);
        serde_json::to_string(&msg).expect("Failed to serialize unsubscribe message")
    }

    /// Creates an unsubscription message for orderbooks.
    ///
    /// # Panics
    ///
    /// Panics if serialization of the unsubscribe message fails.
    #[must_use]
    pub fn unsubscribe_orderbooks_message(symbols: Vec<String>) -> String {
        let msg = AlpacaWsSubscribeMessage::unsubscribe(vec![], vec![], vec![], symbols);
        serde_json::to_string(&msg).expect("Failed to serialize unsubscribe message")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    fn test_client_creation() {
        let client = AlpacaWebSocketClient::new(
            AlpacaEnvironment::Live,
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

    #[rstest]
    fn test_auth_message() {
        let client = AlpacaWebSocketClient::new(
            AlpacaEnvironment::Live,
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

    #[rstest]
    fn test_subscribe_message() {
        let msg = AlpacaWebSocketClient::subscribe_trades_message(vec![
            "AAPL".to_string(),
            "MSFT".to_string(),
        ]);
        assert!(msg.contains("\"action\":\"subscribe\""));
        assert!(msg.contains("\"trades\":[\"AAPL\",\"MSFT\"]"));
    }

    #[rstest]
    fn test_crypto_url() {
        let client = AlpacaWebSocketClient::new(
            AlpacaEnvironment::Live,
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

    #[rstest]
    fn test_options_url() {
        let client = AlpacaWebSocketClient::new(
            AlpacaEnvironment::Live,
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
