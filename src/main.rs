//! Employee roster binary: a thin shell over the `untitled` library.
//!
//! It loads sample data, configures logging, runs the ranking, and maps any
//! error to a non-zero exit code. All reusable logic (and its tests) lives in
//! `src/lib.rs` and the `tests/` directory.
//!
//! Run with logging:
//!   PowerShell:  $env:RUST_LOG="debug"; cargo run
//!   bash:        RUST_LOG=debug cargo run

use log::{error, info, warn};

use untitled::{Calculator, Employee, RosterError, rank_by_combined_score};

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
    let sum = calc.add(a, b)?; // CalcError converts into RosterError via `#[from]`
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