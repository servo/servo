/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `ByteString` struct.

use std::ascii::AsciiExt;
use std::borrow::{ToOwned, Cow};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops;
use std::ops::{Deref, DerefMut};
use std::str;
use std::str::{Bytes, FromStr};
use string_cache::Atom;

/// Encapsulates the IDL `ByteString` type.
#[derive(JSTraceable, Clone, Eq, PartialEq, HeapSizeOf)]
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

    /// Compare `self` to `other`, matching A–Z and a–z as equal.
    pub fn eq_ignore_case(&self, other: &ByteString) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }

    /// Returns `self` with A–Z replaced by a–z.
    pub fn to_lower(&self) -> ByteString {
        ByteString::new(self.0.to_ascii_lowercase())
    }
}

impl Into<Vec<u8>> for ByteString {
    fn into(self) -> Vec<u8> {
        self.0
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
#[derive(Clone, HeapSizeOf)]
pub struct USVString(pub String);


/// Returns whether `s` is a `token`, as defined by
/// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-17).
pub fn is_token(s: &[u8]) -> bool {
    if s.is_empty() {
        return false; // A token must be at least a single character
    }
    s.iter().all(|&x| {
        // http://tools.ietf.org/html/rfc2616#section-2.2
        match x {
            0...31 | 127 => false, // CTLs
            40 |
            41 |
            60 |
            62 |
            64 |
            44 |
            59 |
            58 |
            92 |
            34 |
            47 |
            91 |
            93 |
            63 |
            61 |
            123 |
            125 |
            32 => false, // separators
            x if x > 127 => false, // non-CHARs
            _ => true,
        }
    })
}

/// A DOMString.
#[derive(Clone, Debug, Eq, Hash, HeapSizeOf, Ord, PartialEq, PartialOrd)]
pub struct DOMString(String);

impl !Send for DOMString {}

impl DOMString {
    /// Creates a new `DOMString`.
    pub fn new() -> DOMString {
        DOMString(String::new())
    }

    /// Creates a new `DOMString` from a `String`.
    pub fn from_string(s: String) -> DOMString {
        DOMString(s)
    }

    /// Appends a given string slice onto the end of this String.
    pub fn push_str(&mut self, string: &str) {
        self.0.push_str(string)
    }

    /// Truncates this `DOMString`, removing all contents.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// An iterator over the bytes of this `DOMString`.
    pub fn bytes(&self) -> Bytes {
        self.0.bytes()
    }
}

impl Default for DOMString {
    fn default() -> Self {
        DOMString(String::new())
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
        DOMString(contents)
    }
}

impl<'a> From<&'a str> for DOMString {
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

impl Into<Vec<u8>> for DOMString {
    fn into(self) -> Vec<u8> {
        self.0.into()
    }
}

impl Extend<char> for DOMString {
    fn extend<I>(&mut self, iterable: I) where I: IntoIterator<Item=char> {
        self.0.extend(iterable)
    }
}
