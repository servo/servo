/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Helpers for different FFI pointer kinds that Gecko's FFI layer uses.

use servo_arc::{Arc, RawOffsetArc};
use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr;
use gecko_bindings::structs::root::mozilla::detail::CopyablePtr;

/// Indicates that a given Servo type has a corresponding Gecko FFI type.
pub unsafe trait HasFFI: Sized + 'static {
    /// The corresponding Gecko type that this rust type represents.
    ///
    /// See the examples in `components/style/gecko/conversions.rs`.
    type FFIType: Sized;
}

/// Indicates that a given Servo type has the same layout as the corresponding
/// `HasFFI::FFIType` type.
pub unsafe trait HasSimpleFFI: HasFFI {
    #[inline]
    /// Given a Servo-side reference, converts it to an FFI-safe reference which
    /// can be passed to Gecko.
    ///
    /// &ServoType -> &GeckoType
    fn as_ffi(&self) -> &Self::FFIType {
        unsafe { transmute(self) }
    }
    #[inline]
    /// Given a Servo-side mutable reference, converts it to an FFI-safe mutable
    /// reference which can be passed to Gecko.
    ///
    /// &mut ServoType -> &mut GeckoType
    fn as_ffi_mut(&mut self) -> &mut Self::FFIType {
        unsafe { transmute(self) }
    }
    #[inline]
    /// Given an FFI-safe reference obtained from Gecko converts it to a
    /// Servo-side reference.
    ///
    /// &GeckoType -> &ServoType
    fn from_ffi(ffi: &Self::FFIType) -> &Self {
        unsafe { transmute(ffi) }
    }
    #[inline]
    /// Given an FFI-safe mutable reference obtained from Gecko converts it to a
    /// Servo-side mutable reference.
    ///
    /// &mut GeckoType -> &mut ServoType
    fn from_ffi_mut(ffi: &mut Self::FFIType) -> &mut Self {
        unsafe { transmute(ffi) }
    }
}

/// Indicates that the given Servo type is passed over FFI
/// as a Box
pub unsafe trait HasBoxFFI: HasSimpleFFI {
    #[inline]
    /// Converts a borrowed Arc to a borrowed FFI reference.
    ///
    /// &Arc<ServoType> -> &GeckoType
    fn into_ffi(self: Box<Self>) -> Owned<Self::FFIType> {
        unsafe { transmute(self) }
    }

    /// Drops an owned FFI pointer. This conceptually takes the
    /// Owned<Self::FFIType>, except it's a bit of a paint to do that without
    /// much benefit.
    #[inline]
    unsafe fn drop_ffi(ptr: *mut Self::FFIType) {
        let _ = Box::from_raw(ptr as *mut Self);
    }
}

/// Helper trait for conversions between FFI Strong/Borrowed types and Arcs
///
/// Should be implemented by types which are passed over FFI as Arcs via Strong
/// and Borrowed.
///
/// In this case, the FFIType is the rough equivalent of ArcInner<Self>.
pub unsafe trait HasArcFFI: HasFFI {
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
pub unsafe trait FFIArcHelpers {
    /// The Rust FFI type that we're implementing methods for.
    type Inner: HasArcFFI;

    /// Converts an Arc into a strong FFI reference.
    ///
    /// Arc<ServoType> -> Strong<GeckoType>
    fn into_strong(self) -> Strong<<Self::Inner as HasFFI>::FFIType>;

    /// Produces a borrowed FFI reference by borrowing an Arc.
    ///
    /// &Arc<ServoType> -> &GeckoType
    ///
    /// Then the `arc_as_borrowed` method can go away.
    fn as_borrowed(&self) -> &<Self::Inner as HasFFI>::FFIType;
}

unsafe impl<T: HasArcFFI> FFIArcHelpers for RawOffsetArc<T> {
    type Inner = T;

    #[inline]
    fn into_strong(self) -> Strong<T::FFIType> {
        unsafe { transmute(self) }
    }

    #[inline]
    fn as_borrowed(&self) -> &T::FFIType {
        unsafe { &*(&**self as *const T as *const T::FFIType) }
    }
}

unsafe impl<T: HasArcFFI> FFIArcHelpers for Arc<T> {
    type Inner = T;

    #[inline]
    fn into_strong(self) -> Strong<T::FFIType> {
        Arc::into_raw_offset(self).into_strong()
    }

    #[inline]
    fn as_borrowed(&self) -> &T::FFIType {
        unsafe { &*(&**self as *const T as *const T::FFIType) }
    }
}

#[repr(C)]
#[derive(Debug)]
/// Gecko-FFI-safe owned pointer.
///
/// Cannot be null, and leaks on drop, so needs to be converted into a rust-side
/// `Box` before.
pub struct Owned<GeckoType> {
    ptr: *mut GeckoType,
    _marker: PhantomData<GeckoType>,
}

impl<GeckoType> Owned<GeckoType> {
    /// Converts this instance to a (non-null) instance of `OwnedOrNull`.
    pub fn maybe(self) -> OwnedOrNull<GeckoType> {
        unsafe { transmute(self) }
    }
}

impl<GeckoType> Deref for Owned<GeckoType> {
    type Target = GeckoType;
    fn deref(&self) -> &GeckoType {
        unsafe { &*self.ptr }
    }
}

impl<GeckoType> DerefMut for Owned<GeckoType> {
    fn deref_mut(&mut self) -> &mut GeckoType {
        unsafe { &mut *self.ptr }
    }
}

#[repr(C)]
/// Gecko-FFI-safe owned pointer.
///
/// Can be null, and just as `Owned` leaks on `Drop`.
pub struct OwnedOrNull<GeckoType> {
    ptr: *mut GeckoType,
    _marker: PhantomData<GeckoType>,
}

impl<GeckoType> OwnedOrNull<GeckoType> {
    /// Returns a null pointer.
    #[inline]
    pub fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
            _marker: PhantomData,
        }
    }

    /// Returns whether this pointer is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Gets a immutable reference to the underlying Gecko type, or `None` if
    /// null.
    pub fn borrow(&self) -> Option<&GeckoType> {
        unsafe { transmute(self) }
    }

    /// Gets a mutable reference to the underlying Gecko type, or `None` if
    /// null.
    pub fn borrow_mut(&self) -> Option<&mut GeckoType> {
        unsafe { transmute(self) }
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