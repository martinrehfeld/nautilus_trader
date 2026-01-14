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

//! Execution client for Alpaca Markets.
//!
//! Provides order submission, cancellation, modification, and position management
//! for US equities, cryptocurrencies, and options.

pub mod account;
pub mod reports;
pub mod types;

use std::collections::HashMap;

use nautilus_core::time::get_atomic_clock_realtime;
use nautilus_model::{
    identifiers::{AccountId, ClientId, ClientOrderId, InstrumentId, VenueOrderId},
    instruments::{Instrument, InstrumentAny},
    orders::{Order, OrderAny},
};
use rust_decimal::Decimal;
use log::{debug, info, warn};

use self::{
    account::{AlpacaAccountState, PositionState},
    reports::{parse_fill_report, parse_order_status_report, parse_position_status_report},
    types::{AlpacaOrderBuilder, MultiLegOrderBuilder},
};
use crate::{
    common::{consts::ALPACA_VENUE, enums::AlpacaEnvironment},
    error::{AlpacaError, Result},
    http::{client::AlpacaHttpClient, models::*},
};

/// Configuration for the Alpaca execution client.
#[derive(Debug, Clone)]
pub struct AlpacaExecClientConfig {
    /// API key for authentication.
    pub api_key: String,
    /// API secret for authentication.
    pub api_secret: String,
    /// Trading environment (Live or Paper).
    pub environment: AlpacaEnvironment,
    /// Request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Proxy URL (optional).
    pub proxy_url: Option<String>,
}

/// Alpaca execution client.
///
/// Supports:
/// - US equities (stocks and ETFs)
/// - Cryptocurrencies (BTC/USD, ETH/USD, etc.)
/// - Options (equity options)
///
/// # Crypto-specific constraints
///
/// - Time-in-force must be GTC or IOC
/// - No shorting allowed
/// - No margin trading
///
/// # Options-specific constraints
///
/// - Only day orders supported
/// - Contract multiplier typically 100
#[derive(Debug)]
pub struct AlpacaExecutionClient {
    /// Client ID.
    client_id: ClientId,
    /// Account ID.
    account_id: AccountId,
    /// HTTP client.
    http_client: AlpacaHttpClient,
    /// Account state.
    account_state: Option<AlpacaAccountState>,
    /// Instrument cache (instrument_id -> instrument).
    instruments: HashMap<InstrumentId, InstrumentAny>,
    /// Configuration.
    config: AlpacaExecClientConfig,
}

impl AlpacaExecutionClient {
    /// Creates a new Alpaca execution client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client fails to initialize.
    pub fn new(config: AlpacaExecClientConfig) -> Result<Self> {
        let http_client = AlpacaHttpClient::new(
            config.environment,
            config.api_key.clone(),
            config.api_secret.clone(),
            config.timeout_secs,
            config.proxy_url.clone(),
        )?;

        let client_id = ClientId::from(ALPACA_VENUE);
        let account_id = AccountId::from(format!("{}-master", ALPACA_VENUE).as_str());

        info!("Alpaca execution client initialized");
        info!("Environment: {:?}", config.environment);

        Ok(Self {
            client_id,
            account_id,
            http_client,
            account_state: None,
            instruments: HashMap::new(),
            config,
        })
    }

    /// Connects to the Alpaca API and initializes account state.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Account information cannot be fetched
    /// - Account state initialization fails
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Alpaca...");

        // Fetch account information
        let account_data = self.http_client.get_account().await?;
        info!("Connected to Alpaca account: {}", account_data.id);
        info!("Account equity: ${}", account_data.equity);
        info!("Buying power: ${}", account_data.buying_power);

        // Initialize account state
        self.account_state = Some(AlpacaAccountState::from_alpaca_account(
            &account_data,
            self.account_id,
        ));

