/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `ByteString` struct.
use std::borrow::{Borrow, Cow, ToOwned};
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::LazyLock;
use std::{fmt, ops, slice, str};

use cssparser::CowRcStr;
use html5ever::{LocalName, Namespace};
use js::rust::wrappers::ToJSON;
use js::rust::{HandleObject, HandleValue};
use num_traits::Zero;
use regex::Regex;
use stylo_atoms::Atom;
use utf16string::Utf16String;

use crate::error::Error;
use crate::script_runtime::JSContext as SafeJSContext;

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
            40 | 41 | 60 | 62 | 64 | 44 | 59 | 58 | 92 | 34 | 47 | 91 | 93 | 63 | 61 | 123 |
            125 | 32 => false, // separators
            x if x > 127 => false, // non-CHARs
            _ => true,
        }
    })
}

/// A DOMString.
///
/// This type corresponds to the [`DOMString`] type in WebIDL.
///
/// [`DOMString`]: https://webidl.spec.whatwg.org/#idl-DOMString
///
/// Conceptually, a DOMString has the same value space as a JavaScript String,
/// i.e., an array of 16-bit *code units* representing UTF-16, potentially with
/// unpaired surrogates present (also sometimes called WTF-16).
///
/// Currently, this type stores a Rust `String`, in order to avoid issues when
/// integrating with the rest of the Rust ecosystem and even the rest of the
/// browser itself.
///
/// However, Rust `String`s are guaranteed to be valid UTF-8, and as such have
/// a *smaller value space* than WTF-16 (i.e., some JavaScript String values
/// can not be represented as a Rust `String`). This introduces the question of
/// what to do with values being passed from JavaScript to Rust that contain
/// unpaired surrogates.
///
/// The hypothesis is that it does not matter much how exactly those values are
/// transformed, because  passing unpaired surrogates into the DOM is very rare.
/// Instead Servo withh replace the unpaired surrogate by a U+FFFD replacement
/// character.
///
/// Currently, the lack of crash reports about this issue provides some
/// evidence to support the hypothesis. This evidence will hopefully be used to
/// convince other browser vendors that it would be safe to replace unpaired
/// surrogates at the boundary between JavaScript and native code. (This would
/// unify the `DOMString` and `USVString` types, both in the WebIDL standard
/// and in Servo.)
///
/// This type is currently `!Send`, in order to help with an independent
/// experiment to store `JSString`s rather than Rust `String`s.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct DOMString(String, PhantomData<*const ()>);

impl DOMString {
    /// Creates a new `DOMString`.
    pub fn new() -> DOMString {
        DOMString(String::new(), PhantomData)
    }

    /// Creates a new `DOMString` from a `String`.
    pub fn from_string(s: String) -> DOMString {
        DOMString(s, PhantomData)
    }

    /// Get the internal `&str` value of this [`DOMString`].
    pub fn str(&self) -> &str {
        &self.0
    }

    /// Appends a given string slice onto the end of this String.
    pub fn push_str(&mut self, string: &str) {
        self.0.push_str(string)
    }

    /// Clears this `DOMString`, removing all contents.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Shortens this String to the specified length.
    pub fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    /// Removes newline characters according to <https://infra.spec.whatwg.org/#strip-newlines>.
    pub fn strip_newlines(&mut self) {
        self.0.retain(|c| c != '\r' && c != '\n');
    }

    /// Removes leading and trailing ASCII whitespaces according to
    /// <https://infra.spec.whatwg.org/#strip-leading-and-trailing-ascii-whitespace>.
    pub fn strip_leading_and_trailing_ascii_whitespace(&mut self) {
        if self.0.is_empty() {
            return;
        }

        let trailing_whitespace_len = self
            .0
            .trim_end_matches(|ref c| char::is_ascii_whitespace(c))
            .len();
        self.0.truncate(trailing_whitespace_len);
        if self.0.is_empty() {
            return;
        }

        let first_non_whitespace = self.0.find(|ref c| !char::is_ascii_whitespace(c)).unwrap();
        self.0.replace_range(0..first_non_whitespace, "");
    }

