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

from decimal import Decimal
from unittest.mock import AsyncMock
from unittest.mock import MagicMock

import pytest

from nautilus_trader.adapters.alpaca.providers import AlpacaInstrumentProvider
from nautilus_trader.core import nautilus_pyo3
from nautilus_trader.core.nautilus_pyo3.alpaca import ALPACA_VENUE
from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaEnvironment
from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaHttpClient
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import Logger
from nautilus_trader.model.currencies import USD
from nautilus_trader.model.enums import CurrencyType
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import Symbol
from nautilus_trader.model.identifiers import Venue
from nautilus_trader.model.instruments.currency_pair import CurrencyPair
from nautilus_trader.model.objects import Currency
from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity
from nautilus_trader.test_kit.stubs.identifiers import TestIdStubs


def _create_currency(
    code: str,
    precision: int,
    iso4217: int,
    name: str,
    currency_type: CurrencyType,
) -> Currency:
    return Currency(
        code=code,
        precision=precision,
        iso4217=iso4217,
        name=name,
        currency_type=currency_type,
    )


@pytest.fixture(scope="session")
def live_clock():
    return LiveClock()


@pytest.fixture(scope="session")
def live_logger():
    return Logger("TEST_LOGGER")


@pytest.fixture(scope="session")
def alpaca_http_client():
    client = AlpacaHttpClient(
        environment=AlpacaEnvironment.Paper,
        api_key="SOME_ALPACA_API_KEY",
        api_secret="SOME_ALPACA_API_SECRET",
    )
    return client


@pytest.fixture
def venue() -> Venue:
    return ALPACA_VENUE


@pytest.fixture
def instrument() -> CurrencyPair:
    btc = _create_currency(
        "BTC",
        precision=8,
        iso4217=0,
        name="Bitcoin",
        currency_type=CurrencyType.CRYPTO,
    )
    usd = USD

    return CurrencyPair(
        instrument_id=InstrumentId(Symbol("BTC-USD"), Venue(ALPACA_VENUE)),
        raw_symbol=Symbol("BTC-USD"),
        base_currency=btc,
        quote_currency=usd,
        price_precision=2,
        size_precision=6,
        price_increment=Price.from_str("0.01"),
        size_increment=Quantity.from_str("0.000001"),
        ts_event=0,
        ts_init=0,
        maker_fee=Decimal("-0.0002"),
        taker_fee=Decimal("0.0005"),
    )


@pytest.fixture
def instrument_provider():
    return None


@pytest.fixture
def data_client():
    return None


@pytest.fixture
def exec_client():
    return None


@pytest.fixture
def account_state():
    return None


# TODO: mock actual Alpaca methods, i.e. request_instruments might be wrong
@pytest.fixture
def mock_http_client():
    mock = MagicMock(spec=nautilus_pyo3.AlpacaHttpClient)
    mock.api_key = "test_api_key"
    mock.api_secret = "test_api_secret"

    mock.request_instruments = AsyncMock(return_value=([], []))
    mock.cache_instrument = MagicMock()
    mock.cancel_all_requests = MagicMock()
    mock.is_initialized = MagicMock(return_value=True)
    mock.get_server_time = AsyncMock(return_value=1234567890000)

    mock_account_state = MagicMock()
    mock_account_state.to_dict = MagicMock(
        return_value={
            "account_id": "ALPACA-test",
            "account_type": "CASH",
            "base_currency": "USD",
            "reported": True,
            "balances": [
                {
                    "currency": "USD",
                    "total": "100000.0",
                    "locked": "0.0",
                    "free": "100000.0",
                },
            ],
            "margins": [
                {
                    "type": "MarginBalance",
                    "initial": "0.00",
                    "maintenance": "0.00",
                    "currency": "USD",
                    "instrument_id": None,
                }
            ],
            "info": {},
            "event_id": str(TestIdStubs.uuid()),
            "ts_event": 0,
            "ts_init": 0,
        },
    )
    mock.get_account = AsyncMock(return_value=mock_account_state)

    mock.request_order_status_reports = AsyncMock(return_value=[])
    mock.request_fill_reports = AsyncMock(return_value=[])
    mock.request_position_status_reports = AsyncMock(return_value=[])
    mock.request_trades = AsyncMock(return_value=[])
    mock.request_bars = AsyncMock(return_value=[])

    return mock

# TODO: mock actual Alpaca methods
@pytest.fixture
def mock_instrument_provider(instrument):
    provider = MagicMock(spec=AlpacaInstrumentProvider)
    provider.initialize = AsyncMock()
    provider.instruments_pyo3 = MagicMock(return_value=[MagicMock(name="py_instrument")])
    provider.inst_id_codes = MagicMock(return_value=[])
    provider.get_all = MagicMock(return_value={instrument.id: instrument})
    provider.currencies = MagicMock(return_value={})
    provider.find = MagicMock(return_value=instrument)
    # provider.instrument_types = (nautilus_pyo3.OKXInstrumentType.SPOT,)
    # provider._instrument_types = (nautilus_pyo3.OKXInstrumentType.SPOT,)
    return provider


def _create_ws_mock() -> MagicMock:
    mock = MagicMock(spec=nautilus_pyo3.AlpacaWebSocketClient)
    mock.connect = AsyncMock(return_value=mock)
    mock.url = "wss://stream.data.alpaca.markets"
    return mock
