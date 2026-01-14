# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
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
Alpaca data client implementation.

Provides market data streaming and historical data requests for:
- US Equities (stocks and ETFs) via IEX or SIP feeds
- Cryptocurrencies (BTC/USD, ETH/USD, etc.)
- Options (equity options)
"""

import asyncio

from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.common.enums import LogColor
from nautilus_trader.core import nautilus_pyo3
from nautilus_trader.live.data_client import LiveMarketDataClient
from nautilus_trader.model.identifiers import ClientId


class AlpacaDataClient(LiveMarketDataClient):
    """
    Provides a data client for Alpaca Markets.

    Supports:
    - US Equities (stocks and ETFs) with IEX or SIP data feeds
    - Cryptocurrencies (BTC/USD, ETH/USD, etc.)
    - Options (equity options)

    Parameters
    ----------
    loop : asyncio.AbstractEventLoop
        The event loop for the client.
    client : nautilus_pyo3.AlpacaHttpClient
        The Alpaca HTTP client.
    msgbus : MessageBus
        The message bus for the client.
    cache : Cache
        The cache for the client.
    clock : LiveClock
        The clock for the client.
    instrument_provider : AlpacaInstrumentProvider
        The instrument provider.
    config : AlpacaDataClientConfig
        The configuration for the client.
    name : str, optional
        The custom client ID.

    """

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        client: nautilus_pyo3.AlpacaHttpClient,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        instrument_provider,  # AlpacaInstrumentProvider
        config,  # AlpacaDataClientConfig
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or "ALPACA"),
            venue=nautilus_pyo3.ALPACA_VENUE,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=instrument_provider,
        )

        self._config = config
        self._http_client = client

        # Log configuration
        self._log.info(f"data_feed={config.data_feed}", LogColor.BLUE)
        self._log.info(f"paper_trading={config.paper_trading}", LogColor.BLUE)

        # WebSocket client (will be initialized on connect)
        self._ws_client = None

    async def _connect(self) -> None:
        """
        Connect to Alpaca market data feeds.
        """
        self._log.info("Connecting to Alpaca data feeds...")

        # Initialize WebSocket client
        # TODO: Create WebSocket client based on config.data_feed
        # For now, this is a placeholder

        self._log.info("Connected to Alpaca data feeds", LogColor.GREEN)

    async def _disconnect(self) -> None:
        """
        Disconnect from Alpaca market data feeds.
        """
        self._log.info("Disconnecting from Alpaca data feeds...")

        # Close WebSocket connections
        # TODO: Implement WebSocket cleanup

        self._log.info("Disconnected from Alpaca data feeds", LogColor.GREEN)

    # TODO: Implement subscription methods:
    # - _subscribe_trade_ticks
    # - _subscribe_quote_ticks
    # - _subscribe_bars
    # - _request_bars
    # etc.
