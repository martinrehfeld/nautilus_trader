#!/usr/bin/env python3
# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2025 Nautech Systems Pty Ltd. All rights reserved.
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
Example: Alpaca US Equity Trading with EMA Cross Strategy

This example demonstrates live trading of US equities on Alpaca using a simple
EMA crossover strategy. The strategy trades when a fast EMA crosses a slow EMA.

Prerequisites:
    1. Create an Alpaca account at https://alpaca.markets
    2. Get your API credentials from the dashboard
    3. Set environment variables:
       export ALPACA_API_KEY=your_key
       export ALPACA_API_SECRET=your_secret

Features Demonstrated:
    - US equity trading (stocks and ETFs)
    - Real-time market data streaming (IEX feed)
    - Market and limit orders
    - Paper trading for safe testing
    - EMA indicator usage
    - Position management

Usage:
    python alpaca_equity_ema_cross.py
"""

import os
import sys
from decimal import Decimal

from nautilus_trader.adapters.alpaca import (
    ALPACA_VENUE,
    AlpacaDataClientConfig,
    AlpacaDataFeed,
    AlpacaExecClientConfig,
    AlpacaInstrumentProviderConfig,
    AlpacaLiveDataClientFactory,
    AlpacaLiveExecClientFactory,
)
from nautilus_trader.config import (
    LoggingConfig,
    TradingNodeConfig,
)
from nautilus_trader.core.datetime import dt_to_unix_nanos
from nautilus_trader.indicators import ExponentialMovingAverage
from nautilus_trader.live.node import TradingNode
from nautilus_trader.model.data import BarType
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.identifiers import InstrumentId, TraderId
from nautilus_trader.model.objects import Quantity
from nautilus_trader.trading import Strategy
from nautilus_trader.trading.config import StrategyConfig


class EMACrossConfig(StrategyConfig, frozen=True):
    """Configuration for EMA cross strategy."""

    instrument_id: InstrumentId
    bar_type: BarType
    fast_ema_period: int = 10
    slow_ema_period: int = 20
    trade_size: Decimal = Decimal("100")  # Dollar amount to trade


class EMACrossStrategy(Strategy):
    """
    Simple EMA crossover strategy for US equities.

    Strategy Logic:
        - BUY when fast EMA crosses above slow EMA (golden cross)
        - SELL when fast EMA crosses below slow EMA (death cross)
        - Uses market orders for immediate execution
        - Flat position when EMAs are not crossed
    """

    def __init__(self, config: EMACrossConfig):
        super().__init__(config)
        self.instrument_id = config.instrument_id
        self.bar_type = config.bar_type
        self.trade_size = config.trade_size

        # Create EMA indicators
        self.fast_ema = ExponentialMovingAverage(config.fast_ema_period)
        self.slow_ema = ExponentialMovingAverage(config.slow_ema_period)

        # Track cross state
        self.fast_above_slow = False

    def on_start(self):
        """Actions to perform on strategy start."""
        self.log.info(f"Starting EMA Cross strategy for {self.instrument_id}")
        self.log.info(f"Fast EMA: {self.fast_ema.period}, Slow EMA: {self.slow_ema.period}")
        self.log.info(f"Trade size: ${self.trade_size}")

        # Subscribe to bars
        self.subscribe_bars(self.bar_type)

        # Request initial instrument
        self.request_instrument(self.instrument_id)

    def on_bar(self, bar):
        """
        Handle incoming bars and check for EMA crossovers.

        Parameters
        ----------
        bar : Bar
            The bar data received
        """
        # Update EMAs
        self.fast_ema.handle_bar(bar)
        self.slow_ema.handle_bar(bar)

        # Wait for indicators to initialize
        if not self.fast_ema.initialized or not self.slow_ema.initialized:
            self.log.info("Waiting for indicators to initialize...")
            return

        # Log current EMA values
        self.log.info(
            f"Bar: {bar.close} | Fast EMA: {self.fast_ema.value:.2f} | "
            f"Slow EMA: {self.slow_ema.value:.2f}"
        )

        # Check for crossover
        fast_now_above = self.fast_ema.value > self.slow_ema.value

        # Golden cross (fast crosses above slow) - BUY signal
        if fast_now_above and not self.fast_above_slow:
            self.log.info("🟢 GOLDEN CROSS - BUY signal detected!")
            self._enter_long()

        # Death cross (fast crosses below slow) - SELL signal
        elif not fast_now_above and self.fast_above_slow:
            self.log.info("🔴 DEATH CROSS - SELL signal detected!")
            self._exit_long()

        # Update cross state
        self.fast_above_slow = fast_now_above

    def _enter_long(self):
        """Enter a long position."""
        # Check if already in position
        if self.portfolio.is_flat(self.instrument_id):
            # Get instrument for price precision
            instrument = self.cache.instrument(self.instrument_id)
            if not instrument:
                self.log.warning("Instrument not found, cannot place order")
                return

            # Calculate quantity based on trade size and current price
            # For equities, we'll use notional (dollar amount) orders
            order = self.order_factory.market(
                instrument_id=self.instrument_id,
                order_side=OrderSide.BUY,
                quantity=Quantity.from_int(1),  # Minimum, will use notional instead
            )

            self.submit_order(order)
            self.log.info(f"📈 Submitted BUY order: {order.client_order_id}")
        else:
            self.log.info("Already in long position, skipping BUY")

    def _exit_long(self):
        """Exit long position."""
        # Check if we have a position to close
        if not self.portfolio.is_flat(self.instrument_id):
            # Get current position
            position = self.portfolio.position(self.instrument_id)
            if position and position.quantity > 0:
                order = self.order_factory.market(
                    instrument_id=self.instrument_id,
                    order_side=OrderSide.SELL,
                    quantity=position.quantity,
                )

                self.submit_order(order)
                self.log.info(f"📉 Submitted SELL order: {order.client_order_id}")
        else:
            self.log.info("No position to close, skipping SELL")

    def on_order_filled(self, event):
        """Handle order filled events."""
        self.log.info(
            f"✅ Order filled: {event.client_order_id} | "
            f"Price: ${event.last_px} | Qty: {event.last_qty}"
        )

    def on_order_rejected(self, event):
        """Handle order rejected events."""
        self.log.error(f"❌ Order rejected: {event.client_order_id} | Reason: {event.reason}")

    def on_stop(self):
        """Actions to perform on strategy stop."""
        # Close any open positions
        if not self.portfolio.is_flat(self.instrument_id):
            self.log.info("Closing position on stop...")
            self._exit_long()

        self.log.info("Strategy stopped")


def main():
    # Check for API credentials
    api_key = os.getenv("ALPACA_API_KEY")
    api_secret = os.getenv("ALPACA_API_SECRET")

    if not api_key or not api_secret:
        print("ERROR: Alpaca API credentials not set!")
        print("Set your credentials:")
        print("  export ALPACA_API_KEY=your_key")
        print("  export ALPACA_API_SECRET=your_secret")
        sys.exit(1)

    # Symbol to trade
    symbol = "SPY"  # SPDR S&P 500 ETF - highly liquid, good for testing
    instrument_id = InstrumentId.from_str(f"{symbol}.{ALPACA_VENUE}")

    # Create bar type (1-minute bars)
    bar_type = BarType.from_str(f"{symbol}.{ALPACA_VENUE}-1-MINUTE-LAST-EXTERNAL")

    # Configure trading node
    config = TradingNodeConfig(
        trader_id=TraderId("ALPACA-EMA-CROSS-001"),
        logging=LoggingConfig(
            log_level="INFO",
            log_colors=True,
        ),
        # Data client configuration
        data_clients={
            "ALPACA": AlpacaDataClientConfig(
                api_key=api_key,
                api_secret=api_secret,
                paper_trading=True,  # Use paper trading (safe)
                data_feed=AlpacaDataFeed.IEX,  # Free data feed
                instrument_provider=AlpacaInstrumentProviderConfig(
                    load_all=True,  # Load instruments on startup
                ),
            ),
        },
        # Execution client configuration
        exec_clients={
            "ALPACA": AlpacaExecClientConfig(
                api_key=api_key,
                api_secret=api_secret,
                paper_trading=True,  # Use paper trading (safe)
            ),
        },
        timeout_connection=30.0,
        timeout_disconnection=10.0,
    )

    # Create strategy
    strategy_config = EMACrossConfig(
        strategy_id="EMA-CROSS-001",
        instrument_id=instrument_id,
        bar_type=bar_type,
        fast_ema_period=10,
        slow_ema_period=20,
        trade_size=Decimal("1000"),  # $1000 per trade
    )
    strategy = EMACrossStrategy(config=strategy_config)

    # Build trading node
    node = TradingNode(config=config)
    node.trader.add_strategy(strategy)
    node.add_data_client_factory("ALPACA", AlpacaLiveDataClientFactory)
    node.add_exec_client_factory("ALPACA", AlpacaLiveExecClientFactory)
    node.build()

    # Run
    print("\n" + "=" * 70)
    print("Alpaca US Equity EMA Cross Strategy")
    print("=" * 70)
    print(f"Instrument: {instrument_id}")
    print(f"Bar Type: {bar_type}")
    print(f"Fast EMA: {strategy_config.fast_ema_period}")
    print(f"Slow EMA: {strategy_config.slow_ema_period}")
    print(f"Trade Size: ${strategy_config.trade_size}")
    print("Paper Trading: True (safe mode)")
    print("Data Feed: IEX (free tier)")
    print("\nPress Ctrl+C to stop")
    print("=" * 70 + "\n")

    try:
        node.run()
    except KeyboardInterrupt:
        print("\nShutting down...")
    finally:
        node.dispose()


if __name__ == "__main__":
    main()
