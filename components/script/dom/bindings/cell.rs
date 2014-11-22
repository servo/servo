/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::trace::JSTraceable;
use js::jsapi::{JSTracer};

use servo_util::task_state;
use servo_util::task_state::{SCRIPT, IN_GC};

use std::cell::{RefCell, Ref, RefMut};

/// A mutable field in the DOM.
///
/// This extends the API of `core::cell::RefCell` to allow unsafe access in
/// certain situations, with dynamic checking in debug builds.
pub struct DOMRefCell<T> {
    value: RefCell<T>,
}

// Functionality specific to Servo's `DOMRefCell` type
// ===================================================

impl<T> DOMRefCell<T> {
    /// Return a reference to the contents.
    ///
    /// For use in the layout task only.
    pub unsafe fn borrow_for_layout<'a>(&'a self) -> &'a T {
        debug_assert!(task_state::get().is_layout());
        &*self.value.as_unsafe_cell().get()
    }

    /// Borrow the contents for the purpose of GC tracing.
    ///
    /// This succeeds even if the object is mutably borrowed,
    /// so you have to be careful in trace code!
    pub unsafe fn borrow_for_gc_trace<'a>(&'a self) -> &'a T {
        debug_assert!(task_state::get().contains(SCRIPT | IN_GC));
        &*self.value.as_unsafe_cell().get()
    }

    /// Is the cell mutably borrowed?
    ///
    /// For safety checks in debug builds only.
    pub fn is_mutably_borrowed(&self) -> bool {
        self.value.try_borrow().is_some()
    }

    pub fn try_borrow<'a>(&'a self) -> Option<Ref<'a, T>> {
        debug_assert!(task_state::get().is_script());
        self.value.try_borrow()
    }

    pub fn try_borrow_mut<'a>(&'a self) -> Option<RefMut<'a, T>> {
        debug_assert!(task_state::get().is_script());
        self.value.try_borrow_mut()
    }
}

impl<T: JSTraceable> JSTraceable for DOMRefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (*self).borrow().trace(trc)
    }
}

// Functionality duplicated with `core::cell::RefCell`
// ===================================================
impl<T> DOMRefCell<T> {
    pub fn new(value: T) -> DOMRefCell<T> {
        DOMRefCell {
            value: RefCell::new(value),
        }
    }

    pub fn unwrap(self) -> T {
        self.value.unwrap()
    }

    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        match self.try_borrow() {
            Some(ptr) => ptr,
            None => panic!("DOMRefCell<T> already mutably borrowed")
        }
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        match self.try_borrow_mut() {
            Some(ptr) => ptr,
            None => panic!("DOMRefCell<T> already borrowed")
        }
    }
}
