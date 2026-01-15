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

//! HTTP request and response models for the Alpaca API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

// ================================================================================================
// Asset Models
// ================================================================================================

/// Represents an asset from the Alpaca `/v2/assets` endpoint.
///
/// See: <https://docs.alpaca.markets/reference/get-v2-assets>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaAsset {
    /// Unique asset identifier.
    pub id: String,
    /// Asset class (e.g., "us_equity", "crypto").
    #[serde(rename = "class")]
    pub class: String,
    /// Exchange where the asset is traded.
    pub exchange: String,
    /// Asset symbol.
    pub symbol: String,
    /// Full name of the asset.
    pub name: String,
    /// Trading status (e.g., "active", "inactive").
    pub status: String,
    /// Whether the asset is tradable.
    pub tradable: bool,
    /// Whether the asset is marginable.
    pub marginable: bool,
    /// Whether the asset is shortable.
    pub shortable: bool,
    /// Whether the asset is easy to borrow for shorting.
    pub easy_to_borrow: bool,
    /// Whether fractional trading is supported.
    pub fractionable: bool,
    /// Maintenance margin requirement (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintenance_margin_requirement: Option<f64>,
    /// Minimum order size (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_order_size: Option<String>,
    /// Minimum trade increment (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_trade_increment: Option<String>,
    /// Price increment (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_increment: Option<String>,
    /// Additional attributes (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
}

// ================================================================================================
// Option Contract Models
// ================================================================================================

/// Represents an option contract from the Alpaca `/v2/options/contracts` endpoint.
///
/// See: <https://docs.alpaca.markets/docs/options-trading>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlpacaOptionContract {
    /// Unique option contract identifier.
    pub id: String,
    /// Option contract symbol.
    pub symbol: String,
    /// Full name of the option contract.
    pub name: String,
    /// Trading status (e.g., "active", "inactive").
    pub status: String,
    /// Whether the option is tradable.
    pub tradable: bool,
    /// Expiration date (YYYY-MM-DD).
    pub expiration_date: String,
    /// Root symbol.
    pub root_symbol: String,
    /// Underlying asset symbol.
    pub underlying_symbol: String,
    /// Underlying asset ID.
    pub underlying_asset_id: String,
    /// Option type ("call" or "put").
    #[serde(rename = "type")]
    pub option_type: String,
    /// Option style ("american" or "european").
    pub style: String,
    /// Strike price.
    pub strike_price: String,
    /// Contract multiplier (typically "100").
    pub size: String,
    /// Open interest (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<String>,
    /// Open interest date (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_interest_date: Option<String>,
    /// Close price (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_price: Option<String>,
    /// Close price date (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_price_date: Option<String>,
}

/// Represents a paginated response for option contracts.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlpacaOptionContractsResponse {
    /// List of option contracts.
    pub option_contracts: Vec<AlpacaOptionContract>,
    /// Token for fetching the next page (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

// ================================================================================================
// Account Models
// ================================================================================================

