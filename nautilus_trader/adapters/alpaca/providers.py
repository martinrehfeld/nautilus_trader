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
Alpaca instrument provider implementation.

Loads instruments for US equities, cryptocurrencies, and options from Alpaca Markets.
"""

from nautilus_trader.common.providers import InstrumentProvider
from nautilus_trader.config import InstrumentProviderConfig
from nautilus_trader.core import nautilus_pyo3


class AlpacaInstrumentProvider(InstrumentProvider):
    """
    Provides instruments from Alpaca Markets.

    Supports:
    - US Equities (stocks and ETFs)
    - Cryptocurrencies (BTC/USD, ETH/USD, etc.)
    - Options (equity options)

    Parameters
    ----------
    client : nautilus_pyo3.AlpacaHttpClient
        The Alpaca HTTP client.
    config : InstrumentProviderConfig, optional
        The instrument provider configuration.

    """

    def __init__(
        self,
        client: nautilus_pyo3.AlpacaHttpClient,
        config: InstrumentProviderConfig | None = None,
    ) -> None:
        super().__init__(config=config)

        self._client = client
        self._log_warnings = config.log_warnings if config else True

    async def load_all_async(self, filters: dict | None = None) -> None:
        """
        Load all instruments from Alpaca for the configured asset classes.

        Parameters
        ----------
        filters : dict, optional
            The venue filters to apply.

        """
        # TODO: Implement instrument loading
        # This should:
        # 1. Call Rust HTTP client to fetch instruments
        # 2. Parse responses into Nautilus instruments
        # 3. Add instruments to the provider
        pass

    async def load_ids_async(
        self,
        instrument_ids: list,
        filters: dict | None = None,
    ) -> None:
        """
        Load specific instruments by ID.

        Parameters
        ----------
        instrument_ids : list[InstrumentId]
            The instrument IDs to load.
        filters : dict, optional
            The venue filters to apply.

        """
        # TODO: Implement specific instrument loading
        pass

    async def load_async(self, instrument_id, filters: dict | None = None):
        """
        Load a single instrument by ID.

        Parameters
        ----------
        instrument_id : InstrumentId
            The instrument ID to load.
        filters : dict, optional
            The venue filters to apply.

        """
        # TODO: Implement single instrument loading
        pass
