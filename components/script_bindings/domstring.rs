/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(clippy::non_canonical_partial_ord_impl)]
use std::borrow::{Cow, ToOwned};
use std::cell::{Ref, RefCell};
use std::default::Default;
use std::ops::Deref;
use std::ptr::{self, NonNull};
use std::str::{Chars, FromStr};
use std::sync::LazyLock;
use std::{fmt, slice, str};

use html5ever::{LocalName, Namespace};
use js::conversions::{ToJSValConvertible, jsstr_to_string};
use js::gc::MutableHandleValue;
use js::jsapi::{Heap, JS_GetLatin1StringCharsAndLength, JSContext, JSString};
use js::rust::{Runtime, Trace};
use malloc_size_of::MallocSizeOfOps;
use num_traits::Zero;
use regex::Regex;
use style::Atom;
use style::str::HTML_SPACE_CHARACTERS;

use crate::script_runtime::JSContext as SafeJSContext;
use crate::trace::RootedTraceableBox;

#[derive(Debug, PartialEq, Eq)]
/// A type representing the underlying encoded bytes. Either Latin1 or Utf8.
pub enum EncodedBytes<'a> {
    /// These bytes are Latin1 encoded.
    Latin1Bytes(&'a [u8]),
    /// This is a normal utf8 string.
    Utf8Bytes(&'a str),
}

enum DOMStringType {
    /// A simple rust string
    Rust(String),
    /// A JS String stored in mozjs.
    JSString(RootedTraceableBox<Heap<*mut JSString>>),
    #[cfg(test)]
    /// This is used for testing of the bindings to give
    /// a raw u8 Latin1 encoded string without having a js engine.
    Latin1Vec(Vec<u8>),
}

impl DOMStringType {
    #[allow(unused)]
    /// Returns the str if Rust and otherwise panic. You need to call `make_rust`.
    fn str(&self) -> &str {
        match self {
            DOMStringType::Rust(s) => s,
            DOMStringType::JSString(_rooted_traceable_box) => {
                panic!("Cannot do a string")
            },
            #[cfg(test)]
            &DOMStringType::Latin1Vec(_) => panic!("Cannot do a string"),
        }
    }
}

#[derive(Debug)]
/// A view of the underlying string. This is always converted to Utf8.
pub struct StringView<'a>(Ref<'a, DOMStringType>);

impl<'a> StringView<'a> {
    pub fn split_html_space_characters(&self) -> impl Iterator<Item = &str> {
        self.0
            .str()
            .split(HTML_SPACE_CHARACTERS)
            .filter(|s| !s.is_empty())
    }

    pub fn strip_prefix(&self, needle: &str) -> Option<&str> {
        self.0.str().strip_prefix(needle)
    }

    pub fn chars(&self) -> Chars<'_> {
        self.0.str().chars()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.str().as_bytes()
    }
}

impl Deref for StringView<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.str()
    }
}

impl AsRef<str> for StringView<'_> {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl PartialEq for StringView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.str() == other.0.str()
    }
}

impl PartialEq<&str> for StringView<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.0.str() == *other
    }
}

impl Eq for StringView<'_> {}

impl PartialOrd for StringView<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.str().partial_cmp(other.0.str())
    }
}

impl Ord for StringView<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.str().cmp(other.0.str())
    }
}

impl From<StringView<'_>> for String {
    fn from(value: StringView<'_>) -> Self {
        String::from(value.0.str())
    }
}

/// Safety comment: ??
///
/// This method will _not_ trace the pointer if the rust string exists.
/// The js string could be garbage collected and, hence, violating this
/// could lead to undefined behavior
unsafe impl Trace for DOMStringType {
    unsafe fn trace(&self, tracer: *mut js::jsapi::JSTracer) {
        unsafe {
            match self {
                DOMStringType::Rust(_s) => {},
                DOMStringType::JSString(rooted_traceable_box) => rooted_traceable_box.trace(tracer),
                #[cfg(test)]
                DOMStringType::Latin1Vec(_s) => {},
            }
        }
    }
}

