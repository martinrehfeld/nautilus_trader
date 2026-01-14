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

//! Alpaca data client implementation.
//!
//! This module provides the `AlpacaDataClient` for subscribing to real-time market data
//! and requesting historical data from Alpaca Markets.

pub mod types;

use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use nautilus_common::cache::Cache;
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::{Bar, BarType, QuoteTick, TradeTick},
    identifiers::InstrumentId,
    instruments::InstrumentAny,
};
use serde_json::Value;
use tokio::sync::RwLock;
use log::{debug, info, warn};

use crate::{
    common::{
        enums::{AlpacaAssetClass, AlpacaDataFeed},
        models::{AlpacaWsBar, AlpacaWsQuote, AlpacaWsTrade},
        parse::{create_bar_type, current_timestamp_ns, parse_bar, parse_quote_tick, parse_trade_tick},
    },
    error::{AlpacaError, Result},
    http::client::AlpacaHttpClient,
    websocket::client::AlpacaWebSocketClient,
};

use self::types::{AlpacaDataClientConfig, SubscriptionState};

/// Alpaca data client for market data streaming and historical data requests.
///
/// Supports three asset classes:
/// - **US Equities**: Stocks and ETFs with IEX or SIP data feeds
/// - **Crypto**: Cryptocurrency pairs (e.g., BTC/USD, ETH/USD)
/// - **Options**: Equity options contracts
///
/// # Features
///
/// - Real-time streaming via WebSocket (trades, quotes, bars)
/// - Historical data requests via REST API
/// - Asset class-specific feed management
/// - Automatic reconnection and error handling
#[derive(Debug)]
pub struct AlpacaDataClient {
    /// HTTP client for REST API requests.
    http_client: Arc<AlpacaHttpClient>,
    /// WebSocket clients (one per asset class).
    ws_clients: Arc<RwLock<HashMap<AlpacaAssetClass, AlpacaWebSocketClient>>>,
    /// Data feed configuration.
    _data_feed: AlpacaDataFeed,
    /// Subscription state tracking.
    subscriptions: Arc<RwLock<SubscriptionState>>,
    /// Instrument cache.
    cache: Arc<Cache>,
}

impl AlpacaDataClient {
    /// Creates a new Alpaca data client.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    /// * `cache` - Instrument and data cache
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client fails to initialize.
    pub fn new(config: AlpacaDataClientConfig, cache: Arc<Cache>) -> Result<Self> {
        let environment = if config.paper_trading {
            crate::common::enums::AlpacaEnvironment::Paper
        } else {
            crate::common::enums::AlpacaEnvironment::Live
        };

        let http_client = Arc::new(AlpacaHttpClient::new(
            environment,
            config.api_key.clone(),
            config.api_secret.clone(),
            config.timeout_secs,
            config.proxy_url.clone(),
        )?);

        // Create WebSocket clients for each asset class
        let mut ws_clients = HashMap::new();
        for asset_class in [
            AlpacaAssetClass::UsEquity,
            AlpacaAssetClass::Crypto,
            AlpacaAssetClass::Option,
        ] {
            let ws_client = AlpacaWebSocketClient::new(
                config.api_key.clone(),
                config.api_secret.clone(),
                asset_class,
                config.data_feed,
                None, // No URL override
            );
            ws_clients.insert(asset_class, ws_client);
        }

        Ok(Self {
            http_client,
            ws_clients: Arc::new(RwLock::new(ws_clients)),
            _data_feed: config.data_feed,
            subscriptions: Arc::new(RwLock::new(SubscriptionState::new())),
            cache,
        })
    }

    // 
    // Subscription Methods
    // 

