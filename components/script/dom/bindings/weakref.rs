/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::UnsafeCell;
use std::mem;
use std::ops::{Deref, DerefMut, Drop};

use js::jsapi::JSTracer;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
pub use script_bindings::weakref::*;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::JSTraceable;

/// A mutable weak reference to a JS-managed DOM object. On tracing,
/// the contained weak reference is dropped if the pointee was already
/// collected.
pub(crate) struct MutableWeakRef<T: WeakReferenceable> {
    cell: UnsafeCell<Option<WeakRef<T>>>,
}

impl<T: WeakReferenceable> MutableWeakRef<T> {
    /// Create a new mutable weak reference.
    pub(crate) fn new(value: Option<&T>) -> MutableWeakRef<T> {
        MutableWeakRef {
            cell: UnsafeCell::new(value.map(WeakRef::new)),
        }
    }

    /// Set the pointee of a mutable weak reference.
    pub(crate) fn set(&self, value: Option<&T>) {
        unsafe {
            *self.cell.get() = value.map(WeakRef::new);
        }
    }

    /// DomRoot a mutable weak reference. Returns `None` if the object
    /// was already collected.
    pub(crate) fn root(&self) -> Option<DomRoot<T>> {
        unsafe { &*self.cell.get() }
            .as_ref()
            .and_then(WeakRef::root)
    }
}

impl<T: WeakReferenceable> MallocSizeOf for MutableWeakRef<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

unsafe impl<T: WeakReferenceable> JSTraceable for MutableWeakRef<T> {
    unsafe fn trace(&self, _: *mut JSTracer) {
        let ptr = self.cell.get();
        let should_drop = match *ptr {
            Some(ref value) => !value.is_alive(),
            None => false,
        };
        if should_drop {
            mem::drop((*ptr).take().unwrap());
        }
    }
}

/// A vector of weak references. On tracing, the vector retains
/// only references which still point to live objects.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
#[derive(MallocSizeOf)]
pub(crate) struct WeakRefVec<T: WeakReferenceable> {
    vec: Vec<WeakRef<T>>,
}

impl<T: WeakReferenceable> WeakRefVec<T> {
    /// Create a new vector of weak references.
    pub(crate) fn new() -> Self {
        WeakRefVec { vec: vec![] }
    }

    /// Calls a function on each reference which still points to a
    /// live object. The order of the references isn't preserved.
    pub(crate) fn update<F: FnMut(WeakRefEntry<T>)>(&mut self, mut f: F) {
        let mut i = 0;
        while i < self.vec.len() {
            if self.vec[i].is_alive() {
                f(WeakRefEntry {
                    vec: self,
                    index: &mut i,
                });
            } else {
                self.vec.swap_remove(i);
            }
        }
    }

    /// Clears the vector of its dead references.
    pub(crate) fn retain_alive(&mut self) {
        self.update(|_| ());
    }
}

impl<T: WeakReferenceable> Deref for WeakRefVec<T> {
    type Target = Vec<WeakRef<T>>;

    fn deref(&self) -> &Vec<WeakRef<T>> {
        &self.vec
    }
}

impl<T: WeakReferenceable> DerefMut for WeakRefVec<T> {
    fn deref_mut(&mut self) -> &mut Vec<WeakRef<T>> {
        &mut self.vec
    }
}

/// An entry of a vector of weak references. Passed to the closure
/// given to `WeakRefVec::update`.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub(crate) struct WeakRefEntry<'a, T: WeakReferenceable> {
    vec: &'a mut WeakRefVec<T>,
    index: &'a mut usize,
}

impl<'a, T: WeakReferenceable + 'a> WeakRefEntry<'a, T> {
    /// Remove the entry from the underlying vector of weak references.
    pub(crate) fn remove(self) -> WeakRef<T> {
        let ref_ = self.vec.swap_remove(*self.index);
        mem::forget(self);
        ref_
    }
}

impl<'a, T: WeakReferenceable + 'a> Deref for WeakRefEntry<'a, T> {
    type Target = WeakRef<T>;

    fn deref(&self) -> &WeakRef<T> {
        &self.vec[*self.index]
    }
}

impl<'a, T: WeakReferenceable + 'a> Drop for WeakRefEntry<'a, T> {
    fn drop(&mut self) {
        *self.index += 1;
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct DOMTracker<T: WeakReferenceable> {
    dom_objects: DomRefCell<WeakRefVec<T>>,
}

impl<T: WeakReferenceable> DOMTracker<T> {
    pub(crate) fn new() -> Self {
        Self {
            dom_objects: DomRefCell::new(WeakRefVec::new()),
        }
    }

    pub(crate) fn track(&self, dom_object: &T) {
        self.dom_objects.borrow_mut().push(WeakRef::new(dom_object));
    }

    pub(crate) fn for_each<F: FnMut(DomRoot<T>)>(&self, mut f: F) {
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
