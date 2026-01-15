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

//! Order builders and validation types for the Alpaca execution client.

use nautilus_model::{
    enums::{OrderSide, OrderType, TimeInForce},
    identifiers::ClientOrderId,
    instruments::InstrumentAny,
    orders::{Order, OrderAny},
};
use rust_decimal::Decimal;

use crate::{
    common::enums::AlpacaAssetClass,
    error::AlpacaError,
    http::models::{AlpacaOrderRequest, OrderLeg},
};

/// Order builder for Alpaca orders with asset class-specific validation.
#[derive(Debug, Clone)]
pub struct AlpacaOrderBuilder {
    /// The Nautilus order to build from.
    order: OrderAny,
    /// The instrument associated with the order.
    instrument: InstrumentAny,
}

impl AlpacaOrderBuilder {
    /// Creates a new order builder.
    pub fn new(order: OrderAny, instrument: InstrumentAny) -> Self {
        Self { order, instrument }
    }

    /// Builds an Alpaca order request with asset class-specific constraints.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Asset class constraints are violated
    /// - Required fields are missing
    /// - Order type is not supported for the asset class
    pub fn build(self) -> Result<AlpacaOrderRequest, AlpacaError> {
        let asset_class = self.determine_asset_class();
        let symbol = self.get_symbol_string();

        // Map order side
        let side = match self.order.order_side() {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
            _ => {
                return Err(AlpacaError::InvalidOrderSide(format!(
                    "Unsupported order side: {:?}",
                    self.order.order_side()
                )))
            }
        };

        // Map order type
        let order_type = self.map_order_type()?;

        // Map time in force
        let time_in_force = self.map_time_in_force()?;

        // Apply asset class-specific constraints
        let (order_type, time_in_force, symbol) = match asset_class {
            AlpacaAssetClass::Crypto => self.apply_crypto_constraints(order_type, time_in_force, symbol)?,
            AlpacaAssetClass::Option => self.apply_options_constraints(order_type, time_in_force, symbol)?,
            _ => (order_type, time_in_force, symbol),
        };

        // Build request
        let mut request = AlpacaOrderRequest {
            symbol,
            qty: Some(self.order.quantity().to_string()),
            notional: None,
            side: side.to_string(),
            order_type: order_type.to_string(),
            time_in_force: time_in_force.to_string(),
            limit_price: None,
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: None,
            client_order_id: Some(self.order.client_order_id().to_string()),
            order_class: Some("simple".to_string()),
            take_profit: None,
            stop_loss: None,
            legs: None,
        };

        // Add limit price if applicable
        if let Some(price) = self.order.price() {
            request.limit_price = Some(price.to_string());
        }

        // Add stop price if applicable
        if let Some(trigger_price) = self.order.trigger_price() {
            request.stop_price = Some(trigger_price.to_string());
        }

        // Add extended hours flag for equities
        if matches!(asset_class, AlpacaAssetClass::UsEquity) {
            // Extended hours is not available in the base OrderAny trait
            // This would need to be set via order metadata if needed
            request.extended_hours = Some(false);
        }

        Ok(request)
    }

    fn determine_asset_class(&self) -> AlpacaAssetClass {
        match &self.instrument {
            InstrumentAny::CurrencyPair(_) => AlpacaAssetClass::Crypto,
            InstrumentAny::OptionContract(_) => AlpacaAssetClass::Option,
            _ => AlpacaAssetClass::UsEquity,
        }
    }

    fn get_symbol_string(&self) -> String {
        self.order.instrument_id().symbol.to_string()
    }

    fn map_order_type(&self) -> Result<&str, AlpacaError> {
        match self.order.order_type() {
            OrderType::Market => Ok("market"),
            OrderType::Limit => Ok("limit"),
            OrderType::StopMarket => Ok("stop"),
            OrderType::StopLimit => Ok("stop_limit"),
            OrderType::TrailingStopMarket => Ok("trailing_stop"),
            other => Err(AlpacaError::UnsupportedOrderType(format!(
                "Order type {:?} not supported by Alpaca",
                other
            ))),
        }
    }