/// Represents account information from the Alpaca `/v2/account` endpoint.
///
/// See: <https://docs.alpaca.markets/reference/get-v2-account>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaAccount {
    /// Account ID.
    pub id: String,
    /// Account number.
    pub account_number: String,
    /// Account status (e.g., "ACTIVE").
    pub status: String,
    /// Account currency (typically "USD").
    pub currency: String,
    /// Buying power available.
    pub buying_power: String,
    /// Regt buying power (for day trading).
    pub regt_buying_power: String,
    /// Daytrading buying power.
    pub daytrading_buying_power: String,
    /// Effective buying power.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_buying_power: Option<String>,
    /// Non-marginable buying power.
    pub non_marginable_buying_power: String,
    /// Options buying power.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_buying_power: Option<String>,
    /// Beginning of day daytrading buying power.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bod_dtbp: Option<String>,
    /// Cash available.
    pub cash: String,
    /// Accumulated fees.
    pub accrued_fees: String,
    /// Pending transfer out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_transfer_out: Option<String>,
    /// Pending transfer in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_transfer_in: Option<String>,
    /// Portfolio value.
    pub portfolio_value: String,
    /// Pattern day trader flag.
    pub pattern_day_trader: bool,
    /// Trading blocked flag.
    pub trading_blocked: bool,
    /// Transfers blocked flag.
    pub transfers_blocked: bool,
    /// Account blocked flag.
    pub account_blocked: bool,
    /// Created at timestamp.
    pub created_at: String,
    /// Trade suspended by user flag.
    pub trade_suspended_by_user: bool,
    /// Multiplier (leverage).
    pub multiplier: String,
    /// Shorting enabled flag.
    pub shorting_enabled: bool,
    /// Equity value.
    pub equity: String,
    /// Last equity value.
    pub last_equity: String,
    /// Long market value.
    pub long_market_value: String,
    /// Short market value.
    pub short_market_value: String,
    /// Position market value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_market_value: Option<String>,
    /// Initial margin.
    pub initial_margin: String,
    /// Maintenance margin.
    pub maintenance_margin: String,
    /// Last maintenance margin.
    pub last_maintenance_margin: String,
    /// SMA (Special Memorandum Account).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sma: Option<String>,
    /// Day trade count.
    pub daytrade_count: i32,
    /// Balance ASOF date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_asof: Option<String>,
    /// Crypto status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_status: Option<String>,
    /// Crypto tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_tier: Option<i32>,
    /// Options approved level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_approved_level: Option<i32>,
    /// Options trading level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_trading_level: Option<i32>,
    /// Intraday adjustments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intraday_adjustments: Option<String>,
    /// Pending regulatory TAF fees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_reg_taf_fees: Option<String>,
}

/// Represents a position from the Alpaca `/v2/positions` endpoint.
///
/// See: <https://docs.alpaca.markets/reference/get-v2-positions>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaPosition {
    /// Asset ID.
    pub asset_id: String,
    /// Asset symbol.
    pub symbol: String,
    /// Exchange.
    pub exchange: String,
    /// Asset class (e.g., "us_equity", "crypto").
    pub asset_class: String,
    /// Average entry price.
    pub avg_entry_price: String,
    /// Current quantity.
    pub qty: String,
    /// Available quantity for trading.
    pub qty_available: String,
    /// Side ("long" or "short").
    pub side: String,
    /// Market value.
    pub market_value: String,
    /// Cost basis.
    pub cost_basis: String,
    /// Unrealized P&L.
    pub unrealized_pl: String,
    /// Unrealized P&L percentage.
    pub unrealized_plpc: String,
    /// Unrealized intraday P&L.
    pub unrealized_intraday_pl: String,
    /// Unrealized intraday P&L percentage.
    pub unrealized_intraday_plpc: String,
    /// Current asset price.
    pub current_price: String,
    /// Last day's price.
    pub lastday_price: String,
    /// Change from last day.
    pub change_today: String,
}

// ================================================================================================
// Order Models
// ================================================================================================

/// Represents an order from the Alpaca `/v2/orders` endpoint.
///
/// See: <https://docs.alpaca.markets/reference/postorders>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaOrder {
    /// Order ID.
    pub id: String,
    /// Client order ID.
    pub client_order_id: String,
    /// Created at timestamp.
    pub created_at: String,
    /// Updated at timestamp.
    pub updated_at: String,
    /// Submitted at timestamp.
    pub submitted_at: String,
    /// Filled at timestamp (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_at: Option<String>,
    /// Expired at timestamp (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<String>,
    /// Canceled at timestamp (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canceled_at: Option<String>,
    /// Failed at timestamp (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<String>,
    /// Replaced at timestamp (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_at: Option<String>,
    /// Order that replaces this order (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
    /// Order that this order replaces (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaces: Option<String>,
    /// Asset ID.
    pub asset_id: String,
    /// Asset symbol.
    pub symbol: String,
    /// Asset class.
    pub asset_class: String,
    /// Notional value (optional, for fractional orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    /// Order quantity.
    pub qty: Option<String>,
    /// Filled quantity.
    pub filled_qty: String,
    /// Filled average price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_avg_price: Option<String>,
    /// Order class (e.g., "simple", "bracket", "oco", "oto").
    pub order_class: String,
    /// Order type (e.g., "market", "limit", "stop", "stop_limit", "trailing_stop").
    pub order_type: String,
    /// Order side ("buy" or "sell").
    pub side: String,
    /// Time in force (e.g., "day", "gtc", "opg", "cls", "ioc", "fok").
    pub time_in_force: String,
    /// Limit price (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
    /// Stop price (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    /// Order status (e.g., "new", "accepted", "filled", "canceled", etc.).
    pub status: String,
    /// Extended hours flag.
    pub extended_hours: bool,
    /// Legs for multi-leg orders (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legs: Option<Vec<AlpacaOrder>>,
    /// Trail price (optional, for trailing stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_price: Option<String>,
    /// Trail percent (optional, for trailing stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_percent: Option<String>,
    /// HWMK (High Water Mark).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hwm: Option<String>,
}

