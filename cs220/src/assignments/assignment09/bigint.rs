//! Big integer with infinite precision.

use std::fmt;
use std::iter::zip;
use std::ops::*;

/// An signed integer with infinite precision implemented with an "carrier" vector of `u32`s.
///
/// The vector is interpreted as a base 2^(32 * (len(carrier) - 1)) integer, where negative
/// integers are represented in their [2's complement form](https://en.wikipedia.org/wiki/Two%27s_complement).
///
/// For example, the vector `vec![44,345,3]` represents the integer
/// `44 * (2^32)^2 + 345 * (2^32) + 3`,
/// and the vector `vec![u32::MAX - 5, u32::MAX - 7]` represents the integer
/// `- (5 * 2^32 + 8)`
///
/// You will implement the `Add` and `Sub` trait for this type.
///
/// Unlike standard fix-sized intergers in Rust where overflow will panic, the carrier is extended
/// to save the overflowed bit. On the contrary, if the precision is too much (e.g, vec![0,0] is
/// used to represent 0, where `vec![0]` is sufficent), the carrier is truncated.
///
/// See [this section](https://en.wikipedia.org/wiki/Two%27s_complement#Arithmetic_operations) for a rouge guide on implementation,
/// while keeping in mind that the carrier should be extended to deal with overflow.
///
/// The `sign_extension()`, `two_complement()`, and `truncate()` are non-mandatory helper methods.
///
/// For testing and debugging purposes, the `Display` trait is implemented for you, which shows the
/// integer in hexadecimal form.
#[derive(Debug, Clone)]
pub struct BigInt {
    /// The carrier for `BigInt`.
    ///
    /// Note that the carrier should always be non-empty.
    pub carrier: Vec<u32>,
}

impl BigInt {
    /// Create a new `BigInt` from a `usize`.
    pub fn new(n: u32) -> Self {
        BigInt {
            carrier: Vec::from([n]),
        }
    }

    /// Creates a new `BigInt` from a `Vec<u32>`.
    ///
    /// # Panic
    ///
    /// Panics if `carrier` is empty.
    pub fn new_large(carrier: Vec<u32>) -> Self {
        assert!(!carrier.is_empty());
        BigInt { carrier, }
    }
}

const SIGN_MASK: u32 = 1 << 31;

impl BigInt {
    /// Extend `self` to `len` bits.
    fn sign_extension(&self, len: usize) -> Self {
        let mut carrier = self.carrier.clone();

        // Determine the sign by checking the Most Significant Bit of index 0
        let is_negative = (carrier[0] & SIGN_MASK) != 0;
        let pad_value = if is_negative { !0u32 } else { 0u32 };

        while (carrier.len() * 32) < len {
            carrier.insert(0, pad_value);
        }

        BigInt {
            carrier,
        }
    }

    /// Compute the two's complement of `self`.
    fn two_complement(&self) -> Self {
        // one's complement
        let mut carrier: Vec<u32> = self.carrier.iter()
            .map(|&x| !x)
            .collect();

        // add 1
        let mut carry = 1u64;
        for n in carrier.iter_mut().rev() {
            let sum = *n as u64 + carry;
            *n = sum as u32;
            carry = sum >> 32;
        }

        let orig_negative = (self.carrier[0] & SIGN_MASK) != 0;
        let res_negative = (carrier[0] & SIGN_MASK) != 0;
        if orig_negative && res_negative {
            carrier.insert(0, 0);
        }

        BigInt { carrier }
    }

    /// Truncate a `BigInt` to the minimum length.
    fn truncate(&self) -> Self {
        let mut carrier = self.carrier.clone();

        while carrier.len() > 1 {
            let top = carrier[0];
            let next = carrier[1];
            
            // Remove redundant leading 0s
            if top == 0 && (next & SIGN_MASK) == 0 {
                let _ = carrier.remove(0);
            }
            // Remove redundant leading 0xFFFFFFFFs
            else if top == !0 && (next & SIGN_MASK) != 0 {
                let _ = carrier.remove(0);
            } else {
                break;
            }
        }

        BigInt {
            carrier,
        }
    }
}

impl Add for BigInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // 1. Find the length of the longer BigInt (in blocks)
        let max_blocks = std::cmp::max(self.carrier.len(), rhs.carrier.len());
        
        // 2. Sign-extend both to (max_blocks + 1) blocks. 
        // The extra +1 block safely catches any final signed overflow.
        let target_bits = (max_blocks + 1) * 32;
        let lhs_ext = self.sign_extension(target_bits);
        let rhs_ext = rhs.sign_extension(target_bits);

        // Pre-allocate space to avoid reallocations
        let mut result = Vec::with_capacity(max_blocks + 1);
        let mut carry = 0u64;

        // 3. Cleanly zip them together from LSB to MSB (right to left)
        for (&l_n, &r_n) in lhs_ext.carrier.iter().rev().zip(rhs_ext.carrier.iter().rev()) {
            let sum = l_n as u64 + r_n as u64 + carry;
            result.push(sum as u32); // O(1) fast push to the end
            carry = sum >> 32;
        }

        // 4. Reverse the vector once at the end to restore Big-Endian order
        result.reverse();

        // 5. Truncate any unnecessary leading sign blocks added during step 2
        BigInt { carrier: result }.truncate()
    }
}

impl Sub for BigInt {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let rhs_neg = rhs.two_complement();
        self.add(rhs_neg)
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Hex formatting so that each u32 can be formatted independently.
        for i in self.carrier.iter() {
            write!(f, "{:08x}", i)?;
        }
        Ok(())
    }
}
