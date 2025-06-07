/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::{Heap, JSObject};
use js::rust::HandleObject;
use malloc_size_of_derive::MallocSizeOf;

use crate::interfaces::GlobalScopeHelpers;
use crate::iterable::{Iterable, IterableIterator};
use crate::realms::InRealm;
use crate::root::{Dom, DomRoot, Root};
use crate::script_runtime::{CanGc, JSContext};
use crate::{DomTypes, JSTraceable};

/// A struct to store a reference to the reflector of a DOM object.
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
#[derive(MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
// If you're renaming or moving this field, update the path in plugins::reflector as well
pub struct Reflector {
    #[ignore_malloc_size_of = "defined and measured in rust-mozjs"]
    object: Heap<*mut JSObject>,
}

unsafe impl js::gc::Traceable for Reflector {
    unsafe fn trace(&self, _: *mut js::jsapi::JSTracer) {}
}

#[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
    ///
    /// # Safety
    ///
    /// The provided [`JSObject`] pointer must point to a valid [`JSObject`].
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
pub trait DomObject: js::gc::Traceable + 'static {
    /// Returns the receiver's reflector.
    fn reflector(&self) -> &Reflector;
}

impl DomObject for Reflector {
    fn reflector(&self) -> &Self {
        self
    }
}

/// A trait to initialize the `Reflector` for a DOM object.
pub trait MutDomObject: DomObject {
    /// Initializes the Reflector
    ///
    /// # Safety
    ///
    /// The provided [`JSObject`] pointer must point to a valid [`JSObject`].
    unsafe fn init_reflector(&self, obj: *mut JSObject);
}

impl MutDomObject for Reflector {
    unsafe fn init_reflector(&self, obj: *mut JSObject) {
        self.set_jsobject(obj)
    }
}

pub trait DomGlobalGeneric<D: DomTypes>: DomObject {
    /// Returns the [`GlobalScope`] of the realm that the [`DomObject`] was created in.  If this
    /// object is a `Node`, this will be different from it's owning `Document` if adopted by. For
    /// `Node`s it's almost always better to use `NodeTraits::owning_global`.
    fn global_(&self, realm: InRealm) -> DomRoot<D::GlobalScope>
    where
        Self: Sized,
    {
        D::GlobalScope::from_reflector(self, realm)
    }
}

impl<D: DomTypes, T: DomObject> DomGlobalGeneric<D> for T {}

/// A trait to provide a function pointer to wrap function for DOM objects.
pub trait DomObjectWrap<D: DomTypes>: Sized + DomObject + DomGlobalGeneric<D> {
    /// Function pointer to the general wrap function type
    #[allow(clippy::type_complexity)]
    const WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Box<Self>,
        CanGc,
    ) -> Root<Dom<Self>>;
}

/// A trait to provide a function pointer to wrap function for
/// DOM iterator interfaces.
pub trait DomObjectIteratorWrap<D: DomTypes>: DomObjectWrap<D> + JSTraceable + Iterable {
    /// Function pointer to the wrap function for `IterableIterator<T>`
    #[allow(clippy::type_complexity)]
    const ITER_WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Box<IterableIterator<D, Self>>,
        CanGc,
    ) -> Root<Dom<IterableIterator<D, Self>>>;
}