/// Response from canceling an order.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaCancelStatus {
    /// Order ID.
    pub id: String,
    /// HTTP status code.
    pub status: u16,
    /// Error message if cancellation failed (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

/// Request body for submitting an order.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaOrderRequest {
    /// Asset symbol.
    pub symbol: String,
    /// Order quantity (optional, use notional for fractional orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<String>,
    /// Notional value (optional, for fractional orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    /// Order side ("buy" or "sell").
    pub side: String,
    /// Order type (e.g., "market", "limit", "stop", "stop_limit", "trailing_stop").
    #[serde(rename = "type")]
    pub order_type: String,
    /// Time in force (e.g., "day", "gtc", "opg", "cls", "ioc", "fok").
    pub time_in_force: String,
    /// Limit price (optional, required for "limit" and "stop_limit").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
    /// Stop price (optional, required for "stop" and "stop_limit").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    /// Trail price (optional, for trailing stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_price: Option<String>,
    /// Trail percent (optional, for trailing stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_percent: Option<String>,
    /// Extended hours flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_hours: Option<bool>,
    /// Client order ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Order class (e.g., "simple", "bracket", "oco", "oto", "mleg").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_class: Option<String>,
    /// Take profit leg (optional, for bracket orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<TakeProfitSpec>,
    /// Stop loss leg (optional, for bracket orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<StopLossSpec>,
    /// Legs for multi-leg orders (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legs: Option<Vec<OrderLeg>>,
}

/// Take profit specification for bracket orders.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TakeProfitSpec {
    /// Limit price for take profit.
    pub limit_price: String,
}

/// Stop loss specification for bracket orders.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StopLossSpec {
    /// Stop price for stop loss.
    pub stop_price: String,
    /// Limit price (optional, for stop limit orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
}

/// Represents a leg in a multi-leg order.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderLeg {
    /// Asset symbol.
    pub symbol: String,
    /// Order side ("buy" or "sell").
    pub side: String,
    /// Quantity ratio for this leg.
    pub ratio_qty: i32,
    /// Position intent (optional, for multi-leg orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_intent: Option<String>,
}

// ================================================================================================
// Activity Models
// ================================================================================================

/// Represents an account activity from the `/v2/account/activities` endpoint.
///
/// Used for generating fill reports.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
pub struct AlpacaActivity {
    /// Activity ID.
    pub id: String,
    /// Activity type (e.g., "FILL", "DIV", "INT").
    pub activity_type: String,
    /// Transaction time.
    pub transaction_time: String,
    /// Activity type (for fills, "fill", "partial_fill").
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub fill_type: Option<String>,
    /// Order ID (for fill activities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// Asset symbol (for fill activities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Order side (for fill activities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    /// Quantity filled (for fill activities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<String>,
    /// Fill price (for fill activities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Leaves quantity (for partial fills).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leaves_qty: Option<String>,
    /// Cumulative quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cum_qty: Option<String>,
}

// ================================================================================================
// Market Data Models
// ================================================================================================

