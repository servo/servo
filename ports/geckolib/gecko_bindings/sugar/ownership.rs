/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::sync::Arc;

/// Indicates that a given Servo type has a corresponding
/// Gecko FFI type
/// The correspondence is not defined at this stage,
/// use HasArcFFI or similar traits to define it
pub unsafe trait HasFFI : Sized {
    type FFIType: Sized;
}

/// Indicates that a given Servo type has the same layout
/// as the corresponding HasFFI::FFIType type
pub unsafe trait HasSimpleFFI : HasFFI {
    fn as_ffi(&self) -> &Self::FFIType {
        unsafe { transmute(self) }
    }
    fn as_ffi_mut(&mut self) -> &mut Self::FFIType {
        unsafe { transmute(self) }
    }
    fn from_ffi(ffi: &Self::FFIType) -> &Self {
        unsafe { transmute(ffi) }
    }
    fn from_ffi_mut(ffi: &mut Self::FFIType) -> &mut Self {
        unsafe { transmute(ffi) }
    }
}

/// Indicates that the given Servo type is passed over FFI
/// as a Box
pub unsafe trait HasBoxFFI : HasSimpleFFI {
    fn into_ffi(self: Box<Self>) -> Owned<Self::FFIType> {
        unsafe { transmute(self) }
    }
}

/// Helper trait for conversions between FFI Strong/Borrowed types and Arcs
///
/// Should be implemented by types which are passed over FFI as Arcs
/// via Strong and Borrowed
///
/// In this case, the FFIType is the rough equivalent of ArcInner<Self>
pub unsafe trait HasArcFFI : HasFFI {
    // these methods can't be on Borrowed because it leads to an unspecified
    // impl parameter
    /// Artificially increments the refcount of a borrowed Arc over FFI.
    unsafe fn addref(ptr: Borrowed<Self::FFIType>) {
        forget(ptr.as_arc::<Self>().clone())
    }

    /// Given a (possibly null) borrowed FFI reference, decrements the refcount.
    /// Unsafe since it doesn't consume the backing Arc. Run it only when you
    /// know that a strong reference to the backing Arc is disappearing
    /// (usually on the C++ side) without running the Arc destructor.
    unsafe fn release(ptr: Borrowed<Self::FFIType>) {
        if let Some(arc) = ptr.as_arc_opt::<Self>() {
            let _: Arc<_> = ptr::read(arc as *const Arc<_>);
        }
    }
}

#[repr(C)]
/// Gecko-FFI-safe borrowed Arc (&T where T is an ArcInner).
/// This can be null.
pub struct Borrowed<'a, T: 'a> {
    ptr: *const T,
    _marker: PhantomData<&'a T>,
}

// manual impls because derive doesn't realize that `T: Clone` isn't necessary
impl<'a, T> Copy for Borrowed<'a, T> {}

impl<'a, T> Clone for Borrowed<'a, T> {
    fn clone(&self) -> Self { *self }
}

impl<'a, T> Borrowed<'a, T> {
    pub fn is_null(&self) -> bool {
        self.ptr == ptr::null()
    }

    pub fn as_arc_opt<U>(&self) -> Option<&Arc<U>> where U: HasArcFFI<FFIType = T> {
        unsafe {
            if self.is_null() {
                None
            } else {
                Some(transmute::<&Borrowed<_>, &Arc<_>>(self))
            }
        }
    }

    /// Converts a borrowed FFI reference to a borrowed Arc.
    /// Panics on null
    pub fn as_arc<U>(&self) -> &Arc<U> where U: HasArcFFI<FFIType = T> {
        self.as_arc_opt().unwrap()
    }
}

#[repr(C)]
/// Gecko-FFI-safe Arc (T is an ArcInner).
/// This can be null.
pub struct Strong<T> {
    ptr: *const T,
    _marker: PhantomData<T>,
}

impl<T> Strong<T> {
    pub fn is_null(&self) -> bool {
        self.ptr == ptr::null()
    }

    /// Given a non-null strong FFI reference, converts it into an Arc.
    /// Panics on null.
    pub fn into_arc<U>(self) -> Arc<U> where U: HasArcFFI<FFIType = T> {
        assert!(!self.is_null());
        unsafe { transmute(self) }
    }

    /// Produces a null strong FFI reference
    pub fn null_strong() -> Self {
        unsafe { transmute(ptr::null::<T>()) }
    }
}

pub unsafe trait FFIArcHelpers {
    type Inner: HasArcFFI;
    /// Converts an Arc into a strong FFI reference.
    fn into_strong(self) -> Strong<<Self::Inner as HasFFI>::FFIType>;
    /// Produces a borrowed FFI reference by borrowing an Arc.
    fn as_borrowed(&self) -> Borrowed<<Self::Inner as HasFFI>::FFIType>;
}

unsafe impl<T: HasArcFFI> FFIArcHelpers for Arc<T> {
    type Inner = T;
    fn into_strong(self) -> Strong<T::FFIType> {
        unsafe { transmute(self) }
    }
    fn as_borrowed(&self) -> Borrowed<T::FFIType> {
        let borrowedptr = self as *const Arc<T> as *const Borrowed<T::FFIType>;
        unsafe { ptr::read(borrowedptr) }
    }
}

#[repr(C)]
/// Gecko-FFI-safe owned pointer
/// Cannot be null
pub struct Owned<T> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

impl<T> Owned<T> {
    pub fn into_box<U>(self) -> Box<T> where U: HasBoxFFI<FFIType = T> {
        unsafe { transmute(self) }
    }
}

impl<T> Deref for Owned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}
