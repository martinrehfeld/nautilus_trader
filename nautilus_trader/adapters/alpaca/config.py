# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2026 Andrew Crum. All rights reserved.
#  https://github.com/agcrum
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------
"""
Configuration classes for the Alpaca adapter.

These Python config classes wrap the Rust config types and add NautilusTrader-specific
fields like routing configuration.
"""

from nautilus_trader.config import InstrumentProviderConfig
from nautilus_trader.config import LiveDataClientConfig
from nautilus_trader.config import LiveExecClientConfig
from nautilus_trader.core.nautilus_pyo3 import AlpacaAssetClass
from nautilus_trader.core.nautilus_pyo3 import AlpacaDataFeed


class AlpacaInstrumentProviderConfig(InstrumentProviderConfig, frozen=True):
    """
    Configuration for ``AlpacaInstrumentProvider`` instances.

    Parameters
    ----------
    asset_classes : tuple[AlpacaAssetClass, ...], optional
        The asset classes to load instruments for.
        If None, defaults to US Equities.
    option_underlying_symbols : tuple[str, ...], optional
        Underlying symbols to load option contracts for (required for options).

    """

    asset_classes: tuple[AlpacaAssetClass, ...] | None = None
    option_underlying_symbols: tuple[str, ...] | None = None


class AlpacaDataClientConfig(LiveDataClientConfig, frozen=True):
    """
    Configuration for ``AlpacaDataClient`` instances.

    Parameters
    ----------
    api_key : str, optional
        The Alpaca API key.
        If ``None`` then will source the `ALPACA_API_KEY` environment variable.
    api_secret : str, optional
        The Alpaca API secret.
        If ``None`` then will source the `ALPACA_API_SECRET` environment variable.
    paper_trading : bool, default True
        If the client should connect to paper trading endpoints.
    data_feed : AlpacaDataFeed, default AlpacaDataFeed.IEX
        The market data feed subscription level (IEX or SIP).
    base_url_http : str, optional
        Optional HTTP client custom endpoint override.
    base_url_ws : str, optional
        Optional WebSocket client custom endpoint override.
    proxy_url : str, optional
        Optional proxy URL for HTTP requests.
    http_timeout_secs : int, optional
        The timeout (seconds) for HTTP requests.
    update_instruments_interval_mins : int, optional
        The interval (minutes) between instrument updates.
    instrument_provider : AlpacaInstrumentProviderConfig, optional
        Configuration for the instrument provider.

    """

    api_key: str | None = None
    api_secret: str | None = None
    paper_trading: bool = True
    data_feed: AlpacaDataFeed = AlpacaDataFeed.Iex
    base_url_http: str | None = None
    base_url_ws: str | None = None
    proxy_url: str | None = None
    http_timeout_secs: int | None = None
    update_instruments_interval_mins: int | None = None
    instrument_provider: AlpacaInstrumentProviderConfig | None = None


class AlpacaExecClientConfig(LiveExecClientConfig, frozen=True):
    """
    Configuration for ``AlpacaExecutionClient`` instances.

    Parameters
    ----------
    api_key : str, optional
        The Alpaca API key.
        If ``None`` then will source the `ALPACA_API_KEY` environment variable.
    api_secret : str, optional
        The Alpaca API secret.
        If ``None`` then will source the `ALPACA_API_SECRET` environment variable.
    paper_trading : bool, default True
        If the client should connect to paper trading endpoints.
    base_url_http : str, optional
        Optional HTTP client custom endpoint override.
    proxy_url : str, optional
        Optional proxy URL for HTTP requests.
    http_timeout_secs : int, optional
        The timeout (seconds) for HTTP requests.
    max_retries : int, optional
        Maximum number of retries for failed requests.

    """

    api_key: str | None = None
    api_secret: str | None = None
    paper_trading: bool = True
    base_url_http: str | None = None
    proxy_url: str | None = None
    http_timeout_secs: int | None = None
    max_retries: int | None = None
