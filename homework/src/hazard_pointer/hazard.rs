use core::ptr::{self, NonNull};
#[cfg(not(feature = "check-loom"))]
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering, fence};
use std::collections::HashSet;
use std::fmt;

#[cfg(feature = "check-loom")]
use loom::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering, fence};

use super::HAZARDS;

/// Represents the ownership of a hazard pointer slot.
pub struct Shield {
    slot: NonNull<HazardSlot>,
}

impl Shield {
    /// Creates a new shield for hazard pointer.
    pub fn new(hazards: &HazardBag) -> Self {
        let slot = hazards.acquire_slot().into();
        Self { slot }
    }

    /// Store `pointer` to the hazard slot.
    pub fn set<T>(&self, pointer: *mut T) {
        unsafe { self.slot.as_ref().hazard.store(pointer as *mut (), Ordering::SeqCst); }
    }

    /// Clear the hazard slot.
    pub fn clear(&self) {
        self.set(ptr::null_mut::<()>())
    }

    /// Check if `src` still points to `pointer`. If not, returns the current value.
    ///
    /// For a pointer `p`, if "`src` still pointing to `pointer`" implies that `p` is not retired,
    /// then `Ok(())` means that shields set to `p` are validated.
    pub fn validate<T>(pointer: *mut T, src: &AtomicPtr<T>) -> Result<(), *mut T> {
        let mut current_pointer = src.load(Ordering::SeqCst);
        if current_pointer == pointer {
            Ok(())
        } else {
            Err(current_pointer)
        }
    }

    /// Try protecting `pointer` obtained from `src`. If not, returns the current value.
    ///
    /// If "`src` still pointing to `pointer`" implies that `pointer` is not retired, then `Ok(())`
    /// means that this shield is validated.
    pub fn try_protect<T>(&self, pointer: *mut T, src: &AtomicPtr<T>) -> Result<(), *mut T> {
        self.set(pointer);
        Self::validate(pointer, src).inspect_err(|_| self.clear())
    }

    /// Get a protected pointer from `src`.
    ///
    /// See `try_protect()`.
    pub fn protect<T>(&self, src: &AtomicPtr<T>) -> *mut T {
        let mut pointer = src.load(Ordering::Relaxed);
        while let Err(new) = self.try_protect(pointer, src) {
            pointer = new;
            #[cfg(feature = "check-loom")]
            loom::sync::atomic::spin_loop_hint();
        }
        pointer
    }
}

impl Default for Shield {
    fn default() -> Self {
        Self::new(&HAZARDS)
    }
}

impl Drop for Shield {
    /// Clear and release the ownership of the hazard slot.
    fn drop(&mut self) {
        self.clear();
        unsafe { self.slot.as_ref().active.store(false, Ordering::SeqCst); }
    }
}

impl fmt::Debug for Shield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shield")
            .field("slot address", &self.slot)
            .field("slot data", unsafe { self.slot.as_ref() })
            .finish()
    }
}

/// Global bag (multiset) of hazards pointers.
/// `HazardBag.head` and `HazardSlot.next` form a grow-only list of all hazard slots. Slots are
/// never removed from this list. Instead, it gets deactivated and recycled for other `Shield`s.
#[derive(Debug)]
pub struct HazardBag {
    head: AtomicPtr<HazardSlot>,
}

/// See `HazardBag`
#[derive(Debug)]
struct HazardSlot {
    // Whether this slot is occupied by a `Shield`.
    active: AtomicBool,
    // Machine representation of the hazard pointer.
    hazard: AtomicPtr<()>,
    // Immutable pointer to the next slot in the bag.
    next: *const HazardSlot,
}

impl HazardSlot {
    fn new() -> Self {
        HazardSlot {
            active: AtomicBool::new(false),
            hazard: AtomicPtr::new(ptr::null_mut()),
            next: ptr::null(),
        }
    }
}

impl HazardBag {
    #[cfg(not(feature = "check-loom"))]
    /// Creates a new global hazard set.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    #[cfg(feature = "check-loom")]
    /// Creates a new global hazard set.
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Acquires a slot in the hazard set, either by recycling an inactive slot or allocating a new
    /// slot.
    fn acquire_slot(&self) -> &HazardSlot {
        if let Some(slot) = self.try_acquire_inactive() {
            return slot;
        }

        let mut new_slot = Box::into_raw(Box::new(HazardSlot::new()));
        let mut head_ptr = self.head.load(Ordering::Acquire);
        unsafe {
            (*new_slot).active.store(true, Ordering::Relaxed);
            (*new_slot).next = head_ptr;
        }

        loop {
             
            match self.head.compare_exchange(head_ptr, new_slot, Ordering::Release, Ordering::Relaxed) {
                Ok(_) => return unsafe { &*new_slot },
                Err(actual_head) => {
                    head_ptr = actual_head;
                    unsafe {
                        (*new_slot).next = actual_head;
                    }
                }
            }
        }
    }