impl malloc_size_of::MallocSizeOf for DOMStringType {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match self {
            DOMStringType::Rust(s) => s.size_of(ops),
            DOMStringType::JSString(_rooted_traceable_box) => {
                // Managed by JS Engine
                0
            },
            #[cfg(test)]
            DOMStringType::Latin1Vec(s) => s.size_of(ops),
        }
    }
}

impl std::fmt::Debug for DOMStringType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DOMStringType::Rust(s) => f.debug_struct("DOMString").field("rust_string", s).finish(),
            DOMStringType::JSString(_rooted_traceable_box) => f.debug_struct("DOMString").finish(),
            #[cfg(test)]
            DOMStringType::Latin1Vec(s) => f
                .debug_struct("DOMString")
                .field("latin1_string", s)
                .finish(),
        }
    }
}

#[derive(Debug)]
/// A view of the underlying string. This is never converted to Utf8
pub struct EncodedBytesView<'a>(Ref<'a, DOMStringType>);

impl EncodedBytesView<'_> {
    /// Get the bytes of the string in either latin1 or utf8 without costly conversion.
    pub fn encoded_bytes(&self) -> EncodedBytes<'_> {
        match *self.0 {
            DOMStringType::Rust(ref s) => EncodedBytes::Utf8Bytes(s.as_str()),
            DOMStringType::JSString(ref rooted_traceable_box) => {
                let mut length = 0;
                unsafe {
                    let chars = JS_GetLatin1StringCharsAndLength(
                        Runtime::get().expect("JS runtime has shut down").as_ptr(),
                        ptr::null(),
                        rooted_traceable_box.get(),
                        &mut length,
                    );
                    assert!(!chars.is_null());
                    EncodedBytes::Latin1Bytes(slice::from_raw_parts(chars, length))
                }
            },
            #[cfg(test)]
            DOMStringType::Latin1Vec(ref s) => EncodedBytes::Latin1Bytes(s),
        }
    }

    fn is_empty(&self) -> bool {
        match self.encoded_bytes() {
            EncodedBytes::Latin1Bytes(items) => items.is_empty(),
            EncodedBytes::Utf8Bytes(s) => s.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self.encoded_bytes() {
            EncodedBytes::Latin1Bytes(items) => {
                items.iter().map(|b| if *b < 0xa0 { 1 } else { 2 }).sum()
            },
            EncodedBytes::Utf8Bytes(s) => s.len(),
        }
    }
}

////// A DOMString.
///
/// This type corresponds to the [`DOMString`] type in WebIDL.
///
/// [`DOMString`]: https://webidl.spec.whatwg.org/#idl-DOMString
///
/// Conceptually, a DOMString has the same value space as a JavaScript String,
/// i.e., an array of 16-bit *code units* representing UTF-16, potentially with
/// unpaired surrogates present (also sometimes called WTF-16).
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
/// This string class will keep either the Reference to the mozjs object alive
/// or will have an internal rust string.
/// We currently default to doing most of the string operation on the rust side.
/// You should use `str()` to get the Rust string (represented by a `StringView`
/// which you can deref to a string). You should assume that this conversion costs.
/// You should assume that all the functions incur the conversion cost.
///
#[repr(transparent)]
#[derive(Debug, MallocSizeOf, JSTraceable)]
pub struct DOMString(RefCell<DOMStringType>);

impl Clone for DOMString {
    fn clone(&self) -> Self {
        self.make_rust();
        if let DOMStringType::Rust(ref s) = *self.0.borrow() {
            DOMString::from_string(s.to_owned())
        } else {
            unreachable!()
        }
    }
}

impl DOMString {
    /// Creates a new `DOMString`.
    pub fn new() -> DOMString {
        DOMString(RefCell::new(DOMStringType::Rust(String::new())))
    }

    /// Creates the string from js. If the string can be encoded in latin1, just take the reference
    /// to the JSString. Otherwise do the conversion to utf8 now.
    pub fn from_js_string(cx: SafeJSContext, value: js::gc::HandleValue) -> DOMString {
        let string_ptr = unsafe { js::rust::ToString(*cx, value) };
        let inner = if string_ptr.is_null() {
            DOMStringType::Rust(String::new())
        } else {
            let latin1 = unsafe { js::jsapi::JS_DeprecatedStringHasLatin1Chars(string_ptr) };
            if latin1 {
                let h = RootedTraceableBox::from_box(Heap::boxed(string_ptr));
                DOMStringType::JSString(h)
            } else {
                // We need to convert the string anyway as it is not just latin1
                DOMStringType::Rust(unsafe {
                    jsstr_to_string(*cx, ptr::NonNull::new(string_ptr).unwrap())
                })
            }
        };
        DOMString(RefCell::new(inner))
    }

