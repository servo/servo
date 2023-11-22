/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Helpers for different FFI pointer kinds that Gecko's FFI layer uses.

use crate::gecko_bindings::structs::root::mozilla::detail::CopyablePtr;
use servo_arc::{Arc, RawOffsetArc};
use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr;

/// Helper trait for conversions between FFI Strong/Borrowed types and Arcs
///
/// Should be implemented by types which are passed over FFI as Arcs via Strong
/// and Borrowed.
///
/// In this case, the FFIType is the rough equivalent of ArcInner<Self>.
pub unsafe trait HasArcFFI: Sized + 'static {
    /// The corresponding Gecko type that this rust type represents.
    ///
    /// See the examples in `components/style/gecko/conversions.rs`.
    type FFIType: Sized;

    // these methods can't be on Borrowed because it leads to an unspecified
    // impl parameter
    /// Artificially increments the refcount of a (possibly null) borrowed Arc
    /// over FFI.
    unsafe fn addref_opt(ptr: Option<&Self::FFIType>) {
        forget(Self::arc_from_borrowed(&ptr).clone())
    }

    /// Given a (possibly null) borrowed FFI reference, decrements the refcount.
    /// Unsafe since it doesn't consume the backing Arc. Run it only when you
    /// know that a strong reference to the backing Arc is disappearing
    /// (usually on the C++ side) without running the Arc destructor.
    unsafe fn release_opt(ptr: Option<&Self::FFIType>) {
        if let Some(arc) = Self::arc_from_borrowed(&ptr) {
            let _: RawOffsetArc<_> = ptr::read(arc as *const RawOffsetArc<_>);
        }
    }

    /// Artificially increments the refcount of a borrowed Arc over FFI.
    unsafe fn addref(ptr: &Self::FFIType) {
        forget(Self::as_arc(&ptr).clone())
    }

    /// Given a non-null borrowed FFI reference, decrements the refcount.
    /// Unsafe since it doesn't consume the backing Arc. Run it only when you
    /// know that a strong reference to the backing Arc is disappearing
    /// (usually on the C++ side) without running the Arc destructor.
    unsafe fn release(ptr: &Self::FFIType) {
        let _: RawOffsetArc<_> = ptr::read(Self::as_arc(&ptr) as *const RawOffsetArc<_>);
    }
    #[inline]
    /// Converts a borrowed FFI reference to a borrowed Arc.
    ///
    /// &GeckoType -> &Arc<ServoType>
    fn as_arc<'a>(ptr: &'a &Self::FFIType) -> &'a RawOffsetArc<Self> {
        unsafe { transmute::<&&Self::FFIType, &RawOffsetArc<Self>>(ptr) }
    }

    #[inline]
    /// Converts a borrowed Arc to a borrowed FFI reference.
    ///
    /// &Arc<ServoType> -> &GeckoType
    fn arc_as_borrowed<'a>(arc: &'a RawOffsetArc<Self>) -> &'a &Self::FFIType {
        unsafe { transmute::<&RawOffsetArc<Self>, &&Self::FFIType>(arc) }
    }

    #[inline]
    /// Converts a borrowed nullable FFI reference to a borrowed Arc.
    ///
    /// &GeckoType -> &Arc<ServoType>
    fn arc_from_borrowed<'a>(ptr: &'a Option<&Self::FFIType>) -> Option<&'a RawOffsetArc<Self>> {
        unsafe {
            if let Some(ref reference) = *ptr {
                Some(transmute::<&&Self::FFIType, &RawOffsetArc<_>>(reference))
            } else {
                None
            }
        }
    }
}

/// Gecko-FFI-safe Arc (T is an ArcInner).
///
/// This can be null.
///
/// Leaks on drop. Please don't drop this.
#[repr(C)]
pub struct Strong<GeckoType> {
    ptr: *const GeckoType,
    _marker: PhantomData<GeckoType>,
}

impl<GeckoType> Strong<GeckoType> {
    #[inline]
    /// Returns whether this reference is null.
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    #[inline]
    /// Given a non-null strong FFI reference, converts it into a servo-side
    /// Arc.
    ///
    /// Panics on null.
    ///
    /// Strong<GeckoType> -> Arc<ServoType>
    pub fn into_arc<ServoType>(self) -> RawOffsetArc<ServoType>
    where
        ServoType: HasArcFFI<FFIType = GeckoType>,
    {
        self.into_arc_opt().unwrap()
    }

    #[inline]
    /// Given a strong FFI reference,
    /// converts it into a servo-side Arc
    /// Returns None on null.
    ///
    /// Strong<GeckoType> -> Arc<ServoType>
    pub fn into_arc_opt<ServoType>(self) -> Option<RawOffsetArc<ServoType>>
    where
        ServoType: HasArcFFI<FFIType = GeckoType>,
    {
        if self.is_null() {
            None
        } else {
            unsafe { Some(transmute(self)) }
        }
    }

    #[inline]
    /// Given a reference to a strong FFI reference, converts it to a reference
    /// to a servo-side Arc.
    ///
    /// Returns None on null.
    ///
    /// Strong<GeckoType> -> Arc<ServoType>
    pub fn as_arc_opt<ServoType>(&self) -> Option<&RawOffsetArc<ServoType>>
    where
        ServoType: HasArcFFI<FFIType = GeckoType>,
    {
        if self.is_null() {
            None
        } else {
            unsafe { Some(transmute(self)) }
        }
    }

    #[inline]
    /// Produces a null strong FFI reference.
    pub fn null() -> Self {
        unsafe { transmute(ptr::null::<GeckoType>()) }
    }
}

/// A few helpers implemented on top of Arc<ServoType> to make it more
/// comfortable to use and write safe code with.
pub unsafe trait FFIArcHelpers<T: HasArcFFI> {
    /// Converts an Arc into a strong FFI reference.
    ///
    /// Arc<ServoType> -> Strong<GeckoType>
    fn into_strong(self) -> Strong<T::FFIType>;

    /// Produces a borrowed FFI reference by borrowing an Arc.
    ///
    /// &Arc<ServoType> -> &GeckoType
    ///
    /// Then the `arc_as_borrowed` method can go away.
    fn as_borrowed(&self) -> &T::FFIType;
}

unsafe impl<T: HasArcFFI> FFIArcHelpers<T> for RawOffsetArc<T> {
    #[inline]
    fn into_strong(self) -> Strong<T::FFIType> {
        unsafe { transmute(self) }
    }

    #[inline]
    fn as_borrowed(&self) -> &T::FFIType {
        unsafe { &*(&**self as *const T as *const T::FFIType) }
    }
}

unsafe impl<T: HasArcFFI> FFIArcHelpers<T> for Arc<T> {
    #[inline]
    fn into_strong(self) -> Strong<T::FFIType> {
        Arc::into_raw_offset(self).into_strong()
    }

    #[inline]
    fn as_borrowed(&self) -> &T::FFIType {
        unsafe { &*(&**self as *const T as *const T::FFIType) }
    }
}

impl<T> Deref for CopyablePtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.mPtr
    }
}

impl<T> DerefMut for CopyablePtr<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.mPtr
    }
}
