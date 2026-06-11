//! Employee roster library: domain types and the ranking algorithm.
//!
//! This crate holds the reusable, testable logic. The binary (`main.rs`) is a
//! thin shell that loads sample data, wires up logging, and prints results.

pub mod calculator;

use std::cmp::Ordering;
use std::fmt;

use log::{debug, info};
use thiserror::Error;

pub use calculator::{CalcError, Calculator};

const SALARY_WEIGHT: f64 = 0.7;
const AGE_WEIGHT: f64 = 0.3;

const MAX_PLAUSIBLE_AGE: u8 = 120;

#[derive(Debug, Error)]
pub enum RosterError {
    #[error("employee `{name}` has an invalid salary: {salary} (must be finite and >= 0)")]
    InvalidSalary { name: String, salary: f64 },

    #[error("employee `{name}` has an invalid age: {age} (must be in 1..={max})")]
    InvalidAge { name: String, age: u8, max: u8 },

    #[error("cannot rank an empty roster")]
    EmptyRoster,

    #[error(transparent)]
    Calc(#[from] CalcError),
}

/// A single employee. Construct via [`Employee::new`] so invariants are enforced.
/// Fields are private; read them through the accessor methods.
#[derive(Debug, Clone, PartialEq)]
pub struct Employee {
    name: String,
    age: u8,
    salary: f64,
}

impl Employee {
    /// Build a validated `Employee`.
    ///
    /// # Errors
    /// Returns [`RosterError::InvalidSalary`] or [`RosterError::InvalidAge`] if
    /// the supplied values fall outside the accepted ranges.
    pub fn new(name: impl Into<String>, age: u8, salary: f64) -> Result<Self, RosterError> {
        let name = name.into();

        if !salary.is_finite() || salary < 0.0 {
            return Err(RosterError::InvalidSalary { name, salary });
        }
        if age == 0 || age > MAX_PLAUSIBLE_AGE {
            return Err(RosterError::InvalidAge {
                name,
                age,
                max: MAX_PLAUSIBLE_AGE,
            });
        }

        debug!("validated employee: name={name}, age={age}, salary={salary:.2}");
        Ok(Self { name, age, salary })
    }

    /// The employee's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The employee's age in years.
    pub fn age(&self) -> u8 {
        self.age
    }

    /// The employee's salary.
    pub fn salary(&self) -> f64 {
        self.salary
    }
}

impl fmt::Display for Employee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<12} age={:>3}  salary={:>12.2}",
            self.name, self.age, self.salary
        )
    }
}

/// Inclusive min/max of a series of `f64` values, returned as `(min, max)`.
/// Returns `None` for an empty iterator.
fn min_max<I: IntoIterator<Item = f64>>(values: I) -> Option<(f64, f64)> {
    values.into_iter().fold(None, |acc, v| match acc {
        None => Some((v, v)),
        Some((lo, hi)) => Some((lo.min(v), hi.max(v))),
    })
}

/// Min-max normalize `value` into `[0.0, 1.0]`.
///
/// When `lo == hi` (no spread in the data) every item maps to `0.5` so that the
/// dimension contributes neutrally instead of blowing up via divide-by-zero.
fn normalize(value: f64, lo: f64, hi: f64) -> f64 {
    if (hi - lo).abs() < f64::EPSILON {
        0.5
    } else {
        (value - lo) / (hi - lo)
    }
}

/// Combined, normalized salary+age score for one employee. Higher is "ranked
/// first". Both dimensions are normalized across the whole roster first so they
/// are comparable despite living on very different scales.
fn combined_score(emp: &Employee, salary_bounds: (f64, f64), age_bounds: (f64, f64)) -> f64 {
    let salary_norm = normalize(emp.salary, salary_bounds.0, salary_bounds.1);
    let age_norm = normalize(emp.age as f64, age_bounds.0, age_bounds.1);
    SALARY_WEIGHT * salary_norm + AGE_WEIGHT * age_norm
}

/// Rank employees by descending combined score (salary weighted more heavily
/// than age). Returns a new sorted `Vec`; the input is left untouched.
///
/// # Errors
/// Returns [`RosterError::EmptyRoster`] if `employees` is empty.
pub fn rank_by_combined_score(employees: &[Employee]) -> Result<Vec<Employee>, RosterError> {
    if employees.is_empty() {
        return Err(RosterError::EmptyRoster);
    }

    let salary_bounds =
        min_max(employees.iter().map(|e| e.salary)).ok_or(RosterError::EmptyRoster)?;
    let age_bounds =
        min_max(employees.iter().map(|e| e.age as f64)).ok_or(RosterError::EmptyRoster)?;

    info!(
        "ranking {} employees (salary bounds={:?}, age bounds={:?})",
        employees.len(),
        salary_bounds,
        age_bounds
    );

    let mut ranked = employees.to_vec();
    ranked.sort_by(|a, b| {
        let sa = combined_score(a, salary_bounds, age_bounds);
        let sb = combined_score(b, salary_bounds, age_bounds);
        // Descending by score; `total_cmp` is NaN-safe (scores are finite, but
        // this keeps the comparator total and panic-free regardless).
        match sb.total_cmp(&sa) {
            Ordering::Equal => a.name.cmp(&b.name), // stable, deterministic tiebreak
            other => other,
        }
    });

    Ok(ranked)
}

#[cfg(test)]
mod tests {
    //! Unit tests for *private* internals only. Public-API behavior is exercised
    //! by the integration tests under `tests/`.
    use super::*;

    #[test]
    fn normalize_handles_no_spread() {
        assert_eq!(normalize(5.0, 5.0, 5.0), 0.5);
    }

    #[test]
    fn min_max_of_empty_is_none() {
        assert_eq!(min_max(std::iter::empty()), None);
    }

    #[test]
    fn min_max_finds_bounds() {
        assert_eq!(min_max([3.0, 1.0, 2.0]), Some((1.0, 3.0)));
    }
}