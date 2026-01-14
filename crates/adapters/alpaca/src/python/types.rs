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

//! Python bindings for Alpaca configuration and type structures.

use nautilus_model::identifiers::Venue;
use pyo3::prelude::*;

use crate::{
    common::enums::{AlpacaAssetClass, AlpacaDataFeed},
    config::{
        AlpacaDataClientConfig, AlpacaExecClientConfig, AlpacaInstrumentProviderConfig,
    },
};

#[pymethods]
impl AlpacaInstrumentProviderConfig {
    #[new]
    #[pyo3(signature = (load_all=false, asset_classes=None, option_underlying_symbols=None, log_warnings=true))]
    fn py_new(
        load_all: bool,
        asset_classes: Option<Vec<AlpacaAssetClass>>,
        option_underlying_symbols: Option<Vec<String>>,
        log_warnings: bool,
    ) -> Self {
        Self {
            load_all,
            asset_classes,
            option_underlying_symbols,
            log_warnings,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "AlpacaInstrumentProviderConfig(load_all={}, asset_classes={:?}, option_underlying_symbols={:?}, log_warnings={})",
            self.load_all,
            self.asset_classes,
            self.option_underlying_symbols,
            self.log_warnings
        )
    }

    #[getter]
    fn load_all(&self) -> bool {
        self.load_all
    }

    #[setter]
    fn set_load_all(&mut self, value: bool) {
        self.load_all = value;
    }

    #[getter]
    fn asset_classes(&self) -> Option<Vec<AlpacaAssetClass>> {
        self.asset_classes.clone()
    }

    #[setter]
    fn set_asset_classes(&mut self, value: Option<Vec<AlpacaAssetClass>>) {
        self.asset_classes = value;
    }

    #[getter]
    fn option_underlying_symbols(&self) -> Option<Vec<String>> {
        self.option_underlying_symbols.clone()
    }

    #[setter]
    fn set_option_underlying_symbols(&mut self, value: Option<Vec<String>>) {
        self.option_underlying_symbols = value;
    }

    #[getter]
    fn log_warnings(&self) -> bool {
        self.log_warnings
    }

    #[setter]
    fn set_log_warnings(&mut self, value: bool) {
        self.log_warnings = value;
    }
}

#[pymethods]
impl AlpacaDataClientConfig {
    #[new]
    #[pyo3(signature = (
        venue=None,
        api_key=None,
        api_secret=None,
        data_feed=None,
        paper_trading=true,
        base_url_http=None,
        base_url_ws=None,
        proxy_url=None,
        http_timeout_secs=None,
        update_instruments_interval_mins=None,
        instrument_provider=None
    ))]
    #[allow(clippy::too_many_arguments)]
    fn py_new(
        venue: Option<Venue>,
        api_key: Option<String>,
        api_secret: Option<String>,
        data_feed: Option<AlpacaDataFeed>,
        paper_trading: bool,
        base_url_http: Option<String>,
        base_url_ws: Option<String>,
        proxy_url: Option<String>,
        http_timeout_secs: Option<u64>,
        update_instruments_interval_mins: Option<u64>,
        instrument_provider: Option<AlpacaInstrumentProviderConfig>,
    ) -> Self {
        Self {
            venue: venue.unwrap_or_else(|| Venue::from("ALPACA")),
            api_key,
            api_secret,
            data_feed: data_feed.unwrap_or(AlpacaDataFeed::Iex),
            paper_trading,
            base_url_http,
            base_url_ws,
            proxy_url,
            http_timeout_secs: http_timeout_secs.or(Some(30)),
            update_instruments_interval_mins: update_instruments_interval_mins.or(Some(60)),
            instrument_provider,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "AlpacaDataClientConfig(venue={}, data_feed={:?}, paper_trading={})",
            self.venue, self.data_feed, self.paper_trading
        )
    }

    #[getter]
    fn venue(&self) -> Venue {
        self.venue
    }

    #[setter]
    fn set_venue(&mut self, value: Venue) {
        self.venue = value;
    }

    #[getter]
    fn api_key(&self) -> Option<String> {
        self.api_key.clone()
    }

    #[setter]
    fn set_api_key(&mut self, value: Option<String>) {
        self.api_key = value;
    }

    #[getter]
    fn api_secret(&self) -> Option<String> {
        self.api_secret.clone()
    }

    #[setter]
    fn set_api_secret(&mut self, value: Option<String>) {
        self.api_secret = value;
    }

    #[getter]
    fn data_feed(&self) -> AlpacaDataFeed {
        self.data_feed
    }

    #[setter]
    fn set_data_feed(&mut self, value: AlpacaDataFeed) {
        self.data_feed = value;
    }

    #[getter]
    fn paper_trading(&self) -> bool {
        self.paper_trading
    }

    #[setter]
    fn set_paper_trading(&mut self, value: bool) {
        self.paper_trading = value;
    }

    #[getter]
    fn base_url_http(&self) -> Option<String> {
        self.base_url_http.clone()
    }

    #[setter]
    fn set_base_url_http(&mut self, value: Option<String>) {
        self.base_url_http = value;
    }

    #[getter]
    fn base_url_ws(&self) -> Option<String> {
        self.base_url_ws.clone()
    }

    #[setter]
    fn set_base_url_ws(&mut self, value: Option<String>) {
        self.base_url_ws = value;
    }

    #[getter]
    fn proxy_url(&self) -> Option<String> {
        self.proxy_url.clone()
    }

    #[setter]
    fn set_proxy_url(&mut self, value: Option<String>) {
        self.proxy_url = value;
    }

    #[getter]
    fn http_timeout_secs(&self) -> Option<u64> {
        self.http_timeout_secs
    }

    #[setter]
    fn set_http_timeout_secs(&mut self, value: Option<u64>) {
        self.http_timeout_secs = value;
    }

    #[getter]
    fn update_instruments_interval_mins(&self) -> Option<u64> {
        self.update_instruments_interval_mins
    }

    #[setter]
    fn set_update_instruments_interval_mins(&mut self, value: Option<u64>) {
        self.update_instruments_interval_mins = value;
    }

    #[getter]
    fn instrument_provider(&self) -> Option<AlpacaInstrumentProviderConfig> {
        self.instrument_provider.clone()
    }

    #[setter]
    fn set_instrument_provider(&mut self, value: Option<AlpacaInstrumentProviderConfig>) {
        self.instrument_provider = value;
    }

    #[pyo3(name = "has_credentials")]
    fn py_has_credentials(&self) -> bool {
        self.has_credentials()
    }

    #[pyo3(name = "http_base_url")]
    fn py_http_base_url(&self) -> String {
        self.http_base_url()
    }

    #[pyo3(name = "ws_base_url")]
    fn py_ws_base_url(&self, asset_class: AlpacaAssetClass) -> String {
        self.ws_base_url(asset_class)
    }
}

