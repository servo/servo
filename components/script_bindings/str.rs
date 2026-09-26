/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `ByteString` struct.
use std::borrow::ToOwned;
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ptr::NonNull;
use std::str::FromStr;
use std::{fmt, ops, slice, str};

use js::context::JSContext;
use js::conversions::latin1_to_string;
use js::gc::{HandleObject, HandleValue};
use js::jsapi::{JS_DeprecatedStringHasLatin1Chars, JSString};
use js::rust::wrappers2::{JS_GetTwoByteStringCharsAndLength, ToJSON};
use js::rust::{
    HandleValue as SafeHandleValue, MutableHandleString as SafeMutableHandleString, ToString,
};

pub use crate::domstring::DOMString;
use crate::error::Error;

/// Encapsulates the IDL `ByteString` type.
#[derive(Clone, Debug, Default, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub struct ByteString(Vec<u8>);

impl ByteString {
    /// Creates a new `ByteString`.
    pub fn new(value: Vec<u8>) -> ByteString {
        ByteString(value)
    }

    /// Returns `self` as a string, if it encodes valid UTF-8, and `None`
    /// otherwise.
    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }

    /// Returns the length.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the ByteString is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `self` with A–Z replaced by a–z.
    pub fn to_lower(&self) -> ByteString {
        ByteString::new(self.0.to_ascii_lowercase())
    }
}

impl From<ByteString> for Vec<u8> {
    fn from(byte_string: ByteString) -> Vec<u8> {
        byte_string.0
    }
}

impl Hash for ByteString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl FromStr for ByteString {
    type Err = ();
    fn from_str(s: &str) -> Result<ByteString, ()> {
        Ok(ByteString::new(s.to_owned().into_bytes()))
    }
}

impl ops::Deref for ByteString {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

/// A string that is constructed from a UCS-2 buffer by replacing invalid code
/// points with the replacement character.
#[derive(Clone, Debug, Default, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct USVString(pub String);

impl USVString {
    /// Creates a new `USVString`.
    pub fn new() -> USVString {
        USVString(String::new())
    }
}

impl Deref for USVString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for USVString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for USVString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl PartialEq<str> for USVString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<&'a str> for USVString {
    fn eq(&self, other: &&'a str) -> bool {
        self.0 == *other
    }
}

impl From<String> for USVString {
    fn from(contents: String) -> USVString {
        USVString(contents)
    }
}

impl From<USVString> for String {
    fn from(value: USVString) -> Self {
        value.0
    }
}

impl From<USVString> for DOMString {
    fn from(value: USVString) -> Self {
        value.0.into()
    }
}

/// Returns whether `s` is a `token`, as defined by
/// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-17).
pub fn is_token(s: &[u8]) -> bool {
    if s.is_empty() {
        return false; // A token must be at least a single character
    }
    s.iter().all(|&x| {
        // http://tools.ietf.org/html/rfc2616#section-2.2
        match x {
            0..=31 | 127 => false, // CTLs
            40 | 41 | 60 | 62 | 64 | 44 | 59 | 58 | 92 | 34 | 47 | 91 | 93 | 63 | 61 | 123 |
            125 | 32 => false, // separators
            x if x > 127 => false, // non-CHARs
            _ => true,
        }
    })
}

