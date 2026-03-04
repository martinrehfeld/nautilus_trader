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

use std::sync::Arc;

use nautilus_core::{python::clone_py_object};
use nautilus_network::{python::websocket::WebSocketClientError, ratelimiter::quota::Quota, websocket::{MessageHandler, PingHandler}};
use pyo3::{prelude::*, types::PyBytes};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    common::enums::{AlpacaAssetClass, AlpacaDataFeed},
    websocket::client::AlpacaWebSocketClient,
};


fn to_websocket_pyerr(e: anyhow::Error) -> PyErr {
    PyErr::new::<WebSocketClientError, _>(e.to_string())
}

#[pymethods]
impl AlpacaWebSocketClient {
    /// Create a connected websocket client.
    ///
    /// The handler and ping_handler callbacks are scheduled on the provided event loop
    /// using `call_soon_threadsafe` to ensure they execute on the correct thread.
    /// This is critical for thread safety since WebSocket messages arrive on
    /// a Tokio worker thread, but Python callbacks (like those entering the
    /// kernel via MessageBus) must run on the asyncio event loop thread.
    ///
    /// # Safety
    ///
    /// - Throws an Exception if it is unable to make websocket connection.
    #[staticmethod]
    #[pyo3(name = "connect", signature = (loop_, paper_trading, api_key, api_secret, asset_class, data_feed, url_override, handler, ping_handler = None, post_reconnection = None, keyed_quotas = Vec::new(), default_quota = None))]
    #[allow(clippy::too_many_arguments)]
    fn py_connect(
        loop_: Py<PyAny>,
        paper_trading: bool,
        api_key: String,
        api_secret: String,
        asset_class: AlpacaAssetClass,
        data_feed: AlpacaDataFeed,
        url_override: Option<String>,
        handler: Py<PyAny>,
        ping_handler: Option<Py<PyAny>>,
        post_reconnection: Option<Py<PyAny>>,
        keyed_quotas: Vec<(String, Quota)>,
        default_quota: Option<Quota>,
        py: Python<'_>,
    ) -> PyResult<Bound<'_, PyAny>> {
        let environment = if paper_trading {
            crate::common::AlpacaEnvironment::Paper
        } else {
            crate::common::AlpacaEnvironment::Live
        };

        let call_soon_threadsafe: Py<PyAny> = loop_.getattr(py, "call_soon_threadsafe")?;
        let call_soon_clone = clone_py_object(&call_soon_threadsafe);
        let handler_clone = clone_py_object(&handler);

        let message_handler: MessageHandler = Arc::new(move |msg: Message| {
            Python::attach(|py| {
                let py_bytes = match &msg {
                    Message::Binary(data) => PyBytes::new(py, data),
                    Message::Text(text) => PyBytes::new(py, text.as_bytes()),
                    _ => return,
                };

                if let Err(e) = call_soon_clone.call1(py, (&handler_clone, py_bytes)) {
                    log::error!("Error scheduling message handler on event loop: {e}");
                }
            });
        });

        let ping_handler_fn = ping_handler.map(|ping_handler| {
            let ping_handler_clone = clone_py_object(&ping_handler);
            let call_soon_clone = clone_py_object(&call_soon_threadsafe);

            let ping_handler_fn: PingHandler = Arc::new(move |data: Vec<u8>| {
                Python::attach(|py| {
                    let py_bytes = PyBytes::new(py, &data);
                    if let Err(e) = call_soon_clone.call1(py, (&ping_handler_clone, py_bytes)) {
                        log::error!("Error scheduling ping handler on event loop: {e}");
                    }
                });
            });
            ping_handler_fn
        });

        let post_reconnection_fn = post_reconnection.map(|callback| {
            let callback_clone = clone_py_object(&callback);
            Arc::new(move || {
                Python::attach(|py| {
                    if let Err(e) = callback_clone.call0(py) {
                        log::error!("Error calling post_reconnection handler: {e}");
                    }
                });
            }) as Arc<dyn Fn() + Send + Sync>
        });

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            Self::connect(
                environment,
                api_key,
                api_secret,
                asset_class,
                data_feed,
                url_override,
                message_handler,
                ping_handler_fn,
                post_reconnection_fn,
                keyed_quotas,
                default_quota,
            )
            .await
            .map_err(to_websocket_pyerr)
        })
    }

    #[new]
    #[pyo3(signature = (paper_trading, api_key, api_secret, asset_class, data_feed, url_override=None))]
    fn py_new(
        paper_trading: bool,
        api_key: String,
        api_secret: String,
        asset_class: AlpacaAssetClass,
        data_feed: AlpacaDataFeed,
        url_override: Option<String>,
    ) -> Self {
        let environment = if paper_trading {
            crate::common::AlpacaEnvironment::Paper
        } else {
            crate::common::AlpacaEnvironment::Live
        };
        Self::new(environment, api_key, api_secret, asset_class, data_feed, url_override)
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

    /// Disconnect the client
    #[pyo3(name = "disconnect")]
    fn py_disconnect(&self) -> () {
        self.disconnect();
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

    /// Create a subscription message for orderbooks.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to subscribe to (e.g., ["AAPL", "MSFT"])
    ///
    /// Returns a JSON string to send via WebSocket to subscribe to orderbooks.
    #[staticmethod]
    #[pyo3(name = "subscribe_orderbooks_message")]
    fn py_subscribe_orderbooks_message(symbols: Vec<String>) -> String {
        Self::subscribe_orderbooks_message(symbols)
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

    /// Create an unsubscription message for orderbooks.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to unsubscribe from (e.g., ["AAPL", "MSFT"])
    ///
    /// Returns a JSON string to send via WebSocket to unsubscribe from orderbooks.
    #[staticmethod]
    #[pyo3(name = "unsubscribe_orderbooks_message")]
    fn py_unsubscribe_orderbooks_message(symbols: Vec<String>) -> String {
        Self::unsubscribe_orderbooks_message(symbols)
    }
}
