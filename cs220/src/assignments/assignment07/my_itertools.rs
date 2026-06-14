//! Implement your own minimal `itertools` crate.

use std::collections::HashSet;
use std::hash::Hash;

/// Iterator that iterates over the given iterator and returns only unique elements.
#[derive(Debug)]
pub struct Unique<I: Iterator> {
    it: I,
    seen: HashSet<I::Item>
}

impl<I: Iterator> Iterator for Unique<I>
where
    I::Item: Eq + Hash + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.it.next() {
            if self.seen.insert(item.clone()) {
                return Some(item);
            }
        }
        None
    }
}

/// Iterator that chains two iterators together.
#[derive(Debug)]
pub struct Chain<I1: Iterator, I2: Iterator> {
    it1: I1,
    it2: I2
}

impl<T: Eq + Hash + Clone, I1: Iterator<Item = T>, I2: Iterator<Item = T>> Iterator
    for Chain<I1, I2>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.it1.next() {
            return Some(item);
        }
        if let Some(item) = self.it2.next() {
            return Some(item);
        }
        None
    }
}

/// Iterator that iterates over given iterator and enumerates each element.
#[derive(Debug)]
pub struct Enumerate<I: Iterator> {
    it: I,
    idx: usize
}

impl<I: Iterator> Iterator for Enumerate<I> {
    type Item = (usize, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.it.next() {
            self.idx += 1;
            return Some((self.idx-1, item));
        }
        None
    }
}

/// Iterator that zips two iterators together.
///
/// If one iterator is longer than the other one, the remaining elements for the longer element
/// should be ignored.
#[derive(Debug)]
pub struct Zip<I1: Iterator, I2: Iterator> {
    it1: I1,
    it2: I2,
}

impl<I1: Iterator, I2: Iterator> Iterator for Zip<I1, I2> {
    type Item = (I1::Item, I2::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item1) = self.it1.next() && let Some(item2) = self.it2.next() {
            return Some((item1, item2));
        }
        None
    }
}

/// My Itertools trait.
pub trait MyIterTools: Iterator {
    /// Returns an iterator that iterates over the `self` and returns only unique elements.
    fn my_unique(self) -> Unique<Self>
    where
        Self: Sized,
    {
        Unique {
            it: self,
            seen: HashSet::new(),
        }
    }

    /// Returns an iterator that chains `self` and `other` together.
    fn my_chain<I: Iterator>(self, other: I) -> Chain<Self, I>
    where
        Self: Sized,
    {
        Chain { it1: self, it2: other, }
    }

    /// Returns an iterator that iterates over `self` and enumerates each element.
    fn my_enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate { it: self, idx: 0, }
    }

    /// Returns an iterator that zips `self` and `other` together.
    fn my_zip<I: Iterator>(self, other: I) -> Zip<Self, I>
    where
        Self: Sized,
    {
        Zip { it1: self, it2: other, }
    }

    /// Foldleft for `MyIterTools`
    fn my_fold<T, F>(mut self, init: T, mut f: F) -> T
    where
        Self: Sized,
        F: FnMut(Self::Item, T) -> T,
    {
        let mut val = init;
        while let Some(item) = self.next() {
            val = f(item, val);
        }
        val
    }
}

impl<T: ?Sized> MyIterTools for T where T: Iterator {}
