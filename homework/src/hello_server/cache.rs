//! Thread-safe key/value cache.

use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Mutex, RwLock};

/// Cache that remembers the result for each key.
#[derive(Debug)]
pub struct Cache<K, V> {
    // todo! This is an example cache type. Build your own cache type that satisfies the
    // specification for `get_or_insert_with`.
    inner: Mutex<HashMap<K, Arc<RwLock<Option<V>>>>>,
}

impl<K, V> Default for Cache<K, V> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    /// Retrieve the value or insert a new one created by `f`.
    ///
    /// An invocation to this function should not block another invocation with a different key. For
    /// example, if a thread calls `get_or_insert_with(key1, f1)` and another thread calls
    /// `get_or_insert_with(key2, f2)` (`key1≠key2`, `key1,key2∉cache`) concurrently, `f1` and `f2`
    /// should run concurrently.
    ///
    /// On the other hand, since `f` may consume a lot of resource (= money), it's undesirable to
    /// duplicate the work. That is, `f` should be run only once for each key. Specifically, even
    /// for concurrent invocations of `get_or_insert_with(key, f)`, `f` is called only once per key.
    ///
    /// Hint: the [`Entry`] API may be useful in implementing this function.
    ///
    /// [`Entry`]: https://doc.rust-lang.org/stable/std/collections/hash_map/struct.HashMap.html#method.entry
    pub fn get_or_insert_with<F: FnOnce(K) -> V>(&self, key: K, f: F) -> V {
        // 1. lock the hash map
        let mut map_lock = self.inner.lock().unwrap();

        // 2. gets a reference to the value in the entry
        let slot = match map_lock.entry(key.clone()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let new_slot = Arc::new(RwLock::new(None));
                entry.insert(new_slot.clone());
                new_slot
            }
        };

        // 3. drop the map lock to allow concurrent access with different keys
        drop(map_lock);

        // 4. fast path
        let read_lock = slot.read().unwrap();
        if let Some(value) = &*read_lock {
            return value.clone();
        }
        drop(read_lock);

        // 5. slow path
        let mut write_lock = slot.write().unwrap();
        if let Some(value) = &*write_lock {
            return value.clone();
        }

        let computed_value = f(key);
        *write_lock = Some(computed_value.clone());
        computed_value
    }
}
