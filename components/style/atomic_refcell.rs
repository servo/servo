/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements a container type providing RefCell-like semantics for objects
//! shared across threads.
//!
//! RwLock is traditionally considered to be the |Sync| analogue of RefCell.
//! However, for consumers that can guarantee that they will never mutably
//! borrow the contents concurrently with immutable borrows, an RwLock is
//! overkill, and has key disadvantages:
//! * Performance: Even the fastest existing implementation of RwLock (that of
//!   parking_lot) performs at least two atomic operations during immutable
//!   borrows. This makes mutable borrows significantly cheaper than immutable
//!   borrows, leading to weird incentives when writing performance-critical
//!   code.
//! * Features: Implementing AtomicRefCell on top of RwLock makes it impossible
//!   to implement useful things like AtomicRef{,Mut}::map.
//!
//! As such, we re-implement RefCell semantics from scratch with a single atomic
//! reference count. The primary complication of this scheme relates to keeping
//! things in a consistent state when one thread performs and illegal borrow and
//! panics. Since an AtomicRefCell can be accessed by multiple threads, and since
//! panics are recoverable, we need to ensure that an illegal (pancking) access by
//! one thread does not lead to undefined behavior on other, still-running threads.
//!
//! So we represent things as follows:
//! * Any value with the high bit set (so half the total refcount space) indicates
//!   a mutable borrow.
//! * Mutable borrows perform an atomic compare-and-swap, swapping in the high bit
//!   if the current value is zero. If the current value is non-zero, the thread
//!   panics and the value is left undisturbed.
//! * Immutable borrows perform an atomic increment. If the new value has the high
//!   bit set, the thread panics. The incremented refcount is left as-is, since it
//!   still represents a valid mutable borrow. When the mutable borrow is released,
//!   the refcount is set unconditionally to zero, clearing any stray increments by
//!   panicked threads.
//!
//! There are a few additional purely-academic complications to handle overflow,
//! which are documented in the implementation.
//!
//! The rest of this module is mostly derived by copy-pasting the implementation of
//! RefCell and fixing things up as appropriate. Certain non-threadsafe methods
//! have been removed. We segment the concurrency logic from the rest of the code to
//! keep the tricky parts small and easy to audit.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use std::cell::UnsafeCell;
use std::cmp;
use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;

/// A threadsafe analogue to RefCell.
pub struct AtomicRefCell<T: ?Sized> {
    borrow: AtomicUsize,
    value: UnsafeCell<T>,
}

impl<T> AtomicRefCell<T> {
    /// Creates a new `AtomicRefCell` containing `value`.
    #[inline]
    pub fn new(value: T) -> AtomicRefCell<T> {
        AtomicRefCell {
            borrow: AtomicUsize::new(0),
            value: UnsafeCell::new(value),
        }
    }

    /// Consumes the `AtomicRefCell`, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        debug_assert!(self.borrow.load(atomic::Ordering::Acquire) == 0);
        unsafe { self.value.into_inner() }
    }
}

impl<T: ?Sized> AtomicRefCell<T> {
    /// Immutably borrows the wrapped value.
    #[inline]
    pub fn borrow(&self) -> AtomicRef<T> {
        AtomicRef {
                value: unsafe { &*self.value.get() },
                borrow: AtomicBorrowRef::new(&self.borrow),
        }
    }

    /// Mutably borrows the wrapped value.
    #[inline]
    pub fn borrow_mut(&self) -> AtomicRefMut<T> {
        AtomicRefMut {
                value: unsafe { &mut *self.value.get() },
                borrow: AtomicBorrowRefMut::new(&self.borrow),
        }
    }

    /// Returns a raw pointer to the underlying data in this cell.
    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.value.get()
    }
}

//
// Core synchronization logic. Keep this section small and easy to audit.
//

const HIGH_BIT: usize = !(::std::usize::MAX >> 1);
const MAX_FAILED_BORROWS: usize = HIGH_BIT + (HIGH_BIT >> 1);

struct AtomicBorrowRef<'b> {
    borrow: &'b AtomicUsize,
}

impl<'b> AtomicBorrowRef<'b> {
    #[inline]
    fn new(borrow: &'b AtomicUsize) -> Self {
        let new = borrow.fetch_add(1, atomic::Ordering::AcqRel) + 1;

        // If the new count has the high bit set, panic. The specifics of how
        // we panic is interesting for soundness, but irrelevant for real programs.
        if new & HIGH_BIT != 0 {
            if new == HIGH_BIT {
                // We overflowed into the reserved upper half of the refcount
                // space. Before panicking, decrement the refcount to leave things
                // in a consistent immutable-borrow state.
                //
                // This will never happen in a real program.
                borrow.fetch_sub(1, atomic::Ordering::AcqRel);
                panic!("too many immutable borrows");
            } else if new >= MAX_FAILED_BORROWS {
                // During the mutable borrow, an absurd number of threads have
                // incremented the refcount and panicked. To avoid hypothetically
                // wrapping the refcount, we abort the process once a certain
                // threshold is reached.
                //
                // This will never happen in a real program.
                //
                // FIXME(bholley): We can't currently whole-process abort in
                // stable Rust, but it's coming soon. We should fix this when
                // https://github.com/rust-lang/rust/issues/37838 hits release.
                panic!("Too many failed borrows");
            } else {
                // This is the normal case, and the only one which should happen
                // in a real program.
                panic!("already mutably borrowed");
            }
        }

        AtomicBorrowRef { borrow: borrow }
    }
}

