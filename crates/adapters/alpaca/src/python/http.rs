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

//! Python bindings for the Alpaca HTTP client.

use nautilus_core::python::to_pyvalue_err;
use pyo3::{prelude::*, types::PyList};

use crate::{
    common::enums::AlpacaEnvironment,
    http::{
        client::AlpacaHttpClient,
        models::{AlpacaAccount, AlpacaActivity, AlpacaAsset, AlpacaOrder, AlpacaOrderRequest, AlpacaPosition},
    },
};

#[pymethods]
impl AlpacaHttpClient {
    #[new]
    #[pyo3(signature = (environment, api_key, api_secret, timeout_secs=None, proxy_url=None))]
    fn py_new(
        environment: AlpacaEnvironment,
        api_key: String,
        api_secret: String,
        timeout_secs: Option<u64>,
        proxy_url: Option<String>,
    ) -> PyResult<Self> {
        Self::new(environment, api_key, api_secret, timeout_secs, proxy_url)
            .map_err(to_pyvalue_err)
    }

    fn __repr__(&self) -> String {
        format!(
            "AlpacaHttpClient(trading_url='{}', data_url='{}')",
            self.trading_base_url(),
            self.data_base_url()
        )
    }

    #[getter]
    #[pyo3(name = "trading_base_url")]
    fn py_trading_base_url(&self) -> &str {
        self.trading_base_url()
    }

    #[getter]
    #[pyo3(name = "data_base_url")]
    fn py_data_base_url(&self) -> &str {
        self.data_base_url()
    }

    // ============================================================================
    // Account Endpoints
    // ============================================================================

