/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for url properties.

/// An image url or none, used for example in list-style-image
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq)]
#[derive(ToAnimatedValue, ToComputedValue, ToCss)]
pub enum ImageUrlOrNone<Url> {
    /// `none`
    None,
    /// `An image URL`
    Url(Url),
}
