/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A shareable mutable container for the DOM.

use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use style::thread_state;

/// A mutable field in the DOM.
///
/// This extends the API of `std::cell::RefCell` to allow unsafe access in
/// certain situations, with dynamic checking in debug builds.
#[derive(Clone, Debug, Default, HeapSizeOf, PartialEq)]
pub struct DomRefCell<T> {
    value: RefCell<T>,
}

// Functionality specific to Servo's `DomRefCell` type
// ===================================================

impl<T> DomRefCell<T> {
    /// Return a reference to the contents.
    ///
    /// For use in the layout thread only.
    #[allow(unsafe_code)]
    pub unsafe fn borrow_for_layout(&self) -> &T {
        debug_assert!(thread_state::get().is_layout());
        &*self.value.as_ptr()
    }

    /// Borrow the contents for the purpose of GC tracing.
    ///
    /// This succeeds even if the object is mutably borrowed,
    /// so you have to be careful in trace code!
    #[allow(unsafe_code)]
    pub unsafe fn borrow_for_gc_trace(&self) -> &T {
        // FIXME: IN_GC isn't reliable enough - doesn't catch minor GCs
        // https://github.com/servo/servo/issues/6389
        // debug_assert!(thread_state::get().contains(SCRIPT | IN_GC));
        &*self.value.as_ptr()
    }

    /// Borrow the contents for the purpose of script deallocation.
    ///
    #[allow(unsafe_code)]
    pub unsafe fn borrow_for_script_deallocation(&self) -> &mut T {
        debug_assert!(thread_state::get().contains(thread_state::SCRIPT));
        &mut *self.value.as_ptr()
    }

    /// Version of the above that we use during restyle while the script thread
    /// is blocked.
    pub fn borrow_mut_for_layout(&self) -> RefMut<T> {
        debug_assert!(thread_state::get().is_layout());
        self.value.borrow_mut()
    }
}

// Functionality duplicated with `std::cell::RefCell`
// ===================================================
impl<T> DomRefCell<T> {
    /// Create a new `DomRefCell` containing `value`.
    pub fn new(value: T) -> DomRefCell<T> {
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
    /// Panics if this is called off the script thread.
    ///
    /// Panics if the value is currently mutably borrowed.
    pub fn borrow(&self) -> Ref<T> {
        self.try_borrow().expect("DomRefCell<T> already mutably borrowed")
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope. The value
    /// cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    ///
    /// Panics if this is called off the script thread.
    ///
    /// Panics if the value is currently borrowed.
    pub fn borrow_mut(&self) -> RefMut<T> {
        self.try_borrow_mut().expect("DomRefCell<T> already borrowed")
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
    pub fn try_borrow(&self) -> Result<Ref<T>, BorrowError> {
        debug_assert!(thread_state::get().is_script());
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
    pub fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
        debug_assert!(thread_state::get().is_script());
        self.value.try_borrow_mut()
    }
}
