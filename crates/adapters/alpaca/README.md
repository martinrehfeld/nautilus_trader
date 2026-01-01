# nautilus-alpaca

Alpaca Markets integration adapter for the NautilusTrader platform.

## Overview

This crate provides client bindings (HTTP & WebSocket), data models, and helper utilities
that wrap the official Alpaca API, covering:

- US Equities (stocks and ETFs)
- Cryptocurrency trading
- Options trading

The official Alpaca API reference can be found at <https://docs.alpaca.markets/>.

## Features

- `python`: Enables Python bindings via PyO3
- `extension-module`: Builds as a Python extension module

## API Endpoints

| Component | URL |
|-----------|-----|
| Trading API (Live) | `https://api.alpaca.markets` |
| Trading API (Paper) | `https://paper-api.alpaca.markets` |
| Market Data API | `https://data.alpaca.markets` |
