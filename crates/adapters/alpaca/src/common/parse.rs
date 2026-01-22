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

//! Parsing utilities for converting Alpaca WebSocket messages to Nautilus types.
//!
//! This module handles the conversion of raw Alpaca market data (trades, quotes, bars)
//! into Nautilus data types (TradeTick, QuoteTick, Bar).

use chrono::{DateTime, Utc};
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::{Bar, BarSpecification, BarType, QuoteTick, TradeTick},
    enums::{AggregationSource, AggressorSide, BarAggregation, PriceType},
    identifiers::{InstrumentId, TradeId},
    types::{Price, Quantity},
};
use rust_decimal::{prelude::ToPrimitive, Decimal};

use super::models::{AlpacaWsBar, AlpacaWsQuote, AlpacaWsTrade};
use crate::error::{AlpacaError, Result};

// 
// Timestamp Parsing
// 

/// Parses RFC3339 timestamp string to Unix nanoseconds.
///
/// Alpaca timestamps come in RFC3339 format like "2023-08-25T14:30:00Z".
/// This function converts them to nanoseconds since Unix epoch.
///
/// # Errors
///
/// Returns an error if the timestamp string cannot be parsed.
pub fn parse_timestamp_ns(ts_str: &str) -> Result<UnixNanos> {
    let dt = DateTime::parse_from_rfc3339(ts_str).map_err(|e| {
        AlpacaError::ParseError(format!("Failed to parse timestamp '{}': {}", ts_str, e))
    })?;

    let utc_dt: DateTime<Utc> = dt.into();
    let nanos = utc_dt.timestamp_nanos_opt().ok_or_else(|| {
        AlpacaError::ParseError(format!("Timestamp out of range: {}", ts_str))
    })?;

    Ok(UnixNanos::from(nanos as u64))
}

/// Gets the current timestamp in Unix nanoseconds.
#[must_use]
pub fn current_timestamp_ns() -> UnixNanos {
    let now = Utc::now();
    let nanos = now
        .timestamp_nanos_opt()
        .expect("Current time should be valid");
    UnixNanos::from(nanos as u64)
}

// 
// Price and Quantity Parsing
// 

/// Parses a decimal price with given precision.
///
/// # Errors
///
/// Returns an error if the price cannot be parsed.
pub fn parse_price(value: Decimal, precision: u8) -> Result<Price> {
    // Price::from_raw expects raw value scaled to FIXED_PRECISION (9 or 16)
    // Not scaled to the instrument precision
    const FIXED_PRECISION: u8 = 16; // high-precision feature enabled
    let multiplier = Decimal::from(10_i64.pow(u32::from(FIXED_PRECISION)));
    let scaled = value * multiplier;
    let raw_i128 = scaled.to_i128().ok_or_else(|| {
        AlpacaError::ParseError(format!("Price value {} out of range for FIXED_PRECISION {}", value, FIXED_PRECISION))
    })?;
    let raw = raw_i128.try_into().map_err(|_| {
        AlpacaError::ParseError(format!("Price value {} exceeds i64 range", raw_i128))
    })?;
    Ok(Price::from_raw(raw, precision))
}

/// Parses a decimal quantity with given precision.
///
/// # Errors
///
/// Returns an error if the quantity cannot be parsed.
pub fn parse_quantity(value: Decimal, precision: u8) -> Result<Quantity> {
    // Quantity::from_raw expects raw value scaled to FIXED_PRECISION (9 or 16)
    // Not scaled to the instrument precision
    const FIXED_PRECISION: u8 = 16; // high-precision feature enabled
    let multiplier = Decimal::from(10_i64.pow(u32::from(FIXED_PRECISION)));
    let scaled = value * multiplier;
    let raw_u128 = scaled.to_u128().ok_or_else(|| {
        AlpacaError::ParseError(format!("Quantity value {} out of range for FIXED_PRECISION {}", value, FIXED_PRECISION))
    })?;
    let raw = raw_u128.try_into().map_err(|_| {
        AlpacaError::ParseError(format!("Quantity value {} exceeds u64 range", raw_u128))
    })?;
    Ok(Quantity::from_raw(raw, precision))
}

// 
// Trade Parsing
// 

