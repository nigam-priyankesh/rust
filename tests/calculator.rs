//! Integration tests for the public `Calculator` API.
//!
//! This file is compiled as its own crate and can only see the library's
//! *public* surface, exactly like a real downstream consumer would.

use untitled::{CalcError, Calculator};

#[test]
fn adds_two_positive_numbers() {
    let calc = Calculator::new();
    assert_eq!(calc.add(2, 3), Ok(5));
}

#[test]
fn adds_negative_numbers() {
    let calc = Calculator::new();
    assert_eq!(calc.add(-4, 1), Ok(-3));
}

#[test]
fn reports_overflow() {
    let calc = Calculator::new();
    assert_eq!(
        calc.add(i64::MAX, 1),
        Err(CalcError::Overflow { a: i64::MAX, b: 1 })
    );
}