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
Alpaca instrument provider implementation.

Loads instruments for US equities, cryptocurrencies, and options from Alpaca Markets.
"""

from decimal import Decimal

from nautilus_trader.common.providers import InstrumentProvider
from nautilus_trader.config import InstrumentProviderConfig
from nautilus_trader.core import nautilus_pyo3
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import Symbol
from nautilus_trader.model.instruments import CryptoFuture
from nautilus_trader.model.instruments import CryptoPerpetual
from nautilus_trader.model.instruments import CurrencyPair
from nautilus_trader.model.instruments import Equity
from nautilus_trader.model.instruments import OptionContract
from nautilus_trader.model.objects import Currency
from nautilus_trader.model.objects import Money
from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity


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
        # Create a base InstrumentProviderConfig if we received an Alpaca-specific config
        if config is not None and hasattr(config, 'load_all'):
            # This is AlpacaInstrumentProviderConfig from Rust
            base_config = InstrumentProviderConfig(
                load_all=config.load_all,
                log_warnings=config.log_warnings,
            )
            super().__init__(config=base_config)
            self._log_warnings = config.log_warnings
        else:
            # Standard InstrumentProviderConfig
            super().__init__(config=config)
            self._log_warnings = config.log_warnings if config else True

        self._client = client

    async def load_all_async(self, filters: dict | None = None) -> None:
        """
        Load all instruments from Alpaca for the configured asset classes.

        Parameters
        ----------
        filters : dict, optional
            The venue filters to apply.

        """
        self._log.info("Loading instruments from Alpaca...")

        try:
            # Fetch all active assets
            assets = await self._client.get_assets(status="active", asset_class=None)

            # Parse and add instruments
            for asset_data in assets:
                try:
                    instrument = self._parse_instrument(asset_data)
                    if instrument:
                        self.add(instrument)
                except Exception as e:
                    if self._log_warnings:
                        self._log.warning(f"Error parsing instrument {asset_data.get('symbol', 'unknown')}: {e}")

            self._log.info(f"Loaded {len(self.list_all())} instruments from Alpaca")

        except Exception as e:
            self._log.error(f"Error loading instruments from Alpaca: {e}")
            raise

    async def load_ids_async(
        self,
        instrument_ids: list[InstrumentId],
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
        for instrument_id in instrument_ids:
            try:
                await self.load_async(instrument_id, filters)
            except Exception as e:
                if self._log_warnings:
                    self._log.warning(f"Error loading instrument {instrument_id}: {e}")

    async def load_async(self, instrument_id: InstrumentId, filters: dict | None = None):
        """
        Load a single instrument by ID.

        Parameters
        ----------
        instrument_id : InstrumentId
            The instrument ID to load.
        filters : dict, optional
            The venue filters to apply.

        """
        try:
            symbol = instrument_id.symbol.value
            asset_data = await self._client.get_asset(symbol)

            instrument = self._parse_instrument(asset_data)
            if instrument:
                self.add(instrument)

        except Exception as e:
            if self._log_warnings:
                self._log.warning(f"Error loading instrument {instrument_id}: {e}")
            raise

    def _parse_instrument(self, asset_data: dict):
        """
        Parse asset data into a Nautilus instrument.

        Parameters
        ----------
        asset_data : dict
            Asset data from Alpaca API.

        Returns
        -------
        Instrument or None
            The parsed instrument or None if parsing failed.

        """
        try:
            asset_class = asset_data.get("class", "").lower()
            symbol = asset_data["symbol"]
            name = asset_data.get("name", symbol)

            # Create instrument ID
            instrument_id = InstrumentId(
                symbol=Symbol(symbol),
                venue=nautilus_pyo3.ALPACA_VENUE,
            )

            # Common fields
            price_precision = 2  # Default for equities
            size_precision = 0  # Whole shares by default
            price_increment = Price.from_str("0.01")
            size_increment = Quantity.from_int(1)
            lot_size = Quantity.from_int(1)

            # Margin defaults
            margin_init = Decimal("0.50")  # 50% margin for stocks
            margin_maint = Decimal("0.25")  # 25% maintenance margin

            # Fees
            maker_fee = Decimal("0")
            taker_fee = Decimal("0")

            # Parse based on asset class
            if asset_class == "us_equity":
                # US Equity (stock or ETF)
                return Equity(
                    instrument_id=instrument_id,
                    raw_symbol=Symbol(symbol),
                    currency=Currency.from_str("USD"),
                    price_precision=price_precision,
                    price_increment=price_increment,
                    lot_size=lot_size,
                    isin=asset_data.get("isin"),
                    margin_init=margin_init,
                    margin_maint=margin_maint,
                    maker_fee=maker_fee,
                    taker_fee=taker_fee,
                    ts_event=0,
                    ts_init=0,
                )

            elif asset_class == "crypto":
                # Cryptocurrency pair
                # Alpaca crypto symbols are like "BTC/USD"
                if "/" in symbol:
                    base_currency_str, quote_currency_str = symbol.split("/")
                else:
                    # Fallback parsing
                    base_currency_str = symbol[:3]
                    quote_currency_str = symbol[3:]

                base_currency = Currency.from_str(base_currency_str)
                quote_currency = Currency.from_str(quote_currency_str)

                # Crypto has higher precision
                price_precision = 8
                size_precision = 8
                price_increment = Price.from_str("0.00000001")
                size_increment = Quantity.from_str("0.00000001")

                return CurrencyPair(
                    instrument_id=instrument_id,
                    raw_symbol=Symbol(symbol),
                    base_currency=base_currency,
                    quote_currency=quote_currency,
                    price_precision=price_precision,
                    size_precision=size_precision,
                    price_increment=price_increment,
                    size_increment=size_increment,
                    lot_size=size_increment,
                    max_quantity=Quantity.from_str("1000000"),
                    min_quantity=size_increment,
                    max_price=None,
                    min_price=None,
                    margin_init=margin_init,
                    margin_maint=margin_maint,
                    maker_fee=maker_fee,
                    taker_fee=taker_fee,
                    ts_event=0,
                    ts_init=0,
                )

            elif asset_class == "us_option":
                # Options contract
                # This requires more complex parsing - for now, just log warning
                if self._log_warnings:
                    self._log.warning(f"Option contract parsing not yet implemented: {symbol}")
                return None

            else:
                if self._log_warnings:
                    self._log.warning(f"Unknown asset class: {asset_class} for symbol {symbol}")
                return None

        except Exception as e:
            if self._log_warnings:
                self._log.warning(f"Error parsing instrument: {e}")
            return None
