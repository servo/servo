/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Different objects protected by the same lock

use parking_lot::RwLock;
use std::cell::UnsafeCell;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

/// A shared read/write lock that can protect multiple objects.
#[derive(Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SharedRwLock {
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    arc: Arc<RwLock<()>>,
}

impl fmt::Debug for SharedRwLock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SharedRwLock")
    }
}

impl SharedRwLock {
    /// Create a new shared lock
    pub fn new() -> Self {
        SharedRwLock {
            arc: Arc::new(RwLock::new(()))
        }
    }

    /// Wrap the given data to make its access protected by this lock.
    pub fn wrap<T>(&self, data: T) -> Locked<T> {
        Locked {
            shared_lock: self.clone(),
            data: UnsafeCell::new(data),
        }
    }

    /// Obtain the lock for reading
    pub fn read(&self) -> SharedRwLockReadGuard {
        self.arc.raw_read();
        SharedRwLockReadGuard {
            shared_lock: self
        }
    }

    /// Obtain the lock for writing
    pub fn write(&self) -> SharedRwLockWriteGuard {
        self.arc.raw_write();
        SharedRwLockWriteGuard {
            shared_lock: self
        }
    }

    fn same_as(&self, other: &Self) -> bool {
        // FIXME: Use Arc::ptr_eq once it’s stable. https://github.com/rust-lang/rust/issues/36497
        let a: *const RwLock<()> = Arc::deref(&self.arc);
        let b: *const RwLock<()> = Arc::deref(&other.arc);
        a == b
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
    /// Access the data for reading.
    pub fn read_with<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> &'a T {
        assert!(self.shared_lock.same_as(&guard.shared_lock),
                "StyleRef::read_with called with a guard from an unrelated SharedRwLock");
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
        assert!(self.shared_lock.same_as(&guard.shared_lock),
                "StyleRef::write_with called with a guard from an unrelated SharedRwLock");
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

/// Proof that a shared lock was obtained for reading.
pub struct SharedRwLockReadGuard<'a> {
    shared_lock: &'a SharedRwLock,
}

/// Proof that a shared lock was obtained for writing.
pub struct SharedRwLockWriteGuard<'a> {
    shared_lock: &'a SharedRwLock,
}

impl<'a> Drop for SharedRwLockReadGuard<'a> {
    fn drop(&mut self) {
        // Unsafe: self.lock is private to this module, only ever set after `raw_read()`,
        // and never copied or cloned (see `compile_time_assert` below).
        unsafe {
            self.shared_lock.arc.raw_unlock_read()
        }
    }
}

impl<'a> Drop for SharedRwLockWriteGuard<'a> {
    fn drop(&mut self) {
        // Unsafe: self.lock is private to this module, only ever set after `raw_write()`,
        // and never copied or cloned (see `compile_time_assert` below).
        unsafe {
            self.shared_lock.arc.raw_unlock_write()
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

/// Guards for a document
#[derive(Clone)]
pub struct ReadGuards<'a> {
    /// For author-origin stylesheets
    pub author: &'a SharedRwLockReadGuard<'a>,

    /// For user-agent-origin and user-origin stylesheets
    pub ua_or_user: &'a SharedRwLockReadGuard<'a>,
}

impl<'a> ReadGuards<'a> {
    /// Same guard for all origins
    pub fn same(guard: &'a SharedRwLockReadGuard<'a>) -> Self {
        ReadGuards {
            author: guard,
            ua_or_user: guard,
        }
    }
}