    pub fn from_string(s: String) -> DOMString {
        DOMString(RefCell::new(DOMStringType::Rust(s)))
    }

    /// Transforms the string into rust string if not yet a rust string.
    fn make_rust(&self) {
        let string = {
            let inner = self.0.borrow();
            match *inner {
                DOMStringType::Rust(_) => return,
                DOMStringType::JSString(ref rooted_traceable_box) => unsafe {
                    jsstr_to_string(
                        Runtime::get().expect("JS runtime has shut down").as_ptr(),
                        NonNull::new(rooted_traceable_box.get()).unwrap(),
                    )
                },
                #[cfg(test)]
                DOMStringType::Latin1Vec(ref items) => {
                    let mut v = vec![0; items.len() * 2];
                    let real_size = tendril::encoding_rs::mem::convert_latin1_to_utf8(
                        items.as_slice(),
                        v.as_mut_slice(),
                    );
                    v.truncate(real_size);

                    // Safety: convert_latin1_to_utf8 converts the raw bytes to utf8 and the
                    // buffer is the size specified in the documentation, so this should be safe.
                    unsafe { String::from_utf8_unchecked(v) }
                },
            }
        };
        *self.0.borrow_mut() = DOMStringType::Rust(string);
    }

    /// Debug the current  state of the string without modifying it.
    #[allow(unused)]
    fn debug_js(&self) {
        match *self.0.borrow() {
            DOMStringType::Rust(ref s) => info!("Rust String ({})", s),
            DOMStringType::JSString(ref rooted_traceable_box) => {
                let s = unsafe {
                    jsstr_to_string(
                        Runtime::get().expect("JS runtime has shut down").as_ptr(),
                        ptr::NonNull::new(rooted_traceable_box.get()).unwrap(),
                    )
                };
                info!("JSString ({})", s);
            },
            #[cfg(test)]
            DOMStringType::Latin1Vec(ref items) => info!("Latin1 string"),
        }
    }

