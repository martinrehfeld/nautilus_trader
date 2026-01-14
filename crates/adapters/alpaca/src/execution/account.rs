// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2026 Andrew Crum. All rights reserved.
//  https://github.com/agcrum
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Account state management for the Alpaca execution client.

use std::collections::HashMap;

use nautilus_model::{
    identifiers::{AccountId, InstrumentId},
    types::{currency::Currency, money::Money, quantity::Quantity},
};
use rust_decimal::{prelude::ToPrimitive, Decimal};

use crate::http::models::{AlpacaAccount, AlpacaPosition};

/// Account state tracker for Alpaca accounts.
#[derive(Debug, Clone)]
pub struct AlpacaAccountState {
    /// Account ID.
    pub account_id: AccountId,
    /// Account number.
    pub account_number: String,
    /// Account status.
    pub status: String,
    /// Cash balance.
    pub cash: Money,
    /// Buying power.
    pub buying_power: Money,
    /// Equity value.
    pub equity: Money,
    /// Pattern day trader flag.
    pub pattern_day_trader: bool,
    /// Trading blocked flag.
    pub trading_blocked: bool,
    /// Shorting enabled flag.
    pub shorting_enabled: bool,
    /// Multiplier (leverage).
    pub multiplier: i32,
    /// Day trade count.
    pub daytrade_count: i32,
    /// Initial margin.
    pub initial_margin: Money,
    /// Maintenance margin.
    pub maintenance_margin: Money,
    /// Long market value.
    pub long_market_value: Money,
    /// Short market value.
    pub short_market_value: Money,
    /// Positions indexed by instrument ID.
    positions: HashMap<InstrumentId, PositionState>,
}

