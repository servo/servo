/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for url properties.

/// An image url or none, used for example in list-style-image
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericUrlOrNone<U> {
    /// `none`
    None,
    /// A URL.
    Url(U),
}

pub use self::GenericUrlOrNone as UrlOrNone;

impl<Url> UrlOrNone<Url> {
    /// Initial "none" value for properties such as `list-style-image`
    pub fn none() -> Self {
        UrlOrNone::None
    }

    /// Returns whether the value is `none`.
    pub fn is_none(&self) -> bool {
        match *self {
            UrlOrNone::None => true,
            UrlOrNone::Url(..) => false,
        }
    }
}
