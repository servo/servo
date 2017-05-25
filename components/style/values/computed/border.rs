/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to borders.

use values::computed::{Number, NumberOrPercentage};
use values::computed::length::LengthOrPercentage;
use values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use values::generics::border::BorderImageWidthSide as GenericBorderImageWidthSide;
use values::generics::rect::Rect;

/// A computed value for the `border-image-width` property.
pub type BorderImageWidth = Rect<BorderImageWidthSide>;

/// A computed value for a single side of a `border-image-width` property.
pub type BorderImageWidthSide = GenericBorderImageWidthSide<LengthOrPercentage, Number>;

/// A computed value for the `border-image-slice` property.
pub type BorderImageSlice = GenericBorderImageSlice<NumberOrPercentage>;

impl BorderImageWidthSide {
    /// Returns `1`.
    #[inline]
    pub fn one() -> Self {
        GenericBorderImageWidthSide::Number(1.)
    }
}
