/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// A version of the `Into<T>` trait from the standard library that can be used
/// to convert between two types that are not defined in the script crate.
/// This is intended to be used on dict/enum types generated from WebIDL once
/// those types are moved out of the script crate.
pub(crate) trait Convert<T> {
    fn convert(self) -> T;
}

/// A version of the `TryInto<T>` trait from the standard library that can be used
/// to convert between two types that are not defined in the script crate.
/// This is intended to be used on dict/enum types generated from WebIDL once
/// those types are moved out of the script crate.
#[cfg(feature = "webgpu")]
pub(crate) trait TryConvert<T> {
    type Error;

    fn try_convert(self) -> Result<T, Self::Error>;
}

/// A wrapper type over [`js::jsapi::UTF8Chars`]. This is created to help transferring
/// a rust string to mozjs. The inner [`js::jsapi::UTF8Chars`] can be accessed via the
/// [`std::ops::Deref`] trait.
pub(crate) struct Utf8Chars<'a> {
    lt_marker: std::marker::PhantomData<&'a ()>,
    inner: js::jsapi::UTF8Chars,
}

impl<'a> std::ops::Deref for Utf8Chars<'a> {
    type Target = js::jsapi::UTF8Chars;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> From<&'a str> for Utf8Chars<'a> {
    #[allow(unsafe_code)]
    fn from(value: &'a str) -> Self {
        let start = js::jsapi::mozilla::RangedPtr {
            _phantom_0: std::marker::PhantomData,
            mPtr: value.as_ptr() as *mut _,
        };
        let end = js::jsapi::mozilla::RangedPtr {
            _phantom_0: std::marker::PhantomData,
            mPtr: unsafe { value.as_ptr().byte_add(value.len()) as *mut _ },
        };
        let base = js::jsapi::mozilla::Range {
            _phantom_0: std::marker::PhantomData,
            mStart: start,
            mEnd: end,
        };
        let inner = js::jsapi::UTF8Chars { _base: base };
        Self {
            lt_marker: std::marker::PhantomData,
            inner,
        }
    }
}
