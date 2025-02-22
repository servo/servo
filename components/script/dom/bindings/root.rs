/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Smart pointers for the JS-managed DOM objects.
//!
//! The DOM is made up of DOM objects whose lifetime is entirely controlled by
//! the whims of the SpiderMonkey garbage collector. The types in this module
//! are designed to ensure that any interactions with said Rust types only
//! occur on values that will remain alive the entire time.
//!
//! Here is a brief overview of the important types:
//!
//! - `Root<T>`: a stack-based rooted value.
//! - `DomRoot<T>`: a stack-based reference to a rooted DOM object.
//! - `Dom<T>`: a reference to a DOM object that can automatically be traced by
//!   the GC when encountered as a field of a Rust structure.
//!
//! `Dom<T>` does not allow access to their inner value without explicitly
//! creating a stack-based root via the `root` method. This returns a `DomRoot<T>`,
//! which causes the JS-owned value to be uncollectable for the duration of the
//! `Root` object's lifetime. A reference to the object can then be obtained
//! from the `Root` object. These references are not allowed to outlive their
//! originating `DomRoot<T>`.
//!

use std::cell::{OnceCell, UnsafeCell};
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::{mem, ptr};

use js::jsapi::{JSObject, JSTracer};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
pub(crate) use script_bindings::root::*;
use script_layout_interface::TrustedNodeAddress;
use style::thread_state;

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::node::Node;

pub(crate) struct ThreadLocalStackRoots<'a>(PhantomData<&'a u32>);

impl<'a> ThreadLocalStackRoots<'a> {
    pub(crate) fn new(roots: &'a RootCollection) -> Self {
        STACK_ROOTS.with(|r| r.set(Some(roots)));
        ThreadLocalStackRoots(PhantomData)
    }
}

impl Drop for ThreadLocalStackRoots<'_> {
    fn drop(&mut self) {
        STACK_ROOTS.with(|r| r.set(None));
    }
}

/// Get a slice of references to DOM objects.
pub(crate) trait DomSlice<T>
where
    T: JSTraceable + DomObject,
{
    /// Returns the slice of `T` references.
    fn r(&self) -> &[&T];
}

impl<T> DomSlice<T> for [Dom<T>]
where
    T: JSTraceable + DomObject,
{
    #[inline]
    fn r(&self) -> &[&T] {
        let _ = mem::transmute::<Dom<T>, &T>;
        unsafe { &*(self as *const [Dom<T>] as *const [&T]) }
    }
}

pub(crate) trait ToLayout<T> {
    /// Returns `LayoutDom<T>` containing the same pointer.
    ///
    /// # Safety
    ///
    /// The `self` parameter to this method must meet all the requirements of [`ptr::NonNull::as_ref`].
    unsafe fn to_layout(&self) -> LayoutDom<T>;
}

impl<T: DomObject> ToLayout<T> for Dom<T> {
    unsafe fn to_layout(&self) -> LayoutDom<T> {
        assert_in_layout();
        LayoutDom {
            value: self.as_ptr().as_ref().unwrap(),
        }
    }
}

/// An unrooted reference to a DOM object for use in layout. `Layout*Helpers`
/// traits must be implemented on this.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
#[repr(transparent)]
pub(crate) struct LayoutDom<'dom, T> {
    value: &'dom T,
}

impl<'dom, T> LayoutDom<'dom, T>
where
    T: Castable,
{
    /// Cast a DOM object root upwards to one of the interfaces it derives from.
    pub(crate) fn upcast<U>(&self) -> LayoutDom<'dom, U>
    where
        U: Castable,
        T: DerivedFrom<U>,
    {
        assert_in_layout();
        LayoutDom {
            value: self.value.upcast::<U>(),
        }
    }

    /// Cast a DOM object downwards to one of the interfaces it might implement.
    pub(crate) fn downcast<U>(&self) -> Option<LayoutDom<'dom, U>>
    where
        U: DerivedFrom<T>,
    {
        assert_in_layout();
        self.value.downcast::<U>().map(|value| LayoutDom { value })
    }

    /// Returns whether this inner object is a U.
    pub(crate) fn is<U>(&self) -> bool
    where
        U: DerivedFrom<T>,
    {
        assert_in_layout();
        self.value.is::<U>()
    }
}

impl<T> LayoutDom<'_, T>
where
    T: DomObject,
{
    /// Get the reflector.
    pub(crate) unsafe fn get_jsobject(&self) -> *mut JSObject {
        assert_in_layout();
        self.value.reflector().get_jsobject().get()
    }
}

impl<T> Copy for LayoutDom<'_, T> {}

impl<T> PartialEq for LayoutDom<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.value, other.value)
    }
}

impl<T> Eq for LayoutDom<'_, T> {}

impl<T> Hash for LayoutDom<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.value as *const T).hash(state)
    }
}

impl<T> Clone for LayoutDom<'_, T> {
    #[inline]
    #[allow(clippy::non_canonical_clone_impl)]
    fn clone(&self) -> Self {
        assert_in_layout();
        *self
    }
}

impl LayoutDom<'_, Node> {
    /// Create a new JS-owned value wrapped from an address known to be a
    /// `Node` pointer.
    pub(crate) unsafe fn from_trusted_node_address(inner: TrustedNodeAddress) -> Self {
        assert_in_layout();
        let TrustedNodeAddress(addr) = inner;
        LayoutDom {
            value: &*(addr as *const Node),
        }
    }
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`.  Essentially a `Cell<Dom<T>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable)]
pub(crate) struct MutDom<T: DomObject> {
    val: UnsafeCell<Dom<T>>,
}

