/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values in list-style-image

use values::generics::image::ImageUrlOrNone;

/// URL-generic specified or computed `list-style-image` property.
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq)]
#[derive(ToAnimatedValue, ToComputedValue, ToCss)]
pub struct ListStyleImage<Url>(pub ImageUrlOrNone<Url>);

impl<Url> ListStyleImage<Url> {
    /// Initial value for `list-style-image`.
    #[inline]
    pub fn none() -> Self {
        ListStyleImage(ImageUrlOrNone::None)
    }
}

