/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{ptr, slice};

use js::conversions::{
    latin1_to_string, ConversionResult, FromJSValConvertible, ToJSValConvertible,
};
use js::error::throw_type_error;
use js::glue::{GetProxyHandlerExtra, IsProxyHandlerFamily};
use js::jsapi::{
    JSContext, JSObject, JSString, JS_DeprecatedStringHasLatin1Chars,
    JS_GetLatin1StringCharsAndLength, JS_GetTwoByteStringCharsAndLength, JS_NewStringCopyN,
};
use js::jsval::{ObjectValue, StringValue};
use js::rust::{
    get_object_class, is_dom_class, maybe_wrap_value, HandleValue, MutableHandleValue, ToString,
};
use servo_config::opts;

use crate::reflector::Reflector;
use crate::str::{ByteString, DOMString, USVString};
use crate::utils::{DOMClass, DOMJSClass};

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
                    // FIXME: Add more info like document URL in the message?
                    macro_rules! message {
                        () => {
                            "Found an unpaired surrogate in a DOM string. \
                             If you see this in real web content, \
                             please comment on https://github.com/servo/servo/issues/6564"
                        };
                    }
                    if opts::get().debug.replace_surrogates {
                        error!(message!());
                        s.push('\u{FFFD}');
                    } else {
                        panic!(concat!(
                            message!(),
                            " Use `-Z replace-surrogates` \
                             on the command line to make this non-fatal."
                        ));
                    }
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
