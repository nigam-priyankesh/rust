# Learning Exercises

Exercises based on the code in this repository (`src/lib.rs`, `src/calculator.rs`,
`src/main.rs`). Try each one before opening the answer.

The coding answers (exercises 4–6) are also runnable:

```sh
cargo run --example answers
```

---

## Exercise 1 — Borrowing vs. owning

`rank_by_combined_score` has this signature:

```rust
pub fn rank_by_combined_score(employees: &[Employee]) -> Result<Vec<Employee>, RosterError>
```

**a)** Why does it take `&[Employee]` (a borrowed slice) instead of `Vec<Employee>`?
**b)** Why must it return a *new* `Vec<Employee>` instead of sorting the input in place?

<details>
<summary>Answer</summary>

**a)** Taking `&[Employee]` borrows the caller's data instead of consuming it. The
caller keeps ownership and can use the roster afterwards (e.g. `main.rs` could print
the original order too). It also accepts more types: a `&Vec<Employee>` auto-derefs
to `&[Employee]`, and so does an array or a sub-slice.

**b)** The parameter is an *immutable* borrow, so the function is not allowed to
mutate the input. To sort, it first clones into an owned `Vec` (`employees.to_vec()`,
which works because `Employee` derives `Clone`) and sorts that. If we wanted in-place
sorting, the signature would need `&mut Vec<Employee>` (or `&mut [Employee]`), and the
caller would need a `mut` binding.
</details>

---

## Exercise 2 — `impl Into<String>` and error conversion

**a)** `Employee::new` takes `name: impl Into<String>`. What does that buy us over
taking `name: String` or `name: &str`?

**b)** In `main.rs`, this line converts a `CalcError` into a `RosterError`
automatically:

```rust
let sum = calc.add(a, b)?;
```

What two pieces of code make that conversion work?

<details>
<summary>Answer</summary>

**a)** `impl Into<String>` accepts anything convertible into a `String`: a `&str`
(`Employee::new("Alice", ...)`), an owned `String`, a `Cow<str>`, etc. With
`name: String`, callers with a `&str` would have to write `"Alice".to_string()`
at every call site. With `name: &str`, callers that already own a `String` would
be forced into an unnecessary extra allocation (`&s` borrow, then `to_string()`
inside). The generic version lets each caller pay only the conversion it needs.

**b)** Two things cooperate:

1. The `#[from]` attribute on the `RosterError::Calc` variant:

   ```rust
   #[error(transparent)]
   Calc(#[from] CalcError),
   ```

   `thiserror` generates an `impl From<CalcError> for RosterError` from this.

2. The `?` operator. When `calc.add(a, b)` returns `Err(CalcError)`, `?` calls
   `From::from` on the error to convert it into the function's error type
   (`RosterError`) before returning it. (`#[error(transparent)]` additionally makes
   the wrapper delegate its `Display` and `source()` to the inner `CalcError`.)
</details>

---

## Exercise 3 — Floats and sorting

The comparator in `rank_by_combined_score` uses `sb.total_cmp(&sa)`:

```rust
match sb.total_cmp(&sa) {
    Ordering::Equal => a.name.cmp(&b.name),
    other => other,
}
```

**a)** Why can't we just call `sb.cmp(&sa)` like we would with integers?
**b)** What could go wrong with the common alternative `sb.partial_cmp(&sa).unwrap()`?
**c)** Why does `sb` come before `sa` in the comparison?

<details>
<summary>Answer</summary>

**a)** `f64` does not implement `Ord`, only `PartialOrd`, because `NaN` is not
comparable to anything (`NaN < x`, `NaN > x`, and `NaN == x` are all false). `cmp`
requires a *total* order, so it simply doesn't exist for `f64`.

**b)** `partial_cmp` returns `Option<Ordering>`, and it returns `None` whenever
either value is `NaN` — so `.unwrap()` would panic mid-sort. Worse, a comparator
that isn't a total order is a logic error for `sort_by` and may panic inside the
sort itself. `total_cmp` implements the IEEE 754 `totalOrder` predicate, which
gives every float (including `NaN`) a defined position, so the comparator can
never panic.

**c)** Comparing `sb` to `sa` (instead of `sa` to `sb`) reverses the order, giving a
*descending* sort: the highest combined score ranks first.
</details>

---

