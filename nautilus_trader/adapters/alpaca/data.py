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
Alpaca data client implementation.

Provides market data streaming and historical data requests for:
- US Equities (stocks and ETFs) via IEX or SIP feeds
- Cryptocurrencies (BTC/USD, ETH/USD, etc.)
- Options (equity options)
"""

import asyncio

import msgspec

from nautilus_trader.adapters.alpaca.providers import AlpacaInstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.common.enums import LogColor
from nautilus_trader.core import nautilus_pyo3
from nautilus_trader.core.datetime import millis_to_nanos
from nautilus_trader.live.data_client import LiveMarketDataClient
from nautilus_trader.model.data import Bar
from nautilus_trader.model.data import BarType
from nautilus_trader.model.data import QuoteTick
from nautilus_trader.model.data import TradeTick
from nautilus_trader.model.enums import AggressorSide
from nautilus_trader.model.enums import BarAggregation
from nautilus_trader.model.enums import PriceType
from nautilus_trader.model.identifiers import ClientId
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import TradeId
from nautilus_trader.model.identifiers import Venue
from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity


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
        instrument_provider: AlpacaInstrumentProvider,
        config,  # AlpacaDataClientConfig
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or "ALPACA"),
            venue=Venue(str(config.venue)) if hasattr(config, 'venue') else Venue(nautilus_pyo3.ALPACA_VENUE),
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
        self._ws_client: nautilus_pyo3.WebSocketClient | None = None
        self._decoder = msgspec.json.Decoder()

        # Track subscriptions
        self._trade_subscriptions: set[str] = set()
        self._quote_subscriptions: set[str] = set()
        self._bar_subscriptions: set[str] = set()

    async def _connect(self) -> None:
        """
        Connect to Alpaca market data feeds.
        """
        self._log.info("Connecting to Alpaca data feeds...")

        # Initialize instrument provider
        await self._instrument_provider.initialize()

        # Create Alpaca WebSocket client for market data
        alpaca_ws_client = nautilus_pyo3.AlpacaWebSocketClient(
            api_key=self._config.api_key or "",
            api_secret=self._config.api_secret or "",
            asset_class=self._config.asset_class,
            data_feed=self._config.data_feed,
            url_override=None,
        )

        # Create WebSocket config
        ws_config = nautilus_pyo3.WebSocketConfig(
            url=alpaca_ws_client.url,
            headers=[],
            heartbeat=20,
        )

        # Create WebSocket client
        self._ws_client = await nautilus_pyo3.WebSocketClient.connect(
            loop_=self._loop,
            config=ws_config,
            handler=self._handle_ws_message,
            post_reconnection=self._post_reconnection,
        )

        # Perform initial authentication
        await self._post_connection()

        self._log.info("Connected to Alpaca data feeds", LogColor.GREEN)

    async def _disconnect(self) -> None:
        """
        Disconnect from Alpaca market data feeds.
        """
        self._log.info("Disconnecting from Alpaca data feeds...")

        # Close WebSocket connection
        if self._ws_client:
            await self._ws_client.disconnect()
            self._ws_client = None

        self._log.info("Disconnected from Alpaca data feeds", LogColor.GREEN)

    async def _post_connection(self) -> None:
        """
        Actions to perform post connection.

        Sends authentication message to Alpaca WebSocket.
        """
        if not self._ws_client:
            return

        # Create and send authentication message
        alpaca_ws_client = nautilus_pyo3.AlpacaWebSocketClient(
            api_key=self._config.api_key or "",
            api_secret=self._config.api_secret or "",
            asset_class=self._config.asset_class,
            data_feed=self._config.data_feed,
            url_override=None,
        )

        auth_msg = alpaca_ws_client.auth_message()
        self._log.debug(f"Sending auth message: {auth_msg}")
        await self._ws_client.send_text(auth_msg.encode("utf-8"))

    async def _post_reconnection(self) -> None:
        """
        Actions to perform post reconnection.

        Re-authenticates and re-subscribes to all active streams.
        """
        await self._post_connection()

        # Resubscribe to all active subscriptions
        if self._trade_subscriptions:
            await self._resubscribe_trades(list(self._trade_subscriptions))

        if self._quote_subscriptions:
            await self._resubscribe_quotes(list(self._quote_subscriptions))

        if self._bar_subscriptions:
            await self._resubscribe_bars(list(self._bar_subscriptions))

    async def _resubscribe_trades(self, symbols: list[str]) -> None:
        """Resubscribe to trade streams."""
        if not self._ws_client:
            return

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_trades_message(symbols)
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Resubscribed to trades: {symbols}")

    async def _resubscribe_quotes(self, symbols: list[str]) -> None:
        """Resubscribe to quote streams."""
        if not self._ws_client:
            return

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_quotes_message(symbols)
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Resubscribed to quotes: {symbols}")

    async def _resubscribe_bars(self, symbols: list[str]) -> None:
        """Resubscribe to bar streams."""
        if not self._ws_client:
            return

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_bars_message(symbols)
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Resubscribed to bars: {symbols}")

    def _handle_ws_message(self, raw: bytes) -> None:
        """
        Handle incoming WebSocket message.

        Parameters
        ----------
        raw : bytes
            The raw message bytes from WebSocket.

        """
        try:
            # Decode JSON message
            messages = self._decoder.decode(raw)

            # Alpaca sends array of messages
            if not isinstance(messages, list):
                messages = [messages]

            for msg in messages:
                msg_type = msg.get("T")

                if msg_type == "success":
                    self._log.info(f"Connection success: {msg.get('msg')}", LogColor.GREEN)
                elif msg_type == "subscription":
                    self._log.info(f"Subscription confirmed: {msg}")
                elif msg_type == "error":
                    self._log.error(f"WebSocket error: {msg.get('msg')} (code={msg.get('code')})")
                elif msg_type == "t":
                    self._handle_trade(msg)
                elif msg_type == "q":
                    self._handle_quote(msg)
                elif msg_type == "b":
                    self._handle_bar(msg)
                else:
                    self._log.warning(f"Unknown message type: {msg_type}")

        except Exception as e:
            self._log.error(f"Error handling WebSocket message: {e}")

    def _handle_trade(self, msg: dict) -> None:
        """Handle trade tick message."""
        try:
            symbol = msg["S"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")
            instrument = self._cache.instrument(instrument_id)

            if not instrument:
                self._log.warning(f"Instrument not found: {instrument_id}")
                return

            # Parse trade tick
            trade_tick = TradeTick(
                instrument_id=instrument_id,
                price=Price.from_str(str(msg["p"])),
                size=Quantity.from_int(msg["s"]),
                aggressor_side=AggressorSide.NO_AGGRESSOR,
                trade_id=TradeId(str(msg["i"])),
                ts_event=millis_to_nanos(self._parse_timestamp(msg["t"])),
                ts_init=self._clock.timestamp_ns(),
            )

            self._handle_data(trade_tick)

        except Exception as e:
            self._log.error(f"Error parsing trade tick: {e}")

    def _handle_quote(self, msg: dict) -> None:
        """Handle quote tick message."""
        try:
            symbol = msg["S"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")
            instrument = self._cache.instrument(instrument_id)

            if not instrument:
                self._log.warning(f"Instrument not found: {instrument_id}")
                return

            # Parse quote tick
            quote_tick = QuoteTick(
                instrument_id=instrument_id,
                bid_price=Price.from_str(str(msg["bp"])),
                ask_price=Price.from_str(str(msg["ap"])),
                bid_size=Quantity.from_int(msg["bs"]),
                ask_size=Quantity.from_int(msg["as"]),
                ts_event=millis_to_nanos(self._parse_timestamp(msg["t"])),
                ts_init=self._clock.timestamp_ns(),
            )

            self._handle_data(quote_tick)

        except Exception as e:
            self._log.error(f"Error parsing quote tick: {e}")

    def _handle_bar(self, msg: dict) -> None:
        """Handle bar message."""
        try:
            symbol = msg["S"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")
            instrument = self._cache.instrument(instrument_id)

            if not instrument:
                self._log.warning(f"Instrument not found: {instrument_id}")
                return

            # Create bar type (1-MINUTE bars from Alpaca stream)
            bar_type = BarType(
                instrument_id=instrument_id,
                bar_spec=nautilus_pyo3.BarSpecification(
                    step=1,
                    aggregation=BarAggregation.MINUTE,
                    price_type=PriceType.LAST,
                ),
                aggregation_source=self.id,
            )

            # Parse bar
            bar = Bar(
                bar_type=bar_type,
                open=Price.from_str(str(msg["o"])),
                high=Price.from_str(str(msg["h"])),
                low=Price.from_str(str(msg["l"])),
                close=Price.from_str(str(msg["c"])),
                volume=Quantity.from_int(msg["v"]),
                ts_event=millis_to_nanos(self._parse_timestamp(msg["t"])),
                ts_init=self._clock.timestamp_ns(),
            )

            self._handle_data(bar)

        except Exception as e:
            self._log.error(f"Error parsing bar: {e}")

    def _parse_timestamp(self, ts_str: str) -> int:
        """
        Parse RFC3339 timestamp to milliseconds.

        Parameters
        ----------
        ts_str : str
            RFC3339 timestamp string.

        Returns
        -------
        int
            Timestamp in milliseconds.

        """
        import datetime

        dt = datetime.datetime.fromisoformat(ts_str.rstrip("Z"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=datetime.UTC)
        return int(dt.timestamp() * 1000)

    async def _subscribe_trade_ticks(self, instrument_id: InstrumentId) -> None:
        """
        Subscribe to trade ticks for an instrument.

        Parameters
        ----------
        instrument_id : InstrumentId
            The instrument ID to subscribe to.

        """
        if not self._ws_client:
            self._log.error("WebSocket client not connected")
            return

        symbol = instrument_id.symbol.value
        self._trade_subscriptions.add(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_trades_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Subscribed to trade ticks: {instrument_id}")

    async def _subscribe_quote_ticks(self, instrument_id: InstrumentId) -> None:
        """
        Subscribe to quote ticks for an instrument.

        Parameters
        ----------
        instrument_id : InstrumentId
            The instrument ID to subscribe to.

        """
        if not self._ws_client:
            self._log.error("WebSocket client not connected")
            return

        symbol = instrument_id.symbol.value
        self._quote_subscriptions.add(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_quotes_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Subscribed to quote ticks: {instrument_id}")

    async def _subscribe_bars(self, bar_type: BarType) -> None:
        """
        Subscribe to bars for an instrument.

        Parameters
        ----------
        bar_type : BarType
            The bar type to subscribe to.

        """
        if not self._ws_client:
            self._log.error("WebSocket client not connected")
            return

        symbol = bar_type.instrument_id.symbol.value
        self._bar_subscriptions.add(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_bars_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Subscribed to bars: {bar_type}")

    async def _unsubscribe_trade_ticks(self, instrument_id: InstrumentId) -> None:
        """Unsubscribe from trade ticks."""
        if not self._ws_client:
            return

        symbol = instrument_id.symbol.value
        self._trade_subscriptions.discard(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.unsubscribe_trades_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Unsubscribed from trade ticks: {instrument_id}")

    async def _unsubscribe_quote_ticks(self, instrument_id: InstrumentId) -> None:
        """Unsubscribe from quote ticks."""
        if not self._ws_client:
            return

        symbol = instrument_id.symbol.value
        self._quote_subscriptions.discard(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.unsubscribe_quotes_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Unsubscribed from quote ticks: {instrument_id}")

    async def _unsubscribe_bars(self, bar_type: BarType) -> None:
        """Unsubscribe from bars."""
        if not self._ws_client:
            return

        symbol = bar_type.instrument_id.symbol.value
        self._bar_subscriptions.discard(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.unsubscribe_bars_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Unsubscribed from bars: {bar_type}")

    async def _request_bars(
        self,
        bar_type: BarType,
        start: int | None = None,
        end: int | None = None,
    ) -> None:
        """
        Request historical bars from Alpaca HTTP API.

        Parameters
        ----------
        bar_type : BarType
            The bar type to request.
        start : int, optional
            The start timestamp in nanoseconds.
        end : int, optional
            The end timestamp in nanoseconds.

        """
        # TODO: Implement historical bar requests via HTTP client
        self._log.warning(f"Historical bar requests not yet implemented for {bar_type}")
