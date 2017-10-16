/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Weak-referenceable JS-managed DOM objects.
//!
//! IDL interfaces marked as `weakReferenceable` in `Bindings.conf`
//! automatically implement the `WeakReferenceable` trait in codegen.
//! The instance object is responsible for setting `None` in its own
//! own `WeakBox` when it is collected, through the `DOM_WEAK_SLOT`
//! slot. When all associated `WeakRef` values are dropped, the
//! `WeakBox` itself is dropped too.

use core::nonzero::NonZero;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::DomRoot;
use dom::bindings::trace::JSTraceable;
use heapsize::HeapSizeOf;
use js::jsapi::{JSTracer, JS_GetReservedSlot, JS_SetReservedSlot};
use js::jsval::PrivateValue;
use libc::c_void;
use std::cell::{Cell, UnsafeCell};
use std::mem;
use std::ops::{Deref, DerefMut, Drop};

/// The index of the slot wherein a pointer to the weak holder cell is
/// stored for weak-referenceable bindings. We use slot 1 for holding it,
/// this is unsafe for globals, we disallow weak-referenceable globals
/// directly in codegen.
pub const DOM_WEAK_SLOT: u32 = 1;

/// A weak reference to a JS-managed DOM object.
#[allow_unrooted_interior]
pub struct WeakRef<T: WeakReferenceable> {
    ptr: NonZero<*mut WeakBox<T>>,
}

/// The inner box of weak references, public for the finalization in codegen.
#[must_root]
pub struct WeakBox<T: WeakReferenceable> {
    /// The reference count. When it reaches zero, the `value` field should
    /// have already been set to `None`. The pointee contributes one to the count.
    pub count: Cell<usize>,
    /// The pointer to the JS-managed object, set to None when it is collected.
    pub value: Cell<Option<NonZero<*const T>>>,
}

/// Trait implemented by weak-referenceable interfaces.
pub trait WeakReferenceable: DomObject + Sized {
    /// Downgrade a DOM object reference to a weak one.
    fn downgrade(&self) -> WeakRef<Self> {
        unsafe {
            let object = self.reflector().get_jsobject().get();
            let mut ptr = JS_GetReservedSlot(object,
                                             DOM_WEAK_SLOT)
                              .to_private() as *mut WeakBox<Self>;
            if ptr.is_null() {
                trace!("Creating new WeakBox holder for {:p}.", self);
                ptr = Box::into_raw(Box::new(WeakBox {
                    count: Cell::new(1),
                    value: Cell::new(Some(NonZero::new_unchecked(self))),
                }));
                JS_SetReservedSlot(object, DOM_WEAK_SLOT, PrivateValue(ptr as *const c_void));
            }
            let box_ = &*ptr;
            assert!(box_.value.get().is_some());
            let new_count = box_.count.get() + 1;
            trace!("Incrementing WeakBox refcount for {:p} to {}.",
                   self,
                   new_count);
            box_.count.set(new_count);
            WeakRef {
                ptr: NonZero::new_unchecked(ptr),
            }
        }
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
        unsafe { &*self.ptr.get() }.value.get().map(|ptr| unsafe {
            DomRoot::from_ref(&*ptr.get())
        })
    }

    /// Return whether the weakly-referenced object is still alive.
    pub fn is_alive(&self) -> bool {
        unsafe { &*self.ptr.get() }.value.get().is_some()
    }
}

impl<T: WeakReferenceable> Clone for WeakRef<T> {
    fn clone(&self) -> WeakRef<T> {
        unsafe {
            let box_ = &*self.ptr.get();
            let new_count = box_.count.get() + 1;
            box_.count.set(new_count);
            WeakRef {
                ptr: self.ptr,
            }
        }
    }
}

impl<T: WeakReferenceable> HeapSizeOf for WeakRef<T> {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl<T: WeakReferenceable> PartialEq for WeakRef<T> {
   fn eq(&self, other: &Self) -> bool {
        unsafe {
            (*self.ptr.get()).value.get() == (*other.ptr.get()).value.get()
        }
    }
}

impl<T: WeakReferenceable> PartialEq<T> for WeakRef<T> {
    fn eq(&self, other: &T) -> bool {
        unsafe {
            match (*self.ptr.get()).value.get() {
                Some(ptr) => ptr.get() == other,
                None => false,
            }
        }
    }
}

unsafe impl<T: WeakReferenceable> JSTraceable for WeakRef<T> {
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing.
    }
}

impl<T: WeakReferenceable> Drop for WeakRef<T> {
    fn drop(&mut self) {
        unsafe {
            let (count, value) = {
                let weak_box = &*self.ptr.get();
                assert!(weak_box.count.get() > 0);
                let count = weak_box.count.get() - 1;
                weak_box.count.set(count);
                (count, weak_box.value.get())
            };
            if count == 0 {
                assert!(value.is_none());
                mem::drop(Box::from_raw(self.ptr.get()));
            }
        }
    }
}

/// A mutable weak reference to a JS-managed DOM object. On tracing,
/// the contained weak reference is dropped if the pointee was already
/// collected.
pub struct MutableWeakRef<T: WeakReferenceable> {
    cell: UnsafeCell<Option<WeakRef<T>>>,
}

impl<T: WeakReferenceable> MutableWeakRef<T> {
    /// Create a new mutable weak reference.
    pub fn new(value: Option<&T>) -> MutableWeakRef<T> {
        MutableWeakRef {
            cell: UnsafeCell::new(value.map(WeakRef::new)),
        }
    }

    /// Set the pointee of a mutable weak reference.
    pub fn set(&self, value: Option<&T>) {
        unsafe {
            *self.cell.get() = value.map(WeakRef::new);
        }
    }

    /// DomRoot a mutable weak reference. Returns `None` if the object
    /// was already collected.
    pub fn root(&self) -> Option<DomRoot<T>> {
        unsafe { &*self.cell.get() }.as_ref().and_then(WeakRef::root)
    }
}

impl<T: WeakReferenceable> HeapSizeOf for MutableWeakRef<T> {
    fn heap_size_of_children(&self) -> usize {
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
#[allow_unrooted_interior]
#[derive(HeapSizeOf)]
pub struct WeakRefVec<T: WeakReferenceable> {
    vec: Vec<WeakRef<T>>,
}

impl<T: WeakReferenceable> WeakRefVec<T> {
    /// Create a new vector of weak references.
    pub fn new() -> Self {
        WeakRefVec { vec: vec![] }
    }

    /// Calls a function on each reference which still points to a
    /// live object. The order of the references isn't preserved.
    pub fn update<F: FnMut(WeakRefEntry<T>)>(&mut self, mut f: F) {
        let mut i = 0;
        while i < self.vec.len() {
            if self.vec[i].is_alive() {
                f(WeakRefEntry { vec: self, index: &mut i });
            } else {
                self.vec.swap_remove(i);
            }
        }
    }

    /// Clears the vector of its dead references.
    pub fn retain_alive(&mut self) {
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
#[allow_unrooted_interior]
pub struct WeakRefEntry<'a, T: WeakReferenceable + 'a> {
    vec: &'a mut WeakRefVec<T>,
    index: &'a mut usize,
}

impl<'a, T: WeakReferenceable + 'a> WeakRefEntry<'a, T> {
    /// Remove the entry from the underlying vector of weak references.
    pub fn remove(self) -> WeakRef<T> {
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
