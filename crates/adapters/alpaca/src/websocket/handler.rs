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

//! Message handler for Alpaca WebSocket streams.

use anyhow::anyhow;
use enum_dispatch::enum_dispatch;
use nautilus_common::log_info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

use crate::common::AlpacaDataFeed;


/// Abstract message handler interface.
#[enum_dispatch]
pub trait AlpacaAbstractMesssageHandler: 'static + Send {
    fn process_message(&mut self, raw: &[u8]) -> anyhow::Result<Option<Vec<u8>>>;
    fn is_authenticated(&self) -> bool;
}

#[derive(Debug)]
#[enum_dispatch(AlpacaAbstractMesssageHandler)]
pub enum AlpacaMessageHandler {
    Data(AlpacaDataMessageHandler),
    Trading(AlpacaTradingMessageHandler),
}

impl AlpacaMessageHandler {
    pub fn for_feed(feed_type: AlpacaDataFeed) -> Self {
        match feed_type {
            AlpacaDataFeed::Trading =>
                AlpacaTradingMessageHandler::new().into(),
            _ =>
                AlpacaDataMessageHandler::new().into(),
        }
    }
}


/// Type of WebSocket message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlpacaDataMessageType {
    /// Authentication success message.
    Success,
    /// Error message from server.
    Error,
    /// Subscription confirmation.
    Subscription,
    /// Trade data message.
    Trade,
    /// Quote data message.
    Quote,
    /// Bar data message.
    Bar,
    /// Unknown message type.
    Unknown,
}

impl AlpacaDataMessageType {
    /// Parse message type from the "T" field.
    pub fn from_str(s: &str) -> Self {
        match s {
            "success" => Self::Success,
            "error" => Self::Error,
            "subscription" => Self::Subscription,
            "t" => Self::Trade,
            "q" => Self::Quote,
            "b" => Self::Bar,
            _ => Self::Unknown,
        }
    }
}

/// Authentication request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaAuthMessage {
    /// Action type (always "auth").
    pub action: String,
    /// API key.
    pub key: String,
    /// API secret.
    pub secret: String,
}

impl AlpacaAuthMessage {
    /// Creates a new authentication message.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Alpaca API key
    /// * `api_secret` - Alpaca API secret
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            action: "auth".to_string(),
            key: api_key.into(),
            secret: api_secret.into(),
        }
    }
}

/// Subscription request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaDataSubscriptionMessage {
    /// Action type ("subscribe" or "unsubscribe").
    pub action: String,
    /// Trade symbols to subscribe to (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<String>>,
    /// Quote symbols to subscribe to (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<String>>,
    /// Bar symbols to subscribe to (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bars: Option<Vec<String>>,
}

impl AlpacaDataSubscriptionMessage {
    /// Creates a new subscription message.
    ///
    /// # Arguments
    ///
    /// * `action` - "subscribe" or "unsubscribe"
    /// * `trades` - Trade symbols
    /// * `quotes` - Quote symbols
    /// * `bars` - Bar symbols
    pub fn new(
        action: impl Into<String>,
        trades: Option<Vec<String>>,
        quotes: Option<Vec<String>>,
        bars: Option<Vec<String>>,
    ) -> Self {
        Self {
            action: action.into(),
            trades,
            quotes,
            bars,
        }
    }
}

/// Success message from server.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaDataSuccessMessage {
    /// Message type ("success").
    #[serde(rename = "T")]
    pub msg_type: String,
    /// Success message text.
    pub msg: String,
}

/// Error message from server.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaDataErrorMessage {
    /// Message type ("error").
    #[serde(rename = "T")]
    pub msg_type: String,
    /// Error message text.
    pub msg: String,
    /// Error code.
    pub code: i64,
}

/// Subscription confirmation message.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaDataSubscriptionConfirmation {
    /// Message type ("subscription").
    #[serde(rename = "T")]
    pub msg_type: String,
    /// Confirmed trade subscriptions.
    #[serde(default)]
    pub trades: Vec<String>,
    /// Confirmed quote subscriptions.
    #[serde(default)]
    pub quotes: Vec<String>,
    /// Confirmed bar subscriptions.
    #[serde(default)]
    pub bars: Vec<String>,
}