    fn map_time_in_force(&self) -> Result<&str, AlpacaError> {
        match self.order.time_in_force() {
            TimeInForce::Day => Ok("day"),
            TimeInForce::Gtc => Ok("gtc"),
            TimeInForce::Ioc => Ok("ioc"),
            TimeInForce::Fok => Ok("fok"),
            TimeInForce::AtTheOpen => Ok("opg"),
            TimeInForce::AtTheClose => Ok("cls"),
            other => Err(AlpacaError::UnsupportedTimeInForce(format!(
                "Time in force {:?} not supported by Alpaca",
                other
            ))),
        }
    }

    fn apply_crypto_constraints<'a>(
        &self,
        order_type: &'a str,
        time_in_force: &'a str,
        symbol: String,
    ) -> Result<(&'a str, &'a str, String), AlpacaError> {
        // Crypto only supports: market, limit, stop_limit
        let order_type = match order_type {
            "stop" | "trailing_stop" => {
                log::warn!(
                    "Crypto orders do not support {}, defaulting to market",
                    order_type
                );
                "market"
            }
            other => other,
        };

        // Crypto only supports GTC or IOC
        let time_in_force = match time_in_force {
            "gtc" | "ioc" => time_in_force,
            _ => {
                log::warn!(
                    "Crypto orders only support GTC or IOC, was {}, defaulting to GTC",
                    time_in_force
                );
                "gtc"
            }
        };

        // Crypto doesn't support shorting - warn if selling
        if matches!(self.order.order_side(), OrderSide::Sell) {
            log::debug!("Crypto sell order - ensure position exists (no shorting allowed)");
        }

        // Convert symbol format for crypto (BTC/USD -> BTCUSD)
        let symbol = symbol.replace('/', "");

        Ok((order_type, time_in_force, symbol))
    }

    fn apply_options_constraints<'a>(
        &self,
        order_type: &'a str,
        time_in_force: &'a str,
        symbol: String,
    ) -> Result<(&'a str, &'a str, String), AlpacaError> {
        // Options only support: market, limit
        let order_type = match order_type {
            "stop" | "stop_limit" | "trailing_stop" => {
                log::warn!(
                    "Options orders do not support {}, defaulting to market",
                    order_type
                );
                "market"
            }
            other => other,
        };

        // Options only support day orders
        let time_in_force = if time_in_force != "day" {
            log::warn!(
                "Options only support DAY time-in-force, was {}, defaulting to DAY",
                time_in_force
            );
            "day"
        } else {
            time_in_force
        };

        Ok((order_type, time_in_force, symbol))
    }
}

/// Multi-leg order builder for options spreads.
#[derive(Debug, Clone)]
pub struct MultiLegOrderBuilder {
    /// Order legs.
    legs: Vec<OrderLeg>,
    /// Base quantity.
    quantity: i32,
    /// Order type.
    order_type: String,
    /// Time in force.
    time_in_force: String,
    /// Limit price (optional).
    limit_price: Option<Decimal>,
    /// Client order ID (optional).
    client_order_id: Option<ClientOrderId>,
}

impl MultiLegOrderBuilder {
    /// Creates a new multi-leg order builder.
    pub fn new(quantity: i32) -> Self {
        Self {
            legs: Vec::new(),
            quantity,
            order_type: "market".to_string(),
            time_in_force: "day".to_string(),
            limit_price: None,
            client_order_id: None,
        }
    }

    /// Adds a leg to the order.
    pub fn add_leg(
        mut self,
        symbol: String,
        side: &str,
        ratio_qty: i32,
        position_intent: &str,
    ) -> Self {
        self.legs.push(OrderLeg {
            symbol,
            side: side.to_string(),
            ratio_qty,
            position_intent: Some(position_intent.to_string()),
        });
        self
    }

    /// Sets the order type.
    pub fn order_type(mut self, order_type: String) -> Self {
        self.order_type = order_type;
        self
    }

    /// Sets the time in force.
    pub fn time_in_force(mut self, time_in_force: String) -> Self {
        self.time_in_force = time_in_force;
        self
    }

    /// Sets the limit price.
    pub fn limit_price(mut self, limit_price: Decimal) -> Self {
        self.limit_price = Some(limit_price);
        self
    }

