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

//! Parsers for converting Alpaca assets and option contracts into Nautilus instrument types.

use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::{AssetClass, OptionKind},
    identifiers::{InstrumentId, Symbol, Venue},
    instruments::{CurrencyPair, Equity, InstrumentAny, OptionContract},
    types::{Currency, Price, Quantity},
};
use rust_decimal::Decimal;
use ustr::Ustr;

use crate::{
    error::{AlpacaError, Result},
    http::models::{AlpacaAsset, AlpacaOptionContract},
};

/// The default venue for Alpaca instruments.
pub const ALPACA_VENUE: &str = "ALPACA";

/// Parse an Alpaca asset into a Nautilus `Equity` instrument.
///
/// # Errors
///
/// Returns an error if the asset data is invalid or cannot be parsed.
pub fn parse_equity_instrument(
    asset: &AlpacaAsset,
    venue: &Venue,
    ts_init: UnixNanos,
) -> Result<InstrumentAny> {
    // Parse symbol
    let raw_symbol = Symbol::new(&asset.symbol);

    let instrument_id = InstrumentId::new(raw_symbol, *venue);

    // Determine price precision and increment
    let (price_precision, price_increment) = if let Some(ref increment_str) = asset.price_increment
    {
        let increment_dec = Decimal::from_str(increment_str).map_err(|e| {
            AlpacaError::ParseError(format!("Invalid price_increment '{}': {}", increment_str, e))
        })?;
        // Calculate precision from the scale of the decimal
        let precision = increment_dec.scale() as u8;
        let price_inc = Price::from_str(increment_str).map_err(|e| {
            AlpacaError::ParseError(format!("Invalid price increment: {}", e))
        })?;
        (precision, price_inc)
    } else {
        // Default to penny increments for US equities
        let precision = 2;
        let price_inc = Price::from_str("0.01")
            .map_err(|e| AlpacaError::ParseError(format!("Invalid price increment: {}", e)))?;
        (precision, price_inc)
    };

    // Determine lot size
    let lot_size = if let Some(ref increment_str) = asset.min_trade_increment {
        Some(Quantity::from_str(increment_str).map_err(|e| {
            AlpacaError::ParseError(format!("Invalid lot size: {}", e))
        })?)
    } else {
        Some(Quantity::from_str("1").map_err(|e| {
            AlpacaError::ParseError(format!("Invalid lot size: {}", e))
        })?)
    };

    // Create the Equity instrument
    let instrument = Equity::new_checked(
        instrument_id,
        raw_symbol,
        None, // ISIN not provided by Alpaca API
        Currency::USD(),
        price_precision,
        price_increment,
        lot_size,
        None, // max_quantity
        None, // min_quantity
        None, // max_price
        None, // min_price
        Some(Decimal::ZERO), // margin_init
        Some(Decimal::ZERO), // margin_maint
        Some(Decimal::ZERO), // maker_fee
        Some(Decimal::ZERO), // taker_fee
        ts_init,
        ts_init,
    )
    .map_err(|e| AlpacaError::ParseError(format!("Failed to create Equity: {}", e)))?;

    Ok(InstrumentAny::Equity(instrument))
}

