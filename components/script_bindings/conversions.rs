/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{ptr, slice};

use js::conversions::{
    latin1_to_string, ConversionResult, FromJSValConvertible, ToJSValConvertible,
};
use js::error::throw_type_error;
use js::glue::{
    GetProxyHandlerExtra, GetProxyReservedSlot, IsProxyHandlerFamily, IsWrapper,
    JS_GetReservedSlot, UnwrapObjectDynamic,
};
use js::jsapi::{
    JSContext, JSObject, JSString, JS_DeprecatedStringHasLatin1Chars,
    JS_GetLatin1StringCharsAndLength, JS_GetTwoByteStringCharsAndLength, JS_NewStringCopyN,
};
use js::jsval::{ObjectValue, StringValue, UndefinedValue};
use js::rust::{
    get_object_class, is_dom_class, is_dom_object, maybe_wrap_value, HandleValue,
    MutableHandleValue, ToString,
};

use crate::inheritance::Castable;
use crate::reflector::{DomObject, Reflector};
use crate::root::DomRoot;
use crate::str::{ByteString, DOMString, USVString};
use crate::utils::{DOMClass, DOMJSClass};

/// A trait to check whether a given `JSObject` implements an IDL interface.
pub trait IDLInterface {
    /// Returns whether the given DOM class derives that interface.
    fn derives(_: &'static DOMClass) -> bool;
}

/// A trait to mark an IDL interface as deriving from another one.
pub trait DerivedFrom<T: Castable>: Castable {}

// http://heycam.github.io/webidl/#es-USVString
impl ToJSValConvertible for USVString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.0.to_jsval(cx, rval);
    }
}

/// Behavior for stringification of `JSVal`s.
#[derive(Clone, PartialEq)]
pub enum StringificationBehavior {
    /// Convert `null` to the string `"null"`.
    Default,
    /// Convert `null` to the empty string.
    Empty,
}

// https://heycam.github.io/webidl/#es-DOMString
impl ToJSValConvertible for DOMString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        (**self).to_jsval(cx, rval);
    }
}

// https://heycam.github.io/webidl/#es-DOMString
impl FromJSValConvertible for DOMString {
    type Config = StringificationBehavior;
    unsafe fn from_jsval(
        cx: *mut JSContext,
        value: HandleValue,
        null_behavior: StringificationBehavior,
    ) -> Result<ConversionResult<DOMString>, ()> {
        if null_behavior == StringificationBehavior::Empty && value.get().is_null() {
            Ok(ConversionResult::Success(DOMString::new()))
        } else {
            match ptr::NonNull::new(ToString(cx, value)) {
                Some(jsstr) => Ok(ConversionResult::Success(jsstring_to_str(cx, jsstr))),
                None => {
                    debug!("ToString failed");
                    Err(())
                },
            }
        }
    }
}

/// Convert the given `JSString` to a `DOMString`. Fails if the string does not
/// contain valid UTF-16.
///
/// # Safety
/// cx and s must point to valid values.
pub unsafe fn jsstring_to_str(cx: *mut JSContext, s: ptr::NonNull<JSString>) -> DOMString {
    let latin1 = JS_DeprecatedStringHasLatin1Chars(s.as_ptr());
    DOMString::from_string(if latin1 {
        latin1_to_string(cx, s.as_ptr())
    } else {
        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), s.as_ptr(), &mut length);
        assert!(!chars.is_null());
        let potentially_ill_formed_utf16 = slice::from_raw_parts(chars, length);
        let mut s = String::with_capacity(length);
        for item in char::decode_utf16(potentially_ill_formed_utf16.iter().cloned()) {
            match item {
                Ok(c) => s.push(c),
                Err(_) => {
                    error!("Found an unpaired surrogate in a DOM string.");
                    s.push('\u{FFFD}');
                },
            }
        }
        s
    })
}