    /// Returns the underlying rust string.
    pub fn str(&self) -> StringView<'_> {
        self.make_rust();
        StringView(self.0.borrow())
    }

    /// Use this if you want to work on the `EncodedBytes` directly.
    /// This will not do any conversions for you.
    pub fn view(&self) -> EncodedBytesView<'_> {
        EncodedBytesView(self.0.borrow())
    }

    pub fn clear(&mut self) {
        *self.0.borrow_mut() = DOMStringType::Rust(String::new())
    }

    pub fn is_empty(&self) -> bool {
        self.view().is_empty()
    }

    /// This length (as rust spec) is in bytes not chars.
    pub fn len(&self) -> usize {
        self.view().len()
    }

    pub fn make_ascii_lowercase(&mut self) {
        self.make_rust();
        if let DOMStringType::Rust(ref mut s) = *self.0.borrow_mut() {
            s.make_ascii_lowercase();
        }
    }

    pub fn push_str(&mut self, s: &str) {
        self.make_rust();
        if let DOMStringType::Rust(ref mut string) = *self.0.borrow_mut() {
            string.push_str(s)
        }
    }

    pub fn strip_leading_and_trailing_ascii_whitespace(&mut self) {
        if self.is_empty() {
            return;
        }

        self.make_rust();
        if let DOMStringType::Rust(ref mut s) = *self.0.borrow_mut() {
            let trailing_whitespace_len = s
                .trim_end_matches(|ref c| char::is_ascii_whitespace(c))
                .len();
            s.truncate(trailing_whitespace_len);
            if s.is_empty() {
                return;
            }

            let first_non_whitespace = s.find(|ref c| !char::is_ascii_whitespace(c)).unwrap();
            s.replace_range(0..first_non_whitespace, "");
        }
    }

    /// This is a dom spec
    pub fn is_valid_floating_point_number_string(&self) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^-?(?:\d+\.\d+|\d+|\.\d+)(?:(e|E)(\+|\-)?\d+)?$").unwrap()
        });
        self.make_rust();

        if let DOMStringType::Rust(ref s) = *self.0.borrow() {
            RE.is_match(s) && self.parse_floating_point_number().is_some()
        } else {
            unreachable!()
        }
    }

    pub fn parse<T: FromStr>(&self) -> Result<T, <T as FromStr>::Err> {
        self.make_rust();
        self.str().parse::<T>()
    }

    /// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-floating-point-number-values>
    pub fn parse_floating_point_number(&self) -> Option<f64> {
        self.make_rust();
        parse_floating_point_number(&self.str())
    }

    /// <https://html.spec.whatwg.org/multipage/#best-representation-of-the-number-as-a-floating-point-number>
    pub fn set_best_representation_of_the_floating_point_number(&mut self) {
        if let Some(val) = self.parse_floating_point_number() {
            // [tc39] Step 2: If x is either +0 or -0, return "0".
            let parsed_value = if val.is_zero() { 0.0_f64 } else { val };

            *self.0.borrow_mut() = DOMStringType::Rust(parsed_value.to_string());
        }
    }

    pub fn to_lowercase(&self) -> String {
        self.make_rust();
        self.str().to_lowercase()
    }

    pub fn to_uppercase(&self) -> String {
        self.make_rust();
        self.str().to_uppercase()
    }

    pub fn strip_newlines(&mut self) {
        // > To strip newlines from a string, remove any U+000A LF and U+000D CR code
        // > points from the string.
        self.make_rust();
        if let DOMStringType::Rust(ref mut s) = *self.0.borrow_mut() {
            s.retain(|c| c != '\r' && c != '\n');
        }
    }

    /// Normalize newlines according to <https://infra.spec.whatwg.org/#normalize-newlines>.
    pub fn normalize_newlines(&mut self) {
        self.make_rust();
        // > To normalize newlines in a string, replace every U+000D CR U+000A LF code point
        // > pair with a single U+000A LF code point, and then replace every remaining
        // > U+000D CR code point with a U+000A LF code point.
        if let DOMStringType::Rust(ref mut s) = *self.0.borrow_mut() {
            *s = s.replace("\r\n", "\n").replace("\r", "\n")
        }
    }

    pub fn replace(self, needle: &str, replace_char: &str) -> DOMString {
        self.make_rust();
        let new_string = self.str().to_owned();
        DOMString(RefCell::new(DOMStringType::Rust(
            new_string.replace(needle, replace_char),
        )))
    }

    pub fn find(&self, c: char) -> Option<usize> {
        self.make_rust();
        self.str().find(c)
    }

    /// Pattern is not yet stable in rust, hence, we need different methods for str and char
    pub fn starts_with(&self, c: char) -> bool {
        self.make_rust();
        self.str().starts_with(c)
    }

    pub fn starts_with_str(&self, needle: &str) -> bool {
        self.make_rust();
        self.str().starts_with(needle)
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.make_rust();
        self.str().contains(needle)
    }

    pub fn to_ascii_lowercase(&self) -> String {
        self.make_rust();
        self.str().to_ascii_lowercase()
    }

    pub fn contains_html_space_characters(&self) -> bool {
        self.make_rust();
        self.str().contains(HTML_SPACE_CHARACTERS)
    }

    /// This returns the string in utf8 bytes, i.e., `[u8]`.
    pub fn as_bytes(&self) -> BytesView<'_> {
        self.make_rust();
        BytesView(self.0.borrow())
    }
}

/// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-floating-point-number-values>
pub fn parse_floating_point_number(input: &str) -> Option<f64> {
    // Steps 15-16 are telling us things about IEEE rounding modes
    // for floating-point significands; this code assumes the Rust
    // compiler already matches them in any cases where
    // that actually matters. They are not
    // related to f64::round(), which is for rounding to integers.
    input.trim().parse::<f64>().ok().filter(|value| {
        // A valid number is the same as what rust considers to be valid,
        // except for +1., NaN, and Infinity.
        !(value.is_infinite() || value.is_nan() || input.ends_with('.') || input.starts_with('+'))
    })
}