/// Because this converts to a DOMString it becomes UTF-8 encoded which is closer to
/// the spec definition of <https://infra.spec.whatwg.org/#serialize-a-javascript-value-to-json-bytes>
/// but we generally do not operate on anything that is truly a WTF-16 string.
///
/// <https://infra.spec.whatwg.org/#serialize-a-javascript-value-to-a-json-string>
pub fn serialize_jsval_to_json_utf8(
    cx: &mut JSContext,
    data: HandleValue,
) -> Result<DOMString, Error> {
    #[repr(C)]
    struct ToJSONCallbackData {
        string: Option<String>,
    }

    let mut out_str = ToJSONCallbackData { string: None };

    #[expect(unsafe_code)]
    unsafe extern "C" fn write_callback(
        string: *const u16,
        len: u32,
        data: *mut std::ffi::c_void,
    ) -> bool {
        let data = data as *mut ToJSONCallbackData;
        let string_chars = unsafe { slice::from_raw_parts(string, len as usize) };
        unsafe { &mut *data }
            .string
            .get_or_insert_with(Default::default)
            .push_str(&String::from_utf16_lossy(string_chars));
        true
    }

    // 1. Let result be ? Call(%JSON.stringify%, undefined, « value »).
    unsafe {
        let stringify_result = ToJSON(
            cx,
            data,
            HandleObject::null(),
            HandleValue::null(),
            Some(write_callback),
            &mut out_str as *mut ToJSONCallbackData as *mut _,
        );
        // Note: ToJSON returns false when a JS error is thrown, so we need to return
        // JSFailed to propagate the raised exception
        if !stringify_result {
            return Err(Error::JSFailed);
        }
    }

    // 2. If result is undefined, then throw a TypeError.
    // Note: ToJSON will not call the callback if the data cannot be serialized.
    // 3. Assert: result is a string.
    // 4. Return result.
    out_str
        .string
        .map(Into::into)
        .ok_or_else(|| Error::Type(c"unable to serialize JSON".to_owned()))
}

fn has_latin1_chars(jsstr: *mut JSString) -> bool {
    unsafe { JS_DeprecatedStringHasLatin1Chars(jsstr) }
}

/// Safe wrapper around <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tostring>
pub fn to_js_string(
    cx: &mut JSContext,
    source: SafeHandleValue,
    target: &mut SafeMutableHandleString,
) -> Result<(), Error> {
    rooted!(&in(cx) let jsstr = unsafe { ToString(cx, source) });
    if jsstr.is_null() {
        return Err(Error::JSFailed);
    }
    target.set(*jsstr);
    Ok(())
}

/// <https://infra.spec.whatwg.org/#code-unit>
pub struct CodeUnits(pub Vec<u16>);

/// A ad-hoc replacement for <https://webidl.spec.whatwg.org/#idl-DOMString>,
/// used by `component/script/dom/encoding`.
pub enum ConversionResult {
    CodeUnits(CodeUnits),
    String(String),
}

/// <https://webidl.spec.whatwg.org/#js-DOMString>
/// Implements the js to DOMstring conversion, but using an ad-hoc data structure,
/// because current DOMString implementation does not preserve the exact sequence of code units.
pub fn js_string_to_code_units(
    cx: &mut JSContext,
    data: SafeHandleValue,
    mut target: SafeMutableHandleString,
) -> Result<ConversionResult, Error> {
    // Step 1: If V is null
    // and the conversion is to an IDL type associated with the [LegacyNullToEmptyString] extended attribute,
    // then return the DOMString value that represents the empty string.
    // Note: skipping this step to keep a PASS on
    // /encoding/streams/encode-bad-chunks.any.html,
    // which seems to require the js error returned below.

    // Step 2: Let x be ? ToString(V).
    to_js_string(cx, data, &mut target)?;

    // Step 3: Return the IDL DOMString value that represents
    // the same sequence of code units as the one the JavaScript String value x represents.
    // Note: not using DOMString because it does not preserve the same sequence of code units.
    if has_latin1_chars(*target) {
        let string =
            unsafe { latin1_to_string(cx, NonNull::new(*target).expect("jsstr cannot be null")) };
        Ok(ConversionResult::String(string))
    } else {
        let maybe_ill_formed_code_units = unsafe {
            let mut len = 0;
            let data = JS_GetTwoByteStringCharsAndLength(cx, *target, &mut len);

            // Note: defensive copy to avoid having the js string data move underneath us.
            // For an optimal pattern,
            // see <https://searchfox.org/firefox-main/rev/a4d4f7ecfae304e10b0f6724595f9c931d7c919b/js/src/vm/StringType.cpp#1738>
            std::slice::from_raw_parts(data, len).to_vec()
        };
        Ok(ConversionResult::CodeUnits(CodeUnits(
            maybe_ill_formed_code_units,
        )))
    }
}