    /// Sets the client order ID.
    pub fn client_order_id(mut self, client_order_id: ClientOrderId) -> Self {
        self.client_order_id = Some(client_order_id);
        self
    }

    /// Builds the order request.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Leg count is invalid (< 2 or > 4)
    /// - Ratio quantities are not in simplest form
    /// - Required fields are missing
    pub fn build(self) -> Result<AlpacaOrderRequest, AlpacaError> {
        // Validate leg count
        if self.legs.is_empty() {
            return Err(AlpacaError::InvalidOrderRequest(
                "No legs provided for multi-leg order".to_string(),
            ));
        }

        if self.legs.len() < 2 {
            return Err(AlpacaError::InvalidOrderRequest(
                "Multi-leg orders require at least 2 legs".to_string(),
            ));
        }

        if self.legs.len() > 4 {
            return Err(AlpacaError::InvalidOrderRequest(
                "Multi-leg orders support a maximum of 4 legs".to_string(),
            ));
        }

        // Validate ratio quantities are in simplest form (GCD = 1)
        let ratio_qtys: Vec<i32> = self.legs.iter().map(|leg| leg.ratio_qty).collect();
        let gcd = ratio_qtys.iter().fold(ratio_qtys[0], |acc, &x| gcd(acc, x));
        if gcd > 1 {
            return Err(AlpacaError::InvalidOrderRequest(format!(
                "Leg ratio_qty values must be in simplest form (GCD=1), but GCD is {}",
                gcd
            )));
        }

        // Validate order type for mleg
        let order_type = match self.order_type.as_str() {
            "market" | "limit" => self.order_type.clone(),
            _ => {
                log::warn!(
                    "Multi-leg orders only support market or limit, was {}, defaulting to market",
                    self.order_type
                );
                "market".to_string()
            }
        };

        // Validate limit price is provided for limit orders
        if order_type == "limit" && self.limit_price.is_none() {
            return Err(AlpacaError::InvalidOrderRequest(
                "Limit price required for limit multi-leg orders".to_string(),
            ));
        }

        Ok(AlpacaOrderRequest {
            symbol: String::new(), // Not used for multi-leg orders
            qty: Some(self.quantity.to_string()),
            notional: None,
            side: String::new(), // Not used for multi-leg orders
            order_type,
            time_in_force: self.time_in_force,
            limit_price: self.limit_price.map(|p| p.to_string()),
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: None,
            client_order_id: self.client_order_id.map(|id| id.to_string()),
            order_class: Some("mleg".to_string()),
            take_profit: None,
            stop_loss: None,
            legs: Some(self.legs),
        })
    }
}

/// Computes the greatest common divisor of two integers.
fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a.abs()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(1, 1), 1);
        assert_eq!(gcd(2, 3), 1);
        assert_eq!(gcd(10, 5), 5);
    }

    #[rstest]
    fn test_multi_leg_builder_invalid_leg_count() {
        let builder = MultiLegOrderBuilder::new(1);
        let result = builder.build();
        assert!(result.is_err());

        let builder = MultiLegOrderBuilder::new(1)
            .add_leg("AAPL250117C00200000".to_string(), "buy", 1, "buy_to_open");
        let result = builder.build();
        assert!(result.is_err());
    }

    #[rstest]
    fn test_multi_leg_builder_gcd_validation() {
        let builder = MultiLegOrderBuilder::new(1)
            .add_leg("AAPL250117C00200000".to_string(), "buy", 2, "buy_to_open")
            .add_leg("AAPL250117C00210000".to_string(), "sell", 2, "sell_to_open");
        let result = builder.build();
        assert!(result.is_err());
    }

    #[rstest]
    fn test_multi_leg_builder_valid() {
        let builder = MultiLegOrderBuilder::new(1)
            .add_leg("AAPL250117C00200000".to_string(), "buy", 1, "buy_to_open")
            .add_leg("AAPL250117C00210000".to_string(), "sell", 1, "sell_to_open")
            .order_type("limit".to_string())
            .limit_price(Decimal::new(100, 2));
        let result = builder.build();
        assert!(result.is_ok());
    }
}
