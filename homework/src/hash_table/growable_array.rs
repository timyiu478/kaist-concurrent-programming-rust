//! Growable array.

use core::fmt::Debug;
use core::mem::{self, ManuallyDrop};
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{Atomic, Guard, Owned, Shared};

/// Growable array of `Atomic<T>`.
///
/// This is more complete version of the dynamic sized array from the paper. In the paper, the
/// segment table is an array of arrays (segments) of pointers to the elements. In this
/// implementation, a segment contains the pointers to the elements **or other child segments**. In
/// other words, it is a tree that has segments as internal nodes.
///
/// # Example run
///
/// Suppose `SEGMENT_LOGSIZE = 3` (segment size 8).
///
/// When a new `GrowableArray` is created, `root` is initialized with `Atomic::null()`.
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
/// ```
///
/// When you store element `cat` at the index `0b001`, it first initializes a segment.
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
///                            | height: 1
///                            v
///                 +---+---+---+---+---+---+---+---+
///                 |111|110|101|100|011|010|001|000|
///                 +---+---+---+---+---+---+---+---+
///                                           |
///                                           v
///                                         +---+
///                                         |cat|
///                                         +---+
/// ```
///
/// When you store `fox` at `0b111011`, it is clear that there is no room for indices larger than
/// `0b111`. So it first allocates another segment for upper 3 bits and moves the previous root
/// segment (`0b000XXX` segment) under the `0b000XXX` branch of the the newly allocated segment.
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
///                            | height: 2
///                            v
///                 +---+---+---+---+---+---+---+---+
///                 |111|110|101|100|011|010|001|000|
///                 +---+---+---+---+---+---+---+---+
///                                               |
///                                               v
///                                      +---+---+---+---+---+---+---+---+
///                                      |111|110|101|100|011|010|001|000|
///                                      +---+---+---+---+---+---+---+---+
///                                                                |
///                                                                v
///                                                              +---+
///                                                              |cat|
///                                                              +---+
/// ```
///
/// And then, it allocates another segment for `0b111XXX` indices.
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
///                            | height: 2
///                            v
///                 +---+---+---+---+---+---+---+---+
///                 |111|110|101|100|011|010|001|000|
///                 +---+---+---+---+---+---+---+---+
///                   |                           |
///                   v                           v
/// +---+---+---+---+---+---+---+---+    +---+---+---+---+---+---+---+---+
/// |111|110|101|100|011|010|001|000|    |111|110|101|100|011|010|001|000|
/// +---+---+---+---+---+---+---+---+    +---+---+---+---+---+---+---+---+
///                   |                                            |
///                   v                                            v
///                 +---+                                        +---+
///                 |fox|                                        |cat|
///                 +---+                                        +---+
/// ```
///
/// Finally, when you store `owl` at `0b000110`, it traverses through the `0b000XXX` branch of the
/// height 2 segment and arrives at its `0b110` leaf.
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
///                            | height: 2
///                            v
///                 +---+---+---+---+---+---+---+---+
///                 |111|110|101|100|011|010|001|000|
///                 +---+---+---+---+---+---+---+---+
///                   |                           |
///                   v                           v
/// +---+---+---+---+---+---+---+---+    +---+---+---+---+---+---+---+---+
/// |111|110|101|100|011|010|001|000|    |111|110|101|100|011|010|001|000|
/// +---+---+---+---+---+---+---+---+    +---+---+---+---+---+---+---+---+
///                   |                        |                   |
///                   v                        v                   v
///                 +---+                    +---+               +---+
///                 |fox|                    |owl|               |cat|
///                 +---+                    +---+               +---+
/// ```
///
/// When the array is dropped, only the segments are dropped and the **elements must not be
/// dropped/deallocated**.
///
/// ```text
///                 +---+                    +---+               +---+
///                 |fox|                    |owl|               |cat|
///                 +---+                    +---+               +---+
/// ```
///
/// Instead, it should be handled by the container that the elements actually belong to. For
/// example, in `SplitOrderedList` the destruction of elements are handled by the inner `List`.
#[derive(Debug)]
pub struct GrowableArray<T> {
    root: Atomic<Segment<T>>,
}

const SEGMENT_LOGSIZE: usize = 10;

/// A fixed size array of atomic pointers to other `Segment<T>` or `T`.
///
/// Each segment is either an inner segment with pointers to other, children `Segment<T>` or an
/// element segment with pointers to `T`. This is determined by the height of this segment in the
/// main array, which one needs to track separately. For example, use the main array root's tag.
///
/// Since destructing segments requires its height information, it is not recommended to implement
/// [`Drop`]. Rather, implement and use the custom [`Segment::deallocate`] method that accounts for
/// the height of the segment.
union Segment<T> {
    children: ManuallyDrop<[Atomic<Segment<T>>; 1 << SEGMENT_LOGSIZE]>,
    elements: ManuallyDrop<[Atomic<T>; 1 << SEGMENT_LOGSIZE]>,
}

