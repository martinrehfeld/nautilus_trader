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

//! Python bindings for Alpaca execution client.

use nautilus_model::identifiers::{AccountId, ClientId};
use pyo3::prelude::*;

use crate::{
    common::enums::AlpacaEnvironment,
    config::AlpacaExecClientConfig,
    execution::{AlpacaExecClientConfig as RustExecConfig, AlpacaExecutionClient as RustAlpacaExecutionClient},
    http::client::AlpacaHttpClient,
};

/// Alpaca execution client for order submission and position management.
///
/// Supports:
/// - US equities (stocks and ETFs)
/// - Cryptocurrencies (BTC/USD, ETH/USD, etc.) with GTC/IOC constraints
/// - Options (equity options) with DAY order constraint
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.alpaca")]
pub struct AlpacaExecutionClient {
    pub(crate) inner: RustAlpacaExecutionClient,
}

#[pymethods]
impl AlpacaExecutionClient {
    #[new]
    fn new(
        client_id: String,
        account_id: String,
        config: PyRef<AlpacaExecClientConfig>,
    ) -> PyResult<Self> {
        let rust_config = RustExecConfig {
            api_key: config.api_key.clone().unwrap_or_default(),
            api_secret: config.api_secret.clone().unwrap_or_default(),
            environment: if config.paper_trading {
                AlpacaEnvironment::Paper
            } else {
                AlpacaEnvironment::Live
            },
            timeout_secs: config.http_timeout_secs,
            proxy_url: config.proxy_url.clone(),
        };

        // Create HTTP client
        let http_client = AlpacaHttpClient::new(
            rust_config.environment,
            rust_config.api_key.clone(),
            rust_config.api_secret.clone(),
            rust_config.timeout_secs,
            rust_config.proxy_url.clone(),
        )
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create HTTP client: {:?}",
                e
            ))
        })?;

        // Parse identifiers
        let client_id = ClientId::from(client_id.as_str());
        let account_id = AccountId::from(account_id.as_str());

        match RustAlpacaExecutionClient::new(client_id, account_id, http_client, rust_config) {
            Ok(client) => Ok(Self { inner: client }),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to create execution client: {:?}", e),
            )),
        }
    }

    fn __repr__(&self) -> String {
        "AlpacaExecutionClient()".to_string()
    }
}

/// Factory function to create an Alpaca execution client.
///
/// This function is called from Python factories to create the client with proper
/// integration into NautilusTrader's event loop and message bus.
#[pyfunction]
#[pyo3(signature = (loop_, name, config, msgbus, cache, clock))]
pub fn create_alpaca_exec_client(
    _loop: PyObject,
    name: String,
    config: PyRef<AlpacaExecClientConfig>,
    _msgbus: PyObject,
    _cache: PyObject,
    _clock: PyObject,
) -> PyResult<AlpacaExecutionClient> {
    // Create account ID from name
    let account_id = format!("ALPACA-{}", name);

    // For now, create a basic client
    // TODO: Integrate with msgbus, clock, and event loop properly
    AlpacaExecutionClient::new(name, account_id, config)
}
