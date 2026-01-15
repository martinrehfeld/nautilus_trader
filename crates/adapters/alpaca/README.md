# nautilus-alpaca

Alpaca Markets integration adapter for the NautilusTrader platform.

## Overview

This adapter provides integration with Alpaca Markets for live trading and market data:

**Supported Asset Classes:**
- 🟢 **US Equities** (stocks and ETFs) - Production ready
- 🟢 **Cryptocurrency** (BTC/USD, ETH/USD, etc.) - Production ready
- 🟡 **Options** - Partial support (margin calculator ready, full integration pending)

The official Alpaca API reference: <https://docs.alpaca.markets/>

## Architecture

This adapter follows a **hybrid Rust/Python architecture**:

```
┌─────────────────────────────────────────────────────┐
│              Python Layer                            │
│  • Framework integration (MessageBus, Cache, Clock) │
│  • Event generation                                  │
│  • Business logic orchestration                      │
│                                                      │
│  Components:                                         │
│  • AlpacaDataClient                                 │
│  • AlpacaExecutionClient                            │
│  • AlpacaInstrumentProvider                         │
└─────────────────────────────────────────────────────┘
                        ↓ uses
┌─────────────────────────────────────────────────────┐
│               Rust Layer (PyO3)                      │
│  • High-performance I/O                             │
│  • HTTP client                                       │
│  • WebSocket client                                  │
│  • Message parsing                                   │
│  • Configuration types                               │
└─────────────────────────────────────────────────────┘
```

This pattern (pioneered by the OKX adapter) provides:
- ⚡ High performance I/O in Rust
- 🎯 Seamless NautilusTrader integration in Python
- 🔧 Easy maintenance and testing

## Features

- `python`: Enables Python bindings via PyO3
- `extension-module`: Builds as a Python extension module

## API Endpoints

| Component | URL |
|-----------|-----|
| Trading API (Live) | `https://api.alpaca.markets` |
| Trading API (Paper) | `https://paper-api.alpaca.markets` |
| Market Data API | `https://data.alpaca.markets` |

## Quick Start

### Installation

```bash
# Build from source
make build
```

### Usage

```python
from nautilus_trader.adapters.alpaca import (
    ALPACA_VENUE,
    AlpacaDataClientConfig,
    AlpacaExecClientConfig,
    AlpacaLiveDataClientFactory,
    AlpacaLiveExecClientFactory,
)

# Configure data client
data_config = AlpacaDataClientConfig(
    api_key="your_key",
    api_secret="your_secret",
    paper_trading=True,  # Safe paper trading
)

# Configure execution client
exec_config = AlpacaExecClientConfig(
    api_key="your_key",
    api_secret="your_secret",
    paper_trading=True,
)
```

See [examples/live/alpaca/](../../../examples/live/alpaca/) for complete examples:
- `alpaca_equity_ema_cross.py` - US equity trading with EMA strategy
- `alpaca_crypto_scalper.py` - Cryptocurrency scalping
- `alpaca_options_spread_trader.py` - Multi-leg options spreads

## Implementation Details

### Rust Components (src/)

**HTTP Client** ([http/](./src/http/))
- Async HTTP client with connection pooling
- Automatic retry with exponential backoff
- Request/response types for all endpoints
- Paper and live environment support

**WebSocket Client** ([websocket/](./src/websocket/))
- Real-time market data streaming
- Automatic reconnection with backoff
- Message parsing and authentication
- Support for IEX and SIP data feeds

**Data Types** ([types/](./src/types/))
- Market data (trades, quotes, bars)
- Account information
- Order and position types
- Asset definitions

**Options Margin Calculator** ([margin/](./src/margin/))
- SPAN-like margin calculation
- Support for complex multi-leg strategies
- Position risk analysis

### Python Components (nautilus_trader/adapters/alpaca/)

**Data Client** ([data.py](../../../nautilus_trader/adapters/alpaca/data.py))
- WebSocket subscription management
- Real-time data publishing to MessageBus
- Historical data requests
- Automatic instrument loading

**Execution Client** ([execution.py](../../../nautilus_trader/adapters/alpaca/execution.py))
- Order submission and modification
- Event generation (fills, rejections, etc.)
- Position and balance tracking
- Order status synchronization

**Instrument Provider** ([providers.py](../../../nautilus_trader/adapters/alpaca/providers.py))
- Asset discovery and loading
- Instrument type conversion
- Caching for performance

**Factories** ([factories.py](../../../nautilus_trader/adapters/alpaca/factories.py))
- Client instantiation
- Shared HTTP client caching
- Configuration management

## Development Status

### Completed ✅
- HTTP client with full API coverage
- WebSocket client for market data
- Data client implementation
- Execution client implementation
- Instrument provider
- Options margin calculator
- Configuration types
- Error handling
- Build system integration
- Example strategies

### Future Enhancements 🔮
- Full options trading integration
- Advanced order types (bracket, OCO)
- Real-time account updates via WebSocket
- Historical data caching
- Rate limit optimization

## Testing

```bash
# Run validation tests
.venv/bin/python test_alpaca_adapter.py

# Test with paper trading
export ALPACA_API_KEY=your_paper_key
export ALPACA_API_SECRET=your_paper_secret
python examples/live/alpaca/alpaca_equity_ema_cross.py
```

## Resources

- [Alpaca API Documentation](https://docs.alpaca.markets/)
- [NautilusTrader Documentation](https://nautilustrader.io/)

## License

This adapter is part of NautilusTrader and is licensed under the GNU Lesser General Public License v3.0.
