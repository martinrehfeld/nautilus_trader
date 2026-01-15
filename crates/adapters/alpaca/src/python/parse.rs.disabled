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

//! Python bindings for Alpaca parsing functions.

use nautilus_core::UnixNanos;
use nautilus_model::{
    data::BarType,
    identifiers::InstrumentId,
};
use pyo3::{prelude::*, types::PyDict};
use serde_json;

use crate::common::{
    models::{AlpacaWsBar, AlpacaWsQuote, AlpacaWsTrade},
    parse,
};

/// Parse an RFC3339 timestamp string to Unix nanoseconds.
///
/// Args:
///     ts_str (str): RFC3339 timestamp string (e.g., "2023-08-25T14:30:00Z")
///
/// Returns:
///     int: Unix timestamp in nanoseconds
///
/// Raises:
///     ValueError: If timestamp parsing fails
#[pyfunction]
#[pyo3(name = "parse_timestamp_ns")]
pub fn py_parse_timestamp_ns(ts_str: &str) -> PyResult<u64> {
    parse::parse_timestamp_ns(ts_str)
        .map(|ts| ts.as_u64())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))
}

/// Parse an Alpaca WebSocket trade message to a Nautilus TradeTick.
///
/// Args:
///     msg (dict): WebSocket message containing trade data
///     instrument_id (InstrumentId): Nautilus instrument ID
///     price_precision (int): Price precision for the instrument
///     size_precision (int): Size precision for the instrument
///     ts_init (int): Initialization timestamp in nanoseconds
///
/// Returns:
///     TradeTick: Parsed trade tick
///
/// Raises:
///     ValueError: If parsing fails
#[pyfunction]
#[pyo3(name = "parse_trade_tick")]
pub fn py_parse_trade_tick(
    py: Python<'_>,
    msg: &Bound<'_, PyDict>,
    instrument_id: InstrumentId,
    price_precision: u8,
    size_precision: u8,
    ts_init: u64,
) -> PyResult<Py<PyAny>> {
    // Convert Python dict to JSON Value using the json module
    let json_module = py.import("json")?;
    let json_str = json_module.call_method1("dumps", (msg,))?;
    let json_str: String = json_str.extract()?;

    let trade: AlpacaWsTrade = serde_json::from_str(&json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to deserialize trade: {e}")))?;

    // Parse using Rust function
    let trade_tick = parse::parse_trade_tick(
        &trade,
        instrument_id,
        price_precision,
        size_precision,
        UnixNanos::from(ts_init),
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;

    // Convert to Python object - Box and cast to PyAny
    let obj = Py::new(py, trade_tick)?;
    Ok(obj.into_any())
}

/// Parse an Alpaca WebSocket quote message to a Nautilus QuoteTick.
///
/// Args:
///     msg (dict): WebSocket message containing quote data
///     instrument_id (InstrumentId): Nautilus instrument ID
///     price_precision (int): Price precision for the instrument
///     size_precision (int): Size precision for the instrument
///     ts_init (int): Initialization timestamp in nanoseconds
///
/// Returns:
///     QuoteTick: Parsed quote tick
///
/// Raises:
///     ValueError: If parsing fails
#[pyfunction]
#[pyo3(name = "parse_quote_tick")]
pub fn py_parse_quote_tick(
    py: Python<'_>,
    msg: &Bound<'_, PyDict>,
    instrument_id: InstrumentId,
    price_precision: u8,
    size_precision: u8,
    ts_init: u64,
) -> PyResult<Py<PyAny>> {
    // Convert Python dict to JSON Value using the json module
    let json_module = py.import("json")?;
    let json_str = json_module.call_method1("dumps", (msg,))?;
    let json_str: String = json_str.extract()?;

    let quote: AlpacaWsQuote = serde_json::from_str(&json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to deserialize quote: {e}")))?;

    // Parse using Rust function
    let quote_tick = parse::parse_quote_tick(
        &quote,
        instrument_id,
        price_precision,
        size_precision,
        UnixNanos::from(ts_init),
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;

    // Convert to Python object - Box and cast to PyAny
    let obj = Py::new(py, quote_tick)?;
    Ok(obj.into_any())
}

/// Parse an Alpaca WebSocket bar message to a Nautilus Bar.
///
/// Args:
///     msg (dict): WebSocket message containing bar data
///     bar_type (BarType): Nautilus bar type
///     price_precision (int): Price precision for the instrument
///     size_precision (int): Size precision for volume
///     ts_init (int): Initialization timestamp in nanoseconds
///
/// Returns:
///     Bar: Parsed bar
///
/// Raises:
///     ValueError: If parsing fails
#[pyfunction]
#[pyo3(name = "parse_bar")]
pub fn py_parse_bar(
    py: Python<'_>,
    msg: &Bound<'_, PyDict>,
    bar_type: BarType,
    price_precision: u8,
    size_precision: u8,
    ts_init: u64,
) -> PyResult<Py<PyAny>> {
    // Convert Python dict to JSON Value using the json module
    let json_module = py.import("json")?;
    let json_str = json_module.call_method1("dumps", (msg,))?;
    let json_str: String = json_str.extract()?;

    let bar_data: AlpacaWsBar = serde_json::from_str(&json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to deserialize bar: {e}")))?;

    // Parse using Rust function
    let bar = parse::parse_bar(
        &bar_data,
        bar_type,
        price_precision,
        size_precision,
        UnixNanos::from(ts_init),
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;

    // Convert to Python object - Box and cast to PyAny
    let obj = Py::new(py, bar)?;
    Ok(obj.into_any())
}

/// Create a BarType from instrument ID and timeframe string.
///
/// Args:
///     instrument_id (InstrumentId): Nautilus instrument ID
///     timeframe (str): Alpaca timeframe (e.g., "1Min", "1Hour", "1Day")
///
/// Returns:
///     BarType: Nautilus bar type
///
/// Raises:
///     ValueError: If timeframe format is invalid
#[pyfunction]
#[pyo3(name = "create_bar_type")]
pub fn py_create_bar_type(
    instrument_id: InstrumentId,
    timeframe: &str,
) -> PyResult<BarType> {
    parse::create_bar_type(instrument_id, timeframe)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))
}