/// Parse an Alpaca crypto asset into a Nautilus `CurrencyPair` instrument.
///
/// # Errors
///
/// Returns an error if the asset data is invalid or cannot be parsed.
pub fn parse_crypto_instrument(
    asset: &AlpacaAsset,
    venue: &Venue,
    ts_init: UnixNanos,
) -> Result<InstrumentAny> {
    // Parse symbol (e.g., "BTC/USD" or "BTCUSD")
    let mut symbol_str = asset.symbol.clone();

    // Convert BTCUSD -> BTC/USD format if needed
    if !symbol_str.contains('/') {
        for quote in ["USDT", "USDC", "USD", "BTC"] {
            if symbol_str.ends_with(quote) {
                let base = &symbol_str[..symbol_str.len() - quote.len()];
                symbol_str = format!("{}/{}", base, quote);
                break;
            }
        }
    }

    let raw_symbol = Symbol::new(&symbol_str);

    let instrument_id = InstrumentId::new(raw_symbol, *venue);

    // Parse base and quote currencies
    let (base_str, quote_str) = if let Some((base, quote)) = symbol_str.split_once('/') {
        (base, quote)
    } else {
        // Fallback - assume USD quote
        (symbol_str.as_str(), "USD")
    };

    let base_currency = Currency::from_str(base_str).map_err(|e| {
        AlpacaError::ParseError(format!("Invalid base currency '{}': {}", base_str, e))
    })?;

    let quote_currency = Currency::from_str(quote_str).map_err(|e| {
        AlpacaError::ParseError(format!("Invalid quote currency '{}': {}", quote_str, e))
    })?;

    // Determine price precision and increment
    let (price_precision, price_increment) = if let Some(ref increment_str) = asset.price_increment
    {
        let increment_dec = Decimal::from_str(increment_str).map_err(|e| {
            AlpacaError::ParseError(format!("Invalid price_increment '{}': {}", increment_str, e))
        })?;
        let precision = increment_dec.scale() as u8;
        let price_inc = Price::from_str(increment_str).map_err(|e| {
            AlpacaError::ParseError(format!("Invalid price increment: {}", e))
        })?;
        (precision, price_inc)
    } else {
        // Default for crypto
        let precision = 2;
        let price_inc = Price::from_str("0.01")
            .map_err(|e| AlpacaError::ParseError(format!("Invalid price increment: {}", e)))?;
        (precision, price_inc)
    };

    // Determine size precision and increment
    let (size_precision, size_increment) = if let Some(ref increment_str) = asset.min_trade_increment
    {
        let increment_dec = Decimal::from_str(increment_str).map_err(|e| {
            AlpacaError::ParseError(format!("Invalid min_trade_increment '{}': {}", increment_str, e))
        })?;
        let precision = increment_dec.scale() as u8;
        let size_inc = Quantity::from_str(increment_str).map_err(|e| {
            AlpacaError::ParseError(format!("Invalid size increment: {}", e))
        })?;
        (precision, size_inc)
    } else {
        // Default 8 decimals for crypto
        let precision = 8;
        let size_inc = Quantity::from_str("0.00000001")
            .map_err(|e| AlpacaError::ParseError(format!("Invalid size increment: {}", e)))?;
        (precision, size_inc)
    };

    // Min quantity
    let min_quantity = if let Some(ref min_size_str) = asset.min_order_size {
        Some(Quantity::from_str(min_size_str).map_err(|e| {
            AlpacaError::ParseError(format!("Invalid min quantity: {}", e))
        })?)
    } else {
        None
    };

    // Create the CurrencyPair instrument
    let instrument = CurrencyPair::new_checked(
        instrument_id,
        raw_symbol,
        base_currency,
        quote_currency,
        price_precision,
        size_precision,
        price_increment,
        size_increment,
        None, // multiplier
        None, // lot_size (crypto is fractionable)
        None, // max_quantity
        min_quantity,
        None, // max_notional
        None, // min_notional
        None, // max_price
        None, // min_price
        Some(Decimal::ZERO), // margin_init
        Some(Decimal::ZERO), // margin_maint
        Some(Decimal::ZERO), // maker_fee
        Some(Decimal::ZERO), // taker_fee
        ts_init,
        ts_init,
    )
    .map_err(|e| AlpacaError::ParseError(format!("Failed to create CurrencyPair: {}", e)))?;

    Ok(InstrumentAny::CurrencyPair(instrument))
}