        info!("Alpaca execution client connected");
        Ok(())
    }

    /// Disconnects from the Alpaca API.
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Alpaca execution client disconnected");
        Ok(())
    }

    /// Registers an instrument in the client's cache.
    pub fn register_instrument(&mut self, instrument: InstrumentAny) {
        self.instruments.insert(instrument.id(), instrument);
    }

    /// Gets an instrument from the cache.
    fn get_instrument(&self, instrument_id: &InstrumentId) -> Option<&InstrumentAny> {
        self.instruments.get(instrument_id)
    }

    // 
    // Order Submission
    // 

    /// Submits an order to Alpaca.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Order validation fails
    /// - Instrument not found
    /// - HTTP request fails
    pub async fn submit_order(&mut self, order: OrderAny) -> Result<VenueOrderId> {
        info!("Submitting order: {}", order.client_order_id());

        // Get instrument
        let instrument = self
            .get_instrument(&order.instrument_id())
            .ok_or_else(|| AlpacaError::InstrumentNotFound(order.instrument_id().to_string()))?
            .clone();

        // Build order request
        let builder = AlpacaOrderBuilder::new(order.clone(), instrument);
        let order_request = builder.build()?;

        // Submit order
        let response = self.http_client.submit_order(&order_request).await?;

        let venue_order_id = VenueOrderId::from(response.id.as_str());
        info!(
            "Order {} accepted, venue_order_id={}",
            order.client_order_id(),
            venue_order_id
        );

        Ok(venue_order_id)
    }

    /// Submits a multi-leg options order.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Leg validation fails
    /// - HTTP request fails
    pub async fn submit_multi_leg_order(
        &mut self,
        builder: MultiLegOrderBuilder,
    ) -> Result<VenueOrderId> {
        info!("Submitting multi-leg order");

        let order_request = builder.build()?;
        let response = self.http_client.submit_order(&order_request).await?;

        let venue_order_id = VenueOrderId::from(response.id.as_str());
        info!("Multi-leg order accepted, venue_order_id={}", venue_order_id);

        Ok(venue_order_id)
    }

    // 
    // Order Modification
    // 

    /// Modifies an existing order.
    ///
    /// Only LIMIT orders can be modified.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Order is not a LIMIT order
    /// - Order is not in a modifiable state
    /// - HTTP request fails
    pub async fn modify_order(
        &mut self,
        venue_order_id: VenueOrderId,
        quantity: Option<Decimal>,
        limit_price: Option<Decimal>,
    ) -> Result<()> {
        info!("Modifying order: {}", venue_order_id);

        let mut body = serde_json::Map::new();

        if let Some(qty) = quantity {
            body.insert("qty".to_string(), serde_json::Value::String(qty.to_string()));
        }

        if let Some(price) = limit_price {
            body.insert(
                "limit_price".to_string(),
                serde_json::Value::String(price.to_string()),
            );
        }

        if body.is_empty() {
            warn!("No modifications specified for order {}", venue_order_id);
            return Ok(());
        }

        self.http_client
            .modify_order(&venue_order_id.to_string(), &body)
            .await?;

        info!("Order {} modified", venue_order_id);
        Ok(())
    }

    // 
    // Order Cancellation
    // 

    /// Cancels an order.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn cancel_order(&mut self, venue_order_id: VenueOrderId) -> Result<()> {
        info!("Canceling order: {}", venue_order_id);

        self.http_client
            .cancel_order(&venue_order_id.to_string())
            .await?;

        info!("Order {} canceled", venue_order_id);
        Ok(())
    }

    /// Cancels an order by client order ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn cancel_order_by_client_id(
        &mut self,
        client_order_id: ClientOrderId,
    ) -> Result<()> {
        info!("Canceling order by client_order_id: {}", client_order_id);

        self.http_client
            .cancel_order_by_client_id(&client_order_id.to_string())
            .await?;

        info!("Order {} canceled", client_order_id);
        Ok(())
    }

    /// Cancels all open orders.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn cancel_all_orders(&mut self) -> Result<Vec<AlpacaCancelStatus>> {
        info!("Canceling all orders");

        let statuses = self.http_client.cancel_all_orders().await?;

        info!("All orders canceled");
        Ok(statuses)
    }

    // 
    // Position Management
    // 

    /// Closes a position.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn close_position(&mut self, symbol: &str) -> Result<AlpacaOrder> {
        info!("Closing position: {}", symbol);

        let order = self.http_client.close_position(symbol, None, None).await?;

        info!("Position {} closed", symbol);
        Ok(order)
    }

    /// Closes all positions.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn close_all_positions(
        &mut self,
        cancel_orders: bool,
    ) -> Result<Vec<AlpacaOrder>> {
        info!("Closing all positions (cancel_orders={})", cancel_orders);

        let orders = self
            .http_client
            .close_all_positions(cancel_orders)
            .await?;

        info!("All positions closed");
        Ok(orders)
    }

    // 
    // Reports Generation
    // 

    /// Generates an order status report for a specific order.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Order not found
    /// - HTTP request fails
    /// - Report parsing fails
    pub async fn generate_order_status_report(
        &self,
        venue_order_id: VenueOrderId,
    ) -> Result<Option<nautilus_model::reports::OrderStatusReport>> {
        debug!("Generating order status report for {}", venue_order_id);

        let order = self
            .http_client
            .get_order(&venue_order_id.to_string(), None)
            .await?;

        let instrument_id = InstrumentId::from(format!("{}.{}", order.symbol, ALPACA_VENUE).as_str());
        let instrument = self.get_instrument(&instrument_id);

        if instrument.is_none() {
            warn!("Instrument not found for order: {}", order.symbol);
            return Ok(None);
        }

        let ts_init = get_atomic_clock_realtime().get_time_ns();
        let report = parse_order_status_report(&order, self.account_id, instrument.unwrap(), ts_init)?;

        Ok(Some(report))
    }

    /// Generates order status reports for all orders.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn generate_order_status_reports(
        &self,
    ) -> Result<Vec<nautilus_model::reports::OrderStatusReport>> {
        debug!("Generating order status reports");

        let orders = self.http_client.get_orders(None, None, None, None, None).await?;

        let mut reports = Vec::new();
        let ts_init = get_atomic_clock_realtime().get_time_ns();

        for order in orders {
            let instrument_id = InstrumentId::from(format!("{}.{}", order.symbol, ALPACA_VENUE).as_str());
            if let Some(instrument) = self.get_instrument(&instrument_id) {
                if let Ok(report) = parse_order_status_report(&order, self.account_id, instrument, ts_init) {
                    reports.push(report);
                }
            }
        }

        info!("Generated {} order status reports", reports.len());
        Ok(reports)
    }

    /// Generates position status reports for all positions.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn generate_position_status_reports(
        &self,
    ) -> Result<Vec<nautilus_model::reports::PositionStatusReport>> {
        debug!("Generating position status reports");

        let positions = self.http_client.get_positions().await?;

        let mut reports = Vec::new();
        let ts_init = get_atomic_clock_realtime().get_time_ns();

        for position in positions {
            let instrument_id = InstrumentId::from(format!("{}.{}", position.symbol, ALPACA_VENUE).as_str());
            if let Some(instrument) = self.get_instrument(&instrument_id) {
                if let Ok(report) =
                    parse_position_status_report(&position, self.account_id, instrument, ts_init)
                {
                    reports.push(report);

                    // Update account state
                    if let Some(ref mut account_state) = self.account_state.as_ref() {
                        let pos_state = PositionState::from_alpaca_position(&position, instrument_id);
                        // Note: Can't mutate here, would need interior mutability
                        // This is a simplification for the migration
                    }
                }
            }
        }

        info!("Generated {} position status reports", reports.len());
        Ok(reports)
    }

    /// Generates fill reports from account activities.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn generate_fill_reports(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<nautilus_model::reports::FillReport>> {
        debug!("Generating fill reports");

        let activities = self
            .http_client
            .get_account_activities(Some("FILL"), start, end, None, None)
            .await?;

        let mut reports = Vec::new();
        let ts_init = get_atomic_clock_realtime().get_time_ns();

        for activity in activities {
            // Parse activity as JSON value for flexible access
            if let Ok(activity_value) = serde_json::to_value(&activity) {
                if let Some(symbol) = activity_value.get("symbol").and_then(|v| v.as_str()) {
                    let instrument_id = InstrumentId::from(format!("{}.{}", symbol, ALPACA_VENUE).as_str());
                    if let Some(instrument) = self.get_instrument(&instrument_id) {
                        if let Ok(report) = parse_fill_report(
                            &activity_value,
                            self.account_id,
                            instrument,
                            ts_init,
                        ) {
                            reports.push(report);
                        }
                    }
                }
            }
        }

        info!("Generated {} fill reports", reports.len());
        Ok(reports)
    }

    // 
    // Account Queries
    // 

    /// Gets the current account state.
    pub fn account_state(&self) -> Option<&AlpacaAccountState> {
        self.account_state.as_ref()
    }

    /// Refreshes the account state from the API.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn refresh_account_state(&mut self) -> Result<()> {
        let account_data = self.http_client.get_account().await?;

        if let Some(ref mut state) = self.account_state {
            state.update_from_alpaca_account(&account_data);
        } else {
            self.account_state = Some(AlpacaAccountState::from_alpaca_account(
                &account_data,
                self.account_id,
            ));
        }

        Ok(())
    }

    /// Gets margin information for the account.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn get_margin_info(&self) -> Result<MarginInfo> {
        let account = self.http_client.get_account().await?;

        let equity = Decimal::from_str_exact(&account.equity).unwrap_or_default();
        let margin_enabled = equity >= Decimal::new(2000, 0);

        Ok(MarginInfo {
            equity: Decimal::from_str_exact(&account.equity).unwrap_or_default(),
            cash: Decimal::from_str_exact(&account.cash).unwrap_or_default(),
            buying_power: Decimal::from_str_exact(&account.buying_power).unwrap_or_default(),
            regt_buying_power: Decimal::from_str_exact(&account.regt_buying_power)
                .unwrap_or_default(),
            daytrading_buying_power: Decimal::from_str_exact(&account.daytrading_buying_power)
                .unwrap_or_default(),
            initial_margin: Decimal::from_str_exact(&account.initial_margin).unwrap_or_default(),
            maintenance_margin: Decimal::from_str_exact(&account.maintenance_margin)
                .unwrap_or_default(),
            last_maintenance_margin: Decimal::from_str_exact(&account.last_maintenance_margin)
                .unwrap_or_default(),
            pattern_day_trader: account.pattern_day_trader,
            daytrade_count: account.daytrade_count,
            multiplier: account.multiplier.parse().unwrap_or(1),
            shorting_enabled: account.shorting_enabled,
            margin_enabled,
            status: account.status,
            trading_blocked: account.trading_blocked,
        })
    }

    /// Checks if a symbol is shortable.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn check_shortable(&self, symbol: &str) -> Result<ShortabilityInfo> {
        let asset = self.http_client.get_asset(symbol).await?;

        Ok(ShortabilityInfo {
            symbol: symbol.to_string(),
            shortable: asset.shortable,
            easy_to_borrow: asset.easy_to_borrow,
            marginable: asset.marginable,
            fractionable: asset.fractionable,
            tradable: asset.tradable,
            status: asset.status,
        })
    }

    /// Validates whether a short sell order can be placed.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub async fn validate_short_order(
        &self,
        symbol: &str,
        quantity: Decimal,
    ) -> Result<()> {
        // Check margin/shorting enabled
        let margin_info = self.get_margin_info().await?;

        if !margin_info.margin_enabled {
            return Err(AlpacaError::InsufficientMargin(
                "Margin/shorting requires $2,000+ account equity".to_string(),
            ));
        }

        if !margin_info.shorting_enabled {
            return Err(AlpacaError::ShortingNotEnabled);
        }

        // Check if symbol is shortable
        let short_info = self.check_shortable(symbol).await?;

        if !short_info.shortable {
            return Err(AlpacaError::NotShortable(format!(
                "{} is not shortable",
                symbol
            )));
        }

        if !short_info.easy_to_borrow {
            return Err(AlpacaError::NotShortable(format!(
                "{} is Hard-To-Borrow (HTB), only ETB securities can be shorted",
                symbol
            )));
        }

        Ok(())
    }
}

