/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A thin atomically-reference-counted slice.

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use servo_arc::ThinArc;
use std::ops::Deref;
use std::ptr::NonNull;
use std::{iter, mem};

/// A canary that we stash in ArcSlices.
///
/// Given we cannot use a zero-sized-type for the header, since well, C++
/// doesn't have zsts, and we want to use cbindgen for this type, we may as well
/// assert some sanity at runtime.
///
/// We use an u64, to guarantee that we can use a single singleton for every
/// empty slice, even if the types they hold are aligned differently.
const ARC_SLICE_CANARY: u64 = 0xf3f3f3f3f3f3f3f3;

/// A wrapper type for a refcounted slice using ThinArc.
///
/// cbindgen:derive-eq=false
/// cbindgen:derive-neq=false
#[repr(C)]
#[derive(Debug, Eq, PartialEq, ToShmem)]
pub struct ArcSlice<T>(#[shmem(field_bound)] ThinArc<u64, T>);

impl<T> Deref for ArcSlice<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        debug_assert_eq!(self.0.header.header, ARC_SLICE_CANARY);
        &self.0.slice
    }
}

impl<T> Clone for ArcSlice<T> {
    fn clone(&self) -> Self {
        ArcSlice(self.0.clone())
    }
}

lazy_static! {
    // ThinArc doesn't support alignments greater than align_of::<u64>.
    static ref EMPTY_ARC_SLICE: ArcSlice<u64> = {
        ArcSlice::from_iter_leaked(iter::empty())
    };
}

impl<T> Default for ArcSlice<T> {
    #[allow(unsafe_code)]
    fn default() -> Self {
        debug_assert!(
            mem::align_of::<T>() <= mem::align_of::<u64>(),
            "Need to increase the alignment of EMPTY_ARC_SLICE"
        );
        unsafe {
            let empty: ArcSlice<_> = EMPTY_ARC_SLICE.clone();
            let empty: Self = mem::transmute(empty);
            debug_assert_eq!(empty.len(), 0);
            empty
        }
    }
}

impl<T: Serialize> Serialize for ArcSlice<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.deref().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for ArcSlice<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r = Vec::deserialize(deserializer)?;
        Ok(ArcSlice::from_iter(r.into_iter()))
    }
}

impl<T> ArcSlice<T> {
    /// Creates an Arc for a slice using the given iterator to generate the
    /// slice.
    #[inline]
    pub fn from_iter<I>(items: I) -> Self
    where
        I: Iterator<Item = T> + ExactSizeIterator,
    {
        if items.len() == 0 {
            return Self::default();
        }
        ArcSlice(ThinArc::from_header_and_iter(ARC_SLICE_CANARY, items))
    }

    /// Creates an Arc for a slice using the given iterator to generate the
    /// slice, and marks the arc as intentionally leaked from the refcount
    /// logging point of view.
    #[inline]
    pub fn from_iter_leaked<I>(items: I) -> Self
    where
        I: Iterator<Item = T> + ExactSizeIterator,
    {
        let thin_arc = ThinArc::from_header_and_iter(ARC_SLICE_CANARY, items);
        thin_arc.with_arc(|a| a.mark_as_intentionally_leaked());
        ArcSlice(thin_arc)
    }

    /// Creates a value that can be passed via FFI, and forgets this value
    /// altogether.
    #[inline]
    #[allow(unsafe_code)]
    pub fn forget(self) -> ForgottenArcSlicePtr<T> {
        let ret = unsafe {
            ForgottenArcSlicePtr(NonNull::new_unchecked(self.0.ptr() as *const _ as *mut _))
        };
        mem::forget(self);
        ret
    }

    /// Leaks an empty arc slice pointer, and returns it. Only to be used to
    /// construct ArcSlices from FFI.
    #[inline]
    pub fn leaked_empty_ptr() -> *mut std::os::raw::c_void {
        let empty: ArcSlice<_> = EMPTY_ARC_SLICE.clone();
        let ptr = empty.0.ptr();
        std::mem::forget(empty);
        ptr as *mut _
    }
}

/// The inner pointer of an ArcSlice<T>, to be sent via FFI.
/// The type of the pointer is a bit of a lie, we just want to preserve the type
/// but these pointers cannot be constructed outside of this crate, so we're
/// good.
#[repr(C)]
pub struct ForgottenArcSlicePtr<T>(NonNull<T>);