    /// Find an inactive slot and activate it.
    fn try_acquire_inactive(&self) -> Option<&HazardSlot> {
        let mut slot_ptr: *const HazardSlot = self.head.load(Ordering::Acquire);
        
        while !slot_ptr.is_null() {
            unsafe {
                match (*slot_ptr).active.compare_exchange(
                    false, 
                    true, 
                    Ordering::AcqRel, 
                    Ordering::Relaxed
                ) {
                    Ok(_) => {
                        return Some(&*slot_ptr);
                    }
                    Err(_) => {
                        slot_ptr = (*slot_ptr).next;
                    }
                }
            }
        }
        
        None
    }

    /// Returns all the hazards in the set.
    pub fn all_hazards(&self) -> HashSet<*mut ()> {
        let mut hazards_set = HashSet::new();
        let mut slot_ptr: *const HazardSlot = self.head.load(Ordering::Acquire);
        
        while !slot_ptr.is_null() {
            unsafe {
                let is_active = (*slot_ptr).active.load(Ordering::Acquire);
                let hazard = (*slot_ptr).hazard.load(Ordering::Acquire);
                if is_active && !hazard.is_null() {
                    hazards_set.insert(hazard);
                }
                slot_ptr = (*slot_ptr).next;
            }
        }

        hazards_set
    }
}

impl Default for HazardBag {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for HazardBag {
    /// Frees all slots.
    fn drop(&mut self) {
        let mut slot_ptr = self.head.load(Ordering::Acquire);
        while !slot_ptr.is_null() {
            unsafe {
                let next_ptr = (*slot_ptr).next;
                let _ = Box::from_raw(slot_ptr);
                slot_ptr = next_ptr as *mut HazardSlot;
            }
        }
    }
}

unsafe impl Send for HazardSlot {}
unsafe impl Sync for HazardSlot {}

#[cfg(all(test, not(feature = "check-loom")))]
mod tests {
    use std::collections::HashSet;
    use std::ops::Range;
    use std::sync::Arc;
    use std::sync::atomic::AtomicPtr;
    use std::{mem, thread};

    use super::{HazardBag, Shield};

    const THREADS: usize = 8;
    const VALUES: Range<usize> = 1..1024;

    // `all_hazards` should return hazards protected by shield(s).
    #[test]
    fn all_hazards_protected() {
        let hazard_bag = Arc::new(HazardBag::new());
        (0..THREADS)
            .map(|_| {
                let hazard_bag = hazard_bag.clone();
                thread::spawn(move || {
                    for data in VALUES {
                        let src = AtomicPtr::new(data as *mut ());
                        let shield = Shield::new(&hazard_bag);
                        let _ = shield.protect(&src);
                        // leak the shield so that it is not unprotected.
                        mem::forget(shield);
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|th| th.join().unwrap());
        let all = hazard_bag.all_hazards();
        let values = VALUES.map(|data| data as *mut ()).collect();
        assert!(all.is_superset(&values))
    }

    // `all_hazards` should not return values that are no longer protected.
    #[test]
    fn all_hazards_unprotected() {
        let hazard_bag = Arc::new(HazardBag::new());
        (0..THREADS)
            .map(|_| {
                let hazard_bag = hazard_bag.clone();
                thread::spawn(move || {
                    for data in VALUES {
                        let src = AtomicPtr::new(data as *mut ());
                        let shield = Shield::new(&hazard_bag);
                        let _ = shield.protect(&src);
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|th| th.join().unwrap());
        let all = hazard_bag.all_hazards();
        let values = VALUES.map(|data| data as *mut ()).collect();
        let intersection: HashSet<_> = all.intersection(&values).collect();
        assert!(intersection.is_empty())
    }

    // `acquire_slot` should recycle existing slots.
    #[test]
    fn recycle_slots() {
        let hazard_bag = HazardBag::new();
        // allocate slots
        let shields = (0..1024)
            .map(|_| Shield::new(&hazard_bag))
            .collect::<Vec<_>>();
        // slot addresses
        let old_slots = shields
            .iter()
            .map(|s| s.slot.as_ptr() as usize)
            .collect::<HashSet<_>>();
        // release the slots
        drop(shields);

        let shields = (0..128)
            .map(|_| Shield::new(&hazard_bag))
            .collect::<Vec<_>>();
        let new_slots = shields
            .iter()
            .map(|s| s.slot.as_ptr() as usize)
            .collect::<HashSet<_>>();

        // no new slots should've been created
        assert!(new_slots.is_subset(&old_slots));
    }
}