    /// Get account details.
    ///
    /// Returns the current account information including buying power,
    /// equity, cash, and account status.
    #[pyo3(name = "get_account")]
    fn py_get_account<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let account = client.get_account().await.map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_account = Py::new(py, account)?;
                Ok(py_account.into_any())
            })
        })
    }

    // ============================================================================
    // Asset Endpoints
    // ============================================================================

    /// Get all assets.
    ///
    /// # Arguments
    ///
    /// * `status` - Filter by status (e.g., "active")
    /// * `asset_class` - Filter by asset class (e.g., "us_equity", "crypto")
    #[pyo3(name = "get_assets")]
    #[pyo3(signature = (status=None, asset_class=None))]
    fn py_get_assets<'py>(
        &self,
        py: Python<'py>,
        status: Option<String>,
        asset_class: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let assets = client
                .get_assets(status.as_deref(), asset_class.as_deref())
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_assets: PyResult<Vec<_>> = assets
                    .into_iter()
                    .map(|asset| Py::new(py, asset))
                    .collect();
                let pylist = PyList::new(py, py_assets?).unwrap().into_any().unbind();
                Ok(pylist)
            })
        })
    }

    /// Get a specific asset by symbol or asset ID.
    ///
    /// # Arguments
    ///
    /// * `symbol_or_asset_id` - The symbol (e.g., "AAPL") or asset ID
    #[pyo3(name = "get_asset")]
    fn py_get_asset<'py>(
        &self,
        py: Python<'py>,
        symbol_or_asset_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let asset = client
                .get_asset(&symbol_or_asset_id)
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_asset = Py::new(py, asset)?;
                Ok(py_asset.into_any())
            })
        })
    }

    // ============================================================================
    // Position Endpoints
    // ============================================================================

    /// Get all open positions.
    #[pyo3(name = "get_positions")]
    fn py_get_positions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let positions = client.get_positions().await.map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_positions: PyResult<Vec<_>> = positions
                    .into_iter()
                    .map(|pos| Py::new(py, pos))
                    .collect();
                let pylist = PyList::new(py, py_positions?).unwrap().into_any().unbind();
                Ok(pylist)
            })
        })
    }

    /// Get a specific position by symbol or asset ID.
    ///
    /// # Arguments
    ///
    /// * `symbol_or_asset_id` - The symbol (e.g., "AAPL") or asset ID
    #[pyo3(name = "get_position")]
    fn py_get_position<'py>(
        &self,
        py: Python<'py>,
        symbol_or_asset_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let position = client
                .get_position(&symbol_or_asset_id)
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_position = Py::new(py, position)?;
                Ok(py_position.into_any())
            })
        })
    }

    /// Close a position.
    ///
    /// # Arguments
    ///
    /// * `symbol_or_asset_id` - The symbol or asset ID
    /// * `qty` - Optional quantity to close (None closes entire position)
    /// * `percentage` - Optional percentage to close
    #[pyo3(name = "close_position")]
    #[pyo3(signature = (symbol_or_asset_id, qty=None, percentage=None))]
    fn py_close_position<'py>(
        &self,
        py: Python<'py>,
        symbol_or_asset_id: String,
        qty: Option<String>,
        percentage: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let order = client
                .close_position(
                    &symbol_or_asset_id,
                    qty.as_deref(),
                    percentage.as_deref(),
                )
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_order = Py::new(py, order)?;
                Ok(py_order.into_any())
            })
        })
    }

    /// Close all positions.
    ///
    /// # Arguments
    ///
    /// * `cancel_orders` - Whether to cancel all open orders before closing positions
    #[pyo3(name = "close_all_positions")]
    #[pyo3(signature = (cancel_orders=false))]
    fn py_close_all_positions<'py>(
        &self,
        py: Python<'py>,
        cancel_orders: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let orders = client
                .close_all_positions(cancel_orders)
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_orders: PyResult<Vec<_>> = orders
                    .into_iter()
                    .map(|order| Py::new(py, order))
                    .collect();
                let pylist = PyList::new(py, py_orders?).unwrap().into_any().unbind();
                Ok(pylist)
            })
        })
    }

    // ============================================================================
    // Order Endpoints
    // ============================================================================

    /// Submit a new order.
    ///
    /// # Arguments
    ///
    /// * `request` - Order request details
    #[pyo3(name = "submit_order")]
    fn py_submit_order<'py>(
        &self,
        py: Python<'py>,
        request: AlpacaOrderRequest,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let order = client
                .submit_order(&request)
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_order = Py::new(py, order)?;
                Ok(py_order.into_any())
            })
        })
    }

    /// Get all orders.
    ///
    /// # Arguments
    ///
    /// * `status` - Filter by order status (e.g., "open", "closed", "all")
    /// * `limit` - Maximum number of orders to return
    /// * `after` - Return orders after this timestamp
    /// * `until` - Return orders until this timestamp
    /// * `nested` - If true, roll up multi-leg orders
    #[pyo3(name = "get_orders")]
    #[pyo3(signature = (status=None, limit=None, after=None, until=None, nested=None))]
    fn py_get_orders<'py>(
        &self,
        py: Python<'py>,
        status: Option<String>,
        limit: Option<u32>,
        after: Option<String>,
        until: Option<String>,
        nested: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let orders = client
                .get_orders(
                    status.as_deref(),
                    limit,
                    after.as_deref(),
                    until.as_deref(),
                    nested,
                )
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_orders: PyResult<Vec<_>> = orders
                    .into_iter()
                    .map(|order| Py::new(py, order))
                    .collect();
                let pylist = PyList::new(py, py_orders?).unwrap().into_any().unbind();
                Ok(pylist)
            })
        })
    }

    /// Get a specific order by ID.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID
    /// * `nested` - If true, roll up multi-leg orders
    #[pyo3(name = "get_order")]
    #[pyo3(signature = (order_id, nested=None))]
    fn py_get_order<'py>(
        &self,
        py: Python<'py>,
        order_id: String,
        nested: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let order = client
                .get_order(&order_id, nested)
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_order = Py::new(py, order)?;
                Ok(py_order.into_any())
            })
        })
    }

    /// Cancel an order by ID.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to cancel
    #[pyo3(name = "cancel_order")]
    fn py_cancel_order<'py>(
        &self,
        py: Python<'py>,
        order_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .cancel_order(&order_id)
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| Ok(py.None()))
        })
    }

    /// Cancel all open orders.
    #[pyo3(name = "cancel_all_orders")]
    fn py_cancel_all_orders<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let orders = client.cancel_all_orders().await.map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_orders: PyResult<Vec<_>> = orders
                    .into_iter()
                    .map(|order| Py::new(py, order))
                    .collect();
                let pylist = PyList::new(py, py_orders?).unwrap().into_any().unbind();
                Ok(pylist)
            })
        })
    }

    // ============================================================================
    // Activity Endpoints
    // ============================================================================

    /// Get account activities.
    ///
    /// # Arguments
    ///
    /// * `activity_types` - Optional comma-separated list of activity types (e.g., "FILL")
    /// * `after` - Return activities after this timestamp (RFC3339 format)
    /// * `until` - Return activities until this timestamp (RFC3339 format)
    /// * `page_size` - Number of activities per page
    #[pyo3(name = "get_activities")]
    #[pyo3(signature = (activity_types=None, after=None, until=None, page_size=None))]
    fn py_get_activities<'py>(
        &self,
        py: Python<'py>,
        activity_types: Option<String>,
        after: Option<String>,
        until: Option<String>,
        page_size: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let activities = client
                .get_activities(
                    activity_types.as_deref(),
                    after.as_deref(),
                    until.as_deref(),
                    page_size,
                )
                .await
                .map_err(to_pyvalue_err)?;

            Python::attach(|py| {
                let py_activities: PyResult<Vec<_>> = activities
                    .into_iter()
                    .map(|activity| Py::new(py, activity))
                    .collect();
                let pylist = PyList::new(py, py_activities?).unwrap().into_any().unbind();
                Ok(pylist)
            })
        })
    }

}