impl AlpacaAccountState {
    /// Creates a new account state from an Alpaca account response.
    pub fn from_alpaca_account(account: &AlpacaAccount, account_id: AccountId) -> Self {
        let currency = Currency::USD();

        Self {
            account_id,
            account_number: account.account_number.clone(),
            status: account.status.clone(),
            cash: Money::new(
                Decimal::from_str_exact(&account.cash)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            buying_power: Money::new(
                Decimal::from_str_exact(&account.buying_power)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            equity: Money::new(
                Decimal::from_str_exact(&account.equity)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            pattern_day_trader: account.pattern_day_trader,
            trading_blocked: account.trading_blocked,
            shorting_enabled: account.shorting_enabled,
            multiplier: account.multiplier.parse().unwrap_or(1),
            daytrade_count: account.daytrade_count,
            initial_margin: Money::new(
                Decimal::from_str_exact(&account.initial_margin)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            maintenance_margin: Money::new(
                Decimal::from_str_exact(&account.maintenance_margin)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            long_market_value: Money::new(
                Decimal::from_str_exact(&account.long_market_value)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            short_market_value: Money::new(
                Decimal::from_str_exact(&account.short_market_value)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            positions: HashMap::new(),
        }
    }

    /// Updates the account state from a new Alpaca account response.
    pub fn update_from_alpaca_account(&mut self, account: &AlpacaAccount) {
        let currency = Currency::USD();

        self.account_number = account.account_number.clone();
        self.status = account.status.clone();
        self.cash = Money::new(
            Decimal::from_str_exact(&account.cash)
                .unwrap_or_default()
                .to_f64()
                .unwrap_or(0.0),
            currency,
        );
        self.buying_power = Money::new(
            Decimal::from_str_exact(&account.buying_power)
                .unwrap_or_default()
                .to_f64()
                .unwrap_or(0.0),
            currency,
        );
        self.equity = Money::new(
            Decimal::from_str_exact(&account.equity)
                .unwrap_or_default()
                .to_f64()
                .unwrap_or(0.0),
            currency,
        );
        self.pattern_day_trader = account.pattern_day_trader;
        self.trading_blocked = account.trading_blocked;
        self.shorting_enabled = account.shorting_enabled;
        self.multiplier = account.multiplier.parse().unwrap_or(1);
        self.daytrade_count = account.daytrade_count;
        self.initial_margin = Money::new(
            Decimal::from_str_exact(&account.initial_margin)
                .unwrap_or_default()
                .to_f64()
                .unwrap_or(0.0),
            currency,
        );
        self.maintenance_margin = Money::new(
            Decimal::from_str_exact(&account.maintenance_margin)
                .unwrap_or_default()
                .to_f64()
                .unwrap_or(0.0),
            currency,
        );
        self.long_market_value = Money::new(
            Decimal::from_str_exact(&account.long_market_value)
                .unwrap_or_default()
                .to_f64()
                .unwrap_or(0.0),
            currency,
        );
        self.short_market_value = Money::new(
            Decimal::from_str_exact(&account.short_market_value)
                .unwrap_or_default()
                .to_f64()
                .unwrap_or(0.0),
            currency,
        );
    }

    /// Updates or inserts a position.
    pub fn update_position(&mut self, instrument_id: InstrumentId, position: PositionState) {
        self.positions.insert(instrument_id, position);
    }

    /// Removes a position.
    pub fn remove_position(&mut self, instrument_id: &InstrumentId) {
        self.positions.remove(instrument_id);
    }

    /// Gets a position by instrument ID.
    pub fn get_position(&self, instrument_id: &InstrumentId) -> Option<&PositionState> {
        self.positions.get(instrument_id)
    }

    /// Gets all positions.
    pub fn positions(&self) -> &HashMap<InstrumentId, PositionState> {
        &self.positions
    }

    /// Checks if margin trading is enabled (requires $2,000+ equity).
    pub fn is_margin_enabled(&self) -> bool {
        self.equity.as_f64() >= 2000.0
    }

    /// Gets the available buying power for a given order value.
    pub fn available_buying_power(&self, order_value: Money) -> Money {
        Money::new(
            (self.buying_power.as_decimal() - order_value.as_decimal())
                .to_f64()
                .unwrap_or(0.0),
            self.buying_power.currency,
        )
    }
}

/// Position state tracker.
#[derive(Debug, Clone)]
pub struct PositionState {
    /// Instrument ID.
    pub instrument_id: InstrumentId,
    /// Asset ID.
    pub asset_id: String,
    /// Current quantity (positive for long, negative for short).
    pub quantity: Quantity,
    /// Available quantity for trading.
    pub qty_available: Quantity,
    /// Average entry price.
    pub avg_entry_price: Decimal,
    /// Current market price.
    pub current_price: Decimal,
    /// Market value.
    pub market_value: Money,
    /// Cost basis.
    pub cost_basis: Money,
    /// Unrealized P&L.
    pub unrealized_pl: Money,
    /// Unrealized P&L percentage.
    pub unrealized_plpc: Decimal,
}

impl PositionState {
    /// Creates a new position state from an Alpaca position response.
    pub fn from_alpaca_position(position: &AlpacaPosition, instrument_id: InstrumentId) -> Self {
        let currency = Currency::USD();
        let qty = Decimal::from_str_exact(&position.qty).unwrap_or_default();
        let precision = if qty.fract() != Decimal::ZERO { 9 } else { 0 };

        Self {
            instrument_id,
            asset_id: position.asset_id.clone(),
            quantity: Quantity::new(qty.to_f64().unwrap_or(0.0), precision),
            qty_available: Quantity::new(
                Decimal::from_str_exact(&position.qty_available)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                precision,
            ),
            avg_entry_price: Decimal::from_str_exact(&position.avg_entry_price).unwrap_or_default(),
            current_price: Decimal::from_str_exact(&position.current_price).unwrap_or_default(),
            market_value: Money::new(
                Decimal::from_str_exact(&position.market_value)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            cost_basis: Money::new(
                Decimal::from_str_exact(&position.cost_basis)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            unrealized_pl: Money::new(
                Decimal::from_str_exact(&position.unrealized_pl)
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or(0.0),
                currency,
            ),
            unrealized_plpc: Decimal::from_str_exact(&position.unrealized_plpc).unwrap_or_default(),
        }
    }

    /// Checks if this is a long position.
    pub fn is_long(&self) -> bool {
        self.quantity.as_decimal() > Decimal::ZERO
    }

    /// Checks if this is a short position.
    pub fn is_short(&self) -> bool {
        self.quantity.as_decimal() < Decimal::ZERO
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use nautilus_model::identifiers::{Symbol, Venue};

    #[rstest]
    fn test_account_state_margin_enabled() {
        let account = AlpacaAccount {
            id: "test".to_string(),
            account_number: "123".to_string(),
            status: "ACTIVE".to_string(),
            currency: "USD".to_string(),
            buying_power: "10000".to_string(),
            regt_buying_power: "10000".to_string(),
            daytrading_buying_power: "40000".to_string(),
            non_marginable_buying_power: "10000".to_string(),
            cash: "5000".to_string(),
            accrued_fees: "0".to_string(),
            pending_transfer_out: None,
            pending_transfer_in: None,
            portfolio_value: "10000".to_string(),
            pattern_day_trader: false,
            trading_blocked: false,
            transfers_blocked: false,
            account_blocked: false,
            created_at: "2024-01-01".to_string(),
            trade_suspended_by_user: false,
            multiplier: "2".to_string(),
            shorting_enabled: true,
            equity: "10000".to_string(),
            last_equity: "10000".to_string(),
            long_market_value: "5000".to_string(),
            short_market_value: "0".to_string(),
            initial_margin: "2500".to_string(),
            maintenance_margin: "1500".to_string(),
            last_maintenance_margin: "1500".to_string(),
            sip: "10000".to_string(),
            daytrade_count: 0,
            balance_asof: None,
            crypto_status: None,
        };

        let account_id = AccountId::from("ALPACA-001");
        let state = AlpacaAccountState::from_alpaca_account(&account, account_id);

        assert!(state.is_margin_enabled());
        assert_eq!(state.equity.as_f64(), 10000.0);
    }

    #[rstest]
    fn test_position_state_long_short() {
        let venue = Venue::from("ALPACA");
        let symbol = Symbol::from("AAPL");
        let instrument_id = InstrumentId::new(symbol, venue);

        let position = AlpacaPosition {
            asset_id: "test".to_string(),
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            asset_class: "us_equity".to_string(),
            avg_entry_price: "150.00".to_string(),
            qty: "10".to_string(),
            qty_available: "10".to_string(),
            side: "long".to_string(),
            market_value: "1500.00".to_string(),
            cost_basis: "1500.00".to_string(),
            unrealized_pl: "0.00".to_string(),
            unrealized_plpc: "0.00".to_string(),
            unrealized_intraday_pl: "0.00".to_string(),
            unrealized_intraday_plpc: "0.00".to_string(),
            current_price: "150.00".to_string(),
            lastday_price: "150.00".to_string(),
            change_today: "0.00".to_string(),
        };

        let state = PositionState::from_alpaca_position(&position, instrument_id);
        assert!(state.is_long());
        assert!(!state.is_short());
    }
}
