/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

///! Miscellaneous Code which depends on large libraries that we don't
///  depend on in GeckoLib builds.
use js::conversions::{FromJSValConvertible, ToJSValConvertible, latin1_to_string};
use js::jsapi::{JSContext, JSString, HandleValue, MutableHandleValue};
use js::jsapi::{JS_GetTwoByteStringCharsAndLength, JS_StringHasLatin1Chars};
use js::rust::ToString;
use opts;
use std::char;
use std::ptr;
use std::slice;
use str::DOMString;

/// Behavior for stringification of `JSVal`s.
#[derive(PartialEq, Clone)]
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
    unsafe fn from_jsval(cx: *mut JSContext,
                         value: HandleValue,
                         null_behavior: StringificationBehavior)
                         -> Result<DOMString, ()> {
        if null_behavior == StringificationBehavior::Empty &&
           value.get().is_null() {
            Ok(DOMString::new())
        } else {
            let jsstr = ToString(cx, value);
            if jsstr.is_null() {
                debug!("ToString failed");
                Err(())
            } else {
                Ok(jsstring_to_str(cx, jsstr))
            }
        }
    }
}

/// Convert the given `JSString` to a `DOMString`. Fails if the string does not
/// contain valid UTF-16.
pub unsafe fn jsstring_to_str(cx: *mut JSContext, s: *mut JSString) -> DOMString {
    let latin1 = JS_StringHasLatin1Chars(s);
    DOMString::from_string(if latin1 {
        latin1_to_string(cx, s)
    } else {
        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), s, &mut length);
        assert!(!chars.is_null());
        let potentially_ill_formed_utf16 = slice::from_raw_parts(chars, length as usize);
        let mut s = String::with_capacity(length as usize);
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
                        }
                    }
                    if opts::get().replace_surrogates {
                        error!(message!());
                        s.push('\u{FFFD}');
                    } else {
                        panic!(concat!(message!(), " Use `-Z replace-surrogates` \
                            on the command line to make this non-fatal."));
                    }
                }
            }
        }
        s
    })
}