/// WebSocket message handler for Alpaca market data streams.
///
/// Handles:
/// - Message type routing
/// - Authentication tracking
/// - Subscription management
/// - Data message forwarding
#[derive(Debug)]
pub struct AlpacaDataMessageHandler {
    /// Whether the client is authenticated.
    is_authenticated: bool,
    /// Current trade subscriptions.
    subscribed_trades: HashSet<String>,
    /// Current quote subscriptions.
    subscribed_quotes: HashSet<String>,
    /// Current bar subscriptions.
    subscribed_bars: HashSet<String>,
}

impl Default for AlpacaDataMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AlpacaAbstractMesssageHandler for AlpacaDataMessageHandler {
    /// Processes a raw WebSocket message and routes it appropriately.
    ///
    /// Returns:
    /// - `Some(bytes)` if the message is market data and should be forwarded to the data handler
    /// - `None` if the message is a control message (success, error, subscription)
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw message bytes
    ///
    /// # Errors
    ///
    /// Returns an error if message processing fails
    fn process_message(&mut self, raw: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        let msg_type = self.parse_message_type(raw)?;

        match msg_type {
            AlpacaDataMessageType::Success => {
                log_info!("AlpacaDataMessageHandler.process_message: calling handle_success");
                self.handle_success(raw)?;
                Ok(None)
            }
            AlpacaDataMessageType::Error => {
                self.handle_error(raw)?;
                Ok(None)
            }
            AlpacaDataMessageType::Subscription => {
                self.handle_subscription(raw)?;
                Ok(None)
            }
            AlpacaDataMessageType::Trade | AlpacaDataMessageType::Quote | AlpacaDataMessageType::Bar => {
                // Forward market data messages to handler
                Ok(Some(raw.to_vec()))
            }
            AlpacaDataMessageType::Unknown => {
                log::warn!("Unknown message type, forwarding to handler");
                Ok(Some(raw.to_vec()))
            }
        }
    }


    /// Returns whether the client is authenticated.
    fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }

}

