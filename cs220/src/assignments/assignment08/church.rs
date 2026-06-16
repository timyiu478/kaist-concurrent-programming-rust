//! Church Numerals
//!
//! This exercise involves the use of "Church numerals", a
//! representation of natural numbers using lambda calculus, named after
//! Alonzo Church. Each Church numeral corresponds to a natural number `n`
//! and is represented as a higher-order function that applies a given function `f` `n` times.
//!
//! For more information, see:
//! - <https://en.wikipedia.org/wiki/Church_encoding>
//! - <https://opendsa-server.cs.vt.edu/OpenDSA/Books/PL/html/ChurchNumerals.html>

use std::cell::RefCell;
use std::rc::Rc;

/// Church numerals are represented as higher-order functions that take a function `f`
pub type Church<T> = Rc<dyn Fn(Rc<dyn Fn(T) -> T>) -> Rc<dyn Fn(T) -> T>>;

/// This function returns a Church numeral equivalent of the natural number 1.
/// It takes a function `f` and applies it exactly once.
pub fn one<T: 'static>() -> Church<T> {
    Rc::new(move |f| Rc::new(move |x| f(x)))
}

/// This function returns a Church numeral equivalent of the natural number 2.
/// It takes a function `f` and applies it twice.
pub fn two<T: 'static>() -> Church<T> {
    Rc::new(move |f| Rc::new(move |x| f(f(x))))
}

/// This function represents the Church numeral for zero. As zero applications
/// of `f` should leave the argument unchanged, the function simply returns the input.
pub fn zero<T: 'static>() -> Church<T> {
    Rc::new(|_| Rc::new(|x| x))
}

/// Implement a function to add 1 to a given Church numeral.
pub fn succ<T: 'static>(n: Church<T>) -> Church<T> {
    Rc::new(move |f| { 
        // Why Rc::clone? the outer closure can keep its own copy of 'n' for future calls
        let n_cloned = Rc::clone(&n);
        let n_times = n_cloned(Rc::clone(&f));
        Rc::new(move |x| {
            n_times(f(x))
        })
    })
}

/// Implement a function to add two Church numerals.
pub fn add<T: 'static>(n: Church<T>, m: Church<T>) -> Church<T> {
    Rc::new(move |f| { 
        let n_cloned = Rc::clone(&n);
        let m_cloned = Rc::clone(&m);
        let n_times = n_cloned(Rc::clone(&f));
        let m_times = m_cloned(Rc::clone(&f));
        Rc::new(move |x| {
            n_times(m_times(x))
        })
    })
}

/// Implement a function to multiply (mult) two Church numerals.
pub fn mult<T: 'static>(n: Church<T>, m: Church<T>) -> Church<T> {
    Rc::new(move |f| { 
        let n_cloned = Rc::clone(&n);
        let m_cloned = Rc::clone(&m);
        let n_times = n_cloned(Rc::clone(&f));
        let mn_times = m_cloned(n_times);
        Rc::new(move |x| {
            mn_times(x)
        })
    })
}

/// Implement a function to raise one Church numeral to the power of another.
/// This is the Church numeral equivalent of the natural number operation of exponentiation.
/// Given two natural numbers `n` and `m`, the function should return a Church numeral
/// that represents `n` to the power of `m`. The key is to convert `n` and `m` to Church numerals,
/// and then apply the Church numeral for `m` (the exponent) to the Church numeral for `n` (the
/// base). Note: This function should be implemented *WITHOUT* using the `to_usize` or any
/// `pow`-like method.
pub fn exp<T: 'static>(n: usize, m: usize) -> Church<T> {
    // ACTION ITEM: Uncomment the following lines and replace `todo!()` with your code.
    let n = from_usize(n);
    let m = from_usize(m);

    m(n)
}

/// Implement a function to convert a Church numeral to a usize type.
pub fn to_usize<T: 'static + Default>(n: Church<T>) -> usize {
    // 1. Wrap the RefCell inside an Rc so we can share it
    let count = Rc::new(RefCell::new(0));
    
    // 2. Clone the Rc pointer for the closure to move inside
    let count_cloned = Rc::clone(&count);
    let f = Rc::new(move |x: T| {
        *count_cloned.borrow_mut() += 1;
        x
    });
    
    let n_times = n(f);
    drop((*n_times)(T::default()));

    // 3. Look inside our original Rc pointer to see the final tally!
    *count.borrow()
}

/// Implement a function to convert a usize type to a Church numeral.
pub fn from_usize<T: 'static>(n: usize) -> Church<T> {
    Rc::new(move |f| { 
        Rc::new(move |x| {
           let mut v = x;
           for _ in 0..n {
               v = Rc::clone(&f)(v);
           }
           v
        })
    })
}
