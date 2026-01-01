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
Alpaca data client implementation.
"""

from __future__ import annotations

import asyncio
import contextlib
from decimal import Decimal
from typing import TYPE_CHECKING

import msgspec

from nautilus_trader.adapters.alpaca.common.constants import ALPACA_VENUE
from nautilus_trader.adapters.alpaca.config import AlpacaDataClientConfig
from nautilus_trader.adapters.alpaca.providers import AlpacaInstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.common.enums import LogColor
from nautilus_trader.data.messages import RequestBars
from nautilus_trader.data.messages import RequestInstrument
from nautilus_trader.data.messages import RequestInstruments
from nautilus_trader.data.messages import RequestQuoteTicks
from nautilus_trader.data.messages import RequestTradeTicks
from nautilus_trader.data.messages import SubscribeBars
from nautilus_trader.data.messages import SubscribeInstrument
from nautilus_trader.data.messages import SubscribeInstruments
from nautilus_trader.data.messages import SubscribeOrderBook
from nautilus_trader.data.messages import SubscribeQuoteTicks
from nautilus_trader.data.messages import SubscribeTradeTicks
from nautilus_trader.data.messages import UnsubscribeBars
from nautilus_trader.data.messages import UnsubscribeInstrument
from nautilus_trader.data.messages import UnsubscribeInstruments
from nautilus_trader.data.messages import UnsubscribeOrderBook
from nautilus_trader.data.messages import UnsubscribeQuoteTicks
from nautilus_trader.data.messages import UnsubscribeTradeTicks
from nautilus_trader.live.data_client import LiveMarketDataClient
from nautilus_trader.model.data import QuoteTick
from nautilus_trader.model.data import TradeTick
from nautilus_trader.model.enums import AggressorSide
from nautilus_trader.model.identifiers import ClientId
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import TradeId
from nautilus_trader.model.instruments import CurrencyPair
from nautilus_trader.model.instruments import OptionContract
from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity


if TYPE_CHECKING:
    from nautilus_trader.adapters.alpaca.http.client import AlpacaHttpClient


class AlpacaDataClient(LiveMarketDataClient):
    """
    Provides a data client for Alpaca Markets.

    Supports:
    - US equities (stocks and ETFs)
    - Cryptocurrencies (BTC/USD, ETH/USD, etc.)
    - Options (equity options)

    Parameters
    ----------
    loop : asyncio.AbstractEventLoop
        The event loop for the client.
    client : AlpacaHttpClient
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
        client: AlpacaHttpClient,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        instrument_provider: AlpacaInstrumentProvider,
        config: AlpacaDataClientConfig,
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or ALPACA_VENUE.value),
            venue=ALPACA_VENUE,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=instrument_provider,
        )

        self._instrument_provider: AlpacaInstrumentProvider = instrument_provider
        self._http_client = client
        self._config = config

        # WebSocket connection state
        self._ws_connected = False
        self._ws_task: asyncio.Task | None = None
        self._subscribed_trades: set[str] = set()
        self._subscribed_quotes: set[str] = set()
        self._subscribed_bars: set[str] = set()

        # Log configuration
        self._log.info(f"paper_trading={config.paper_trading}", LogColor.BLUE)
        self._log.info(f"data_feed={config.data_feed.value}", LogColor.BLUE)

    @property
    def instrument_provider(self) -> AlpacaInstrumentProvider:
        return self._instrument_provider

    async def _connect(self) -> None:
        """
        Connect the data client.
        """
        await self.instrument_provider.initialize()
        self._send_all_instruments_to_data_engine()
        self._log.info("Alpaca data client connected", LogColor.GREEN)

    async def _disconnect(self) -> None:
        """
        Disconnect the data client.
        """
        # Close WebSocket if connected
        if self._ws_task is not None:
            self._ws_task.cancel()
            with contextlib.suppress(asyncio.CancelledError):
                await self._ws_task
            self._ws_task = None

        # Close HTTP client
        await self._http_client.close()
        self._log.info("Alpaca data client disconnected", LogColor.GREEN)

    def _send_all_instruments_to_data_engine(self) -> None:
        """
        Send all loaded instruments to the data engine.
        """
        for instrument in self.instrument_provider.get_all().values():
            self._handle_data(instrument)

        for currency in self.instrument_provider.currencies().values():
            self._cache.add_currency(currency)

    # -- SUBSCRIPTIONS ---------------------------------------------------------------------

    async def _subscribe_instrument(self, command: SubscribeInstrument) -> None:
        self._log.info(f"Subscribed to instrument updates for {command.instrument_id}")

    async def _subscribe_instruments(self, command: SubscribeInstruments) -> None:
        self._log.info("Subscribed to instruments updates")

    async def _subscribe_order_book_deltas(self, command: SubscribeOrderBook) -> None:
        self._log.warning(
            f"Cannot subscribe to order book deltas for {command.instrument_id}: "
            "not supported by Alpaca",
        )

    async def _subscribe_order_book_snapshots(self, command: SubscribeOrderBook) -> None:
        self._log.warning(
            f"Cannot subscribe to order book snapshots for {command.instrument_id}: "
            "not supported by Alpaca",
        )

    async def _subscribe_quote_ticks(self, command: SubscribeQuoteTicks) -> None:
        symbol = command.instrument_id.symbol.value
        self._subscribed_quotes.add(symbol)
        self._log.info(f"Subscribed to quote ticks for {command.instrument_id}")

    async def _subscribe_trade_ticks(self, command: SubscribeTradeTicks) -> None:
        symbol = command.instrument_id.symbol.value
        self._subscribed_trades.add(symbol)
        self._log.info(f"Subscribed to trade ticks for {command.instrument_id}")

    async def _subscribe_bars(self, command: SubscribeBars) -> None:
        bar_type = str(command.bar_type)
        self._subscribed_bars.add(bar_type)
        self._log.info(f"Subscribed to bars for {command.bar_type}")

    async def _unsubscribe_instrument(self, command: UnsubscribeInstrument) -> None:
        self._log.info(f"Unsubscribed from instrument updates for {command.instrument_id}")

    async def _unsubscribe_instruments(self, command: UnsubscribeInstruments) -> None:
        self._log.info("Unsubscribed from instruments updates")

    async def _unsubscribe_order_book_deltas(self, command: UnsubscribeOrderBook) -> None:
        self._log.warning(
            f"Cannot unsubscribe from order book deltas for {command.instrument_id}: "
            "not supported by Alpaca",
        )

    async def _unsubscribe_order_book_snapshots(self, command: UnsubscribeOrderBook) -> None:
        self._log.warning(
            f"Cannot unsubscribe from order book snapshots for {command.instrument_id}: "
            "not supported by Alpaca",
        )

    async def _unsubscribe_quote_ticks(self, command: UnsubscribeQuoteTicks) -> None:
        symbol = command.instrument_id.symbol.value
        self._subscribed_quotes.discard(symbol)
        self._log.info(f"Unsubscribed from quote ticks for {command.instrument_id}")

    async def _unsubscribe_trade_ticks(self, command: UnsubscribeTradeTicks) -> None:
        symbol = command.instrument_id.symbol.value
        self._subscribed_trades.discard(symbol)
        self._log.info(f"Unsubscribed from trade ticks for {command.instrument_id}")

    async def _unsubscribe_bars(self, command: UnsubscribeBars) -> None:
        bar_type = str(command.bar_type)
        self._subscribed_bars.discard(bar_type)
        self._log.info(f"Unsubscribed from bars for {command.bar_type}")

    # -- REQUESTS -------------------------------------------------------------------------

    async def _request_instrument(self, request: RequestInstrument) -> None:
        instrument = self.instrument_provider.find(request.instrument_id)
        if instrument:
            self._handle_data(instrument)
            self._log.debug(f"Sent instrument {request.instrument_id}")
        else:
            self._log.error(f"Instrument not found: {request.instrument_id}")

    async def _request_instruments(self, request: RequestInstruments) -> None:
        instruments = []
        for instrument_id in request.instrument_ids:
            instrument = self.instrument_provider.find(instrument_id)
            if instrument:
                instruments.append(instrument)
                self._handle_data(instrument)
                self._log.debug(f"Sent instrument {instrument_id}")
            else:
                self._log.warning(f"Instrument not found: {instrument_id}")

        if not instruments:
            self._log.warning("No instruments found for request")
        else:
            self._log.info(f"Sent {len(instruments)} instruments")

    async def _request_quote_ticks(self, request: RequestQuoteTicks) -> None:
        """
        Request historical quote ticks from Alpaca.
        """
        instrument_id = request.instrument_id
        symbol = instrument_id.symbol.value

        try:
            instrument = self._cache.instrument(instrument_id)
            if instrument is None:
                self._log.error(f"Instrument not found in cache: {instrument_id}")
                return

            # Determine API path based on instrument type
            api_path = self._get_quotes_api_path(instrument, symbol)

            # Build request parameters
            params: dict = {"symbols": symbol, "limit": request.limit or 1000}
            if request.start:
                params["start"] = request.start.isoformat()
            if request.end:
                params["end"] = request.end.isoformat()

            # Request quotes from Alpaca data API
            response = await self._http_client.get(
                api_path,
                params=params,
                use_data_api=True,
            )
            data = msgspec.json.decode(response)

            quotes = data.get("quotes", [])
            quote_ticks = []

            for quote in quotes:
                quote_tick = self._parse_quote_tick(quote, instrument_id)
                if quote_tick:
                    quote_ticks.append(quote_tick)

            self._handle_quote_ticks(
                instrument_id,
                quote_ticks,
                request.id,
                request.start,
                request.end,
                request.params,
            )

        except Exception as e:
            self._log.exception(f"Error requesting quote ticks for {instrument_id}", e)

    async def _request_trade_ticks(self, request: RequestTradeTicks) -> None:
        """
        Request historical trade ticks from Alpaca.
        """
        instrument_id = request.instrument_id
        symbol = instrument_id.symbol.value

        try:
            instrument = self._cache.instrument(instrument_id)
            if instrument is None:
                self._log.error(f"Instrument not found in cache: {instrument_id}")
                return

            # Determine API path based on instrument type
            api_path = self._get_trades_api_path(instrument, symbol)

            # Build request parameters
            params: dict = {"symbols": symbol, "limit": request.limit or 1000}
            if request.start:
                params["start"] = request.start.isoformat()
            if request.end:
                params["end"] = request.end.isoformat()

            # Request trades from Alpaca data API
            response = await self._http_client.get(
                api_path,
                params=params,
                use_data_api=True,
            )
            data = msgspec.json.decode(response)

            trades = data.get("trades", [])
            trade_ticks = []

            for trade in trades:
                trade_tick = self._parse_trade_tick(trade, instrument_id)
                if trade_tick:
                    trade_ticks.append(trade_tick)

            self._handle_trade_ticks(
                instrument_id,
                trade_ticks,
                request.id,
                request.start,
                request.end,
                request.params,
            )

        except Exception as e:
            self._log.exception(f"Error requesting trade ticks for {instrument_id}", e)

    async def _request_bars(self, request: RequestBars) -> None:
        """
        Request historical bars from Alpaca.
        """
        self._log.warning("Historical bar requests not yet implemented for Alpaca")

    def _parse_quote_tick(
        self,
        quote: dict,
        instrument_id: InstrumentId,
    ) -> QuoteTick | None:
        """
        Parse an Alpaca quote into a QuoteTick.
        """
        try:
            instrument = self._cache.instrument(instrument_id)
            if instrument is None:
                return None

            # Parse timestamp
            ts_str = quote.get("t")
            if ts_str:
                from datetime import datetime

                ts = datetime.fromisoformat(ts_str)
                ts_event = int(ts.timestamp() * 1_000_000_000)
            else:
                ts_event = self._clock.timestamp_ns()

            return QuoteTick(
                instrument_id=instrument_id,
                bid_price=Price(Decimal(str(quote.get("bp", 0))), instrument.price_precision),
                ask_price=Price(Decimal(str(quote.get("ap", 0))), instrument.price_precision),
                bid_size=Quantity(Decimal(str(quote.get("bs", 0))), 0),
                ask_size=Quantity(Decimal(str(quote.get("as", 0))), 0),
                ts_event=ts_event,
                ts_init=self._clock.timestamp_ns(),
            )
        except Exception as e:
            self._log.warning(f"Error parsing quote tick: {e}")
            return None

    def _parse_trade_tick(
        self,
        trade: dict,
        instrument_id: InstrumentId,
    ) -> TradeTick | None:
        """
        Parse an Alpaca trade into a TradeTick.
        """
        try:
            instrument = self._cache.instrument(instrument_id)
            if instrument is None:
                return None

            # Parse timestamp
            ts_str = trade.get("t")
            if ts_str:
                from datetime import datetime

                ts = datetime.fromisoformat(ts_str)
                ts_event = int(ts.timestamp() * 1_000_000_000)
            else:
                ts_event = self._clock.timestamp_ns()

            return TradeTick(
                instrument_id=instrument_id,
                price=Price(Decimal(str(trade.get("p", 0))), instrument.price_precision),
                size=Quantity(Decimal(str(trade.get("s", 0))), 0),
                aggressor_side=AggressorSide.NO_AGGRESSOR,  # Alpaca doesn't provide this
                trade_id=TradeId(str(trade.get("i", ts_event))),
                ts_event=ts_event,
                ts_init=self._clock.timestamp_ns(),
            )
        except Exception as e:
            self._log.warning(f"Error parsing trade tick: {e}")
            return None

    # -- ASSET CLASS ROUTING ---------------------------------------------------------------

    def _get_quotes_api_path(self, instrument, symbol: str) -> str:
        """
        Get the API path for quote requests based on instrument type.
        """
        if isinstance(instrument, CurrencyPair):
            # Crypto quotes endpoint
            # Convert BTC/USD -> BTCUSD for API
            api_symbol = symbol.replace("/", "")
            return f"/v1beta3/crypto/us/quotes/{api_symbol}"
        elif isinstance(instrument, OptionContract):
            # Options quotes endpoint
            return f"/v1beta1/options/quotes/{symbol}"
        else:
            # Stock quotes endpoint (default)
            return f"/v2/stocks/{symbol}/quotes"

    def _get_trades_api_path(self, instrument, symbol: str) -> str:
        """
        Get the API path for trade requests based on instrument type.
        """
        if isinstance(instrument, CurrencyPair):
            # Crypto trades endpoint
            # Convert BTC/USD -> BTCUSD for API
            api_symbol = symbol.replace("/", "")
            return f"/v1beta3/crypto/us/trades/{api_symbol}"
        elif isinstance(instrument, OptionContract):
            # Options trades endpoint
            return f"/v1beta1/options/trades/{symbol}"
        else:
            # Stock trades endpoint (default)
            return f"/v2/stocks/{symbol}/trades"

    def _get_bars_api_path(self, instrument, symbol: str) -> str:
        """
        Get the API path for bar requests based on instrument type.
        """
        if isinstance(instrument, CurrencyPair):
            # Crypto bars endpoint
            api_symbol = symbol.replace("/", "")
            return f"/v1beta3/crypto/us/bars/{api_symbol}"
        elif isinstance(instrument, OptionContract):
            # Options bars endpoint
            return f"/v1beta1/options/bars/{symbol}"
        else:
            # Stock bars endpoint (default)
            return f"/v2/stocks/{symbol}/bars"

    def _get_ws_stream_path(self, instrument) -> str:
        """
        Get the WebSocket stream path based on instrument type.
        """
        data_feed = self._config.data_feed.value  # "iex" or "sip"
        if isinstance(instrument, CurrencyPair):
            # Crypto WebSocket stream
            return "wss://stream.data.alpaca.markets/v1beta3/crypto/us"
        elif isinstance(instrument, OptionContract):
            # Options WebSocket stream
            return "wss://stream.data.alpaca.markets/v1beta1/options"
        else:
            # Stock WebSocket stream (IEX or SIP feed)
            return f"wss://stream.data.alpaca.markets/v2/{data_feed}"