impl AlpacaDataMessageHandler {
    /// Creates a new message handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_authenticated: false,
            subscribed_trades: HashSet::new(),
            subscribed_quotes: HashSet::new(),
            subscribed_bars: HashSet::new(),
        }
    }

    /// Returns the current trade subscriptions.
    #[must_use]
    pub fn subscribed_trades(&self) -> &HashSet<String> {
        &self.subscribed_trades
    }

    /// Returns the current quote subscriptions.
    #[must_use]
    pub fn subscribed_quotes(&self) -> &HashSet<String> {
        &self.subscribed_quotes
    }

    /// Returns the current bar subscriptions.
    #[must_use]
    pub fn subscribed_bars(&self) -> &HashSet<String> {
        &self.subscribed_bars
    }

    /// Resets authentication state (used after reconnection).
    pub fn reset_authentication(&mut self) {
        self.is_authenticated = false;
    }

    /// Parses a raw WebSocket message and returns the message type.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw message bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed as JSON
    pub fn parse_message_type(&self, raw: &[u8]) -> anyhow::Result<AlpacaDataMessageType> {
        let value: Value = serde_json::from_slice(raw)?;

        if let Some(msg_type) = value.get("T").and_then(|t| t.as_str()) {
            return Ok(AlpacaDataMessageType::from_str(msg_type));
        }

        Ok(AlpacaDataMessageType::Unknown)
    }

    /// Handles a success message.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw message bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed
    pub fn handle_success(&mut self, raw: &[u8]) -> anyhow::Result<AlpacaDataSuccessMessage> {
        let msg: AlpacaDataSuccessMessage = serde_json::from_slice(raw)?;

        // Check for authentication success
        if msg.msg.to_lowercase().contains("authenticated") {
            self.is_authenticated = true;
            log::info!("Successfully authenticated");
        }

        Ok(msg)
    }

    /// Handles an error message.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw message bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed
    pub fn handle_error(&self, raw: &[u8]) -> anyhow::Result<AlpacaDataErrorMessage> {
        let msg: AlpacaDataErrorMessage = serde_json::from_slice(raw)?;
        log::error!("WebSocket error (code {}): {}", msg.code, msg.msg);
        Ok(msg)
    }

    /// Handles a subscription confirmation message.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw message bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed
    pub fn handle_subscription(
        &mut self,
        raw: &[u8],
    ) -> anyhow::Result<AlpacaDataSubscriptionConfirmation> {
        let msg: AlpacaDataSubscriptionConfirmation = serde_json::from_slice(raw)?;

        // TODO: AFAIK, this would also trigger as a response to a "unscubscribe",
        //       so logic for _removal_ of symbols is needed!
        //       If Alpaca always replies with a full list of all subscribed symbols,
        //       it could be as easy as clearing all subscriptions before re-inserting
        //       the symbols.

        // Update subscription state
        for symbol in &msg.trades {
            self.subscribed_trades.insert(symbol.clone());
        }
        for symbol in &msg.quotes {
            self.subscribed_quotes.insert(symbol.clone());
        }
        for symbol in &msg.bars {
            self.subscribed_bars.insert(symbol.clone());
        }

        log::info!(
            "Subscription confirmed - trades: {:?}, quotes: {:?}, bars: {:?}",
            msg.trades,
            msg.quotes,
            msg.bars
        );

        Ok(msg)
    }

    /// Clears subscription tracking (used after unsubscribe).
    ///
    /// # Arguments
    ///
    /// * `trades` - Trade symbols to unsubscribe
    /// * `quotes` - Quote symbols to unsubscribe
    /// * `bars` - Bar symbols to unsubscribe
    pub fn clear_subscriptions(
        &mut self,
        trades: Option<&[String]>,
        quotes: Option<&[String]>,
        bars: Option<&[String]>,
    ) {
        if let Some(symbols) = trades {
            for symbol in symbols {
                self.subscribed_trades.remove(symbol);
            }
        }
        if let Some(symbols) = quotes {
            for symbol in symbols {
                self.subscribed_quotes.remove(symbol);
            }
        }
        if let Some(symbols) = bars {
            for symbol in symbols {
                self.subscribed_bars.remove(symbol);
            }
        }
    }
}

/// Type of WebSocket message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlpacaTradingMessageType {
    /// Authorization success message.
    Authorization,
    /// Subscription confirmation.
    Subscription,
    /// Trade update message.
    TradeUpdate,
    /// Unknown message type.
    Unknown,
}

impl AlpacaTradingMessageType {
    /// Parse message type from the "T" field.
    pub fn from_str(s: &str) -> Self {
        match s {
            "authorization" => Self::Authorization,
            "listening" => Self::Subscription,
            "trade_updates" => Self::TradeUpdate,
            _ => Self::Unknown,
        }
    }
}

/// Subscription request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaTradingSubscriptionMessage {
    /// Action type ("listen").
    pub action: String,
    /// Message data.
    pub data: AlpacaTradingSubscriptionMessageData,
}

/// Subscription request message data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaTradingSubscriptionMessageData {
    /// Streams to subscripe to.
    pub streams: Vec<String>,
}

impl AlpacaTradingSubscriptionMessage {
    /// Creates a new subscription message.
    ///
    /// # Arguments
    ///
    /// * `streams` - List of streams
    pub fn new(
        streams: Vec<String>,
    ) -> Self {
        Self {
            action: "listen".to_string(),
            data: AlpacaTradingSubscriptionMessageData { streams },
        }
    }
}

/// Authorization message from server.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaTradingAuthorizationMessage {
    /// Action ("authenticate" or "auth").
    pub action: String,
    /// Status ("authorized" or "unauthorized")
    pub status: String,
    /// Optional message text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Subscription confirmation message.
#[derive(Debug, Clone, Deserialize)]
pub struct AlpacaTradingSubscriptionConfirmation {
    /// Confirmed stream subscriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streams: Option<Vec<String>>,
}

