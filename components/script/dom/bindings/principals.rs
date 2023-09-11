/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;

use js::glue::{
    CreateRustJSPrincipals, DestroyRustJSPrincipals, GetRustJSPrincipalsPrivate,
    JSPrincipalsCallbacks,
};
use js::jsapi::{JSPrincipals, JS_DropPrincipals, JS_HoldPrincipals};
use js::rust::Runtime;
use servo_url::MutableOrigin;

/// An owned reference to Servo's `JSPrincipals` instance.
#[repr(transparent)]
pub struct ServoJSPrincipals(NonNull<JSPrincipals>);

impl ServoJSPrincipals {
    pub fn new(origin: &MutableOrigin) -> Self {
        unsafe {
            let private: Box<MutableOrigin> = Box::new(origin.clone());
            let raw = CreateRustJSPrincipals(&PRINCIPALS_CALLBACKS, Box::into_raw(private) as _);
            // The created `JSPrincipals` object has an initial reference
            // count of zero, so the following code will set it to one
            Self::from_raw_nonnull(NonNull::new_unchecked(raw))
        }
    }

    /// Construct `Self` from a raw `*mut JSPrincipals`, incrementing its
    /// reference count.
    #[inline]
    pub unsafe fn from_raw_nonnull(raw: NonNull<JSPrincipals>) -> Self {
        JS_HoldPrincipals(raw.as_ptr());
        Self(raw)
    }

    #[inline]
    pub unsafe fn origin(&self) -> MutableOrigin {
        let origin = GetRustJSPrincipalsPrivate(self.0.as_ptr()) as *mut MutableOrigin;
        (*origin).clone()
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
        unsafe { JS_DropPrincipals(Runtime::get(), self.as_raw()) };
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
        Self(ManuallyDrop::new(ServoJSPrincipals(self.0 .0)), PhantomData)
    }
}

impl Deref for ServoJSPrincipalsRef<'_> {
    type Target = ServoJSPrincipals;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unused)]
pub unsafe extern "C" fn destroy_servo_jsprincipal(principals: *mut JSPrincipals) {
    Box::from_raw(GetRustJSPrincipalsPrivate(principals) as *mut MutableOrigin);
    DestroyRustJSPrincipals(principals);
}

const PRINCIPALS_CALLBACKS: JSPrincipalsCallbacks = JSPrincipalsCallbacks {
    write: None,
    isSystemOrAddonPrincipal: Some(principals_is_system_or_addon_principal),
};

unsafe extern "C" fn principals_is_system_or_addon_principal(_: *mut JSPrincipals) -> bool {
    false
}

//TODO is same_origin_domain equivalent to subsumes for our purposes
pub unsafe extern "C" fn subsumes(obj: *mut JSPrincipals, other: *mut JSPrincipals) -> bool {
    if let (Some(obj), Some(other)) = (NonNull::new(obj), NonNull::new(other)) {
        let obj = ServoJSPrincipalsRef::from_raw_nonnull(obj);
        let other = ServoJSPrincipalsRef::from_raw_nonnull(other);
        let obj_origin = obj.origin();
        let other_origin = other.origin();
        obj_origin.same_origin_domain(&other_origin)
    } else {
        warn!("Received null JSPrincipals asrgument.");
        false
    }
}
