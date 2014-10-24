/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::trace::JSTraceable;
use js::jsapi::{JSTracer};

#[cfg(debug)]
use layout_interface::LayoutBorrowMarker;

use std::cell;
use std::cell::RefCell;
use std::mem;

/// A mutable field in DOM for large sized value.
/// This has a special method to return the pointer of itself
/// for used in layout task.
/// This simply wraps `RefCell<T>` to add the special method.
pub struct DOMRefCell<T> {
    base: RefCell<T>,
}

pub type Ref<'a, T> = cell::Ref<'a, T>;
pub type RefMut<'a, T> = cell::RefMut<'a, T>;


impl<T> DOMRefCell<T> {
    #[inline(always)]
    pub fn new(value: T) -> DOMRefCell<T> {
        DOMRefCell {
            base: RefCell::new(value),
        }
    }

    #[inline(always)]
    pub fn unwrap(self) -> T {
        self.base.unwrap()
    }

    #[inline(always)]
    pub fn try_borrow<'a>(&'a self) -> Option<Ref<'a, T>> {
        check_not_called_from_layout();
        self.base.try_borrow()
    }

    #[inline(always)]
    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        self.base.borrow()
    }

    #[inline(always)]
    pub fn try_borrow_mut<'a>(&'a self) -> Option<RefMut<'a, T>> {
        check_not_called_from_layout();
        self.base.try_borrow_mut()
    }

    #[inline(always)]
    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        self.base.borrow_mut()
    }

    /// This returns the pointer which refers T in `RefCell<T>` directly.
    pub unsafe fn borrow_for_layout<'a>(&'a self) -> &'a T {
        check_called_in_layout();

        let val = mem::transmute::<&RefCell<T>, &T>(&self.base);
        val
    }
}

impl<T: JSTraceable> JSTraceable for DOMRefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (*self).base.borrow().trace(trc)
    }
}

#[cfg(debug)]
fn check_called_in_layout() {
    if LayoutBorrowMarker.get().is_none() {
        fail!("you must not call this method in non layout task.");
    }
}

#[cfg(not(debug))]
fn check_called_in_layout() {
}

#[cfg(debug)]
fn check_not_called_from_layout() {
    if LayoutBorrowMarker.get().is_some() {
        fail!("you must not call this method in layout task.");
    }
}

#[cfg(not(debug))]
fn check_not_called_from_layout() {
}