    /// Subscribes to trade ticks for an instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument to subscribe to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Instrument not found in cache
    /// - WebSocket connection fails
    /// - Subscription message fails to send
    pub async fn subscribe_trades(&self, instrument_id: InstrumentId) -> Result<()> {
        let instrument = self.get_instrument(&instrument_id)?;
        let symbol = instrument_id.symbol;
        let asset_class = self.determine_asset_class(&instrument);

        // Register instrument
        {
            let mut subs = self.subscriptions.write().await;
            subs.register_instrument(instrument_id, symbol, asset_class);
        }

        // Ensure WebSocket is connected
        self.ensure_ws_connected(asset_class).await?;

        // Subscribe to trades
        let ws_clients = self.ws_clients.read().await;
        if let Some(ws_client) = ws_clients.get(&asset_class) {
            let msg = AlpacaWebSocketClient::subscribe_trades_message(vec![symbol.to_string()]);
            // TODO: Send message via WebSocket (needs integration with WebSocket client)
            debug!("Subscribe trades message: {}", msg);

            let mut subs = self.subscriptions.write().await;
            subs.add_trade(asset_class, symbol.to_string());
            info!("Subscribed to trades for {}", instrument_id);
        }

        Ok(())
    }

    /// Subscribes to quote ticks for an instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument to subscribe to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Instrument not found in cache
    /// - WebSocket connection fails
    /// - Subscription message fails to send
    pub async fn subscribe_quotes(&self, instrument_id: InstrumentId) -> Result<()> {
        let instrument = self.get_instrument(&instrument_id)?;
        let symbol = instrument_id.symbol;
        let asset_class = self.determine_asset_class(&instrument);

        // Register instrument
        {
            let mut subs = self.subscriptions.write().await;
            subs.register_instrument(instrument_id, symbol, asset_class);
        }

        // Ensure WebSocket is connected
        self.ensure_ws_connected(asset_class).await?;

        // Subscribe to quotes
        let ws_clients = self.ws_clients.read().await;
        if let Some(ws_client) = ws_clients.get(&asset_class) {
            let msg = AlpacaWebSocketClient::subscribe_quotes_message(vec![symbol.to_string()]);
            // TODO: Send message via WebSocket (needs integration with WebSocket client)
            debug!("Subscribe quotes message: {}", msg);

            let mut subs = self.subscriptions.write().await;
            subs.add_quote(asset_class, symbol.to_string());
            info!("Subscribed to quotes for {}", instrument_id);
        }

        Ok(())
    }

    /// Subscribes to bars for an instrument.
    ///
    /// # Arguments
    ///
    /// * `bar_type` - The bar type to subscribe to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Instrument not found in cache
    /// - WebSocket connection fails
    /// - Subscription message fails to send
    /// - Bar aggregation is not supported (only 1-minute bars via WebSocket)
    pub async fn subscribe_bars(&self, bar_type: BarType) -> Result<()> {
        let instrument_id = bar_type.instrument_id();
        let instrument = self.get_instrument(&instrument_id)?;
        let symbol = instrument_id.symbol;
        let asset_class = self.determine_asset_class(&instrument);

        // WebSocket only supports 1-minute bars
        if bar_type.spec().step != NonZeroUsize::new(1).unwrap() || bar_type.spec().aggregation != nautilus_model::enums::BarAggregation::Minute {
            return Err(AlpacaError::InvalidRequest(
                "WebSocket bars only support 1-minute aggregation".to_string(),
            ));
        }

        // Register instrument
        {
            let mut subs = self.subscriptions.write().await;
            subs.register_instrument(instrument_id, symbol, asset_class);
        }

        // Ensure WebSocket is connected
        self.ensure_ws_connected(asset_class).await?;

        // Subscribe to bars
        let ws_clients = self.ws_clients.read().await;
        if let Some(ws_client) = ws_clients.get(&asset_class) {
            let msg = AlpacaWebSocketClient::subscribe_bars_message(vec![symbol.to_string()]);
            // TODO: Send message via WebSocket (needs integration with WebSocket client)
            debug!("Subscribe bars message: {}", msg);

            let mut subs = self.subscriptions.write().await;
            subs.add_bar(asset_class, symbol.to_string());
            info!("Subscribed to bars for {}", bar_type);
        }

        Ok(())
    }

