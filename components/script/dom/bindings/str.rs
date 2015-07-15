/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `ByteString` struct.

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::hash::{Hash, Hasher};
use std::ops;
use std::str;
use std::str::FromStr;

/// Encapsulates the IDL `ByteString` type.
#[derive(JSTraceable,Clone,Eq,PartialEq)]
pub struct ByteString(Vec<u8>);

impl ByteString {
    /// Creates a new `ByteString`.
    pub fn new(value: Vec<u8>) -> ByteString {
        ByteString(value)
    }

    /// Returns `self` as a string, if it encodes valid UTF-8, and `None`
    /// otherwise.
    pub fn as_str<'a>(&'a self) -> Option<&'a str> {
        let ByteString(ref vec) = *self;
        str::from_utf8(&vec).ok()
    }

    /// Returns the length.
    pub fn len(&self) -> usize {
        let ByteString(ref vector) = *self;
        vector.len()
    }

    /// Compare `self` to `other`, matching A–Z and a–z as equal.
    pub fn eq_ignore_case(&self, other: &ByteString) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }

    /// Returns `self` with A–Z replaced by a–z.
    pub fn to_lower(&self) -> ByteString {
        ByteString::new(self.0.to_ascii_lowercase())
    }

    /// Returns whether `self` is a `token`, as defined by
    /// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-17).
    pub fn is_token(&self) -> bool {
        let ByteString(ref vec) = *self;
        if vec.len() == 0 {
            return false; // A token must be at least a single character
        }
        vec.iter().all(|&x| {
            // http://tools.ietf.org/html/rfc2616#section-2.2
            match x {
                0...31 | 127 => false, // CTLs
                40 | 41 | 60 | 62 | 64 |
                44 | 59 | 58 | 92 | 34 |
                47 | 91 | 93 | 63 | 61 |
                123 | 125 | 32  => false, // separators
                x if x > 127 => false, // non-CHARs
                _ => true
            }
        })
    }

    /// Returns whether `self` is a `field-value`, as defined by
    /// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-32).
    pub fn is_field_value(&self) -> bool {
        // Classifications of characters necessary for the [CRLF] (SP|HT) rule
        #[derive(PartialEq)]
        enum PreviousCharacter {
            Other,
            CR,
            LF,
            SPHT // SP or HT
        }
        let ByteString(ref vec) = *self;
        let mut prev = PreviousCharacter::Other; // The previous character
        vec.iter().all(|&x| {
            // http://tools.ietf.org/html/rfc2616#section-2.2
            match x {
                13  => { // CR
                    if prev == PreviousCharacter::Other || prev == PreviousCharacter::SPHT {
                        prev = PreviousCharacter::CR;
                        true
                    } else {
                        false
                    }
                },
                10 => { // LF
                    if prev == PreviousCharacter::CR {
                        prev = PreviousCharacter::LF;
                        true
                    } else {
                        false
                    }
                },
                32 => { // SP
                    if prev == PreviousCharacter::LF || prev == PreviousCharacter::SPHT {
                        prev = PreviousCharacter::SPHT;
                        true
                    } else if prev == PreviousCharacter::Other {
                        // Counts as an Other here, since it's not preceded by a CRLF
                        // SP is not a CTL, so it can be used anywhere
                        // though if used immediately after a CR the CR is invalid
                        // We don't change prev since it's already Other
                        true
                    } else {
                        false
                    }
                },
                9 => { // HT
                    if prev == PreviousCharacter::LF || prev == PreviousCharacter::SPHT {
                        prev = PreviousCharacter::SPHT;
                        true
                    } else {
                        false
                    }
                },
                0...31 | 127 => false, // CTLs
                x if x > 127 => false, // non ASCII
                _ if prev == PreviousCharacter::Other || prev == PreviousCharacter::SPHT => {
                    prev = PreviousCharacter::Other;
                    true
                },
                _ => false // Previous character was a CR/LF but not part of the [CRLF] (SP|HT) rule
            }
        })
    }
}

impl Hash for ByteString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ByteString(ref vec) = *self;
        vec.hash(state);
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