impl<T: DomObject> MutDom<T> {
    /// Create a new `MutDom`.
    pub(crate) fn new(initial: &T) -> MutDom<T> {
        assert_in_script();
        MutDom {
            val: UnsafeCell::new(Dom::from_ref(initial)),
        }
    }

    /// Set this `MutDom` to the given value.
    pub(crate) fn set(&self, val: &T) {
        assert_in_script();
        unsafe {
            *self.val.get() = Dom::from_ref(val);
        }
    }

    /// Get the value in this `MutDom`.
    pub(crate) fn get(&self) -> DomRoot<T> {
        assert_in_script();
        unsafe { DomRoot::from_ref(&*ptr::read(self.val.get())) }
    }
}

impl<T: DomObject> MallocSizeOf for MutDom<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

impl<T: DomObject> PartialEq for MutDom<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.val.get() == *other.val.get() }
    }
}

impl<T: DomObject + PartialEq> PartialEq<T> for MutDom<T> {
    fn eq(&self, other: &T) -> bool {
        unsafe { **self.val.get() == *other }
    }
}

pub(crate) fn assert_in_layout() {
    debug_assert!(thread_state::get().is_layout());
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`, with nullability represented by an enclosing Option wrapper.
/// Essentially a `Cell<Option<Dom<T>>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable)]
pub(crate) struct MutNullableDom<T: DomObject> {
    ptr: UnsafeCell<Option<Dom<T>>>,
}

impl<T: DomObject> MutNullableDom<T> {
    /// Create a new `MutNullableDom`.
    pub(crate) fn new(initial: Option<&T>) -> MutNullableDom<T> {
        assert_in_script();
        MutNullableDom {
            ptr: UnsafeCell::new(initial.map(Dom::from_ref)),
        }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    pub(crate) fn or_init<F>(&self, cb: F) -> DomRoot<T>
    where
        F: FnOnce() -> DomRoot<T>,
    {
        assert_in_script();
        match self.get() {
            Some(inner) => inner,
            None => {
                let inner = cb();
                self.set(Some(&inner));
                inner
            },
        }
    }

    /// Retrieve a copy of the inner optional `Dom<T>` as `LayoutDom<T>`.
    /// For use by layout, which can't use safe types like Temporary.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) unsafe fn get_inner_as_layout(&self) -> Option<LayoutDom<T>> {
        assert_in_layout();
        (*self.ptr.get()).as_ref().map(|js| js.to_layout())
    }

    /// Get a rooted value out of this object
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn get(&self) -> Option<DomRoot<T>> {
        assert_in_script();
        unsafe { ptr::read(self.ptr.get()).map(|o| DomRoot::from_ref(&*o)) }
    }

    /// Set this `MutNullableDom` to the given value.
    pub(crate) fn set(&self, val: Option<&T>) {
        assert_in_script();
        unsafe {
            *self.ptr.get() = val.map(|p| Dom::from_ref(p));
        }
    }

    /// Gets the current value out of this object and sets it to `None`.
    pub(crate) fn take(&self) -> Option<DomRoot<T>> {
        let value = self.get();
        self.set(None);
        value
    }
}

impl<T: DomObject> PartialEq for MutNullableDom<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.ptr.get() == *other.ptr.get() }
    }
}

impl<T: DomObject> PartialEq<Option<&T>> for MutNullableDom<T> {
    fn eq(&self, other: &Option<&T>) -> bool {
        unsafe { *self.ptr.get() == other.map(Dom::from_ref) }
    }
}

impl<T: DomObject> Default for MutNullableDom<T> {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn default() -> MutNullableDom<T> {
        assert_in_script();
        MutNullableDom {
            ptr: UnsafeCell::new(None),
        }
    }
}

impl<T: DomObject> MallocSizeOf for MutNullableDom<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

/// A holder that allows to lazily initialize the value only once
/// `Dom<T>`, using OnceCell
/// Essentially a `OnceCell<Dom<T>>`.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DomOnceCell<T: DomObject> {
    ptr: OnceCell<Dom<T>>,
}

impl<T> DomOnceCell<T>
where
    T: DomObject,
{
    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn init_once<F>(&self, cb: F) -> &T
    where
        F: FnOnce() -> DomRoot<T>,
    {
        assert_in_script();
        self.ptr.get_or_init(|| Dom::from_ref(&cb()))
    }
}

impl<T: DomObject> Default for DomOnceCell<T> {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn default() -> DomOnceCell<T> {
        assert_in_script();
        DomOnceCell {
            ptr: OnceCell::new(),
        }
    }
}

impl<T: DomObject> MallocSizeOf for DomOnceCell<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

#[cfg_attr(crown, allow(crown::unrooted_must_root))]
unsafe impl<T: DomObject> JSTraceable for DomOnceCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if let Some(ptr) = self.ptr.get() {
            ptr.trace(trc);
        }
    }
}

impl<'dom, T> LayoutDom<'dom, T>
where
    T: 'dom + DomObject,
{
    /// Returns a reference to the interior of this JS object. The fact
    /// that this is unsafe is what necessitates the layout wrappers.
    pub(crate) fn unsafe_get(self) -> &'dom T {
        assert_in_layout();
        self.value
    }

    /// Transforms a slice of `Dom<T>` into a slice of `LayoutDom<T>`.
    // FIXME(nox): This should probably be done through a ToLayout trait.
    pub(crate) unsafe fn to_layout_slice(slice: &'dom [Dom<T>]) -> &'dom [LayoutDom<'dom, T>] {
        // This doesn't compile if Dom and LayoutDom don't have the same
        // representation.
        let _ = mem::transmute::<Dom<T>, LayoutDom<T>>;
        &*(slice as *const [Dom<T>] as *const [LayoutDom<T>])
    }
}
