//! Symbolic differentiation with rational coefficents.

use std::fmt;
use std::ops::*;

/// Rational number represented by two isize, numerator and denominator.
///
/// Each Rational number should be normalized so that `demoninator` is nonnegative and `numerator`
/// and `demoninator` are coprime. See `normalize` for examples. As a corner case, 0 is represented
/// by `Rational { numerator: 0, demoninator: 0 }`.
///
/// For "natural use", it also overloads standard arithmetic operations, i.e, `+`, `-`, `*`, and
/// `/`.
///
/// See [here](https://doc.rust-lang.org/core/ops/index.html) for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    numerator: isize,
    denominator: isize,
}

// Some useful constants.

/// Zero
pub const ZERO: Rational = Rational::new(0, 0);
/// One
pub const ONE: Rational = Rational::new(1, 1);
/// Minus one
pub const MINUS_ONE: Rational = Rational::new(-1, 1);

// Standard Euclidean GCD algorithm for isize
fn gcd(mut a: isize, mut b: isize) -> isize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

impl Rational {
    /// Creates a new rational number.
    pub const fn new(numerator: isize, denominator: isize) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Normalizes the fraction according to the rules:
    /// 1. If numerator is 0, return Rational { 0, 0 }
    /// 2. Denominator must be nonnegative
    /// 3. Numerator and denominator must be coprime
    fn normalize(mut num: isize, mut den: isize) -> Self {
        // Rule 1: Zero corner case
        if num == 0 {
            return Rational { numerator: 0, denominator: 0 };
        }

        // Rule 2: Fix sign so denominator is always positive
        if den < 0 {
            num = -num;
            den = -den;
        }

        // Rule 3: Reduce using GCD
        let common = gcd(num.abs(), den.abs());
        
        Rational {
            numerator: num / common,
            denominator: den / common,
        }
    }
}

impl Add for Rational {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.denominator == 0 {
            return rhs
        } else if rhs.denominator == 0 {
            return self
        }
        let num = self.numerator * rhs.denominator + rhs.numerator * self.denominator;
        let den = self.denominator * rhs.denominator;
        Rational::normalize(num, den)
    }
}

impl Mul for Rational {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.denominator == 0 || rhs.denominator == 0 {
            return ZERO;
        }
        let num = self.numerator * rhs.numerator;
        let den = self.denominator * rhs.denominator;
        Rational::normalize(num, den)
    }
}

impl Sub for Rational {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.denominator == 0 {
            return Rational::normalize(-rhs.numerator, rhs.denominator);
        } else if rhs.denominator == 0 {
            return self;
        }
        let num = self.numerator * rhs.denominator - rhs.numerator * self.denominator;
        let den = self.denominator * rhs.denominator;
        Rational::normalize(num, den)
    }
}

impl Div for Rational {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.denominator == 0 || rhs.denominator == 0 {
            return ZERO;
        }
        // To divide, multiply by the reciprocal (flip rhs numerator and denominator)
        let num = self.numerator * rhs.denominator;
        let den = self.denominator * rhs.numerator;
        Rational::normalize(num, den)
    }
}

/// Differentiable functions.
///
/// For simplicity, we only consider infinitely differentiable functions.
pub trait Differentiable: Clone {
    /// Differentiate.
    ///
    /// Since the return type is `Self`, this trait can only be implemented
    /// for types that are closed under differentiation.
    fn diff(&self) -> Self;
}

impl Differentiable for Rational {
    /// HINT: Consult <https://en.wikipedia.org/wiki/Differentiation_rules#Constant_term_rule>
    fn diff(&self) -> Self {
        ZERO
    }
}

/// Singleton polynomial.
///
/// Unlike regular polynomials, this type only represents a single term.
/// The `Const` variant is included to make `Polynomial` closed under differentiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingletonPolynomial {
    /// Constant polynomial.
    Const(Rational),
    /// Non-const polynomial.
    Polynomial {
        /// Coefficent of polynomial. Must be non-zero.
        coeff: Rational,
        /// Power of polynomial. Must be non-zero.
        power: Rational,
    },
}

impl SingletonPolynomial {
    /// Creates a new const polynomial.
    pub fn new_c(r: Rational) -> Self {
        SingletonPolynomial::Const(r)
    }

    /// Creates a new polynomial.
    pub fn new_poly(coeff: Rational, power: Rational) -> Self {
        SingletonPolynomial::Polynomial {coeff, power}
    }
}

