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

use std::any::Any;
use std::ffi::c_void;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr;
use std::rc::{Rc, Weak};

use js::context::NoGC;
use js::jsapi::JSTracer;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::JSTraceable;
use crate::conversions::{IDLInterface, get_dom_class};
use crate::dom::UnrootedDom;
use crate::reflector::DomObject;
use crate::root::DomRoot;

/// A weak reference to a JS-managed DOM object.
///
/// `T` may be a parent interface of the object's concrete type (e.g. a
/// `WeakRef<Element>` to an `HTMLDivElement`). The object lives in an
/// `Rc` of its concrete type, so the weak pointer is stored type-erased and
/// refcounting and deallocation go through that concrete type.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub struct WeakRef<T: WeakReferenceable> {
    weak: Weak<dyn Any>,
    marker: PhantomData<T>,
}

/// Trait implemented by weak-referenceable interfaces.
pub trait WeakReferenceable: IDLInterface + DomObject + Sized {
    /// Downgrade a DOM object reference to a weak one.
    ///
    /// Panics if the object has not been reflected yet.
    fn downgrade(&self) -> WeakRef<Self> {
        let object = self.reflector().get_jsobject().get();
        assert!(!object.is_null(), "Cannot downgrade an unreflected object");
        // SAFETY: The reflector is a valid, non-null JS object.
        let dom_class = unsafe { get_dom_class(object) }.expect("Cannot downgrade an object whos reflector is not a DOM object");
        let downgrade = dom_class
            .downgrade_weak
            .expect("Cannot downgrade an object whos concrete type is not weak referenceable");
        WeakRef {
            // SAFETY: `downgrade` is `downgrade_erased`, for the concrete type of `self`, because
            // it comes from the `DOMClass` the reflector. Objects of weak-referenceable interfaces
            // are reflected from an `Rc<C>`, and `self` is borrowed, so that `Rc` is still alive.
            // DOM structs have the parent as the first field at offset 0, so `self` has the same
            // address as the concrete type.
            weak: unsafe { downgrade(self as *const Self as *const c_void) },
            marker: PhantomData,
        }
    }
}

/// Create a type-erased weak reference to the `Rc<T>` whose value is at `ptr`.
///
/// # Safety
/// `ptr` must be the value pointer (as returned by `Rc::as_ptr`) of an `Rc<T>`
/// that is still alive.
pub(crate) unsafe fn downgrade_erased<T: WeakReferenceable>(ptr: *const c_void) -> Weak<dyn Any> {
    // SAFETY: Guaranteed by the caller. `ManuallyDrop` leaves the strong count
    // untouched, because this temporary `Rc` does not own a strong reference.
    let rc = ManuallyDrop::new(unsafe { Rc::from_raw(ptr as *const T) });
    let weak: Weak<T> = Rc::downgrade(&rc);
    weak
}

impl<T: WeakReferenceable> Clone for WeakRef<T> {
    fn clone(&self) -> Self {
        WeakRef {
            weak: self.weak.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: WeakReferenceable> Eq for WeakRef<T> {}

impl<T: WeakReferenceable> Hash for WeakRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.weak.as_ptr().cast::<()>().hash(state);
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
        // SAFETY: `T` is the concrete type or one of its parents. DOM structs have the parent as
        // the first field at offset 0, so `self` has the same address as the concrete type.
        self.weak
            .upgrade()
            .map(|x| DomRoot::from_ref(unsafe { &*Rc::as_ptr(&x).cast::<T>() }))
    }

    /// Return an [`UnrootedDom`] reference to the data contained in this [`WeakRef`],
    /// if it is still alive.
    pub fn unrooted<'a>(&self, no_gc: &'a NoGC) -> Option<UnrootedDom<'a, T>> {
        // SAFETY: `T` is the concrete type or one of its parents. DOM structs have the parent as
        // the first field at offset 0, so `self` has the same address as the concrete type.
        self.weak
            .upgrade()
            .map(|value| UnrootedDom::from_ref(unsafe { &*Rc::as_ptr(&value).cast::<T>() }, no_gc))
    }

    /// Return whether the weakly-referenced object is still alive.
    pub fn is_alive(&self) -> bool {
        self.weak.strong_count() > 0
    }
}

impl<T: WeakReferenceable> MallocSizeOf for WeakRef<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T: WeakReferenceable> PartialEq for WeakRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.weak.ptr_eq(&other.weak)
    }
}

impl<T: WeakReferenceable> PartialEq<T> for WeakRef<T> {
    fn eq(&self, other: &T) -> bool {
        self.is_alive() && ptr::eq(self.weak.as_ptr().cast::<T>(), other)
    }
}

unsafe impl<T: WeakReferenceable> JSTraceable for WeakRef<T> {
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing.
    }
}
