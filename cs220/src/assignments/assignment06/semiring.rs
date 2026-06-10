//! Semiring

use std::collections::HashMap;
use std::fmt::Debug;

use itertools::Itertools;

/// Semiring.
///
/// Consult <https://en.wikipedia.org/wiki/Semiring>.
pub trait Semiring: Debug + Clone + PartialEq {
    /// Additive identity.
    fn zero() -> Self;
    /// Multiplicative identity.
    fn one() -> Self;
    /// Addition operation.
    fn add(&self, rhs: &Self) -> Self;
    /// Multiplication operation.
    fn mul(&self, rhs: &Self) -> Self;
}

/// Converts integer to semiring value.
pub fn from_usize<T: Semiring>(value: usize) -> T {
    let mut result = T::zero();
    let one = T::one();

    for _ in 0..value {
        result = T::add(&result, &one);
    }

    result
}

impl Semiring for u64 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn add(&self, rhs: &Self) -> Self {
        *self + *rhs
    }

    fn mul(&self, rhs: &Self) -> Self {
        *self * *rhs
    }
}

impl Semiring for i64 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn add(&self, rhs: &Self) -> Self {
        *self + *rhs
    }

    fn mul(&self, rhs: &Self) -> Self {
        *self * *rhs
    }
}

impl Semiring for f64 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn add(&self, rhs: &Self) -> Self {
        *self + *rhs
    }

    fn mul(&self, rhs: &Self) -> Self {
        *self * *rhs
    }
}

/// Polynomials with coefficient in `C`.
///
/// For example, polynomial `x^2 + 5x + 6` is represented in `Polynomial<u64>` as follows:
///
/// ```ignore
/// Polynomial {
///     coefficients: {
///         2: 1,
///         1: 5,
///         0: 6,
///     },
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polynomial<C: Semiring> {
    coefficients: HashMap<u64, C>,
}

impl<C: Semiring> Semiring for Polynomial<C> {
    fn zero() -> Self {
        Polynomial {
            coefficients: HashMap::new(),
        }
    }

    fn one() -> Self {
        Polynomial {
            coefficients: HashMap::from([(0, C::one())]),
        }
    }

    fn add(&self, rhs: &Self) -> Self {
        let mut result_coefficients = self.coefficients.clone();

        for (power, rhs_coeff) in &rhs.coefficients {
            let zero_coeff = C::zero();
            let lhs_coeff = result_coefficients.get(power).unwrap_or(&zero_coeff);
            let new_coeff = lhs_coeff.add(rhs_coeff);
            if new_coeff == C::zero() {
                drop(result_coefficients.remove(power));
            } else {
                drop(result_coefficients.insert(*power, new_coeff));
            }
        }

        Polynomial {
            coefficients: result_coefficients,
        }
    }

    fn mul(&self, rhs: &Self) -> Self {
        let mut result_coefficients = HashMap::new();

        for (power1, coeff1) in &self.coefficients {
            for (power2, coeff2) in &rhs.coefficients {
                let new_power = power1 + power2;
                let new_coeff = coeff1.mul(coeff2);

                let zero_coeff = C::zero();
                let current_coeff = result_coefficients.get(&new_power).unwrap_or(&zero_coeff);
                let accumulated_coeff = current_coeff.add(&new_coeff);

                if accumulated_coeff == C::zero() {
                    drop(result_coefficients.remove(&new_power));
                } else {
                    drop(result_coefficients.insert(new_power, accumulated_coeff));
                }
            }
        }

        Polynomial {
            coefficients: result_coefficients,
        }
    }
}

impl<C: Semiring> Polynomial<C> {
    /// Constructs polynomial `x`.
    pub fn x() -> Self {
        Polynomial {
            coefficients: HashMap::from([(1, C::one())]),
        }
    }

    /// Evaluates the polynomial with the given value.
    pub fn eval(&self, value: C) -> C {
        let mut total = C::zero();
        for (power, coeff) in &self.coefficients {
            let mut x_to_n = C::one();
            for _ in 0..*power {
                x_to_n = x_to_n.mul(&value);
            }
            let term_val = x_to_n.mul(coeff);
            total = total.add(&term_val);
        }
        total
    }

    /// Constructs polynomial `ax^n`.
    pub fn term(a: C, n: u64) -> Self {
        Polynomial {
            coefficients: HashMap::from([(n, a)]),
        }
    }
}

impl<C: Semiring> From<C> for Polynomial<C> {
    fn from(value: C) -> Self {
        Polynomial {
            coefficients: HashMap::from([(0, value)]),
        }
    }
}

/// Given a string `s`, parse it into a `Polynomial<C>`.
/// You may assume that `s` follows the criteria below.
/// Therefore, you do not have to return `Err`.
///
/// Assumptions:
/// - Each term is separated by ` + `.
/// - Each term is one of the following form: `a`, `x`, `ax`, `x^n`, and `ax^n`, where `a` is a
///   `usize` number and `n` is a `u64` number. This `a` should then be converted to a `C` type.
/// - In `a`, it is guaranteed that `a >= 1`.
/// - In `ax` and `ax^n`, it is guaranteed that `a >= 2`.
/// - In `x^n` and `ax^n`, it is guaranteed that `n >= 2`.
/// - All terms have unique degrees.
///
/// Consult `assignment06/grade.rs` for example valid strings.
///
/// Hint: `.split`, `.parse`, and `Polynomial::term`
impl<C: Semiring> std::str::FromStr for Polynomial<C> {
    type Err = (); // Ignore this for now...

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let terms = s.split(" + ");

        let mut p = Polynomial::zero();

        for term in terms {
            if let Some((left, right)) = term.split_once('^') {
                let n: u64 = right.parse().unwrap();
                if left == "x" { // x^n
                    p = p.add(&Polynomial::term(C::one(), n));
                } else { // ax^n
                    let a_str = left.strip_suffix('x').unwrap();
                    let a: usize = a_str.parse().unwrap();
                    p = p.add(&Polynomial::term(from_usize(a), n));
                }
            } else if term.contains('x') {
                if term == "x" { // x
                    p = p.add(&Polynomial::term(C::one(), 1));
                } else { // ax
                    let a_str = term.strip_suffix('x').unwrap();
                    let a: usize = a_str.parse().unwrap();
                    p = p.add(&Polynomial::term(from_usize(a), 1));
                }
            } else { // a
                let a: usize = term.parse().unwrap();
                p = p.add(&Polynomial::term(from_usize(a), 0));
            }
        }

        Ok(p)
    }
}
