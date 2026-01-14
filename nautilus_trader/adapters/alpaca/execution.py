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
Alpaca execution client implementation.

Provides order submission and position management for:
- US Equities (stocks and ETFs)
- Cryptocurrencies (BTC/USD, ETH/USD, etc.) with GTC/IOC constraints
- Options (equity options) with DAY order constraint
"""

import asyncio

from nautilus_trader.adapters.alpaca.providers import AlpacaInstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.common.enums import LogColor
from nautilus_trader.core import nautilus_pyo3
from nautilus_trader.core.datetime import millis_to_nanos
from nautilus_trader.execution.messages import CancelAllOrders
from nautilus_trader.execution.messages import CancelOrder
from nautilus_trader.execution.messages import ModifyOrder
from nautilus_trader.execution.messages import QueryOrder
from nautilus_trader.execution.messages import SubmitOrder
from nautilus_trader.execution.reports import FillReport
from nautilus_trader.execution.reports import OrderStatusReport
from nautilus_trader.execution.reports import PositionStatusReport
from nautilus_trader.live.execution_client import LiveExecutionClient
from nautilus_trader.model.enums import AccountType
from nautilus_trader.model.enums import LiquiditySide
from nautilus_trader.model.enums import OmsType
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.enums import OrderStatus
from nautilus_trader.model.enums import OrderType
from nautilus_trader.model.enums import TimeInForce
from nautilus_trader.model.identifiers import AccountId
from nautilus_trader.model.identifiers import ClientId
from nautilus_trader.model.identifiers import ClientOrderId
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import TradeId
from nautilus_trader.model.identifiers import Venue
from nautilus_trader.model.identifiers import VenueOrderId
from nautilus_trader.model.objects import AccountBalance
from nautilus_trader.model.objects import Currency
from nautilus_trader.model.objects import MarginBalance
from nautilus_trader.model.objects import Money
from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity


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
        instrument_provider: AlpacaInstrumentProvider,
        config,  # AlpacaExecClientConfig
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or "ALPACA"),
            venue=Venue(str(config.venue)) if hasattr(config, 'venue') else Venue(nautilus_pyo3.ALPACA_VENUE),
            oms_type=OmsType.NETTING,
            account_type=AccountType.MARGIN if not config.paper_trading else AccountType.CASH,
            base_currency=Currency.from_str("USD"),
            instrument_provider=instrument_provider,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
        )

        self._config = config
        self._http_client = client

        # Set account ID
        account_id = AccountId(f"{name or 'ALPACA'}-master")
        self._set_account_id(account_id)

        # Track order mappings
        self._client_order_id_to_venue_order_id: dict[ClientOrderId, VenueOrderId] = {}
        self._venue_order_id_to_client_order_id: dict[VenueOrderId, ClientOrderId] = {}

        # Log configuration
        self._log.info(f"paper_trading={config.paper_trading}", LogColor.BLUE)
        self._log.info(f"http_timeout_secs={config.http_timeout_secs}", LogColor.BLUE)

    async def _connect(self) -> None:
        """
        Connect to Alpaca execution services.
        """
        self._log.info("Connecting to Alpaca execution services...")

        # Initialize instrument provider
        await self._instrument_provider.initialize()

        # Fetch account information
        try:
            account_info = await self._http_client.get_account()
            self._log.info(f"Account fetched: {account_info['account_number']}", LogColor.GREEN)

            # Generate account state
            await self._generate_account_state(account_info)

            # Fetch open orders and positions
            await self._generate_order_status_reports()
            await self._generate_position_status_reports()

        except Exception as e:
            self._log.error(f"Error connecting to Alpaca: {e}")
            raise

        self._log.info("Connected to Alpaca execution services", LogColor.GREEN)

    async def _disconnect(self) -> None:
        """
        Disconnect from Alpaca execution services.
        """
        self._log.info("Disconnecting from Alpaca execution services...")

        # Clear order mappings
        self._client_order_id_to_venue_order_id.clear()
        self._venue_order_id_to_client_order_id.clear()

        self._log.info("Disconnected from Alpaca execution services", LogColor.GREEN)

    async def _generate_account_state(self, account_info: dict) -> None:
        """
        Generate account state from Alpaca account info.

        Parameters
        ----------
        account_info : dict
            Account information from Alpaca API.

        """
        # Parse account balances
        equity = float(account_info.get("equity", "0"))
        cash = float(account_info.get("cash", "0"))
        buying_power = float(account_info.get("buying_power", "0"))
        initial_margin = float(account_info.get("initial_margin", "0"))
        maintenance_margin = float(account_info.get("maintenance_margin", "0"))

        usd = Currency.from_str("USD")

        # Create account balances
        balances = [
            AccountBalance(
                total=Money(equity, usd),
                locked=Money(0, usd),
                free=Money(cash, usd),
            ),
        ]

        # Create margin balance
        margins = [
            MarginBalance(
                initial=Money(initial_margin, usd),
                maintenance=Money(maintenance_margin, usd),
                currency=usd,
            ),
        ]

        # Generate account state event
        self.generate_account_state(
            balances=balances,
            margins=margins,
            reported=True,
            ts_event=self._clock.timestamp_ns(),
        )

    async def _generate_order_status_reports(self) -> None:
        """Generate order status reports for all open orders."""
        try:
            orders = await self._http_client.get_orders(status="open")

            for order_data in orders:
                report = self._parse_order_status_report(order_data)
                if report:
                    self._send_order_status_report(report)

        except Exception as e:
            self._log.error(f"Error generating order status reports: {e}")

    async def _generate_position_status_reports(self) -> None:
        """Generate position status reports for all open positions."""
        try:
            positions = await self._http_client.get_positions()

            for position_data in positions:
                report = self._parse_position_status_report(position_data)
                if report:
                    self._send_position_status_report(report)

        except Exception as e:
            self._log.error(f"Error generating position status reports: {e}")

    def _parse_order_status_report(self, order_data: dict) -> OrderStatusReport | None:
        """Parse order data into OrderStatusReport."""
        try:
            symbol = order_data["symbol"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")

            # Parse order status
            status_map = {
                "new": OrderStatus.ACCEPTED,
                "accepted": OrderStatus.ACCEPTED,
                "partially_filled": OrderStatus.PARTIALLY_FILLED,
                "filled": OrderStatus.FILLED,
                "done_for_day": OrderStatus.CANCELED,
                "canceled": OrderStatus.CANCELED,
                "expired": OrderStatus.EXPIRED,
                "replaced": OrderStatus.CANCELED,
                "pending_cancel": OrderStatus.PENDING_CANCEL,
                "pending_replace": OrderStatus.PENDING_UPDATE,
                "rejected": OrderStatus.REJECTED,
                "suspended": OrderStatus.REJECTED,
                "pending_new": OrderStatus.SUBMITTED,
            }

            venue_order_id = VenueOrderId(order_data["id"])
            client_order_id = ClientOrderId(order_data.get("client_order_id", order_data["id"]))

            # Track order mapping
            self._client_order_id_to_venue_order_id[client_order_id] = venue_order_id
            self._venue_order_id_to_client_order_id[venue_order_id] = client_order_id

            return OrderStatusReport(
                account_id=self.account_id,
                instrument_id=instrument_id,
                client_order_id=client_order_id,
                venue_order_id=venue_order_id,
                order_side=OrderSide.BUY if order_data["side"] == "buy" else OrderSide.SELL,
                order_type=self._parse_order_type(order_data["type"]),
                time_in_force=self._parse_time_in_force(order_data["time_in_force"]),
                order_status=status_map.get(order_data["status"], OrderStatus.ACCEPTED),
                quantity=Quantity.from_str(order_data["qty"]),
                filled_qty=Quantity.from_str(order_data.get("filled_qty", "0")),
                price=Price.from_str(order_data["limit_price"]) if order_data.get("limit_price") else None,
                report_id=nautilus_pyo3.UUID4(),
                ts_accepted=millis_to_nanos(self._parse_timestamp_ms(order_data["created_at"])),
                ts_last=millis_to_nanos(self._parse_timestamp_ms(order_data["updated_at"])),
                ts_init=self._clock.timestamp_ns(),
            )

        except Exception as e:
            self._log.error(f"Error parsing order status report: {e}")
            return None

    def _parse_position_status_report(self, position_data: dict) -> PositionStatusReport | None:
        """Parse position data into PositionStatusReport."""
        try:
            symbol = position_data["symbol"]
            instrument_id = InstrumentId.from_str(f"{symbol}.{self.venue}")

            qty = float(position_data["qty"])
            side = OrderSide.BUY if qty > 0 else OrderSide.SELL
            abs_qty = abs(qty)

            return PositionStatusReport(
                account_id=self.account_id,
                instrument_id=instrument_id,
                position_side=self._position_side_from_qty(qty),
                quantity=Quantity.from_str(str(abs_qty)),
                report_id=nautilus_pyo3.UUID4(),
                ts_last=self._clock.timestamp_ns(),
                ts_init=self._clock.timestamp_ns(),
            )

        except Exception as e:
            self._log.error(f"Error parsing position status report: {e}")
            return None

    def _position_side_from_qty(self, qty: float):
        """Determine position side from quantity."""
        from nautilus_trader.model.enums import PositionSide

        if qty > 0:
            return PositionSide.LONG
        elif qty < 0:
            return PositionSide.SHORT
        return PositionSide.FLAT

    def _parse_order_type(self, order_type_str: str) -> OrderType:
        """Parse Alpaca order type to Nautilus OrderType."""
        type_map = {
            "market": OrderType.MARKET,
            "limit": OrderType.LIMIT,
            "stop": OrderType.STOP_MARKET,
            "stop_limit": OrderType.STOP_LIMIT,
            "trailing_stop": OrderType.TRAILING_STOP_MARKET,
        }
        return type_map.get(order_type_str.lower(), OrderType.MARKET)

    def _parse_time_in_force(self, tif_str: str) -> TimeInForce:
        """Parse Alpaca time-in-force to Nautilus TimeInForce."""
        tif_map = {
            "day": TimeInForce.DAY,
            "gtc": TimeInForce.GTC,
            "opg": TimeInForce.AT_THE_OPEN,
            "cls": TimeInForce.AT_THE_CLOSE,
            "ioc": TimeInForce.IOC,
            "fok": TimeInForce.FOK,
        }
        return tif_map.get(tif_str.lower(), TimeInForce.GTC)

    def _parse_timestamp_ms(self, ts_str: str) -> int:
        """Parse RFC3339 timestamp to milliseconds."""
        import datetime

        dt = datetime.datetime.fromisoformat(ts_str.rstrip("Z"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=datetime.UTC)
        return int(dt.timestamp() * 1000)

    async def _submit_order(self, command: SubmitOrder) -> None:
        """
        Submit an order to Alpaca.

        Parameters
        ----------
        command : SubmitOrder
            The submit order command.

        """
        order = command.order

        if order.is_closed:
            self._log.warning(f"Cannot submit already closed order: {order}")
            return

        try:
            # Generate OrderSubmitted event
            self.generate_order_submitted(
                strategy_id=order.strategy_id,
                instrument_id=order.instrument_id,
                client_order_id=order.client_order_id,
                ts_event=self._clock.timestamp_ns(),
            )

            # Build order request
            symbol = order.instrument_id.symbol.value

            # Submit order via HTTP
            response = await self._http_client.submit_order(
                symbol=symbol,
                side="buy" if order.side == OrderSide.BUY else "sell",
                order_type=self._order_type_to_alpaca(order.order_type),
                time_in_force=self._time_in_force_to_alpaca(order.time_in_force),
                qty=str(order.quantity),
                limit_price=str(order.price) if order.has_price else None,
                stop_price=str(order.trigger_price) if order.has_trigger_price else None,
                client_order_id=order.client_order_id.value,
            )

            # Parse response and generate OrderAccepted event
            venue_order_id = VenueOrderId(response["id"])

            # Track order mapping
            self._client_order_id_to_venue_order_id[order.client_order_id] = venue_order_id
            self._venue_order_id_to_client_order_id[venue_order_id] = order.client_order_id

            self.generate_order_accepted(
                strategy_id=order.strategy_id,
                instrument_id=order.instrument_id,
                client_order_id=order.client_order_id,
                venue_order_id=venue_order_id,
                ts_event=self._clock.timestamp_ns(),
            )

        except Exception as e:
            self._log.error(f"Error submitting order: {e}")
            self.generate_order_rejected(
                strategy_id=order.strategy_id,
                instrument_id=order.instrument_id,
                client_order_id=order.client_order_id,
                reason=str(e),
                ts_event=self._clock.timestamp_ns(),
            )

    async def _modify_order(self, command: ModifyOrder) -> None:
        """
        Modify an order on Alpaca.

        Parameters
        ----------
        command : ModifyOrder
            The modify order command.

        """
        try:
            # Get venue order ID
            venue_order_id = self._client_order_id_to_venue_order_id.get(command.client_order_id)

            if not venue_order_id:
                self._log.error(f"No venue order ID found for {command.client_order_id}")
                self.generate_order_modify_rejected(
                    strategy_id=command.strategy_id,
                    instrument_id=command.instrument_id,
                    client_order_id=command.client_order_id,
                    venue_order_id=None,
                    reason="Order not found",
                    ts_event=self._clock.timestamp_ns(),
                )
                return

            # Modify order via HTTP
            response = await self._http_client.modify_order(
                order_id=venue_order_id.value,
                qty=str(command.quantity) if command.quantity else None,
                limit_price=str(command.price) if command.price else None,
            )

            # Generate OrderUpdated event
            self.generate_order_updated(
                strategy_id=command.strategy_id,
                instrument_id=command.instrument_id,
                client_order_id=command.client_order_id,
                venue_order_id=venue_order_id,
                quantity=command.quantity,
                price=command.price,
                trigger_price=command.trigger_price,
                ts_event=self._clock.timestamp_ns(),
            )

        except Exception as e:
            self._log.error(f"Error modifying order: {e}")
            self.generate_order_modify_rejected(
                strategy_id=command.strategy_id,
                instrument_id=command.instrument_id,
                client_order_id=command.client_order_id,
                venue_order_id=venue_order_id if venue_order_id else None,
                reason=str(e),
                ts_event=self._clock.timestamp_ns(),
            )

    async def _cancel_order(self, command: CancelOrder) -> None:
        """
        Cancel an order on Alpaca.

        Parameters
        ----------
        command : CancelOrder
            The cancel order command.

        """
        try:
            # Get venue order ID
            venue_order_id = self._client_order_id_to_venue_order_id.get(command.client_order_id)

            if not venue_order_id:
                self._log.error(f"No venue order ID found for {command.client_order_id}")
                self.generate_order_cancel_rejected(
                    strategy_id=command.strategy_id,
                    instrument_id=command.instrument_id,
                    client_order_id=command.client_order_id,
                    venue_order_id=None,
                    reason="Order not found",
                    ts_event=self._clock.timestamp_ns(),
                )
                return

            # Cancel order via HTTP
            await self._http_client.cancel_order(order_id=venue_order_id.value)

            # Generate OrderCanceled event
            self.generate_order_canceled(
                strategy_id=command.strategy_id,
                instrument_id=command.instrument_id,
                client_order_id=command.client_order_id,
                venue_order_id=venue_order_id,
                ts_event=self._clock.timestamp_ns(),
            )

        except Exception as e:
            self._log.error(f"Error canceling order: {e}")
            self.generate_order_cancel_rejected(
                strategy_id=command.strategy_id,
                instrument_id=command.instrument_id,
                client_order_id=command.client_order_id,
                venue_order_id=venue_order_id if venue_order_id else None,
                reason=str(e),
                ts_event=self._clock.timestamp_ns(),
            )

    async def _cancel_all_orders(self, command: CancelAllOrders) -> None:
        """
        Cancel all orders on Alpaca.

        Parameters
        ----------
        command : CancelAllOrders
            The cancel all orders command.

        """
        try:
            await self._http_client.cancel_all_orders()
            self._log.info("All orders canceled")

        except Exception as e:
            self._log.error(f"Error canceling all orders: {e}")

    def _order_type_to_alpaca(self, order_type: OrderType) -> str:
        """Convert Nautilus OrderType to Alpaca order type."""
        type_map = {
            OrderType.MARKET: "market",
            OrderType.LIMIT: "limit",
            OrderType.STOP_MARKET: "stop",
            OrderType.STOP_LIMIT: "stop_limit",
            OrderType.TRAILING_STOP_MARKET: "trailing_stop",
        }
        return type_map.get(order_type, "market")

    def _time_in_force_to_alpaca(self, tif: TimeInForce) -> str:
        """Convert Nautilus TimeInForce to Alpaca time-in-force."""
        tif_map = {
            TimeInForce.DAY: "day",
            TimeInForce.GTC: "gtc",
            TimeInForce.AT_THE_OPEN: "opg",
            TimeInForce.AT_THE_CLOSE: "cls",
            TimeInForce.IOC: "ioc",
            TimeInForce.FOK: "fok",
        }
        return tif_map.get(tif, "day")
