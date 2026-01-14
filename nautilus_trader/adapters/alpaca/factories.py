# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
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

Thin Python wrappers over Rust factory implementations for backward
compatibility with NautilusTrader's factory pattern.
"""

from __future__ import annotations

import asyncio

try:
    # Import Rust implementations
    from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaDataClient
    from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaDataClientConfig
    from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaExecClientConfig
    from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaExecutionClient
    from nautilus_trader.core.nautilus_pyo3.alpaca import create_alpaca_data_client
    from nautilus_trader.core.nautilus_pyo3.alpaca import create_alpaca_exec_client
except ImportError:
    # Fallback to alternative import path
    try:
        from nautilus_pyo3.alpaca import AlpacaDataClient
        from nautilus_pyo3.alpaca import AlpacaDataClientConfig
        from nautilus_pyo3.alpaca import AlpacaExecClientConfig
        from nautilus_pyo3.alpaca import AlpacaExecutionClient
        from nautilus_pyo3.alpaca import create_alpaca_data_client
        from nautilus_pyo3.alpaca import create_alpaca_exec_client
    except ImportError as e:
        raise ImportError(
            "Alpaca adapter Rust module not found. "
            "Please ensure the Rust extension is built with: uv build"
        ) from e

from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.live.factories import LiveDataClientFactory
from nautilus_trader.live.factories import LiveExecClientFactory


class AlpacaLiveDataClientFactory(LiveDataClientFactory):
    """
    Provides an Alpaca live data client factory.

    This is a thin wrapper over the Rust implementation that maintains
    compatibility with NautilusTrader's factory pattern used in examples.
    """

    @staticmethod
    def create(  # type: ignore
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: AlpacaDataClientConfig,
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
            The Alpaca data client instance.

        """
        # Call Rust factory function
        return create_alpaca_data_client(
            loop=loop,
            name=name,
            config=config,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
        )


class AlpacaLiveExecClientFactory(LiveExecClientFactory):
    """
    Provides an Alpaca live execution client factory.

    This is a thin wrapper over the Rust implementation that maintains
    compatibility with NautilusTrader's factory pattern used in examples.
    """

    @staticmethod
    def create(  # type: ignore
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: AlpacaExecClientConfig,
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
            The Alpaca execution client instance.

        """
        # Call Rust factory function
        return create_alpaca_exec_client(
            loop=loop,
            name=name,
            config=config,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
        )
