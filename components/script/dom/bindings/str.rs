/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `ByteString` struct.

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops;
use std::str;
use std::str::FromStr;

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

    /// Returns ownership of the underlying Vec<u8> and copies an empty
    /// vec in its place
    pub fn bytes(&mut self) -> Vec<u8> {
        mem::replace(&mut self.0, Vec::new())
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
