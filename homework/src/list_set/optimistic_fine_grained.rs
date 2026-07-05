use std::cmp::Ordering::{self, *};
use std::mem::{self, ManuallyDrop};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst, *};

use crossbeam_epoch::{Atomic, Guard, Owned, Shared, pin};
use cs431::lock::seqlock::{ReadGuard, SeqLock};

use crate::ConcurrentSet;

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: SeqLock<Atomic<Node<T>>>,
}

/// Concurrent sorted singly linked list using fine-grained optimistic loc      king.
#[derive(Debug)]
pub struct OptimisticFineGrainedListSet<T> {
    head: SeqLock<Atomic<Node<T>>>,
}

unsafe impl<T: Send> Send for OptimisticFineGrainedListSet<T> {}
unsafe impl<T: Sync> Sync for OptimisticFineGrainedListSet<T> {}

#[derive(Debug)]
struct Cursor<'g, T> {
    // Reference to the `next` field of previous node which points to the current node.
    prev: ReadGuard<'g, Atomic<Node<T>>>,
    curr: Shared<'g, Node<T>>,
}

impl<T> Node<T> {
    fn new(data: T, next: Shared<'_, Self>) -> Owned<Self> {
        Owned::new(Self {
            data,
            next: SeqLock::new(next.into()),
        })
    }
}

impl<'g, T: Ord> Cursor<'g, T> {
    /// Moves the cursor to the position of key in the sorted list.
    /// Returns whether the value was found.
    ///
    /// Return `Err(())` if the cursor cannot move.
    fn find(&mut self, key: &T, guard: &'g Guard) -> Result<bool, ()> {
        loop {
            if self.curr.is_null() {
                return Ok(false);
            }
            let curr_node = unsafe { self.curr.as_ref().unwrap() };
            let data = &curr_node.data;
            match data.cmp(key) {
                Ordering::Equal => return Ok(true),
                Ordering::Greater => return Ok(false),
                Ordering::Less => {
                    let next_guard = unsafe { curr_node.next.read_lock() };

                    // We take ownership of the old guard out of self.prev using mem::replace
                    let old_prev = mem::replace(&mut self.prev, next_guard);

                    if !old_prev.finish() {
                        return Err(());
                    }

                    self.curr = self.prev.load(Acquire, guard);
                }
            }
        }
    }
}

impl<T> OptimisticFineGrainedListSet<T> {
    /// Creates a new list.
    pub fn new() -> Self {
        Self {
            head: SeqLock::new(Atomic::null()),
        }
    }

    fn head<'g>(&'g self, guard: &'g Guard) -> Cursor<'g, T> {
        let prev = unsafe { self.head.read_lock() };
        let curr = prev.load(Acquire, guard);
        Cursor { prev, curr }
    }
}

impl<T: Ord> OptimisticFineGrainedListSet<T> {
    fn find<'g>(&'g self, key: &T, guard: &'g Guard) -> Result<(bool, Cursor<'g, T>), ()> {
        let mut cursor = self.head(guard);

        match cursor.find(key, guard) {
            Ok(found) => Ok((found, cursor)),
            Err(()) => {
                // Consuming the leftover ReadGuard on failure
                let _ = cursor.prev.finish();
                Err(())
            }
        }
    }
}

impl<T: Ord> ConcurrentSet<T> for OptimisticFineGrainedListSet<T> {
    fn contains(&self, key: &T) -> bool {
        let guard = pin();
        loop {
            if let Ok((found, mut cursor)) = self.find(key, &guard)
                && cursor.prev.finish()
            {
                return found;
            }
        }
    }

    fn insert(&self, key: T) -> bool {
        let guard = pin();
        loop {
            if let Ok((found, mut cursor)) = self.find(&key, &guard) {
                if found {
                    if cursor.prev.finish() {
                        return false;
                    }
                    continue;
                }
                // Assumed remove will acquire write locks for both prev and curr
                if let Ok(mut write_prev) = cursor.prev.upgrade() {
                    let node = Node::new(key, cursor.curr);
                    write_prev.store(node, Release);
                    return true;
                }
            }
        }
    }

    fn remove(&self, key: &T) -> bool {
        let guard = pin();
        loop {
            if let Ok((found, mut cursor)) = self.find(key, &guard) {
                if !found {
                    if cursor.prev.finish() {
                        return false;
                    }
                    continue;
                }
                if let Ok(mut write_prev) = cursor.prev.upgrade() {
                    let next_node = unsafe { &cursor.curr.as_ref().unwrap().next };
                    let next_guard = unsafe { next_node.read_lock() };
                    if let Ok(mut write_curr) = next_guard.upgrade() {
                        write_prev.store(write_curr.load(Acquire, &guard), Release);
                        unsafe { guard.defer_destroy(cursor.curr) };
                        return true;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Iter<'g, T> {
    // Can be dropped without validation, because the only way to use cursor.curr is next().
    cursor: ManuallyDrop<Cursor<'g, T>>,
    guard: &'g Guard,
}

impl<T> OptimisticFineGrainedListSet<T> {
    /// An iterator visiting all elements. `next()` returns `Some(Err(()))` when validation fails.
    /// In that case, the user must restart the iteration.
    pub fn iter<'g>(&'g self, guard: &'g Guard) -> Iter<'g, T> {
        Iter {
            cursor: ManuallyDrop::new(self.head(guard)),
            guard,
        }
    }
}

impl<'g, T> Iterator for Iter<'g, T> {
    type Item = Result<&'g T, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.cursor.curr;

        if current.is_null() {
            let prev_guard = unsafe { std::ptr::read(&self.cursor.prev) };
            if !prev_guard.finish() {
                return Some(Err(()));
            }
            return None;
        }
        let current_node = unsafe { current.as_ref().unwrap() };
        let next_guard = unsafe { current_node.next.read_lock() };
        let old_prev = mem::replace(&mut self.cursor.prev, next_guard);

        if !old_prev.finish() {
            return Some(Err(()));
        }

        self.cursor.curr = self.cursor.prev.load(Acquire, self.guard);

        Some(Ok(&current_node.data))
    }
}

impl<T> Drop for OptimisticFineGrainedListSet<T> {
    fn drop(&mut self) {
        let guard = pin();

        let head_lock = mem::replace(&mut self.head, SeqLock::new(Atomic::null()));
        let mut current_shared = head_lock.into_inner().load(Relaxed, &guard);

        while let Some(node_ref) = unsafe { current_shared.as_ref() } {
            let mut current_owned = unsafe { current_shared.into_owned() };
            let next_lock = mem::replace(&mut current_owned.next, SeqLock::new(Atomic::null()));
            current_shared = next_lock.into_inner().load(Relaxed, &guard);
        }
    }
}

impl<T> Default for OptimisticFineGrainedListSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
