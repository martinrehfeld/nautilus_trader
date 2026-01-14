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

//! Alpaca-specific enumeration types.

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Alpaca trading environment.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, AsRefStr, EnumIter,
)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AlpacaEnvironment {
    /// Live trading environment.
    #[default]
    Live,
    /// Paper trading environment (simulated).
    Paper,
}

/// Alpaca asset class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, AsRefStr, EnumIter)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlpacaAssetClass {
    /// US Equities (stocks and ETFs).
    UsEquity,
    /// Cryptocurrency.
    Crypto,
    /// Options contracts.
    Option,
}

/// Alpaca market data feed subscription level.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, AsRefStr, EnumIter,
)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AlpacaDataFeed {
    /// IEX exchange data (free tier, may be delayed).
    #[default]
    Iex,
    /// SIP consolidated feed (paid tier, real-time).
    Sip,
}

/// Alpaca order side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, AsRefStr, EnumIter)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AlpacaOrderSide {
    /// Buy order.
    Buy,
    /// Sell order.
    Sell,
}

/// Alpaca order type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, AsRefStr, EnumIter)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlpacaOrderType {
    /// Market order - execute immediately at best available price.
    Market,
    /// Limit order - execute at specified price or better.
    Limit,
    /// Stop order - becomes market order when stop price is reached.
    Stop,
    /// Stop-limit order - becomes limit order when stop price is reached.
    StopLimit,
    /// Trailing stop order - stop price trails market by specified amount.
    TrailingStop,
}

/// Alpaca time-in-force for orders.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, AsRefStr, EnumIter,
)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AlpacaTimeInForce {
    /// Day order - valid for current trading day only.
    #[default]
    Day,
    /// Good-till-canceled - remains active until filled or canceled.
    Gtc,
    /// On-open - executes at market open.
    Opg,
    /// On-close - executes at market close.
    Cls,
    /// Immediate-or-cancel - execute immediately or cancel.
    Ioc,
    /// Fill-or-kill - execute entire order immediately or cancel.
    Fok,
}

/// Alpaca order status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, AsRefStr, EnumIter)]
#[cfg_attr(feature = "python", pyclass(module = "nautilus_pyo3.alpaca"))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlpacaOrderStatus {
    /// Order has been received but not yet accepted.
    New,
    /// Order has been accepted by the exchange.
    Accepted,
    /// Order has been partially filled.
    PartiallyFilled,
    /// Order has been completely filled.
    Filled,
    /// Order cancellation request is pending.
    PendingCancel,
    /// Order has been canceled.
    Canceled,
    /// Order has been rejected.
    Rejected,
    /// Order has expired.
    Expired,
    /// Order replacement is pending.
    PendingReplace,
    /// Order has been replaced.
    Replaced,
    /// Order is pending new (not yet submitted).
    PendingNew,
    /// Order has been stopped.
    Stopped,
    /// Order has been suspended.
    Suspended,
    /// Order is calculated (for complex orders).
    Calculated,
    /// Order is held.
    Held,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_default() {
        assert_eq!(AlpacaEnvironment::default(), AlpacaEnvironment::Live);
    }

    #[test]
    fn test_data_feed_default() {
        assert_eq!(AlpacaDataFeed::default(), AlpacaDataFeed::Iex);
    }

    #[test]
    fn test_time_in_force_default() {
        assert_eq!(AlpacaTimeInForce::default(), AlpacaTimeInForce::Day);
    }

    #[test]
    fn test_asset_class_serialization() {
        let asset_class = AlpacaAssetClass::UsEquity;
        let json = serde_json::to_string(&asset_class).unwrap();
        assert_eq!(json, "\"us_equity\"");
    }

    #[test]
    fn test_order_type_serialization() {
        let order_type = AlpacaOrderType::StopLimit;
        let json = serde_json::to_string(&order_type).unwrap();
        assert_eq!(json, "\"stop_limit\"");
    }
}