/// Represents a trade tick from market data endpoints.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlpacaTrade {
    /// Trade timestamp (RFC3339 format).
    pub t: String,
    /// Trade price.
    pub p: Decimal,
    /// Trade size (volume).
    pub s: i64,
    /// Exchange code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    /// Trade conditions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<Vec<String>>,
    /// Trade ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i: Option<i64>,
    /// Tape (for equities: A, B, or C).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<String>,
}

/// Represents a quote tick from market data endpoints.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlpacaQuote {
    /// Quote timestamp (RFC3339 format).
    pub t: String,
    /// Ask price.
    pub ap: Decimal,
    /// Ask size.
    pub as_: i64,
    /// Bid price.
    pub bp: Decimal,
    /// Bid size.
    pub bs: i64,
    /// Ask exchange.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ax: Option<String>,
    /// Bid exchange.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bx: Option<String>,
    /// Conditions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<Vec<String>>,
    /// Tape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<String>,
}

/// Represents a bar (OHLCV candle) from market data endpoints.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlpacaBar {
    /// Bar timestamp (RFC3339 format).
    pub t: String,
    /// Open price.
    pub o: Decimal,
    /// High price.
    pub h: Decimal,
    /// Low price.
    pub l: Decimal,
    /// Close price.
    pub c: Decimal,
    /// Volume.
    pub v: i64,
    /// Number of trades (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    /// VWAP (volume-weighted average price).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vw: Option<Decimal>,
}

/// Historical bars response wrapper.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlpacaBarsResponse {
    /// Symbol.
    pub symbol: String,
    /// List of bars.
    pub bars: Vec<AlpacaBar>,
    /// Next page token (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

/// Historical trades response wrapper.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlpacaTradesResponse {
    /// Symbol.
    pub symbol: String,
    /// List of trades.
    pub trades: Vec<AlpacaTrade>,
    /// Next page token (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

/// Historical quotes response wrapper.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlpacaQuotesResponse {
    /// Symbol.
    pub symbol: String,
    /// List of quotes.
    pub quotes: Vec<AlpacaQuote>,
    /// Next page token (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    fn test_alpaca_asset_deserialization() {
        let json = r#"{
            "id": "8ccae427-5dd0-45b3-b5fe-7ba5e422c766",
            "class": "us_equity",
            "exchange": "NASDAQ",
            "symbol": "AAPL",
            "name": "Apple Inc.",
            "status": "active",
            "tradable": true,
            "marginable": true,
            "shortable": true,
            "easy_to_borrow": true,
            "fractionable": true
        }"#;

        let asset: AlpacaAsset = serde_json::from_str(json).unwrap();
        assert_eq!(asset.symbol, "AAPL");
        assert_eq!(asset.class, "us_equity");
        assert!(asset.tradable);
    }

    #[rstest]
    fn test_alpaca_option_contract_deserialization() {
        let json = r#"{
            "id": "option-id-123",
            "symbol": "AAPL250117C00150000",
            "name": "AAPL Jan 17 2025 $150 Call",
            "status": "active",
            "tradable": true,
            "expiration_date": "2025-01-17",
            "root_symbol": "AAPL",
            "underlying_symbol": "AAPL",
            "underlying_asset_id": "asset-id-456",
            "type": "call",
            "style": "american",
            "strike_price": "150.00",
            "size": "100"
        }"#;

        let contract: AlpacaOptionContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.symbol, "AAPL250117C00150000");
        assert_eq!(contract.option_type, "call");
        assert_eq!(contract.strike_price, "150.00");
    }

    #[rstest]
    fn test_alpaca_order_request_serialization() {
        let request = AlpacaOrderRequest {
            symbol: "AAPL".to_string(),
            qty: Some("100".to_string()),
            notional: None,
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            time_in_force: "day".to_string(),
            limit_price: Some("150.50".to_string()),
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: Some(false),
            client_order_id: Some("my-order-123".to_string()),
            order_class: Some("simple".to_string()),
            take_profit: None,
            stop_loss: None,
            legs: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("AAPL"));
        assert!(json.contains("limit"));
    }
}
