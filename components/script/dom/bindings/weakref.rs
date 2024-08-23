/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Weak-referenceable JS-managed DOM objects.
//!
//! IDL interfaces marked as `weakReferenceable` in `Bindings.conf`
//! automatically implement the `WeakReferenceable` trait in codegen.
//! The instance object is responsible for setting `None` in its own
//! own `WeakBox` when it is collected, through the `DOM_WEAK_SLOT`
//! slot. When all associated `WeakRef` values are dropped, the
//! `WeakBox` itself is dropped too.

use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut, Drop};
use std::{mem, ptr};

use js::glue::JS_GetReservedSlot;
use js::jsapi::{JSTracer, JS_SetReservedSlot};
use js::jsval::{PrivateValue, UndefinedValue};
use libc::c_void;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
pub use script_bindings::weakref::*;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::JSTraceable;

#[derive(MallocSizeOf)]
pub struct DOMTracker<T: WeakReferenceable> {
    dom_objects: DomRefCell<WeakRefVec<T>>,
}

impl<T: WeakReferenceable> DOMTracker<T> {
    pub fn new() -> Self {
        Self {
            dom_objects: DomRefCell::new(WeakRefVec::new()),
        }
    }

    pub fn track(&self, dom_object: &T) {
        self.dom_objects.borrow_mut().push(WeakRef::new(dom_object));
    }

    pub fn for_each<F: FnMut(DomRoot<T>)>(&self, mut f: F) {
        self.dom_objects.borrow_mut().update(|weak_ref| {
            let root = weak_ref.root().unwrap();
            f(root);
        });
    }
}

#[allow(unsafe_code)]
unsafe impl<T: WeakReferenceable> JSTraceable for DOMTracker<T> {
    unsafe fn trace(&self, _: *mut JSTracer) {
        self.dom_objects.borrow_mut().retain_alive();
    }
}
