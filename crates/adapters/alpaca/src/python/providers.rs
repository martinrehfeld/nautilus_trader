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

//! Python bindings for Alpaca instrument provider.

use pyo3::prelude::*;

use crate::{
    http::client::AlpacaHttpClient,
    providers::AlpacaInstrumentProvider as RustAlpacaInstrumentProvider,
};

/// Alpaca instrument provider.
///
/// Loads instruments for US equities, cryptocurrencies, and options from Alpaca Markets.
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.alpaca")]
pub struct AlpacaInstrumentProvider {
    inner: RustAlpacaInstrumentProvider,
}

#[pymethods]
impl AlpacaInstrumentProvider {
    #[new]
    fn new(
        http_client: PyRef<AlpacaHttpClient>,
        config: PyRef<crate::config::AlpacaInstrumentProviderConfig>,
    ) -> PyResult<Self> {
        // Convert Python config to Rust config
        let rust_config = crate::providers::AlpacaInstrumentProviderConfig {
            asset_classes: config.asset_classes.clone().unwrap_or_default(),
            option_underlying_symbols: config.option_underlying_symbols.clone(),
            venue: nautilus_model::identifiers::Venue::from("ALPACA"),
            log_warnings: config.log_warnings,
        };

        // Clone the HTTP client
        let client = http_client.inner.clone();

        Ok(Self {
            inner: RustAlpacaInstrumentProvider::new(client, rust_config),
        })
    }

    /// Load all instruments for the configured asset classes.
    ///
    /// Returns
    /// -------
    /// list[InstrumentAny]
    ///     List of loaded instruments
    fn load_all(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        // This needs to be async, so we'll need to run it on the tokio runtime
        let instruments = py.allow_threads(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { self.inner.load_all().await })
        });

        match instruments {
            Ok(instruments) => {
                // Convert Vec<InstrumentAny> to Python list
                let py_list = pyo3::types::PyList::empty_bound(py);
                for instrument in instruments {
                    // TODO: Convert InstrumentAny to Python object
                    // This requires the instrument to be wrapped in a PyClass
                    // For now, we'll return an error indicating this needs implementation
                    return Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                        "Instrument conversion to Python not yet implemented",
                    ));
                }
                Ok(py_list.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to load instruments: {:?}", e),
            )),
        }
    }

    fn __repr__(&self) -> String {
        "AlpacaInstrumentProvider()".to_string()
    }
}
