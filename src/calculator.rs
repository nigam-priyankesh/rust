//! A tiny `Calculator` "class" (a struct with methods) used to demonstrate
//! adding two numbers
//! crate: typed errors, logging, and unit tests.

use log::debug;
use thiserror::Error;

/// Errors that can arise from a calculator operation.
#[derive(Debug, Error, PartialEq)]
pub enum CalcError {
    #[error("integer overflow while adding {a} + {b}")]
    Overflow { a: i64, b: i64 },
}

/// A stateless calculator. The "class" is just a unit struct here, but methods
/// hang off it the way they would in an object-oriented language.
#[derive(Debug, Default, Clone, Copy)]
pub struct Calculator;

impl Calculator {
    /// Construct a calculator.
    pub fn new() -> Self {
        Self
    }

    /// Add two numbers, returning an error on overflow instead of panicking.
    ///
    /// # Errors
    /// Returns [`CalcError::Overflow`] if the result does not fit in an `i64`.
    pub fn add(&self, a: i64, b: i64) -> Result<i64, CalcError> {
        match a.checked_add(b) {
            Some(sum) => {
                debug!("add: {a} + {b} = {sum}");
                Ok(sum)
            }
            None => Err(CalcError::Overflow { a, b }),
        }
    }
}