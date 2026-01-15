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

//! Python bindings for the Alpaca adapter.

pub mod enums;
pub mod http;
pub mod margin;
// pub mod parse;  // Disabled - file renamed to parse.rs.disabled
pub mod types;
pub mod websocket;

use pyo3::prelude::*;

use crate::common::consts::ALPACA_NAUTILUS_BROKER_ID;

/// Alpaca adapter Python module.
///
/// Loaded as `nautilus_pyo3.alpaca`.
#[pymodule]
#[rustfmt::skip]
pub fn alpaca(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Constants
    m.add(stringify!(ALPACA_NAUTILUS_BROKER_ID), ALPACA_NAUTILUS_BROKER_ID)?;
    // Add ALPACA_VENUE as an alias for backward compatibility
    m.add("ALPACA_VENUE", ALPACA_NAUTILUS_BROKER_ID)?;

    // Enumerations
    m.add_class::<crate::common::enums::AlpacaEnvironment>()?;
    m.add_class::<crate::common::enums::AlpacaAssetClass>()?;
    m.add_class::<crate::common::enums::AlpacaDataFeed>()?;
    m.add_class::<crate::common::enums::AlpacaOrderSide>()?;
    m.add_class::<crate::common::enums::AlpacaOrderType>()?;
    m.add_class::<crate::common::enums::AlpacaTimeInForce>()?;
    m.add_class::<crate::common::enums::AlpacaOrderStatus>()?;

    // Configuration types
    m.add_class::<crate::config::AlpacaInstrumentProviderConfig>()?;
    m.add_class::<crate::config::AlpacaDataClientConfig>()?;
    m.add_class::<crate::config::AlpacaExecClientConfig>()?;

    // HTTP client and models
    m.add_class::<crate::http::client::AlpacaHttpClient>()?;
    m.add_class::<crate::http::models::AlpacaAccount>()?;
    m.add_class::<crate::http::models::AlpacaAsset>()?;
    m.add_class::<crate::http::models::AlpacaPosition>()?;
    m.add_class::<crate::http::models::AlpacaOrder>()?;
    m.add_class::<crate::http::models::AlpacaOrderRequest>()?;
    m.add_class::<crate::http::models::AlpacaActivity>()?;

    // WebSocket client
    m.add_class::<crate::websocket::client::AlpacaWebSocketClient>()?;

    // Margin calculator and related types
    m.add_class::<margin::AlpacaOptionsMarginCalculator>()?;
    m.add_class::<margin::OptionPosition>()?;
    m.add_class::<margin::OrderLeg>()?;
    m.add_class::<margin::CostBasisResult>()?;
    m.add_class::<margin::MarginValidationResult>()?;

    // Parsing functions - Commented out, not used in Python layer
    // m.add_function(wrap_pyfunction!(parse::py_parse_timestamp_ns, m)?)?;
    // m.add_function(wrap_pyfunction!(parse::py_parse_trade_tick, m)?)?;
    // m.add_function(wrap_pyfunction!(parse::py_parse_quote_tick, m)?)?;
    // m.add_function(wrap_pyfunction!(parse::py_parse_bar, m)?)?;
    // m.add_function(wrap_pyfunction!(parse::py_create_bar_type, m)?)?;

    // Note: AlpacaInstrumentProvider, AlpacaDataClient and AlpacaExecutionClient are implemented in Python
    // (nautilus_trader/adapters/alpaca/data.py and execution.py) following the
    // hybrid architecture pattern used in OKX/Bybit adapters. Rust provides the
    // HTTP/WebSocket clients, and Python provides the framework integration.

    Ok(())
}
