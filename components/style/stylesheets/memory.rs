/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Memory reporting for the style system when running inside of Gecko.

#[cfg(feature = "gecko")]
use gecko_bindings::bindings::Gecko_HaveSeenPtr;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::SeenPtrs;
#[cfg(feature = "gecko")]
use hash::HashMap;
#[cfg(feature = "gecko")]
use servo_arc::Arc;
use shared_lock::SharedRwLockReadGuard;
use smallvec::{Array, SmallVec};
use std::collections::HashSet;
use std::hash::{BuildHasher, Hash};
use std::os::raw::c_void;

/// Like gecko_bindings::structs::MallocSizeOf, but without the Option<>
/// wrapper.
///
/// Note that functions of this type should be called via do_malloc_size_of(),
/// rather than directly.
#[derive(Clone, Copy)]
pub struct MallocSizeOfFn(pub unsafe extern "C" fn(ptr: *const c_void) -> usize);

/// Like MallocSizeOfFn, but can take an interior pointer.
#[derive(Clone, Copy)]
pub struct MallocEnclosingSizeOfFn(pub unsafe extern "C" fn(ptr: *const c_void) -> usize);

/// Servo-side counterpart to mozilla::SizeOfState. The only difference is that
/// this struct doesn't contain the SeenPtrs table, just a pointer to it.
#[cfg(feature = "gecko")]
pub struct SizeOfState {
    /// Function that measures the size of heap blocks.
    pub malloc_size_of: MallocSizeOfFn,
    /// Table recording heap blocks that have already been measured.
    pub seen_ptrs: *mut SeenPtrs,
}

/// Check if an allocation is empty.
pub unsafe fn is_empty<T>(ptr: *const T) -> bool {
    return ptr as usize <= ::std::mem::align_of::<T>();
}

/// Call malloc_size_of on ptr, first checking that the allocation isn't empty.
pub unsafe fn do_malloc_size_of<T>(malloc_size_of: MallocSizeOfFn, ptr: *const T) -> usize {
    if is_empty(ptr) {
        0
    } else {
        (malloc_size_of.0)(ptr as *const c_void)
    }
}

/// Call malloc_enclosing_size_of on ptr, which must not be empty.
pub unsafe fn do_malloc_enclosing_size_of<T>(
    malloc_enclosing_size_of: MallocEnclosingSizeOfFn, ptr: *const T) -> usize
{
    assert!(!is_empty(ptr));
    (malloc_enclosing_size_of.0)(ptr as *const c_void)
}

/// Trait for measuring the size of heap data structures.
pub trait MallocSizeOf {
    /// Measure the size of any heap-allocated structures that hang off this
    /// value, but not the space taken up by the value itself.
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize;
}

/// Like MallocSizeOf, but takes a SizeOfState which allows it to measure
/// graph-like structures such as those containing Arcs.
#[cfg(feature = "gecko")]
pub trait MallocSizeOfWithRepeats {
    /// Measure the size of any heap-allocated structures that hang off this
    /// value, but not the space taken up by the value itself.
    fn malloc_size_of_children(&self, state: &mut SizeOfState) -> usize;
}

/// Like MallocSizeOf, but operates with the global SharedRwLockReadGuard
/// locked.
pub trait MallocSizeOfWithGuard {
    /// Like MallocSizeOf::malloc_size_of_children, but with a |guard| argument.
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn
    ) -> usize;
}

impl<A: MallocSizeOf, B: MallocSizeOf> MallocSizeOf for (A, B) {
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        self.0.malloc_size_of_children(malloc_size_of) +
            self.1.malloc_size_of_children(malloc_size_of)
    }
}

impl<T: MallocSizeOf> MallocSizeOf for Vec<T> {
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        self.iter().fold(
            unsafe { do_malloc_size_of(malloc_size_of, self.as_ptr()) },
            |n, elem| n + elem.malloc_size_of_children(malloc_size_of))
    }
}

