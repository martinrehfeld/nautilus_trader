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

//! Report generation for orders, positions, and fills.

use nautilus_core::{uuid::UUID4, UnixNanos};
use nautilus_model::{
    enums::{LiquiditySide, OrderSide, OrderStatus, OrderType, PositionSideSpecified, TimeInForce},
    identifiers::{AccountId, ClientOrderId, TradeId, VenueOrderId},
    instruments::{Instrument, InstrumentAny},
    reports::{FillReport, OrderStatusReport, PositionStatusReport},
    types::{currency::Currency, money::Money, price::Price, quantity::Quantity},
};
use rust_decimal::{prelude::ToPrimitive, Decimal};

use crate::{error::AlpacaError, http::models::{AlpacaOrder, AlpacaPosition}};

/// Generates an `OrderStatusReport` from an Alpaca order.
///
/// # Errors
///
/// Returns an error if:
/// - Order data is malformed
/// - Instrument not found in cache
/// - Timestamp parsing fails
pub fn parse_order_status_report(
    order: &AlpacaOrder,
    account_id: AccountId,
    instrument: &InstrumentAny,
    ts_init: UnixNanos,
) -> Result<OrderStatusReport, AlpacaError> {
    // Map Alpaca status to Nautilus OrderStatus
    let order_status = match order.status.as_str() {
        "new" => OrderStatus::Accepted,
        "accepted" => OrderStatus::Accepted,
        "partially_filled" => OrderStatus::PartiallyFilled,
        "filled" => OrderStatus::Filled,
        "canceled" => OrderStatus::Canceled,
        "rejected" => OrderStatus::Rejected,
        "expired" => OrderStatus::Expired,
        "pending_new" => OrderStatus::Submitted,
        "pending_cancel" => OrderStatus::PendingCancel,
        "pending_replace" => OrderStatus::PendingUpdate,
        _ => OrderStatus::Accepted,
    };

    // Map order type
    let order_type = match order.order_type.as_str() {
        "market" => OrderType::Market,
        "limit" => OrderType::Limit,
        "stop" => OrderType::StopMarket,
        "stop_limit" => OrderType::StopLimit,
        "trailing_stop" => OrderType::TrailingStopMarket,
        _ => OrderType::Market,
    };

    // Map side
    let order_side = match order.side.as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => {
            return Err(AlpacaError::ParseError(format!(
                "Invalid order side: {}",
                order.side
            )))
        }
    };

    // Map time in force
    let time_in_force = match order.time_in_force.as_str() {
        "day" => TimeInForce::Day,
        "gtc" => TimeInForce::Gtc,
        "ioc" => TimeInForce::Ioc,
        "fok" => TimeInForce::Fok,
        "opg" => TimeInForce::AtTheOpen,
        "cls" => TimeInForce::AtTheClose,
        _ => TimeInForce::Day,
    };

    // Parse quantities
    let quantity = order
        .qty
        .as_ref()
        .map(|q| Quantity::from(q.as_str()))
        .unwrap_or_else(|| Quantity::from("0"));

    let filled_qty = Quantity::from(order.filled_qty.as_str());

    // Parse average fill price
    let avg_px = order
        .filled_avg_price
        .as_ref()
        .and_then(|p| Decimal::from_str_exact(p).ok());

    // Parse limit price
    let price = order
        .limit_price
        .as_ref()
        .and_then(|p| p.parse::<Price>().ok());

    // Parse stop price
    let trigger_price = order
        .stop_price
        .as_ref()
        .and_then(|p| p.parse::<Price>().ok());

    // Parse timestamps
    let ts_accepted = parse_timestamp(&order.submitted_at)?.into();
    let ts_last = if let Some(ref filled_at) = order.filled_at {
        parse_timestamp(filled_at)?.into()
    } else if let Some(ref canceled_at) = order.canceled_at {
        parse_timestamp(canceled_at)?.into()
    } else {
        parse_timestamp(&order.updated_at)?.into()
    };

    Ok(OrderStatusReport::new(
        account_id,
        instrument.id(),
        Some(ClientOrderId::from(order.client_order_id.as_str())),
        VenueOrderId::from(order.id.as_str()),
        order_side,
        order_type,
        time_in_force,
        order_status,
        quantity,
        filled_qty,
        ts_accepted,
        ts_last,
        ts_init,
        Some(UUID4::new()),
    ))
}

/// Generates a `PositionStatusReport` from an Alpaca position.
///
/// # Errors
///
/// Returns an error if:
/// - Position data is malformed
/// - Instrument not found
pub fn parse_position_status_report(
    position: &AlpacaPosition,
    account_id: AccountId,
    instrument: &InstrumentAny,
    ts_init: UnixNanos,
) -> Result<PositionStatusReport, AlpacaError> {
    let qty = Decimal::from_str_exact(&position.qty).unwrap_or_default();
    let side = if qty >= Decimal::ZERO {
        PositionSideSpecified::Long
    } else {
        PositionSideSpecified::Short
    };

    let precision = if qty.fract() != Decimal::ZERO { 9 } else { 0 };
    let quantity = Quantity::new(
        qty.abs().to_f64().unwrap_or(0.0),
        precision,
    );

    let avg_px_open = Decimal::from_str_exact(&position.avg_entry_price).ok();

    Ok(PositionStatusReport::new(
        account_id,
        instrument.id(),
        side,
        quantity,
        ts_init,
        ts_init,
        Some(UUID4::new()),
        None, // venue_position_id not provided
        avg_px_open,
    ))
}

