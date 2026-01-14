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

//! Instrument providers for loading Alpaca assets and option contracts.

pub mod parsers;

use std::collections::HashMap;

use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Venue},
    instruments::{Instrument, InstrumentAny},
};

use crate::{
    common::enums::AlpacaAssetClass,
    error::{AlpacaError, Result},
    http::{client::AlpacaHttpClient, models::AlpacaOptionContractsResponse},
};

use self::parsers::{parse_crypto_instrument, parse_equity_instrument, parse_option_instrument};

/// Configuration for the Alpaca instrument provider.
#[derive(Clone, Debug)]
pub struct AlpacaInstrumentProviderConfig {
    /// Asset classes to load instruments for.
    pub asset_classes: Vec<AlpacaAssetClass>,
    /// Underlying symbols to load option contracts for (required for options).
    pub option_underlying_symbols: Option<Vec<String>>,
    /// Venue to assign to instruments.
    pub venue: Venue,
    /// Whether to log warnings for parsing errors.
    pub log_warnings: bool,
}

impl Default for AlpacaInstrumentProviderConfig {
    fn default() -> Self {
        Self {
            asset_classes: vec![AlpacaAssetClass::UsEquity],
            option_underlying_symbols: None,
            venue: Venue::from("ALPACA"),
            log_warnings: true,
        }
    }
}

impl AlpacaInstrumentProviderConfig {
    /// Creates a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the asset classes to load.
    #[must_use]
    pub fn with_asset_classes(mut self, asset_classes: Vec<AlpacaAssetClass>) -> Self {
        self.asset_classes = asset_classes;
        self
    }

    /// Sets the option underlying symbols.
    #[must_use]
    pub fn with_option_underlying_symbols(mut self, symbols: Vec<String>) -> Self {
        self.option_underlying_symbols = Some(symbols);
        self
    }

    /// Sets the venue.
    #[must_use]
    pub fn with_venue(mut self, venue: Venue) -> Self {
        self.venue = venue;
        self
    }

    /// Sets whether to log warnings.
    #[must_use]
    pub fn with_log_warnings(mut self, log_warnings: bool) -> Self {
        self.log_warnings = log_warnings;
        self
    }
}

/// Provides instruments from Alpaca Markets.
///
/// Supports:
/// - US equities (stocks and ETFs)
/// - Cryptocurrencies (BTC/USD, ETH/USD, etc.)
/// - Options (equity options)
#[derive(Debug)]
pub struct AlpacaInstrumentProvider {
    client: AlpacaHttpClient,
    config: AlpacaInstrumentProviderConfig,
    instruments: HashMap<InstrumentId, InstrumentAny>,
}

impl AlpacaInstrumentProvider {
    /// Creates a new [`AlpacaInstrumentProvider`].
    #[must_use]
    pub fn new(client: AlpacaHttpClient, config: AlpacaInstrumentProviderConfig) -> Self {
        Self {
            client,
            config,
            instruments: HashMap::new(),
        }
    }

