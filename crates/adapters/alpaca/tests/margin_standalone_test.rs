// Standalone test to verify margin module compiles and works
use rust_decimal_macros::dec;

// Import just what we need from the margin module
use nautilus_alpaca::margin::{AlpacaOptionsMarginCalculator, OptionPosition, OrderLeg};

#[test]
fn test_margin_calculator_basic() {
    let calculator = AlpacaOptionsMarginCalculator::default();

    // Test bull call spread
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
    assert_eq!(margin, dec!(1000));
}

#[test]
fn test_margin_calculator_iron_condor() {
    let calculator = AlpacaOptionsMarginCalculator::default();

    let positions = vec![
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
    assert_eq!(margin, dec!(500));
}

#[test]
fn test_cost_basis_calculation() {
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
    let premiums = vec![dec!(5), dec!(2)];

    let result = calculator
        .calculate_order_cost_basis(&legs, &premiums, None)
        .unwrap();

    assert_eq!(result.maintenance_margin, dec!(1000));
    assert_eq!(result.net_premium, dec!(300));
    assert_eq!(result.cost_basis, dec!(1300));
}
