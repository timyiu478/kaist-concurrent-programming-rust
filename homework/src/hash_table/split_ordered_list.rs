//! Split-ordered linked list.

use core::mem::{self, MaybeUninit};
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{Guard, Owned};
use cs431::lockfree::list::{Cursor, List, Node};

use super::growable_array::GrowableArray;
use crate::ConcurrentMap;

/// Lock-free map from `usize` in range \[0, 2^63-1\] to `V`.
///
/// NOTE: We don't care about hashing in this homework for simplicity.
#[derive(Debug)]
pub struct SplitOrderedList<V> {
    /// Lock-free list sorted by recursive-split order.
    ///
    /// Use `MaybeUninit::uninit()` when creating sentinel nodes.
    list: List<usize, MaybeUninit<V>>,
    /// Array of pointers to the buckets.
    buckets: GrowableArray<Node<usize, MaybeUninit<V>>>,
    /// Number of buckets.
    size: AtomicUsize,
    /// Number of items.
    count: AtomicUsize,
}

impl<V> Default for SplitOrderedList<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> SplitOrderedList<V> {
    /// `size` is doubled when `count > size * LOAD_FACTOR`.
    const LOAD_FACTOR: usize = 2;

    /// Creates a new split ordered list.
    pub fn new() -> Self {
        Self {
            list: List::new(),
            buckets: GrowableArray::new(),
            size: AtomicUsize::new(2),
            count: AtomicUsize::new(0),
        }
    }

    /// Creates a cursor and moves it to the bucket for the given index.  If the bucket doesn't
    /// exist, recursively initializes the buckets.
    fn lookup_bucket<'s>(
        &'s self,
        index: usize,
        guard: &'s Guard,
    ) -> Cursor<'s, usize, MaybeUninit<V>> {
        let mut bucket_index = index % self.size.load(Acquire);
        let mut indexes_to_be_created = Vec::new();

        // Find missing buckets
        loop {
            let bucket_ptr = self.buckets.get(bucket_index, guard);
            if bucket_ptr.load(Acquire, guard).is_null() {
                indexes_to_be_created.push(bucket_index);
                if bucket_index == 0 {
                    break;
                }
                let parent_bucket = bucket_index ^ (1 << bucket_index.ilog2());
                bucket_index = parent_bucket;
            } else {
                break;
            }
        }

        let parent_bucket_shared = self.buckets.get(bucket_index, guard).load(Acquire, guard);

        let mut cursor = if parent_bucket_shared.is_null() {
            // If even bucket 0 hasn't been created, we start from the list head
            self.list.head(guard)
        } else {
            // Short cut from parent bucket
            let parent_bucket_ptr = self.buckets.get(bucket_index, guard);
            Cursor::new(parent_bucket_ptr, parent_bucket_ptr.load(Acquire, guard))
        };

        // Create bucket
        for i in indexes_to_be_created.into_iter().rev() {
            let bucket_key = i.reverse_bits();
            let mut node = Owned::new(Node::new(bucket_key, MaybeUninit::uninit()));
            loop {
                match cursor.find_harris(&bucket_key, guard) {
                    Err(_) => {}
                    Ok(r) => {
                        if r {
                            break;
                        }
                    } // Another thread created
                }
                match cursor.insert(node, guard) {
                    Err(e) => node = e, // CAS failed, retry
                    Ok(_) => {
                        break;
                    }
                }
            }

            let bucket_atomic = self.buckets.get(i, guard);
            let _ = bucket_atomic.compare_exchange(
                crossbeam_epoch::Shared::null(),
                cursor.curr(),
                Release,
                Relaxed,
                guard,
            );
        }

        cursor
    }

    /// Moves the bucket cursor returned from `lookup_bucket` to the position of the given key.
    /// Returns `(size, found, cursor)`
    fn find<'s>(
        &'s self,
        key: &usize,
        guard: &'s Guard,
    ) -> (usize, bool, Cursor<'s, usize, MaybeUninit<V>>) {
        let key_index = key.reverse_bits() | 1;
        let mut cursor = self.lookup_bucket(*key, guard);

        loop {
            match cursor.find_harris(&key_index, guard) {
                Err(_) => {}
                Ok(r) => {
                    return (self.size.load(Acquire), r, cursor);
                }
            }
        }
    }

    fn assert_valid_key(key: usize) {
        assert!(key.leading_zeros() != 0);
    }
}

impl<V> ConcurrentMap<usize, V> for SplitOrderedList<V> {
    fn lookup<'a>(&'a self, key: &usize, guard: &'a Guard) -> Option<&'a V> {
        Self::assert_valid_key(*key);

        let (_, found, cursor) = self.find(key, guard);

        if found {
            let maybe_uninit_ref = cursor.lookup(); // &'a MaybeUninit<V>
            let v_ref = unsafe { &*maybe_uninit_ref.as_ptr() }; // &'a V
            Some(v_ref)
        } else {
            None
        }
    }

    fn insert(&self, key: usize, value: V, guard: &Guard) -> Result<(), V> {
        Self::assert_valid_key(key);

        let key_index = key.reverse_bits() | 1;

        // Wrap the value so we can move it or reconstruct it on retry
        let mut current_value = value;

        loop {
            let (size, found, mut cursor) = self.find(&key, guard);

            if found {
                return Err(current_value);
            }

            let mut v = MaybeUninit::<V>::uninit();
            v.write(current_value);
            let node = Owned::new(Node::new(key_index, v));

            match cursor.insert(node, guard) {
                Ok(_) => {
                    let new_count = self.count.fetch_add(1, Relaxed) + 1;

                    if new_count > size * SplitOrderedList::<V>::LOAD_FACTOR {
                        let _ = self.size.compare_exchange(size, 2 * size, Release, Relaxed);
                    }
                    return Ok(());
                }
                Err(returned_node) => {
                    current_value = unsafe { returned_node.into_box().into_value().assume_init() };
                    continue;
                }
            }
        }
    }

    fn delete<'a>(&'a self, key: &usize, guard: &'a Guard) -> Result<&'a V, ()> {
        Self::assert_valid_key(*key);

        loop {
            let (_, found, mut cursor) = self.find(key, guard);

            if found {
                match cursor.delete(guard) {
                    Ok(maybe_uninit_ref) => {
                        self.count.fetch_sub(1, Relaxed);

                        let v_ref = unsafe { &*maybe_uninit_ref.as_ptr() };
                        return Ok(v_ref);
                    }
                    Err(_) => {
                        continue;
                    }
                }
            } else {
                return Err(());
            }
        }
    }
}