/// Parse an Alpaca option contract into a Nautilus `OptionContract` instrument.
///
/// # Errors
///
/// Returns an error if the contract data is invalid or cannot be parsed.
pub fn parse_option_instrument(
    contract: &AlpacaOptionContract,
    venue: &Venue,
    ts_init: UnixNanos,
) -> Result<InstrumentAny> {
    // Parse symbol
    let raw_symbol = Symbol::new(&contract.symbol);

    let instrument_id = InstrumentId::new(raw_symbol, *venue);

    // Parse option kind
    let option_kind = match contract.option_type.as_str() {
        "call" => OptionKind::Call,
        "put" => OptionKind::Put,
        _ => {
            return Err(AlpacaError::ParseError(format!(
                "Invalid option type: {}",
                contract.option_type
            )))
        }
    };

    // Parse strike price
    let strike_price = Price::from_str(&contract.strike_price).map_err(|e| {
        AlpacaError::ParseError(format!(
            "Invalid strike_price '{}': {}",
            contract.strike_price, e
        ))
    })?;

    // Parse expiration date -> nanoseconds
    let expiration_date = NaiveDate::parse_from_str(&contract.expiration_date, "%Y-%m-%d")
        .map_err(|e| {
            AlpacaError::ParseError(format!(
                "Invalid expiration_date '{}': {}",
                contract.expiration_date, e
            ))
        })?;
    // Options expire at 4 PM ET (market close)
    let expiration_dt = expiration_date
        .and_hms_opt(16, 0, 0)
        .ok_or_else(|| AlpacaError::ParseError("Invalid expiration time".to_string()))?;
    let expiration_dt = DateTime::<Utc>::from_naive_utc_and_offset(expiration_dt, Utc);
    let expiration_ns = UnixNanos::from(expiration_dt.timestamp_nanos_opt().ok_or_else(|| {
        AlpacaError::ParseError("Expiration timestamp out of range".to_string())
    })? as u64);

    // Activation is typically when the contract was listed
    // Use 90 days before expiration as estimate
    let activation_dt = expiration_dt - chrono::Duration::days(90);
    let activation_ns = UnixNanos::from(activation_dt.timestamp_nanos_opt().ok_or_else(|| {
        AlpacaError::ParseError("Activation timestamp out of range".to_string())
    })? as u64);

    // Contract multiplier (typically 100 for equity options)
    let multiplier_str = if contract.size.is_empty() {
        "100"
    } else {
        &contract.size
    };
    let multiplier = Quantity::from_str(multiplier_str)
        .map_err(|e| AlpacaError::ParseError(format!("Invalid multiplier: {}", e)))?;

    // Price precision for options (typically $0.01)
    let price_precision = 2;
    let price_increment = Price::from_str("0.01")
        .map_err(|e| AlpacaError::ParseError(format!("Invalid price increment: {}", e)))?;

    // Parse underlying symbol
    let underlying = Ustr::from(&contract.underlying_symbol);

    // Create the OptionContract instrument
    let instrument = OptionContract::new_checked(
        instrument_id,
        raw_symbol,
        AssetClass::Equity, // Alpaca options are on equities
        None,               // exchange
        underlying,
        option_kind,
        strike_price,
        Currency::USD(),
        activation_ns,
        expiration_ns,
        price_precision,
        price_increment,
        multiplier,
        multiplier, // For options, lot size = multiplier
        None,       // max_quantity
        None,       // min_quantity
        None,       // max_price
        None,       // min_price
        Some(Decimal::ZERO), // margin_init
        Some(Decimal::ZERO), // margin_maint
        Some(Decimal::ZERO), // maker_fee
        Some(Decimal::ZERO), // taker_fee
        ts_init,
        ts_init,
    )
    .map_err(|e| AlpacaError::ParseError(format!("Failed to create OptionContract: {}", e)))?;

    Ok(InstrumentAny::OptionContract(instrument))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    fn test_parse_equity_instrument() {
        let asset = AlpacaAsset {
            id: "test-id".to_string(),
            class: "us_equity".to_string(),
            exchange: "NASDAQ".to_string(),
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            status: "active".to_string(),
            tradable: true,
            marginable: true,
            shortable: true,
            easy_to_borrow: true,
            fractionable: true,
            maintenance_margin_requirement: Some(0.3),
            min_order_size: None,
            min_trade_increment: Some("1".to_string()),
            price_increment: Some("0.01".to_string()),
            attributes: None,
        };

        let venue = Venue::from(ALPACA_VENUE);
        let ts_init = UnixNanos::from(1_000_000_000_u64);

        let result = parse_equity_instrument(&asset, &venue, ts_init);
        assert!(result.is_ok());

        if let Ok(InstrumentAny::Equity(equity)) = result {
            assert_eq!(equity.id.symbol.as_str(), "AAPL");
            assert_eq!(equity.price_precision, 2);
        } else {
            panic!("Expected Equity instrument");
        }
    }

    #[rstest]
    fn test_parse_crypto_instrument() {
        let asset = AlpacaAsset {
            id: "test-id".to_string(),
            class: "crypto".to_string(),
            exchange: "CRYPTO".to_string(),
            symbol: "BTC/USD".to_string(),
            name: "Bitcoin".to_string(),
            status: "active".to_string(),
            tradable: true,
            marginable: false,
            shortable: false,
            easy_to_borrow: false,
            fractionable: true,
            maintenance_margin_requirement: None,
            min_order_size: Some("0.0001".to_string()),
            min_trade_increment: Some("0.00000001".to_string()),
            price_increment: Some("0.01".to_string()),
            attributes: None,
        };

        let venue = Venue::from(ALPACA_VENUE);
        let ts_init = UnixNanos::from(1_000_000_000_u64);

        let result = parse_crypto_instrument(&asset, &venue, ts_init);
        assert!(result.is_ok());

        if let Ok(InstrumentAny::CurrencyPair(pair)) = result {
            assert_eq!(pair.id.symbol.as_str(), "BTC/USD");
            assert_eq!(pair.base_currency.code.as_str(), "BTC");
            assert_eq!(pair.quote_currency.code.as_str(), "USD");
        } else {
            panic!("Expected CurrencyPair instrument");
        }
    }
}