impl Differentiable for SingletonPolynomial {
    /// HINT: Consult <https://en.wikipedia.org/wiki/Power_rule>
    fn diff(&self) -> Self {
        match self {
            SingletonPolynomial::Const(r) => SingletonPolynomial::Const(ZERO),
            SingletonPolynomial::Polynomial {coeff, power} => {
                // 1. "Drop it down": Multiply coeff by power
                let new_coeff = *coeff * *power;
                
                // 2. "Subtract one": We need a Rational representing the number 1
                let one = Rational { numerator: 1, denominator: 1 };
                let new_power = *power - one;
                
                // 3. Check our invariants: If the new power hit zero, it becomes a Const
                if new_power.numerator == 0 {
                    SingletonPolynomial::Const(new_coeff)
                } else {
                    SingletonPolynomial::Polynomial {
                        coeff: new_coeff,
                        power: new_power,
                    }
                }
            },
        }
    }
}

/// Expoential function.(`e^x`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exp;

impl Exp {
    /// Creates a new exponential function.
    pub fn new() -> Self {
        Exp{}
    }
}

impl Default for Exp {
    fn default() -> Self {
        Self::new()
    }
}

impl Differentiable for Exp {
    /// HINT: Consult <https://en.wikipedia.org/wiki/Differentiation_rules#Derivatives_of_exponential_and_logarithmic_functions>
    fn diff(&self) -> Self {
        *self
    }
}

/// Trigonometric functions.
///
/// The trig fucntions carry their coefficents to be closed under differntiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trignometric {
    /// Sine function.
    Sine {
        /// Coefficent
        coeff: Rational,
    },
    /// Cosine function.
    Cosine {
        /// Coefficent
        coeff: Rational,
    },
}

impl Trignometric {
    /// Creates a new sine function.
    pub fn new_sine(coeff: Rational) -> Self {
        Trignometric::Sine {coeff}
    }

    /// Creates a new cosine function.
    pub fn new_cosine(coeff: Rational) -> Self {
        Trignometric::Cosine {coeff}
    }
}

impl Differentiable for Trignometric {
    /// HINT: Consult <https://en.wikipedia.org/wiki/Differentiation_rules#Derivatives_of_trigonometric_functions>
    fn diff(&self) -> Self {
        match self {
            Trignometric::Sine {coeff} => Trignometric::new_cosine(*coeff),
            Trignometric::Cosine {coeff} => Trignometric::new_sine(ZERO-*coeff),
        }
    }
}

/// Basic functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseFuncs {
    /// Constant
    Const(Rational),
    /// Polynomial
    Poly(SingletonPolynomial),
    /// Exponential
    Exp(Exp),
    /// Trignometirc
    Trig(Trignometric),
}

impl Differentiable for BaseFuncs {
    fn diff(&self) -> Self {
        match self {
            BaseFuncs::Const(c) => BaseFuncs::Const(c.diff()),
            BaseFuncs::Poly(p)  => BaseFuncs::Poly(p.diff()),
            BaseFuncs::Exp(e)   => BaseFuncs::Exp(e.diff()),
            BaseFuncs::Trig(t)  => BaseFuncs::Trig(t.diff()),
        }
    }
}

/// Complex functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplexFuncs<F> {
    /// Basic functions
    Func(F),
    /// Addition
    Add(Box<ComplexFuncs<F>>, Box<ComplexFuncs<F>>),
    /// Subtraction
    Sub(Box<ComplexFuncs<F>>, Box<ComplexFuncs<F>>),
    /// Multipliciation
    Mul(Box<ComplexFuncs<F>>, Box<ComplexFuncs<F>>),
    /// Division
    Div(Box<ComplexFuncs<F>>, Box<ComplexFuncs<F>>),
    /// Composition
    Comp(Box<ComplexFuncs<F>>, Box<ComplexFuncs<F>>),
}

impl<F: Differentiable> Differentiable for Box<F> {
    fn diff(&self) -> Self {
        Box::new(self.as_ref().diff())
    }
}

impl<F: Differentiable> Differentiable for ComplexFuncs<F> {
    /// HINT: Consult <https://en.wikipedia.org/wiki/Differentiation_rules#Elementary_rules_of_differentiation>
    fn diff(&self) -> Self {
        match self {
            ComplexFuncs::Func(f) => ComplexFuncs::Func(f.diff()),
            ComplexFuncs::Add(f, g) => {
                ComplexFuncs::Add(f.diff(), g.diff())
            },
            ComplexFuncs::Sub(f, g) => {
                ComplexFuncs::Sub(f.diff(), g.diff())
            },
            ComplexFuncs::Mul(f, g) => {
                let left_side = ComplexFuncs::Mul(f.diff(), g.clone());
                let right_side = ComplexFuncs::Mul(f.clone(), g.diff());
                ComplexFuncs::Add(Box::new(left_side), Box::new(right_side))
            },
            ComplexFuncs::Div(f, g) => {
                let top_left = ComplexFuncs::Mul(f.diff(), g.clone());
                let top_right = ComplexFuncs::Mul(f.clone(), g.diff());
                let numerator = ComplexFuncs::Sub(Box::new(top_left), Box::new(top_right));
                let denominator = ComplexFuncs::Mul(g.clone(), g.clone());
                ComplexFuncs::Div(Box::new(numerator), Box::new(denominator))
            },
            ComplexFuncs::Comp(f, g) => {
                let outer_diff = ComplexFuncs::Comp(f.diff(), g.clone());
                let inner_diff = g.diff();
                ComplexFuncs::Mul(Box::new(outer_diff), inner_diff)
            },
        }
    }
}

