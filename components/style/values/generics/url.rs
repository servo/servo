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
pub enum UrlOrNone<Url> {
    /// `none`
    None,
    /// `A URL`
    Url(Url),
}

impl<Url> UrlOrNone<Url> {
    /// Initial "none" value for properties such as `list-style-image`
    pub fn none() -> Self {
        UrlOrNone::None
    }
}