/// WebSocket message handler for Alpaca trading data streams.
///
/// Handles:
/// - Message type routing
/// - Authentication tracking
/// - Subscription management
/// - Data message forwarding
#[derive(Debug)]
pub struct AlpacaTradingMessageHandler {
    /// Whether the client is authenticated.
    is_authenticated: bool,
    /// Current stream subscriptions.
    subscribed_streams: HashSet<String>,
}

impl Default for AlpacaTradingMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AlpacaAbstractMesssageHandler for AlpacaTradingMessageHandler {
    /// Processes a raw WebSocket message and routes it appropriately.
    ///
    /// Returns:
    /// - `Some(bytes)` if the message is market data and should be forwarded to the data handler
    /// - `None` if the message is a control message (success, error, subscription)
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw message bytes
    ///
    /// # Errors
    ///
    /// Returns an error if message processing fails
    fn process_message(&mut self, raw: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        let msg: Value = serde_json::from_slice(raw)?;
        let msg_type = self.parse_message_type(&msg);

        match msg_type {
            AlpacaTradingMessageType::Authorization => {
                let data = msg.get("data").ok_or(anyhow!("missing `data` in AlpacaTradingAuthorizationMessage message"))?;
                self.handle_authorization(data)?;
                Ok(None)
            }
            AlpacaTradingMessageType::Subscription => {
                let data = msg.get("data").ok_or(anyhow!("missing `data` in AlpacaTradingSubscriptionConfirmation message"))?;
                self.handle_subscription(data)?;
                Ok(None)
            }
            AlpacaTradingMessageType::TradeUpdate => {
                // Forward data messages to handler
                Ok(Some(raw.to_vec()))
            }
            AlpacaTradingMessageType::Unknown => {
                log::warn!("Unknown message type, forwarding to handler");
                Ok(Some(raw.to_vec()))
            }
        }
    }

    /// Returns whether the client is authenticated.
    fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }
}

