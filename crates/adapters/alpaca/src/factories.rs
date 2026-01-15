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

//! Factory functions for creating Alpaca clients and providers.

use std::{env, sync::Arc};

use crate::{
    common::enums::AlpacaEnvironment,
    config::{AlpacaDataClientConfig, AlpacaExecClientConfig, AlpacaInstrumentProviderConfig},
    error::{AlpacaError, Result},
    http::client::AlpacaHttpClient,
    providers::AlpacaInstrumentProvider,
};

/// Factory for creating Alpaca HTTP clients with optional credential loading from environment.
#[derive(Debug)]
pub struct AlpacaHttpClientFactory;

impl AlpacaHttpClientFactory {
    /// Creates a new HTTP client with the given configuration.
    ///
    /// If API credentials are not provided in the config, attempts to read them from
    /// environment variables `ALPACA_API_KEY` and `ALPACA_API_SECRET`.
    ///
    /// # Errors
    ///
    /// Returns an error if credentials are not provided and not found in environment.
    pub fn create(config: &AlpacaDataClientConfig) -> Result<AlpacaHttpClient> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| env::var("ALPACA_API_KEY").ok())
            .ok_or_else(|| {
                AlpacaError::InvalidRequest(
                    "API key not provided and ALPACA_API_KEY environment variable not set"
                        .to_string(),
                )
            })?;

        let api_secret = config
            .api_secret
            .clone()
            .or_else(|| env::var("ALPACA_API_SECRET").ok())
            .ok_or_else(|| {
                AlpacaError::InvalidRequest(
                    "API secret not provided and ALPACA_API_SECRET environment variable not set"
                        .to_string(),
                )
            })?;

        let environment = if config.paper_trading {
            AlpacaEnvironment::Paper
        } else {
            AlpacaEnvironment::Live
        };

        AlpacaHttpClient::new(
            environment,
            api_key,
            api_secret,
            config.http_timeout_secs,
            config.proxy_url.clone(),
        )
        .map_err(|e| AlpacaError::InvalidRequest(format!("Failed to create HTTP client: {:?}", e)))
    }

    /// Creates a new HTTP client for execution with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if credentials are not provided and not found in environment.
    pub fn create_for_exec(config: &AlpacaExecClientConfig) -> Result<AlpacaHttpClient> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| env::var("ALPACA_API_KEY").ok())
            .ok_or_else(|| {
                AlpacaError::InvalidRequest(
                    "API key not provided and ALPACA_API_KEY environment variable not set"
                        .to_string(),
                )
            })?;

        let api_secret = config
            .api_secret
            .clone()
            .or_else(|| env::var("ALPACA_API_SECRET").ok())
            .ok_or_else(|| {
                AlpacaError::InvalidRequest(
                    "API secret not provided and ALPACA_API_SECRET environment variable not set"
                        .to_string(),
                )
            })?;

        let environment = if config.paper_trading {
            AlpacaEnvironment::Paper
        } else {
            AlpacaEnvironment::Live
        };

        AlpacaHttpClient::new(
            environment,
            api_key,
            api_secret,
            config.http_timeout_secs,
            config.proxy_url.clone(),
        )
        .map_err(|e| AlpacaError::InvalidRequest(format!("Failed to create HTTP client: {:?}", e)))
    }
}

/// Factory for creating Alpaca instrument providers.
#[derive(Debug)]
pub struct AlpacaInstrumentProviderFactory;

impl AlpacaInstrumentProviderFactory {
    /// Creates a new instrument provider with the given HTTP client and configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn create(
        client: AlpacaHttpClient,
        config: Option<AlpacaInstrumentProviderConfig>,
    ) -> Result<AlpacaInstrumentProvider> {
        let provider_config = if let Some(cfg) = config {
            crate::providers::AlpacaInstrumentProviderConfig {
                asset_classes: cfg.asset_classes.unwrap_or_default(),
                option_underlying_symbols: cfg.option_underlying_symbols,
                venue: nautilus_model::identifiers::Venue::from("ALPACA"),
                log_warnings: cfg.log_warnings,
            }
        } else {
            crate::providers::AlpacaInstrumentProviderConfig::default()
        };

        Ok(AlpacaInstrumentProvider::new(client, provider_config))
    }
}

/// Cached HTTP client instance for reuse across data and execution clients.
static mut CACHED_HTTP_CLIENT: Option<Arc<AlpacaHttpClient>> = None;
static mut CACHED_CLIENT_CONFIG_HASH: u64 = 0;

