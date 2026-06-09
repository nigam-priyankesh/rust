//! Integration tests for the public roster API (`Employee`, `RosterError`,
//! `rank_by_combined_score`). Private helpers like `normalize` are unit-tested
//! inline in `src/lib.rs` and are intentionally not reachable from here.

use untitled::{Employee, RosterError, rank_by_combined_score};

#[test]
fn rejects_invalid_salary() {
    let err = Employee::new("X", 30, f64::NAN).unwrap_err();
    assert!(matches!(err, RosterError::InvalidSalary { .. }));
}

#[test]
fn rejects_zero_age() {
    let err = Employee::new("X", 0, 1.0).unwrap_err();
    assert!(matches!(err, RosterError::InvalidAge { .. }));
}

#[test]
fn empty_roster_is_an_error() {
    assert!(matches!(
        rank_by_combined_score(&[]),
        Err(RosterError::EmptyRoster)
    ));
}

#[test]
fn higher_salary_ranks_first_when_age_equal() {
    let low = Employee::new("Low", 40, 50_000.0).unwrap();
    let high = Employee::new("High", 40, 90_000.0).unwrap();
    let ranked = rank_by_combined_score(&[low, high]).unwrap();
    assert_eq!(ranked[0].name(), "High");
}