/// Margin information for an Alpaca account.
#[derive(Debug, Clone)]
pub struct MarginInfo {
    pub equity: Decimal,
    pub cash: Decimal,
    pub buying_power: Decimal,
    pub regt_buying_power: Decimal,
    pub daytrading_buying_power: Decimal,
    pub initial_margin: Decimal,
    pub maintenance_margin: Decimal,
    pub last_maintenance_margin: Decimal,
    pub pattern_day_trader: bool,
    pub daytrade_count: i32,
    pub multiplier: i32,
    pub shorting_enabled: bool,
    pub margin_enabled: bool,
    pub status: String,
    pub trading_blocked: bool,
}

/// Shortability information for a symbol.
#[derive(Debug, Clone)]
pub struct ShortabilityInfo {
    pub symbol: String,
    pub shortable: bool,
    pub easy_to_borrow: bool,
    pub marginable: bool,
    pub fractionable: bool,
    pub tradable: bool,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    fn test_execution_client_creation() {
        let config = AlpacaExecClientConfig {
            api_key: "test_key".to_string(),
            api_secret: "test_secret".to_string(),
            environment: AlpacaEnvironment::Paper,
            timeout_secs: Some(30),
            proxy_url: None,
        };

        let client = AlpacaExecutionClient::new(config);
        assert!(client.is_ok());
    }
}
