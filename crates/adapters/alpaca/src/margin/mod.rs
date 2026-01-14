// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2025 Andrew Crum. All rights reserved.
//  https://nautechsystems.io
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

//! Alpaca options margin calculation using the Universal Spread Rule.
//!
//! This module provides margin calculations for options positions and multi-leg
//! orders following Alpaca's Universal Spread Rule methodology.
//!
//! The Universal Spread Rule calculates maintenance margin by:
//! 1. Modeling each option position as a piecewise linear payoff function
//! 2. Evaluating the portfolio payoff at all strike prices (inflection points)
//! 3. Finding the theoretical maximum loss across all evaluation points
//! 4. Using that maximum loss as the maintenance margin requirement
//!
//! This approach correctly handles complex spreads and offsetting positions that
//! traditional per-leg margin calculations would over-estimate.
//!
//! # Examples
//!
//! ```rust
//! use rust_decimal::Decimal;
//! use nautilus_alpaca::margin::{AlpacaOptionsMarginCalculator, OptionPosition};
//!
//! let calculator = AlpacaOptionsMarginCalculator::default();
//!
//! // For existing positions
//! let positions = vec![
//!     OptionPosition {
//!         strike: Decimal::from(100),
//!         is_call: true,
//!         is_long: true,
//!         quantity: 1,
//!         expiration: None,
//!     },
//!     OptionPosition {
//!         strike: Decimal::from(110),
//!         is_call: true,
//!         is_long: false,
//!         quantity: 1,
//!         expiration: None,
//!     },
//! ];
//!
//! let margin = calculator.calculate_maintenance_margin(&positions, None);
//! ```

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The default contract multiplier for options (100 shares per contract).
const DEFAULT_CONTRACT_MULTIPLIER: u32 = 100;

/// Represents an option position for margin calculation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionPosition {
    /// The option strike price.
    pub strike: Decimal,
    /// True for call options, False for put options.
    pub is_call: bool,
    /// True for long positions, False for short positions.
    pub is_long: bool,
    /// Number of contracts (always positive).
    pub quantity: i64,
    /// Optional expiration date for grouping (using string for simplicity).
    pub expiration: Option<String>,
}

/// Represents an order leg for cost basis calculation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderLeg {
    /// The option strike price.
    pub strike: Decimal,
    /// True for call options, False for put options.
    pub is_call: bool,
    /// Order side: "buy" or "sell".
    pub side: String,
    /// Quantity ratio for this leg.
    pub ratio_qty: i64,
}

/// Result of order cost basis calculation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostBasisResult {
    /// The maintenance margin requirement.
    pub maintenance_margin: Decimal,
    /// Net premium (positive=debit, negative=credit).
    pub net_premium: Decimal,
    /// Total cost basis.
    pub cost_basis: Decimal,
}

/// Result of position margin validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarginValidationResult {
    /// Whether the new position is within margin limits.
    pub is_valid: bool,
    /// The total margin with the new position.
    pub new_margin: Decimal,
    /// Description of the validation result.
    pub message: String,
}

/// Calculator for Alpaca options margin using the Universal Spread Rule.
///
/// The Universal Spread Rule evaluates option positions using piecewise linear
/// payoff functions to find the theoretical maximum loss, which becomes the
/// maintenance margin requirement.
///
/// This calculator can be used for:
/// - Pre-trade margin validation in the execution client
/// - Real-time risk monitoring of option positions
/// - Multi-leg order cost basis calculation
#[derive(Debug, Clone)]
pub struct AlpacaOptionsMarginCalculator {
    /// The default contract multiplier for options (100 shares per contract).
    default_contract_multiplier: u32,
}

impl Default for AlpacaOptionsMarginCalculator {
    fn default() -> Self {
        Self::new(DEFAULT_CONTRACT_MULTIPLIER)
    }
}

impl AlpacaOptionsMarginCalculator {
    /// Creates a new margin calculator with the specified contract multiplier.
    ///
    /// # Parameters
    ///
    /// * `default_contract_multiplier` - The default contract multiplier for options (typically 100).
    #[must_use]
    pub const fn new(default_contract_multiplier: u32) -> Self {
        Self {
            default_contract_multiplier,
        }
    }