/// Parses an Alpaca WebSocket trade message to a Nautilus `TradeTick`.
///
/// # Arguments
///
/// * `trade` - The Alpaca trade message from WebSocket.
/// * `instrument_id` - The Nautilus instrument ID.
/// * `price_precision` - Price precision for the instrument.
/// * `size_precision` - Size precision for the instrument.
/// * `ts_init` - Initialization timestamp.
///
/// # Errors
///
/// Returns an error if:
/// - Timestamp parsing fails
/// - Price/quantity conversion fails
/// - Required fields are missing
pub fn parse_trade_tick(
    trade: &AlpacaWsTrade,
    instrument_id: InstrumentId,
    price_precision: u8,
    size_precision: u8,
    ts_init: UnixNanos,
) -> Result<TradeTick> {
    // Parse timestamp
    let ts_event = parse_timestamp_ns(&trade.timestamp)?;

    // Parse price
    let price = parse_price(trade.price, price_precision)?;

    // Parse size
    let size = parse_quantity(trade.size, size_precision)?;

    // Parse trade ID (use trade_id if available, otherwise use timestamp)
    let trade_id = trade
        .trade_id
        .map(|id| TradeId::new(&id.to_string()))
        .unwrap_or_else(|| TradeId::new(&ts_event.as_u64().to_string()));

    // Alpaca doesn't provide aggressor side information
    let aggressor_side = AggressorSide::NoAggressor;

    Ok(TradeTick::new(
        instrument_id,
        price,
        size,
        aggressor_side,
        trade_id,
        ts_event,
        ts_init,
    ))
}

// 
// Quote Parsing
// 

/// Parses an Alpaca WebSocket quote message to a Nautilus `QuoteTick`.
///
/// # Arguments
///
/// * `quote` - The Alpaca quote message from WebSocket.
/// * `instrument_id` - The Nautilus instrument ID.
/// * `price_precision` - Price precision for the instrument.
/// * `size_precision` - Size precision for the instrument.
/// * `ts_init` - Initialization timestamp.
///
/// # Errors
///
/// Returns an error if:
/// - Timestamp parsing fails
/// - Price/quantity conversion fails
/// - Required fields are missing
pub fn parse_quote_tick(
    quote: &AlpacaWsQuote,
    instrument_id: InstrumentId,
    price_precision: u8,
    size_precision: u8,
    ts_init: UnixNanos,
) -> Result<QuoteTick> {
    // Parse timestamp
    let ts_event = parse_timestamp_ns(&quote.timestamp)?;

    eprintln!("🔍 parse_quote_tick INPUT: bid_price={}, ask_price={}, bid_size={}, ask_size={}, price_prec={}, size_prec={}",
              quote.bid_price, quote.ask_price, quote.bid_size, quote.ask_size, price_precision, size_precision);

    // Parse bid price and size
    let bid_price = parse_price(quote.bid_price, price_precision)?;
    let bid_size = parse_quantity(quote.bid_size, size_precision)?;

    // Parse ask price and size
    let ask_price = parse_price(quote.ask_price, price_precision)?;
    let ask_size = parse_quantity(quote.ask_size, size_precision)?;

    Ok(QuoteTick::new(
        instrument_id,
        bid_price,
        ask_price,
        bid_size,
        ask_size,
        ts_event,
        ts_init,
    ))
}

// 
// Bar Parsing
// 

/// Parses an Alpaca WebSocket bar message to a Nautilus `Bar`.
///
/// # Arguments
///
/// * `bar` - The Alpaca bar message from WebSocket.
/// * `bar_type` - The Nautilus bar type specification.
/// * `price_precision` - Price precision for the instrument.
/// * `size_precision` - Size precision for the instrument (for volume).
/// * `ts_init` - Initialization timestamp.
///
/// # Errors
///
/// Returns an error if:
/// - Timestamp parsing fails
/// - Price/quantity conversion fails
/// - Required fields are missing
pub fn parse_bar(
    bar: &AlpacaWsBar,
    bar_type: BarType,
    price_precision: u8,
    size_precision: u8,
    ts_init: UnixNanos,
) -> Result<Bar> {
    // Parse timestamp (bar start time)
    let ts_event = parse_timestamp_ns(&bar.timestamp)?;

    // Parse OHLC prices
    let open = parse_price(bar.open, price_precision)?;
    let high = parse_price(bar.high, price_precision)?;
    let low = parse_price(bar.low, price_precision)?;
    let close = parse_price(bar.close, price_precision)?;

    // Parse volume
    let volume = parse_quantity(bar.volume, size_precision)?;

    Ok(Bar::new(
        bar_type,
        open,
        high,
        low,
        close,
        volume,
        ts_event,
        ts_init,
    ))
}

