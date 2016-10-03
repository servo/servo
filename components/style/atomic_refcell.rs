/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Container type providing RefCell-like semantics for objects shared across
/// threads.
///
/// RwLock is traditionally considered to be the |Sync| analogue of RefCell.
/// However, for consumers that can guarantee that they will never mutably
/// borrow the contents concurrently with immutable borrows, an RwLock feels
/// like overkill.
///
/// The RwLock in the standard library is indeed heavyweight, since it heap-
/// allocates an OS-specific primitive and delegates operations to it. However,
/// parking_lot provides a pure-rust implementation of the standard
/// synchronization primitives, with uncontended borrows compiling down to a
/// single atomic operation. This is exactly how we would build an atomic
/// RefCell, so we newtype it with some API sugar.
pub struct AtomicRefCell<T>(RwLock<T>);

pub type AtomicRef<'a, T> = RwLockReadGuard<'a, T>;
pub type AtomicRefMut<'a, T> = RwLockWriteGuard<'a, T>;
impl<T> AtomicRefCell<T> {
    pub fn new(value: T) -> Self {
        AtomicRefCell(RwLock::new(value))
    }
    pub fn borrow(&self) -> AtomicRef<T> {
        self.0.try_read().unwrap()
    }
    pub fn borrow_mut(&self) -> AtomicRefMut<T> {
        self.0.try_write().unwrap()
    }
}

impl<T: Default> Default for AtomicRefCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
