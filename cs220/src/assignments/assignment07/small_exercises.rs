//! Implement functions using `Iterator` trait

struct FindIter<'s, T: Eq> {
    query: &'s [T],
    base: &'s [T],
    curr: usize,
}

impl<T: Eq> Iterator for FindIter<'_, T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining_base = &self.base[self.curr..];

        if let Some(offset) = remaining_base
            .windows(self.query.len())
            .position(|window| window == self.query) 
        {
            let match_index = self.curr + offset;
            self.curr = match_index + 1;
            return Some(match_index);
        }

        None
    }
}

/// Returns an iterator over substring query indexes in the base.
pub fn find<'s, T: Eq>(query: &'s [T], base: &'s [T]) -> impl 's + Iterator<Item = usize> {
    FindIter {
        query,
        base,
        curr: 0,
    }
}

/// Implement generic fibonacci iterator
struct FibIter<T> {
    first: T,
    second: T
}

impl<T: std::ops::Add<Output = T> + Copy> FibIter<T> {
    fn new(first: T, second: T) -> Self {
        FibIter { first, second }
    }
}

impl<T> Iterator for FibIter<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let third = self.first + self.second;
        let first = self.first;
        self.first = self.second;
        self.second = third;
        Some(first)
    }
}

/// Returns and iterator over the generic fibonacci sequence starting from `first` and `second`.
/// This is a generic version of `fibonacci` function, which works for any types that implements
/// `std::ops::Add` trait.
pub fn fib<T>(first: T, second: T) -> impl Iterator<Item = T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    FibIter::new(first, second)
}

/// Endpoint of range, inclusive or exclusive.
#[derive(Debug)]
pub enum Endpoint {
    /// Inclusive endpoint
    Inclusive(isize),

    /// Exclusive endpoint
    Exclusive(isize),
}

struct RangeIter {
    left: isize,
    right: isize,
    step: isize,
    curr: isize
}

impl RangeIter {
    fn new(endpoints: (Endpoint, Endpoint), step: isize) -> Self {
        let left_val = match endpoints.0 {
            Endpoint::Inclusive(val) => val,
            Endpoint::Exclusive(val) => if step > 0 { val + 1 } else { val - 1 },
        };

        let right_val = match endpoints.1 {
            Endpoint::Inclusive(val) => val,
            Endpoint::Exclusive(val) => if step > 0 { val - 1 } else { val + 1 },
        };

        RangeIter {
            left: left_val,
            right: right_val,
            step,
            curr: left_val,
        }
    }
}

impl Iterator for RangeIter {
    type Item = isize;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.step > 0 && self.curr >= self.left && self.curr <= self.right) ||
            (self.step < 0 && self.curr >= self.right && self.curr <= self.left) {
            let val = self.curr;
            self.curr += self.step;
            return Some(val);
        }
        None
    }
}

/// Returns an iterator over the range [left, right) with the given step.
pub fn range(left: Endpoint, right: Endpoint, step: isize) -> impl Iterator<Item = isize> {
    RangeIter::new((left, right), step)
}

/// Write an iterator that returns all divisors of n in increasing order.
/// Assume n > 0.
///
/// Hint: trying all candidates from 1 to n will most likely time out!
/// To optimize it, make use of the following fact:
/// if x is a divisor of n that is greater than sqrt(n),
/// then n/x is a divisor of n that is smaller than sqrt(n).
struct Divisors {
    n: u64,
    sqrt_n: u64,
    x: u64,
    divisors: Vec<u64>,
}

impl Iterator for Divisors {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        while self.x < self.sqrt_n {
            self.x += 1;
            if self.n.is_multiple_of(self.x) {
                let n_div_x = self.n/self.x;
                if self.x != n_div_x {
                    self.divisors.push(n_div_x);
                }
                return Some(self.x);
            }
        }
        self.divisors.pop()
    }
}

/// Returns an iterator over the divisors of n.
pub fn divisors(n: u64) -> impl Iterator<Item = u64> {
    Divisors {
        n,
        sqrt_n: (n as f64).sqrt() as u64,
        x: 0,
        divisors: Vec::new(),
    }
}
