//! Generators
//!
//! HINT: Look at the `generator_grade.rs` file to see how the generator is used.

/// Yielded value. It can be either a value or a stop signal.
enum Yielded<T> {
    Value(T),
    Stop,
}

/// Generator
/// - You can call `next()` method to get the next value.
/// - The generator should stop when it yields `Yielded::Stop`.
///
/// Reference:
/// - [Python generator](https://python-reference.readthedocs.io/en/latest/docs/generator/)
#[derive(Debug)]
pub struct Generator<T, S> {
    state: S,
    f: fn(&mut S) -> Yielded<T>,
}

impl<T, S> Iterator for Generator<T, S> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.f)(&mut self.state) {
            Yielded::Value(item) => Some(item),
            Yielded::Stop => None,
        }
    }
}

/// Returns a generator that yields fibonacci numbers.
///
/// HINT: Consult <https://en.wikipedia.org/wiki/Fibonacci_sequence>
pub fn fib_generator(first: usize, second: usize) -> Generator<usize, (usize, usize)> {
    Generator {
        state: (first, second),
        f: |s| {
            let current = s.0;
            let next_val = s.0 + s.1;
            s.0 = s.1;
            s.1 = next_val;
            Yielded::Value(current)
        },
    }
}

/// Returns a generator that yields collatz numbers.
///
/// HINT: Consult <https://en.wikipedia.org/wiki/Collatz_conjecture>
pub fn collatz_conjecture(start: usize) -> Generator<usize, usize> {
    Generator {
        state: start,
        f: |s| {
            if *s == 0 {
                return Yielded::Stop;
            }

            let current = *s;

            if current == 1 {
                *s = 0;
            } else if current % 2 == 0 {
                *s = current / 2;
            } else {
                *s = 3 * current + 1;
            }

            Yielded::Value(current)
        },
    }
}