    /// Unsubscribes from trade ticks for an instrument.
    pub async fn unsubscribe_trades(&self, instrument_id: InstrumentId) -> Result<()> {
        let (asset_class, symbol) = {
            let subs = self.subscriptions.read().await;
            let asset_class = subs
                .get_asset_class(&instrument_id)
                .ok_or_else(|| AlpacaError::NotFound(format!("No subscription for {}", instrument_id)))?;
            let symbol = subs
                .get_symbol(&instrument_id)
                .ok_or_else(|| AlpacaError::NotFound(format!("Symbol not found for {}", instrument_id)))?
                .clone();
            (asset_class, symbol)
        };

        let ws_clients = self.ws_clients.read().await;
        if let Some(ws_client) = ws_clients.get(&asset_class) {
            let msg = AlpacaWebSocketClient::unsubscribe_trades_message(vec![symbol.to_string()]);
            // TODO: Send message via WebSocket (needs integration with WebSocket client)
            debug!("Unsubscribe trades message: {}", msg);

            let mut subs = self.subscriptions.write().await;
            subs.remove_trade(asset_class, &symbol.to_string());
            info!("Unsubscribed from trades for {}", instrument_id);
        }

        Ok(())
    }

    /// Unsubscribes from quote ticks for an instrument.
    pub async fn unsubscribe_quotes(&self, instrument_id: InstrumentId) -> Result<()> {
        let (asset_class, symbol) = {
            let subs = self.subscriptions.read().await;
            let asset_class = subs
                .get_asset_class(&instrument_id)
                .ok_or_else(|| AlpacaError::NotFound(format!("No subscription for {}", instrument_id)))?;
            let symbol = subs
                .get_symbol(&instrument_id)
                .ok_or_else(|| AlpacaError::NotFound(format!("Symbol not found for {}", instrument_id)))?
                .clone();
            (asset_class, symbol)
        };

        let ws_clients = self.ws_clients.read().await;
        if let Some(ws_client) = ws_clients.get(&asset_class) {
            let msg = AlpacaWebSocketClient::unsubscribe_quotes_message(vec![symbol.to_string()]);
            // TODO: Send message via WebSocket (needs integration with WebSocket client)
            debug!("Unsubscribe quotes message: {}", msg);

            let mut subs = self.subscriptions.write().await;
            subs.remove_quote(asset_class, &symbol.to_string());
            info!("Unsubscribed from quotes for {}", instrument_id);
        }

        Ok(())
    }

    /// Unsubscribes from bars for an instrument.
    pub async fn unsubscribe_bars(&self, bar_type: BarType) -> Result<()> {
        let instrument_id = bar_type.instrument_id();
        let (asset_class, symbol) = {
            let subs = self.subscriptions.read().await;
            let asset_class = subs
                .get_asset_class(&instrument_id)
                .ok_or_else(|| AlpacaError::NotFound(format!("No subscription for {}", instrument_id)))?;
            let symbol = subs
                .get_symbol(&instrument_id)
                .ok_or_else(|| AlpacaError::NotFound(format!("Symbol not found for {}", instrument_id)))?
                .clone();
            (asset_class, symbol)
        };

        let ws_clients = self.ws_clients.read().await;
        if let Some(ws_client) = ws_clients.get(&asset_class) {
            let msg = AlpacaWebSocketClient::unsubscribe_bars_message(vec![symbol.to_string()]);
            // TODO: Send message via WebSocket (needs integration with WebSocket client)
            debug!("Unsubscribe bars message: {}", msg);

            let mut subs = self.subscriptions.write().await;
            subs.remove_bar(asset_class, &symbol.to_string());
            info!("Unsubscribed from bars for {}", bar_type);
        }

        Ok(())
    }

    // 
    // Historical Data Request Methods
    // 