    /// Calculates the payoff of an option position at a given underlying price.
    ///
    /// Uses piecewise linear payoff functions:
    /// - Long Call:  max(0, P - strike)
    /// - Short Call: -max(0, P - strike)
    /// - Long Put:   max(0, strike - P)
    /// - Short Put:  -max(0, strike - P)
    ///
    /// # Parameters
    ///
    /// * `strike` - The option strike price.
    /// * `is_call` - True for call options, False for put options.
    /// * `is_long` - True for long positions, False for short positions.
    /// * `quantity` - Number of contracts (always positive).
    /// * `underlying_price` - The underlying price at which to evaluate the payoff.
    ///
    /// # Returns
    ///
    /// The payoff value (positive = profit, negative = loss).
    #[must_use]
    pub fn calculate_payoff(
        strike: Decimal,
        is_call: bool,
        is_long: bool,
        quantity: i64,
        underlying_price: Decimal,
    ) -> Decimal {
        let intrinsic = if is_call {
            (underlying_price - strike).max(Decimal::ZERO)
        } else {
            (strike - underlying_price).max(Decimal::ZERO)
        };

        let quantity_decimal = Decimal::from(quantity);
        if is_long {
            intrinsic * quantity_decimal
        } else {
            -intrinsic * quantity_decimal
        }
    }

    /// Calculates maintenance margin for a portfolio of option positions.
    ///
    /// Uses the Universal Spread Rule: evaluate the portfolio payoff at all
    /// strike prices (inflection points) to find the theoretical maximum loss.
    ///
    /// # Parameters
    ///
    /// * `positions` - List of option positions.
    /// * `contract_multiplier` - The contract multiplier (default: 100).
    ///
    /// # Returns
    ///
    /// The maintenance margin requirement (always non-negative).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_decimal::Decimal;
    /// use nautilus_alpaca::margin::{AlpacaOptionsMarginCalculator, OptionPosition};
    ///
    /// let calculator = AlpacaOptionsMarginCalculator::default();
    /// let positions = vec![
    ///     OptionPosition {
    ///         strike: Decimal::from(100),
    ///         is_call: true,
    ///         is_long: true,
    ///         quantity: 1,
    ///         expiration: None,
    ///     },
    ///     OptionPosition {
    ///         strike: Decimal::from(110),
    ///         is_call: true,
    ///         is_long: false,
    ///         quantity: 1,
    ///         expiration: None,
    ///     },
    /// ];
    /// let margin = calculator.calculate_maintenance_margin(&positions, None);
    /// assert_eq!(margin, Decimal::from(1000)); // 10 point spread * 100 multiplier
    /// ```
    #[must_use]
    pub fn calculate_maintenance_margin(
        &self,
        positions: &[OptionPosition],
        contract_multiplier: Option<u32>,
    ) -> Decimal {
        if positions.is_empty() {
            return Decimal::ZERO;
        }

        let multiplier = contract_multiplier.unwrap_or(self.default_contract_multiplier);

        // Group positions by expiration if available
        let mut expiry_groups: HashMap<String, Vec<&OptionPosition>> = HashMap::new();
        for pos in positions {
            let expiry = pos.expiration.clone().unwrap_or_else(|| "default".to_string());
            expiry_groups.entry(expiry).or_default().push(pos);
        }

        // Calculate margin for each expiry group and sum
        let mut total_margin = Decimal::ZERO;
        for group_positions in expiry_groups.values() {
            let group_margin = self.calculate_margin_for_group(group_positions, multiplier);
            total_margin += group_margin;
        }

        total_margin
    }

