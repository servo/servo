/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Different objects protected by the same lock

#[cfg(feature = "gecko")]
use atomic_refcell::{AtomicRefCell, AtomicRef, AtomicRefMut};
#[cfg(feature = "servo")]
use parking_lot::RwLock;
use servo_arc::Arc;
use std::cell::UnsafeCell;
use std::fmt;
#[cfg(feature = "gecko")]
use std::ptr;

/// A shared read/write lock that can protect multiple objects.
///
/// In Gecko builds, we don't need the blocking behavior, just the safety. As
/// such we implement this with an AtomicRefCell instead in Gecko builds,
/// which is ~2x as fast, and panics (rather than deadlocking) when things go
/// wrong (which is much easier to debug on CI).
///
/// Servo needs the blocking behavior for its unsynchronized animation setup,
/// but that may not be web-compatible and may need to be changed (at which
/// point Servo could use AtomicRefCell too).
#[derive(Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SharedRwLock {
    #[cfg(feature = "servo")]
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    arc: Arc<RwLock<()>>,

    #[cfg(feature = "gecko")]
    cell: Arc<AtomicRefCell<SomethingZeroSizedButTyped>>,
}

#[cfg(feature = "gecko")]
struct SomethingZeroSizedButTyped;

impl fmt::Debug for SharedRwLock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SharedRwLock")
    }
}

impl SharedRwLock {
    /// Create a new shared lock (servo).
    #[cfg(feature = "servo")]
    pub fn new() -> Self {
        SharedRwLock {
            arc: Arc::new(RwLock::new(()))
        }
    }

    /// Create a new shared lock (gecko).
    #[cfg(feature = "gecko")]
    pub fn new() -> Self {
        SharedRwLock {
            cell: Arc::new(AtomicRefCell::new(SomethingZeroSizedButTyped))
        }
    }

    /// Wrap the given data to make its access protected by this lock.
    pub fn wrap<T>(&self, data: T) -> Locked<T> {
        Locked {
            shared_lock: self.clone(),
            data: UnsafeCell::new(data),
        }
    }

    /// Obtain the lock for reading (servo).
    #[cfg(feature = "servo")]
    pub fn read(&self) -> SharedRwLockReadGuard {
        self.arc.raw_read();
        SharedRwLockReadGuard(self)
    }

    /// Obtain the lock for reading (gecko).
    #[cfg(feature = "gecko")]
    pub fn read(&self) -> SharedRwLockReadGuard {
        SharedRwLockReadGuard(self.cell.borrow())
    }

    /// Obtain the lock for writing (servo).
    #[cfg(feature = "servo")]
    pub fn write(&self) -> SharedRwLockWriteGuard {
        self.arc.raw_write();
        SharedRwLockWriteGuard(self)
    }

    /// Obtain the lock for writing (gecko).
    #[cfg(feature = "gecko")]
    pub fn write(&self) -> SharedRwLockWriteGuard {
        SharedRwLockWriteGuard(self.cell.borrow_mut())
    }
}

/// Proof that a shared lock was obtained for reading (servo).
#[cfg(feature = "servo")]
pub struct SharedRwLockReadGuard<'a>(&'a SharedRwLock);
/// Proof that a shared lock was obtained for writing (gecko).
#[cfg(feature = "gecko")]
pub struct SharedRwLockReadGuard<'a>(AtomicRef<'a, SomethingZeroSizedButTyped>);
#[cfg(feature = "servo")]
impl<'a> Drop for SharedRwLockReadGuard<'a> {
    fn drop(&mut self) {
        // Unsafe: self.lock is private to this module, only ever set after `raw_read()`,
        // and never copied or cloned (see `compile_time_assert` below).
        unsafe {
            self.0.arc.raw_unlock_read()
        }
    }
}

/// Proof that a shared lock was obtained for writing (servo).
#[cfg(feature = "servo")]
pub struct SharedRwLockWriteGuard<'a>(&'a SharedRwLock);
/// Proof that a shared lock was obtained for writing (gecko).
#[cfg(feature = "gecko")]
pub struct SharedRwLockWriteGuard<'a>(AtomicRefMut<'a, SomethingZeroSizedButTyped>);
#[cfg(feature = "servo")]
impl<'a> Drop for SharedRwLockWriteGuard<'a> {
    fn drop(&mut self) {
        // Unsafe: self.lock is private to this module, only ever set after `raw_write()`,
        // and never copied or cloned (see `compile_time_assert` below).
        unsafe {
            self.0.arc.raw_unlock_write()
        }
    }
}

/// Data protect by a shared lock.
pub struct Locked<T> {
    shared_lock: SharedRwLock,
    data: UnsafeCell<T>,
}

