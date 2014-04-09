/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[deriving(Eq, Clone, Encodable)]
pub enum AttrValue {
    StringAttrValue(~str),
    UIntAttrValue(~str, u32),
}

impl AttrValue {
    pub fn is_string(&self) -> bool {
        match *self {
            StringAttrValue(..) => true,
            _ => false
        }
    }

    pub fn is_uint(&self) -> bool {
        match *self {
            UIntAttrValue(..) => true,
            _ => false
        }
    }
}

impl AttrValue {
    pub fn as_str_slice<'a>(&'a self) -> &'a str {
        match *self {
            StringAttrValue(ref value) => value.as_slice(),
            UIntAttrValue(ref value, _) => value.as_slice(),
        }
    }

    pub fn as_owned_str(&self) -> ~str {
        match *self {
            StringAttrValue(ref value) => value.clone(),
            UIntAttrValue(ref value, _) => value.clone(),
        }
    }

    pub fn as_uint(&self) -> Option<u32> {
        match *self {
            UIntAttrValue(_, value) => Some(value),
            _ => None
        }
    }
}
