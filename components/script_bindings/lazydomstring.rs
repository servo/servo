use std::borrow::ToOwned;
use std::cell::OnceCell;
use std::default::Default;
use std::ffi::CString;
use std::ops::Deref;
use std::ptr::{self, NonNull};
use std::str::{EncodeUtf16, FromStr};
use std::sync::LazyLock;
use std::{fmt, slice, str};

use ascii::ToAsciiChar;
use html5ever::{LocalName, Namespace};
use js::conversions::{ToJSValConvertible, jsstr_to_string};
use js::gc::MutableHandleValue;
use js::jsapi::{Heap, JS_GetLatin1StringCharsAndLength, JSContext, JSString};
use js::rust::Trace;
use malloc_size_of::MallocSizeOfOps;
use regex::Regex;
use style::Atom;
use tendril::encoding_rs;

use crate::str::DOMString;

fn char_to_latin1_u8(c: char) -> u8 {
    c.to_ascii_char().unwrap().into()
}

fn latin1_u8_to_char(c: u8) -> char {
    c.to_ascii_char().unwrap().into()
}

#[derive(Copy, Clone, Debug)]
pub enum EncodedBytes<'a> {
    Latin1Bytes(&'a [u8]),
    Utf8Bytes(&'a str),
}

impl<'a> EncodedBytes<'a> {
    pub fn encode_utf16(self) -> Vec<u16> {
        match self {
            EncodedBytes::Latin1Bytes(s) => {
                String::from_iter(s.iter().map(|c| latin1_u8_to_char(*c)))
                    .encode_utf16()
                    .collect()
            },
            EncodedBytes::Utf8Bytes(s) => s.encode_utf16().collect(),
        }
    }

    pub fn split_commas(self) -> Box<dyn Iterator<Item = EncodedBytes<'a>> + 'a> {
        match self {
            EncodedBytes::Latin1Bytes(s) => Box::new(
                s.split(|byte| *byte == char_to_latin1_u8(','))
                    .map(EncodedBytes::Latin1Bytes),
            ),
            EncodedBytes::Utf8Bytes(s) => Box::new(s.split(',').map(EncodedBytes::Utf8Bytes)),
        }
    }

    pub fn char_indices(self) -> Box<dyn Iterator<Item = (usize, char)> + 'a> {
        match self {
            EncodedBytes::Latin1Bytes(items) => Box::new(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, c)| (index, latin1_u8_to_char(*c))),
            ),
            EncodedBytes::Utf8Bytes(s) => Box::new(s.char_indices()),
        }
    }
}

impl<'a> PartialEq<str> for EncodedBytes<'a> {
    fn eq(&self, other: &str) -> bool {
        match self {
            EncodedBytes::Utf8Bytes(s) => *s == other,
            EncodedBytes::Latin1Bytes(s) => {
                let v = s.iter().map(|c| *c as char as u8).collect::<Vec<u8>>();
                v == *s
            },
        }
    }
}

impl<'a> PartialEq<&str> for EncodedBytes<'a> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            EncodedBytes::Utf8Bytes(s) => s == other,
            EncodedBytes::Latin1Bytes(s) => {
                let v = s.iter().map(|c| *c as char as u8).collect::<Vec<u8>>();
                &String::from_utf8(v).unwrap() == other
            },
        }
    }
}

impl<'a> PartialEq<&str> for Box<EncodedBytes<'a>> {
    fn eq(&self, other: &&str) -> bool {
        match self.deref() {
            EncodedBytes::Utf8Bytes(s) => s == other,
            EncodedBytes::Latin1Bytes(s) => {
                let v = s.iter().map(|c| *c as char as u8).collect::<Vec<u8>>();
                &String::from_utf8(v).unwrap() == other
            },
        }
    }
}

#[cfg_attr(crown, allow(crown::unrooted_must_root))]
/// This string class will keep either the Reference toe the mozjs object alive
/// or will have an internal rust string.
/// We currently default to doing most of the string operation on the rust side.
/// As this conversion was anyway needed, it does not much extra cost.
/// You should assume that all the functions incur the conversion cost.
pub struct LazyDOMString {
    rust_string: OnceCell<String>,
    js_context: Option<*mut JSContext>,
    js_string: Option<std::boxed::Box<Heap<*mut JSString>>>,
}