    /// Requests historical bars from Alpaca.
    ///
    /// # Arguments
    ///
    /// * `bar_type` - The bar type specification
    /// * `start` - Start timestamp (optional)
    /// * `end` - End timestamp (optional)
    /// * `limit` - Maximum number of bars (default: 1000, max: 10000)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Instrument not found in cache
    /// - HTTP request fails
    /// - Response parsing fails
    pub async fn request_bars(
        &self,
        bar_type: BarType,
        start: Option<UnixNanos>,
        end: Option<UnixNanos>,
        limit: Option<u32>,
    ) -> Result<Vec<Bar>> {
        let instrument_id = bar_type.instrument_id();
        let instrument = self.get_instrument(&instrument_id)?;
        let symbol = instrument_id.symbol.to_string();
        let asset_class = self.determine_asset_class(&instrument);

        // Convert bar type to Alpaca timeframe
        let timeframe = self.bar_type_to_timeframe(&bar_type)?;

        // Format timestamps
        let start_str = start.map(|ts| self.format_timestamp(ts));
        let end_str = end.map(|ts| self.format_timestamp(ts));

        // Make request based on asset class
        let response = match asset_class {
            AlpacaAssetClass::UsEquity => {
                self.http_client
                    .get_stock_bars(&symbol, &timeframe, start_str.as_deref(), end_str.as_deref(), limit)
                    .await?
            }
            AlpacaAssetClass::Crypto => {
                self.http_client
                    .get_crypto_bars(&symbol, &timeframe, start_str.as_deref(), end_str.as_deref(), limit)
                    .await?
            }
            AlpacaAssetClass::Option => {
                return Err(AlpacaError::NotImplemented(
                    "Historical bars for options not yet implemented".to_string(),
                ));
            }
        };

        // Parse response
        let bars = self.parse_bars_response(response, bar_type, &instrument)?;
        info!("Retrieved {} bars for {}", bars.len(), bar_type);

        Ok(bars)
    }

    /// Requests historical trade ticks from Alpaca.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument to request trades for
    /// * `start` - Start timestamp (optional)
    /// * `end` - End timestamp (optional)
    /// * `limit` - Maximum number of trades (default: 1000, max: 10000)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Instrument not found in cache
    /// - HTTP request fails
    /// - Response parsing fails
    pub async fn request_trades(
        &self,
        instrument_id: InstrumentId,
        start: Option<UnixNanos>,
        end: Option<UnixNanos>,
        limit: Option<u32>,
    ) -> Result<Vec<TradeTick>> {
        let instrument = self.get_instrument(&instrument_id)?;
        let symbol = instrument_id.symbol.to_string();
        let asset_class = self.determine_asset_class(&instrument);

        // Format timestamps
        let start_str = start.map(|ts| self.format_timestamp(ts));
        let end_str = end.map(|ts| self.format_timestamp(ts));

        // Make request based on asset class
        let response = match asset_class {
            AlpacaAssetClass::UsEquity => {
                self.http_client
                    .get_stock_trades(&symbol, start_str.as_deref(), end_str.as_deref(), limit)
                    .await?
            }
            AlpacaAssetClass::Crypto => {
                self.http_client
                    .get_crypto_trades(&symbol, start_str.as_deref(), end_str.as_deref(), limit)
                    .await?
            }
            AlpacaAssetClass::Option => {
                return Err(AlpacaError::NotImplemented(
                    "Historical trades for options not yet implemented".to_string(),
                ));
            }
        };

        // Parse response
        let trades = self.parse_trades_response(response, instrument_id, &instrument)?;
        info!("Retrieved {} trades for {}", trades.len(), instrument_id);

        Ok(trades)
    }