/// Returns a cached HTTP client, creating one if it doesn't exist or if the config has changed.
///
/// This function uses a simple caching mechanism to avoid creating multiple HTTP clients
/// with the same configuration. Note: This is not thread-safe and should only be used
/// in single-threaded contexts or with external synchronization.
///
/// # Errors
///
/// Returns an error if the client cannot be created.
pub fn get_cached_http_client(config: &AlpacaDataClientConfig) -> Result<Arc<AlpacaHttpClient>> {
    unsafe {
        // Simple hash of config for cache invalidation
        let config_hash = compute_config_hash(config);

        let cached_hash = std::ptr::read(std::ptr::addr_of!(CACHED_CLIENT_CONFIG_HASH));
        let client_ptr = std::ptr::addr_of!(CACHED_HTTP_CLIENT);

        if cached_hash != config_hash || std::ptr::read(client_ptr).is_none() {
            let client = AlpacaHttpClientFactory::create(config)?;
            std::ptr::write(std::ptr::addr_of_mut!(CACHED_HTTP_CLIENT), Some(Arc::new(client)));
            std::ptr::write(std::ptr::addr_of_mut!(CACHED_CLIENT_CONFIG_HASH), config_hash);
        }

        Ok(std::ptr::read(client_ptr)
            .expect("Client should be initialized"))
    }
}

/// Returns a cached HTTP client for execution, creating one if needed.
///
/// # Errors
///
/// Returns an error if the client cannot be created.
pub fn get_cached_http_client_for_exec(
    config: &AlpacaExecClientConfig,
) -> Result<Arc<AlpacaHttpClient>> {
    unsafe {
        // For execution clients, we use the same cache mechanism
        // Convert exec config to data config for hashing
        let data_config = AlpacaDataClientConfig {
            api_key: config.api_key.clone(),
            api_secret: config.api_secret.clone(),
            paper_trading: config.paper_trading,
            base_url_http: config.base_url_http.clone(),
            http_timeout_secs: config.http_timeout_secs,
            ..Default::default()
        };

        let config_hash = compute_config_hash(&data_config);

        let cached_hash = std::ptr::read(std::ptr::addr_of!(CACHED_CLIENT_CONFIG_HASH));
        let client_ptr = std::ptr::addr_of!(CACHED_HTTP_CLIENT);

        if cached_hash != config_hash || std::ptr::read(client_ptr).is_none() {
            let client = AlpacaHttpClientFactory::create_for_exec(config)?;
            std::ptr::write(std::ptr::addr_of_mut!(CACHED_HTTP_CLIENT), Some(Arc::new(client)));
            std::ptr::write(std::ptr::addr_of_mut!(CACHED_CLIENT_CONFIG_HASH), config_hash);
        }

        Ok(std::ptr::read(client_ptr)
            .expect("Client should be initialized"))
    }
}

/// Computes a simple hash of the config for cache invalidation.
fn compute_config_hash(config: &AlpacaDataClientConfig) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    config.api_key.hash(&mut hasher);
    config.api_secret.hash(&mut hasher);
    config.paper_trading.hash(&mut hasher);
    config.base_url_http.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    fn test_config_hash_same_for_identical_configs() {
        let config1 = AlpacaDataClientConfig::new()
            .with_credentials("key".to_string(), "secret".to_string())
            .with_paper_trading(true);

        let config2 = AlpacaDataClientConfig::new()
            .with_credentials("key".to_string(), "secret".to_string())
            .with_paper_trading(true);

        assert_eq!(compute_config_hash(&config1), compute_config_hash(&config2));
    }

    #[rstest]
    fn test_config_hash_different_for_different_configs() {
        let config1 = AlpacaDataClientConfig::new()
            .with_credentials("key1".to_string(), "secret1".to_string())
            .with_paper_trading(true);

        let config2 = AlpacaDataClientConfig::new()
            .with_credentials("key2".to_string(), "secret2".to_string())
            .with_paper_trading(true);

        assert_ne!(compute_config_hash(&config1), compute_config_hash(&config2));
    }

    #[rstest]
    fn test_instrument_provider_factory() {
        // Note: This test would require a valid HTTP client
        // In a real scenario, you'd use a mock client or integration test
        let client = AlpacaHttpClient::new(
            AlpacaEnvironment::Paper,
            "test_key".to_string(),
            "test_secret".to_string(),
            Some(30),
            None,
        )
        .expect("Failed to create client");

        let provider_config = AlpacaInstrumentProviderConfig::default();
        let result = AlpacaInstrumentProviderFactory::create(client, Some(provider_config));

        assert!(result.is_ok());
    }
}