/// Creates a bar type from instrument ID and timeframe string.
///
/// # Arguments
///
/// * `instrument_id` - The Nautilus instrument ID.
/// * `timeframe` - Alpaca timeframe string (e.g., "1Min", "1Hour", "1Day").
///
/// # Errors
///
/// Returns an error if the timeframe format is invalid.
pub fn create_bar_type(instrument_id: InstrumentId, timeframe: &str) -> Result<BarType> {
    let (step, aggregation) = parse_timeframe(timeframe)?;

    let spec = BarSpecification::new(
        step,
        aggregation,
        PriceType::Last,
    );

    Ok(BarType::new(instrument_id, spec, AggregationSource::External))
}

/// Parses Alpaca timeframe string to (step, aggregation) tuple.
///
/// Supports:
/// - Minutes: "1Min", "5Min", "15Min", "30Min"
/// - Hours: "1Hour", "2Hour", "4Hour"
/// - Days: "1Day"
/// - Weeks: "1Week"
/// - Months: "1Month"
///
/// # Errors
///
/// Returns an error if the timeframe format is not recognized.
pub fn parse_timeframe(timeframe: &str) -> Result<(usize, BarAggregation)> {
    if timeframe.ends_with("Min") {
        let step = timeframe
            .trim_end_matches("Min")
            .parse::<usize>()
            .map_err(|_| AlpacaError::ParseError(format!("Invalid minute timeframe: {}", timeframe)))?;
        Ok((step, BarAggregation::Minute))
    } else if timeframe.ends_with("Hour") {
        let step = timeframe
            .trim_end_matches("Hour")
            .parse::<usize>()
            .map_err(|_| AlpacaError::ParseError(format!("Invalid hour timeframe: {}", timeframe)))?;
        Ok((step, BarAggregation::Hour))
    } else if timeframe.ends_with("Day") {
        let step = timeframe
            .trim_end_matches("Day")
            .parse::<usize>()
            .map_err(|_| AlpacaError::ParseError(format!("Invalid day timeframe: {}", timeframe)))?;
        Ok((step, BarAggregation::Day))
    } else if timeframe.ends_with("Week") {
        let step = timeframe
            .trim_end_matches("Week")
            .parse::<usize>()
            .map_err(|_| AlpacaError::ParseError(format!("Invalid week timeframe: {}", timeframe)))?;
        Ok((step, BarAggregation::Week))
    } else if timeframe.ends_with("Month") {
        let step = timeframe
            .trim_end_matches("Month")
            .parse::<usize>()
            .map_err(|_| AlpacaError::ParseError(format!("Invalid month timeframe: {}", timeframe)))?;
        Ok((step, BarAggregation::Month))
    } else {
        Err(AlpacaError::ParseError(format!(
            "Unsupported timeframe: {}",
            timeframe
        )))
    }
}