#[pymethods]
impl AlpacaExecClientConfig {
    #[new]
    #[pyo3(signature = (
        venue=None,
        api_key=None,
        api_secret=None,
        paper_trading=true,
        base_url_http=None,
        base_url_ws=None,
        proxy_url=None,
        http_timeout_secs=None,
        max_retries=None,
        retry_delay_secs=None,
        instrument_provider=None
    ))]
    #[allow(clippy::too_many_arguments)]
    fn py_new(
        venue: Option<Venue>,
        api_key: Option<String>,
        api_secret: Option<String>,
        paper_trading: bool,
        base_url_http: Option<String>,
        base_url_ws: Option<String>,
        proxy_url: Option<String>,
        http_timeout_secs: Option<u64>,
        max_retries: Option<u32>,
        retry_delay_secs: Option<f64>,
        instrument_provider: Option<AlpacaInstrumentProviderConfig>,
    ) -> Self {
        Self {
            venue: venue.unwrap_or_else(|| Venue::from("ALPACA")),
            api_key,
            api_secret,
            paper_trading,
            base_url_http,
            base_url_ws,
            proxy_url,
            http_timeout_secs: http_timeout_secs.or(Some(30)),
            max_retries: max_retries.or(Some(3)),
            retry_delay_secs: retry_delay_secs.or(Some(1.0)),
            instrument_provider,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "AlpacaExecClientConfig(venue={}, paper_trading={}, max_retries={:?})",
            self.venue, self.paper_trading, self.max_retries
        )
    }

    #[getter]
    fn venue(&self) -> Venue {
        self.venue
    }

    #[setter]
    fn set_venue(&mut self, value: Venue) {
        self.venue = value;
    }

    #[getter]
    fn api_key(&self) -> Option<String> {
        self.api_key.clone()
    }

    #[setter]
    fn set_api_key(&mut self, value: Option<String>) {
        self.api_key = value;
    }

    #[getter]
    fn api_secret(&self) -> Option<String> {
        self.api_secret.clone()
    }

    #[setter]
    fn set_api_secret(&mut self, value: Option<String>) {
        self.api_secret = value;
    }

    #[getter]
    fn paper_trading(&self) -> bool {
        self.paper_trading
    }

    #[setter]
    fn set_paper_trading(&mut self, value: bool) {
        self.paper_trading = value;
    }

    #[getter]
    fn base_url_http(&self) -> Option<String> {
        self.base_url_http.clone()
    }

    #[setter]
    fn set_base_url_http(&mut self, value: Option<String>) {
        self.base_url_http = value;
    }

    #[getter]
    fn base_url_ws(&self) -> Option<String> {
        self.base_url_ws.clone()
    }

    #[setter]
    fn set_base_url_ws(&mut self, value: Option<String>) {
        self.base_url_ws = value;
    }

    #[getter]
    fn proxy_url(&self) -> Option<String> {
        self.proxy_url.clone()
    }

    #[setter]
    fn set_proxy_url(&mut self, value: Option<String>) {
        self.proxy_url = value;
    }

    #[getter]
    fn http_timeout_secs(&self) -> Option<u64> {
        self.http_timeout_secs
    }

    #[setter]
    fn set_http_timeout_secs(&mut self, value: Option<u64>) {
        self.http_timeout_secs = value;
    }

    #[getter]
    fn max_retries(&self) -> Option<u32> {
        self.max_retries
    }

    #[setter]
    fn set_max_retries(&mut self, value: Option<u32>) {
        self.max_retries = value;
    }

    #[getter]
    fn retry_delay_secs(&self) -> Option<f64> {
        self.retry_delay_secs
    }

    #[setter]
    fn set_retry_delay_secs(&mut self, value: Option<f64>) {
        self.retry_delay_secs = value;
    }

    #[getter]
    fn instrument_provider(&self) -> Option<AlpacaInstrumentProviderConfig> {
        self.instrument_provider.clone()
    }

    #[setter]
    fn set_instrument_provider(&mut self, value: Option<AlpacaInstrumentProviderConfig>) {
        self.instrument_provider = value;
    }

    #[pyo3(name = "has_credentials")]
    fn py_has_credentials(&self) -> bool {
        self.has_credentials()
    }

    #[pyo3(name = "http_base_url")]
    fn py_http_base_url(&self) -> String {
        self.http_base_url()
    }

    #[pyo3(name = "ws_base_url")]
    fn py_ws_base_url(&self) -> String {
        self.ws_base_url()
    }
}
