/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use owning_ref::{OwningRef, StableAddress};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

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

pub struct AtomicRef<'a, T: 'a>(RwLockReadGuard<'a, T>);
unsafe impl<'a, T> StableAddress for AtomicRef<'a, T> {}

impl<'a, T> Deref for AtomicRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

pub struct AtomicRefMut<'a, T: 'a>(RwLockWriteGuard<'a, T>);
unsafe impl<'a, T> StableAddress for AtomicRefMut<'a, T> {}

impl<'a, T> Deref for AtomicRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<'a, T> DerefMut for AtomicRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}

impl<'a, T: 'a + Debug> Debug for AtomicRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0.deref())
    }
}

impl<'a, T: 'a + Debug> Debug for AtomicRefMut<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0.deref())
    }
}

impl<T> AtomicRefCell<T> {
    pub fn new(value: T) -> Self {
        AtomicRefCell(RwLock::new(value))
    }
    pub fn borrow(&self) -> AtomicRef<T> {
        AtomicRef(self.0.try_read().expect("already mutably borrowed"))
    }
    pub fn borrow_mut(&self) -> AtomicRefMut<T> {
        AtomicRefMut(self.0.try_write().expect("already borrowed"))
    }
}

impl<T: Default> Default for AtomicRefCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/*
 * Implement Ref{,Mut}::map()-like semantics for AtomicRef{,Mut}. We can't quite
 * use AtomicRef{,Mut} as the mapped type, but we can use some trait tricks to
 * allow us to pass MappedAtomicRef{,Mut} back into AtomicRef{,Mut}::map().
 *
 * Note: We cannot implement an AtomicRefMut::map() method until we have mutable
 * OwningRef. See https://github.com/Kimundi/owning-ref-rs/pull/16
 */
pub type MappedAtomicRef<'a, T: 'a, U: 'a> = OwningRef<AtomicRef<'a, T>, U>;

pub trait Map<'a, Base, Curr> {
    fn map<New, F>(self, f: F) -> MappedAtomicRef<'a, Base, New>
        where F: FnOnce(&Curr) -> &New;
}

impl<'a, Base> Map<'a, Base, Base> for AtomicRef<'a, Base> {
    fn map<New, F>(self, f: F) -> MappedAtomicRef<'a, Base, New>
        where F: FnOnce(&Base) -> &New
    {
        OwningRef::new(self).map(f)
    }
}

impl<'a, Base, Curr> Map<'a, Base, Curr> for MappedAtomicRef<'a, Base, Curr> {
    fn map<New, F>(self, f: F) -> MappedAtomicRef<'a, Base, New>
        where F: FnOnce(&Curr) -> &New
    {
        self.map(f)
    }
}

impl<'a, Base> AtomicRef<'a, Base> {
    pub fn map<Curr, New, M, F>(orig: M, f: F) -> MappedAtomicRef<'a, Base, New>
        where F: FnOnce(&Curr) -> &New, M: Map<'a, Base, Curr>
    {
        orig.map(f)
    }
}