    /// Requests historical quote ticks from Alpaca.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument to request quotes for
    /// * `start` - Start timestamp (optional)
    /// * `end` - End timestamp (optional)
    /// * `limit` - Maximum number of quotes (default: 1000, max: 10000)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Instrument not found in cache
    /// - HTTP request fails
    /// - Response parsing fails
    pub async fn request_quotes(
        &self,
        instrument_id: InstrumentId,
        start: Option<UnixNanos>,
        end: Option<UnixNanos>,
        limit: Option<u32>,
    ) -> Result<Vec<QuoteTick>> {
        let instrument = self.get_instrument(&instrument_id)?;
        let symbol = instrument_id.symbol.to_string();
        let asset_class = self.determine_asset_class(&instrument);

        // Format timestamps
        let start_str = start.map(|ts| self.format_timestamp(ts));
        let end_str = end.map(|ts| self.format_timestamp(ts));

        // Make request based on asset class
        let response = match asset_class {
            AlpacaAssetClass::UsEquity => {
                self.http_client
                    .get_stock_quotes(&symbol, start_str.as_deref(), end_str.as_deref(), limit)
                    .await?
            }
            AlpacaAssetClass::Crypto => {
                return Err(AlpacaError::NotImplemented(
                    "Historical quotes for crypto not yet implemented".to_string(),
                ));
            }
            AlpacaAssetClass::Option => {
                return Err(AlpacaError::NotImplemented(
                    "Historical quotes for options not yet implemented".to_string(),
                ));
            }
        };

        // Parse response
        let quotes = self.parse_quotes_response(response, instrument_id, &instrument)?;
        info!("Retrieved {} quotes for {}", quotes.len(), instrument_id);

        Ok(quotes)
    }

    // 
    // WebSocket Message Handling
    // 

    /// Handles incoming WebSocket messages.
    ///
    /// This method should be called by the WebSocket handler when messages arrive.
    /// It parses the message and routes it to the appropriate handler.
    pub async fn handle_ws_message(&self, raw: &[u8]) -> Result<()> {
        // Parse JSON message
        let msg: Value = serde_json::from_slice(raw)
            .map_err(|e| AlpacaError::ParseError(format!("Failed to parse WebSocket message: {}", e)))?;

        // Get message type
        let msg_type = msg
            .get("T")
            .and_then(|t| t.as_str())
            .ok_or_else(|| AlpacaError::ParseError("Message missing 'T' field".to_string()))?;

        // Route based on message type
        match msg_type {
            "t" => self.handle_trade_message(msg).await,
            "q" => self.handle_quote_message(msg).await,
            "b" => self.handle_bar_message(msg).await,
            "subscription" | "success" | "error" => {
                debug!("Control message: {:?}", msg);
                Ok(())
            }
            _ => {
                warn!("Unknown message type: {}", msg_type);
                Ok(())
            }
        }
    }

    async fn handle_trade_message(&self, msg: Value) -> Result<()> {
        // Deserialize to AlpacaWsTrade
        let trade: AlpacaWsTrade = serde_json::from_value(msg)
            .map_err(|e| AlpacaError::ParseError(format!("Failed to parse trade: {}", e)))?;

        // Find instrument
        let instrument_id = self.find_instrument_id(&trade.symbol).await?;
        let instrument = self.get_instrument(&instrument_id)?;
        let (price_precision, size_precision) = self.get_precisions(&instrument);

        // Parse to TradeTick
        let ts_init = current_timestamp_ns();
        let trade_tick = parse_trade_tick(&trade, instrument_id, price_precision, size_precision, ts_init)?;

        // TODO: Send to data engine
        debug!("Parsed trade tick: {:?}", trade_tick);

        Ok(())
    }

