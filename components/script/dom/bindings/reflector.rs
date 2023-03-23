/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::iterable::{Iterable, IterableIterator};
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;
use js::jsapi::{Heap, JSObject};
use js::rust::HandleObject;
use std::{default::Default, ops::Deref};

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T, U>(obj: Box<T>, global: &U) -> DomRoot<T>
where
    T: DomObject + DomObjectWrap,
    U: DerivedFrom<GlobalScope>,
{
    let global_scope = global.upcast();
    unsafe { T::WRAP(global_scope.get_cx(), global_scope, obj) }
}

/// A struct to store a reference to the reflector of a DOM object.
#[allow(unrooted_must_root)]
#[derive(MallocSizeOf)]
#[unrooted_must_root_lint::must_root]
// If you're renaming or moving this field, update the path in plugins::reflector as well
pub struct Reflector {
    #[ignore_malloc_size_of = "defined and measured in rust-mozjs"]
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
    const WRAP: unsafe fn(JSContext, &GlobalScope, Box<Self>) -> Root<Dom<Self>>;
}

/// A trait to provide a function pointer to wrap function for
/// DOM iterator interfaces.
pub trait DomObjectIteratorWrap: DomObjectWrap + JSTraceable + Iterable {
    /// Function pointer to the wrap function for IterableIterator<T>
    const ITER_WRAP: unsafe fn(
        JSContext,
        &GlobalScope,
        Box<IterableIterator<Self>>,
    ) -> Root<Dom<IterableIterator<Self>>>;
}

/// A marker trait denoting DOM objects that are constrained to a single
/// realm. It's implemented by most DOM objects with a notable exception being
/// `WindowProxy`.
///
/// The reflectors of transplantable types may move between realms, and the
/// compartment invariants [prohibit][1] tracable references from crossing
/// compartment boundaries. For this reason, references to transplantable types
/// must be held by `*TransplantableDom*<T>`, which are designed to handle
/// cross-realm cases. Even when using such reference wrappers, a care must be
/// taken to ensure cross-realm references do not occur. For instance,
/// a transplantable DOM object must hold references to other DOM objects by
/// `*TransplantableDom*<T>` because their realms can "move" in relative to
/// the transplantable DOM object's.
///
/// [1]: https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Internals/Garbage_collection#compartments
pub trait Untransplantable: DomObject {}

/// Unsafely adds [`Untransplantable`] to the wrapped [`DomObject`].
#[derive(JSTraceable)]
#[repr(transparent)]
pub struct AssertUntransplantable<T: DomObject>(T);

impl<T: DomObject> AssertUntransplantable<T> {
    /// Wrap a reference with `AssertUntransplantable`.
    ///
    /// # Safety
    ///
    /// The constructed `&Self` must not be traced from a GC thing with an
    /// associated compartment. For example, if you create `Dom<T>` from the
    /// returned reference, storing it in a `DomObject` might be unsafe.
    #[inline]
    pub unsafe fn from_ref(x: &T) -> &Self {
        // Safety: `*x` and `Self` has the same representation
        &*(x as *const T as *const Self)
    }
}

impl<T: DomObject> DomObject for AssertUntransplantable<T> {
    #[inline]
    fn reflector(&self) -> &Reflector {
        self.0.reflector()
    }
}

impl<T: DomObject> Untransplantable for AssertUntransplantable<T> {}

impl<T: DomObject> Deref for AssertUntransplantable<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
