/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::hash::{Hash, sip};
use std::str;

#[deriving(Encodable,Clone,TotalEq,Eq)]
pub struct ByteString(Vec<u8>);

impl ByteString {
    pub fn new(value: Vec<u8>) -> ByteString {
        ByteString(value)
    }
    pub fn as_str<'a>(&'a self) -> Option<&'a str> {
        let ByteString(ref vec) = *self;
        str::from_utf8(vec.as_slice())
    }

    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        let ByteString(ref vector) = *self;
        vector.as_slice()
    }

    pub fn eq_ignore_case(&self, other: &ByteString) -> bool {
        // XXXManishearth make this more efficient
        self.to_lower() == other.to_lower()
    }

    pub fn to_lower(&self) -> ByteString {
        let ByteString(ref vec) = *self;
        ByteString::new(vec.iter().map(|&x| {
            if x > 'A' as u8 && x < 'Z' as u8 {
                x + ('a' as u8) - ('A' as u8)
            } else {
                x
            }
        }).collect())
    }
    pub fn is_token(&self) -> bool {
        let ByteString(ref vec) = *self;
        vec.iter().all(|&x| {
            // http://tools.ietf.org/html/rfc2616#section-2.2
            match x {
                0..31 | 127 => false, // CTLs
                40 | 41 | 60 | 62 | 64 |
                44 | 59 | 58 | 92 | 34 |
                47 | 91 | 93 | 63 | 61 |
                123 | 125 | 32  => false, // separators
                _ => true
            }
        })
    }
}

impl Hash for ByteString {
    fn hash(&self, state: &mut sip::SipState) {
        let ByteString(ref vec) = *self;
        vec.hash(state);
    }
}