impl std::fmt::Debug for LazyDOMString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyDOMString")
            .field("rust_string", &self.rust_string)
            .finish()
    }
}

impl Clone for LazyDOMString {
    fn clone(&self) -> Self {
        self.make_me_string();
        Self {
            rust_string: self.rust_string.clone(),
            js_context: None,
            js_string: None,
        }
    }
}

unsafe impl Trace for LazyDOMString {
    unsafe fn trace(&self, tracer: *mut js::jsapi::JSTracer) {
        // We can safely delete the jsstring if we already converted to a rust string.
        if self.rust_string.get().is_none() {
            if let Some(ref s) = self.js_string {
                unsafe { s.trace(tracer) }
            }
        }
    }
}

impl LazyDOMString {
    /// Creates a new `DOMString`.
    pub fn new() -> LazyDOMString {
        LazyDOMString {
            rust_string: OnceCell::from(String::new()),
            js_context: None,
            js_string: None,
        }
    }

    /// This method will do some work if necessary but not an allocation.
    pub fn bytes<'a>(&'a self) -> EncodedBytes<'a> {
        self.debug_js();
        match self.rust_string.get() {
            Some(s) => EncodedBytes::Utf8Bytes(s.as_str()),
            None => {
                let mut length = 0;
                unsafe {
                    let chars = JS_GetLatin1StringCharsAndLength(
                        self.js_context.unwrap(),
                        ptr::null(),
                        self.js_string.as_ref().unwrap().get(),
                        &mut length,
                    );
                    assert!(!chars.is_null());

                    EncodedBytes::Latin1Bytes(slice::from_raw_parts(chars, length))
                }
            },
        }
    }

    /// This method is here for compatibilities sake.
    pub fn str(&self) -> EncodedBytes<'_> {
        self.debug_js();
        self.bytes()
    }

    /// Creates a new `DOMString` from a `String`.
    pub fn from_string(s: String) -> LazyDOMString {
        LazyDOMString {
            rust_string: OnceCell::from(s),
            js_context: None,
            js_string: None,
        }
    }

    /// Debug the current  state of the string
    #[allow(unused)]
    fn debug_js(&self) {
        if self.js_string.is_some() && self.rust_string.get().is_none() {
            unsafe {
                println!(
                    "jsstring {:?}",
                    jsstr_to_string(
                        self.js_context.unwrap(),
                        ptr::NonNull::new(self.js_string.as_ref().unwrap().get()).unwrap()
                    )
                );
            }
        } else {
            println!("only rust string {:?}", self.rust_string.get().unwrap());
        }
    }

    pub fn from_js_string(cx: *mut JSContext, value: js::gc::HandleValue) -> LazyDOMString {
        let string_ptr = unsafe { js::rust::ToString(cx, value) };
        if !string_ptr.is_null() {
            let latin1 = unsafe { js::jsapi::JS_DeprecatedStringHasLatin1Chars(string_ptr) };
            if latin1 {
                let h = Heap::boxed(string_ptr);
                LazyDOMString {
                    rust_string: OnceCell::new(),
                    js_context: Some(cx),
                    js_string: Some(h),
                }
            } else {
                // We need to convert the string anyway as it is not just latin1
                LazyDOMString::from_string(unsafe {
                    jsstr_to_string(cx, ptr::NonNull::new(string_ptr).unwrap())
                })
            }
        } else {
            LazyDOMString::from_string(String::new())
        }
    }

    fn make_me_string(&self) {
        self.rust_string.get_or_init(|| unsafe {
            info!("Converting js string to rust");
            jsstr_to_string(
                self.js_context.unwrap(),
                NonNull::new(self.js_string.as_ref().unwrap().get()).unwrap(),
            )
        });
    }

    pub fn encode_utf16(&self) -> EncodeUtf16<'_> {
        self.make_me_string();
        self.rust_str().encode_utf16()
    }

    pub fn rust_str(&self) -> &str {
        self.make_me_string();
        self.rust_string.get().unwrap()
    }

    pub fn to_domstring(&self) -> DOMString {
        self.make_me_string();
        DOMString::from_string(self.rust_string.get().unwrap().to_owned())
    }

    pub fn clear(&mut self) {
        if let Some(val) = self.rust_string.get_mut() {
            val.clear();
        } else {
            self.debug_js();
            self.rust_string
                .set(String::new())
                .expect("Error in clearing");
        }
    }

    pub fn push_str(&mut self, s: &str) {
        self.make_me_string();
        self.rust_string.get_mut().unwrap().push_str(s)
    }

    pub fn is_empty(&self) -> bool {
        self.make_me_string();
        self.rust_string.get().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.str().len()
    }

    pub fn strip_leading_and_trailing_ascii_whitespace(&mut self) {
        if self.is_empty() {
            return;
        }

        self.make_me_string();
        let s = self.rust_string.get_mut().unwrap();

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

    pub fn make_ascii_lowercase(&mut self) {
        self.make_me_string();
        self.rust_string.get_mut().unwrap().make_ascii_lowercase();
    }

    pub fn is_valid_floating_point_number_string(&self) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^-?(?:\d+\.\d+|\d+|\.\d+)(?:(e|E)(\+|\-)?\d+)?$").unwrap()
        });

        RE.is_match(self.rust_str()) && self.parse_floating_point_number().is_some()
    }

    pub fn parse<T: FromStr + std::fmt::Debug>(&self) -> Result<T, <T as FromStr>::Err> {
        self.make_me_string();
        self.str().parse::<T>()
    }

    /// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-floating-point-number-values>
    pub fn parse_floating_point_number(&self) -> Option<f64> {
        self.to_domstring().parse_floating_point_number()
    }

    pub fn set_best_representation_of_the_floating_point_number(&mut self) {
        self.to_domstring()
            .set_best_representation_of_the_floating_point_number();
    }

    pub fn strip_newlines(&mut self) {
        self.make_me_string();
        self.rust_string
            .get_mut()
            .unwrap()
            .retain(|c| c != '\r' && c != '\n');
    }

    pub fn replace(self, needle: &str, replace_char: &str) -> LazyDOMString {
        self.make_me_string();
        let new_string = self.rust_string.get().unwrap().to_owned();
        LazyDOMString::from_string(new_string.replace(needle, replace_char))
    }

    pub fn split(&mut self, c: char) -> impl Iterator<Item = &str> {
        self.make_me_string();
        self.rust_string.get().unwrap().split(c)
    }
}

