/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Weak-referenceable JS-managed DOM objects.

use core::nonzero::NonZero;
use dom::bindings::js::Root;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::Reflectable;
use js::jsapi::{JSTracer, JS_GetReservedSlot, JS_SetReservedSlot};
use js::jsval::PrivateValue;
use libc::c_void;
use std::cell::{Cell, UnsafeCell};
use std::mem;
use util::mem::HeapSizeOf;

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
    /// have already been set to `None`.
    pub count: Cell<usize>,
    /// The pointer to the JS-managed object, set to None when it is collected.
    pub value: Cell<Option<NonZero<*const T>>>,
}

/// Trait implemented by weak-referenceable interfaces.
pub trait WeakReferenceable: Reflectable + Sized {
    /// Downgrade a DOM object reference to a weak one.
    fn downgrade(&self) -> WeakRef<Self> {
        unsafe {
            let object = self.reflector().get_jsobject().get();
            let mut ptr =
                JS_GetReservedSlot(object, DOM_WEAK_SLOT).to_private() as *mut WeakBox<Self>;
            if ptr.is_null() {
                debug!("Creating new WeakBox holder for {:p}.", self);
                ptr = Box::into_raw(box WeakBox {
                    count: Cell::new(1),
                    value: Cell::new(Some(NonZero::new(self))),
                });
                JS_SetReservedSlot(object, DOM_WEAK_SLOT, PrivateValue(ptr as *const c_void));
            }
            let box_ = &*ptr;
            assert!(box_.value.get().is_some());
            let new_count = box_.count.get() + 1;
            debug!("Incrementing WeakBox refcount for {:p} to {}.", self, new_count);
            box_.count.set(new_count);
            WeakRef { ptr: NonZero::new(ptr) }
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

    /// Root a weak reference. Returns `None` if the object was already collected.
    pub fn root(&self) -> Option<Root<T>> {
        unsafe { &**self.ptr }.value.get().map(Root::new)
    }

    /// Return whether the weakly-referenced object is still alive.
    pub fn is_alive(&self) -> bool {
        unsafe { &**self.ptr }.value.get().is_some()
    }
}

impl<T: WeakReferenceable> Clone for WeakRef<T> {
    fn clone(&self) -> WeakRef<T> {
        unsafe {
            let box_ = &**self.ptr;
            let new_count = box_.count.get() + 1;
            box_.count.set(new_count);
            WeakRef { ptr: self.ptr }
        }
    }
}

impl<T: WeakReferenceable> HeapSizeOf for WeakRef<T> {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

no_jsmanaged_fields!(WeakRef<T: WeakReferenceable>);

impl<T: WeakReferenceable> Drop for WeakRef<T> {
    fn drop(&mut self) {
        unsafe {
            let (count, value) = {
                let box_ = &**self.ptr;
                let count = box_.count.get() - 1;
                box_.count.set(count);
                (count, box_.value.get())
            };
            if count == 0 {
                assert!(value.is_none());
                mem::drop(Box::from_raw(*self.ptr));
            }
        }
    }
}

/// A transient weak reference to a JS-managed DOM object. On tracing,
/// the contained weak reference is dropped if found to point to an object
/// that was already collected.
pub struct TransientWeakRef<T: WeakReferenceable> {
    cell: UnsafeCell<Option<WeakRef<T>>>,
}

impl<T: WeakReferenceable> TransientWeakRef<T> {
    /// Create a new transient weak reference.
    pub fn new(value: Option<&T>) -> TransientWeakRef<T> {
        TransientWeakRef {
            cell: UnsafeCell::new(value.map(WeakRef::new)),
        }
    }

    /// Root a transient weak reference. Returns `None` if the object
    /// was already collected.
    pub fn root(&self) -> Option<Root<T>> {
        unsafe { &*self.cell.get() }.as_ref().and_then(WeakRef::root)
    }
}

impl<T: WeakReferenceable> HeapSizeOf for TransientWeakRef<T> {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl<T: WeakReferenceable> JSTraceable for TransientWeakRef<T> {
    fn trace(&self, _: *mut JSTracer) {
        let ptr = self.cell.get();
        unsafe {
            let should_drop = match *ptr {
                Some(ref value) => !value.is_alive(),
                None => false,
            };
            if should_drop {
                mem::drop((*ptr).take().unwrap());
            }
        }
    }
}