pub struct BytesView<'a>(Ref<'a, DOMStringType>);

impl Deref for BytesView<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.str().as_bytes()
    }
}

impl Ord for DOMString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.make_rust();
        other.make_rust();
        self.str().cmp(&other.str())
    }
}

impl PartialOrd for DOMString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.make_rust();
        other.make_rust();
        self.str().partial_cmp(&other.str())
    }
}

impl Extend<char> for DOMString {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        self.make_rust();
        if let DOMStringType::Rust(ref mut s) = *self.0.borrow_mut() {
            s.extend(iter)
        }
    }
}

impl ToJSValConvertible for DOMString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.make_rust();
        unsafe {
            self.str().to_jsval(cx, rval);
        }
    }
}

impl std::hash::Hash for DOMString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.make_rust();
        self.str().hash(state);
    }
}

impl std::fmt::Display for DOMString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.make_rust();
        fmt::Display::fmt(self.str().deref(), f)
    }
}

impl Default for DOMString {
    fn default() -> Self {
        DOMString::new()
    }
}

impl std::cmp::PartialEq<str> for DOMString {
    fn eq(&self, other: &str) -> bool {
        self.make_rust();
        self.str().deref() == other
    }
}

impl std::cmp::PartialEq<&str> for DOMString {
    fn eq(&self, other: &&str) -> bool {
        self.make_rust();
        self.str().deref() == *other
    }
}

impl std::cmp::PartialEq<String> for DOMString {
    fn eq(&self, other: &String) -> bool {
        self.eq(&other.as_str())
    }
}

impl std::cmp::PartialEq<DOMString> for String {
    fn eq(&self, other: &DOMString) -> bool {
        other.eq(self)
    }
}

impl std::cmp::PartialEq<DOMString> for str {
    fn eq(&self, other: &DOMString) -> bool {
        other.eq(self)
    }
}

impl std::cmp::PartialEq for DOMString {
    fn eq(&self, other: &DOMString) -> bool {
        self.make_rust();
        other.make_rust();
        self.str() == other.str()
    }
}

impl std::cmp::Eq for DOMString {}

impl From<std::string::String> for DOMString {
    fn from(value: String) -> Self {
        DOMString::from_string(value)
    }
}

impl From<DOMString> for LocalName {
    fn from(contents: DOMString) -> LocalName {
        contents.make_rust();
        LocalName::from(contents.str().deref())
    }
}

impl From<&DOMString> for LocalName {
    fn from(contents: &DOMString) -> LocalName {
        contents.make_rust();
        LocalName::from(contents.str().deref())
    }
}

impl From<DOMString> for Namespace {
    fn from(contents: DOMString) -> Namespace {
        contents.make_rust();
        Namespace::from(contents.str().deref())
    }
}

impl From<DOMString> for Atom {
    fn from(contents: DOMString) -> Atom {
        contents.make_rust();
        Atom::from(contents.str().deref())
    }
}

impl From<&str> for DOMString {
    fn from(contents: &str) -> DOMString {
        DOMString(RefCell::new(DOMStringType::Rust(String::from(contents))))
    }
}

impl From<DOMString> for String {
    fn from(val: DOMString) -> Self {
        val.make_rust();
        val.str().to_owned()
    }
}

impl From<DOMString> for Vec<u8> {
    fn from(value: DOMString) -> Self {
        value.make_rust();
        value.str().as_bytes().to_vec()
    }
}

impl From<Cow<'_, str>> for DOMString {
    fn from(value: Cow<'_, str>) -> Self {
        DOMString(RefCell::new(DOMStringType::Rust(value.into_owned())))
    }
}