// http://heycam.github.io/webidl/#es-USVString
impl FromJSValConvertible for USVString {
    type Config = ();
    unsafe fn from_jsval(
        cx: *mut JSContext,
        value: HandleValue,
        _: (),
    ) -> Result<ConversionResult<USVString>, ()> {
        let Some(jsstr) = ptr::NonNull::new(ToString(cx, value)) else {
            debug!("ToString failed");
            return Err(());
        };
        let latin1 = JS_DeprecatedStringHasLatin1Chars(jsstr.as_ptr());
        if latin1 {
            // FIXME(ajeffrey): Convert directly from DOMString to USVString
            return Ok(ConversionResult::Success(USVString(String::from(
                jsstring_to_str(cx, jsstr),
            ))));
        }
        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), jsstr.as_ptr(), &mut length);
        assert!(!chars.is_null());
        let char_vec = slice::from_raw_parts(chars, length);
        Ok(ConversionResult::Success(USVString(
            String::from_utf16_lossy(char_vec),
        )))
    }
}

// http://heycam.github.io/webidl/#es-ByteString
impl ToJSValConvertible for ByteString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        let jsstr = JS_NewStringCopyN(
            cx,
            self.as_ptr() as *const libc::c_char,
            self.len() as libc::size_t,
        );
        if jsstr.is_null() {
            panic!("JS_NewStringCopyN failed");
        }
        rval.set(StringValue(&*jsstr));
    }
}

// http://heycam.github.io/webidl/#es-ByteString
impl FromJSValConvertible for ByteString {
    type Config = ();
    unsafe fn from_jsval(
        cx: *mut JSContext,
        value: HandleValue,
        _option: (),
    ) -> Result<ConversionResult<ByteString>, ()> {
        let string = ToString(cx, value);
        if string.is_null() {
            debug!("ToString failed");
            return Err(());
        }

        let latin1 = JS_DeprecatedStringHasLatin1Chars(string);
        if latin1 {
            let mut length = 0;
            let chars = JS_GetLatin1StringCharsAndLength(cx, ptr::null(), string, &mut length);
            assert!(!chars.is_null());

            let char_slice = slice::from_raw_parts(chars as *mut u8, length);
            return Ok(ConversionResult::Success(ByteString::new(
                char_slice.to_vec(),
            )));
        }

        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), string, &mut length);
        let char_vec = slice::from_raw_parts(chars, length);

        if char_vec.iter().any(|&c| c > 0xFF) {
            throw_type_error(cx, "Invalid ByteString");
            Err(())
        } else {
            Ok(ConversionResult::Success(ByteString::new(
                char_vec.iter().map(|&c| c as u8).collect(),
            )))
        }
    }
}

impl ToJSValConvertible for Reflector {
    unsafe fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        let obj = self.get_jsobject().get();
        assert!(!obj.is_null());
        rval.set(ObjectValue(obj));
        maybe_wrap_value(cx, rval);
    }
}

impl<T: DomObject + IDLInterface> FromJSValConvertible for DomRoot<T> {
    type Config = ();

    unsafe fn from_jsval(
        cx: *mut JSContext,
        value: HandleValue,
        _config: Self::Config,
    ) -> Result<ConversionResult<DomRoot<T>>, ()> {
        Ok(match root_from_handlevalue(value, cx) {
            Ok(result) => ConversionResult::Success(result),
            Err(()) => ConversionResult::Failure("value is not an object".into()),
        })
    }
}

impl<T: DomObject> ToJSValConvertible for DomRoot<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.reflector().to_jsval(cx, rval);
    }
}

/// Get the `DOMClass` from `obj`, or `Err(())` if `obj` is not a DOM object.
///
/// # Safety
/// obj must point to a valid, non-null JS object.
#[allow(clippy::result_unit_err)]
pub unsafe fn get_dom_class(obj: *mut JSObject) -> Result<&'static DOMClass, ()> {
    let clasp = get_object_class(obj);
    if is_dom_class(&*clasp) {
        trace!("plain old dom object");
        let domjsclass: *const DOMJSClass = clasp as *const DOMJSClass;
        return Ok(&(*domjsclass).dom_class);
    }
    if is_dom_proxy(obj) {
        trace!("proxy dom object");
        let dom_class: *const DOMClass = GetProxyHandlerExtra(obj) as *const DOMClass;
        return Ok(&*dom_class);
    }
    trace!("not a dom object");
    Err(())
}

/// Returns whether `obj` is a DOM object implemented as a proxy.
///
/// # Safety
/// obj must point to a valid, non-null JS object.
pub unsafe fn is_dom_proxy(obj: *mut JSObject) -> bool {
    unsafe {
        let clasp = get_object_class(obj);
        ((*clasp).flags & js::JSCLASS_IS_PROXY) != 0 && IsProxyHandlerFamily(obj)
    }
}

