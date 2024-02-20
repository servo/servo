/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! A replacement for `Box<str>` that has a defined layout for FFI.

use crate::owned_slice::OwnedSlice;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// A struct that basically replaces a Box<str>, but with a defined layout,
/// suitable for FFI.
#[repr(C)]
#[derive(Clone, Default, Eq, MallocSizeOf, PartialEq, ToShmem)]
pub struct OwnedStr(OwnedSlice<u8>);

impl fmt::Debug for OwnedStr {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(formatter)
    }
}

impl Deref for OwnedStr {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { std::str::from_utf8_unchecked(&*self.0) }
    }
}

impl DerefMut for OwnedStr {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::str::from_utf8_unchecked_mut(&mut *self.0) }
    }
}

impl OwnedStr {
    /// Convert the OwnedStr into a boxed str.
    #[inline]
    pub fn into_box(self) -> Box<str> {
        self.into_string().into_boxed_str()
    }

    /// Convert the OwnedStr into a `String`.
    #[inline]
    pub fn into_string(self) -> String {
        unsafe { String::from_utf8_unchecked(self.0.into_vec()) }
    }
}

impl From<OwnedStr> for String {
    #[inline]
    fn from(b: OwnedStr) -> Self {
        b.into_string()
    }
}

impl From<OwnedStr> for Box<str> {
    #[inline]
    fn from(b: OwnedStr) -> Self {
        b.into_box()
    }
}

impl From<Box<str>> for OwnedStr {
    #[inline]
    fn from(b: Box<str>) -> Self {
        Self::from(b.into_string())
    }
}

impl From<String> for OwnedStr {
    #[inline]
    fn from(s: String) -> Self {
        OwnedStr(s.into_bytes().into())
    }
}