/// Use this to match &str against lazydomstring efficiently.
/// You are only allowed to match ascii strings otherwise this macro will
/// lead to wrong results.
/// ```ignore
/// let s = DOMString::from_string(String::from("test"));
/// let value = match_domstring!(s, 0,
/// "test1" => 1,
/// "test2" => 2,
/// "test" => 3,);
/// assert_eq!(value, 3);
/// ```
#[macro_export]
macro_rules! match_domstring_ascii {
    ( $input:expr, $default:expr,
        $(
            $p: expr  => $then: expr
        ),+
        $(,)?
    ) => {
        {
            use $crate::domstring::EncodedBytes;
            $(
                debug_assert!(($p).is_ascii());
            )+

            let view = $input.view();
            let s = view.encoded_bytes();
            $(
                if EncodedBytes::Latin1Bytes($p.as_bytes()) == s {
                    $then
                } else
            )+
            $(
                if EncodedBytes::Utf8Bytes($p) == s {
                    $then
                } else
            )+
            {
                $default
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_latin1(l1vec: Vec<u8>) -> DOMString {
        DOMString(RefCell::new(DOMStringType::Latin1Vec(l1vec)))
    }

    #[test]
    fn string_functions() {
        let s = DOMString::from("AbBcC❤&%$#");
        let s_copy = s.clone();
        assert_eq!(s.to_ascii_lowercase(), "abbcc❤&%$#");
        assert_eq!(s, s_copy);
        assert_eq!(s.len(), 12);
        assert_eq!(s_copy.len(), 12);
        assert!(s.starts_with('A'));
        assert_eq!(s.find('#'), Some(11));
        let s2 = DOMString::from("");
        assert!(s2.is_empty());
    }

    #[test]
    fn string_functions_latin1() {
        {
            let s = from_latin1(vec![
                b'A', b'b', b'B', b'c', b'C', b'&', b'%', b'$', b'#', 0xB2,
            ]);
            assert_eq!(s.to_ascii_lowercase(), "abbcc&%$#²");
        }
        {
            let s = from_latin1(vec![
                b'A', b'b', b'B', b'c', b'C', b'&', b'%', b'$', b'#', 0xB2,
            ]);
            assert_eq!(s.len(), 11);
            assert!(s.starts_with('A'));
            assert_eq!(s.find('#'), Some(8));
            assert_eq!(s.find(char::from_u32(0x00B2).unwrap()), Some(9));
            assert_eq!(s.find('d'), None);
        }
        {
            let s = from_latin1(vec![]);
            assert!(s.is_empty());
        }
    }

    #[test]
    fn test_convert() {
        let s = from_latin1(vec![b'a', b'b', b'c', b'%', b'$']);
        s.make_rust();
        assert_eq!(&*s.str(), "abc%$");
    }

    #[test]
    fn partial_eq() {
        let s = from_latin1(vec![b'a', b'b', b'c', b'%', b'$']);
        let string = String::from("abc%$");
        let s2 = DOMString::from_string(string.clone());
        assert_eq!(s, s2);
        assert_eq!(s, string);
    }

    #[test]
    fn encoded_bytes() {
        let bytes = vec![b'a', b'b', b'c', b'%', b'$', 0xB2];
        let s = from_latin1(bytes.clone());
        if let EncodedBytes::Latin1Bytes(s) = s.view().encoded_bytes() {
            assert_eq!(s, bytes)
        }
    }

    #[test]
    fn testing_stringview() {
        let s = from_latin1(vec![b'a', b'b', b'c', b'%', b'$', 0xB2]);

        assert_eq!(
            s.str().chars().collect::<Vec<char>>(),
            vec!['a', 'b', 'c', '%', '$', '²']
        );
        assert_eq!(s.str().as_bytes(), String::from("abc%$²").as_bytes());
    }

    // We need to be extra careful here as two strings that have different
    // representation need to have the same hash.
    // Additionally, the interior mutability is only used for the conversion
    // which is forced by Hash. Hence, it is safe to have this interior mutability.
    #[test]
    fn test_hash() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        fn hash_value(d: &DOMString) -> u64 {
            let mut hasher = DefaultHasher::new();
            d.hash(&mut hasher);
            hasher.finish()
        }

        let s = from_latin1(vec![b'a', b'b', b'c', b'%', b'$', 0xB2]);
        let s_converted = from_latin1(vec![b'a', b'b', b'c', b'%', b'$', 0xB2]);
        s_converted.make_rust();
        let s2 = DOMString::from_string(String::from("abc%$²"));

        let hash_s = hash_value(&s);
        let hash_s_converted = hash_value(&s_converted);
        let hash_s2 = hash_value(&s2);

        assert_eq!(hash_s, hash_s2);
        assert_eq!(hash_s, hash_s_converted);
    }

    // Testing match_lazydomstring if it executes the statements in the match correctly
    #[test]
    fn test_match_executing() {
        // executing
        {
            let s = from_latin1(vec![b'a', b'b', b'c']);
            match_domstring_ascii!( s, (),
                "abc" => assert!(true),
                "bcd" => assert!(false),
            );
        }

        {
            let s = from_latin1(vec![b'a', b'b', b'c', b'/']);
            match_domstring_ascii!( s, (),
                "abc/" => assert!(true),
                "bcd" => assert!(false),
            );
        }

        {
            let s = from_latin1(vec![b'a', b'b', b'c', b'%', b'$']);
            match_domstring_ascii!( s, (),
                "bcd" => assert!(false),
                "abc%$" => assert!(true),
            );
        }

        {
            let s = DOMString::from_string(String::from("abcde"));
            match_domstring_ascii!( s, (),
                "abc" => assert!(true),
                "bcd" => assert!(false),
            );
        }
        {
            let s = DOMString::from_string(String::from("abc%$"));
            match_domstring_ascii!( s, (),
                "bcd" => assert!(false),
                "abc%$" => assert!(true),
            );
        }
        {
            let s = from_latin1(vec![b'a', b'b', b'c']);
            match_domstring_ascii!( s, (),
                "abcdd" => assert!(false),
                "bcd" => assert!(false),
            );
        }
    }

    // Testing match_lazydomstring if it evaluates to the correct expression
    #[test]
    fn test_match_returning_result() {
        {
            let s = from_latin1(vec![b'a', b'b', b'c']);
            let res = match_domstring_ascii!( s, false,
                "abc" => true,
                "bcd" => false,
            );
            assert_eq!(res, true);
        }
        {
            let s = from_latin1(vec![b'a', b'b', b'c', b'/']);
            let res = match_domstring_ascii!( s, false,
                "abc/" => true,
                "bcd" => false,
            );
            assert_eq!(res, true);
        }
        {
            let s = from_latin1(vec![b'a', b'b', b'c', b'%', b'$']);
            let res = match_domstring_ascii!( s, false,
                "bcd" => false,
                "abc%$" => true,
            );
            assert_eq!(res, true);
        }

        {
            let s = DOMString::from_string(String::from("abcde"));
            let res = match_domstring_ascii!( s, true,
                "abc" => false,
                "bcd" => false,
            );
            assert_eq!(res, true);
        }
        {
            let s = DOMString::from_string(String::from("abc%$"));
            let res = match_domstring_ascii!( s, false,
                "bcd" => false,
                "abc%$" => true,
            );
            assert_eq!(res, true);
        }
        {
            let s = from_latin1(vec![b'a', b'b', b'c']);
            let res = match_domstring_ascii!( s, true,
                "abcdd" => false,
                "bcd" => false,
            );
            assert_eq!(res, true);
        }
    }

    #[test]
    #[should_panic]
    fn test_match_panic() {
        let s = DOMString::from_string(String::from("abcd"));
        let _res = match_domstring_ascii!(s, false,
        "❤" => true);
    }

    #[test]
    #[should_panic]
    fn test_match_panic2() {
        let s = DOMString::from_string(String::from("abcd"));
        let _res = match_domstring_ascii!(s, false,
            "abc" => false,
        "❤" => true);
    }

    #[test]
    fn test_strip_whitespace() {
        {
            let mut s = from_latin1(vec![
                b' ', b' ', b' ', b'\n', b' ', b'a', b'b', b'c', b'%', b'$', 0xB2, b' ',
            ]);

            s.strip_leading_and_trailing_ascii_whitespace();
            s.make_rust();
            assert_eq!(&*s.str(), "abc%$²");
        }
        {
            let mut s = DOMString::from_string(String::from("   \n  abc%$ "));

            s.strip_leading_and_trailing_ascii_whitespace();
            s.make_rust();
            assert_eq!(&*s.str(), "abc%$");
        }
    }
}
