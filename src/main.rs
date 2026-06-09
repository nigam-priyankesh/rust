//! Employee roster: load employees, validate them, and rank them by a combined
//! salary + age score.
//!
//! Production concerns addressed here:
//!   * Structured logging via the `log` facade + `env_logger` backend.
//!   * Explicit, typed error handling via `thiserror` (no `unwrap`/`panic` on
//!     recoverable paths).
//!   * Input validation at construction time so invalid state is unrepresentable.
//!
//! Run with logging:
//!   PowerShell:  $env:RUST_LOG="debug"; cargo run
//!   bash:        RUST_LOG=debug cargo run
//!
//! Debugging: build in the (default) dev profile, which carries full debug info.
//! Set breakpoints in `rank_by_combined_score` or `Employee::new` and launch the
//! "Run" configuration in RustRover with the debugger (the bug icon).

mod calculator;

use std::cmp::Ordering;
use std::fmt;

use log::{debug, error, info, warn};
use thiserror::Error;

use calculator::Calculator;

/// Relative importance of salary vs. age when computing the combined score.
/// They do not need to sum to 1.0, but keeping them normalized makes the
/// resulting score easy to reason about.
const SALARY_WEIGHT: f64 = 0.7;
const AGE_WEIGHT: f64 = 0.3;

/// Sane upper bound used purely for sanity-checking input data.
const MAX_PLAUSIBLE_AGE: u8 = 120;

/// Errors that can arise while building or processing the employee roster.
#[derive(Debug, Error)]
enum RosterError {
    #[error("employee `{name}` has an invalid salary: {salary} (must be finite and >= 0)")]
    InvalidSalary { name: String, salary: f64 },

    #[error("employee `{name}` has an invalid age: {age} (must be in 1..={max})")]
    InvalidAge { name: String, age: u8, max: u8 },

    #[error("cannot rank an empty roster")]
    EmptyRoster,

    #[error(transparent)]
    Calc(#[from] calculator::CalcError),
}

/// A single employee. Construct via [`Employee::new`] so invariants are enforced.
#[derive(Debug, Clone, PartialEq)]
struct Employee {
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
    fn new(name: impl Into<String>, age: u8, salary: f64) -> Result<Self, RosterError> {
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
fn rank_by_combined_score(employees: &[Employee]) -> Result<Vec<Employee>, RosterError> {
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

/// Build the sample roster. In a real service this would come from a DB or API;
/// invalid rows are logged and skipped rather than aborting the whole load.
fn build_sample_roster() -> Vec<Employee> {
    let raw = [
        ("Alice", 34u8, 95_000.0),
        ("Bob", 45, 72_000.0),
        ("Carol", 29, 110_000.0),
        ("Dave", 52, 88_000.0),
        ("Erin", 38, 88_000.0),
        ("Frank", 0, 50_000.0), // invalid age -> skipped + warned
        ("Grace", 41, -10.0),   // invalid salary -> skipped + warned
    ];

    raw.into_iter()
        .filter_map(
            |(name, age, salary)| match Employee::new(name, age, salary) {
                Ok(emp) => Some(emp),
                Err(e) => {
                    warn!("skipping invalid record: {e}");
                    None
                }
            },
        )
        .collect()
}

/// Real work lives here so `main` stays a thin shell that maps errors to a
/// process exit code.
fn run() -> Result<(), RosterError> {
    let calc = Calculator::new();
    let (a, b) = (7, 35);
    let sum = calc.add(a, b).map_err(RosterError::Calc)?;
    println!("{a} + {b} = {sum}");

    let roster = build_sample_roster();
    info!("loaded {} valid employee(s)", roster.len());

    let ranked = rank_by_combined_score(&roster)?;

    println!("\nEmployees ranked by combined salary+age score (best first):");
    println!("{:-<60}", "");
    for (rank, emp) in ranked.iter().enumerate() {
        println!("{:>2}. {emp}", rank + 1);
    }
    println!("{:-<60}", "");

    Ok(())
}


fn main() {
    // Default to `info` if RUST_LOG is unset so the program is useful out of the
    // box; override via the RUST_LOG env var.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("employee-roster starting up");

    if let Err(e) = run() {
        error!("fatal: {e}");
        std::process::exit(1);
    }

    info!("done");
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(ranked[0].name, "High");
    }

    #[test]
    fn normalize_handles_no_spread() {
        assert_eq!(normalize(5.0, 5.0, 5.0), 0.5);
    }
}
