//! Assignment 08: First-class functions.

/// Repeat
///
/// Returns an anonymous function that applies the given function `f` for `n` times.
/// For instance, `repeat(3, f)(x)` roughly translates to `f(f(f(x)))`.
///
/// Refer `test_repeat` in `assignment08_grade.rs` for detailed examples.
pub fn repeat<T, F: FnMut(T) -> T>(n: usize, mut f: F) -> impl FnMut(T) -> T {
    // move is needed because the function captures n and f
    // they are stored in the stack of repeat function call
    // if the closure gets the references of them, the references will be invalid
    // after the repeat function returns
    move |x| {
        let mut v = x;
        for _ in 0..n {
            v = f(v);
        }
        v
    }
}

/// Funny Map
///
/// Applies the given function `f` for `i` times for the `i`-th element of the given vector.
///
/// For instance, `funny_map(f, [v0, v1, v2, v3])` roughly translates to `[v0, f(v1), f(f(v2)),
/// f(f(f(v3)))]`.
///
/// Refer `test_funny_map` in `assignment08_grade.rs` for detailed examples.
pub fn funny_map<T, F: Fn(T) -> T>(f: F, vs: Vec<T>) -> Vec<T> {
    let mut new_vs: Vec<T> = Vec::new();

    for (i, mut v) in vs.into_iter().enumerate() {
        for _ in 0..i {
            v = f(v);
        }
        new_vs.push(v);
    }

    new_vs
}

/// Count Repeat
///
/// Returns the number of the elements in the set {`x`, `f(x)`, `f(f(x))`, `f(f(f(x)))`, ...}. You
/// may assume that the answer is finite and small enough.
///
/// Refer `test_count_repeat` in `assignment08_grade.rs` for detailed examples.
pub fn count_repeat<T, F: Fn(T) -> T>(f: F, x: T) -> usize
where
    T: PartialEq + Copy,
{
    // 
    // 99 ──> 1 ──> 2 ──> 3 ──> 4 ──> 5
    //        |                 |
    //        ------------------
    // Tail: 99
    // Loop: 1 2 3 4
    // Start of the loop: 1

    // Phase 1: Find a meeting point inside the loop
    let mut slow = f(x);
    let mut fast = f(f(x));

    while slow != fast {
        slow = f(slow);
        fast = f(f(fast));
    }

    // Phase 2: Find the length of the tail
    let mut tail_steps = 0;
    let mut ptr1 = x;
    let mut ptr2 = slow;
    while ptr1 != ptr2 {
        ptr1 = f(ptr1);
        ptr2 = f(ptr2);
        tail_steps += 1;
    }

    // Phase 3: Find the length of the loop
    let mut loop_steps = 1;
    slow = f(ptr1); // ptr1 is now exactly at the start of the loop
    while slow != ptr1 {
        slow = f(slow);
        loop_steps += 1;
    }

    // Total steps from start until a repeat is encountered is Tail + Loop
    tail_steps + loop_steps
}

/// Either `T1`, or `T2`.
///
/// Fill out `map` method for this type.
#[derive(Debug, PartialEq, Eq)]
pub enum Either2<T1, T2> {
    /// Case 1.
    Case1 {
        /// The inner value.
        inner: T1,
    },
    /// Case 2.
    Case2 {
        /// The inner value.
        inner: T2,
    },
}

impl<T1, T2> Either2<T1, T2> {
    /// Maps the inner value.
    ///
    /// If the inner value is case 1, apply `f1`, and if it is case 2, apply `f2`.
    ///
    /// Refer `test_either2_map` in `assignment08_grade.rs` for detailed examples.
    pub fn map<U1, U2, F1, F2>(self, f1: F1, f2: F2) -> Either2<U1, U2>
    where
        F1: FnOnce(T1) -> U1,
        F2: FnOnce(T2) -> U2,
    {
        match self {
            Either2::Case1 { inner } => Either2::Case1 { inner: f1(inner) },
            Either2::Case2 { inner } => Either2::Case2 { inner: f2(inner) },
        }
    }
}
