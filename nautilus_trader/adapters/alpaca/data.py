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
from nautilus_trader.data.messages import RequestInstrument
from nautilus_trader.live.data_client import LiveMarketDataClient
from nautilus_trader.model.data import BarType
from nautilus_trader.model.identifiers import ClientId
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import Venue
from nautilus_trader.model.instruments import Instrument


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
        self._orderbook_subscriptions: set[str] = set()

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
            url_override=self._config.base_url_ws,
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

        self._log.info(f"Connected to Alpaca data feed {alpaca_ws_client.url}", LogColor.GREEN)

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
            url_override=self._config.base_url_ws,
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

        if self._orderbook_subscriptions:
            await self._resubscribe_orderbooks(list(self._orderbook_subscriptions))

    async def _resubscribe_trades(self, symbols: list[str]) -> None:
        """Resubscribe to trade streams."""
        if not self._ws_client:
            return

        normalized_symbols = [self._normalize_symbol(s) for s in symbols]
        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_trades_message(normalized_symbols)
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Resubscribed to trades: {symbols}")

    async def _resubscribe_quotes(self, symbols: list[str]) -> None:
        """Resubscribe to quote streams."""
        if not self._ws_client:
            return

        normalized_symbols = [self._normalize_symbol(s) for s in symbols]
        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_quotes_message(normalized_symbols)
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Resubscribed to quotes: {symbols}")

    async def _resubscribe_bars(self, symbols: list[str]) -> None:
        """Resubscribe to bar streams."""
        if not self._ws_client:
            return

        normalized_symbols = [self._normalize_symbol(s) for s in symbols]
        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_bars_message(normalized_symbols)
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Resubscribed to bars: {symbols}")

    async def _resubscribe_orderbooks(self, symbols: list[str]) -> None:
        """Resubscribe to orderbook streams."""
        if not self._ws_client:
            return

        normalized_symbols = [self._normalize_symbol(s) for s in symbols]
        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_orderbooks_message(normalized_symbols)
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Resubscribed to orderbooks: {symbols}")

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
                elif msg_type == "o":
                    self._handle_orderbook(msg)
                else:
                    self._log.warning(f"Unknown message type: {msg_type}")

        except Exception as e:
            self._log.error(f"Error handling WebSocket message: {e}")

    def _handle_trade(self, msg: dict) -> None:
        """Handle trade tick message."""
        try:
            from nautilus_trader.core import nautilus_pyo3
            from nautilus_trader.core.nautilus_pyo3.alpaca import parse_trade_tick
            from nautilus_trader.model.data import TradeTick

            symbol = msg["S"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")
            instrument = self._cache.instrument(instrument_id)

            if not instrument:
                self._log.warning(f"Instrument not found: {instrument_id}")
                return

            # Convert Python InstrumentId to PyO3 InstrumentId
            pyo3_instrument_id = nautilus_pyo3.InstrumentId.from_str(instrument_id.value)

            # Parse trade tick using Rust (returns PyO3 TradeTick)
            pyo3_trade_tick = parse_trade_tick(
                msg,
                pyo3_instrument_id,
                instrument.price_precision,
                instrument.size_precision,
                self._clock.timestamp_ns(),
            )

            # Convert PyO3 type to Python type
            trade_tick = TradeTick.from_pyo3(pyo3_trade_tick)
            self._handle_data(trade_tick)

        except Exception as e:
            self._log.error(f"Error parsing trade tick: {e}")

    def _handle_quote(self, msg: dict) -> None:
        """Handle quote tick message."""
        try:
            from nautilus_trader.core import nautilus_pyo3
            from nautilus_trader.core.nautilus_pyo3.alpaca import parse_quote_tick
            from nautilus_trader.model.data import QuoteTick

            symbol = msg["S"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")
            instrument = self._cache.instrument(instrument_id)

            if not instrument:
                self._log.warning(f"Instrument not found: {instrument_id}")
                return

            # Convert Python InstrumentId to PyO3 InstrumentId
            pyo3_instrument_id = nautilus_pyo3.InstrumentId.from_str(instrument_id.value)

            # Parse quote tick using Rust (returns PyO3 QuoteTick)
            pyo3_quote_tick = parse_quote_tick(
                msg,
                pyo3_instrument_id,
                instrument.price_precision,
                instrument.size_precision,
                self._clock.timestamp_ns(),
            )

            # Convert PyO3 type to Python type
            quote_tick = QuoteTick.from_pyo3(pyo3_quote_tick)
            self._handle_data(quote_tick)

        except Exception as e:
            self._log.error(f"Error parsing quote tick: {e}")

    def _handle_bar(self, msg: dict) -> None:
        """Handle bar message."""
        try:
            from nautilus_trader.core import nautilus_pyo3
            from nautilus_trader.core.nautilus_pyo3.alpaca import create_bar_type, parse_bar
            from nautilus_trader.model.data import Bar

            symbol = msg["S"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")
            instrument = self._cache.instrument(instrument_id)

            if not instrument:
                self._log.warning(f"Instrument not found: {instrument_id}")
                return

            # Convert Python InstrumentId to PyO3 InstrumentId
            pyo3_instrument_id = nautilus_pyo3.InstrumentId.from_str(instrument_id.value)

            # Create bar type using Rust (1-MINUTE bars from Alpaca stream)
            bar_type = create_bar_type(pyo3_instrument_id, "1Min")

            # Parse bar using Rust (returns PyO3 Bar)
            pyo3_bar = parse_bar(
                msg,
                bar_type,
                instrument.price_precision,
                instrument.size_precision,
                self._clock.timestamp_ns(),
            )

            # Convert PyO3 type to Python type
            bar = Bar.from_pyo3(pyo3_bar)
            self._handle_data(bar)

        except Exception as e:
            self._log.error(f"Error parsing bar: {e}")

    def _handle_orderbook(self, msg: dict) -> None:
        """Handle orderbook message."""
        try:
            symbol = msg["S"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")

            # Extract orderbook data
            bids = msg.get("b", [])
            asks = msg.get("a", [])
            reset = msg.get("r", False)
            timestamp = msg.get("t", "")

            # For now, just log the orderbook data
            # TODO: Convert to NautilusTrader OrderBookDeltas/OrderBookDepth10
            self._log.info(
                f"📖 Orderbook {symbol} ({'RESET' if reset else 'UPDATE'}): "
                f"{len(bids)} bids, {len(asks)} asks @ {timestamp}"
            )

            if bids:
                best_bid = bids[0]
                self._log.debug(f"  Best bid: ${best_bid['p']} x {best_bid['s']}")
            if asks:
                best_ask = asks[0]
                self._log.debug(f"  Best ask: ${best_ask['p']} x {best_ask['s']}")

        except Exception as e:
            self._log.error(f"Error handling orderbook: {e}")

    async def _subscribe_trade_ticks(self, command) -> None:
        """
        Subscribe to trade ticks for an instrument.

        Parameters
        ----------
        command : SubscribeTradeTicks
            The subscribe trade ticks command.

        """
        if not self._ws_client:
            self._log.error("WebSocket client not connected")
            return

        instrument_id = command.instrument_id
        symbol = instrument_id.symbol.value
        normalized_symbol = self._normalize_symbol(symbol)
        self._trade_subscriptions.add(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_trades_message([normalized_symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Subscribed to trade ticks: {instrument_id}")

    async def _subscribe_quote_ticks(self, command) -> None:
        """
        Subscribe to quote ticks for an instrument.

        Parameters
        ----------
        command : SubscribeQuoteTicks
            The subscribe quote ticks command.

        """
        if not self._ws_client:
            self._log.error("WebSocket client not connected")
            return

        instrument_id = command.instrument_id
        symbol = instrument_id.symbol.value
        normalized_symbol = self._normalize_symbol(symbol)
        self._quote_subscriptions.add(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_quotes_message([normalized_symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Subscribed to quote ticks: {instrument_id}")

    def _normalize_symbol(self, symbol: str) -> str:
        """
        Normalize symbol for Alpaca API.

        For crypto: Keep the slash (e.g., BTC/USD stays as BTC/USD)
        For equities: Remove any slashes (though equities typically don't have slashes)
        """
        # Crypto symbols should keep the slash for the crypto WebSocket endpoint
        # Only equities would need normalization, but they don't have slashes anyway
        return symbol

    async def _subscribe_bars(self, command) -> None:
        """
        Subscribe to bars for an instrument.

        Parameters
        ----------
        command : SubscribeBars
            The subscribe bars command.

        """
        if not self._ws_client:
            self._log.error("WebSocket client not connected")
            return

        bar_type = command.bar_type
        symbol = bar_type.instrument_id.symbol.value
        normalized_symbol = self._normalize_symbol(symbol)
        self._bar_subscriptions.add(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_bars_message([normalized_symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Subscribed to bars: {bar_type}")

    async def _subscribe_order_book_deltas(self, command) -> None:
        """
        Subscribe to order book deltas for an instrument.

        Parameters
        ----------
        command : SubscribeOrderBookDeltas
            The subscribe order book deltas command.

        """
        if not self._ws_client:
            self._log.error("WebSocket client not connected")
            return

        instrument_id = command.instrument_id
        symbol = instrument_id.symbol.value
        normalized_symbol = self._normalize_symbol(symbol)
        self._orderbook_subscriptions.add(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.subscribe_orderbooks_message([normalized_symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Subscribed to orderbook: {instrument_id}")

    async def _unsubscribe_trade_ticks(self, command) -> None:
        """Unsubscribe from trade ticks."""
        if not self._ws_client:
            return

        instrument_id = command.instrument_id
        symbol = instrument_id.symbol.value
        self._trade_subscriptions.discard(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.unsubscribe_trades_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Unsubscribed from trade ticks: {instrument_id}")

    async def _unsubscribe_quote_ticks(self, command) -> None:
        """Unsubscribe from quote ticks."""
        if not self._ws_client:
            return

        instrument_id = command.instrument_id
        symbol = instrument_id.symbol.value
        self._quote_subscriptions.discard(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.unsubscribe_quotes_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Unsubscribed from quote ticks: {instrument_id}")

    async def _unsubscribe_bars(self, command) -> None:
        """Unsubscribe from bars."""
        if not self._ws_client:
            return

        bar_type = command.bar_type
        symbol = bar_type.instrument_id.symbol.value
        self._bar_subscriptions.discard(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.unsubscribe_bars_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Unsubscribed from bars: {bar_type}")

    async def _unsubscribe_order_book_deltas(self, command) -> None:
        """Unsubscribe from order book deltas."""
        if not self._ws_client:
            return

        instrument_id = command.instrument_id
        symbol = instrument_id.symbol.value
        self._orderbook_subscriptions.discard(symbol)

        msg = nautilus_pyo3.AlpacaWebSocketClient.unsubscribe_orderbooks_message([symbol])
        await self._ws_client.send_text(msg.encode("utf-8"))
        self._log.info(f"Unsubscribed from orderbook: {instrument_id}")

    async def _request_instrument(self, request: RequestInstrument) -> None:
        """
        Request instrument data.

        Parameters
        ----------
        request : RequestInstrument
            The request for instrument data.

        """
        if request.start is not None:
            self._log.warning(
                f"Requesting instrument {request.instrument_id} with specified `start` which has no effect",
            )

        if request.end is not None:
            self._log.warning(
                f"Requesting instrument {request.instrument_id} with specified `end` which has no effect",
            )

        instrument: Instrument | None = self._instrument_provider.find(request.instrument_id)
        if instrument is None:
            self._log.error(f"Cannot find instrument for {request.instrument_id}")
            return

        self._handle_instrument(instrument, request.id, request.start, request.end, request.params)

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
