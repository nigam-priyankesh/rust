//! Runnable answers for the coding exercises in `EXERCISES.md` (4–6).
//!
//! Run with:
//!   cargo run --example answers
//!
//! Each section asserts its own expected results, so a clean run means the
//! solutions actually behave as the answers claim.

use untitled::{CalcError, Calculator, Employee, RosterError};

// --- Exercise 4: extension trait -------------------------------------------

/// Adds `mul` to `Calculator` from outside its defining module — the pattern
/// you need when the type comes from a crate you can't edit.
trait CalculatorExt {
    fn mul(&self, a: i64, b: i64) -> Result<i64, CalcError>;
}

impl CalculatorExt for Calculator {
    fn mul(&self, a: i64, b: i64) -> Result<i64, CalcError> {
        a.checked_mul(b).ok_or(CalcError::Overflow { a, b })
    }
}

// --- Exercise 5: iterator practice ------------------------------------------

/// Mean salary of the roster, or `None` if it is empty.
fn average_salary(employees: &[Employee]) -> Option<f64> {
    if employees.is_empty() {
        return None;
    }
    let total: f64 = employees.iter().map(Employee::salary).sum();
    Some(total / employees.len() as f64)
}

/// The youngest employee, or `None` if the roster is empty.
fn youngest(employees: &[Employee]) -> Option<&Employee> {
    employees.iter().min_by_key(|e| e.age())
}

// --- Exercise 6: pluggable validation ---------------------------------------

/// Build a roster from raw rows, letting the caller decide what to do with
/// each invalid row via the `on_invalid` callback.
fn build_roster<F>(raw: &[(&str, u8, f64)], mut on_invalid: F) -> Vec<Employee>
where
    F: FnMut(RosterError),
{
    raw.iter()
        .filter_map(|&(name, age, salary)| match Employee::new(name, age, salary) {
            Ok(emp) => Some(emp),
            Err(e) => {
                on_invalid(e);
                None
            }
        })
        .collect()
}

fn main() {
    // Exercise 4
    let calc = Calculator::new();
    assert_eq!(calc.mul(6, 7), Ok(42));
    assert_eq!(
        calc.mul(i64::MAX, 2),
        Err(CalcError::Overflow { a: i64::MAX, b: 2 })
    );
    println!("exercise 4: 6 * 7 = {}", calc.mul(6, 7).unwrap());

    // Exercise 5
    let roster = vec![
        Employee::new("Alice", 34, 95_000.0).unwrap(),
        Employee::new("Bob", 45, 72_000.0).unwrap(),
        Employee::new("Carol", 29, 110_000.0).unwrap(),
    ];
    let avg = average_salary(&roster).unwrap();
    assert!((avg - 92_333.333_333_333_33).abs() < 1e-6);
    let young = youngest(&roster).unwrap();
    assert_eq!(young.name(), "Carol");
    assert_eq!(average_salary(&[]), None);
    assert!(youngest(&[]).is_none());
    println!("exercise 5: average salary = {avg:.2}, youngest = {}", young.name());

    // Exercise 6
    let raw = [
        ("Alice", 34u8, 95_000.0),
        ("Frank", 0, 50_000.0), // invalid age
        ("Grace", 41, -10.0),   // invalid salary
    ];

    let mut problems = Vec::new();
    let roster = build_roster(&raw, |e| problems.push(e));
    assert_eq!(roster.len(), 1);
    assert_eq!(problems.len(), 2);
    println!(
        "exercise 6: kept {} valid row(s), collected {} error(s):",
        roster.len(),
        problems.len()
    );
    for e in &problems {
        println!("  - {e}");
    }

    println!("\nall exercise answers verified");
}