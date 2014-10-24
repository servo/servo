/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::trace::JSTraceable;
use js::jsapi::{JSTracer};

use servo_util::task_state;
use servo_util::task_state::{Script, InGC};

use std::cell::{Cell, UnsafeCell};
use std::kinds::marker;

/// A mutable field in the DOM.
///
/// This extends the API of `core::cell::RefCell` to allow unsafe access in
/// certain situations, with dynamic checking in debug builds.
pub struct DOMRefCell<T> {
    value: UnsafeCell<T>,
    borrow: Cell<BorrowFlag>,
    nocopy: marker::NoCopy,
    nosync: marker::NoSync,
}

// Functionality specific to Servo's `DOMRefCell` type
// ===================================================

impl<T> DOMRefCell<T> {
    /// Return a reference to the contents.
    ///
    /// For use in the layout task only.
    pub unsafe fn borrow_for_layout<'a>(&'a self) -> &'a T {
        debug_assert!(task_state::get().is_layout());
        &*self.value.get()
    }

    /// Borrow the contents for the purpose of GC tracing.
    ///
    /// This succeeds even if the object is mutably borrowed,
    /// so you have to be careful in trace code!
    pub unsafe fn borrow_for_gc_trace<'a>(&'a self) -> &'a T {
        debug_assert!(task_state::get().contains(Script | InGC));
        &*self.value.get()
    }

    /// Is the cell mutably borrowed?
    ///
    /// For safety checks in debug builds only.
    pub fn is_mutably_borrowed(&self) -> bool {
        self.borrow.get() == WRITING
    }

    pub fn try_borrow<'a>(&'a self) -> Option<Ref<'a, T>> {
        debug_assert!(task_state::get().is_script());
        match self.borrow.get() {
            WRITING => None,
            borrow => {
                self.borrow.set(borrow + 1);
                Some(Ref { _parent: self })
            }
        }
    }

    pub fn try_borrow_mut<'a>(&'a self) -> Option<RefMut<'a, T>> {
        debug_assert!(task_state::get().is_script());
        match self.borrow.get() {
            UNUSED => {
                self.borrow.set(WRITING);
                Some(RefMut { _parent: self })
            },
            _ => None
        }
    }
}

impl<T: JSTraceable> JSTraceable for DOMRefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (*self).borrow().trace(trc)
    }
}

// Functionality duplicated with `core::cell::RefCell`
// ===================================================
//
// This can shrink once rust-lang/rust#18131 is fixed.

// Values [1, MAX-1] represent the number of `Ref` active
// (will not outgrow its range since `uint` is the size of the address space)
type BorrowFlag = uint;
static UNUSED: BorrowFlag = 0;
static WRITING: BorrowFlag = -1;

impl<T> DOMRefCell<T> {
    pub fn new(value: T) -> DOMRefCell<T> {
        DOMRefCell {
            value: UnsafeCell::new(value),
            borrow: Cell::new(UNUSED),
            nocopy: marker::NoCopy,
            nosync: marker::NoSync,
        }
    }

    pub fn unwrap(self) -> T {
        debug_assert!(self.borrow.get() == UNUSED);
        unsafe{self.value.unwrap()}
    }

    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        match self.try_borrow() {
            Some(ptr) => ptr,
            None => fail!("DOMRefCell<T> already mutably borrowed")
        }
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        match self.try_borrow_mut() {
            Some(ptr) => ptr,
            None => fail!("DOMRefCell<T> already borrowed")
        }
    }
}

pub struct Ref<'b, T:'b> {
    _parent: &'b DOMRefCell<T>
}

#[unsafe_destructor]
impl<'b, T> Drop for Ref<'b, T> {
    fn drop(&mut self) {
        let borrow = self._parent.borrow.get();
        debug_assert!(borrow != WRITING && borrow != UNUSED);
        self._parent.borrow.set(borrow - 1);
    }
}

impl<'b, T> Deref<T> for Ref<'b, T> {
    #[inline]
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self._parent.value.get() }
    }
}

pub struct RefMut<'b, T:'b> {
    _parent: &'b DOMRefCell<T>
}

#[unsafe_destructor]
impl<'b, T> Drop for RefMut<'b, T> {
    fn drop(&mut self) {
        let borrow = self._parent.borrow.get();
        debug_assert!(borrow == WRITING);
        self._parent.borrow.set(UNUSED);
    }
}

impl<'b, T> Deref<T> for RefMut<'b, T> {
    #[inline]
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self._parent.value.get() }
    }
}

impl<'b, T> DerefMut<T> for RefMut<'b, T> {
    #[inline]
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self._parent.value.get() }
    }
}
