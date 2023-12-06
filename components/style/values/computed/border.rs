/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to borders.

use crate::values::computed::length::{NonNegativeLength, NonNegativeLengthPercentage};
use crate::values::computed::{NonNegativeNumber, NonNegativeNumberOrPercentage};
use crate::values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use crate::values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use crate::values::generics::border::BorderRadius as GenericBorderRadius;
use crate::values::generics::border::BorderSpacing as GenericBorderSpacing;
use crate::values::generics::border::GenericBorderImageSideWidth;
use crate::values::generics::rect::Rect;
use crate::values::generics::size::Size2D;
use crate::values::generics::NonNegative;
use crate::Zero;
use app_units::Au;

pub use crate::values::specified::border::BorderImageRepeat;

/// A computed value for -webkit-text-stroke-width.
pub type LineWidth = Au;

/// A computed value for border-width (and the like).
pub type BorderSideWidth = Au;

/// A computed value for the `border-image-width` property.
pub type BorderImageWidth = Rect<BorderImageSideWidth>;

/// A computed value for a single side of a `border-image-width` property.
pub type BorderImageSideWidth =
    GenericBorderImageSideWidth<NonNegativeLengthPercentage, NonNegativeNumber>;

/// A computed value for the `border-image-slice` property.
pub type BorderImageSlice = GenericBorderImageSlice<NonNegativeNumberOrPercentage>;

/// A computed value for the `border-radius` property.
pub type BorderRadius = GenericBorderRadius<NonNegativeLengthPercentage>;

/// A computed value for the `border-*-radius` longhand properties.
pub type BorderCornerRadius = GenericBorderCornerRadius<NonNegativeLengthPercentage>;

/// A computed value for the `border-spacing` longhand property.
pub type BorderSpacing = GenericBorderSpacing<NonNegativeLength>;

impl BorderImageSideWidth {
    /// Returns `1`.
    #[inline]
    pub fn one() -> Self {
        GenericBorderImageSideWidth::Number(NonNegative(1.))
    }
}

impl BorderImageSlice {
    /// Returns the `100%` value.
    #[inline]
    pub fn hundred_percent() -> Self {
        GenericBorderImageSlice {
            offsets: Rect::all(NonNegativeNumberOrPercentage::hundred_percent()),
            fill: false,
        }
    }
}

impl BorderSpacing {
    /// Returns `0 0`.
    pub fn zero() -> Self {
        GenericBorderSpacing(Size2D::new(
            NonNegativeLength::zero(),
            NonNegativeLength::zero(),
        ))
    }

    /// Returns the horizontal spacing.
    pub fn horizontal(&self) -> Au {
        Au::from(*self.0.width())
    }

    /// Returns the vertical spacing.
    pub fn vertical(&self) -> Au {
        Au::from(*self.0.height())
    }
}
