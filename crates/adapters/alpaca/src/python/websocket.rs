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

//! Python bindings for the Alpaca WebSocket client.

use pyo3::prelude::*;

use crate::{
    common::enums::{AlpacaAssetClass, AlpacaDataFeed},
    websocket::client::AlpacaWebSocketClient,
};

#[pymethods]
impl AlpacaWebSocketClient {
    #[new]
    #[pyo3(signature = (api_key, api_secret, asset_class, data_feed, url_override=None))]
    fn py_new(
        api_key: String,
        api_secret: String,
        asset_class: AlpacaAssetClass,
        data_feed: AlpacaDataFeed,
        url_override: Option<String>,
    ) -> Self {
        Self::new(api_key, api_secret, asset_class, data_feed, url_override)
    }

    fn __repr__(&self) -> String {
        format!(
            "AlpacaWebSocketClient(url='{}', asset_class={:?}, connected={})",
            self.url(),
            self.asset_class(),
            self.is_connected()
        )
    }

    #[getter]
    #[pyo3(name = "url")]
    fn py_url(&self) -> &str {
        self.url()
    }

    #[getter]
    #[pyo3(name = "asset_class")]
    fn py_asset_class(&self) -> AlpacaAssetClass {
        self.asset_class()
    }

    #[getter]
    #[pyo3(name = "is_connected")]
    fn py_is_connected(&self) -> bool {
        self.is_connected()
    }

    /// Get the authentication message for establishing WebSocket connection.
    ///
    /// Returns a JSON string that must be sent after connecting to authenticate.
    #[pyo3(name = "auth_message")]
    fn py_auth_message(&self) -> String {
        self.auth_message()
    }

    /// Create a subscription message for trades.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to subscribe to (e.g., ["AAPL", "MSFT"])
    ///
    /// Returns a JSON string to send via WebSocket to subscribe to trades.
    #[staticmethod]
    #[pyo3(name = "subscribe_trades_message")]
    fn py_subscribe_trades_message(symbols: Vec<String>) -> String {
        Self::subscribe_trades_message(symbols)
    }

    /// Create a subscription message for quotes.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to subscribe to (e.g., ["AAPL", "MSFT"])
    ///
    /// Returns a JSON string to send via WebSocket to subscribe to quotes.
    #[staticmethod]
    #[pyo3(name = "subscribe_quotes_message")]
    fn py_subscribe_quotes_message(symbols: Vec<String>) -> String {
        Self::subscribe_quotes_message(symbols)
    }

    /// Create a subscription message for bars.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to subscribe to (e.g., ["AAPL", "MSFT"])
    ///
    /// Returns a JSON string to send via WebSocket to subscribe to bars.
    #[staticmethod]
    #[pyo3(name = "subscribe_bars_message")]
    fn py_subscribe_bars_message(symbols: Vec<String>) -> String {
        Self::subscribe_bars_message(symbols)
    }

    /// Create an unsubscription message for trades.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to unsubscribe from (e.g., ["AAPL", "MSFT"])
    ///
    /// Returns a JSON string to send via WebSocket to unsubscribe from trades.
    #[staticmethod]
    #[pyo3(name = "unsubscribe_trades_message")]
    fn py_unsubscribe_trades_message(symbols: Vec<String>) -> String {
        Self::unsubscribe_trades_message(symbols)
    }

    /// Create an unsubscription message for quotes.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to unsubscribe from (e.g., ["AAPL", "MSFT"])
    ///
    /// Returns a JSON string to send via WebSocket to unsubscribe from quotes.
    #[staticmethod]
    #[pyo3(name = "unsubscribe_quotes_message")]
    fn py_unsubscribe_quotes_message(symbols: Vec<String>) -> String {
        Self::unsubscribe_quotes_message(symbols)
    }

    /// Create an unsubscription message for bars.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to unsubscribe from (e.g., ["AAPL", "MSFT"])
    ///
    /// Returns a JSON string to send via WebSocket to unsubscribe from bars.
    #[staticmethod]
    #[pyo3(name = "unsubscribe_bars_message")]
    fn py_unsubscribe_bars_message(symbols: Vec<String>) -> String {
        Self::unsubscribe_bars_message(symbols)
    }
}
