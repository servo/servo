/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Deref, DerefMut};

use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use js::context::NoGC;

/// A borrowed mutable reference to the contents of an AtomicRefCell,
/// anchored to the lifetime of a [NoGC] token.
pub(crate) struct AtomicSafeRefMut<'a, T> {
    ref_mut: AtomicRefMut<'a, T>,
    _anchor: &'a NoGC,
}

pub(crate) trait AtomicSafeBorrowMut {
    type Target;

    /// A version of [DomRefCell::safe_borrow_mut] for [AtomicRefCell].
    /// The resulting borrowed mutable reference statically guarantees
    /// that no garbage collection can occur while the borrow is live.
    fn safe_borrow_mut<'a: 'b, 'b>(&'a self, no_gc: &'b NoGC)
    -> AtomicSafeRefMut<'b, Self::Target>;
}

impl<T> AtomicSafeBorrowMut for AtomicRefCell<T> {
    type Target = T;
    fn safe_borrow_mut<'a: 'b, 'b>(
        &'a self,
        no_gc: &'b NoGC,
    ) -> AtomicSafeRefMut<'b, Self::Target> {
        AtomicSafeRefMut {
            ref_mut: self.borrow_mut(),
            _anchor: no_gc,
        }
    }
}

impl<'a, T> Deref for AtomicSafeRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.ref_mut
    }
}

impl<'a, T> DerefMut for AtomicSafeRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ref_mut
    }
}