impl AlpacaTradingMessageHandler {
    /// Creates a new message handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_authenticated: false,
            subscribed_streams: HashSet::new(),
        }
    }

    /// Returns the current trade subscriptions.
    #[must_use]
    pub fn subscribed_streams(&self) -> &HashSet<String> {
        &self.subscribed_streams
    }

    /// Resets authentication state (used after reconnection).
    pub fn reset_authentication(&mut self) {
        self.is_authenticated = false;
    }

    /// Parses a WebSocket message and returns the message type.
    ///
    /// # Arguments
    ///
    /// * `msg` - Json Value
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed as JSON
    pub fn parse_message_type(&self, msg: &Value) -> AlpacaTradingMessageType {
        if let Some(msg_type) = msg.get("stream").and_then(|t| t.as_str()) {
            return AlpacaTradingMessageType::from_str(msg_type);
        }

        AlpacaTradingMessageType::Unknown
    }

    /// Handles a authorization message.
    ///
    /// # Arguments
    ///
    /// * `data` - Json value
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed
    pub fn handle_authorization(&mut self, data: &Value) -> anyhow::Result<AlpacaTradingAuthorizationMessage> {
        let msg: AlpacaTradingAuthorizationMessage = serde_json::from_value(data.clone())?;

        if msg.status == "authorized" {
            self.is_authenticated = true;
            log::info!("Successfully authorized");
        } else {
            self.is_authenticated = false;
            log::error!("Authorization failed");
        }

        Ok(msg)
    }

    /// Handles a subscription confirmation message.
    ///
    /// # Arguments
    ///
    /// * `data` - Json value
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed
    pub fn handle_subscription(
        &mut self,
        data: &Value,
    ) -> anyhow::Result<AlpacaTradingSubscriptionConfirmation> {
        let msg: AlpacaTradingSubscriptionConfirmation = serde_json::from_value(data.clone())?;

        if msg.streams.is_none() {
            self.clear_subscriptions();
        } else {
            // Update subscription state
            for stream in msg.streams.as_ref().unwrap() {
                self.subscribed_streams.insert(stream.clone());
            }
        }

        log::info!(
            "Subscription confirmed - streams: {:?}",
            msg.streams,
        );

        Ok(msg)
    }

    /// Clears all subscribed streams.
    pub fn clear_subscriptions(
        &mut self,
    ) {
        self.subscribed_streams = HashSet::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    fn test_auth_message_creation() {
        let auth = AlpacaAuthMessage::new("test_key", "test_secret");
        assert_eq!(auth.action, "auth");
        assert_eq!(auth.key, "test_key");
        assert_eq!(auth.secret, "test_secret");
    }

    #[rstest]
    fn test_subscription_message_creation() {
        let sub = AlpacaDataSubscriptionMessage::new(
            "subscribe",
            Some(vec!["AAPL".to_string()]),
            None,
            Some(vec!["SPY".to_string()]),
        );
        assert_eq!(sub.action, "subscribe");
        assert_eq!(sub.trades, Some(vec!["AAPL".to_string()]));
        assert_eq!(sub.quotes, None);
        assert_eq!(sub.bars, Some(vec!["SPY".to_string()]));
    }

    #[rstest]
    fn test_message_type_parsing() {
        let handler = AlpacaDataMessageHandler::new();

        let success_msg = r#"{"T":"success","msg":"authenticated"}"#;
        let msg_type = handler.parse_message_type(success_msg.as_bytes()).unwrap();
        assert_eq!(msg_type, AlpacaDataMessageType::Success);

        let error_msg = r#"{"T":"error","msg":"invalid credentials","code":401}"#;
        let msg_type = handler.parse_message_type(error_msg.as_bytes()).unwrap();
        assert_eq!(msg_type, AlpacaDataMessageType::Error);

        let trade_msg = r#"{"T":"t","S":"AAPL","p":150.25,"s":100}"#;
        let msg_type = handler.parse_message_type(trade_msg.as_bytes()).unwrap();
        assert_eq!(msg_type, AlpacaDataMessageType::Trade);
    }

    #[rstest]
    fn test_handle_success() {
        let mut handler = AlpacaDataMessageHandler::new();
        assert!(!handler.is_authenticated());

        let msg = r#"{"T":"success","msg":"authenticated"}"#;
        handler.handle_success(msg.as_bytes()).unwrap();
        assert!(handler.is_authenticated());
    }

    #[rstest]
    fn test_handle_subscription() {
        let mut handler = AlpacaDataMessageHandler::new();

        let msg = r#"{"T":"subscription","trades":["AAPL","MSFT"],"quotes":[],"bars":["SPY"]}"#;
        handler.handle_subscription(msg.as_bytes()).unwrap();

        assert!(handler.subscribed_trades.contains("AAPL"));
        assert!(handler.subscribed_trades.contains("MSFT"));
        assert!(handler.subscribed_bars.contains("SPY"));
        assert_eq!(handler.subscribed_quotes.len(), 0);
    }

    #[rstest]
    fn test_process_message_control() {
        let mut handler = AlpacaDataMessageHandler::new();

        // Success message should return None (control message)
        let success_msg = r#"{"T":"success","msg":"authenticated"}"#;
        let result = handler.process_message(success_msg.as_bytes()).unwrap();
        assert!(result.is_none());
        assert!(handler.is_authenticated());
    }

    #[rstest]
    fn test_process_message_data() {
        let mut handler = AlpacaDataMessageHandler::new();

        // Trade message should return Some (data message to forward)
        let trade_msg = r#"{"T":"t","S":"AAPL","p":150.25,"s":100}"#;
        let result = handler.process_message(trade_msg.as_bytes()).unwrap();
        assert!(result.is_some());
    }

    #[rstest]
    fn test_clear_subscriptions() {
        let mut handler = AlpacaDataMessageHandler::new();

        // Add some subscriptions
        handler.subscribed_trades.insert("AAPL".to_string());
        handler.subscribed_trades.insert("MSFT".to_string());
        handler.subscribed_bars.insert("SPY".to_string());

        // Clear some subscriptions
        handler.clear_subscriptions(
            Some(&["AAPL".to_string()]),
            None,
            Some(&["SPY".to_string()]),
        );

        assert!(!handler.subscribed_trades.contains("AAPL"));
        assert!(handler.subscribed_trades.contains("MSFT"));
        assert!(!handler.subscribed_bars.contains("SPY"));
    }
}
