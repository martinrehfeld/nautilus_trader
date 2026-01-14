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

//! Alpaca enumerations Python bindings.

use std::str::FromStr;

use nautilus_core::python::to_pyvalue_err;
use pyo3::{PyTypeInfo, prelude::*, types::PyType};
use strum::IntoEnumIterator;

use crate::common::enums::{
    AlpacaAssetClass, AlpacaDataFeed, AlpacaEnvironment, AlpacaOrderSide, AlpacaOrderStatus,
    AlpacaOrderType, AlpacaTimeInForce,
};

#[pymethods]
impl AlpacaEnvironment {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!(
            "<{}.{}: '{}'>",
            stringify!(AlpacaEnvironment),
            self.name(),
            self.value(),
        )
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    #[getter]
    #[must_use]
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    #[getter]
    #[must_use]
    pub fn value(&self) -> String {
        self.to_string().to_lowercase()
    }

    #[staticmethod]
    #[must_use]
    fn variants() -> Vec<String> {
        Self::iter().map(|x| x.to_string()).collect()
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyAny>) -> PyResult<Self> {
        let data_str: String = data.str()?.extract()?;
        Self::from_str(&data_str).map_err(to_pyvalue_err)
    }

    #[classattr]
    #[pyo3(name = "LIVE")]
    fn py_live() -> Self {
        Self::Live
    }

    #[classattr]
    #[pyo3(name = "PAPER")]
    fn py_paper() -> Self {
        Self::Paper
    }
}

#[pymethods]
impl AlpacaAssetClass {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!(
            "<{}.{}: '{}'>",
            stringify!(AlpacaAssetClass),
            self.name(),
            self.value(),
        )
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    #[getter]
    #[must_use]
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    #[getter]
    #[must_use]
    pub fn value(&self) -> String {
        self.to_string()
    }

    #[staticmethod]
    #[must_use]
    fn variants() -> Vec<String> {
        Self::iter().map(|x| x.to_string()).collect()
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyAny>) -> PyResult<Self> {
        let data_str: String = data.str()?.extract()?;
        Self::from_str(&data_str).map_err(to_pyvalue_err)
    }

    #[classattr]
    #[pyo3(name = "US_EQUITY")]
    fn py_us_equity() -> Self {
        Self::UsEquity
    }

    #[classattr]
    #[pyo3(name = "CRYPTO")]
    fn py_crypto() -> Self {
        Self::Crypto
    }

    #[classattr]
    #[pyo3(name = "OPTION")]
    fn py_option() -> Self {
        Self::Option
    }
}

#[pymethods]
impl AlpacaDataFeed {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!(
            "<{}.{}: '{}'>",
            stringify!(AlpacaDataFeed),
            self.name(),
            self.value(),
        )
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    #[getter]
    #[must_use]
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    #[getter]
    #[must_use]
    pub fn value(&self) -> String {
        self.to_string().to_lowercase()
    }

    #[staticmethod]
    #[must_use]
    fn variants() -> Vec<String> {
        Self::iter().map(|x| x.to_string()).collect()
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyAny>) -> PyResult<Self> {
        let data_str: String = data.str()?.extract()?;
        Self::from_str(&data_str).map_err(to_pyvalue_err)
    }

    #[classattr]
    #[pyo3(name = "IEX")]
    fn py_iex() -> Self {
        Self::Iex
    }

    #[classattr]
    #[pyo3(name = "SIP")]
    fn py_sip() -> Self {
        Self::Sip
    }
}

#[pymethods]
impl AlpacaOrderSide {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!(
            "<{}.{}: '{}'>",
            stringify!(AlpacaOrderSide),
            self.name(),
            self.value(),
        )
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    #[getter]
    #[must_use]
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    #[getter]
    #[must_use]
    pub fn value(&self) -> String {
        self.to_string().to_lowercase()
    }

    #[staticmethod]
    #[must_use]
    fn variants() -> Vec<String> {
        Self::iter().map(|x| x.to_string()).collect()
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyAny>) -> PyResult<Self> {
        let data_str: String = data.str()?.extract()?;
        Self::from_str(&data_str).map_err(to_pyvalue_err)
    }

    #[classattr]
    #[pyo3(name = "BUY")]
    fn py_buy() -> Self {
        Self::Buy
    }

    #[classattr]
    #[pyo3(name = "SELL")]
    fn py_sell() -> Self {
        Self::Sell
    }
}

#[pymethods]
impl AlpacaOrderType {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!(
            "<{}.{}: '{}'>",
            stringify!(AlpacaOrderType),
            self.name(),
            self.value(),
        )
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    #[getter]
    #[must_use]
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    #[getter]
    #[must_use]
    pub fn value(&self) -> String {
        self.to_string()
    }

    #[staticmethod]
    #[must_use]
    fn variants() -> Vec<String> {
        Self::iter().map(|x| x.to_string()).collect()
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyAny>) -> PyResult<Self> {
        let data_str: String = data.str()?.extract()?;
        Self::from_str(&data_str).map_err(to_pyvalue_err)
    }