// 
// Tests
// 

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    use nautilus_model::identifiers::{Symbol, Venue};

    #[rstest]
    fn test_parse_timestamp_ns() {
        let ts_str = "2023-08-25T14:30:00Z";
        let result = parse_timestamp_ns(ts_str);
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_parse_timestamp_with_millis() {
        let ts_str = "2023-08-25T14:30:00.123Z";
        let result = parse_timestamp_ns(ts_str);
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_parse_timestamp_with_offset() {
        let ts_str = "2023-08-25T14:30:00-04:00";
        let result = parse_timestamp_ns(ts_str);
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_parse_timestamp_invalid() {
        let ts_str = "not-a-timestamp";
        let result = parse_timestamp_ns(ts_str);
        assert!(result.is_err());
    }

    #[rstest]
    fn test_parse_timeframe_minutes() {
        let result = parse_timeframe("5Min");
        assert!(result.is_ok());
        let (step, agg) = result.unwrap();
        assert_eq!(step, 5);
        assert_eq!(agg, BarAggregation::Minute);
    }

    #[rstest]
    fn test_parse_timeframe_hours() {
        let result = parse_timeframe("4Hour");
        assert!(result.is_ok());
        let (step, agg) = result.unwrap();
        assert_eq!(step, 4);
        assert_eq!(agg, BarAggregation::Hour);
    }

    #[rstest]
    fn test_parse_timeframe_days() {
        let result = parse_timeframe("1Day");
        assert!(result.is_ok());
        let (step, agg) = result.unwrap();
        assert_eq!(step, 1);
        assert_eq!(agg, BarAggregation::Day);
    }

    #[rstest]
    fn test_parse_timeframe_invalid() {
        let result = parse_timeframe("5Second");
        assert!(result.is_err());
    }

    #[rstest]
    fn test_parse_trade_tick() {
        let trade = AlpacaWsTrade {
            msg_type: "t".to_string(),
            symbol: "AAPL".to_string(),
            trade_id: Some(123456),
            exchange: Some("V".to_string()),
            price: Decimal::new(15025, 2), // 150.25
            size: 100,
            timestamp: "2023-08-25T14:30:00Z".to_string(),
            conditions: Some(vec!["@".to_string()]),
            tape: Some("C".to_string()),
        };

        let instrument_id = InstrumentId::new(
            Symbol::new("AAPL"),
            Venue::new("ALPACA"),
        );
        let ts_init = current_timestamp_ns();

        let result = parse_trade_tick(&trade, instrument_id, 2, 0, ts_init);
        assert!(result.is_ok());

        let tick = result.unwrap();
        assert_eq!(tick.instrument_id, instrument_id);
        assert_eq!(tick.size.raw, 100);
    }

    #[rstest]
    fn test_parse_quote_tick() {
        let quote = AlpacaWsQuote {
            msg_type: "q".to_string(),
            symbol: "MSFT".to_string(),
            ask_exchange: Some("Q".to_string()),
            ask_price: Decimal::new(33050, 2), // 330.50
            ask_size: 200,
            bid_exchange: Some("Q".to_string()),
            bid_price: Decimal::new(33045, 2), // 330.45
            bid_size: 150,
            timestamp: "2023-08-25T14:30:00Z".to_string(),
            conditions: Some(vec!["R".to_string()]),
            tape: Some("C".to_string()),
        };

        let instrument_id = InstrumentId::new(
            Symbol::new("MSFT"),
            Venue::new("ALPACA"),
        );
        let ts_init = current_timestamp_ns();

        let result = parse_quote_tick(&quote, instrument_id, 2, 0, ts_init);
        assert!(result.is_ok());

        let tick = result.unwrap();
        assert_eq!(tick.instrument_id, instrument_id);
        assert_eq!(tick.ask_size.raw, 200);
        assert_eq!(tick.bid_size.raw, 150);
    }

    #[rstest]
    fn test_parse_bar() {
        let bar_data = AlpacaWsBar {
            msg_type: "b".to_string(),
            symbol: "SPY".to_string(),
            open: Decimal::new(44010, 2), // 440.10
            high: Decimal::new(44050, 2), // 440.50
            low: Decimal::new(44000, 2),  // 440.00
            close: Decimal::new(44025, 2), // 440.25
            volume: 1000000,
            timestamp: "2023-08-25T14:30:00Z".to_string(),
            trade_count: Some(5000),
            vwap: Some(Decimal::new(44022, 2)),
        };

        let instrument_id = InstrumentId::new(
            Symbol::new("SPY"),
            Venue::new("ALPACA"),
        );
        let bar_type = create_bar_type(instrument_id, "1Min").unwrap();
        let ts_init = current_timestamp_ns();

        let result = parse_bar(&bar_data, bar_type, 2, 0, ts_init);
        assert!(result.is_ok());

        let bar = result.unwrap();
        assert_eq!(bar.bar_type, bar_type);
        assert_eq!(bar.volume.raw, 1000000);
    }
}