## Exercise 4 — Extend the calculator (coding)

`Calculator` only knows how to `add`. Give it overflow-safe `mul` (multiply) support
with the same error-handling style — **but** do it from *outside* `src/calculator.rs`
(imagine the calculator comes from a third-party crate you can't edit).

Hint: Rust does not allow inherent `impl Calculator { ... }` blocks outside the
crate that defines `Calculator`. What's the idiomatic workaround?

<details>
<summary>Answer</summary>

The idiomatic workaround is an **extension trait**: define a trait holding the new
method, then implement it for the foreign type.

```rust
use untitled::{CalcError, Calculator};

trait CalculatorExt {
    fn mul(&self, a: i64, b: i64) -> Result<i64, CalcError>;
}

impl CalculatorExt for Calculator {
    fn mul(&self, a: i64, b: i64) -> Result<i64, CalcError> {
        // checked_mul mirrors the checked_add used by Calculator::add.
        a.checked_mul(b).ok_or(CalcError::Overflow { a, b })
    }
}
```

Callers just need the trait in scope (`use ...::CalculatorExt;`) and can then write
`calc.mul(6, 7)?`. This is the same pattern crates like `itertools` use to add
methods to standard-library iterators.

Note `ok_or` here: it converts `Option<i64>` into `Result<i64, CalcError>` in one
step, equivalent to the `match` used in `Calculator::add`.

(If you *can* edit `src/calculator.rs`, an inherent method in the existing
`impl Calculator` block is simpler — reusing `CalcError::Overflow` the same way.)
</details>

---

## Exercise 5 — Iterator practice (coding)

Using only iterator adapters (no `for` loops, no indexing), write:

```rust
/// Mean salary of the roster, or `None` if it is empty.
fn average_salary(employees: &[Employee]) -> Option<f64>

/// The youngest employee, or `None` if the roster is empty.
fn youngest(employees: &[Employee]) -> Option<&Employee>
```

Hints: `iter().map(...).sum::<f64>()`, and `min_by_key` exists on iterators.

<details>
<summary>Answer</summary>

```rust
use untitled::Employee;

fn average_salary(employees: &[Employee]) -> Option<f64> {
    if employees.is_empty() {
        return None;
    }
    let total: f64 = employees.iter().map(Employee::salary).sum();
    Some(total / employees.len() as f64)
}

fn youngest(employees: &[Employee]) -> Option<&Employee> {
    employees.iter().min_by_key(|e| e.age())
}
```

Points worth noticing:

- `map(Employee::salary)` passes the accessor *method as a function* — equivalent
  to `map(|e| e.salary())`.
- `min_by_key` works here because `u8` implements `Ord`. We could **not** write
  `min_by_key(|e| e.salary())`, because `f64` is not `Ord` (see Exercise 3) — for
  salary you'd reach for `min_by(|a, b| a.salary().total_cmp(&b.salary()))`.
- `min_by_key` already returns `None` for an empty iterator, so `youngest` needs no
  explicit empty check.
- The fields are private, so everything goes through the public accessors —
  the encapsulation set up by `Employee::new` still holds.
</details>

---

## Exercise 6 — Make validation pluggable (coding)

`build_sample_roster` in `main.rs` silently decides that invalid rows are skipped
with a warning. Write a reusable version where the *caller* decides what happens to
invalid rows, by passing a closure:

```rust
fn build_roster<F>(raw: &[(&str, u8, f64)], mut on_invalid: F) -> Vec<Employee>
where
    F: FnMut(RosterError),
```

Then show two call sites: one that logs-and-skips (current behavior) and one that
collects all errors into a `Vec` for reporting.

<details>
<summary>Answer</summary>

```rust
use untitled::{Employee, RosterError};

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

// Call site 1: log and skip (what main.rs does today).
let roster = build_roster(&raw, |e| log::warn!("skipping invalid record: {e}"));

// Call site 2: collect every error for later reporting.
let mut problems = Vec::new();
let roster = build_roster(&raw, |e| problems.push(e));
```

Why `FnMut` and not `Fn`? The second call site's closure *mutates* its captured
environment (`problems.push(e)`), and `Fn` closures may only read their captures.
`FnMut` accepts both closures, so it's the least restrictive bound that still works
— and that's why the parameter itself is declared `mut on_invalid`.
</details>