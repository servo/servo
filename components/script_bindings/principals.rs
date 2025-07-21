/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;

use js::glue::{CreateRustJSPrincipals, GetRustJSPrincipalsPrivate};
use js::jsapi::{JS_DropPrincipals, JS_HoldPrincipals, JSPrincipals};
use js::rust::Runtime;
use servo_url::MutableOrigin;

use crate::DomTypes;
use crate::interfaces::DomHelpers;

/// An owned reference to Servo's `JSPrincipals` instance.
#[repr(transparent)]
pub struct ServoJSPrincipals(NonNull<JSPrincipals>);

#[derive(Debug)]
pub struct PrincipalInfo {
    origin: MutableOrigin,
    is_system_or_addon_principal: bool,
}

impl ServoJSPrincipals {
    pub fn new<D: DomTypes>(origin: &MutableOrigin, is_system_or_addon_principal: bool) -> Self {
        unsafe {
            let private: Box<PrincipalInfo> = Box::new(PrincipalInfo {
                origin: origin.clone(),
                is_system_or_addon_principal,
            });
            let raw = CreateRustJSPrincipals(
                <D as DomHelpers<D>>::principals_callbacks(is_system_or_addon_principal),
                Box::into_raw(private) as _,
            );
            // The created `JSPrincipals` object has an initial reference
            // count of zero, so the following code will set it to one
            Self::from_raw_nonnull(NonNull::new_unchecked(raw))
        }
    }

    /// Construct `Self` from a raw `*mut JSPrincipals`, incrementing its
    /// reference count.
    ///
    /// # Safety
    /// `raw` must point to a valid JSPrincipals value.
    #[inline]
    pub unsafe fn from_raw_nonnull(raw: NonNull<JSPrincipals>) -> Self {
        JS_HoldPrincipals(raw.as_ptr());
        Self(raw)
    }

    #[inline]
    pub fn origin(&self) -> MutableOrigin {
        unsafe {
            let info = GetRustJSPrincipalsPrivate(self.0.as_ptr()) as *mut PrincipalInfo;
            (*info).origin.clone()
        }
    }

    #[inline]
    pub fn is_system_or_addon_principal(&self) -> bool {
        unsafe {
            let info = GetRustJSPrincipalsPrivate(self.0.as_ptr()) as *mut PrincipalInfo;
            (*info).is_system_or_addon_principal
        }
    }

    #[inline]
    pub fn as_raw_nonnull(&self) -> NonNull<JSPrincipals> {
        self.0
    }

    #[inline]
    pub fn as_raw(&self) -> *mut JSPrincipals {
        self.0.as_ptr()
    }
}

impl Clone for ServoJSPrincipals {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::from_raw_nonnull(self.as_raw_nonnull()) }
    }
}

impl Drop for ServoJSPrincipals {
    #[inline]
    fn drop(&mut self) {
        if let Some(cx) = Runtime::get() {
            unsafe { JS_DropPrincipals(cx.as_ptr(), self.as_raw()) };
        }
    }
}

/// A borrowed reference to Servo's `JSPrincipals` instance. Does not update the
/// reference count on creation and deletion.
pub struct ServoJSPrincipalsRef<'a>(ManuallyDrop<ServoJSPrincipals>, PhantomData<&'a ()>);

impl ServoJSPrincipalsRef<'_> {
    /// Construct `Self` from a raw `NonNull<JSPrincipals>`.
    ///
    /// # Safety
    ///
    /// `ServoJSPrincipalsRef` does not update the reference count of the
    /// wrapped `JSPrincipals` object. It's up to the caller to ensure the
    /// returned `ServoJSPrincipalsRef` object or any clones are not used past
    /// the lifetime of the wrapped object.
    #[inline]
    pub unsafe fn from_raw_nonnull(raw: NonNull<JSPrincipals>) -> Self {
        // Don't use `ServoJSPrincipals::from_raw_nonnull`; we don't want to
        // update the reference count
        Self(ManuallyDrop::new(ServoJSPrincipals(raw)), PhantomData)
    }

    /// Construct `Self` from a raw `*mut JSPrincipals`.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if `raw` is null. See also
    /// [`Self::from_raw_nonnull`].
    #[inline]
    pub unsafe fn from_raw_unchecked(raw: *mut JSPrincipals) -> Self {
        Self::from_raw_nonnull(NonNull::new_unchecked(raw))
    }
}

impl Clone for ServoJSPrincipalsRef<'_> {
    #[inline]
    fn clone(&self) -> Self {
        Self(ManuallyDrop::new(ServoJSPrincipals(self.0.0)), PhantomData)
    }
}

impl Deref for ServoJSPrincipalsRef<'_> {
    type Target = ServoJSPrincipals;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
