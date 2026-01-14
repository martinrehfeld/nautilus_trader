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
Alpaca execution client implementation.

Provides order submission and position management for:
- US Equities (stocks and ETFs)
- Cryptocurrencies (BTC/USD, ETH/USD, etc.) with GTC/IOC constraints
- Options (equity options) with DAY order constraint
"""

import asyncio

from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.common.enums import LogColor
from nautilus_trader.core import nautilus_pyo3
from nautilus_trader.live.execution_client import LiveExecutionClient
from nautilus_trader.model.identifiers import AccountId
from nautilus_trader.model.identifiers import ClientId


class AlpacaExecutionClient(LiveExecutionClient):
    """
    Provides an execution client for Alpaca Markets.

    Supports:
    - US Equities (stocks and ETFs)
    - Cryptocurrencies with constraints:
      * Only GTC or IOC time-in-force
      * No shorting allowed
    - Options with constraints:
      * Only DAY orders supported
      * Contract multiplier handling (typically 100)

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
    config : AlpacaExecClientConfig
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
        config,  # AlpacaExecClientConfig
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or "ALPACA"),
            venue=nautilus_pyo3.ALPACA_VENUE,
            account_id=AccountId(f"ALPACA-{name or 'default'}"),
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=instrument_provider,
        )

        self._config = config
        self._http_client = client

        # Log configuration
        self._log.info(f"paper_trading={config.paper_trading}", LogColor.BLUE)

    async def _connect(self) -> None:
        """
        Connect to Alpaca execution services.
        """
        self._log.info("Connecting to Alpaca execution services...")

        # Fetch account information
        # TODO: Implement account fetching and state management

        self._log.info("Connected to Alpaca execution services", LogColor.GREEN)

    async def _disconnect(self) -> None:
        """
        Disconnect from Alpaca execution services.
        """
        self._log.info("Disconnecting from Alpaca execution services...")

        # Clean up resources
        # TODO: Implement cleanup

        self._log.info("Disconnected from Alpaca execution services", LogColor.GREEN)

    # TODO: Implement order management methods:
    # - _submit_order
    # - _submit_order_list
    # - _modify_order
    # - _cancel_order
    # - _cancel_all_orders
    # - generate_order_status_reports
    # - generate_fill_reports
    # - generate_position_status_reports
    # etc.
