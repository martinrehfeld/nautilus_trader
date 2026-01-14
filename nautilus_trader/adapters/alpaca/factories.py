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
Alpaca adapter factory functions.

Provides factories for creating Alpaca data and execution clients that integrate
with NautilusTrader's event system.
"""

from __future__ import annotations

import asyncio
from functools import lru_cache

from nautilus_trader.adapters.alpaca.data import AlpacaDataClient
from nautilus_trader.adapters.alpaca.execution import AlpacaExecutionClient
from nautilus_trader.adapters.alpaca.providers import AlpacaInstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.core import nautilus_pyo3
from nautilus_trader.live.factories import LiveDataClientFactory
from nautilus_trader.live.factories import LiveExecClientFactory


@lru_cache(1)
def get_cached_alpaca_http_client(
    api_key: str | None = None,
    api_secret: str | None = None,
    paper_trading: bool = True,
    base_url: str | None = None,
    timeout_secs: int | None = None,
    proxy_url: str | None = None,
) -> nautilus_pyo3.AlpacaHttpClient:
    """
    Cache and return an Alpaca HTTP client with the given credentials.

    If a cached client with matching parameters already exists, the cached client will be returned.

    Parameters
    ----------
    api_key : str, optional
        The API key for the client.
    api_secret : str, optional
        The API secret for the client.
    paper_trading : bool, default True
        If the client should use paper trading endpoints.
    base_url : str, optional
        The base URL for the API endpoints.
    timeout_secs : int, optional
        The timeout (seconds) for HTTP requests.
    proxy_url : str, optional
        The proxy URL for HTTP requests.

    Returns
    -------
    AlpacaHttpClient

    """
    # Determine environment
    environment = (
        nautilus_pyo3.AlpacaEnvironment.Paper
        if paper_trading
        else nautilus_pyo3.AlpacaEnvironment.Live
    )

    return nautilus_pyo3.AlpacaHttpClient(
        environment=environment,
        api_key=api_key or "",
        api_secret=api_secret or "",
        timeout_secs=timeout_secs,
        proxy_url=proxy_url,
    )


@lru_cache(1)
def get_cached_alpaca_instrument_provider(
    client: nautilus_pyo3.AlpacaHttpClient,
    config=None,  # InstrumentProviderConfig
) -> AlpacaInstrumentProvider:
    """
    Cache and return an Alpaca instrument provider.

    If a cached provider already exists, then that provider will be returned.

    Parameters
    ----------
    client : AlpacaHttpClient
        The Alpaca HTTP client.
    config : InstrumentProviderConfig, optional
        The instrument provider configuration, by default None.

    Returns
    -------
    AlpacaInstrumentProvider

    """
    return AlpacaInstrumentProvider(
        client=client,
        config=config,
    )


class AlpacaLiveDataClientFactory(LiveDataClientFactory):
    """
    Provides an Alpaca live data client factory.
    """

    @staticmethod
    def create(  # type: ignore
        loop: asyncio.AbstractEventLoop,
        name: str,
        config,  # AlpacaDataClientConfig
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> AlpacaDataClient:
        """
        Create a new Alpaca data client.

        Parameters
        ----------
        loop : asyncio.AbstractEventLoop
            The event loop for the client.
        name : str
            The custom client ID.
        config : AlpacaDataClientConfig
            The client configuration.
        msgbus : MessageBus
            The message bus for the client.
        cache : Cache
            The cache for the client.
        clock : LiveClock
            The clock for the client.

        Returns
        -------
        AlpacaDataClient

        """
        client: nautilus_pyo3.AlpacaHttpClient = get_cached_alpaca_http_client(
            api_key=config.api_key,
            api_secret=config.api_secret,
            paper_trading=config.paper_trading,
            base_url=config.base_url_http,
            timeout_secs=config.http_timeout_secs,
            proxy_url=config.proxy_url,
        )

        provider = get_cached_alpaca_instrument_provider(
            client=client,
            config=config.instrument_provider,
        )

        return AlpacaDataClient(
            loop=loop,
            client=client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=provider,
            config=config,
            name=name,
        )


class AlpacaLiveExecClientFactory(LiveExecClientFactory):
    """
    Provides an Alpaca live execution client factory.
    """

    @staticmethod
    def create(  # type: ignore
        loop: asyncio.AbstractEventLoop,
        name: str,
        config,  # AlpacaExecClientConfig
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> AlpacaExecutionClient:
        """
        Create a new Alpaca execution client.

        Parameters
        ----------
        loop : asyncio.AbstractEventLoop
            The event loop for the client.
        name : str
            The custom client ID.
        config : AlpacaExecClientConfig
            The client configuration.
        msgbus : MessageBus
            The message bus for the client.
        cache : Cache
            The cache for the client.
        clock : LiveClock
            The clock for the client.

        Returns
        -------
        AlpacaExecutionClient

        """
        client: nautilus_pyo3.AlpacaHttpClient = get_cached_alpaca_http_client(
            api_key=config.api_key,
            api_secret=config.api_secret,
            paper_trading=config.paper_trading,
            base_url=config.base_url_http,
            timeout_secs=config.http_timeout_secs,
            proxy_url=config.proxy_url,
        )

        provider = get_cached_alpaca_instrument_provider(
            client=client,
            config=None,
        )

        return AlpacaExecutionClient(
            loop=loop,
            client=client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=provider,
            config=config,
            name=name,
        )