    /// Calculates margin for a group of positions with the same expiration.
    ///
    /// # Parameters
    ///
    /// * `positions` - Positions in this expiration group.
    /// * `contract_multiplier` - The contract multiplier.
    ///
    /// # Returns
    ///
    /// The margin requirement for this group.
    fn calculate_margin_for_group(
        &self,
        positions: &[&OptionPosition],
        contract_multiplier: u32,
    ) -> Decimal {
        if positions.is_empty() {
            return Decimal::ZERO;
        }

        // Collect all strike prices as evaluation points (inflection points)
        let mut strikes = BTreeSet::new();
        for pos in positions {
            strikes.insert(pos.strike);
        }

        if strikes.is_empty() {
            return Decimal::ZERO;
        }

        // Add boundary evaluation points
        let max_strike = *strikes.iter().max().expect("strikes should not be empty");
        let mut evaluation_points: Vec<Decimal> = strikes.into_iter().collect();

        // Add point below minimum strike (where puts have maximum value)
        evaluation_points.insert(0, Decimal::ZERO);

        // Add point above maximum strike (where calls continue linearly)
        evaluation_points.push(max_strike + Decimal::from(100));

        // Evaluate portfolio payoff at each point
        let mut min_payoff = Decimal::ZERO;
        for price in evaluation_points {
            let mut total_payoff = Decimal::ZERO;
            for pos in positions {
                let payoff = Self::calculate_payoff(
                    pos.strike,
                    pos.is_call,
                    pos.is_long,
                    pos.quantity,
                    price,
                );
                total_payoff += payoff;
            }

            if total_payoff < min_payoff {
                min_payoff = total_payoff;
            }
        }

        // Margin is the absolute value of the maximum loss, multiplied by contract size
        min_payoff.abs() * Decimal::from(contract_multiplier)
    }

    /// Calculates the cost basis for a multi-leg options order.
    ///
    /// Cost Basis = Maintenance Margin + Net Premium
    ///
    /// Where Net Premium is:
    /// - Positive for net debit (you pay to open)
    /// - Negative for net credit (you receive to open)
    ///
    /// # Parameters
    ///
    /// * `legs` - List of order legs.
    /// * `leg_premiums` - Premium for each leg (positive values).
    /// * `contract_multiplier` - The contract multiplier (default: 100).
    ///
    /// # Returns
    ///
    /// A `CostBasisResult` containing maintenance margin, net premium, and cost basis.
    ///
    /// # Errors
    ///
    /// Returns an error if the number of legs doesn't match the number of premiums.
    pub fn calculate_order_cost_basis(
        &self,
        legs: &[OrderLeg],
        leg_premiums: &[Decimal],
        contract_multiplier: Option<u32>,
    ) -> Result<CostBasisResult, String> {
        if legs.is_empty() {
            return Ok(CostBasisResult {
                maintenance_margin: Decimal::ZERO,
                net_premium: Decimal::ZERO,
                cost_basis: Decimal::ZERO,
            });
        }

        if legs.len() != leg_premiums.len() {
            return Err(format!(
                "Number of legs ({}) must match number of premiums ({})",
                legs.len(),
                leg_premiums.len()
            ));
        }

        let multiplier = contract_multiplier.unwrap_or(self.default_contract_multiplier);

        // Convert legs to position format for margin calculation
        let mut positions = Vec::new();
        let mut net_premium = Decimal::ZERO;

        for (leg, &premium) in legs.iter().zip(leg_premiums.iter()) {
            let is_long = leg.side.to_lowercase() == "buy";
            let qty = leg.ratio_qty;

            positions.push(OptionPosition {
                strike: leg.strike,
                is_call: leg.is_call,
                is_long,
                quantity: qty,
                expiration: None,
            });

            // Calculate net premium
            let premium_amount = premium * Decimal::from(qty) * Decimal::from(multiplier);
            if is_long {
                net_premium += premium_amount; // Buying = paying premium
            } else {
                net_premium -= premium_amount; // Selling = receiving premium
            }
        }

        // Calculate maintenance margin using universal spread rule
        let maintenance_margin =
            self.calculate_maintenance_margin(&positions, Some(multiplier));

        // Cost basis is margin plus net premium (if debit)
        let cost_basis = maintenance_margin + net_premium.max(Decimal::ZERO);

        Ok(CostBasisResult {
            maintenance_margin,
            net_premium,
            cost_basis,
        })
    }