    /// <https://html.spec.whatwg.org/multipage/#valid-floating-point-number>
    pub fn is_valid_floating_point_number_string(&self) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^-?(?:\d+\.\d+|\d+|\.\d+)(?:(e|E)(\+|\-)?\d+)?$").unwrap()
        });

        RE.is_match(&self.0) && self.parse_floating_point_number().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-floating-point-number-values>
    pub fn parse_floating_point_number(&self) -> Option<f64> {
        // Steps 15-16 are telling us things about IEEE rounding modes
        // for floating-point significands; this code assumes the Rust
        // compiler already matches them in any cases where
        // that actually matters. They are not
        // related to f64::round(), which is for rounding to integers.
        let input = &self.0;
        if let Ok(val) = input.trim().parse::<f64>() {
            if !(
                // A valid number is the same as what rust considers to be valid,
                // except for +1., NaN, and Infinity.
                val.is_infinite() || val.is_nan() || input.ends_with('.') || input.starts_with('+')
            ) {
                return Some(val);
            }
        }
        None
    }

    /// Applies the same processing as `parse_floating_point_number` with some additional handling
    /// according to ECMA's string conversion steps.
    ///
    /// Used for specific elements when handling floating point values, namely the `number` and
    /// `range` inputs, as well as `meter` and `progress` elements.
    ///
    /// <https://html.spec.whatwg.org/multipage/#best-representation-of-the-number-as-a-floating-point-number>
    /// <https://tc39.es/ecma262/#sec-numeric-types-number-tostring>
    pub fn set_best_representation_of_the_floating_point_number(&mut self) {
        if let Some(val) = self.parse_floating_point_number() {
            // [tc39] Step 2: If x is either +0 or -0, return "0".
            let parsed_value = if val.is_zero() { 0.0_f64 } else { val };

            self.0 = parsed_value.to_string()
        }
    }
}

/// Because this converts to a DOMString it becomes UTF-8 encoded which is closer to
/// the spec definition of <https://infra.spec.whatwg.org/#serialize-a-javascript-value-to-json-bytes>
/// but we generally do not operate on anything that is truly a WTF-16 string.
///
/// <https://infra.spec.whatwg.org/#serialize-a-javascript-value-to-a-json-string>
pub fn serialize_jsval_to_json_utf8(
    cx: SafeJSContext,
    data: HandleValue,
) -> Result<DOMString, Error> {
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
        let string_chars = slice::from_raw_parts(string, len as usize);
        (*data)
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

impl Borrow<str> for DOMString {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Default for DOMString {
    fn default() -> Self {
        DOMString(String::new(), PhantomData)
    }
}

impl Deref for DOMString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.0
    }
}

impl DerefMut for DOMString {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for DOMString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DOMString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl PartialEq<str> for DOMString {
    fn eq(&self, other: &str) -> bool {
        &**self == other
    }
}

impl<'a> PartialEq<&'a str> for DOMString {
    fn eq(&self, other: &&'a str) -> bool {
        &**self == *other
    }
}

impl From<String> for DOMString {
    fn from(contents: String) -> DOMString {
        DOMString(contents, PhantomData)
    }
}

impl From<&str> for DOMString {
    fn from(contents: &str) -> DOMString {
        DOMString::from(String::from(contents))
    }
}

impl<'a> From<Cow<'a, str>> for DOMString {
    fn from(contents: Cow<'a, str>) -> DOMString {
        match contents {
            Cow::Owned(s) => DOMString::from(s),
            Cow::Borrowed(s) => DOMString::from(s),
        }
    }
}

impl From<DOMString> for LocalName {
    fn from(contents: DOMString) -> LocalName {
        LocalName::from(contents.0)
    }
}

impl From<DOMString> for Namespace {
    fn from(contents: DOMString) -> Namespace {
        Namespace::from(contents.0)
    }
}

impl From<DOMString> for Atom {
    fn from(contents: DOMString) -> Atom {
        Atom::from(contents.0)
    }
}

impl From<DOMString> for String {
    fn from(contents: DOMString) -> String {
        contents.0
    }
}

impl From<DOMString> for Vec<u8> {
    fn from(contents: DOMString) -> Vec<u8> {
        contents.0.into()
    }
}

impl<'a> From<DOMString> for Cow<'a, str> {
    fn from(contents: DOMString) -> Cow<'a, str> {
        contents.0.into()
    }
}

impl<'a> From<DOMString> for CowRcStr<'a> {
    fn from(contents: DOMString) -> CowRcStr<'a> {
        contents.0.into()
    }
}

impl Extend<char> for DOMString {
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = char>,
    {
        self.0.extend(iterable)
    }
}

impl From<Utf16String> for DOMString {
    fn from(value: Utf16String) -> Self {
        Self::from(value.to_string())
    }
}

impl From<DOMString> for Utf16String {
    fn from(value: DOMString) -> Self {
        Self::from(value.str())
    }
}
