/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use std::default::Default;

use js::jsapi::{Heap, JSObject};
use js::rust::HandleObject;

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::iterable::{Iterable, IterableIterator};
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use crate::realms::AlreadyInRealm;
use crate::script_runtime::JSContext;

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T, U>(obj: Box<T>, global: &U) -> DomRoot<T>
where
    T: DomObject + DomObjectWrap,
    U: DerivedFrom<GlobalScope>,
{
    let global_scope = global.upcast();
    unsafe { T::WRAP(GlobalScope::get_cx(), global_scope, None, obj) }
}

pub fn reflect_dom_object_with_proto<T, U>(
    obj: Box<T>,
    global: &U,
    proto: Option<HandleObject>,
) -> DomRoot<T>
where
    T: DomObject + DomObjectWrap,
    U: DerivedFrom<GlobalScope>,
{
    let global_scope = global.upcast();
    unsafe { T::WRAP(GlobalScope::get_cx(), global_scope, proto, obj) }
}

/// A struct to store a reference to the reflector of a DOM object.
#[allow(crown::unrooted_must_root)]
#[derive(MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
// If you're renaming or moving this field, update the path in plugins::reflector as well
pub struct Reflector {
    #[ignore_malloc_size_of = "defined and measured in rust-mozjs"]
    object: Heap<*mut JSObject>,
}

#[allow(crown::unrooted_must_root)]
impl PartialEq for Reflector {
    fn eq(&self, other: &Reflector) -> bool {
        self.object.get() == other.object.get()
    }
}

impl Reflector {
    /// Get the reflector.
    #[inline]
    pub fn get_jsobject(&self) -> HandleObject {
        // We're rooted, so it's safe to hand out a handle to object in Heap
        unsafe { HandleObject::from_raw(self.object.handle()) }
    }

    /// Initialize the reflector. (May be called only once.)
    pub unsafe fn set_jsobject(&self, object: *mut JSObject) {
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
    // These are used by the bindings and do not need `default()` functions.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Reflector {
        Reflector {
            object: Heap::default(),
        }
    }
}

/// A trait to provide access to the `Reflector` for a DOM object.
pub trait DomObject: JSTraceable + 'static {
    /// Returns the receiver's reflector.
    fn reflector(&self) -> &Reflector;

    /// Returns the global scope of the realm that the DomObject was created in.
    fn global(&self) -> DomRoot<GlobalScope>
    where
        Self: Sized,
    {
        let realm = AlreadyInRealm::assert_for_cx(GlobalScope::get_cx());
        GlobalScope::from_reflector(self, &realm)
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
    unsafe fn init_reflector(&self, obj: *mut JSObject);
}

impl MutDomObject for Reflector {
    unsafe fn init_reflector(&self, obj: *mut JSObject) {
        self.set_jsobject(obj)
    }
}

/// A trait to provide a function pointer to wrap function for DOM objects.
pub trait DomObjectWrap: Sized + DomObject {
    /// Function pointer to the general wrap function type
    const WRAP: unsafe fn(
        JSContext,
        &GlobalScope,
        Option<HandleObject>,
        Box<Self>,
    ) -> Root<Dom<Self>>;
}

/// A trait to provide a function pointer to wrap function for
/// DOM iterator interfaces.
pub trait DomObjectIteratorWrap: DomObjectWrap + JSTraceable + Iterable {
    /// Function pointer to the wrap function for `IterableIterator<T>`
    const ITER_WRAP: unsafe fn(
        JSContext,
        &GlobalScope,
        Option<HandleObject>,
        Box<IterableIterator<Self>>,
    ) -> Root<Dom<IterableIterator<Self>>>;
}
