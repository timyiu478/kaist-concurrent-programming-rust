use std::cmp::Ordering::*;
use std::sync::{Mutex, MutexGuard};
use std::{mem, ptr};

use crate::ConcurrentSet;

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: Mutex<*mut Node<T>>,
}

/// Concurrent sorted singly linked list using fine-grained lock-coupling.
#[derive(Debug)]
pub struct FineGrainedListSet<T> {
    head: Mutex<*mut Node<T>>,
}

unsafe impl<T: Send> Send for FineGrainedListSet<T> {}
unsafe impl<T: Send> Sync for FineGrainedListSet<T> {}

/// Reference to the `next` field of previous node which points to the current node.
///
/// For example, given the following linked list:
///
/// ```text
/// head -> 1 -> 2 -> 3 -> null
/// ```
///
/// If `cursor` is currently at node 2, then `cursor.0` should be the `MutexGuard` obtained from the
/// `next` of node 1. In particular, `cursor.0.as_ref().unwrap()` creates a shared reference to node
/// 2.
struct Cursor<'l, T>(MutexGuard<'l, *mut Node<T>>);

impl<T> Node<T> {
    fn new(data: T, next: *mut Self) -> *mut Self {
        Box::into_raw(Box::new(Self {
            data,
            next: Mutex::new(next),
        }))
    }
}

impl<T: Ord> Cursor<'_, T> {
    /// Moves the cursor to the position of key in the sorted list.
    /// Returns whether the value was found.
    fn find(&mut self, key: &T) -> bool {
        unsafe {
            while let Some(node) = (*self.0).as_ref() {
                if node.data == *key {
                    return true;
                }
                if node.data > *key {
                    return false;
                }
                if let Ok(next_g) = node.next.lock() {
                    self.0 = next_g;
                } else {
                    break;
                }
            }
        }

        false
    }
}

impl<T> FineGrainedListSet<T> {
    /// Creates a new list.
    pub fn new() -> Self {
        Self {
            head: Mutex::new(ptr::null_mut()),
        }
    }
}

impl<T: Ord> FineGrainedListSet<T> {
    fn find(&self, key: &T) -> (bool, Cursor<'_, T>) {
        match self.head.lock() {
            Ok(g) => {
                let mut cursor = Cursor(g);
                (cursor.find(key), cursor)
            }
            Err(_) => panic!("Fail to acquire head lock"),
        }
    }
}

impl<T: Ord> ConcurrentSet<T> for FineGrainedListSet<T> {
    fn contains(&self, key: &T) -> bool {
        self.find(key).0
    }

    fn insert(&self, key: T) -> bool {
        let (found, mut cursor) = self.find(&key);

        if found {
            false
        } else {
            let target_ptr = *cursor.0;
            let new_node = Node::new(key, target_ptr);
            *cursor.0 = new_node;

            true
        }
    }

    fn remove(&self, key: &T) -> bool {
        let (found, mut cursor) = self.find(key);

        if !found {
            return false;
        }

        unsafe {
            let target_ptr = *cursor.0;
            if target_ptr.is_null() {
                return false;
            }
            let mut target_node = &*target_ptr;

            // lock the node that going to be removed
            if let Ok(next_g) = target_node.next.lock() {
                // update pointer
                *cursor.0 = *next_g;
            }

            // claim the ownership of target node and then drop it
            let _ = Box::from_raw(target_ptr);
        }

        true
    }
}

#[derive(Debug)]
pub struct Iter<'l, T> {
    cursor: MutexGuard<'l, *mut Node<T>>,
}

impl<T> FineGrainedListSet<T> {
    /// An iterator visiting all elements.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            cursor: self.head.lock().unwrap(),
        }
    }
}

impl<'l, T> Iterator for Iter<'l, T> {
    type Item = &'l T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if (*self.cursor).is_null() {
                return None;
            }
            let current_node: &Node<T> = &**self.cursor;
            if let Ok(next_g) = current_node.next.lock() {
                self.cursor = next_g;
                Some(&current_node.data)
            } else {
                None
            }
        }
    }
}

impl<T> Drop for FineGrainedListSet<T> {
    fn drop(&mut self) {
        // bypass mutex to get the mut pointer of the Node
        let mut current = *self.head.get_mut().unwrap();

        while !current.is_null() {
            unsafe {
                let mut node_box = Box::from_raw(current);
                current = *node_box.next.get_mut().unwrap();
            }
        }
    }
}

impl<T> Default for FineGrainedListSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