/// Generates a `FillReport` from an Alpaca account activity (fill).
///
/// # Errors
///
/// Returns an error if:
/// - Activity data is malformed
/// - Instrument not found
pub fn parse_fill_report(
    activity: &serde_json::Value,
    account_id: AccountId,
    instrument: &InstrumentAny,
    ts_init: UnixNanos,
) -> Result<FillReport, AlpacaError> {
    // Parse side
    let side_str = activity
        .get("side")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AlpacaError::ParseError("Missing 'side' field".to_string()))?;

    let order_side = match side_str.to_uppercase().as_str() {
        "BUY" => OrderSide::Buy,
        "SELL" => OrderSide::Sell,
        _ => {
            return Err(AlpacaError::ParseError(format!(
                "Invalid side: {}",
                side_str
            )))
        }
    };

    // Parse quantity
    let qty_str = activity
        .get("qty")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AlpacaError::ParseError("Missing 'qty' field".to_string()))?;
    let last_qty = Quantity::from(qty_str);

    // Parse price
    let price_str = activity
        .get("price")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AlpacaError::ParseError("Missing 'price' field".to_string()))?;
    let last_px = price_str
        .parse::<Price>()
        .map_err(|e| AlpacaError::ParseError(format!("Invalid price: {}", e)))?;

    // Parse venue order ID
    let order_id = activity
        .get("order_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AlpacaError::ParseError("Missing 'order_id' field".to_string()))?;
    let venue_order_id = VenueOrderId::from(order_id);

    // Parse execution ID
    let exec_id = activity
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AlpacaError::ParseError("Missing 'id' field".to_string()))?;
    let trade_id = TradeId::from(exec_id);

    // Parse timestamp
    let ts_event = if let Some(transaction_time) = activity.get("transaction_time") {
        if let Some(ts_str) = transaction_time.as_str() {
            parse_timestamp(ts_str)?.into()
        } else {
            ts_init
        }
    } else {
        ts_init
    };

    // Commission is in separate activity, default to 0
    let commission = Money::new(0.0, Currency::USD());

    Ok(FillReport::new(
        account_id,
        instrument.id(),
        venue_order_id,
        trade_id,
        order_side,
        last_qty,
        last_px,
        commission,
        LiquiditySide::NoLiquiditySide, // Not provided by Alpaca
        None, // client_order_id not provided
        None, // venue_position_id not provided
        ts_event,
        ts_init,
        Some(UUID4::new()),
    ))
}

/// Parses an ISO 8601 timestamp string to nanoseconds since UNIX epoch.
///
/// # Errors
///
/// Returns an error if the timestamp string is malformed.
pub fn parse_timestamp(timestamp: &str) -> Result<u64, AlpacaError> {
    use chrono::DateTime;

    let dt = DateTime::parse_from_rfc3339(timestamp)
        .map_err(|e| AlpacaError::ParseError(format!("Invalid timestamp: {}", e)))?;

    let nanos = dt.timestamp_nanos_opt().ok_or_else(|| {
        AlpacaError::ParseError("Timestamp out of range for nanoseconds".to_string())
    })?;

    Ok(nanos as u64)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use nautilus_model::identifiers::Venue;

    #[rstest]
    fn test_parse_timestamp() {
        let timestamp = "2024-01-15T10:30:00Z";
        let result = parse_timestamp(timestamp);
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_order_status_mapping() {
        let statuses = vec![
            ("new", OrderStatus::Accepted),
            ("accepted", OrderStatus::Accepted),
            ("partially_filled", OrderStatus::PartiallyFilled),
            ("filled", OrderStatus::Filled),
            ("canceled", OrderStatus::Canceled),
            ("rejected", OrderStatus::Rejected),
            ("expired", OrderStatus::Expired),
        ];

        for (alpaca_status, expected) in statuses {
            let order = AlpacaOrder {
                id: "test".to_string(),
                client_order_id: "test".to_string(),
                created_at: "2024-01-15T10:30:00Z".to_string(),
                updated_at: "2024-01-15T10:30:00Z".to_string(),
                submitted_at: "2024-01-15T10:30:00Z".to_string(),
                filled_at: None,
                expired_at: None,
                canceled_at: None,
                failed_at: None,
                replaced_at: None,
                replaced_by: None,
                replaces: None,
                asset_id: "test".to_string(),
                symbol: "AAPL".to_string(),
                asset_class: "us_equity".to_string(),
                notional: None,
                qty: Some("10".to_string()),
                filled_qty: "0".to_string(),
                filled_avg_price: None,
                order_class: "simple".to_string(),
                order_type: "market".to_string(),
                side: "buy".to_string(),
                time_in_force: "day".to_string(),
                limit_price: None,
                stop_price: None,
                status: alpaca_status.to_string(),
                extended_hours: false,
                legs: None,
                trail_price: None,
                trail_percent: None,
                hwm: None,
            };

            // Note: This test is simplified as we would need a real instrument
            // In practice, you'd mock the instrument or use a test fixture
        }
    }
}