// Python bindings for HTTP models
#[pymethods]
impl AlpacaAccount {
    fn __repr__(&self) -> String {
        format!(
            "AlpacaAccount(id='{}', account_number='{}', status='{}')",
            self.id, self.account_number, self.status
        )
    }

    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    #[getter]
    fn account_number(&self) -> &str {
        &self.account_number
    }

    #[getter]
    fn status(&self) -> &str {
        &self.status
    }

    #[getter]
    fn currency(&self) -> &str {
        &self.currency
    }

    #[getter]
    fn cash(&self) -> &str {
        &self.cash
    }

    #[getter]
    fn portfolio_value(&self) -> &str {
        &self.portfolio_value
    }

    #[getter]
    fn buying_power(&self) -> &str {
        &self.buying_power
    }

    #[getter]
    fn equity(&self) -> &str {
        &self.equity
    }

    #[getter]
    fn initial_margin(&self) -> &str {
        &self.initial_margin
    }

    #[getter]
    fn maintenance_margin(&self) -> &str {
        &self.maintenance_margin
    }
}

#[pymethods]
impl AlpacaPosition {
    fn __repr__(&self) -> String {
        format!(
            "AlpacaPosition(symbol='{}', qty='{}', market_value='{}')",
            self.symbol, self.qty, self.market_value
        )
    }

    #[getter]
    fn asset_id(&self) -> &str {
        &self.asset_id
    }

    #[getter]
    fn symbol(&self) -> &str {
        &self.symbol
    }

    #[getter]
    fn exchange(&self) -> &str {
        &self.exchange
    }

    #[getter]
    fn asset_class(&self) -> &str {
        &self.asset_class
    }

    #[getter]
    fn qty(&self) -> &str {
        &self.qty
    }

    #[getter]
    fn avg_entry_price(&self) -> &str {
        &self.avg_entry_price
    }

    #[getter]
    fn side(&self) -> &str {
        &self.side
    }

    #[getter]
    fn market_value(&self) -> &str {
        &self.market_value
    }

    #[getter]
    fn cost_basis(&self) -> &str {
        &self.cost_basis
    }

    #[getter]
    fn unrealized_pl(&self) -> &str {
        &self.unrealized_pl
    }

    #[getter]
    fn unrealized_plpc(&self) -> &str {
        &self.unrealized_plpc
    }

    #[getter]
    fn current_price(&self) -> &str {
        &self.current_price
    }
}

#[pymethods]
impl AlpacaOrder {
    fn __repr__(&self) -> String {
        format!(
            "AlpacaOrder(id='{}', symbol='{}', side='{}', type='{}', status='{}')",
            self.id, self.symbol, self.side, self.order_type, self.status
        )
    }

    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    #[getter]
    fn client_order_id(&self) -> &str {
        &self.client_order_id
    }

    #[getter]
    fn symbol(&self) -> &str {
        &self.symbol
    }

    #[getter]
    fn side(&self) -> &str {
        &self.side
    }

    #[getter]
    fn order_type(&self) -> &str {
        &self.order_type
    }

    #[getter]
    fn qty(&self) -> Option<&str> {
        self.qty.as_deref()
    }

    #[getter]
    fn notional(&self) -> Option<&str> {
        self.notional.as_deref()
    }

    #[getter]
    fn filled_qty(&self) -> &str {
        &self.filled_qty
    }

    #[getter]
    fn status(&self) -> &str {
        &self.status
    }

    #[getter]
    fn time_in_force(&self) -> &str {
        &self.time_in_force
    }

    #[getter]
    fn limit_price(&self) -> Option<&str> {
        self.limit_price.as_deref()
    }

    #[getter]
    fn stop_price(&self) -> Option<&str> {
        self.stop_price.as_deref()
    }

