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

//! Python bindings for Alpaca options margin calculator.

use pyo3::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::margin::{
    AlpacaOptionsMarginCalculator as RustMarginCalculator,
    CostBasisResult as RustCostBasisResult,
    MarginValidationResult as RustMarginValidationResult,
    OptionPosition as RustOptionPosition,
    OrderLeg as RustOrderLeg,
};

/// Python wrapper for OptionPosition
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.alpaca")]
#[derive(Clone, Debug)]
pub struct OptionPosition {
    inner: RustOptionPosition,
}

#[pymethods]
impl OptionPosition {
    #[new]
    #[pyo3(signature = (strike, is_call, is_long, quantity, expiration=None))]
    fn new(
        strike: &str,
        is_call: bool,
        is_long: bool,
        quantity: i64,
        expiration: Option<String>,
    ) -> PyResult<Self> {
        let strike_decimal = Decimal::from_str(strike)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid strike price: {}", e)
            ))?;

        Ok(Self {
            inner: RustOptionPosition {
                strike: strike_decimal,
                is_call,
                is_long,
                quantity,
                expiration,
            },
        })
    }

    #[getter]
    fn strike(&self) -> String {
        self.inner.strike.to_string()
    }

    #[getter]
    fn is_call(&self) -> bool {
        self.inner.is_call
    }

    #[getter]
    fn is_long(&self) -> bool {
        self.inner.is_long
    }

    #[getter]
    fn quantity(&self) -> i64 {
        self.inner.quantity
    }

    #[getter]
    fn expiration(&self) -> Option<String> {
        self.inner.expiration.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "OptionPosition(strike={}, is_call={}, is_long={}, quantity={}, expiration={:?})",
            self.inner.strike,
            self.inner.is_call,
            self.inner.is_long,
            self.inner.quantity,
            self.inner.expiration
        )
    }
}

/// Python wrapper for OrderLeg
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.alpaca")]
#[derive(Clone, Debug)]
pub struct OrderLeg {
    inner: RustOrderLeg,
}

#[pymethods]
impl OrderLeg {
    #[new]
    fn new(
        strike: &str,
        is_call: bool,
        side: String,
        ratio_qty: i64,
    ) -> PyResult<Self> {
        let strike_decimal = Decimal::from_str(strike)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid strike price: {}", e)
            ))?;

        Ok(Self {
            inner: RustOrderLeg {
                strike: strike_decimal,
                is_call,
                side,
                ratio_qty,
            },
        })
    }

    #[getter]
    fn strike(&self) -> String {
        self.inner.strike.to_string()
    }

    #[getter]
    fn is_call(&self) -> bool {
        self.inner.is_call
    }

    #[getter]
    fn side(&self) -> String {
        self.inner.side.clone()
    }

    #[getter]
    fn ratio_qty(&self) -> i64 {
        self.inner.ratio_qty
    }

    fn __repr__(&self) -> String {
        format!(
            "OrderLeg(strike={}, is_call={}, side={}, ratio_qty={})",
            self.inner.strike,
            self.inner.is_call,
            self.inner.side,
            self.inner.ratio_qty
        )
    }
}

/// Python wrapper for CostBasisResult
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.alpaca")]
#[derive(Clone, Debug)]
pub struct CostBasisResult {
    inner: RustCostBasisResult,
}

#[pymethods]
impl CostBasisResult {
    #[getter]
    fn maintenance_margin(&self) -> String {
        self.inner.maintenance_margin.to_string()
    }

    #[getter]
    fn net_premium(&self) -> String {
        self.inner.net_premium.to_string()
    }

    #[getter]
    fn cost_basis(&self) -> String {
        self.inner.cost_basis.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "CostBasisResult(maintenance_margin={}, net_premium={}, cost_basis={})",
            self.inner.maintenance_margin,
            self.inner.net_premium,
            self.inner.cost_basis
        )
    }
}

/// Python wrapper for MarginValidationResult
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.alpaca")]
#[derive(Clone, Debug)]
pub struct MarginValidationResult {
    inner: RustMarginValidationResult,
}

#[pymethods]
impl MarginValidationResult {
    #[getter]
    fn is_valid(&self) -> bool {
        self.inner.is_valid
    }

    #[getter]
    fn new_margin(&self) -> String {
        self.inner.new_margin.to_string()
    }

    #[getter]
    fn message(&self) -> String {
        self.inner.message.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "MarginValidationResult(is_valid={}, new_margin={}, message={})",
            self.inner.is_valid,
            self.inner.new_margin,
            self.inner.message
        )
    }
}

/// Alpaca options margin calculator using the Universal Spread Rule.
///
/// The Universal Spread Rule calculates maintenance margin by:
/// 1. Modeling each option position as a piecewise linear payoff function
/// 2. Evaluating the portfolio payoff at all strike prices (inflection points)
/// 3. Finding the theoretical maximum loss across all evaluation points
/// 4. Using that maximum loss as the maintenance margin requirement
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.alpaca")]
#[derive(Debug)]
pub struct AlpacaOptionsMarginCalculator {
    inner: RustMarginCalculator,
}