    #[classattr]
    #[pyo3(name = "MARKET")]
    fn py_market() -> Self {
        Self::Market
    }

    #[classattr]
    #[pyo3(name = "LIMIT")]
    fn py_limit() -> Self {
        Self::Limit
    }

    #[classattr]
    #[pyo3(name = "STOP")]
    fn py_stop() -> Self {
        Self::Stop
    }

    #[classattr]
    #[pyo3(name = "STOP_LIMIT")]
    fn py_stop_limit() -> Self {
        Self::StopLimit
    }

    #[classattr]
    #[pyo3(name = "TRAILING_STOP")]
    fn py_trailing_stop() -> Self {
        Self::TrailingStop
    }
}

#[pymethods]
impl AlpacaTimeInForce {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!(
            "<{}.{}: '{}'>",
            stringify!(AlpacaTimeInForce),
            self.name(),
            self.value(),
        )
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    #[getter]
    #[must_use]
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    #[getter]
    #[must_use]
    pub fn value(&self) -> String {
        self.to_string().to_lowercase()
    }

    #[staticmethod]
    #[must_use]
    fn variants() -> Vec<String> {
        Self::iter().map(|x| x.to_string()).collect()
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyAny>) -> PyResult<Self> {
        let data_str: String = data.str()?.extract()?;
        Self::from_str(&data_str).map_err(to_pyvalue_err)
    }

    #[classattr]
    #[pyo3(name = "DAY")]
    fn py_day() -> Self {
        Self::Day
    }

    #[classattr]
    #[pyo3(name = "GTC")]
    fn py_gtc() -> Self {
        Self::Gtc
    }

    #[classattr]
    #[pyo3(name = "OPG")]
    fn py_opg() -> Self {
        Self::Opg
    }

    #[classattr]
    #[pyo3(name = "CLS")]
    fn py_cls() -> Self {
        Self::Cls
    }

    #[classattr]
    #[pyo3(name = "IOC")]
    fn py_ioc() -> Self {
        Self::Ioc
    }

    #[classattr]
    #[pyo3(name = "FOK")]
    fn py_fok() -> Self {
        Self::Fok
    }
}

#[pymethods]
impl AlpacaOrderStatus {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!(
            "<{}.{}: '{}'>",
            stringify!(AlpacaOrderStatus),
            self.name(),
            self.value(),
        )
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    #[getter]
    #[must_use]
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    #[getter]
    #[must_use]
    pub fn value(&self) -> String {
        self.to_string()
    }

    #[staticmethod]
    #[must_use]
    fn variants() -> Vec<String> {
        Self::iter().map(|x| x.to_string()).collect()
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyAny>) -> PyResult<Self> {
        let data_str: String = data.str()?.extract()?;
        Self::from_str(&data_str).map_err(to_pyvalue_err)
    }

    #[classattr]
    #[pyo3(name = "NEW")]
    fn py_new_status() -> Self {
        Self::New
    }

    #[classattr]
    #[pyo3(name = "ACCEPTED")]
    fn py_accepted() -> Self {
        Self::Accepted
    }

    #[classattr]
    #[pyo3(name = "PARTIALLY_FILLED")]
    fn py_partially_filled() -> Self {
        Self::PartiallyFilled
    }

    #[classattr]
    #[pyo3(name = "FILLED")]
    fn py_filled() -> Self {
        Self::Filled
    }

    #[classattr]
    #[pyo3(name = "PENDING_CANCEL")]
    fn py_pending_cancel() -> Self {
        Self::PendingCancel
    }

    #[classattr]
    #[pyo3(name = "CANCELED")]
    fn py_canceled() -> Self {
        Self::Canceled
    }

    #[classattr]
    #[pyo3(name = "REJECTED")]
    fn py_rejected() -> Self {
        Self::Rejected
    }

    #[classattr]
    #[pyo3(name = "EXPIRED")]
    fn py_expired() -> Self {
        Self::Expired
    }

    #[classattr]
    #[pyo3(name = "PENDING_REPLACE")]
    fn py_pending_replace() -> Self {
        Self::PendingReplace
    }

    #[classattr]
    #[pyo3(name = "REPLACED")]
    fn py_replaced() -> Self {
        Self::Replaced
    }

    #[classattr]
    #[pyo3(name = "PENDING_NEW")]
    fn py_pending_new() -> Self {
        Self::PendingNew
    }

    #[classattr]
    #[pyo3(name = "STOPPED")]
    fn py_stopped() -> Self {
        Self::Stopped
    }

    #[classattr]
    #[pyo3(name = "SUSPENDED")]
    fn py_suspended() -> Self {
        Self::Suspended
    }

    #[classattr]
    #[pyo3(name = "CALCULATED")]
    fn py_calculated() -> Self {
        Self::Calculated
    }

    #[classattr]
    #[pyo3(name = "HELD")]
    fn py_held() -> Self {
        Self::Held
    }
}
