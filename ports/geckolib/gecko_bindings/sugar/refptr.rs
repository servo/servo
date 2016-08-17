/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::ptr;
use std::sync::Arc;

/// Helper trait for conversions between FFI Strong/Borrowed types and Arcs
///
/// Should be implemented by types which are passed over FFI as Arcs
/// via Strong and Borrowed
pub unsafe trait HasArcFFI where Self: Sized {
    /// Gecko's name for the type
    /// This is equivalent to ArcInner<Self>
    type FFIType: Sized;

    /// Given a non-null borrowed FFI reference, this produces a temporary
    /// Arc which is borrowed by the given closure and used.
    /// Panics on null.
    fn with<F, Output>(raw: Borrowed<Self::FFIType>, cb: F) -> Output
               where F: FnOnce(&Arc<Self>) -> Output {
        Self::maybe_with(raw, |opt| cb(opt.unwrap()))
    }

    /// Given a maybe-null borrowed FFI reference, this produces a temporary
    /// Option<Arc> (None if null) which is borrowed by the given closure and used
    fn maybe_with<F, Output>(maybe_raw: Borrowed<Self::FFIType>, cb: F) -> Output
                         where F: FnOnce(Option<&Arc<Self>>) -> Output {
        cb(Self::borrowed_as(&maybe_raw))
    }

    /// Given a non-null strong FFI reference, converts it into an Arc.
    /// Panics on null.
    fn into(ptr: Strong<Self::FFIType>) -> Arc<Self> {
        assert!(!ptr.is_null());
        unsafe { transmute(ptr) }
    }

    fn borrowed_as<'a>(ptr: &'a Borrowed<'a, Self::FFIType>) -> Option<&'a Arc<Self>> {
        unsafe {
            if ptr.is_null() {
                None
            } else {
                Some(transmute::<&Borrowed<_>, &Arc<_>>(ptr))
            }
        }
    }

    /// Converts an Arc into a strong FFI reference.
    fn from_arc(owned: Arc<Self>) -> Strong<Self::FFIType> {
        unsafe { transmute(owned) }
    }

    /// Artificially increments the refcount of a borrowed Arc over FFI.
    unsafe fn addref(ptr: Borrowed<Self::FFIType>) {
        Self::with(ptr, |arc| forget(arc.clone()));
    }

    /// Given a (possibly null) borrowed FFI reference, decrements the refcount.
    /// Unsafe since it doesn't consume the backing Arc. Run it only when you
    /// know that a strong reference to the backing Arc is disappearing
    /// (usually on the C++ side) without running the Arc destructor.
    unsafe fn release(ptr: Borrowed<Self::FFIType>) {
        if let Some(arc) = Self::borrowed_as(&ptr) {
            let _: Arc<_> = ptr::read(arc as *const Arc<_>);
        }
    }

    /// Produces a borrowed FFI reference by borrowing an Arc.
    fn to_borrowed<'a>(arc: &'a Arc<Self>)
        -> Borrowed<'a, Self::FFIType> {
        let borrowedptr = arc as *const Arc<Self> as *const Borrowed<'a, Self::FFIType>;
        unsafe { ptr::read(borrowedptr) }
    }

    /// Produces a null strong FFI reference
    fn null_strong() -> Strong<Self::FFIType> {
        unsafe { transmute(ptr::null::<Self::FFIType>()) }
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
}