impl<'b> Drop for AtomicBorrowRef<'b> {
    #[inline]
    fn drop(&mut self) {
        let old = self.borrow.fetch_sub(1, atomic::Ordering::AcqRel);
        // This assertion is technically incorrect in the case where another
        // thread hits the hypothetical overflow case, since we might observe
        // the refcount before it fixes it up (and panics). But that never will
        // never happen in a real program, and this is a debug_assert! anyway.
        debug_assert!(old & HIGH_BIT == 0);
    }
}

struct AtomicBorrowRefMut<'b> {
    borrow: &'b AtomicUsize,
}

impl<'b> Drop for AtomicBorrowRefMut<'b> {
    #[inline]
    fn drop(&mut self) {
        let old = self.borrow.swap(0, atomic::Ordering::AcqRel);
        // The old value may be slightly higher than HIGH_BIT if another thread
        // tried to immutably borrow and panicked.
        debug_assert!(old >= HIGH_BIT);
    }
}

impl<'b> AtomicBorrowRefMut<'b> {
    #[inline]
    fn new(borrow: &'b AtomicUsize) -> AtomicBorrowRefMut<'b> {
        // Use compare-and-swap to avoid corrupting the immutable borrow count
        // on illegal mutable borrows.
        let old = borrow.compare_and_swap(0, HIGH_BIT, atomic::Ordering::AcqRel);
        assert!(old == 0, "already borrowed");
        AtomicBorrowRefMut {
            borrow: borrow
        }
    }
}

unsafe impl<T: ?Sized + Send + Sync> Send for AtomicRefCell<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for AtomicRefCell<T> {}

//
// End of core synchronization logic. No tricky thread stuff allowed below
// this point.
//

impl<T: Clone> Clone for AtomicRefCell<T> {
    #[inline]
    fn clone(&self) -> AtomicRefCell<T> {
        AtomicRefCell::new(self.borrow().clone())
    }
}

impl<T: Default> Default for AtomicRefCell<T> {
    #[inline]
    fn default() -> AtomicRefCell<T> {
        AtomicRefCell::new(Default::default())
    }
}

impl<T: ?Sized + PartialEq> PartialEq for AtomicRefCell<T> {
    #[inline]
    fn eq(&self, other: &AtomicRefCell<T>) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: ?Sized + Eq> Eq for AtomicRefCell<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for AtomicRefCell<T> {
    #[inline]
    fn partial_cmp(&self, other: &AtomicRefCell<T>) -> Option<cmp::Ordering> {
        self.borrow().partial_cmp(&*other.borrow())
    }

    #[inline]
    fn lt(&self, other: &AtomicRefCell<T>) -> bool {
        *self.borrow() < *other.borrow()
    }

    #[inline]
    fn le(&self, other: &AtomicRefCell<T>) -> bool {
        *self.borrow() <= *other.borrow()
    }

    #[inline]
    fn gt(&self, other: &AtomicRefCell<T>) -> bool {
        *self.borrow() > *other.borrow()
    }

    #[inline]
    fn ge(&self, other: &AtomicRefCell<T>) -> bool {
        *self.borrow() >= *other.borrow()
    }
}

impl<T: ?Sized + Ord> Ord for AtomicRefCell<T> {
    #[inline]
    fn cmp(&self, other: &AtomicRefCell<T>) -> cmp::Ordering {
        self.borrow().cmp(&*other.borrow())
    }
}

impl<T> From<T> for AtomicRefCell<T> {
    fn from(t: T) -> AtomicRefCell<T> {
        AtomicRefCell::new(t)
    }
}

impl<'b> Clone for AtomicBorrowRef<'b> {
    #[inline]
    fn clone(&self) -> AtomicBorrowRef<'b> {
        AtomicBorrowRef::new(self.borrow)
    }
}

/// A wrapper type for an immutably borrowed value from an `AtomicRefCell<T>`.
pub struct AtomicRef<'b, T: ?Sized + 'b> {
    value: &'b T,
    borrow: AtomicBorrowRef<'b>,
}


impl<'b, T: ?Sized> Deref for AtomicRef<'b, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'b, T: ?Sized> AtomicRef<'b, T> {
    /// Copies an `AtomicRef`.
    #[inline]
    pub fn clone(orig: &AtomicRef<'b, T>) -> AtomicRef<'b, T> {
        AtomicRef {
            value: orig.value,
            borrow: orig.borrow.clone(),
        }
    }

    /// Make a new `AtomicRef` for a component of the borrowed data.
    #[inline]
    pub fn map<U: ?Sized, F>(orig: AtomicRef<'b, T>, f: F) -> AtomicRef<'b, U>
        where F: FnOnce(&T) -> &U
    {
        AtomicRef {
            value: f(orig.value),
            borrow: orig.borrow,
        }
    }
}

impl<'b, T: ?Sized> AtomicRefMut<'b, T> {
    /// Make a new `AtomicRefMut` for a component of the borrowed data, e.g. an enum
    /// variant.
    #[inline]
    pub fn map<U: ?Sized, F>(orig: AtomicRefMut<'b, T>, f: F) -> AtomicRefMut<'b, U>
        where F: FnOnce(&mut T) -> &mut U
    {
        AtomicRefMut {
            value: f(orig.value),
            borrow: orig.borrow,
        }
    }
}

/// A wrapper type for a mutably borrowed value from an `AtomicRefCell<T>`.
pub struct AtomicRefMut<'b, T: ?Sized + 'b> {
    value: &'b mut T,
    borrow: AtomicBorrowRefMut<'b>,
}

impl<'b, T: ?Sized> Deref for AtomicRefMut<'b, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'b, T: ?Sized> DerefMut for AtomicRefMut<'b, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

impl<'b, T: ?Sized + Debug + 'b> Debug for AtomicRef<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
     }
}

impl<'b, T: ?Sized + Debug + 'b> Debug for AtomicRefMut<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}
