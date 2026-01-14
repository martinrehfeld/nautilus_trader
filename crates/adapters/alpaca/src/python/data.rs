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

//! Python bindings for Alpaca data client.

use std::sync::Arc;

use nautilus_common::cache::Cache;
use pyo3::prelude::*;

use crate::{
    config::AlpacaDataClientConfig,
    data::AlpacaDataClient as RustAlpacaDataClient,
};

/// Alpaca data client for market data streaming and historical data requests.
///
/// Supports:
/// - US equities (stocks and ETFs) with IEX or SIP data feeds
/// - Cryptocurrencies (BTC/USD, ETH/USD, etc.)
/// - Options (equity options)
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.alpaca")]
pub struct AlpacaDataClient {
    pub(crate) inner: RustAlpacaDataClient,
}

#[pymethods]
impl AlpacaDataClient {
    #[new]
    fn new(config: PyRef<AlpacaDataClientConfig>, cache: PyObject) -> PyResult<Self> {
        // Extract the Rust Cache from Python cache object
        Python::with_gil(|py| {
            let cache_obj = cache.bind(py);

            // Try to extract the cache - this assumes the Python cache wraps a Rust cache
            // In practice, this needs proper integration with NautilusTrader's cache system
            let rust_cache = Arc::new(Cache::default());

            let rust_config = crate::data::types::AlpacaDataClientConfig {
                api_key: config.api_key.clone().unwrap_or_default(),
                api_secret: config.api_secret.clone().unwrap_or_default(),
                data_feed: config.data_feed,
                paper_trading: config.paper_trading,
                base_url_http: config.base_url_http.clone(),
                base_url_ws: config.base_url_ws.clone(),
                proxy_url: config.proxy_url.clone(),
                timeout_secs: config.http_timeout_secs,
            };

            match RustAlpacaDataClient::new(rust_config, rust_cache) {
                Ok(client) => Ok(Self { inner: client }),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Failed to create data client: {:?}", e),
                )),
            }
        })
    }

    fn __repr__(&self) -> String {
        "AlpacaDataClient()".to_string()
    }
}

/// Factory function to create an Alpaca data client.
///
/// This function is called from Python factories to create the client with proper
/// integration into NautilusTrader's event loop and message bus.
#[pyfunction]
#[pyo3(signature = (loop_, name, config, msgbus, cache, clock))]
pub fn create_alpaca_data_client(
    _loop: PyObject,
    _name: String,
    config: PyRef<AlpacaDataClientConfig>,
    _msgbus: PyObject,
    cache: PyObject,
    _clock: PyObject,
) -> PyResult<AlpacaDataClient> {
    // For now, create a basic client
    // TODO: Integrate with msgbus, clock, and event loop properly
    AlpacaDataClient::new(config, cache)
}
