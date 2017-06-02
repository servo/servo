/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to borders.

use values::computed::{Number, NumberOrPercentage};
use values::computed::length::LengthOrPercentage;
use values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use values::generics::border::BorderImageSideWidth as GenericBorderImageSideWidth;
use values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use values::generics::border::BorderRadius as GenericBorderRadius;
use values::generics::rect::Rect;

/// A computed value for the `border-image-width` property.
pub type BorderImageWidth = Rect<BorderImageSideWidth>;

/// A computed value for a single side of a `border-image-width` property.
pub type BorderImageSideWidth = GenericBorderImageSideWidth<LengthOrPercentage, Number>;

/// A computed value for the `border-image-slice` property.
pub type BorderImageSlice = GenericBorderImageSlice<NumberOrPercentage>;

/// A computed value for the `border-radius` property.
pub type BorderRadius = GenericBorderRadius<LengthOrPercentage>;

/// A computed value for the `border-*-radius` longhand properties.
pub type BorderCornerRadius = GenericBorderCornerRadius<LengthOrPercentage>;

impl BorderImageSideWidth {
    /// Returns `1`.
    #[inline]
    pub fn one() -> Self {
        GenericBorderImageSideWidth::Number(1.)
    }
}
