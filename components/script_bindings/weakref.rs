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

use std::cell::Cell;
use std::ops::Drop;
use std::{mem, ptr};

use js::glue::JS_GetReservedSlot;
use js::jsapi::{JSTracer, JS_SetReservedSlot};
use js::jsval::{PrivateValue, UndefinedValue};
use libc::c_void;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::reflector::DomObject;
use crate::root::DomRoot;
use crate::JSTraceable;

/// The index of the slot wherein a pointer to the weak holder cell is
/// stored for weak-referenceable bindings. We use slot 1 for holding it,
/// this is unsafe for globals, we disallow weak-referenceable globals
/// directly in codegen.
pub const DOM_WEAK_SLOT: u32 = 1;

/// A weak reference to a JS-managed DOM object.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub struct WeakRef<T: WeakReferenceable> {
    ptr: ptr::NonNull<WeakBox<T>>,
}

/// The inner box of weak references, public for the finalization in codegen.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct WeakBox<T: WeakReferenceable> {
    /// The reference count. When it reaches zero, the `value` field should
    /// have already been set to `None`. The pointee contributes one to the count.
    pub count: Cell<usize>,
    /// The pointer to the JS-managed object, set to None when it is collected.
    pub value: Cell<Option<ptr::NonNull<T>>>,
}

/// Trait implemented by weak-referenceable interfaces.
pub trait WeakReferenceable: DomObject + Sized {
    /// Downgrade a DOM object reference to a weak one.
    fn downgrade(&self) -> WeakRef<Self> {
        unsafe {
            let object = self.reflector().get_jsobject().get();
            let mut slot = UndefinedValue();
            JS_GetReservedSlot(object, DOM_WEAK_SLOT, &mut slot);
            let mut ptr = slot.to_private() as *mut WeakBox<Self>;
            if ptr.is_null() {
                trace!("Creating new WeakBox holder for {:p}.", self);
                ptr = Box::into_raw(Box::new(WeakBox {
                    count: Cell::new(1),
                    value: Cell::new(Some(ptr::NonNull::from(self))),
                }));
                let val = PrivateValue(ptr as *const c_void);
                JS_SetReservedSlot(object, DOM_WEAK_SLOT, &val);
            }
            let box_ = &*ptr;
            assert!(box_.value.get().is_some());
            let new_count = box_.count.get() + 1;
            trace!(
                "Incrementing WeakBox refcount for {:p} to {}.",
                self,
                new_count
            );
            box_.count.set(new_count);
            WeakRef {
                ptr: ptr::NonNull::new_unchecked(ptr),
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
        unsafe { &*self.ptr.as_ptr() }
            .value
            .get()
            .map(|ptr| unsafe { DomRoot::from_ref(&*ptr.as_ptr()) })
    }

    /// Return whether the weakly-referenced object is still alive.
    pub fn is_alive(&self) -> bool {
        unsafe { &*self.ptr.as_ptr() }.value.get().is_some()
    }
}

impl<T: WeakReferenceable> Clone for WeakRef<T> {
    fn clone(&self) -> WeakRef<T> {
        unsafe {
            let box_ = &*self.ptr.as_ptr();
            let new_count = box_.count.get() + 1;
            box_.count.set(new_count);
            WeakRef { ptr: self.ptr }
        }
    }
}

impl<T: WeakReferenceable> MallocSizeOf for WeakRef<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T: WeakReferenceable> PartialEq for WeakRef<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            (*self.ptr.as_ptr()).value.get().map(ptr::NonNull::as_ptr) ==
                (*other.ptr.as_ptr()).value.get().map(ptr::NonNull::as_ptr)
        }
    }
}

impl<T: WeakReferenceable> PartialEq<T> for WeakRef<T> {
    fn eq(&self, other: &T) -> bool {
        unsafe {
            match self.ptr.as_ref().value.get() {
                Some(ptr) => ptr::eq(ptr.as_ptr(), other),
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
                let weak_box = &*self.ptr.as_ptr();
                assert!(weak_box.count.get() > 0);
                let count = weak_box.count.get() - 1;
                weak_box.count.set(count);
                (count, weak_box.value.get())
            };
            if count == 0 {
                assert!(value.is_none());
                mem::drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}