    async fn handle_quote_message(&self, msg: Value) -> Result<()> {
        // Deserialize to AlpacaWsQuote
        let quote: AlpacaWsQuote = serde_json::from_value(msg)
            .map_err(|e| AlpacaError::ParseError(format!("Failed to parse quote: {}", e)))?;

        // Find instrument
        let instrument_id = self.find_instrument_id(&quote.symbol).await?;
        let instrument = self.get_instrument(&instrument_id)?;
        let (price_precision, size_precision) = self.get_precisions(&instrument);

        // Parse to QuoteTick
        let ts_init = current_timestamp_ns();
        let quote_tick = parse_quote_tick(&quote, instrument_id, price_precision, size_precision, ts_init)?;

        // TODO: Send to data engine
        debug!("Parsed quote tick: {:?}", quote_tick);

        Ok(())
    }

    async fn handle_bar_message(&self, msg: Value) -> Result<()> {
        // Deserialize to AlpacaWsBar
        let bar_data: AlpacaWsBar = serde_json::from_value(msg)
            .map_err(|e| AlpacaError::ParseError(format!("Failed to parse bar: {}", e)))?;

        // Find instrument
        let instrument_id = self.find_instrument_id(&bar_data.symbol).await?;
        let instrument = self.get_instrument(&instrument_id)?;
        let (price_precision, size_precision) = self.get_precisions(&instrument);

        // Create bar type (assuming 1-minute bars)
        let bar_type = create_bar_type(instrument_id, "1Min")?;

        // Parse to Bar
        let ts_init = current_timestamp_ns();
        let bar = parse_bar(&bar_data, bar_type, price_precision, size_precision, ts_init)?;

        // TODO: Send to data engine
        debug!("Parsed bar: {:?}", bar);

        Ok(())
    }

    // 
    // Helper Methods
    // 

    async fn ensure_ws_connected(&self, asset_class: AlpacaAssetClass) -> Result<()> {
        let ws_clients = self.ws_clients.read().await;
        if let Some(ws_client) = ws_clients.get(&asset_class) {
            if !ws_client.is_connected() {
                info!("Connecting WebSocket for {:?}", asset_class);
                // TODO: Implement actual connection logic
                // This would involve starting a WebSocket task and handling authentication
            }
        }
        Ok(())
    }

    fn get_instrument(&self, instrument_id: &InstrumentId) -> Result<InstrumentAny> {
        self.cache
            .instrument(instrument_id)
            .cloned()
            .ok_or_else(|| AlpacaError::NotFound(format!("Instrument not found: {}", instrument_id)))
    }

    fn determine_asset_class(&self, instrument: &InstrumentAny) -> AlpacaAssetClass {
        match instrument {
            InstrumentAny::CurrencyPair(_) => AlpacaAssetClass::Crypto,
            InstrumentAny::OptionContract(_) => AlpacaAssetClass::Option,
            _ => AlpacaAssetClass::UsEquity,
        }
    }

    fn get_precisions(&self, instrument: &InstrumentAny) -> (u8, u8) {
        match instrument {
            InstrumentAny::Equity(e) => (e.price_precision, 0),
            InstrumentAny::CurrencyPair(c) => (c.price_precision, c.size_precision),
            InstrumentAny::OptionContract(o) => (o.price_precision, o.size_precision),
            _ => (2, 0), // Default
        }
    }

    async fn find_instrument_id(&self, symbol: &str) -> Result<InstrumentId> {
        let subs = self.subscriptions.read().await;
        for (instrument_id, sym) in &subs.instruments {
            if sym.to_string() == symbol {
                return Ok(*instrument_id);
            }
        }
        Err(AlpacaError::NotFound(format!("No subscription for symbol: {}", symbol)))
    }

    fn bar_type_to_timeframe(&self, bar_type: &BarType) -> Result<String> {
        let spec = bar_type.spec();
        let step = spec.step;
        let aggregation = spec.aggregation;

        use nautilus_model::enums::BarAggregation;
        match aggregation {
            BarAggregation::Minute => Ok(format!("{}Min", step)),
            BarAggregation::Hour => Ok(format!("{}Hour", step)),
            BarAggregation::Day => Ok(format!("{}Day", step)),
            BarAggregation::Week => Ok(format!("{}Week", step)),
            BarAggregation::Month => Ok(format!("{}Month", step)),
            _ => Err(AlpacaError::InvalidRequest(format!(
                "Unsupported bar aggregation: {:?}",
                aggregation
            ))),
        }
    }

