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

use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use std::{mem, ptr};

use js::jsapi::JSTracer;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::JSTraceable;
use crate::reflector::DomObject;
use crate::root::DomRoot;

/// A weak reference to a JS-managed DOM object.
#[derive(Clone)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub struct WeakRef<T: WeakReferenceable>(Weak<T>);

/// Trait implemented by weak-referenceable interfaces.
pub trait WeakReferenceable: DomObject + Sized {
    /// Downgrade a DOM object reference to a weak one.
    fn downgrade(&self) -> WeakRef<Self> {
        let rc = unsafe { Rc::from_raw(self as *const Self) };
        let weak = WeakRef(Rc::downgrade(&rc));
        mem::forget(rc);
        weak
    }
}

impl<T: WeakReferenceable> Eq for WeakRef<T> {}

impl<T: WeakReferenceable> Hash for WeakRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl<T: WeakReferenceable> WeakRef<T> {
    /// Create a new weak reference from a `WeakReferenceable` interface instance.
    /// This is just a convenience wrapper around `<T as WeakReferenceable>::downgrade`
    /// to not have to import `WeakReferenceable`.
    pub fn new(value: &T) -> Self {
        value.downgrade()
    }

    /// DomRoot a weak reference. Returns `None` if the object was already collected.
    pub fn root(&self) -> Option<DomRoot<T>> {
        self.0.upgrade().map(|x| DomRoot::from_ref(&*x))
    }

    /// Return whether the weakly-referenced object is still alive.
    pub fn is_alive(&self) -> bool {
        self.0.strong_count() > 0
    }
}

impl<T: WeakReferenceable> MallocSizeOf for WeakRef<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T: WeakReferenceable> PartialEq for WeakRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl<T: WeakReferenceable> PartialEq<T> for WeakRef<T> {
    fn eq(&self, other: &T) -> bool {
        match self.0.upgrade() {
            Some(ptr) => ptr::eq(Rc::as_ptr(&ptr), other),
            None => false,
        }
    }
}

unsafe impl<T: WeakReferenceable> JSTraceable for WeakRef<T> {
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing.
    }
}
