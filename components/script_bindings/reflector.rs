/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use js::jsapi::{AddAssociatedMemory, Heap, JSObject, MemoryUse, RemoveAssociatedMemory};
use js::rust::HandleObject;
use malloc_size_of_derive::MallocSizeOf;

use crate::interfaces::GlobalScopeHelpers;
use crate::iterable::{Iterable, IterableIterator};
use crate::realms::InRealm;
use crate::root::{Dom, DomRoot, Root};
use crate::script_runtime::{CanGc, JSContext};
use crate::{DomTypes, JSTraceable};

pub trait AssociatedMemorySize: Default {
    fn size(&self) -> usize;
}

impl AssociatedMemorySize for () {
    fn size(&self) -> usize {
        0
    }
}

#[derive(Default, MallocSizeOf)]
pub struct AssociatedMemory(Cell<usize>);

impl AssociatedMemorySize for AssociatedMemory {
    fn size(&self) -> usize {
        self.0.get()
    }
}

/// A struct to store a reference to the reflector of a DOM object.
#[derive(MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
// If you're renaming or moving this field, update the path in plugins::reflector as well
pub struct Reflector<T = ()> {
    #[ignore_malloc_size_of = "defined and measured in rust-mozjs"]
    object: Heap<*mut JSObject>,
    /// Associated memory size (of rust side). Used for memory reporting to SM.
    size: T,
}

unsafe impl<T> js::gc::Traceable for Reflector<T> {
    unsafe fn trace(&self, _: *mut js::jsapi::JSTracer) {}
}

impl<T> PartialEq for Reflector<T> {
    fn eq(&self, other: &Reflector<T>) -> bool {
        self.object.get() == other.object.get()
    }
}

impl<T> Reflector<T> {
    /// Get the reflector.
    #[inline]
    pub fn get_jsobject(&self) -> HandleObject<'_> {
        // We're rooted, so it's safe to hand out a handle to object in Heap
        unsafe { HandleObject::from_raw(self.object.handle()) }
    }

    /// Initialize the reflector. (May be called only once.)
    ///
    /// # Safety
    ///
    /// The provided [`JSObject`] pointer must point to a valid [`JSObject`].
    unsafe fn set_jsobject(&self, object: *mut JSObject) {
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
}

impl<T: AssociatedMemorySize> Reflector<T> {
    /// Create an uninitialized `Reflector`.
    // These are used by the bindings and do not need `default()` functions.
    #[expect(clippy::new_without_default)]
    pub fn new() -> Reflector<T> {
        Reflector {
            object: Heap::default(),
            size: T::default(),
        }
    }

    pub fn rust_size<D>(&self, _: &D) -> usize {
        size_of::<D>() + size_of::<Box<D>>() + self.size.size()
    }

    /// This function should be called from finalize of the DOM objects
    pub fn drop_memory<D>(&self, d: &D) {
        unsafe {
            RemoveAssociatedMemory(self.object.get(), self.rust_size(d), MemoryUse::DOMBinding);
        }
    }
}

impl Reflector<AssociatedMemory> {
    /// Update the associated memory size.
    pub fn update_memory_size<D>(&self, d: &D, new_size: usize) {
        if self.size.size() == new_size {
            return;
        }
        unsafe {
            RemoveAssociatedMemory(self.object.get(), self.rust_size(d), MemoryUse::DOMBinding);
            self.size.0.set(new_size);
            AddAssociatedMemory(self.object.get(), self.rust_size(d), MemoryUse::DOMBinding);
        }
    }
}

/// A trait to provide access to the `Reflector` for a DOM object.
pub trait DomObject: js::gc::Traceable + 'static {
    type ReflectorType: AssociatedMemorySize;
    /// Returns the receiver's reflector.
    fn reflector(&self) -> &Reflector<Self::ReflectorType>;
}

impl DomObject for Reflector<()> {
    type ReflectorType = ();

    fn reflector(&self) -> &Reflector<Self::ReflectorType> {
        self
    }
}

impl DomObject for Reflector<AssociatedMemory> {
    type ReflectorType = AssociatedMemory;

    fn reflector(&self) -> &Reflector<Self::ReflectorType> {
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
    unsafe fn init_reflector<D>(&self, obj: *mut JSObject);
}

impl MutDomObject for Reflector<()> {
    unsafe fn init_reflector<D>(&self, obj: *mut JSObject) {
        unsafe {
            js::jsapi::AddAssociatedMemory(
                obj,
                size_of::<D>() + size_of::<Box<D>>(),
                MemoryUse::DOMBinding,
            );
            self.set_jsobject(obj);
        }
    }
}

impl MutDomObject for Reflector<AssociatedMemory> {
    unsafe fn init_reflector<D>(&self, obj: *mut JSObject) {
        unsafe {
            js::jsapi::AddAssociatedMemory(
                obj,
                size_of::<D>() + size_of::<Box<D>>(),
                MemoryUse::DOMBinding,
            );
            self.set_jsobject(obj);
        }
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
    #[expect(clippy::type_complexity)]
    const WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Box<Self>,
        CanGc,
    ) -> Root<Dom<Self>>;
}

/// A trait to provide a function pointer to wrap function for DOM objects.
pub trait WeakReferenceableDomObjectWrap<D: DomTypes>:
    Sized + DomObject + DomGlobalGeneric<D>
{
    /// Function pointer to the general wrap function type
    #[expect(clippy::type_complexity)]
    const WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Rc<Self>,
        CanGc,
    ) -> Root<Dom<Self>>;
}

/// A trait to provide a function pointer to wrap function for
/// DOM iterator interfaces.
pub trait DomObjectIteratorWrap<D: DomTypes>: DomObjectWrap<D> + JSTraceable + Iterable {
    /// Function pointer to the wrap function for `IterableIterator<T>`
    #[expect(clippy::type_complexity)]
    const ITER_WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Box<IterableIterator<D, Self>>,
        CanGc,
    ) -> Root<Dom<IterableIterator<D, Self>>>;
}
