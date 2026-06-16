/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::UnsafeCell;
use std::ops::Deref;
use std::{mem, ptr};

use js::context::NoGC;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::DomObject;
use crate::assert::{assert_in_layout, assert_in_script};
use crate::conversions::DerivedFrom;
use crate::inheritance::Castable;
use crate::root::{Dom, DomRoot};

pub trait ToLayout<'dom, T: DomObject, L: LayoutFromRaw<'dom, T>> {
    /// Returns `LayoutDom<T>` containing the same pointer.
    ///
    /// # Safety
    ///
    /// The `self` parameter to this method must meet all the requirements of [`ptr::NonNull::as_ref`].
    unsafe fn to_layout(&self) -> L;
}

impl<'dom, T: DomObject, L: LayoutFromRaw<'dom, T>> ToLayout<'dom, T, L> for Dom<T> {
    unsafe fn to_layout(&self) -> L {
        assert_in_layout();
        L::from_raw(unsafe { self.as_ptr().as_ref().unwrap() })
    }
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`.  Essentially a `Cell<Dom<T>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable)]
pub struct MutDom<T: DomObject> {
    val: UnsafeCell<Dom<T>>,
}

impl<T: DomObject> MutDom<T> {
    /// Create a new `MutDom`.
    pub fn new(initial: &T) -> MutDom<T> {
        assert_in_script();
        MutDom {
            val: UnsafeCell::new(Dom::from_ref(initial)),
        }
    }

    /// Set this `MutDom` to the given value.
    pub fn set(&self, val: &T) {
        assert_in_script();
        unsafe {
            *self.val.get() = Dom::from_ref(val);
        }
    }

    /// Get the value in this `MutDom`.
    pub fn get(&self) -> DomRoot<T> {
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

/// A struct to make Unrooted Dom objects work. By taking a no_gc as reference, we ensure that the lifetime of this object
/// is bounded by the lifetime of NoGC which enforces no gc happening.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub struct UnrootedDom<'a, T: DomObject> {
    inner: Dom<T>,
    no_gc: &'a NoGC,
}

impl<'a, T: DomObject> UnrootedDom<'a, T> {
    /// Construct an `UnrootedDom` with the lifetime of `NoGC`. This is safe, as `NoGC` implies no garbage collection will happen
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub fn from_dom(object: Dom<T>, no_gc: &'a NoGC) -> UnrootedDom<'a, T> {
        UnrootedDom {
            inner: object,
            no_gc,
        }
    }
}

impl<'a, T: DomObject> Deref for UnrootedDom<'a, T> {
    type Target = Dom<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Safety:
/// We enforce the same lifetime as the given `UnrootedDom`, so the same
/// guarantee about no GC happening in this lifetime.
impl<'a, T: Castable> UnrootedDom<'a, T> {
    /// Cast a DOM object root upwards to one of the interfaces it derives from.
    pub fn upcast<U>(dom: UnrootedDom<'a, T>) -> UnrootedDom<'a, U>
    where
        U: Castable,
        T: DerivedFrom<U>,
    {
        UnrootedDom {
            inner: unsafe { mem::transmute::<Dom<T>, Dom<U>>(dom.inner) },
            no_gc: dom.no_gc,
        }
    }

    /// Cast a DOM object root downwards to one of the interfaces it might implement.
    pub fn downcast<U>(dom: UnrootedDom<'a, T>) -> Option<UnrootedDom<'a, U>>
    where
        U: DerivedFrom<T>,
    {
        if dom.is::<U>() {
            Some(UnrootedDom {
                inner: unsafe { mem::transmute::<Dom<T>, Dom<U>>(dom.inner) },
                no_gc: dom.no_gc,
            })
        } else {
            None
        }
    }
}

impl<'a, T: DomObject> PartialEq<T> for UnrootedDom<'a, T> {
    fn eq(&self, other: &T) -> bool {
        self.inner == other
    }
}

impl<'a, T: DomObject> PartialEq<UnrootedDom<'a, T>> for UnrootedDom<'a, T> {
    fn eq(&self, other: &UnrootedDom<'a, T>) -> bool {
        self.inner == other.inner
    }
}

/// Trait that creates a specific struct from a raw DomObject.
/// The implementer needs to be sure that this does not violate any lifetimes
/// # Safety
/// The dom object needs the lifetimes to be safe.
/// Only [`LayoutDom`] should implement this.
pub unsafe trait LayoutFromRaw<'dom, T: DomObject> {
    fn from_raw(d: &'dom T) -> Self;
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`, with nullability represented by an enclosing Option wrapper.
/// Essentially a `Cell<Option<Dom<T>>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable)]
pub struct MutNullableDom<T: DomObject> {
    ptr: UnsafeCell<Option<Dom<T>>>,
}

impl<T: DomObject> MutNullableDom<T> {
    /// Create a new `MutNullableDom`.
    pub fn new(initial: Option<&T>) -> MutNullableDom<T> {
        assert_in_script();
        MutNullableDom {
            ptr: UnsafeCell::new(initial.map(Dom::from_ref)),
        }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    pub fn or_init<F>(&self, cb: F) -> DomRoot<T>
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
    /// # Safety
    /// Needs to meet the safety requirements of [`lLayoutFromRaw`].
    pub unsafe fn get_inner_as_layout<'dom, L: LayoutFromRaw<'dom, T>>(&'dom self) -> Option<L> {
        assert_in_layout();
        unsafe { (*self.ptr.get()).as_ref().map(|js| js.to_layout()) }
    }

    /// Get a rooted value out of this object
    pub fn get(&self) -> Option<DomRoot<T>> {
        assert_in_script();
        unsafe { ptr::read(self.ptr.get()).map(|o| DomRoot::from_ref(&*o)) }
    }

    /// Get the `DomObject` without rooting it. Constructing an UnrootedDom. This is safe
    /// as we take a reference to NoGC and bound the lifetime by NoGC bound. This implies that
    /// while the `UnrootedDom` is alive we do not have a GC run.
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub fn get_unrooted<'a>(&self, no_gc: &'a NoGC) -> Option<UnrootedDom<'a, T>> {
        assert_in_script();
        let ptr = unsafe { ptr::read(self.ptr.get()) };
        ptr.map(|o| Dom::from_ref(&*o))
            .map(|dom| UnrootedDom { inner: dom, no_gc })
    }

    /// Set this `MutNullableDom` to the given value.
    pub fn set(&self, val: Option<&T>) {
        assert_in_script();
        unsafe {
            *self.ptr.get() = val.map(|p| Dom::from_ref(p));
        }
    }

    /// Gets the current value out of this object and sets it to `None`.
    pub fn take(&self) -> Option<DomRoot<T>> {
        let value = self.get();
        self.set(None);
        value
    }

    /// Sets the current value of this [`MutNullableDom`] to `None`.
    pub fn clear(&self) {
        self.set(None)
    }

    /// Runs the given callback on the object if it's not null.
    pub fn if_is_some<F, R>(&self, cb: F) -> Option<&R>
    where
        F: FnOnce(&T) -> &R,
    {
        unsafe {
            if let Some(ref value) = *self.ptr.get() {
                Some(cb(value))
            } else {
                None
            }
        }
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
