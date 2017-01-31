/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use dom::bindings::js::Root;
use dom::globalscope::GlobalScope;
use js::jsapi::{HandleObject, Heap, JSContext, JSObject, JSRuntime};
use js::jsapi::{JS_GetObjectRuntime, JS_GetContext};
use std::default::Default;

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T, U>(
        obj: Box<T>,
        global: &U,
        wrap_fn: unsafe fn(*mut JSContext, HandleObject, Box<T>) -> Root<T>)
        -> Root<T>
    where T: DomObject, U: DomObject
{
    let cx = global.reflector().get_cx();
    let jsobject = global.reflector().get_jsobject();
    unsafe {
        wrap_fn(cx, jsobject, obj)
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
    object: Heap<*mut JSObject>,
}

#[allow(unrooted_must_root)]
impl PartialEq for Reflector {
    fn eq(&self, other: &Reflector) -> bool {
        self.object.get() == other.object.get()
    }
}

impl Reflector {
    /// Get the reflector.
    #[inline]
    pub fn get_jsobject(&self) -> HandleObject {
        self.object.handle()
    }

    /// Get the JS runtime.
    #[inline]
    pub fn get_runtime(&self) -> *mut JSRuntime {
        unsafe { JS_GetObjectRuntime(self.object.get()) }
    }

    /// Get the JS context.
    #[inline]
    pub fn get_cx(&self) -> *mut JSContext {
        unsafe { JS_GetContext(self.get_runtime()) }
    }

    /// Initialize the reflector. (May be called only once.)
    pub fn set_jsobject(&mut self, object: *mut JSObject) {
        assert!(self.object.get().is_null());
        assert!(!object.is_null());
        self.object.set(object);
    }

    /// Return a pointer to the memory location at which the JS reflector
    /// object is stored. Used to root the reflector, as
    /// required by the JSAPI rooting APIs.
    pub fn rootable(&self) -> &Heap<*mut JSObject> {
        &self.object
    }

    /// Create an uninitialized `Reflector`.
    pub fn new() -> Reflector {
        Reflector {
            object: Heap::default(),
        }
    }
}

/// A trait to provide access to the `Reflector` for a DOM object.
pub trait DomObject {
    /// Returns the receiver's reflector.
    fn reflector(&self) -> &Reflector;

    /// Returns the global scope of the realm that the DomObject was created in.
    fn global(&self) -> Root<GlobalScope> where Self: Sized {
        GlobalScope::from_reflector(self)
    }
}

/// A trait to initialize the `Reflector` for a DOM object.
pub trait MutDomObject: DomObject {
    /// Initializes the Reflector
    fn init_reflector(&mut self, obj: *mut JSObject);
}
