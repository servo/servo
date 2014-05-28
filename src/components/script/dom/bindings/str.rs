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

    pub fn is_field_value(&self) -> bool {
        // Classifications of characters necessary for the [CRLF] (SP|HT) rule
        #[deriving(Eq)]
        enum PreviousCharacter {
            Other,
            CR,
            LF,
            SP_HT // SP or HT
        }
        let ByteString(ref vec) = *self;
        let mut prev = Other; // The previous character
        vec.iter().all(|&x| {
            // http://tools.ietf.org/html/rfc2616#section-2.2
            match x {
                13  => { // CR
                    if prev == Other || prev == SP_HT {
                        prev = CR;
                        true
                    } else {
                        false
                    }
                },
                10 => { // LF
                    if prev == CR {
                        prev = LF;
                        true
                    } else {
                        false
                    }
                },
                32 | 9 => { // SP | HT
                    if prev == LF || prev == SP_HT {
                        prev = SP_HT;
                        true
                    } else {
                        false
                    }
                },
                0..31 | 127 => false, // CTLs
                x if x > 127 => false, // non ASCII
                _ if prev == Other || prev == SP_HT => {
                    prev = Other;
                    true
                },
                _ => false // Previous character was a CR/LF but not part of the [CRLF] (SP|HT) rule
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