impl<T> Segment<T> {
    /// Create a new segment filled with null pointers. It is up to the callee to whether to use
    /// this as an intermediate or an element segment.
    fn new() -> Owned<Self> {
        Owned::new(
            // SAFETY: An array of null pointers can be interperted as either an intermediate
            // segment or an element segment.
            unsafe { mem::zeroed() },
        )
    }

    /// Deallocates a segment of `height`.
    ///
    /// # Safety
    ///
    /// - `self` must actually have height `height`.
    /// - There should be no other references to possible children segments.
    unsafe fn deallocate(self, height: usize) {
        // Base Case: If height is 1, this segment contains actual `elements`.
        if height <= 1 {
            return;
        }

        let childen = unsafe { ManuallyDrop::into_inner(self.children) };

        for segment_atomic in childen {
            let mut ptr = unsafe { segment_atomic.load(Relaxed, crossbeam_epoch::unprotected()) };
            if !ptr.is_null() {
                unsafe {
                    let child_owned = ptr.into_owned();
                    let child_boxed = child_owned.into_box();
                    child_boxed.deallocate(height - 1);
                }
            }
        }
    }
}

impl<T> Debug for Segment<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Segment")
    }
}

impl<T> Drop for GrowableArray<T> {
    /// Deallocate segments, but not the individual elements.
    fn drop(&mut self) {
        unsafe {
            let root_shared = self.root.load(Relaxed, crossbeam_epoch::unprotected());
            let height = root_shared.tag();
            let root_owned = root_shared.into_owned();
            let root_boxed = root_owned.into_box();
            root_boxed.deallocate(height);
        };
    }
}

impl<T> Default for GrowableArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GrowableArray<T> {
    /// Create a new growable array.
    pub fn new() -> Self {
        Self {
            root: Atomic::null(),
        }
    }

    /// Returns the reference to the `Atomic` pointer at `index`. Allocates new segments if
    /// necessary.
    pub fn get<'g>(&self, index: usize, guard: &'g Guard) -> &'g Atomic<T> {
        // 1. Calculate path (chunks from leaf to root)
        let mut path = Vec::new();
        let mut idx = index;
        let mask = (1 << SEGMENT_LOGSIZE) - 1;
        loop {
            path.push(idx & mask);
            idx >>= SEGMENT_LOGSIZE;
            if idx == 0 {
                break;
            }
        }

        // 2. Grow the tree upwards if necessary
        let mut root_shared = self.root.load(Acquire, guard);
        loop {
            let current_height = if root_shared.is_null() {
                0
            } else {
                root_shared.tag()
            };
            let target_height = path.len(); // Total height required for this path

            if target_height > current_height || root_shared.is_null() {
                // Prepare the base new root segment layer locally
                let mut current_owned = Segment::new();
                unsafe {
                    current_owned.children[0].store(root_shared, Relaxed);
                }

                // If the height gap is greater than 1, build all intermediate parent layers locally
                for h in (current_height + 2)..=target_height {
                    let mut parent_owned = Segment::new();
                    let current_tagged = current_owned.with_tag(h - 1); // Tag child with its height
                    unsafe {
                        parent_owned.children[0].store(current_tagged, Relaxed);
                    }
                    current_owned = parent_owned;
                }

                // Finalize the top of our newly prepared chain with the target height tag
                let new_root_tagged = current_owned.with_tag(target_height);

                // Attempt a single atomic swap to install the entire multi-level chain
                match self.root.compare_exchange(
                    root_shared,
                    new_root_tagged,
                    Release,
                    Acquire,
                    guard,
                ) {
                    Ok(shared) => {
                        root_shared = shared;
                        break; // Growth succeeded
                    }
                    Err(e) => {
                        // Another thread grew the tree first.
                        // e.new (our chain) safely drops out of scope. Retry with updated root.
                        root_shared = e.current;
                    }
                }
            } else {
                break; // Tree is already tall enough
            }
        }

        // 3. Traverse downwards to the leaf segment (Height == 1)
        let mut current_node = root_shared;
        let mut current_height = current_node.tag();

        while current_height > 1 {
            // If the tree is currently taller than our computed path, pad upper layers with index 0
            let chunk = if current_height <= path.len() {
                path[current_height - 1]
            } else {
                0
            };

            let child_atomic = unsafe {
                // Safe because current_height > 1, meaning it is an internal branch node
                let segment = current_node.deref();
                &(*segment.children)[chunk]
            };

            let mut child_shared = child_atomic.load(Acquire, guard);

            // Lazily allocate missing child nodes downward using Compare-and-Swap
            if child_shared.is_null() {
                let new_child = Segment::new().with_tag(current_height - 1);

                match child_atomic.compare_exchange(
                    Shared::null(),
                    new_child,
                    Release,
                    Acquire,
                    guard,
                ) {
                    Ok(shared) => child_shared = shared,
                    Err(e) => child_shared = e.current, // Another thread allocated it first
                }
            }

            current_node = child_shared;
            current_height -= 1;
        }

        // 4. We are at height 1 (the leaf segment containing elements). Return the element
        //    reference.
        let leaf_chunk = path[0];
        unsafe {
            let segment = current_node.deref();
            &(*segment.elements)[leaf_chunk]
        }
    }
}