#[pymethods]
impl AlpacaOptionsMarginCalculator {
    #[new]
    #[pyo3(signature = (default_contract_multiplier=100))]
    fn new(default_contract_multiplier: u32) -> Self {
        Self {
            inner: RustMarginCalculator::new(default_contract_multiplier),
        }
    }

    /// Calculate option payoff at a given underlying price.
    ///
    /// Parameters
    /// ----------
    /// strike : str
    ///     The option strike price (as decimal string)
    /// is_call : bool
    ///     True for call options, False for put options
    /// is_long : bool
    ///     True for long positions, False for short positions
    /// quantity : int
    ///     Number of contracts (always positive)
    /// underlying_price : str
    ///     The underlying price at which to evaluate payoff (as decimal string)
    ///
    /// Returns
    /// -------
    /// str
    ///     The payoff value as decimal string (positive = profit, negative = loss)
    #[staticmethod]
    fn calculate_payoff(
        strike: &str,
        is_call: bool,
        is_long: bool,
        quantity: i64,
        underlying_price: &str,
    ) -> PyResult<String> {
        let strike_decimal = Decimal::from_str(strike)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid strike price: {}", e)
            ))?;
        let price_decimal = Decimal::from_str(underlying_price)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid underlying price: {}", e)
            ))?;

        let payoff = RustMarginCalculator::calculate_payoff(
            strike_decimal,
            is_call,
            is_long,
            quantity,
            price_decimal,
        );

        Ok(payoff.to_string())
    }

    /// Calculate maintenance margin for a portfolio of option positions.
    ///
    /// Parameters
    /// ----------
    /// positions : list[OptionPosition]
    ///     List of option positions
    /// contract_multiplier : int, optional
    ///     The contract multiplier (default: 100)
    ///
    /// Returns
    /// -------
    /// str
    ///     The maintenance margin requirement as decimal string
    fn calculate_maintenance_margin(
        &self,
        positions: Vec<PyRef<OptionPosition>>,
        contract_multiplier: Option<u32>,
    ) -> String {
        let rust_positions: Vec<RustOptionPosition> = positions
            .iter()
            .map(|p| p.inner.clone())
            .collect();

        let margin = self.inner.calculate_maintenance_margin(&rust_positions, contract_multiplier);
        margin.to_string()
    }

    /// Calculate cost basis for a multi-leg options order.
    ///
    /// Cost Basis = Maintenance Margin + Net Premium (if debit)
    ///
    /// Parameters
    /// ----------
    /// legs : list[OrderLeg]
    ///     List of order legs
    /// leg_premiums : list[str]
    ///     Premium for each leg as decimal strings (positive values)
    /// contract_multiplier : int, optional
    ///     The contract multiplier (default: 100)
    ///
    /// Returns
    /// -------
    /// CostBasisResult
    ///     Result containing maintenance margin, net premium, and cost basis
    fn calculate_order_cost_basis(
        &self,
        legs: Vec<PyRef<OrderLeg>>,
        leg_premiums: Vec<String>,
        contract_multiplier: Option<u32>,
    ) -> PyResult<CostBasisResult> {
        let rust_legs: Vec<RustOrderLeg> = legs
            .iter()
            .map(|l| l.inner.clone())
            .collect();

        let premiums: Result<Vec<Decimal>, _> = leg_premiums
            .iter()
            .map(|p| Decimal::from_str(p))
            .collect();

        let premiums = premiums.map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid premium value: {}", e)
            )
        })?;

        let result = self.inner
            .calculate_order_cost_basis(&rust_legs, &premiums, contract_multiplier)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

        Ok(CostBasisResult { inner: result })
    }

    /// Validate if adding a new position would exceed available margin.
    ///
    /// Parameters
    /// ----------
    /// current_positions : list[OptionPosition]
    ///     Current option positions
    /// new_position : OptionPosition
    ///     The new position to add
    /// available_margin : str
    ///     Available margin in the account as decimal string
    /// contract_multiplier : int, optional
    ///     The contract multiplier (default: 100)
    ///
    /// Returns
    /// -------
    /// MarginValidationResult
    ///     Validation result with is_valid flag, new margin, and message
    fn validate_position_margin(
        &self,
        current_positions: Vec<PyRef<OptionPosition>>,
        new_position: PyRef<OptionPosition>,
        available_margin: &str,
        contract_multiplier: Option<u32>,
    ) -> PyResult<MarginValidationResult> {
        let rust_positions: Vec<RustOptionPosition> = current_positions
            .iter()
            .map(|p| p.inner.clone())
            .collect();

        let available = Decimal::from_str(available_margin)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid available margin: {}", e)
            ))?;

        let result = self.inner.validate_position_margin(
            &rust_positions,
            &new_position.inner,
            available,
            contract_multiplier,
        );

        Ok(MarginValidationResult { inner: result })
    }

    fn __repr__(&self) -> String {
        "AlpacaOptionsMarginCalculator()".to_string()
    }
}
