use js::{
    glue::{
        CreateRustJSPrincipals, DestroyRustJSPrincipals, GetRustJSPrincipalsPrivate,
        JSPrincipalsCallbacks,
    },
    jsapi::{JSPrincipals, JS_DropPrincipals, JS_HoldPrincipals},
    rust::Runtime,
};
use servo_url::MutableOrigin;
use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

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

/// A borrowed reference to Servo's `JSPrincipals` instance.
#[derive(Clone, Copy)]
pub struct ServoJSPrincipalsRef<'a>(NonNull<JSPrincipals>, PhantomData<&'a ()>);

impl ServoJSPrincipalsRef<'_> {
    /// Construct `Self` from a raw `NonNull<JSPrincipals>`.
    #[inline]
    pub unsafe fn from_raw_nonnull(raw: NonNull<JSPrincipals>) -> Self {
        Self(raw, PhantomData)
    }

    /// Construct `Self` from a raw `*mut JSPrincipals`.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if `raw` is null.
    #[inline]
    pub unsafe fn from_raw_unchecked(raw: *mut JSPrincipals) -> Self {
        Self::from_raw_nonnull(NonNull::new_unchecked(raw))
    }
}

impl Deref for ServoJSPrincipalsRef<'_> {
    type Target = ServoJSPrincipals;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.0 as *const NonNull<JSPrincipals> as *const ServoJSPrincipals) }
    }
}

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
    let obj = ServoJSPrincipalsRef::from_raw_unchecked(obj);
    let other = ServoJSPrincipalsRef::from_raw_unchecked(other);
    let obj_origin = obj.origin();
    let other_origin = other.origin();
    obj_origin.same_origin_domain(&other_origin)
}
