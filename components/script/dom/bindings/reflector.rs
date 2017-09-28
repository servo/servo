/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use dom::bindings::conversions::DerivedFrom;
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use js::jsapi::{HandleObject, JSContext, JSObject, Heap};
use std::default::Default;

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T, U>(
        obj: Box<T>,
        global: &U,
        wrap_fn: unsafe fn(*mut JSContext, &GlobalScope, Box<T>) -> DomRoot<T>)
        -> DomRoot<T>
    where T: DomObject, U: DerivedFrom<GlobalScope>
{
    let global_scope = global.upcast();
    unsafe {
        wrap_fn(global_scope.get_cx(), global_scope, obj)
    }
}

/// A struct to store a reference to the reflector of a DOM object.
#[allow(unrooted_must_root)]
#[derive(HeapSizeOf)]
#[must_root]
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
pub trait DomObject: 'static {
    /// Returns the receiver's reflector.
    fn reflector(&self) -> &Reflector;

    /// Returns the global scope of the realm that the DomObject was created in.
    fn global(&self) -> DomRoot<GlobalScope> where Self: Sized {
        GlobalScope::from_reflector(self)
    }
}

impl DomObject for Reflector {
    fn reflector(&self) -> &Self {
        self
    }
}

/// A trait to initialize the `Reflector` for a DOM object.
pub trait MutDomObject: DomObject {
    /// Initializes the Reflector
    fn init_reflector(&mut self, obj: *mut JSObject);
}

impl MutDomObject for Reflector {
    fn init_reflector(&mut self, obj: *mut JSObject) {
        self.set_jsobject(obj)
    }
}