#[cfg(feature = "gecko")]
impl<T: MallocSizeOfWithRepeats> MallocSizeOfWithRepeats for Arc<T> {
    fn malloc_size_of_children(&self, state: &mut SizeOfState) -> usize {
        let mut n = 0;
        let heap_ptr = self.heap_ptr();
        if unsafe { !is_empty(heap_ptr) && !Gecko_HaveSeenPtr(state.seen_ptrs, heap_ptr) } {
            n += unsafe { (state.malloc_size_of.0)(heap_ptr) };
            n += (**self).malloc_size_of_children(state);
        }
        n
    }
}

impl<T: MallocSizeOfWithGuard> MallocSizeOfWithGuard for Vec<T> {
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn,
    ) -> usize {
        self.iter().fold(
            unsafe { do_malloc_size_of(malloc_size_of, self.as_ptr()) },
            |n, elem| n + elem.malloc_size_of_children(guard, malloc_size_of))
    }
}

/// Trait for measuring the heap usage of a Box<T>.
pub trait MallocSizeOfBox {
    /// Measure shallowly the size of the memory used by the T -- anything
    /// pointed to by the T must be measured separately.
    fn malloc_shallow_size_of_box(&self, malloc_size_of: MallocSizeOfFn) -> usize;
}

impl<T> MallocSizeOfBox for Box<T> {
    fn malloc_shallow_size_of_box(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        unsafe { do_malloc_size_of(malloc_size_of, &**self as *const T) }
    }
}

/// Trait for measuring the heap usage of a vector.
pub trait MallocSizeOfVec {
    /// Measure shallowly the size of the memory used by the Vec's elements --
    /// anything pointed to by the elements must be measured separately, using
    /// iteration.
    fn malloc_shallow_size_of_vec(&self, malloc_size_of: MallocSizeOfFn) -> usize;
}

impl<T> MallocSizeOfVec for Vec<T> {
    fn malloc_shallow_size_of_vec(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        unsafe { do_malloc_size_of(malloc_size_of, self.as_ptr()) }
    }
}

impl<A: Array> MallocSizeOfVec for SmallVec<A> {
    fn malloc_shallow_size_of_vec(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        if self.spilled() {
            unsafe { do_malloc_size_of(malloc_size_of, self.as_ptr()) }
        } else {
            0
        }
    }
}

/// Trait for measuring the heap usage of a hash table.
pub trait MallocSizeOfHash {
    /// Measure shallowly the size of the memory used within a hash table --
    /// anything pointer to by the keys and values must be measured separately,
    /// using iteration.
    fn malloc_shallow_size_of_hash(&self, malloc_enclosing_size_of: MallocEnclosingSizeOfFn)
                                   -> usize;
}

impl<T, S> MallocSizeOfHash for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    fn malloc_shallow_size_of_hash(&self, malloc_enclosing_size_of: MallocEnclosingSizeOfFn)
                                   -> usize {
        // The first value from the iterator gives us an interior pointer.
        // malloc_enclosing_size_of() then gives us the storage size. This
        // assumes that the HashSet's contents (values and hashes) are all
        // stored in a single contiguous heap allocation.
        let mut n = 0;
        for v in self.iter() {
            n += unsafe { do_malloc_enclosing_size_of(malloc_enclosing_size_of, v as *const T) };
            break;
        }
        n
    }
}

#[cfg(feature = "gecko")]
impl<K, V, S> MallocSizeOfHash for HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn malloc_shallow_size_of_hash(&self, malloc_enclosing_size_of: MallocEnclosingSizeOfFn)
                                   -> usize {
        // The first value from the iterator gives us an interior pointer.
        // malloc_enclosing_size_of() then gives us the storage size. This
        // assumes that the HashMap's contents (keys, values, and hashes) are
        // all stored in a single contiguous heap allocation.
        let mut n = 0;
        for v in self.values() {
            n += unsafe { do_malloc_enclosing_size_of(malloc_enclosing_size_of, v as *const V) };
            break;
        }
        n
    }
}