    /// Returns the current timestamp in nanoseconds.
    fn timestamp_ns(&self) -> UnixNanos {
        UnixNanos::from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        )
    }

    /// Loads all instruments for the configured asset classes.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if instrument parsing fails.
    pub async fn load_all(&mut self) -> Result<Vec<InstrumentAny>> {
        let mut all_instruments = Vec::new();

        // Clone asset_classes to avoid borrow checker issues
        let asset_classes = self.config.asset_classes.clone();

        for asset_class in &asset_classes {
            match self.load_asset_class(*asset_class).await {
                Ok(instruments) => all_instruments.extend(instruments),
                Err(e) => {
                    if self.config.log_warnings {
                        eprintln!("Warning: Failed to load {}: {}", asset_class, e);
                    }
                }
            }
        }

        Ok(all_instruments)
    }

    /// Loads instruments for a specific asset class.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if instrument parsing fails.
    pub async fn load_asset_class(
        &mut self,
        asset_class: AlpacaAssetClass,
    ) -> Result<Vec<InstrumentAny>> {
        match asset_class {
            AlpacaAssetClass::UsEquity => self.load_equities().await,
            AlpacaAssetClass::Crypto => self.load_crypto().await,
            AlpacaAssetClass::Option => self.load_options().await,
        }
    }

    /// Loads US equity instruments.
    async fn load_equities(&mut self) -> Result<Vec<InstrumentAny>> {
        let params = [("status", "active"), ("asset_class", "us_equity")];

        let assets: Vec<crate::http::models::AlpacaAsset> = self.client.get("/v2/assets", Some(&params)).await?;

        let ts_init = self.timestamp_ns();
        let mut instruments = Vec::new();

        for asset in assets {
            if !asset.tradable {
                continue;
            }

            match parse_equity_instrument(&asset, &self.config.venue, ts_init) {
                Ok(instrument) => {
                    let instrument_id = instrument.id();
                    instruments.push(instrument.clone());
                    self.instruments.insert(instrument_id, instrument);
                }
                Err(e) => {
                    if self.config.log_warnings {
                        eprintln!("Warning: Failed to parse equity {}: {}", asset.symbol, e);
                    }
                }
            }
        }

        Ok(instruments)
    }

    /// Loads cryptocurrency instruments.
    async fn load_crypto(&mut self) -> Result<Vec<InstrumentAny>> {
        let params = [("status", "active"), ("asset_class", "crypto")];

        let assets: Vec<crate::http::models::AlpacaAsset> = self.client.get("/v2/assets", Some(&params)).await?;

        let ts_init = self.timestamp_ns();
        let mut instruments = Vec::new();

        for asset in assets {
            if !asset.tradable {
                continue;
            }

            match parse_crypto_instrument(&asset, &self.config.venue, ts_init) {
                Ok(instrument) => {
                    let instrument_id = instrument.id();
                    instruments.push(instrument.clone());
                    self.instruments.insert(instrument_id, instrument);
                }
                Err(e) => {
                    if self.config.log_warnings {
                        eprintln!("Warning: Failed to parse crypto {}: {}", asset.symbol, e);
                    }
                }
            }
        }

        Ok(instruments)
    }

    /// Loads option contract instruments.
    async fn load_options(&mut self) -> Result<Vec<InstrumentAny>> {
        let underlying_symbols = self
            .config
            .option_underlying_symbols
            .as_ref()
            .ok_or_else(|| {
                AlpacaError::InvalidRequest(
                    "option_underlying_symbols is required for loading options".to_string(),
                )
            })?
            .clone();

        let mut all_instruments = Vec::new();

        for underlying in &underlying_symbols {
            match self.load_options_for_underlying(underlying).await {
                Ok(instruments) => all_instruments.extend(instruments),
                Err(e) => {
                    if self.config.log_warnings {
                        eprintln!("Warning: Failed to load options for {}: {}", underlying, e);
                    }
                }
            }
        }

        Ok(all_instruments)
    }

    /// Loads option contracts for a specific underlying symbol.
    async fn load_options_for_underlying(&mut self, underlying: &str) -> Result<Vec<InstrumentAny>> {
        let mut all_instruments = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut params = vec![
                ("underlying_symbols", underlying.to_string()),
                ("status", "active".to_string()),
            ];

            if let Some(token) = page_token {
                params.push(("page_token", token));
            }

            let data: AlpacaOptionContractsResponse = self.client.get("/v2/options/contracts", Some(&params)).await?;

            let ts_init = self.timestamp_ns();

            for contract in data.option_contracts {
                if !contract.tradable {
                    continue;
                }

                match parse_option_instrument(&contract, &self.config.venue, ts_init) {
                    Ok(instrument) => {
                        let instrument_id = instrument.id();
                        all_instruments.push(instrument.clone());
                        self.instruments.insert(instrument_id, instrument);
                    }
                    Err(e) => {
                        if self.config.log_warnings {
                            eprintln!(
                                "Warning: Failed to parse option {}: {}",
                                contract.symbol, e
                            );
                        }
                    }
                }
            }

            // Check for more pages
            if let Some(next_token) = data.next_page_token {
                page_token = Some(next_token);
            } else {
                break;
            }
        }

        Ok(all_instruments)
    }

    /// Loads a single instrument by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the instrument is not found or cannot be loaded.
    pub async fn load_instrument(&mut self, instrument_id: &InstrumentId) -> Result<InstrumentAny> {
        // Check if already loaded
        if let Some(instrument) = self.instruments.get(instrument_id) {
            return Ok(instrument.clone());
        }

        // Load from API
        let symbol = instrument_id.symbol.as_str();
        let asset: crate::http::models::AlpacaAsset = self.client.get::<HashMap<String, String>, _>(&format!("/v2/assets/{}", symbol), None).await?;

        let ts_init = self.timestamp_ns();

        let instrument = match asset.class.as_str() {
            "us_equity" => parse_equity_instrument(&asset, &self.config.venue, ts_init)?,
            "crypto" => parse_crypto_instrument(&asset, &self.config.venue, ts_init)?,
            _ => {
                return Err(AlpacaError::ParseError(format!(
                    "Unsupported asset class: {}",
                    asset.class
                )))
            }
        };

        let instrument_id = instrument.id();
        let instrument_clone = instrument.clone();
        self.instruments.insert(instrument_id, instrument);
        Ok(instrument_clone)
    }

    /// Returns all loaded instruments.
    #[must_use]
    pub fn instruments(&self) -> &HashMap<InstrumentId, InstrumentAny> {
        &self.instruments
    }

    /// Returns a specific loaded instrument by ID.
    #[must_use]
    pub fn get_instrument(&self, instrument_id: &InstrumentId) -> Option<&InstrumentAny> {
        self.instruments.get(instrument_id)
    }

    /// Returns the number of loaded instruments.
    #[must_use]
    pub fn count(&self) -> usize {
        self.instruments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    fn test_config_default() {
        let config = AlpacaInstrumentProviderConfig::default();
        assert_eq!(config.asset_classes.len(), 1);
        assert!(matches!(
            config.asset_classes[0],
            AlpacaAssetClass::UsEquity
        ));
        assert!(config.log_warnings);
    }

    #[rstest]
    fn test_config_builder() {
        let config = AlpacaInstrumentProviderConfig::new()
            .with_asset_classes(vec![AlpacaAssetClass::Crypto])
            .with_option_underlying_symbols(vec!["AAPL".to_string()])
            .with_log_warnings(false);

        assert_eq!(config.asset_classes.len(), 1);
        assert!(matches!(config.asset_classes[0], AlpacaAssetClass::Crypto));
        assert!(config.option_underlying_symbols.is_some());
        assert!(!config.log_warnings);
    }
}