/// Evaluate functions.
pub trait Evaluate {
    ///  Evaluate `self` at `x`.
    fn evaluate(&self, x: f64) -> f64;
}

impl Evaluate for Rational {
    fn evaluate(&self, x: f64) -> f64 {
        (self.numerator as f64) / (self.denominator as f64)
    }
}

impl Evaluate for SingletonPolynomial {
    fn evaluate(&self, x: f64) -> f64 {
        match self {
            SingletonPolynomial::Const(r) => r.evaluate(x),
            SingletonPolynomial::Polynomial {coeff, power} => {
                let x_to_n = x.powf(power.evaluate(x));
                coeff.evaluate(x) * x_to_n
            }
        }
    }
}

impl Evaluate for Exp {
    fn evaluate(&self, x: f64) -> f64 {
        x.exp()
    }
}

impl Evaluate for Trignometric {
    fn evaluate(&self, x: f64) -> f64 {
        match self {
            Trignometric::Sine{coeff} => coeff.evaluate(x) * x.sin(),
            Trignometric::Cosine{coeff} => coeff.evaluate(x) * x.cos(),
        }
    }
}

impl Evaluate for BaseFuncs {
    fn evaluate(&self, x: f64) -> f64 {
        match self {
            BaseFuncs::Const(r) => r.evaluate(x),
            BaseFuncs::Poly(p) => p.evaluate(x),
            BaseFuncs::Exp(e) => e.evaluate(x),
            BaseFuncs::Trig(t) => t.evaluate(x),
        }
    }
}

impl<F: Evaluate> Evaluate for ComplexFuncs<F> {
    fn evaluate(&self, x: f64) -> f64 {
        match self {
            ComplexFuncs::Func(f) => f.evaluate(x),
            ComplexFuncs::Add(f, g) => {
                f.evaluate(x) + g.evaluate(x)
            },
            ComplexFuncs::Sub(f, g) => {
                f.evaluate(x) - g.evaluate(x)
            },
            ComplexFuncs::Mul(f, g) => f.evaluate(x) * g.evaluate(x),
            ComplexFuncs::Div(f, g) => f.evaluate(x) / g.evaluate(x),
            ComplexFuncs::Comp(f, g) => f.evaluate(g.evaluate(x)),
        }
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == ZERO {
            return write!(f, "0");
        } else if self.denominator == 1 {
            return write!(f, "{}", self.numerator);
        }
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl fmt::Display for SingletonPolynomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Const(r) => write!(f, "{r}"),
            Self::Polynomial { coeff, power } => {
                // coeff or power is zero
                if *coeff == ZERO {
                    return write!(f, "0");
                } else if *power == ZERO {
                    return write!(f, "{coeff}");
                }

                // Standard form of px^q
                let coeff = if *coeff == ONE {
                    "".to_string()
                } else if *coeff == MINUS_ONE {
                    "-".to_string()
                } else {
                    format!("({coeff})")
                };
                let var = if *power == ONE {
                    "x".to_string()
                } else {
                    format!("x^({power})")
                };
                write!(f, "{coeff}{var}")
            }
        }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exp(x)")
    }
}

impl fmt::Display for Trignometric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (func, coeff) = match self {
            Trignometric::Sine { coeff } => ("sin(x)", coeff),
            Trignometric::Cosine { coeff } => ("cos(x)", coeff),
        };

        if *coeff == ZERO {
            write!(f, "0")
        } else if *coeff == ONE {
            write!(f, "{func}")
        } else if *coeff == MINUS_ONE {
            write!(f, "-{func}")
        } else {
            write!(f, "({coeff}){func}")
        }
    }
}

impl fmt::Display for BaseFuncs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Const(r) => write!(f, "{r}"),
            Self::Poly(p) => write!(f, "{p}"),
            Self::Exp(e) => write!(f, "{e}"),
            Self::Trig(t) => write!(f, "{t}"),
        }
    }
}

impl<F: Differentiable + fmt::Display> fmt::Display for ComplexFuncs<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComplexFuncs::Func(func) => write!(f, "{func}"),
            ComplexFuncs::Add(l, r) => write!(f, "({l} + {r})"),
            ComplexFuncs::Sub(l, r) => write!(f, "({l} - {r})"),
            ComplexFuncs::Mul(l, r) => write!(f, "({l} * {r})"),
            ComplexFuncs::Div(l, r) => write!(f, "({l} / {r})"),
            ComplexFuncs::Comp(l, r) => write!(f, "({l} ∘ {r})"),
        }
    }
}