// Unsafe: the data inside `UnsafeCell` is only accessed in `read_with` and `write_with`,
// where guards ensure synchronization.
unsafe impl<T: Send> Send for Locked<T> {}
unsafe impl<T: Send + Sync> Sync for Locked<T> {}

impl<T: fmt::Debug> fmt::Debug for Locked<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let guard = self.shared_lock.read();
        self.read_with(&guard).fmt(f)
    }
}

impl<T> Locked<T> {
    #[cfg(feature = "servo")]
    fn same_lock_as(&self, lock: &SharedRwLock) -> bool {
        Arc::ptr_eq(&self.shared_lock.arc, &lock.arc)
    }

    #[cfg(feature = "gecko")]
    fn same_lock_as(&self, derefed_guard: &SomethingZeroSizedButTyped) -> bool {
        ptr::eq(self.shared_lock.cell.as_ptr(), derefed_guard)
    }

    /// Access the data for reading.
    pub fn read_with<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> &'a T {
        assert!(self.same_lock_as(&guard.0),
                "Locked::read_with called with a guard from an unrelated SharedRwLock");
        let ptr = self.data.get();

        // Unsafe:
        //
        // * The guard guarantees that the lock is taken for reading,
        //   and we’ve checked that it’s the correct lock.
        // * The returned reference borrows *both* the data and the guard,
        //   so that it can outlive neither.
        unsafe {
            &*ptr
        }
    }

    /// Access the data for writing.
    pub fn write_with<'a>(&'a self, guard: &'a mut SharedRwLockWriteGuard) -> &'a mut T {
        assert!(self.same_lock_as(&guard.0),
                "Locked::write_with called with a guard from an unrelated SharedRwLock");
        let ptr = self.data.get();

        // Unsafe:
        //
        // * The guard guarantees that the lock is taken for writing,
        //   and we’ve checked that it’s the correct lock.
        // * The returned reference borrows *both* the data and the guard,
        //   so that it can outlive neither.
        // * We require a mutable borrow of the guard,
        //   so that one write guard can only be used once at a time.
        unsafe {
            &mut *ptr
        }
    }
}

#[allow(dead_code)]
mod compile_time_assert {
    use super::{SharedRwLockReadGuard, SharedRwLockWriteGuard};

    trait Marker1 {}
    impl<T: Clone> Marker1 for T {}
    impl<'a> Marker1 for SharedRwLockReadGuard<'a> {}  // Assert SharedRwLockReadGuard: !Clone
    impl<'a> Marker1 for SharedRwLockWriteGuard<'a> {}  // Assert SharedRwLockWriteGuard: !Clone

    trait Marker2 {}
    impl<T: Copy> Marker2 for T {}
    impl<'a> Marker2 for SharedRwLockReadGuard<'a> {}  // Assert SharedRwLockReadGuard: !Copy
    impl<'a> Marker2 for SharedRwLockWriteGuard<'a> {}  // Assert SharedRwLockWriteGuard: !Copy
}

/// Like ToCss, but with a lock guard given by the caller.
pub trait ToCssWithGuard {
    /// Serialize `self` in CSS syntax, writing to `dest`, using the given lock guard.
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write;

    /// Serialize `self` in CSS syntax using the given lock guard and return a string.
    ///
    /// (This is a convenience wrapper for `to_css` and probably should not be overridden.)
    #[inline]
    fn to_css_string(&self, guard: &SharedRwLockReadGuard) -> String {
        let mut s = String::new();
        self.to_css(guard, &mut s).unwrap();
        s
    }
}

/// Parameters needed for deep clones.
#[cfg(feature = "gecko")]
pub struct DeepCloneParams {
    /// The new sheet we're cloning rules into.
    pub reference_sheet: *const ::gecko_bindings::structs::ServoStyleSheet,
}

/// Parameters needed for deep clones.
#[cfg(feature = "servo")]
pub struct DeepCloneParams;


/// A trait to do a deep clone of a given CSS type. Gets a lock and a read
/// guard, in order to be able to read and clone nested structures.
pub trait DeepCloneWithLock : Sized {
    /// Deep clones this object.
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self;
}

/// Guards for a document
#[derive(Clone)]
pub struct StylesheetGuards<'a> {
    /// For author-origin stylesheets
    pub author: &'a SharedRwLockReadGuard<'a>,

    /// For user-agent-origin and user-origin stylesheets
    pub ua_or_user: &'a SharedRwLockReadGuard<'a>,
}

impl<'a> StylesheetGuards<'a> {
    /// Same guard for all origins
    pub fn same(guard: &'a SharedRwLockReadGuard<'a>) -> Self {
        StylesheetGuards {
            author: guard,
            ua_or_user: guard,
        }
    }
}