impl ToJSValConvertible for LazyDOMString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.make_me_string();
        unsafe {
            self.rust_str().to_jsval(cx, rval);
        }
    }
}

impl std::hash::Hash for LazyDOMString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.make_me_string();
        self.rust_string.get().hash(state);
    }
}

impl From<&str> for LazyDOMString {
    fn from(contents: &str) -> LazyDOMString {
        LazyDOMString::from_string(String::from(contents))
    }
}

impl From<LazyDOMString> for String {
    fn from(val: LazyDOMString) -> Self {
        val.make_me_string();
        val.rust_str().to_owned()
    }
}

impl From<LazyDOMString> for Vec<u8> {
    fn from(mut value: LazyDOMString) -> Self {
        value.make_me_string();
        value.rust_string.take().unwrap().as_bytes().to_vec()
    }
}

impl malloc_size_of::MallocSizeOf for LazyDOMString {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if let Some(s) = self.rust_string.get() {
            s.size_of(ops)
        } else {
            0
        }
    }
}

impl std::fmt::Display for LazyDOMString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.make_me_string();
        fmt::Display::fmt(self.rust_string.get().unwrap(), f)
    }
}

impl Default for LazyDOMString {
    fn default() -> Self {
        LazyDOMString::new()
    }
}

impl std::cmp::PartialEq<&str> for LazyDOMString {
    fn eq(&self, other: &&str) -> bool {
        self.make_me_string();
        self.rust_string.get().unwrap() == *other
    }
}

impl std::cmp::PartialEq for LazyDOMString {
    fn eq(&self, other: &Self) -> bool {
        self.make_me_string();
        other.make_me_string();
        self.rust_str() == other.rust_str()
    }
}

impl std::cmp::Eq for LazyDOMString {}

impl From<std::string::String> for LazyDOMString {
    fn from(value: String) -> Self {
        LazyDOMString::from_string(value)
    }
}

impl From<LazyDOMString> for LocalName {
    fn from(contents: LazyDOMString) -> LocalName {
        contents.make_me_string();
        LocalName::from(contents.rust_str())
    }
}

