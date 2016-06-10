/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use dom::bindings::global::{GlobalRef, GlobalRoot, global_root_from_reflector};
use dom::bindings::js::Root;
use js::jsapi::{HandleObject, JSContext, JSObject};
use std::cell::UnsafeCell;
use std::ptr;

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T: Reflectable>(obj: Box<T>,
                                          global: GlobalRef,
                                          wrap_fn: unsafe fn(*mut JSContext, GlobalRef, Box<T>) -> Root<T>)
                                          -> Root<T> {
    unsafe {
        wrap_fn(global.get_cx(), global, obj)
    }
}

/// A struct to store a reference to the reflector of a DOM object.
#[allow(unrooted_must_root)]
#[must_root]
#[servo_lang = "reflector"]
#[derive(HeapSizeOf)]
// If you're renaming or moving this field, update the path in plugins::reflector as well
pub struct Reflector {
    #[ignore_heap_size_of = "defined and measured in rust-mozjs"]
    object: UnsafeCell<*mut JSObject>,
}

#[allow(unrooted_must_root)]
impl PartialEq for Reflector {
    fn eq(&self, other: &Reflector) -> bool {
        unsafe { *self.object.get() == *other.object.get() }
    }
}

impl Reflector {
    /// Get the reflector.
    #[inline]
    pub fn get_jsobject(&self) -> HandleObject {
        unsafe { HandleObject::from_marked_location(self.object.get()) }
    }

    /// Initialize the reflector. (May be called only once.)
    pub fn set_jsobject(&mut self, object: *mut JSObject) {
        unsafe {
            let obj = self.object.get();
            assert!((*obj).is_null());
            assert!(!object.is_null());
            *obj = object;
        }
    }

    /// Return a pointer to the memory location at which the JS reflector
    /// object is stored. Used to root the reflector, as
    /// required by the JSAPI rooting APIs.
    pub fn rootable(&self) -> *mut *mut JSObject {
        self.object.get()
    }

    /// Create an uninitialized `Reflector`.
    pub fn new() -> Reflector {
        Reflector {
            object: UnsafeCell::new(ptr::null_mut()),
        }
    }
}

/// A trait to provide access to the `Reflector` for a DOM object.
pub trait Reflectable {
    /// Returns the receiver's reflector.
    fn reflector(&self) -> &Reflector;
    /// Initializes the Reflector
    fn init_reflector(&mut self, obj: *mut JSObject);

    /// Returns the global object of the realm that the Reflectable was created in.
    fn global(&self) -> GlobalRoot where Self: Sized {
        global_root_from_reflector(self)
    }
}