    /// Validates if adding a new position would exceed available margin.
    ///
    /// # Parameters
    ///
    /// * `current_positions` - Current option positions.
    /// * `new_position` - The new position to add.
    /// * `available_margin` - Available margin in the account.
    /// * `contract_multiplier` - The contract multiplier.
    ///
    /// # Returns
    ///
    /// A `MarginValidationResult` containing:
    /// - `is_valid`: Whether the new position is within margin limits
    /// - `new_margin`: The total margin with the new position
    /// - `message`: Description of the validation result
    #[must_use]
    pub fn validate_position_margin(
        &self,
        current_positions: &[OptionPosition],
        new_position: &OptionPosition,
        available_margin: Decimal,
        contract_multiplier: Option<u32>,
    ) -> MarginValidationResult {
        // Calculate current margin
        let current_margin = self.calculate_maintenance_margin(current_positions, contract_multiplier);

        // Calculate margin with new position
        let mut all_positions = current_positions.to_vec();
        all_positions.push(new_position.clone());
        let new_margin = self.calculate_maintenance_margin(&all_positions, contract_multiplier);

        // Check if within limits
        let margin_delta = new_margin - current_margin;
        let is_valid = new_margin <= available_margin;

        let message = if is_valid {
            match margin_delta.cmp(&Decimal::ZERO) {
                Ordering::Less | Ordering::Equal => {
                    format!(
                        "Position reduces margin requirement by ${:.2} (new total: ${:.2})",
                        margin_delta.abs(),
                        new_margin
                    )
                }
                Ordering::Greater => {
                    format!(
                        "Position increases margin by ${:.2} (new total: ${:.2}, available: ${:.2})",
                        margin_delta, new_margin, available_margin
                    )
                }
            }
        } else {
            format!(
                "Insufficient margin: need ${:.2}, available ${:.2} (shortfall: ${:.2})",
                new_margin,
                available_margin,
                new_margin - available_margin
            )
        };

        MarginValidationResult {
            is_valid,
            new_margin,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[rstest]
    fn test_calculate_payoff_long_call() {
        // Long call: max(0, P - strike)
        let payoff = AlpacaOptionsMarginCalculator::calculate_payoff(
            dec!(100),
            true,  // is_call
            true,  // is_long
            1,     // quantity
            dec!(110),
        );
        assert_eq!(payoff, dec!(10));

        // Out of the money
        let payoff = AlpacaOptionsMarginCalculator::calculate_payoff(
            dec!(100),
            true,
            true,
            1,
            dec!(90),
        );
        assert_eq!(payoff, dec!(0));
    }

    #[rstest]
    fn test_calculate_payoff_short_call() {
        // Short call: -max(0, P - strike)
        let payoff = AlpacaOptionsMarginCalculator::calculate_payoff(
            dec!(100),
            true,  // is_call
            false, // is_long (short)
            1,
            dec!(110),
        );
        assert_eq!(payoff, dec!(-10));

        // Out of the money
        let payoff = AlpacaOptionsMarginCalculator::calculate_payoff(
            dec!(100),
            true,
            false,
            1,
            dec!(90),
        );
        assert_eq!(payoff, dec!(0));
    }

    #[rstest]
    fn test_calculate_payoff_long_put() {
        // Long put: max(0, strike - P)
        let payoff = AlpacaOptionsMarginCalculator::calculate_payoff(
            dec!(100),
            false, // is_call (put)
            true,  // is_long
            1,
            dec!(90),
        );
        assert_eq!(payoff, dec!(10));

        // Out of the money
        let payoff = AlpacaOptionsMarginCalculator::calculate_payoff(
            dec!(100),
            false,
            true,
            1,
            dec!(110),
        );
        assert_eq!(payoff, dec!(0));
    }

    #[rstest]
    fn test_calculate_payoff_short_put() {
        // Short put: -max(0, strike - P)
        let payoff = AlpacaOptionsMarginCalculator::calculate_payoff(
            dec!(100),
            false, // is_call (put)
            false, // is_long (short)
            1,
            dec!(90),
        );
        assert_eq!(payoff, dec!(-10));

        // Out of the money
        let payoff = AlpacaOptionsMarginCalculator::calculate_payoff(
            dec!(100),
            false,
            false,
            1,
            dec!(110),
        );
        assert_eq!(payoff, dec!(0));
    }

    #[rstest]
    fn test_bull_call_spread() {
        // Buy 100 call, sell 110 call
        // Max loss is the debit paid (premium)
        // Max gain at any price >= 110 is 10 points (1000 with multiplier)
        let calculator = AlpacaOptionsMarginCalculator::default();
        let positions = vec![
            OptionPosition {
                strike: dec!(100),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
            OptionPosition {
                strike: dec!(110),
                is_call: true,
                is_long: false,
                quantity: 1,
                expiration: None,
            },
        ];

        let margin = calculator.calculate_maintenance_margin(&positions, None);
        // Max loss is 10 points * 100 multiplier = 1000
        assert_eq!(margin, dec!(1000));
    }

    #[rstest]
    fn test_bear_put_spread() {
        // Buy 110 put, sell 100 put
        // Max loss is 10 points
        let calculator = AlpacaOptionsMarginCalculator::default();
        let positions = vec![
            OptionPosition {
                strike: dec!(110),
                is_call: false,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
            OptionPosition {
                strike: dec!(100),
                is_call: false,
                is_long: false,
                quantity: 1,
                expiration: None,
            },
        ];

        let margin = calculator.calculate_maintenance_margin(&positions, None);
        // Max loss is 10 points * 100 multiplier = 1000
        assert_eq!(margin, dec!(1000));
    }

    #[rstest]
    fn test_iron_condor() {
        // Sell 95 put, buy 90 put, sell 110 call, buy 115 call
        // Max loss is max of the two spreads = 5 points
        let calculator = AlpacaOptionsMarginCalculator::default();
        let positions = vec![
            // Bear put spread (sell 95 put, buy 90 put)
            OptionPosition {
                strike: dec!(95),
                is_call: false,
                is_long: false,
                quantity: 1,
                expiration: None,
            },
            OptionPosition {
                strike: dec!(90),
                is_call: false,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
            // Bear call spread (sell 110 call, buy 115 call)
            OptionPosition {
                strike: dec!(110),
                is_call: true,
                is_long: false,
                quantity: 1,
                expiration: None,
            },
            OptionPosition {
                strike: dec!(115),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
        ];

        let margin = calculator.calculate_maintenance_margin(&positions, None);
        // Max loss is 5 points * 100 multiplier = 500
        assert_eq!(margin, dec!(500));
    }

    #[rstest]
    fn test_butterfly_spread() {
        // Buy 1 100 call, sell 2 110 calls, buy 1 120 call
        // Max loss is at the wings (debit paid)
        // Max loss is 10 points
        let calculator = AlpacaOptionsMarginCalculator::default();
        let positions = vec![
            OptionPosition {
                strike: dec!(100),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
            OptionPosition {
                strike: dec!(110),
                is_call: true,
                is_long: false,
                quantity: 2,
                expiration: None,
            },
            OptionPosition {
                strike: dec!(120),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
        ];

        let margin = calculator.calculate_maintenance_margin(&positions, None);
        // Max loss is 10 points * 100 multiplier = 1000
        assert_eq!(margin, dec!(1000));
    }

    #[rstest]
    fn test_empty_positions() {
        let calculator = AlpacaOptionsMarginCalculator::default();
        let positions = vec![];
        let margin = calculator.calculate_maintenance_margin(&positions, None);
        assert_eq!(margin, dec!(0));
    }

    #[rstest]
    fn test_cost_basis_net_debit() {
        // Bull call spread with net debit
        let calculator = AlpacaOptionsMarginCalculator::default();
        let legs = vec![
            OrderLeg {
                strike: dec!(100),
                is_call: true,
                side: "buy".to_string(),
                ratio_qty: 1,
            },
            OrderLeg {
                strike: dec!(110),
                is_call: true,
                side: "sell".to_string(),
                ratio_qty: 1,
            },
        ];
        let premiums = vec![dec!(5), dec!(2)]; // Buy at 5, sell at 2 = 3 debit

        let result = calculator.calculate_order_cost_basis(&legs, &premiums, None).unwrap();

        // Maintenance margin = 1000 (max loss)
        assert_eq!(result.maintenance_margin, dec!(1000));
        // Net premium = (5 - 2) * 100 = 300 debit
        assert_eq!(result.net_premium, dec!(300));
        // Cost basis = margin + max(0, net_premium) = 1000 + 300 = 1300
        assert_eq!(result.cost_basis, dec!(1300));
    }

    #[rstest]
    fn test_cost_basis_net_credit() {
        // Bear call spread with net credit
        let calculator = AlpacaOptionsMarginCalculator::default();
        let legs = vec![
            OrderLeg {
                strike: dec!(110),
                is_call: true,
                side: "sell".to_string(),
                ratio_qty: 1,
            },
            OrderLeg {
                strike: dec!(120),
                is_call: true,
                side: "buy".to_string(),
                ratio_qty: 1,
            },
        ];
        let premiums = vec![dec!(5), dec!(2)]; // Sell at 5, buy at 2 = 3 credit

        let result = calculator.calculate_order_cost_basis(&legs, &premiums, None).unwrap();

        // Maintenance margin = 1000 (max loss is 10 points)
        assert_eq!(result.maintenance_margin, dec!(1000));
        // Net premium = -(5 - 2) * 100 = -300 credit
        assert_eq!(result.net_premium, dec!(-300));
        // Cost basis = margin + max(0, net_premium) = 1000 + 0 = 1000 (credit doesn't reduce cost basis)
        assert_eq!(result.cost_basis, dec!(1000));
    }

    #[rstest]
    fn test_validate_position_margin_sufficient() {
        let calculator = AlpacaOptionsMarginCalculator::default();
        let current_positions = vec![
            OptionPosition {
                strike: dec!(100),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
        ];
        let new_position = OptionPosition {
            strike: dec!(110),
            is_call: true,
            is_long: false,
            quantity: 1,
            expiration: None,
        };

        let result = calculator.validate_position_margin(
            &current_positions,
            &new_position,
            dec!(5000),
            None,
        );

        assert!(result.is_valid);
        assert_eq!(result.new_margin, dec!(1000));
        assert!(result.message.contains("reduces margin"));
    }

    #[rstest]
    fn test_validate_position_margin_insufficient() {
        let calculator = AlpacaOptionsMarginCalculator::default();
        let current_positions = vec![
            OptionPosition {
                strike: dec!(100),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
        ];
        let new_position = OptionPosition {
            strike: dec!(110),
            is_call: true,
            is_long: false,
            quantity: 1,
            expiration: None,
        };

        let result = calculator.validate_position_margin(
            &current_positions,
            &new_position,
            dec!(500), // Insufficient margin
            None,
        );

        assert!(!result.is_valid);
        assert_eq!(result.new_margin, dec!(1000));
        assert!(result.message.contains("Insufficient margin"));
    }

    #[rstest]
    fn test_legs_premiums_mismatch() {
        let calculator = AlpacaOptionsMarginCalculator::default();
        let legs = vec![
            OrderLeg {
                strike: dec!(100),
                is_call: true,
                side: "buy".to_string(),
                ratio_qty: 1,
            },
        ];
        let premiums = vec![dec!(5), dec!(2)]; // Mismatch: 1 leg, 2 premiums

        let result = calculator.calculate_order_cost_basis(&legs, &premiums, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must match"));
    }

    #[rstest]
    fn test_custom_contract_multiplier() {
        let calculator = AlpacaOptionsMarginCalculator::new(10); // Non-standard multiplier
        let positions = vec![
            OptionPosition {
                strike: dec!(100),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: None,
            },
            OptionPosition {
                strike: dec!(110),
                is_call: true,
                is_long: false,
                quantity: 1,
                expiration: None,
            },
        ];

        let margin = calculator.calculate_maintenance_margin(&positions, None);
        // Max loss is 10 points * 10 multiplier = 100
        assert_eq!(margin, dec!(100));
    }

    #[rstest]
    fn test_expiration_grouping() {
        let calculator = AlpacaOptionsMarginCalculator::default();
        let positions = vec![
            // Group 1: June expiration
            OptionPosition {
                strike: dec!(100),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: Some("2025-06-20".to_string()),
            },
            OptionPosition {
                strike: dec!(110),
                is_call: true,
                is_long: false,
                quantity: 1,
                expiration: Some("2025-06-20".to_string()),
            },
            // Group 2: July expiration
            OptionPosition {
                strike: dec!(100),
                is_call: true,
                is_long: true,
                quantity: 1,
                expiration: Some("2025-07-18".to_string()),
            },
            OptionPosition {
                strike: dec!(110),
                is_call: true,
                is_long: false,
                quantity: 1,
                expiration: Some("2025-07-18".to_string()),
            },
        ];

        let margin = calculator.calculate_maintenance_margin(&positions, None);
        // Two identical bull call spreads, each with 1000 margin = 2000 total
        assert_eq!(margin, dec!(2000));
    }
}
