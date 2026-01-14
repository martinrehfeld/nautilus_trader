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
Alpaca Markets adapter for NautilusTrader.

This adapter provides integration with Alpaca Markets for:
- US Equities (stocks and ETFs)
- Cryptocurrency trading
- Options trading

This module provides thin Python wrappers over the Rust implementation
for backward compatibility with existing examples and user code.

API Documentation: https://docs.alpaca.markets/
"""

from __future__ import annotations

try:
    # Import Rust components (constants, enums, HTTP client, margin calculator)
    from nautilus_trader.core.nautilus_pyo3.alpaca import ALPACA_VENUE
    from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaAssetClass
    from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaDataFeed
    from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaHttpClient
    from nautilus_trader.core.nautilus_pyo3.alpaca import AlpacaOptionsMarginCalculator
except ImportError:
    # Fallback: Try importing from nautilus_pyo3 directly (alternative module structure)
    try:
        from nautilus_pyo3.alpaca import ALPACA_VENUE
        from nautilus_pyo3.alpaca import AlpacaAssetClass
        from nautilus_pyo3.alpaca import AlpacaDataFeed
        from nautilus_pyo3.alpaca import AlpacaHttpClient
        from nautilus_pyo3.alpaca import AlpacaOptionsMarginCalculator
    except ImportError as e:
        # Provide helpful error message if Rust module not built
        raise ImportError(
            "Alpaca adapter Rust module not found. "
            "Please ensure the Rust extension is built with: uv build"
        ) from e

# Import Python config classes (with routing support)
from nautilus_trader.adapters.alpaca.config import AlpacaDataClientConfig
from nautilus_trader.adapters.alpaca.config import AlpacaExecClientConfig
from nautilus_trader.adapters.alpaca.config import AlpacaInstrumentProviderConfig

# Import Python implementations (hybrid architecture)
from nautilus_trader.adapters.alpaca.data import AlpacaDataClient
from nautilus_trader.adapters.alpaca.execution import AlpacaExecutionClient
from nautilus_trader.adapters.alpaca.providers import AlpacaInstrumentProvider

# Import factories from Python module
from nautilus_trader.adapters.alpaca.factories import AlpacaLiveDataClientFactory
from nautilus_trader.adapters.alpaca.factories import AlpacaLiveExecClientFactory


__all__ = [
    # Constants and enums
    "ALPACA_VENUE",
    "AlpacaAssetClass",
    "AlpacaDataFeed",
    # Config classes
    "AlpacaDataClientConfig",
    "AlpacaExecClientConfig",
    "AlpacaInstrumentProviderConfig",
    # Client classes
    "AlpacaDataClient",
    "AlpacaExecutionClient",
    "AlpacaHttpClient",
    "AlpacaInstrumentProvider",
    # Factories
    "AlpacaLiveDataClientFactory",
    "AlpacaLiveExecClientFactory",
    # Utilities
    "AlpacaOptionsMarginCalculator",
]