    fn format_timestamp(&self, ts: UnixNanos) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp((ts.as_u64() / 1_000_000_000) as i64, 0)
            .expect("Valid timestamp");
        dt.to_rfc3339()
    }

    fn parse_bars_response(
        &self,
        response: crate::http::models::AlpacaBarsResponse,
        bar_type: BarType,
        instrument: &InstrumentAny,
    ) -> Result<Vec<Bar>> {
        let (price_precision, size_precision) = self.get_precisions(instrument);
        let ts_init = current_timestamp_ns();

        let mut bars = Vec::new();
        for bar_data in &response.bars {
            // Convert HTTP response bar to WebSocket bar format for parsing
            let ws_bar = AlpacaWsBar {
                msg_type: "b".to_string(),
                symbol: bar_type.instrument_id().symbol.to_string(),
                open: bar_data.o,
                high: bar_data.h,
                low: bar_data.l,
                close: bar_data.c,
                volume: bar_data.v as u64,
                timestamp: bar_data.t.clone(),
                trade_count: bar_data.n.map(|n| n as u64),
                vwap: bar_data.vw,
            };

            let bar = parse_bar(&ws_bar, bar_type, price_precision, size_precision, ts_init)?;
            bars.push(bar);
        }

        Ok(bars)
    }

    fn parse_trades_response(
        &self,
        response: crate::http::models::AlpacaTradesResponse,
        instrument_id: InstrumentId,
        instrument: &InstrumentAny,
    ) -> Result<Vec<TradeTick>> {
        let (price_precision, size_precision) = self.get_precisions(instrument);
        let ts_init = current_timestamp_ns();

        let mut trades = Vec::new();
        for trade_data in &response.trades {
            // Convert HTTP response trade to WebSocket trade format for parsing
            let ws_trade = AlpacaWsTrade {
                msg_type: "t".to_string(),
                symbol: instrument_id.symbol.to_string(),
                trade_id: trade_data.i.map(|id| id as u64),
                exchange: trade_data.x.clone(),
                price: trade_data.p,
                size: trade_data.s as u64,
                timestamp: trade_data.t.clone(),
                conditions: trade_data.c.clone(),
                tape: trade_data.z.clone(),
            };

            let trade = parse_trade_tick(&ws_trade, instrument_id, price_precision, size_precision, ts_init)?;
            trades.push(trade);
        }

        Ok(trades)
    }

    fn parse_quotes_response(
        &self,
        response: crate::http::models::AlpacaQuotesResponse,
        instrument_id: InstrumentId,
        instrument: &InstrumentAny,
    ) -> Result<Vec<QuoteTick>> {
        let (price_precision, size_precision) = self.get_precisions(instrument);
        let ts_init = current_timestamp_ns();

        let mut quotes = Vec::new();
        for quote_data in &response.quotes {
            // Convert HTTP response quote to WebSocket quote format for parsing
            let ws_quote = AlpacaWsQuote {
                msg_type: "q".to_string(),
                symbol: instrument_id.symbol.to_string(),
                ask_exchange: quote_data.ax.clone(),
                ask_price: quote_data.ap,
                ask_size: quote_data.as_ as u64,
                bid_exchange: quote_data.bx.clone(),
                bid_price: quote_data.bp,
                bid_size: quote_data.bs as u64,
                timestamp: quote_data.t.clone(),
                conditions: quote_data.c.clone(),
                tape: quote_data.z.clone(),
            };

            let quote = parse_quote_tick(&ws_quote, instrument_id, price_precision, size_precision, ts_init)?;
            quotes.push(quote);
        }

        Ok(quotes)
    }
}

#[cfg(test)]
mod tests {
    // Tests would go here - need to mock HTTP client and cache
}
