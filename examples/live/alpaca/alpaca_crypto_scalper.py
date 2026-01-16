#!/usr/bin/env python3
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
Example: Alpaca Cryptocurrency Scalping Strategy

This example demonstrates cryptocurrency trading on Alpaca using a simple
scalping strategy based on price momentum and volatility.

Prerequisites:
    1. Create an Alpaca account at https://alpaca.markets
    2. Get your API credentials from the dashboard
    3. Set environment variables:
       export ALPACA_API_KEY=your_key
       export ALPACA_API_SECRET=your_secret

Features Demonstrated:
    - Cryptocurrency trading (BTC/USD, ETH/USD, etc.)
    - Real-time crypto market data streaming
    - GTC (Good-Till-Cancelled) and IOC (Immediate-Or-Cancel) orders
    - Limit orders for precise entry/exit
    - Crypto-specific constraints (no shorting, 24/7 trading)
    - Position sizing and risk management

Supported Crypto Pairs:
    - BTC/USD (Bitcoin)
    - ETH/USD (Ethereum)
    - DOGE/USD (Dogecoin)
    - And other major pairs

Usage:
    python alpaca_crypto_scalper.py
"""

import os
import sys
from decimal import Decimal

from nautilus_trader.adapters.alpaca import ALPACA_VENUE
from nautilus_trader.adapters.alpaca import AlpacaAssetClass
from nautilus_trader.adapters.alpaca import AlpacaDataClientConfig
from nautilus_trader.adapters.alpaca import AlpacaDataFeed
from nautilus_trader.adapters.alpaca import AlpacaExecClientConfig
from nautilus_trader.adapters.alpaca import AlpacaInstrumentProviderConfig
from nautilus_trader.adapters.alpaca import AlpacaLiveDataClientFactory
from nautilus_trader.adapters.alpaca import AlpacaLiveExecClientFactory
from nautilus_trader.config import LoggingConfig
from nautilus_trader.config import TradingNodeConfig
from nautilus_trader.indicators import AverageTrueRange
from nautilus_trader.live.node import TradingNode
from nautilus_trader.model.data import BarType
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.enums import TimeInForce
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import TraderId
from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity
from nautilus_trader.trading import Strategy
from nautilus_trader.trading.config import StrategyConfig


class CryptoScalperConfig(StrategyConfig, frozen=True):
    """
    Configuration for crypto scalper strategy.
    """

    instrument_id: InstrumentId
    bar_type: BarType
    atr_period: int = 14
    volatility_threshold: float = 0.02  # 2% volatility threshold
    profit_target_atr_multiple: float = 2.0  # Take profit at 2x ATR
    stop_loss_atr_multiple: float = 1.0  # Stop loss at 1x ATR
    position_size_usd: Decimal = Decimal(100)  # Position size in USD


class CryptoScalperStrategy(Strategy):
    """
    Cryptocurrency scalping strategy using volatility breakouts.

    Strategy Logic:
        - Enter long when price breaks above recent high with sufficient volatility
        - Take profit at 2x ATR above entry
        - Stop loss at 1x ATR below entry
        - Uses limit orders for better fills
        - Flat position when no setup

    """

    def __init__(self, config: CryptoScalperConfig):
        super().__init__(config)
        self.instrument_id = config.instrument_id
        self.bar_type = config.bar_type
        self.position_size_usd = config.position_size_usd
        self.volatility_threshold = config.volatility_threshold
        self.profit_target_multiple = config.profit_target_atr_multiple
        self.stop_loss_multiple = config.stop_loss_atr_multiple

        # Create ATR indicator for volatility measurement
        self.atr = AverageTrueRange(config.atr_period)

        # Track state
        self.entry_price = None
        self.last_high = None

    def on_start(self):
        """
        Actions to perform on strategy start.
        """
        self.log.info(f"Starting Crypto Scalper for {self.instrument_id}")
        self.log.info(f"ATR Period: {self.atr.period}")
        self.log.info(f"Position Size: ${self.position_size_usd}")
        self.log.info(f"Volatility Threshold: {self.volatility_threshold * 100}%")

        # Subscribe to bars
        self.subscribe_bars(self.bar_type)

        # Request initial instrument
        self.request_instrument(self.instrument_id)

    def on_bar(self, bar):
        """
        Handle incoming bars and check for entry/exit signals.

        Parameters
        ----------
        bar : Bar
            The bar data received

        """
        # Update ATR
        self.atr.handle_bar(bar)

        # Wait for ATR to initialize
        if not self.atr.initialized:
            self.log.info("Waiting for ATR to initialize...")
            return

        current_price = bar.close
        atr_value = self.atr.value

        # Calculate volatility as percentage of price
        volatility_pct = float(atr_value) / float(current_price)

        self.log.info(
            f"Bar: ${current_price} | ATR: ${atr_value:.2f} | Volatility: {volatility_pct:.2%}",
        )

        # Update last high
        if self.last_high is None or bar.high > self.last_high:
            self.last_high = bar.high

        # Check if we have a position
        position = self.portfolio.position(self.instrument_id)
        is_flat = self.portfolio.is_flat(self.instrument_id)

        if is_flat:
            # Look for entry signal
            self._check_entry_signal(bar, volatility_pct)
        else:
            # Manage existing position
            self._manage_position(bar, position)

    def _check_entry_signal(self, bar, volatility_pct):
        """
        Check for entry signals.
        """
        # Entry conditions:
        # 1. Volatility above threshold
        # 2. Price breaks above recent high
        if (
            volatility_pct >= self.volatility_threshold
            and self.last_high
            and bar.close > self.last_high
        ):
            self.log.info("🚀 BREAKOUT SIGNAL - Entering long position!")
            self._enter_long(bar)

    def _enter_long(self, bar):
        """
        Enter a long position with limit order.
        """
        instrument = self.cache.instrument(self.instrument_id)
        if not instrument:
            self.log.warning("Instrument not found, cannot place order")
            return

        # Calculate quantity based on position size
        entry_price = bar.close
        quantity = self.position_size_usd / entry_price
        quantity = Quantity.from_str(str(quantity))

        # Place limit buy order slightly above current price for quick fill
        # For crypto, we use GTC (Good-Till-Cancelled)
        limit_price = Price(entry_price * Decimal("1.001"), instrument.price_precision)

        order = self.order_factory.limit(
            instrument_id=self.instrument_id,
            order_side=OrderSide.BUY,
            quantity=quantity,
            price=limit_price,
            time_in_force=TimeInForce.GTC,  # GTC for crypto
        )

        self.submit_order(order)
        self.entry_price = entry_price
        self.log.info(
            f"📈 Submitted BUY limit order: {order.client_order_id} | "
            f"Price: ${limit_price} | Qty: {quantity}",
        )

    def _manage_position(self, bar, position):
        """Manage existing position - check for exit signals."""
        if not position or not self.entry_price:
            return

        current_price = bar.close
        atr_value = self.atr.value

        # Calculate profit target and stop loss
        profit_target = self.entry_price + (atr_value * self.profit_target_multiple)
        stop_loss = self.entry_price - (atr_value * self.stop_loss_multiple)

        # Check for exit conditions
        if current_price >= profit_target:
            self.log.info(f"✅ PROFIT TARGET HIT at ${current_price}")
            self._exit_position(position, "profit_target")
        elif current_price <= stop_loss:
            self.log.info(f"🛑 STOP LOSS HIT at ${current_price}")
            self._exit_position(position, "stop_loss")
        else:
            # Log position status
            pnl = (current_price - self.entry_price) * position.quantity
            pnl_pct = ((current_price / self.entry_price) - 1) * 100
            self.log.info(
                f"Position: Entry=${self.entry_price:.2f} | Current=${current_price:.2f} | "
                f"P/L: ${pnl:.2f} ({pnl_pct:+.2f}%) | "
                f"Target=${profit_target:.2f} | Stop=${stop_loss:.2f}",
            )

    def _exit_position(self, position, reason: str):
        """
        Exit position with limit order.
        """
        instrument = self.cache.instrument(self.instrument_id)
        if not instrument:
            return

        # Place limit sell order slightly below current price for quick fill
        current_price = self.cache.price(self.instrument_id, PriceType.LAST)
        if not current_price:
            return

        limit_price = Price(
            current_price.as_decimal() * Decimal("0.999"),
            instrument.price_precision,
        )

        order = self.order_factory.limit(
            instrument_id=self.instrument_id,
            order_side=OrderSide.SELL,
            quantity=position.quantity,
            price=limit_price,
            time_in_force=TimeInForce.GTC,  # GTC for crypto
        )

        self.submit_order(order)
        self.log.info(
            f"📉 Submitted SELL limit order ({reason}): {order.client_order_id} | "
            f"Price: ${limit_price} | Qty: {position.quantity}",
        )

        # Reset entry price
        self.entry_price = None

    def on_order_filled(self, event):
        """
        Handle order filled events.
        """
        self.log.info(
            f"✅ Order filled: {event.client_order_id} | "
            f"Price: ${event.last_px} | Qty: {event.last_qty}",
        )

    def on_order_rejected(self, event):
        """
        Handle order rejected events.
        """
        self.log.error(f"❌ Order rejected: {event.client_order_id} | Reason: {event.reason}")
        # Reset state on rejection
        self.entry_price = None

    def on_stop(self):
        """
        Actions to perform on strategy stop.
        """
        # Close any open positions
        if not self.portfolio.is_flat(self.instrument_id):
            self.log.info("Closing position on stop...")
            position = self.portfolio.position(self.instrument_id)
            if position:
                self._exit_position(position, "strategy_stop")

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

    # Crypto symbol to trade
    symbol = "BTC/USD"  # Bitcoin - adjust as needed (ETH/USD, DOGE/USD, etc.)
    instrument_id = InstrumentId.from_str(f"{symbol}.{ALPACA_VENUE}")

    # Create bar type (5-minute bars for scalping)
    bar_type = BarType.from_str(f"{symbol}.{ALPACA_VENUE}-5-MINUTE-LAST-EXTERNAL")

    # Configure trading node
    config = TradingNodeConfig(
        trader_id=TraderId("ALPACA-CRYPTO-SCALPER-001"),
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
                data_feed=AlpacaDataFeed.IEX,  # Crypto uses same feed
                instrument_provider=AlpacaInstrumentProviderConfig(
                    load_all=True,  # Load instruments on startup
                    asset_classes=frozenset({AlpacaAssetClass.CRYPTO}),
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
    strategy_config = CryptoScalperConfig(
        strategy_id="CRYPTO-SCALPER-001",
        instrument_id=instrument_id,
        bar_type=bar_type,
        atr_period=14,
        volatility_threshold=0.02,  # 2% volatility
        profit_target_atr_multiple=2.0,  # 2x ATR take profit
        stop_loss_atr_multiple=1.0,  # 1x ATR stop loss
        position_size_usd=Decimal(100),  # $100 per trade
    )
    strategy = CryptoScalperStrategy(config=strategy_config)

    # Build trading node
    node = TradingNode(config=config)
    node.trader.add_strategy(strategy)
    node.add_data_client_factory("ALPACA", AlpacaLiveDataClientFactory)
    node.add_exec_client_factory("ALPACA", AlpacaLiveExecClientFactory)
    node.build()

    # Run
    print("\n" + "=" * 70)
    print("Alpaca Cryptocurrency Scalping Strategy")
    print("=" * 70)
    print(f"Instrument: {instrument_id}")
    print(f"Bar Type: {bar_type}")
    print(f"ATR Period: {strategy_config.atr_period}")
    print(f"Volatility Threshold: {strategy_config.volatility_threshold * 100}%")
    print(f"Profit Target: {strategy_config.profit_target_atr_multiple}x ATR")
    print(f"Stop Loss: {strategy_config.stop_loss_atr_multiple}x ATR")
    print(f"Position Size: ${strategy_config.position_size_usd}")
    print("Paper Trading: True (safe mode)")
    print("\nCrypto Trading Notes:")
    print("  - 24/7 trading (no market hours)")
    print("  - GTC orders supported (Good-Till-Cancelled)")
    print("  - No shorting allowed (only long positions)")
    print("  - Symbol format: BTC/USD (will be converted to BTCUSD)")
    print("\nPress Ctrl+C to stop")
    print("=" * 70 + "\n")

    try:
        node.run()
    except KeyboardInterrupt:
        print("\nShutting down...")
    finally:
        node.dispose()


if __name__ == "__main__":
    # Import PriceType here for the _exit_position method
    from nautilus_trader.model.enums import PriceType

    main()