    #[getter]
    fn filled_avg_price(&self) -> Option<&str> {
        self.filled_avg_price.as_deref()
    }
}

#[pymethods]
impl AlpacaOrderRequest {
    #[new]
    #[pyo3(signature = (
        symbol,
        side,
        order_type,
        time_in_force,
        qty=None,
        notional=None,
        limit_price=None,
        stop_price=None,
        trail_price=None,
        trail_percent=None,
        extended_hours=None,
        client_order_id=None,
        order_class=None
    ))]
    #[allow(clippy::too_many_arguments)]
    fn py_new(
        symbol: String,
        side: String,
        order_type: String,
        time_in_force: String,
        qty: Option<String>,
        notional: Option<String>,
        limit_price: Option<String>,
        stop_price: Option<String>,
        trail_price: Option<String>,
        trail_percent: Option<String>,
        extended_hours: Option<bool>,
        client_order_id: Option<String>,
        order_class: Option<String>,
    ) -> Self {
        Self {
            symbol,
            side,
            order_type,
            time_in_force,
            qty,
            notional,
            limit_price,
            stop_price,
            trail_price,
            trail_percent,
            extended_hours,
            client_order_id,
            order_class,
            take_profit: None, // TODO: Support TakeProfitSpec in Python
            stop_loss: None,   // TODO: Support StopLossSpec in Python
            legs: None,        // TODO: Support OrderLeg in Python
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "AlpacaOrderRequest(symbol='{}', side='{}', type='{}', qty={:?})",
            self.symbol, self.side, self.order_type, self.qty
        )
    }
}

#[pymethods]
impl AlpacaAsset {
    fn __repr__(&self) -> String {
        format!(
            "AlpacaAsset(symbol='{}', name='{}', class='{}', status='{}')",
            self.symbol, self.name, self.class, self.status
        )
    }

    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    #[getter]
    fn class(&self) -> &str {
        &self.class
    }

    #[getter]
    fn exchange(&self) -> &str {
        &self.exchange
    }

    #[getter]
    fn symbol(&self) -> &str {
        &self.symbol
    }

    #[getter]
    fn name(&self) -> &str {
        &self.name
    }

    #[getter]
    fn status(&self) -> &str {
        &self.status
    }

    #[getter]
    fn tradable(&self) -> bool {
        self.tradable
    }

    #[getter]
    fn marginable(&self) -> bool {
        self.marginable
    }

    #[getter]
    fn shortable(&self) -> bool {
        self.shortable
    }

    #[getter]
    fn easy_to_borrow(&self) -> bool {
        self.easy_to_borrow
    }

    #[getter]
    fn fractionable(&self) -> bool {
        self.fractionable
    }

    #[getter]
    fn maintenance_margin_requirement(&self) -> Option<f64> {
        self.maintenance_margin_requirement
    }

    #[getter]
    fn min_order_size(&self) -> Option<&str> {
        self.min_order_size.as_deref()
    }

    #[getter]
    fn min_trade_increment(&self) -> Option<&str> {
        self.min_trade_increment.as_deref()
    }

    #[getter]
    fn price_increment(&self) -> Option<&str> {
        self.price_increment.as_deref()
    }
}

#[pymethods]
impl AlpacaActivity {
    fn __repr__(&self) -> String {
        format!(
            "AlpacaActivity(id='{}', type='{}', symbol={:?})",
            self.id, self.activity_type, self.symbol
        )
    }

    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    #[getter]
    fn activity_type(&self) -> &str {
        &self.activity_type
    }

    #[getter]
    fn transaction_time(&self) -> &str {
        &self.transaction_time
    }

    #[getter]
    fn fill_type(&self) -> Option<&str> {
        self.fill_type.as_deref()
    }

    #[getter]
    fn order_id(&self) -> Option<&str> {
        self.order_id.as_deref()
    }

    #[getter]
    fn symbol(&self) -> Option<&str> {
        self.symbol.as_deref()
    }

    #[getter]
    fn side(&self) -> Option<&str> {
        self.side.as_deref()
    }

    #[getter]
    fn qty(&self) -> Option<&str> {
        self.qty.as_deref()
    }

    #[getter]
    fn price(&self) -> Option<&str> {
        self.price.as_deref()
    }

    #[getter]
    fn leaves_qty(&self) -> Option<&str> {
        self.leaves_qty.as_deref()
    }

    #[getter]
    fn cum_qty(&self) -> Option<&str> {
        self.cum_qty.as_deref()
    }
}
