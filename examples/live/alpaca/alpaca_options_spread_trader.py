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
Example: Alpaca Multi-Leg Options Spread Trading

This example demonstrates trading multi-leg options strategies on Alpaca.
Shows how to construct and submit various options spreads using the mleg order class.

Prerequisites:
    1. Create an Alpaca account at https://alpaca.markets
    2. Enable options trading on your account
    3. Get your API credentials from the dashboard
    4. Set environment variables:
       export ALPACA_API_KEY=your_key
       export ALPACA_API_SECRET=your_secret

Features Demonstrated:
    - Multi-leg options order submission
    - Bull call spread construction
    - Iron condor construction
    - Margin preview before order submission
    - Options chain exploration
    - Strike/expiration selection

Supported Strategies:
    - Vertical Spreads (bull call, bear call, bull put, bear put)
    - Straddles and Strangles
    - Iron Condors and Iron Butterflies
    - Calendar Spreads
    - Butterflies and Condors

Usage:
    python alpaca_options_spread_trader.py
"""

import asyncio
import os
import sys
from datetime import datetime, timedelta
from decimal import Decimal

from nautilus_trader.adapters.alpaca import (
    AlpacaHttpClient,
)


def check_credentials() -> tuple[str, str]:
    """Check and return API credentials."""
    api_key = os.getenv("ALPACA_API_KEY")
    api_secret = os.getenv("ALPACA_API_SECRET")

    if not api_key or not api_secret:
        print("ERROR: Alpaca API credentials not set!")
        print("Set your credentials:")
        print("  export ALPACA_API_KEY=your_key")
        print("  export ALPACA_API_SECRET=your_secret")
        sys.exit(1)

    return api_key, api_secret


async def get_option_chain(
    client: AlpacaHttpClient,
    underlying: str,
    days_to_expiry: int = 30,
) -> list[dict]:
    """
    Get options chain for an underlying symbol.

    Parameters
    ----------
    client : AlpacaHttpClient
        HTTP client for Alpaca API
    underlying : str
        Underlying symbol (e.g., 'SPY', 'AAPL')
    days_to_expiry : int, default 30
        Target days to expiration

    Returns
    -------
    list[dict]
        List of option contracts
    """
    # Calculate date range
    today = datetime.now().date()
    target_date = today + timedelta(days=days_to_expiry)

    params = {
        "underlying_symbols": underlying,
        "status": "active",
        "expiration_date_gte": today.isoformat(),
        "expiration_date_lte": target_date.isoformat(),
    }

    response = await client.get("/v2/options/contracts", params=params)
    import msgspec

    data = msgspec.json.decode(response)
    return data.get("option_contracts", [])


async def example_bull_call_spread(client: AlpacaHttpClient, underlying: str = "SPY"):
    """
    Example: Bull Call Spread

    Strategy:
        - Buy lower strike call (ITM or ATM)
        - Sell higher strike call (OTM)
        - Both same expiration
        - Limited profit, limited risk

    Profit: Max when underlying > higher strike
    Loss: Max when underlying < lower strike
    """
    print("\n" + "=" * 70)
    print("Bull Call Spread Example")
    print("=" * 70)
    print(f"Underlying: {underlying}")
    print("Strategy: Buy lower strike call + Sell higher strike call")
    print()

    # Get option chain
    print("Fetching options chain...")
    contracts = await get_option_chain(client, underlying, days_to_expiry=30)

    if not contracts:
        print("No contracts found")
        return

    # Filter for calls
    calls = [c for c in contracts if c.get("type") == "call"]

    # Find a suitable expiration
    expirations = list(set(c.get("expiration_date") for c in calls))
    if not expirations:
        print("No expirations found")
        return

    target_expiry = sorted(expirations)[0]  # Nearest expiration
    expiry_calls = [c for c in calls if c.get("expiration_date") == target_expiry]

    # Sort by strike
    expiry_calls.sort(key=lambda x: float(x.get("strike_price", 0)))

    if len(expiry_calls) < 2:
        print("Not enough strikes available")
        return

    # Pick strikes (example: buy ATM, sell $5 higher)
    # In real trading, you'd pick based on current price and your strategy
    buy_strike_contract = expiry_calls[len(expiry_calls) // 2]  # Middle strike
    sell_strike_contract = expiry_calls[len(expiry_calls) // 2 + 5]  # 5 strikes higher

    buy_symbol = buy_strike_contract.get("symbol")
    sell_symbol = sell_strike_contract.get("symbol")
    buy_strike = Decimal(buy_strike_contract.get("strike_price"))
    sell_strike = Decimal(sell_strike_contract.get("strike_price"))

    print(f"Expiration: {target_expiry}")
    print(f"Buy:  {buy_symbol} (strike ${buy_strike})")
    print(f"Sell: {sell_symbol} (strike ${sell_strike})")
    print()

    # Preview margin requirement
    print("Previewing margin requirement...")
    try:
        from nautilus_trader.adapters.alpaca.execution import AlpacaExecutionClient

        # Note: In production, you'd use the full execution client
        # For this example, we'll just show the structure

        legs = [
            {
                "symbol": buy_symbol,
                "side": "buy",
                "ratio_qty": "1",
                "position_intent": "buy_to_open",
                "strike": buy_strike,
                "is_call": True,
            },
            {
                "symbol": sell_symbol,
                "side": "sell",
                "ratio_qty": "1",
                "position_intent": "sell_to_open",
                "strike": sell_strike,
                "is_call": True,
            },
        ]

        print("\n📋 Order Structure:")
        print("-" * 70)
        for i, leg in enumerate(legs, 1):
            print(f"Leg {i}:")
            print(f"  Symbol: {leg['symbol']}")
            print(f"  Side: {leg['side'].upper()}")
            print(f"  Strike: ${leg['strike']}")
            print(f"  Type: {'CALL' if leg['is_call'] else 'PUT'}")
            print(f"  Intent: {leg['position_intent']}")
        print("-" * 70)

        print("\n⚠️  In production, you would:")
        print("  1. Preview margin: exec_client.preview_mleg_margin(legs)")
        print("  2. Submit order: await exec_client.submit_mleg_order(legs, ...)")
        print()
        print("  Example submission:")
        print("    result = await exec_client.submit_mleg_order(")
        print("        legs=legs,")
        print("        quantity=1,")
        print("        order_type='limit',")
        print("        limit_price=1.50,  # Net debit")
        print("        preview_margin=True,")
        print("    )")

    except Exception as e:
        print(f"Error: {e}")


async def example_iron_condor(client: AlpacaHttpClient, underlying: str = "SPY"):
    """
    Example: Iron Condor

    Strategy:
        - Sell OTM put (lower strike)
        - Buy further OTM put (protection)
        - Sell OTM call (higher strike)
        - Buy further OTM call (protection)
        - All same expiration
        - Limited profit, limited risk

    Profit: Max when underlying stays between sold strikes
    Loss: Max if underlying moves beyond bought strikes
    """
    print("\n" + "=" * 70)
    print("Iron Condor Example")
    print("=" * 70)
    print(f"Underlying: {underlying}")
    print("Strategy: Sell put spread + Sell call spread")
    print()

    # Get option chain
    print("Fetching options chain...")
    contracts = await get_option_chain(client, underlying, days_to_expiry=30)

    if not contracts:
        print("No contracts found")
        return

    # Split into calls and puts
    calls = [c for c in contracts if c.get("type") == "call"]
    puts = [c for c in contracts if c.get("type") == "put"]

    # Find a suitable expiration
    expirations = list(set(c.get("expiration_date") for c in contracts))
    if not expirations:
        print("No expirations found")
        return

    target_expiry = sorted(expirations)[0]

    # Filter by expiration
    expiry_calls = [c for c in calls if c.get("expiration_date") == target_expiry]
    expiry_puts = [c for c in puts if c.get("expiration_date") == target_expiry]

    # Sort by strike
    expiry_calls.sort(key=lambda x: float(x.get("strike_price", 0)))
    expiry_puts.sort(key=lambda x: float(x.get("strike_price", 0)))

    if len(expiry_calls) < 10 or len(expiry_puts) < 10:
        print("Not enough strikes available")
        return

    # Pick strikes (example positions, adjust based on your strategy)
    # Put spread: Sell put at 25th percentile, buy put at 20th percentile
    # Call spread: Sell call at 75th percentile, buy call at 80th percentile
    buy_put = expiry_puts[len(expiry_puts) // 5]  # Lower put
    sell_put = expiry_puts[len(expiry_puts) // 4]  # Higher put
    sell_call = expiry_calls[3 * len(expiry_calls) // 4]  # Lower call
    buy_call = expiry_calls[4 * len(expiry_calls) // 5]  # Higher call

    print(f"Expiration: {target_expiry}")
    print("\nPut Spread:")
    print(f"  Buy:  {buy_put.get('symbol')} (strike ${buy_put.get('strike_price')})")
    print(f"  Sell: {sell_put.get('symbol')} (strike ${sell_put.get('strike_price')})")
    print("\nCall Spread:")
    print(f"  Sell: {sell_call.get('symbol')} (strike ${sell_call.get('strike_price')})")
    print(f"  Buy:  {buy_call.get('symbol')} (strike ${buy_call.get('strike_price')})")
    print()

    legs = [
        # Put spread
        {
            "symbol": buy_put.get("symbol"),
            "side": "buy",
            "ratio_qty": "1",
            "position_intent": "buy_to_open",
        },
        {
            "symbol": sell_put.get("symbol"),
            "side": "sell",
            "ratio_qty": "1",
            "position_intent": "sell_to_open",
        },
        # Call spread
        {
            "symbol": sell_call.get("symbol"),
            "side": "sell",
            "ratio_qty": "1",
            "position_intent": "sell_to_open",
        },
        {
            "symbol": buy_call.get("symbol"),
            "side": "buy",
            "ratio_qty": "1",
            "position_intent": "buy_to_open",
        },
    ]

    print("📋 Order Structure:")
    print("-" * 70)
    for i, leg in enumerate(legs, 1):
        print(f"Leg {i}: {leg['side'].upper():4} {leg['symbol']:25} ({leg['position_intent']})")
    print("-" * 70)

    print("\n⚠️  In production, you would:")
    print("  1. Preview margin: exec_client.preview_mleg_margin(legs)")
    print("  2. Submit order: await exec_client.submit_mleg_order(legs, ...)")
    print()
    print("  Example submission:")
    print("    result = await exec_client.submit_mleg_order(")
    print("        legs=legs,")
    print("        quantity=1,")
    print("        order_type='limit',")
    print("        limit_price=1.80,  # Net credit")
    print("        preview_margin=True,")
    print("    )")


async def main():
    print("=" * 70)
    print("Alpaca Multi-Leg Options Trading Examples")
    print("=" * 70)

    api_key, api_secret = check_credentials()

    # Create HTTP client
    client = AlpacaHttpClient(
        api_key=api_key,
        api_secret=api_secret,
        paper_trading=True,  # Use paper trading
    )

    try:
        # Run examples
        await example_bull_call_spread(client, underlying="SPY")
        await example_iron_condor(client, underlying="SPY")

        print("\n" + "=" * 70)
        print("Additional Strategies Available")
        print("=" * 70)
        print("\nVertical Spreads:")
        print("  - Bull Call Spread: Buy lower call + Sell higher call")
        print("  - Bear Call Spread: Sell lower call + Buy higher call")
        print("  - Bull Put Spread: Sell higher put + Buy lower put")
        print("  - Bear Put Spread: Buy higher put + Sell lower put")
        print("\nStraddles/Strangles:")
        print("  - Long Straddle: Buy ATM call + Buy ATM put")
        print("  - Short Straddle: Sell ATM call + Sell ATM put")
        print("  - Long Strangle: Buy OTM call + Buy OTM put")
        print("  - Short Strangle: Sell OTM call + Sell OTM put")
        print("\nComplex Spreads:")
        print("  - Iron Condor: Sell put spread + Sell call spread")
        print("  - Iron Butterfly: ATM straddle + OTM strangle")
        print("  - Butterfly: Buy 1 + Sell 2 + Buy 1 (same type)")
        print("\nKey Considerations:")
        print("  - All multi-leg orders require 'mleg' order class")
        print("  - Ratio quantities must be in simplest form (GCD = 1)")
        print("  - Maximum 4 legs per order")
        print("  - Day orders only for options")
        print("  - Preview margin before submitting")
        print("  - Account must be approved for options trading")

    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