/// The index of the slot wherein a pointer to the reflected DOM object is
/// stored for non-proxy bindings.
// We use slot 0 for holding the raw object.  This is safe for both
// globals and non-globals.
pub const DOM_OBJECT_SLOT: u32 = 0;

/// Get the private pointer of a DOM object from a given reflector.
///
/// # Safety
/// obj must point to a valid non-null JS object.
pub unsafe fn private_from_object(obj: *mut JSObject) -> *const libc::c_void {
    let mut value = UndefinedValue();
    if is_dom_object(obj) {
        JS_GetReservedSlot(obj, DOM_OBJECT_SLOT, &mut value);
    } else {
        debug_assert!(is_dom_proxy(obj));
        GetProxyReservedSlot(obj, 0, &mut value);
    };
    if value.is_undefined() {
        ptr::null()
    } else {
        value.to_private()
    }
}

pub enum PrototypeCheck {
    Derive(fn(&'static DOMClass) -> bool),
    Depth { depth: usize, proto_id: u16 },
}

/// Get a `*const libc::c_void` for the given DOM object, unwrapping any
/// wrapper around it first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not an object for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
///
/// # Safety
/// obj must point to a valid, non-null JS object.
/// cx must point to a valid, non-null JS context.
#[inline]
#[allow(clippy::result_unit_err)]
pub unsafe fn private_from_proto_check(
    mut obj: *mut JSObject,
    cx: *mut JSContext,
    proto_check: PrototypeCheck,
) -> Result<*const libc::c_void, ()> {
    let dom_class = get_dom_class(obj).or_else(|_| {
        if IsWrapper(obj) {
            trace!("found wrapper");
            obj = UnwrapObjectDynamic(obj, cx, /* stopAtWindowProxy = */ false);
            if obj.is_null() {
                trace!("unwrapping security wrapper failed");
                Err(())
            } else {
                assert!(!IsWrapper(obj));
                trace!("unwrapped successfully");
                get_dom_class(obj)
            }
        } else {
            trace!("not a dom wrapper");
            Err(())
        }
    })?;

    let prototype_matches = match proto_check {
        PrototypeCheck::Derive(f) => (f)(dom_class),
        PrototypeCheck::Depth { depth, proto_id } => {
            dom_class.interface_chain[depth] as u16 == proto_id
        },
    };

    if prototype_matches {
        trace!("good prototype");
        Ok(private_from_object(obj))
    } else {
        trace!("bad prototype");
        Err(())
    }
}

/// Get a `*const T` for a DOM object accessible from a `JSObject`.
///
/// # Safety
/// obj must point to a valid, non-null JS object.
/// cx must point to a valid, non-null JS context.
#[allow(clippy::result_unit_err)]
pub unsafe fn native_from_object<T>(obj: *mut JSObject, cx: *mut JSContext) -> Result<*const T, ()>
where
    T: DomObject + IDLInterface,
{
    unsafe {
        private_from_proto_check(obj, cx, PrototypeCheck::Derive(T::derives))
            .map(|ptr| ptr as *const T)
    }
}

/// Get a `DomRoot<T>` for the given DOM object, unwrapping any wrapper
/// around it first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not a reflector for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
///
/// # Safety
/// obj must point to a valid, non-null JS object.
/// cx must point to a valid, non-null JS context.
#[allow(clippy::result_unit_err)]
pub unsafe fn root_from_object<T>(obj: *mut JSObject, cx: *mut JSContext) -> Result<DomRoot<T>, ()>
where
    T: DomObject + IDLInterface,
{
    native_from_object(obj, cx).map(|ptr| unsafe { DomRoot::from_ref(&*ptr) })
}

/// Get a `DomRoot<T>` for a DOM object accessible from a `HandleValue`.
/// Caller is responsible for throwing a JS exception if needed in case of error.
///
/// # Safety
/// cx must point to a valid, non-null JS context.
#[allow(clippy::result_unit_err)]
pub unsafe fn root_from_handlevalue<T>(v: HandleValue, cx: *mut JSContext) -> Result<DomRoot<T>, ()>
where
    T: DomObject + IDLInterface,
{
    if !v.get().is_object() {
        return Err(());
    }
    root_from_object(v.get().to_object(), cx)
}