impl From<LazyDOMString> for Namespace {
    fn from(contents: LazyDOMString) -> Namespace {
        contents.make_me_string();
        Namespace::from(contents.rust_str())
    }
}

impl From<LazyDOMString> for Atom {
    fn from(contents: LazyDOMString) -> Atom {
        contents.make_me_string();
        Atom::from(contents.rust_str())
    }
}

impl From<EncodedBytes<'_>> for LazyDOMString {
    fn from(value: EncodedBytes<'_>) -> Self {
        match value {
            EncodedBytes::Utf8Bytes(s) => LazyDOMString::from_string(s.to_string()),
            EncodedBytes::Latin1Bytes(s) => {
                LazyDOMString::from_string(encoding_rs::mem::decode_latin1(s).into_owned())
            },
        }
    }
}

pub trait StringTrait<'a>
where
    Self: Sized,
{
    fn split(self, split_char: char) -> Box<dyn Iterator<Item = Self> + 'a>;
    fn parse<T: FromStr + std::fmt::Debug>(self) -> Result<T, <T as FromStr>::Err>;
    fn len(self) -> usize;
    fn empty(self) -> bool;
    fn split_at(self, index: usize) -> (Self, Self)
    where
        Self: Sized;
    fn contains(self, char: char) -> bool;
    fn split_commas(self) -> Box<dyn Iterator<Item = Self> + 'a> {
        self.split(',')
    }
    /// You should assume this function is slow.
    fn chars(self) -> Box<dyn Iterator<Item = char> + 'a>;
}

impl<'a> StringTrait<'a> for EncodedBytes<'a> {
    fn split(self, split_char: char) -> Box<dyn Iterator<Item = EncodedBytes<'a>> + 'a> {
        match self {
            EncodedBytes::Utf8Bytes(s) => {
                Box::new(s.split(split_char).map(EncodedBytes::Utf8Bytes))
            },
            EncodedBytes::Latin1Bytes(items) => Box::new(
                items
                    .split(move |c| *c == char_to_latin1_u8(split_char))
                    .map(EncodedBytes::Latin1Bytes),
            ),
        }
    }

    fn parse<T: FromStr + std::fmt::Debug>(self) -> Result<T, <T as FromStr>::Err> {
        match self {
            EncodedBytes::Latin1Bytes(s) => {
                let f = encoding_rs::mem::decode_latin1(s);
                f.parse::<T>()
            },
            EncodedBytes::Utf8Bytes(s) => s.parse::<T>(),
        }
    }

    fn len(self) -> usize {
        match self {
            EncodedBytes::Utf8Bytes(s) => s.len(),
            EncodedBytes::Latin1Bytes(items) => items.len(),
        }
    }

    fn empty(self) -> bool {
        match self {
            EncodedBytes::Latin1Bytes(items) => items.is_empty(),
            EncodedBytes::Utf8Bytes(s) => s.is_empty(),
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        match self {
            EncodedBytes::Utf8Bytes(s) => {
                let (a, b) = s.split_at(index);
                (EncodedBytes::Utf8Bytes(a), EncodedBytes::Utf8Bytes(b))
            },
            EncodedBytes::Latin1Bytes(items) => {
                let (a, b) = items.split_at(index);
                (EncodedBytes::Latin1Bytes(a), EncodedBytes::Latin1Bytes(b))
            },
        }
    }

    fn contains(self, char: char) -> bool {
        match self {
            EncodedBytes::Utf8Bytes(s) => s.contains(char),
            EncodedBytes::Latin1Bytes(items) => items.contains(&char_to_latin1_u8(char)),
        }
    }

    fn chars(self) -> Box<dyn Iterator<Item = char> + 'a> {
        match self {
            EncodedBytes::Utf8Bytes(s) => Box::new(s.chars()),
            EncodedBytes::Latin1Bytes(s) => Box::new(s.iter().map(|c| latin1_u8_to_char(*c))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn latin1_u8_to_char_test_only_ascii() {
        let latin1 = vec![
            ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0',
            '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A',
            'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
            'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c',
            'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
            'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~',
        ];
        for i in latin1 {
            assert!(latin1_u8_to_char(i as u8) == i);
        }
    }

    #[test]
    #[should_panic]
    fn error_conversion_to_char() {
        latin1_u8_to_char(b"\xA0"[0]);
    }
}
