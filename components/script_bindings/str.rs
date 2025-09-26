/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `ByteString` struct.
use std::borrow::{Borrow, ToOwned};
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::{fmt, ops, slice, str};

use js::gc::{HandleObject, HandleValue};
use js::rust::wrappers::ToJSON;

pub use crate::domstring::DOMString;
use crate::error::Error;
use crate::script_runtime::JSContext;

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

impl Borrow<str> for USVString {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for USVString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.0
    }
}

impl DerefMut for USVString {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        &mut self.0
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
        fmt::Display::fmt(&**self, f)
    }
}

impl PartialEq<str> for USVString {
    fn eq(&self, other: &str) -> bool {
        &**self == other
    }
}

impl<'a> PartialEq<&'a str> for USVString {
    fn eq(&self, other: &&'a str) -> bool {
        &**self == *other
    }
}

impl From<String> for USVString {
    fn from(contents: String) -> USVString {
        USVString(contents)
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
            40 | 41 | 60 | 62 | 64 | 44 | 59 | 58 | 92 | 34 | 47 | 91 | 93 | 63 | 61 | 123
            | 125 | 32 => false, // separators
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
pub fn serialize_jsval_to_json_utf8(cx: JSContext, data: HandleValue) -> Result<DOMString, Error> {
    #[repr(C)]
    struct ToJSONCallbackData {
        string: Option<String>,
    }

    let mut out_str = ToJSONCallbackData { string: None };

    #[allow(unsafe_code)]
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
            *cx,
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
        .ok_or_else(|| Error::Type("unable to serialize JSON".to_owned()))
}
