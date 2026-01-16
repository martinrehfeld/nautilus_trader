#!/usr/bin/env python3
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

import os
import sys

from nautilus_trader.adapters.alpaca import ALPACA_VENUE
from nautilus_trader.adapters.alpaca import AlpacaDataClientConfig
from nautilus_trader.adapters.alpaca import AlpacaLiveDataClientFactory
from nautilus_trader.adapters.alpaca import AlpacaInstrumentProviderConfig

from nautilus_trader.config import LiveExecEngineConfig
from nautilus_trader.config import LoggingConfig
from nautilus_trader.config import TradingNodeConfig
from nautilus_trader.live.node import TradingNode
from nautilus_trader.model.currencies import USD
from nautilus_trader.model.data import BarType
from nautilus_trader.model.instruments import Equity
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import TraderId
from nautilus_trader.model.identifiers import Symbol

from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity

from nautilus_trader.test_kit.strategies.tester_data import DataTester
from nautilus_trader.test_kit.strategies.tester_data import DataTesterConfig


# *** THIS IS A TEST STRATEGY WITH NO ALPHA ADVANTAGE WHATSOEVER. ***
# *** IT IS NOT INTENDED TO BE USED TO TRADE LIVE WITH REAL MONEY. ***


# Check for API credentials
api_key = os.getenv("ALPACA_API_KEY")
api_secret = os.getenv("ALPACA_API_SECRET")

if not api_key or not api_secret:
    print("ERROR: Alpaca API credentials not set!")
    print("Set your credentials:")
    print("  export ALPACA_API_KEY=your_key")
    print("  export ALPACA_API_SECRET=your_secret")
    sys.exit(1)

# Fake instrument for test channel, see https://docs.alpaca.markets/docs/streaming-market-data#connection
symbol = "FAKEPACA" 
instrument_id = InstrumentId.from_str(f"{symbol}.{ALPACA_VENUE}")
test_websocket_url = "wss://stream.data.alpaca.markets/v2/test"

# Configure the trading node
config_node = TradingNodeConfig(
    trader_id=TraderId("TESTER-001"),
    logging=LoggingConfig(
        log_level="INFO",
        # log_level_file="DEBUG",
        use_pyo3=True,
    ),
    exec_engine=LiveExecEngineConfig(
        reconciliation=False,  # Not applicable
    ),
    data_clients={
        "ALPACA": AlpacaDataClientConfig(
            api_key=api_key,
            api_secret=api_secret,
            base_url_ws=test_websocket_url,  # Override with test channel endpoint
            instrument_provider=AlpacaInstrumentProviderConfig(
                load_all=True,
            ),
        ),
    },
    timeout_connection=20.0,
    timeout_disconnection=5.0,
    timeout_post_stop=5.0,
)

# Instantiate the node with a configuration
node = TradingNode(config=config_node)

# Configure and initialize the tester
config_tester = DataTesterConfig(
    instrument_ids=[instrument_id],
    bar_types=[
        BarType.from_str(f"{instrument_id.value}-1-MINUTE-LAST-EXTERNAL"),
    ],
    # subscribe_book_deltas=True,
    # subscribe_book_depth=True,
    # subscribe_book_at_interval=True,  # Only legacy Cython wrapped book (not PyO3)
    subscribe_quotes=True,
    subscribe_trades=True,
    # subscribe_mark_prices=True,
    subscribe_index_prices=False,  # Only for some derivatives
    # subscribe_funding_rates=True,
    subscribe_bars=True,
    # subscribe_instrument=True,
    # subscribe_instrument_status=True,
    # subscribe_instrument_close=True,
    # request_bars=True,
    # book_group_size=Decimal("1"),  # Only PyO3 wrapped book (not legacy Cython)
    # book_depth=5,
    # book_levels_to_print=50,
    # book_interval_ms=100,
    # manage_book=True,
    # use_pyo3_book=True,
    # request_instruments=True,
    # request_bars=True,
    # request_trades=True,
    # requests_start_delta=pd.Timedelta(minutes=60),
)
tester = DataTester(config=config_tester)

node.trader.add_actor(tester)

# Register your client factories with the node (can take user-defined factories)
node.add_data_client_factory("ALPACA", AlpacaLiveDataClientFactory)

node.build()

# Manually add test instrument to cache since the InstrumentProvider won't have it
instrument = Equity(
    instrument_id=instrument_id,
    raw_symbol=Symbol(symbol),
    currency=USD,
    price_precision=2,
    price_increment=Price.from_str("0.01"),
    lot_size=Quantity.from_int(1),
    ts_event=0,
    ts_init=0,
)
node.cache.add_instrument(instrument)

# Stop and dispose of the node with SIGINT/CTRL+C
if __name__ == "__main__":
    try:
        node.run()
    finally:
        node.dispose()
