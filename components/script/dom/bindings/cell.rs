/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A shareable mutable container for the DOM.

use std::cell::{BorrowError, BorrowMutError};
#[cfg(not(feature = "refcell_backtrace"))]
pub(crate) use std::cell::{Ref, RefCell, RefMut};

#[cfg(feature = "refcell_backtrace")]
pub(crate) use accountable_refcell::{ref_filter_map, Ref, RefCell, RefMut};
#[cfg(not(feature = "refcell_backtrace"))]
pub(crate) use ref_filter_map::ref_filter_map;

use crate::dom::bindings::root::{assert_in_layout, assert_in_script};

/// A mutable field in the DOM.
///
/// This extends the API of `std::cell::RefCell` to allow unsafe access in
/// certain situations, with dynamic checking in debug builds.
#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq)]
pub(crate) struct DomRefCell<T> {
    value: RefCell<T>,
}

// Functionality specific to Servo's `DomRefCell` type
// ===================================================

impl<T> DomRefCell<T> {
    /// Return a reference to the contents.  For use in layout only.
    ///
    /// # Safety
    ///
    /// Unlike RefCell::borrow, this method is unsafe because it does not return a Ref, thus leaving
    /// the borrow flag untouched. Mutably borrowing the RefCell while the reference returned by
    /// this method is alive is undefined behaviour.
    ///
    /// # Panics
    ///
    /// Panics if this is called from anywhere other than the layout thread
    ///
    /// Panics if the value is currently mutably borrowed.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn borrow_for_layout(&self) -> &T {
        assert_in_layout();
        self.value
            .try_borrow_unguarded()
            .expect("cell is mutably borrowed")
    }

    /// Borrow the contents for the purpose of script deallocation.
    ///
    /// # Safety
    ///
    /// Unlike RefCell::borrow, this method is unsafe because it does not return a Ref, thus leaving
    /// the borrow flag untouched. Mutably borrowing the RefCell while the reference returned by
    /// this method is alive is undefined behaviour.
    ///
    /// # Panics
    ///
    /// Panics if this is called from anywhere other than the script thread.
    #[allow(unsafe_code, clippy::mut_from_ref)]
    pub(crate) unsafe fn borrow_for_script_deallocation(&self) -> &mut T {
        assert_in_script();
        &mut *self.value.as_ptr()
    }

    /// Mutably borrow a cell for layout. Ideally this would use
    /// `RefCell::try_borrow_mut_unguarded` but that doesn't exist yet.
    ///
    /// # Safety
    ///
    /// Unlike RefCell::borrow, this method is unsafe because it does not return a Ref, thus leaving
    /// the borrow flag untouched. Mutably borrowing the RefCell while the reference returned by
    /// this method is alive is undefined behaviour.
    ///
    /// # Panics
    ///
    /// Panics if this is called from anywhere other than the layout thread.
    #[allow(unsafe_code, clippy::mut_from_ref)]
    pub(crate) unsafe fn borrow_mut_for_layout(&self) -> &mut T {
        assert_in_layout();
        &mut *self.value.as_ptr()
    }
}

// Functionality duplicated with `std::cell::RefCell`
// ===================================================
impl<T> DomRefCell<T> {
    /// Create a new `DomRefCell` containing `value`.
    pub(crate) fn new(value: T) -> DomRefCell<T> {
        DomRefCell {
            value: RefCell::new(value),
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    #[track_caller]
    pub(crate) fn borrow(&self) -> Ref<T> {
        self.value.borrow()
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope. The value
    /// cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    #[track_caller]
    pub(crate) fn borrow_mut(&self) -> RefMut<T> {
        self.value.borrow_mut()
    }

    /// Attempts to immutably borrow the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// Returns `None` if the value is currently mutably borrowed.
    ///
    /// # Panics
    ///
    /// Panics if this is called off the script thread.
    pub(crate) fn try_borrow(&self) -> Result<Ref<T>, BorrowError> {
        assert_in_script();
        self.value.try_borrow()
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope. The value
    /// cannot be borrowed while this borrow is active.
    ///
    /// Returns `None` if the value is currently borrowed.
    ///
    /// # Panics
    ///
    /// Panics if this is called off the script thread.
    pub(crate) fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
        assert_in_script();
        self.value.try_borrow_mut()
    }
}

impl<T: Default> DomRefCell<T> {
    /// Takes the wrapped value, leaving `Default::default()` in its place.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub(crate) fn take(&self) -> T {
        self.value.take()
    }
}
